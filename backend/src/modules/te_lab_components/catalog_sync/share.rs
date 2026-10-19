//! Setting up and recognising a catalog-v2 share: the v1 blocker file and the cutover
//! marker 0.1.x clients check before syncing, the folder layout, and canonical JSON writes.

use std::{fs, path::Path, process};

use serde::Serialize;
use uuid::Uuid;

use super::{
    canonical_json, checksum, signing_bytes, types::CatalogCutoverMarker, CatalogSharedSyncPaths,
    CATALOG_BOOTSTRAP_COMPLETE_KEY, CATALOG_SNAPSHOT_SCHEMA_VERSION, CATALOG_SYNC_SCHEMA_VERSION,
};
use crate::{
    modules::te_lab_components::{
        catalog_model::{
            Part, StockPlacement, StorageArea, StorageContainer, CATALOG_SCHEMA_VERSION,
        },
        store::InventoryDb,
    },
    platform::{
        common::{db_error, now_timestamp, CommandResult},
        sync::{
            sha256_hex, sign_canonical_bytes, verify_canonical_bytes, write_file_atomically,
            CHECKSUM_PREFIX,
        },
    },
};

const LEGACY_STREAM_BLOCKER_PREFIX: &str = "TE Lab Components legacy sync v1 is blocked";

/// Whether the shared catalog stream can be used. A shared root that has never held Lab data
/// is set up by the first client to sync; one that still holds the retired v1 stream (or a
/// damaged marker) stays unavailable, and catalog changes remain local.
pub(super) fn prepare_catalog_share(
    db: &InventoryDb,
    paths: &CatalogSharedSyncPaths,
) -> CommandResult<bool> {
    if catalog_cutover_ready(paths)? {
        return Ok(true);
    }
    if paths.legacy_inventory_path.exists() || paths.cutover_marker_path.exists() {
        return Ok(false);
    }
    initialize_catalog_share(db, paths)?;
    Ok(true)
}

/// Writes the v1 blocker file and the catalog-v2 marker that 0.1.x clients check before
/// syncing, then publishes this client's catalog as the first snapshot.
pub(super) fn initialize_catalog_share(
    db: &InventoryDb,
    paths: &CatalogSharedSyncPaths,
) -> CommandResult<()> {
    let shared_dir = paths
        .catalog_root
        .parent()
        .ok_or_else(|| "Catalog shared path has no parent directory.".to_string())?;
    fs::create_dir_all(shared_dir).map_err(|error| error.to_string())?;
    write_through_temp_file(
        &paths.legacy_inventory_path,
        format!(
            "{LEGACY_STREAM_BLOCKER_PREFIX}. Use shared/catalog-v2 with schema 2.
"
        )
        .as_bytes(),
        false,
    )?;

    ensure_catalog_layout(paths)?;
    let mut marker = CatalogCutoverMarker {
        schema_version: CATALOG_SNAPSHOT_SCHEMA_VERSION,
        sync_schema_version: CATALOG_SYNC_SCHEMA_VERSION,
        local_fingerprint: catalog_fingerprint(db)?,
        source_client_id: db.get_or_create_client_id()?,
        created_at_utc: now_timestamp(),
        checksum: String::new(),
        auth: None,
    };
    marker.checksum = checksum(&marker)?;
    marker.auth = sign_canonical_bytes("catalog.sync.cutover.v2", &signing_bytes(&marker)?)
        .map_err(|error| error.to_string())?;
    write_canonical_json(&paths.cutover_marker_path, &marker, true)?;
    db.put_sync_value(CATALOG_BOOTSTRAP_COMPLETE_KEY, now_timestamp().as_bytes())?;
    db.flush();
    Ok(())
}

pub(super) fn read_cutover_marker(
    paths: &CatalogSharedSyncPaths,
) -> CommandResult<Option<CatalogCutoverMarker>> {
    if !paths.cutover_marker_path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&paths.cutover_marker_path).map_err(|error| error.to_string())?;
    let marker: CatalogCutoverMarker =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if marker.schema_version != CATALOG_SNAPSHOT_SCHEMA_VERSION
        || marker.sync_schema_version != CATALOG_SYNC_SCHEMA_VERSION
    {
        return Err("Lab catalog cutover marker uses an unsupported schema.".to_string());
    }
    if marker.checksum != checksum(&marker)? {
        return Err("Lab catalog cutover marker checksum is invalid.".to_string());
    }
    verify_canonical_bytes(
        "catalog.sync.cutover.v2",
        &signing_bytes(&marker)?,
        marker.auth.as_deref(),
    )?;
    Ok(Some(marker))
}

pub(super) fn catalog_cutover_ready(paths: &CatalogSharedSyncPaths) -> CommandResult<bool> {
    Ok(read_cutover_marker(paths)?.is_some() && ensure_legacy_stream_blocked(paths).is_ok())
}

pub(super) fn ensure_legacy_stream_blocked(paths: &CatalogSharedSyncPaths) -> CommandResult<()> {
    if !paths.legacy_inventory_path.is_file() {
        return Err(
            "Legacy Lab sync stream is not blocked; shared catalog v2 remains read-only."
                .to_string(),
        );
    }
    let text =
        fs::read_to_string(&paths.legacy_inventory_path).map_err(|error| error.to_string())?;
    if !text.starts_with(LEGACY_STREAM_BLOCKER_PREFIX) {
        return Err("Unexpected file occupies the legacy Lab sync path.".to_string());
    }
    Ok(())
}

pub(super) fn catalog_fingerprint(db: &InventoryDb) -> CommandResult<String> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct FingerprintSource {
        schema_version: u32,
        parts: Vec<Part>,
        storage_areas: Vec<StorageArea>,
        storage_containers: Vec<StorageContainer>,
        stock_placements: Vec<StockPlacement>,
    }
    let mut parts = db.load_parts()?;
    parts.sort_by(|left, right| left.entry_uuid.cmp(&right.entry_uuid));
    let mut storage_areas = db.load_storage_areas()?;
    storage_areas.sort_by(|left, right| left.area_uuid.cmp(&right.area_uuid));
    let mut storage_containers = db.load_storage_containers()?;
    storage_containers.sort_by(|left, right| left.container_uuid.cmp(&right.container_uuid));
    let mut stock_placements = db.load_stock_placements()?;
    stock_placements.sort_by(|left, right| left.placement_uuid.cmp(&right.placement_uuid));
    let bytes = canonical_json(&FingerprintSource {
        schema_version: CATALOG_SCHEMA_VERSION,
        parts,
        storage_areas,
        storage_containers,
        stock_placements,
    })?;
    Ok(format!("{CHECKSUM_PREFIX}{}", sha256_hex(&bytes)))
}

pub(super) fn ensure_catalog_layout(paths: &CatalogSharedSyncPaths) -> CommandResult<()> {
    fs::create_dir_all(&paths.ops_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&paths.snapshots_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&paths.locks_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&paths.backups_dir).map_err(|error| error.to_string())?;
    Ok(())
}

/// Writes `value` as canonical JSON through a temporary file, so other computers never read
/// a partial file. An existing file is replaced only when `replace_existing` is set.
pub(super) fn write_canonical_json<T: Serialize>(
    path: &Path,
    value: &T,
    replace_existing: bool,
) -> CommandResult<()> {
    write_through_temp_file(path, &canonical_json(value)?, replace_existing)
}

fn write_through_temp_file(path: &Path, bytes: &[u8], replace_existing: bool) -> CommandResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| "Shared file path has no parent directory.".to_string())?;
    fs::create_dir_all(parent).map_err(db_error)?;
    let temp_path = parent.join(format!(
        ".{}.tmp-{}-{}",
        path.file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_default(),
        process::id(),
        Uuid::new_v4().simple()
    ));
    write_file_atomically(&temp_path, path, bytes, replace_existing).map_err(db_error)
}
