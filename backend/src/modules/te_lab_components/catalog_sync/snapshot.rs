//! Catalog snapshots. A computer with no catalog yet starts from the latest one; snapshots
//! are refreshed after changes but never replace operation files.

use std::{collections::HashMap, fs};

use uuid::Uuid;

use super::{
    checksum, entity_state_key,
    outbox::count_pending_local_operations,
    share::write_canonical_json,
    signing_bytes, sync_watermarks,
    types::{
        CatalogEntityState, CatalogManifest, CatalogSnapshot, CatalogTombstone, CatalogWatermark,
    },
    CatalogSharedSyncPaths, CATALOG_BOOTSTRAP_COMPLETE_KEY, CATALOG_SNAPSHOT_SCHEMA_VERSION,
    CATALOG_SYNC_SCHEMA_VERSION,
};
use crate::{
    modules::te_lab_components::{
        catalog_model::{
            validate_part, validate_stock_placement, validate_storage_area,
            validate_storage_container, CATALOG_SCHEMA_VERSION,
        },
        store::{InventoryDb, SyncKeyspace},
    },
    platform::{
        common::{now_timestamp, CommandResult},
        sync::{sign_canonical_bytes, verify_canonical_bytes},
    },
};

const SNAPSHOT_FILE_SUFFIX: &str = ".snapshot.json";

pub(super) fn publish_snapshot(
    db: &InventoryDb,
    paths: &CatalogSharedSyncPaths,
    force: bool,
) -> CommandResult<bool> {
    if count_pending_local_operations(db, Some(paths))? > 0 {
        return Ok(false);
    }
    if !force && read_manifest(paths)?.is_some() {
        return Ok(false);
    }
    let snapshot = build_snapshot(db)?;
    let snapshot_file = format!("{}{}", snapshot.snapshot_id, SNAPSHOT_FILE_SUFFIX);
    let snapshot_path = paths.snapshots_dir.join(&snapshot_file);
    write_canonical_json(&snapshot_path, &snapshot, false)?;
    let mut manifest = CatalogManifest {
        schema_version: CATALOG_SNAPSHOT_SCHEMA_VERSION,
        sync_schema_version: CATALOG_SYNC_SCHEMA_VERSION,
        snapshot_id: snapshot.snapshot_id.clone(),
        snapshot_file,
        snapshot_checksum: snapshot.checksum.clone(),
        app_version: snapshot.app_version.clone(),
        source_client_id: snapshot.source_client_id.clone(),
        created_at_utc: snapshot.created_at_utc.clone(),
        part_count: snapshot.parts.len(),
        area_count: snapshot.storage_areas.len(),
        container_count: snapshot.storage_containers.len(),
        placement_count: snapshot.stock_placements.len(),
        tombstone_count: snapshot.tombstones.len(),
        watermarks: snapshot.watermarks.clone(),
        checksum: String::new(),
        auth: None,
    };
    manifest.checksum = checksum(&manifest)?;
    manifest.auth = sign_canonical_bytes("catalog.sync.manifest.v2", &signing_bytes(&manifest)?)
        .map_err(|error| error.to_string())?;
    write_canonical_json(&paths.manifest_path, &manifest, true)?;
    db.set_last_snapshot_id(&manifest.snapshot_id)?;
    db.flush();
    Ok(true)
}

pub(super) fn build_snapshot(db: &InventoryDb) -> CommandResult<CatalogSnapshot> {
    let mut parts = db.load_parts()?;
    parts.sort_by(|left, right| left.entry_uuid.cmp(&right.entry_uuid));
    let mut storage_areas = db.load_storage_areas()?;
    storage_areas.sort_by(|left, right| left.area_uuid.cmp(&right.area_uuid));
    let mut storage_containers = db.load_storage_containers()?;
    storage_containers.sort_by(|left, right| left.container_uuid.cmp(&right.container_uuid));
    let mut stock_placements = db.load_stock_placements()?;
    stock_placements.sort_by(|left, right| left.placement_uuid.cmp(&right.placement_uuid));
    let mut tombstones = Vec::new();
    db.scan_sync_tombstones::<CatalogTombstone, _>(usize::MAX, |_, tombstone| {
        tombstones.push(tombstone);
        Ok(true)
    })?;
    tombstones.sort_by(|left, right| {
        left.entity_type
            .as_str()
            .cmp(right.entity_type.as_str())
            .then_with(|| left.entity_uuid.cmp(&right.entity_uuid))
    });
    let mut entity_states = Vec::new();
    db.scan_sync_entry_states::<CatalogEntityState, _>(usize::MAX, |_, state| {
        entity_states.push(state);
        Ok(true)
    })?;
    entity_states.sort_by(|left, right| {
        left.entity_type
            .as_str()
            .cmp(right.entity_type.as_str())
            .then_with(|| left.entity_uuid.cmp(&right.entity_uuid))
    });
    let mut watermarks = sync_watermarks(db)?
        .into_iter()
        .map(|(client_id, local_seq)| CatalogWatermark {
            client_id,
            local_seq,
        })
        .collect::<Vec<_>>();
    watermarks.sort_by(|left, right| left.client_id.cmp(&right.client_id));
    let mut snapshot = CatalogSnapshot {
        schema_version: CATALOG_SNAPSHOT_SCHEMA_VERSION,
        sync_schema_version: CATALOG_SYNC_SCHEMA_VERSION,
        snapshot_id: format!("catalog-snapshot-{}", Uuid::new_v4().simple()),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        source_client_id: db.get_or_create_client_id()?,
        created_at_utc: now_timestamp(),
        parts,
        storage_areas,
        storage_containers,
        stock_placements,
        tombstones,
        entity_states,
        watermarks,
        checksum: String::new(),
        auth: None,
    };
    snapshot.checksum = checksum(&snapshot)?;
    snapshot.auth = sign_canonical_bytes("catalog.sync.snapshot.v2", &signing_bytes(&snapshot)?)
        .map_err(|error| error.to_string())?;
    Ok(snapshot)
}

pub(super) fn apply_latest_snapshot_if_safe(
    db: &InventoryDb,
    paths: &CatalogSharedSyncPaths,
    pending_count: usize,
) -> CommandResult<bool> {
    if pending_count > 0 {
        return Ok(false);
    }
    let Some(manifest) = read_manifest(paths)? else {
        return Ok(false);
    };
    if db.last_snapshot_id()?.as_deref() == Some(manifest.snapshot_id.as_str()) {
        return Ok(false);
    }
    if db.has_catalog_entities()? {
        return Ok(false);
    }
    let snapshot = read_snapshot(paths, &manifest)?;
    validate_snapshot_entities(&snapshot)?;
    let changed = db.replace_catalog_snapshot(
        &snapshot.parts,
        &snapshot.storage_areas,
        &snapshot.storage_containers,
        &snapshot.stock_placements,
    )?;
    db.clear_sync_keyspace(SyncKeyspace::Tombstone)?;
    for tombstone in &snapshot.tombstones {
        db.put_sync_tombstone(
            &entity_state_key(tombstone.entity_type, &tombstone.entity_uuid),
            tombstone,
        )?;
    }
    db.clear_sync_keyspace(SyncKeyspace::EntryState)?;
    for state in &snapshot.entity_states {
        db.put_sync_entry_state(
            &entity_state_key(state.entity_type, &state.entity_uuid),
            state,
        )?;
    }
    db.clear_sync_keyspace(SyncKeyspace::Watermark)?;
    for watermark in &snapshot.watermarks {
        if watermark.local_seq > 0 {
            db.set_sync_watermark(&watermark.client_id, watermark.local_seq)?;
        }
    }
    db.set_schema_version(CATALOG_SCHEMA_VERSION)?;
    db.set_sync_schema_version(CATALOG_SYNC_SCHEMA_VERSION.into())?;
    db.set_last_snapshot_id(&snapshot.snapshot_id)?;
    db.put_sync_value(
        CATALOG_BOOTSTRAP_COMPLETE_KEY,
        snapshot.created_at_utc.as_bytes(),
    )?;
    if changed {
        db.increment_sync_revision()?;
    }
    db.flush();
    Ok(changed)
}

pub(super) fn validate_snapshot_entities(snapshot: &CatalogSnapshot) -> CommandResult<()> {
    let areas = snapshot
        .storage_areas
        .iter()
        .map(|area| (area.area_uuid.as_str(), area))
        .collect::<HashMap<_, _>>();
    let containers = snapshot
        .storage_containers
        .iter()
        .map(|container| (container.container_uuid.as_str(), container))
        .collect::<HashMap<_, _>>();
    let parts = snapshot
        .parts
        .iter()
        .map(|part| (part.entry_uuid.as_str(), part))
        .collect::<HashMap<_, _>>();
    for area in &snapshot.storage_areas {
        validate_storage_area(area)?;
    }
    for container in &snapshot.storage_containers {
        validate_storage_container(container)?;
        if !areas.contains_key(container.area_uuid.as_str()) {
            return Err("Catalog snapshot container references a missing area.".to_string());
        }
    }
    for part in &snapshot.parts {
        validate_part(part)?;
    }
    for placement in &snapshot.stock_placements {
        if !parts.contains_key(placement.part_uuid.as_str()) {
            return Err("Catalog snapshot placement references a missing part.".to_string());
        }
        let container = containers
            .get(placement.container_uuid.as_str())
            .ok_or_else(|| {
                "Catalog snapshot placement references a missing container.".to_string()
            })?;
        validate_stock_placement(placement, container)?;
    }
    Ok(())
}

pub(super) fn read_manifest(
    paths: &CatalogSharedSyncPaths,
) -> CommandResult<Option<CatalogManifest>> {
    if !paths.manifest_path.exists() {
        return Ok(None);
    }
    let bytes = fs::read(&paths.manifest_path).map_err(|error| error.to_string())?;
    let manifest: CatalogManifest =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if manifest.schema_version != CATALOG_SNAPSHOT_SCHEMA_VERSION
        || manifest.sync_schema_version != CATALOG_SYNC_SCHEMA_VERSION
    {
        return Err("Shared Lab catalog manifest uses an unsupported schema.".to_string());
    }
    if manifest.checksum != checksum(&manifest)? {
        return Err("Shared Lab catalog manifest checksum is invalid.".to_string());
    }
    verify_canonical_bytes(
        "catalog.sync.manifest.v2",
        &signing_bytes(&manifest)?,
        manifest.auth.as_deref(),
    )?;
    Ok(Some(manifest))
}

pub(super) fn read_snapshot(
    paths: &CatalogSharedSyncPaths,
    manifest: &CatalogManifest,
) -> CommandResult<CatalogSnapshot> {
    let snapshot_path = paths.snapshots_dir.join(&manifest.snapshot_file);
    let bytes = fs::read(&snapshot_path).map_err(|error| error.to_string())?;
    let snapshot: CatalogSnapshot =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if snapshot.schema_version != CATALOG_SNAPSHOT_SCHEMA_VERSION
        || snapshot.sync_schema_version != CATALOG_SYNC_SCHEMA_VERSION
        || snapshot.snapshot_id != manifest.snapshot_id
    {
        return Err("Shared Lab catalog snapshot identity or schema is invalid.".to_string());
    }
    if snapshot.checksum != manifest.snapshot_checksum || snapshot.checksum != checksum(&snapshot)?
    {
        return Err("Shared Lab catalog snapshot checksum is invalid.".to_string());
    }
    verify_canonical_bytes(
        "catalog.sync.snapshot.v2",
        &signing_bytes(&snapshot)?,
        snapshot.auth.as_deref(),
    )?;
    Ok(snapshot)
}
