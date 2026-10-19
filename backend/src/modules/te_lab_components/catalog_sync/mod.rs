//! Lab Components catalog sync (`shared\catalog-v2\`). Each computer writes its catalog
//! changes as operation files and the newest operation for an entity wins whole; snapshots
//! only bootstrap an empty computer and are never compacted. The file format is the one 0.1.x
//! clients read, so it does not change here. File primitives (canonical JSON, checksums,
//! atomic writes) and status building are shared with the entry engine in `platform::sync`.

mod apply;
mod operation_files;
mod outbox;
mod share;
mod snapshot;
mod types;

#[cfg(test)]
mod tests;

use std::{collections::HashMap, path::PathBuf};

use serde::Serialize;

pub(crate) use self::outbox::{
    changed_fields, queue_delete_operation, queue_upsert_operation, recover_local_sync_state,
};
pub(crate) use self::types::CatalogEntityType;
use self::{
    apply::pull_remote_operations,
    outbox::{count_pending_local_operations, push_pending_local_operations},
    share::{catalog_cutover_ready, ensure_catalog_layout, prepare_catalog_share},
    snapshot::{apply_latest_snapshot_if_safe, publish_snapshot},
};
use crate::{
    modules::te_lab_components::store::InventoryDb,
    platform::{
        common::{db_error, CommandResult, InventorySharedStatus},
        sync::{
            build_shared_status, canonical_json_bytes,
            canonical_json_bytes_without_checksum_or_auth, content_checksum,
            module_shared_root_from_env, record_sync_findings, shared_sync_enabled,
            sync_disabled_status, SyncFindings,
        },
        ModuleId,
    },
};

pub(super) const CATALOG_SYNC_SCHEMA_VERSION: u16 = 2;
pub(super) const CATALOG_SNAPSHOT_SCHEMA_VERSION: u16 = 2;
const SHARED_ROOT_ENV: &str = "INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT";
pub(super) const CATALOG_BOOTSTRAP_COMPLETE_KEY: &str = "meta:catalog_v2_bootstrap_complete";
const CATALOG_SHARE_UNAVAILABLE_MESSAGE: &str =
    "Lab shared catalog is not available yet; catalog changes stay on this computer.";

#[derive(Debug, Clone)]
pub(crate) struct CatalogSharedSyncPaths {
    pub shared_root: PathBuf,
    pub catalog_root: PathBuf,
    pub manifest_path: PathBuf,
    pub ops_dir: PathBuf,
    pub snapshots_dir: PathBuf,
    pub locks_dir: PathBuf,
    pub backups_dir: PathBuf,
    pub cutover_marker_path: PathBuf,
    pub legacy_inventory_path: PathBuf,
}

impl CatalogSharedSyncPaths {
    pub(crate) fn from_shared_root(shared_root: impl Into<PathBuf>) -> Self {
        let shared_root = shared_root.into();
        let shared_dir = shared_root.join("shared");
        let catalog_root = shared_dir.join("catalog-v2");
        Self {
            manifest_path: catalog_root.join("manifest.json"),
            ops_dir: catalog_root.join("ops"),
            snapshots_dir: catalog_root.join("snapshots"),
            locks_dir: catalog_root.join("locks"),
            backups_dir: catalog_root.join("backups"),
            cutover_marker_path: catalog_root.join("cutover.json"),
            legacy_inventory_path: shared_dir.join("inventory"),
            catalog_root,
            shared_root,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct CatalogSharedSyncRunResult {
    pub entries_changed: bool,
    pub shared: InventorySharedStatus,
}

pub(crate) fn resolved_catalog_shared_sync_paths() -> CatalogSharedSyncPaths {
    CatalogSharedSyncPaths::from_shared_root(module_shared_root_from_env(
        SHARED_ROOT_ENV,
        ModuleId::TeLabComponents,
    ))
}

pub(crate) fn startup_catalog_status(message: impl Into<String>) -> InventorySharedStatus {
    let message = message.into();
    if !shared_sync_enabled() {
        return disabled_status(None, message);
    }
    let paths = resolved_catalog_shared_sync_paths();
    let ready = catalog_cutover_ready(&paths).unwrap_or(false);
    InventorySharedStatus {
        available: ready,
        can_modify: true,
        enabled: true,
        has_local_only_changes: None,
        message: if ready {
            message
        } else {
            CATALOG_SHARE_UNAVAILABLE_MESSAGE.to_string()
        },
        mutation_mode: if ready { "shared" } else { "local" }.to_string(),
        revision: None,
        last_snapshot_id: None,
        shared_root_path: Some(paths.shared_root.to_string_lossy().into_owned()),
        warning: None,
    }
}

pub(crate) fn shared_catalog_status(
    db: &InventoryDb,
    message: impl Into<String>,
) -> InventorySharedStatus {
    if !shared_sync_enabled() {
        return disabled_status(Some(db), message);
    }
    let paths = resolved_catalog_shared_sync_paths();
    let ready = catalog_cutover_ready(&paths).unwrap_or(false);
    let pending_count = count_pending_local_operations(db, ready.then_some(&paths)).unwrap_or(0);
    build_shared_status(
        db,
        &paths.shared_root,
        ready,
        pending_count,
        0,
        message.into(),
    )
}

pub(crate) fn queued_local_status(db: &InventoryDb) -> InventorySharedStatus {
    if !shared_sync_enabled() {
        return disabled_status(
            Some(db),
            "Local catalog change saved. Shared sync is disabled; sync is not a backup.",
        );
    }
    let paths = resolved_catalog_shared_sync_paths();
    let ready = catalog_cutover_ready(&paths).unwrap_or(false);
    let pending_count = count_pending_local_operations(db, ready.then_some(&paths)).unwrap_or(1);
    build_shared_status(
        db,
        &paths.shared_root,
        ready,
        pending_count.max(1),
        0,
        if ready {
            "Local catalog change saved and queued for shared sync.".to_string()
        } else {
            "Local catalog change saved; the shared catalog is unavailable.".to_string()
        },
    )
}

pub(crate) fn run_shared_sync(db: &InventoryDb) -> CommandResult<CatalogSharedSyncRunResult> {
    if !shared_sync_enabled() {
        return Ok(CatalogSharedSyncRunResult {
            entries_changed: false,
            shared: disabled_status(
                Some(db),
                "Shared sync is disabled for this process. No shared path was accessed.",
            ),
        });
    }
    run_shared_sync_with_root(
        db,
        module_shared_root_from_env(SHARED_ROOT_ENV, ModuleId::TeLabComponents),
    )
}

pub(crate) fn publish_pending_local_changes(
    db: &InventoryDb,
) -> CommandResult<CatalogSharedSyncRunResult> {
    if !shared_sync_enabled() {
        return Ok(CatalogSharedSyncRunResult {
            entries_changed: false,
            shared: disabled_status(
                Some(db),
                "Catalog change saved locally. Shared sync is disabled; sync is not a backup.",
            ),
        });
    }
    publish_pending_local_changes_with_root(
        db,
        module_shared_root_from_env(SHARED_ROOT_ENV, ModuleId::TeLabComponents),
    )
}

pub(crate) fn run_shared_sync_with_root(
    db: &InventoryDb,
    shared_root: impl Into<PathBuf>,
) -> CommandResult<CatalogSharedSyncRunResult> {
    db.ensure_catalog_schema()?;
    let paths = CatalogSharedSyncPaths::from_shared_root(shared_root);
    if !paths.shared_root.exists() {
        let pending_count = count_pending_local_operations(db, None)?;
        return Ok(CatalogSharedSyncRunResult {
            entries_changed: false,
            shared: build_shared_status(
                db,
                &paths.shared_root,
                false,
                pending_count,
                0,
                "Shared workspace unavailable. Saving catalog changes locally.".to_string(),
            ),
        });
    }
    if !prepare_catalog_share(db, &paths)? {
        let pending_count = count_pending_local_operations(db, None)?;
        return Ok(CatalogSharedSyncRunResult {
            entries_changed: false,
            shared: build_shared_status(
                db,
                &paths.shared_root,
                false,
                pending_count,
                0,
                CATALOG_SHARE_UNAVAILABLE_MESSAGE.to_string(),
            ),
        });
    }

    ensure_catalog_layout(&paths)?;
    let pending_before_snapshot = count_pending_local_operations(db, Some(&paths))?;
    let snapshot_changed = apply_latest_snapshot_if_safe(db, &paths, pending_before_snapshot)?;
    let pushed_count = push_pending_local_operations(db, &paths)?;
    let pull_report = pull_remote_operations(db, &paths)?;
    let snapshot_published = publish_snapshot(
        db,
        &paths,
        pushed_count > 0 || snapshot_changed || pull_report.entries_changed,
    )?;
    let pending_count = count_pending_local_operations(db, Some(&paths))?;
    record_sync_findings(
        db,
        &SyncFindings {
            skipped_files: pull_report.corrupt_count,
            ..SyncFindings::default()
        },
    )?;
    let mut message = if pull_report.corrupt_count > 0 {
        format!(
            "Lab catalog shared sync ready. Ignored {} corrupt remote file(s).",
            pull_report.corrupt_count
        )
    } else {
        "Lab catalog shared sync ready.".to_string()
    };
    if pull_report.quarantined_count > 0 {
        message.push_str(&format!(
            " Quarantined {} orphan or conflicting operation(s).",
            pull_report.quarantined_count
        ));
    }
    if snapshot_published {
        message.push_str(" Snapshot refreshed.");
    }
    Ok(CatalogSharedSyncRunResult {
        entries_changed: snapshot_changed || pull_report.entries_changed,
        shared: build_shared_status(
            db,
            &paths.shared_root,
            true,
            pending_count,
            pull_report.corrupt_count,
            message,
        ),
    })
}

pub(super) fn publish_pending_local_changes_with_root(
    db: &InventoryDb,
    shared_root: impl Into<PathBuf>,
) -> CommandResult<CatalogSharedSyncRunResult> {
    db.ensure_catalog_schema()?;
    let paths = CatalogSharedSyncPaths::from_shared_root(shared_root);
    if !paths.shared_root.exists() || !prepare_catalog_share(db, &paths)? {
        let pending_count = count_pending_local_operations(db, None)?;
        return Ok(CatalogSharedSyncRunResult {
            entries_changed: false,
            shared: build_shared_status(
                db,
                &paths.shared_root,
                false,
                pending_count,
                0,
                "Catalog change saved locally; the shared catalog is unavailable.".to_string(),
            ),
        });
    }
    ensure_catalog_layout(&paths)?;
    let pushed_count = push_pending_local_operations(db, &paths)?;
    let pending_count = count_pending_local_operations(db, Some(&paths))?;
    let message = if pushed_count > 0 {
        format!("Published {pushed_count} Lab catalog change(s) to shared sync.")
    } else {
        "Lab catalog shared operation log is up to date.".to_string()
    };
    Ok(CatalogSharedSyncRunResult {
        entries_changed: false,
        shared: build_shared_status(db, &paths.shared_root, true, pending_count, 0, message),
    })
}

pub(super) fn sync_watermarks(db: &InventoryDb) -> CommandResult<HashMap<String, u64>> {
    let mut watermarks = HashMap::new();
    db.scan_sync_watermarks(usize::MAX, |client_id, local_seq| {
        watermarks.insert(client_id, local_seq);
        Ok(true)
    })?;
    Ok(watermarks)
}

pub(super) fn entity_state_key(entity_type: CatalogEntityType, entity_uuid: &str) -> String {
    format!("{}--{}", entity_type.as_str(), entity_uuid.trim())
}

/// Status while shared sync is switched off for this process.
pub(super) fn disabled_status(
    db: Option<&InventoryDb>,
    message: impl Into<String>,
) -> InventorySharedStatus {
    let pending_count = db.map(|db| count_pending_local_operations(db, None).unwrap_or(0));
    sync_disabled_status(db, pending_count, message)
}

/// The `checksum` a catalog file stores for its content.
pub(super) fn checksum<T: Serialize>(value: &T) -> CommandResult<String> {
    content_checksum(value).map_err(db_error)
}

/// The bytes a catalog file's `auth` signature covers.
pub(super) fn signing_bytes<T: Serialize>(value: &T) -> CommandResult<Vec<u8>> {
    canonical_json_bytes_without_checksum_or_auth(value).map_err(db_error)
}

/// Catalog files are written as canonical JSON.
pub(super) fn canonical_json<T: Serialize>(value: &T) -> CommandResult<Vec<u8>> {
    canonical_json_bytes(value).map_err(db_error)
}
