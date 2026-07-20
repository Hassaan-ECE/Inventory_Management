use std::{env, ffi::OsString, fs, path::PathBuf};

use crate::{model::InventorySharedStatus, store::InventoryDb};

use super::{
    queue::count_pending_local_operations, SharedSyncPaths, SyncCoreResult, DEFAULT_SHARED_ROOT,
    SHARED_ROOT_ENV, SHARED_SYNC_ENABLED_ENV,
};

const SHARED_SYNC_DISABLED_MESSAGE: &str =
    "Shared sync is disabled for this process. Changes stay on this computer; sync is not a backup.";

pub(crate) fn shared_sync_enabled() -> bool {
    shared_sync_enabled_from_env_value(env::var_os(SHARED_SYNC_ENABLED_ENV))
}

/// Production default is ON (ME/TE Parts family). Explicitly set
/// `INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED` to `0` / `false` / `no` / `off` to opt out.
fn shared_sync_enabled_from_env_value(value: Option<OsString>) -> bool {
    match value {
        None => true,
        Some(value) => {
            let normalized = value.to_string_lossy().trim().to_ascii_lowercase();
            !matches!(normalized.as_str(), "0" | "false" | "no" | "off")
        }
    }
}

pub(crate) fn resolve_shared_root() -> PathBuf {
    resolve_shared_root_from_env_value(env::var_os(SHARED_ROOT_ENV))
}

pub(crate) fn resolved_shared_sync_paths() -> SharedSyncPaths {
    SharedSyncPaths::from_shared_root(resolve_shared_root())
}

pub(crate) fn resolve_shared_root_from_env_value(value: Option<OsString>) -> PathBuf {
    value
        .and_then(|value| {
            let path = value.to_string_lossy().trim().to_string();
            (!path.is_empty()).then_some(PathBuf::from(path))
        })
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SHARED_ROOT))
}

pub(crate) fn ensure_operation_log_layout(paths: &SharedSyncPaths) -> SyncCoreResult<()> {
    fs::create_dir_all(&paths.ops_dir)?;
    fs::create_dir_all(&paths.snapshots_dir)?;
    fs::create_dir_all(&paths.locks_dir)?;
    fs::create_dir_all(&paths.backups_dir)?;
    Ok(())
}

pub(crate) fn shared_inventory_status(
    db: &InventoryDb,
    message: impl Into<String>,
) -> InventorySharedStatus {
    if !shared_sync_enabled() {
        return disabled_shared_status(Some(db), SHARED_SYNC_DISABLED_MESSAGE);
    }
    let paths = resolved_shared_sync_paths();
    let available = paths.shared_root.exists();
    let pending_count =
        count_pending_local_operations(db, available.then_some(&paths)).unwrap_or(0);
    build_shared_status(db, &paths, available, pending_count, 0, message.into())
}

pub(crate) fn startup_inventory_status(message: impl Into<String>) -> InventorySharedStatus {
    if !shared_sync_enabled() {
        return disabled_shared_status(None, SHARED_SYNC_DISABLED_MESSAGE);
    }
    let paths = resolved_shared_sync_paths();
    let available = paths.shared_root.exists();
    InventorySharedStatus {
        available,
        can_modify: true,
        enabled: true,
        has_local_only_changes: None,
        message: message.into(),
        mutation_mode: if available { "shared" } else { "local" }.to_string(),
        revision: None,
        last_snapshot_id: None,
        shared_root_path: Some(paths.shared_root.to_string_lossy().into_owned()),
    }
}

pub(crate) fn queued_local_status(db: &InventoryDb) -> InventorySharedStatus {
    if !shared_sync_enabled() {
        return disabled_shared_status(Some(db), SHARED_SYNC_DISABLED_MESSAGE);
    }
    let paths = resolved_shared_sync_paths();
    let available = paths.shared_root.exists();
    let pending_count =
        count_pending_local_operations(db, available.then_some(&paths)).unwrap_or(1);
    build_shared_status(
        db,
        &paths,
        available,
        pending_count.max(1),
        0,
        "Local change saved and queued for shared sync.".to_string(),
    )
}

pub(crate) fn disabled_shared_status(
    db: Option<&InventoryDb>,
    message: impl Into<String>,
) -> InventorySharedStatus {
    InventorySharedStatus {
        available: false,
        can_modify: true,
        enabled: false,
        has_local_only_changes: db
            .map(|db| count_pending_local_operations(db, None).unwrap_or(0) > 0),
        message: message.into(),
        mutation_mode: "local".to_string(),
        revision: db.and_then(|db| db.sync_revision().ok().map(|revision| revision.to_string())),
        last_snapshot_id: db.and_then(|db| db.last_snapshot_id().ok().flatten()),
        shared_root_path: None,
    }
}

pub(super) fn build_shared_status(
    db: &InventoryDb,
    paths: &SharedSyncPaths,
    available: bool,
    pending_count: usize,
    corrupt_count: usize,
    message: String,
) -> InventorySharedStatus {
    let has_pending = pending_count > 0;
    let mut message = message;
    if has_pending && !message.contains("Pending local") {
        message = format!("{message} Pending local changes: {pending_count}.");
    }
    if corrupt_count > 0 && !message.contains("corrupt") {
        message = format!("{message} Corrupt remote files ignored: {corrupt_count}.");
    }

    InventorySharedStatus {
        available,
        can_modify: true,
        enabled: true,
        has_local_only_changes: Some(has_pending),
        message,
        mutation_mode: if available && !has_pending {
            "shared".to_string()
        } else {
            "local".to_string()
        },
        revision: db.sync_revision().ok().map(|revision| revision.to_string()),
        last_snapshot_id: db.last_snapshot_id().ok().flatten(),
        shared_root_path: Some(paths.shared_root.to_string_lossy().into_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::shared_sync_enabled_from_env_value;
    use std::ffi::OsString;

    #[test]
    fn shared_sync_defaults_on_and_allows_explicit_opt_out() {
        assert!(shared_sync_enabled_from_env_value(None));
        assert!(shared_sync_enabled_from_env_value(Some(OsString::from(""))));
        for value in ["1", " true ", "YES", "on", "anything"] {
            assert!(shared_sync_enabled_from_env_value(Some(OsString::from(
                value
            ))));
        }
        for value in ["0", "false", " no ", "OFF"] {
            assert!(!shared_sync_enabled_from_env_value(Some(OsString::from(
                value
            ))));
        }
    }
}
