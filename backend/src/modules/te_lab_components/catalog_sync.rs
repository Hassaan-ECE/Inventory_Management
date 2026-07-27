use std::{
    collections::HashMap,
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process,
};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::{
    modules::te_lab_components::{
        catalog_migration::ensure_catalog_initialized,
        catalog_model::{
            validate_part, validate_stock_placement, validate_storage_area,
            validate_storage_container, Part, StockPlacement, StorageArea, StorageContainer,
            CATALOG_SCHEMA_VERSION,
        },
        model::{now_timestamp, CommandResult, InventorySharedStatus},
        store::{InventoryDb, SyncKeyspace},
        sync::{sign_canonical_bytes, verify_canonical_bytes},
    },
    platform::{default_shared_root, ModuleId},
};

const CATALOG_SYNC_SCHEMA_VERSION: u16 = 2;
const CATALOG_SNAPSHOT_SCHEMA_VERSION: u16 = 2;
const SHARED_ROOT_ENV: &str = "INVENTORY_MANAGEMENT_LAB_COMPONENTS_SHARED_ROOT";
const SHARED_SYNC_ENABLED_ENV: &str = "INVENTORY_MANAGEMENT_SHARED_SYNC_ENABLED";
const OP_FILE_SUFFIX: &str = ".op.json";
const SNAPSHOT_FILE_SUFFIX: &str = ".snapshot.json";
const LOCAL_SEQ_WIDTH: usize = 12;
const CHECKSUM_PREFIX: &str = "sha256:";
const CATALOG_BOOTSTRAP_COMPLETE_KEY: &str = "meta:catalog_v2_bootstrap_complete";
const LEGACY_STREAM_BLOCKER_PREFIX: &str = "TE Lab Components legacy sync v1 is blocked";

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CatalogEntityType {
    Part,
    StorageArea,
    StorageContainer,
    StockPlacement,
}

impl CatalogEntityType {
    fn as_str(self) -> &'static str {
        match self {
            Self::Part => "part",
            Self::StorageArea => "storage_area",
            Self::StorageContainer => "storage_container",
            Self::StockPlacement => "stock_placement",
        }
    }

    fn upsert_rank(self) -> u8 {
        match self {
            Self::StorageArea => 0,
            Self::StorageContainer => 1,
            Self::Part => 2,
            Self::StockPlacement => 3,
        }
    }

    fn delete_rank(self) -> u8 {
        match self {
            Self::StockPlacement => 4,
            Self::Part => 5,
            Self::StorageContainer => 6,
            Self::StorageArea => 7,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)]
pub(crate) enum CatalogEntity {
    Part(Part),
    StorageArea(StorageArea),
    StorageContainer(StorageContainer),
    StockPlacement(StockPlacement),
}

impl CatalogEntity {
    pub(crate) fn entity_type(&self) -> CatalogEntityType {
        match self {
            Self::Part(_) => CatalogEntityType::Part,
            Self::StorageArea(_) => CatalogEntityType::StorageArea,
            Self::StorageContainer(_) => CatalogEntityType::StorageContainer,
            Self::StockPlacement(_) => CatalogEntityType::StockPlacement,
        }
    }

    pub(crate) fn entity_id(&self) -> &str {
        match self {
            Self::Part(part) => &part.entry_uuid,
            Self::StorageArea(area) => &area.area_uuid,
            Self::StorageContainer(container) => &container.container_uuid,
            Self::StockPlacement(placement) => &placement.placement_uuid,
        }
    }

    pub(crate) fn updated_at(&self) -> &str {
        match self {
            Self::Part(part) => &part.updated_at,
            Self::StorageArea(area) => &area.updated_at,
            Self::StorageContainer(container) => &container.updated_at,
            Self::StockPlacement(placement) => &placement.updated_at,
        }
    }
}

impl From<Part> for CatalogEntity {
    fn from(value: Part) -> Self {
        Self::Part(value)
    }
}

impl From<StorageArea> for CatalogEntity {
    fn from(value: StorageArea) -> Self {
        Self::StorageArea(value)
    }
}

impl From<StorageContainer> for CatalogEntity {
    fn from(value: StorageContainer) -> Self {
        Self::StorageContainer(value)
    }
}

impl From<StockPlacement> for CatalogEntity {
    fn from(value: StockPlacement) -> Self {
        Self::StockPlacement(value)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum CatalogOperationType {
    #[serde(rename = "catalog.entity.upsert")]
    Upsert,
    #[serde(rename = "catalog.entity.delete")]
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogOperationPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity: Option<CatalogEntity>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_fields: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deleted_at_utc: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogOperationEnvelope {
    pub schema_version: u16,
    pub op_id: String,
    pub client_id: String,
    pub device_id: String,
    pub local_seq: u64,
    pub app_version: String,
    pub created_at_utc: String,
    #[serde(rename = "type")]
    pub operation_type: CatalogOperationType,
    pub entity_type: CatalogEntityType,
    pub entity_id: String,
    pub base_version: Option<String>,
    pub mutation_ts_utc: String,
    pub payload: CatalogOperationPayload,
    #[serde(default)]
    pub checksum: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogAppliedMarker {
    op_id: String,
    client_id: String,
    local_seq: u64,
    checksum: String,
    applied_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogEntityState {
    entity_type: CatalogEntityType,
    entity_uuid: String,
    last_op_id: String,
    mutation_ts_utc: String,
    deleted: bool,
    base_version: Option<String>,
    #[serde(default)]
    changed_fields: Vec<String>,
    source_client_id: String,
    source_local_seq: u64,
    updated_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogTombstone {
    entity_type: CatalogEntityType,
    entity_uuid: String,
    deleted_at_utc: String,
    op_id: String,
    client_id: String,
    local_seq: u64,
    base_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogConflictRecord {
    conflict_id: String,
    entity_type: CatalogEntityType,
    entity_uuid: String,
    incoming_op_id: String,
    incoming_client_id: String,
    incoming_local_seq: u64,
    current_op_id: Option<String>,
    reason: String,
    detected_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogCorruptRemoteRecord {
    record_id: String,
    path: String,
    detail: String,
    detected_at_utc: String,
    content_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogWatermark {
    client_id: String,
    local_seq: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogSnapshot {
    schema_version: u16,
    sync_schema_version: u16,
    snapshot_id: String,
    app_version: String,
    source_client_id: String,
    created_at_utc: String,
    parts: Vec<Part>,
    storage_areas: Vec<StorageArea>,
    storage_containers: Vec<StorageContainer>,
    stock_placements: Vec<StockPlacement>,
    tombstones: Vec<CatalogTombstone>,
    entity_states: Vec<CatalogEntityState>,
    watermarks: Vec<CatalogWatermark>,
    checksum: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    auth: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogManifest {
    schema_version: u16,
    sync_schema_version: u16,
    snapshot_id: String,
    snapshot_file: String,
    snapshot_checksum: String,
    app_version: String,
    source_client_id: String,
    created_at_utc: String,
    part_count: usize,
    area_count: usize,
    container_count: usize,
    placement_count: usize,
    tombstone_count: usize,
    watermarks: Vec<CatalogWatermark>,
    checksum: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    auth: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogCutoverMarker {
    schema_version: u16,
    sync_schema_version: u16,
    local_fingerprint: String,
    source_client_id: String,
    created_at_utc: String,
    checksum: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    auth: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogSharedCutoverPreview {
    pub local_fingerprint: String,
    pub shared_root_path: String,
    pub shared_root_available: bool,
    pub catalog_v2_initialized: bool,
    pub legacy_stream_state: String,
    pub part_count: usize,
    pub area_count: usize,
    pub container_count: usize,
    pub placement_count: usize,
    pub warnings: Vec<String>,
    pub blocking: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct CatalogSharedCutoverCommitInput {
    pub local_fingerprint: String,
    pub confirmed: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CatalogSharedCutoverCommitResult {
    pub local_fingerprint: String,
    pub legacy_backup_path: Option<String>,
    pub catalog_root_path: String,
    pub noop: bool,
    pub message: String,
}

#[derive(Debug, Clone)]
pub(crate) struct CatalogSharedSyncRunResult {
    pub entries_changed: bool,
    pub shared: InventorySharedStatus,
}

pub(crate) fn resolved_catalog_shared_sync_paths() -> CatalogSharedSyncPaths {
    CatalogSharedSyncPaths::from_shared_root(resolve_shared_root())
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
            "Lab catalog shared cutover is not active; changes remain local until reviewed cutover."
                .to_string()
        },
        mutation_mode: if ready { "shared" } else { "local" }.to_string(),
        revision: None,
        last_snapshot_id: None,
        shared_root_path: Some(paths.shared_root.to_string_lossy().into_owned()),
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
    build_status(db, &paths, ready, pending_count, 0, message.into())
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
    build_status(
        db,
        &paths,
        ready,
        pending_count.max(1),
        0,
        if ready {
            "Local catalog change saved and queued for shared sync.".to_string()
        } else {
            "Local catalog change saved; shared catalog cutover is still required.".to_string()
        },
    )
}

pub(crate) fn preview_shared_cutover(
    db: &InventoryDb,
) -> CommandResult<CatalogSharedCutoverPreview> {
    preview_shared_cutover_with_root(db, resolve_shared_root())
}

pub(crate) fn commit_shared_cutover(
    input: CatalogSharedCutoverCommitInput,
    db: &InventoryDb,
) -> CommandResult<CatalogSharedCutoverCommitResult> {
    commit_shared_cutover_with_root(input, db, resolve_shared_root())
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
    run_shared_sync_with_root(db, resolve_shared_root())
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
    publish_pending_local_changes_with_root(db, resolve_shared_root())
}

pub(crate) fn recover_local_sync_state(db: &InventoryDb) -> CommandResult<()> {
    if db.schema_version()? != Some(CATALOG_SCHEMA_VERSION) {
        return Ok(());
    }
    db.set_sync_schema_version(CATALOG_SYNC_SCHEMA_VERSION.into())?;
    let mut max_local_seq = 0u64;
    // Skip non-catalog / corrupt outbox rows so recovery never blocks startup.
    db.scan_sync_outbox_raw(None, usize::MAX, |local_seq, value| {
        max_local_seq = max_local_seq.max(local_seq);
        match serde_json::from_slice::<CatalogOperationEnvelope>(value) {
            Ok(op) => {
                if let Err(error) = validate_operation(&op) {
                    eprintln!(
                        "Lab catalog sync recovery: skipping outbox seq {local_seq} (invalid catalog op): {error}"
                    );
                }
            }
            Err(error) => {
                eprintln!(
                    "Lab catalog sync recovery: skipping outbox seq {local_seq} (unrecognized op): {error}"
                );
            }
        }
        Ok(true)
    })?;
    if db.next_local_seq()? <= max_local_seq {
        db.set_next_local_seq(max_local_seq + 1)?;
    }
    db.flush();
    Ok(())
}

pub(crate) fn changed_fields<T: Serialize>(before: &T, after: &T) -> Vec<String> {
    let Ok(Value::Object(before)) = serde_json::to_value(before) else {
        return Vec::new();
    };
    let Ok(Value::Object(after)) = serde_json::to_value(after) else {
        return Vec::new();
    };
    let mut keys = before
        .keys()
        .chain(after.keys())
        .filter(|key| key.as_str() != "updatedAt")
        .cloned()
        .collect::<Vec<_>>();
    keys.sort();
    keys.dedup();
    keys.into_iter()
        .filter(|key| before.get(key) != after.get(key))
        .collect()
}

pub(crate) fn queue_upsert_operation(
    db: &InventoryDb,
    entity: CatalogEntity,
    changed_fields: Vec<String>,
    base_version: Option<String>,
) -> CommandResult<CatalogOperationEnvelope> {
    ensure_catalog_initialized(db)?;
    validate_entity(db, &entity)?;
    let entity_type = entity.entity_type();
    let entity_id = entity.entity_id().to_string();
    let mutation_ts_utc = nonempty_timestamp(entity.updated_at());
    let operation = build_operation(
        db,
        CatalogOperationType::Upsert,
        entity_type,
        entity_id,
        base_version,
        mutation_ts_utc,
        CatalogOperationPayload {
            entity: Some(entity),
            changed_fields,
            deleted_at_utc: None,
        },
    )?;
    persist_local_operation(db, &operation)?;
    Ok(operation)
}

pub(crate) fn queue_delete_operation(
    db: &InventoryDb,
    entity_type: CatalogEntityType,
    entity_uuid: impl Into<String>,
    deleted_at_utc: impl Into<String>,
    base_version: Option<String>,
) -> CommandResult<CatalogOperationEnvelope> {
    ensure_catalog_initialized(db)?;
    let entity_id = entity_uuid.into();
    let deleted_at_utc = nonempty_timestamp(&deleted_at_utc.into());
    let operation = build_operation(
        db,
        CatalogOperationType::Delete,
        entity_type,
        entity_id,
        base_version,
        deleted_at_utc.clone(),
        CatalogOperationPayload {
            entity: None,
            changed_fields: Vec::new(),
            deleted_at_utc: Some(deleted_at_utc),
        },
    )?;
    persist_local_operation(db, &operation)?;
    Ok(operation)
}

fn build_operation(
    db: &InventoryDb,
    operation_type: CatalogOperationType,
    entity_type: CatalogEntityType,
    entity_id: String,
    base_version: Option<String>,
    mutation_ts_utc: String,
    payload: CatalogOperationPayload,
) -> CommandResult<CatalogOperationEnvelope> {
    if entity_id.trim().is_empty() {
        return Err("Catalog sync entity identity cannot be empty.".to_string());
    }
    db.set_sync_schema_version(CATALOG_SYNC_SCHEMA_VERSION.into())?;
    let local_seq = db.reserve_next_local_seq()?;
    let mut operation = CatalogOperationEnvelope {
        schema_version: CATALOG_SYNC_SCHEMA_VERSION,
        op_id: Uuid::new_v4().simple().to_string(),
        client_id: db.get_or_create_client_id()?,
        device_id: db.get_or_create_device_id()?,
        local_seq,
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        created_at_utc: now_timestamp(),
        operation_type,
        entity_type,
        entity_id,
        base_version,
        mutation_ts_utc,
        payload,
        checksum: String::new(),
        auth: None,
    };
    operation.checksum = canonical_checksum(&operation)?;
    operation.auth = sign_canonical_bytes(
        "catalog.sync.operation.v2",
        &canonical_bytes_without_checksum_or_auth(&operation)?,
    )
    .map_err(|error| error.to_string())?;
    validate_operation(&operation)?;
    Ok(operation)
}

fn persist_local_operation(
    db: &InventoryDb,
    operation: &CatalogOperationEnvelope,
) -> CommandResult<()> {
    db.put_sync_outbox_record(operation.local_seq, operation)?;
    mark_operation_applied(db, operation)?;
    record_entity_state(db, operation)?;
    if operation.operation_type == CatalogOperationType::Delete {
        let tombstone = CatalogTombstone {
            entity_type: operation.entity_type,
            entity_uuid: operation.entity_id.clone(),
            deleted_at_utc: operation
                .payload
                .deleted_at_utc
                .clone()
                .unwrap_or_else(|| operation.mutation_ts_utc.clone()),
            op_id: operation.op_id.clone(),
            client_id: operation.client_id.clone(),
            local_seq: operation.local_seq,
            base_version: operation.base_version.clone(),
        };
        db.put_sync_tombstone(
            &entity_state_key(operation.entity_type, &operation.entity_id),
            &tombstone,
        )?;
    } else {
        let key = entity_state_key(operation.entity_type, &operation.entity_id);
        if db.has_sync_tombstone(&key)? {
            db.delete_sync_tombstone(&key)?;
        }
    }
    db.increment_sync_revision()?;
    db.flush();
    Ok(())
}

fn preview_shared_cutover_with_root(
    db: &InventoryDb,
    shared_root: impl Into<PathBuf>,
) -> CommandResult<CatalogSharedCutoverPreview> {
    ensure_catalog_initialized(db)?;
    let paths = CatalogSharedSyncPaths::from_shared_root(shared_root);
    let local_fingerprint = catalog_fingerprint(db)?;
    let parts = db.load_parts()?;
    let areas = db.load_storage_areas()?;
    let containers = db.load_storage_containers()?;
    let placements = db.load_stock_placements()?;
    let shared_root_available = paths.shared_root.exists();
    let catalog_v2_initialized = read_cutover_marker(&paths)?.is_some();
    let legacy_stream_state = legacy_stream_state(&paths)?;
    let mut warnings = vec![
        "Stop every Lab Components writer before committing shared cutover.".to_string(),
        "The legacy shared/inventory directory will be retained as a dated backup and replaced by a blocker file."
            .to_string(),
        "Shared sync is not a backup; retain the workbook, local database, and shared-root copies."
            .to_string(),
    ];
    if !placements.is_empty() {
        warnings.push(
            "Verify placement quantities and units before designating this client as the first v2 snapshot source."
                .to_string(),
        );
    }
    let blocking = !shared_root_available || legacy_stream_state == "unexpected_file";
    Ok(CatalogSharedCutoverPreview {
        local_fingerprint,
        shared_root_path: paths.shared_root.to_string_lossy().into_owned(),
        shared_root_available,
        catalog_v2_initialized,
        legacy_stream_state,
        part_count: parts.len(),
        area_count: areas.len(),
        container_count: containers.len(),
        placement_count: placements.len(),
        warnings,
        blocking,
    })
}

fn commit_shared_cutover_with_root(
    input: CatalogSharedCutoverCommitInput,
    db: &InventoryDb,
    shared_root: impl Into<PathBuf>,
) -> CommandResult<CatalogSharedCutoverCommitResult> {
    if !input.confirmed {
        return Err(
            "Confirm the reviewed Lab catalog shared cutover before committing.".to_string(),
        );
    }
    let paths = CatalogSharedSyncPaths::from_shared_root(shared_root);
    let preview = preview_shared_cutover_with_root(db, paths.shared_root.clone())?;
    if preview.blocking {
        return Err(
            "Shared cutover is blocked. Review the cutover preview and shared root.".to_string(),
        );
    }
    if preview.local_fingerprint != input.local_fingerprint {
        return Err(
            "The local catalog changed after cutover preview. Run the preview again.".to_string(),
        );
    }

    if let Some(marker) = read_cutover_marker(&paths)? {
        ensure_legacy_stream_blocked(&paths)?;
        return Ok(CatalogSharedCutoverCommitResult {
            local_fingerprint: marker.local_fingerprint,
            legacy_backup_path: None,
            catalog_root_path: paths.catalog_root.to_string_lossy().into_owned(),
            noop: true,
            message: "Lab catalog shared cutover was already committed.".to_string(),
        });
    }

    let shared_dir = paths
        .catalog_root
        .parent()
        .ok_or_else(|| "Catalog shared path has no parent directory.".to_string())?;
    fs::create_dir_all(shared_dir).map_err(|error| error.to_string())?;
    let mut legacy_backup_path = None;
    let mut blocker_created = false;
    let write_result = (|| -> CommandResult<()> {
        if paths.legacy_inventory_path.is_dir() {
            let backup_path = shared_dir.join(format!(
                "legacy-inventory-v1-backup-{}",
                Uuid::new_v4().simple()
            ));
            fs::rename(&paths.legacy_inventory_path, &backup_path)
                .map_err(|error| format!("Could not quarantine legacy Lab sync: {error}"))?;
            legacy_backup_path = Some(backup_path);
        } else if paths.legacy_inventory_path.is_file() {
            ensure_legacy_stream_blocked(&paths)?;
        }

        if !paths.legacy_inventory_path.exists() {
            write_new_file(
                &paths.legacy_inventory_path,
                format!("{LEGACY_STREAM_BLOCKER_PREFIX}. Use shared/catalog-v2 with schema 2.\n")
                    .as_bytes(),
            )?;
            blocker_created = true;
        }

        ensure_catalog_layout(&paths)?;
        let mut marker = CatalogCutoverMarker {
            schema_version: CATALOG_SNAPSHOT_SCHEMA_VERSION,
            sync_schema_version: CATALOG_SYNC_SCHEMA_VERSION,
            local_fingerprint: preview.local_fingerprint.clone(),
            source_client_id: db.get_or_create_client_id()?,
            created_at_utc: now_timestamp(),
            checksum: String::new(),
            auth: None,
        };
        marker.checksum = canonical_checksum(&marker)?;
        marker.auth = sign_canonical_bytes(
            "catalog.sync.cutover.v2",
            &canonical_bytes_without_checksum_or_auth(&marker)?,
        )
        .map_err(|error| error.to_string())?;
        write_json_atomic(&paths.cutover_marker_path, &marker, false)?;
        push_pending_local_operations(db, &paths)?;
        publish_snapshot(db, &paths, true)?;
        db.put_sync_value(CATALOG_BOOTSTRAP_COMPLETE_KEY, now_timestamp().as_bytes())?;
        db.flush();
        Ok(())
    })();

    if let Err(error) = write_result {
        let _ = fs::remove_file(&paths.cutover_marker_path);
        if blocker_created {
            let _ = fs::remove_file(&paths.legacy_inventory_path);
        }
        if let Some(backup_path) = &legacy_backup_path {
            if !paths.legacy_inventory_path.exists() {
                let _ = fs::rename(backup_path, &paths.legacy_inventory_path);
            }
        }
        return Err(format!("Lab catalog shared cutover rolled back: {error}"));
    }

    Ok(CatalogSharedCutoverCommitResult {
        local_fingerprint: preview.local_fingerprint,
        legacy_backup_path: legacy_backup_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned()),
        catalog_root_path: paths.catalog_root.to_string_lossy().into_owned(),
        noop: false,
        message: "Lab catalog shared sync v2 cutover completed; the legacy v1 stream is blocked."
            .to_string(),
    })
}

fn run_shared_sync_with_root(
    db: &InventoryDb,
    shared_root: impl Into<PathBuf>,
) -> CommandResult<CatalogSharedSyncRunResult> {
    ensure_catalog_initialized(db)?;
    let paths = CatalogSharedSyncPaths::from_shared_root(shared_root);
    if !paths.shared_root.exists() {
        let pending_count = count_pending_local_operations(db, None)?;
        return Ok(CatalogSharedSyncRunResult {
            entries_changed: false,
            shared: build_status(
                db,
                &paths,
                false,
                pending_count,
                0,
                "Shared workspace unavailable. Saving catalog changes locally.".to_string(),
            ),
        });
    }
    if !catalog_cutover_ready(&paths)? {
        let pending_count = count_pending_local_operations(db, None)?;
        return Ok(CatalogSharedSyncRunResult {
            entries_changed: false,
            shared: build_status(
                db,
                &paths,
                false,
                pending_count,
                0,
                "Shared catalog cutover is required before v2 synchronization.".to_string(),
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
        shared: build_status(
            db,
            &paths,
            true,
            pending_count,
            pull_report.corrupt_count,
            message,
        ),
    })
}

fn publish_pending_local_changes_with_root(
    db: &InventoryDb,
    shared_root: impl Into<PathBuf>,
) -> CommandResult<CatalogSharedSyncRunResult> {
    ensure_catalog_initialized(db)?;
    let paths = CatalogSharedSyncPaths::from_shared_root(shared_root);
    if !paths.shared_root.exists() || !catalog_cutover_ready(&paths)? {
        let pending_count = count_pending_local_operations(db, None)?;
        return Ok(CatalogSharedSyncRunResult {
            entries_changed: false,
            shared: build_status(
                db,
                &paths,
                false,
                pending_count,
                0,
                "Catalog change saved locally; shared v2 cutover is unavailable.".to_string(),
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
        shared: build_status(db, &paths, true, pending_count, 0, message),
    })
}

#[derive(Debug, Clone, Copy, Default)]
struct PullReport {
    entries_changed: bool,
    corrupt_count: usize,
    quarantined_count: usize,
}

fn pull_remote_operations(
    db: &InventoryDb,
    paths: &CatalogSharedSyncPaths,
) -> CommandResult<PullReport> {
    let watermarks = sync_watermarks(db)?;
    let (operations, corrupt_count) = scan_operation_files(db, paths, &watermarks)?;
    let mut unseen = Vec::new();
    let mut max_watermarks = watermarks;
    for operation in operations {
        max_watermarks
            .entry(operation.client_id.clone())
            .and_modify(|value| *value = (*value).max(operation.local_seq))
            .or_insert(operation.local_seq);
        if !db.has_sync_applied_marker(&operation.op_id)? {
            unseen.push(operation);
        }
    }
    if unseen.is_empty() {
        for (client_id, local_seq) in max_watermarks {
            if local_seq > 0 {
                db.set_sync_watermark(&client_id, local_seq)?;
            }
        }
        return Ok(PullReport {
            corrupt_count,
            ..PullReport::default()
        });
    }

    let mut grouped = HashMap::<String, Vec<CatalogOperationEnvelope>>::new();
    for operation in &unseen {
        grouped
            .entry(entity_state_key(
                operation.entity_type,
                &operation.entity_id,
            ))
            .or_default()
            .push(operation.clone());
    }

    let mut candidates = Vec::new();
    let mut quarantined_count = 0usize;
    for mut group in grouped.into_values() {
        group.sort_by(compare_operations);
        let winner = group.pop().expect("group is nonempty");
        for superseded in group {
            record_conflict(db, &superseded, None, "superseded_remote_operation")?;
            quarantined_count += 1;
        }
        candidates.push(winner);
    }
    candidates.sort_by(|left, right| {
        operation_apply_rank(left)
            .cmp(&operation_apply_rank(right))
            .then_with(|| compare_operations(left, right))
    });

    let mut entries_changed = false;
    for operation in &candidates {
        match apply_remote_candidate(db, operation) {
            Ok(changed) => entries_changed |= changed,
            Err(error) => {
                let current =
                    current_entity_state(db, operation.entity_type, &operation.entity_id)?;
                record_conflict(
                    db,
                    operation,
                    current.as_ref(),
                    &format!("quarantined: {error}"),
                )?;
                quarantined_count += 1;
            }
        }
    }

    for operation in &unseen {
        mark_operation_applied(db, operation)?;
    }
    for (client_id, local_seq) in max_watermarks {
        if local_seq > 0 {
            db.set_sync_watermark(&client_id, local_seq)?;
        }
    }
    if entries_changed || !unseen.is_empty() {
        db.increment_sync_revision()?;
    }
    db.flush();
    Ok(PullReport {
        entries_changed,
        corrupt_count,
        quarantined_count,
    })
}

fn apply_remote_candidate(
    db: &InventoryDb,
    operation: &CatalogOperationEnvelope,
) -> CommandResult<bool> {
    validate_operation(operation)?;
    let current_state = current_entity_state(db, operation.entity_type, &operation.entity_id)?;
    if let Some(current_state) = &current_state {
        if !operation_wins_state(operation, current_state) {
            record_conflict(
                db,
                operation,
                Some(current_state),
                "stale_incoming_operation",
            )?;
            return Ok(false);
        }
    } else if let Some(current_timestamp) =
        current_entity_timestamp(db, operation.entity_type, &operation.entity_id)?
    {
        if operation.mutation_ts_utc < current_timestamp {
            record_conflict(db, operation, None, "older_than_local_entity")?;
            return Ok(false);
        }
    }

    if let (Some(base_version), Some(current_timestamp)) = (
        operation.base_version.as_deref(),
        current_entity_timestamp(db, operation.entity_type, &operation.entity_id)?,
    ) {
        if base_version != current_timestamp {
            record_conflict(
                db,
                operation,
                current_state.as_ref(),
                "base_version_mismatch_last_writer_wins",
            )?;
        }
    }

    let changed = match operation.operation_type {
        CatalogOperationType::Upsert => {
            let entity = operation
                .payload
                .entity
                .as_ref()
                .ok_or_else(|| "Upsert operation has no entity payload.".to_string())?;
            validate_entity(db, entity)?;
            put_entity(db, entity)?
        }
        CatalogOperationType::Delete => {
            delete_entity_if_safe(db, operation.entity_type, &operation.entity_id)?
        }
    };
    record_entity_state(db, operation)?;
    let key = entity_state_key(operation.entity_type, &operation.entity_id);
    if operation.operation_type == CatalogOperationType::Delete {
        let tombstone = CatalogTombstone {
            entity_type: operation.entity_type,
            entity_uuid: operation.entity_id.clone(),
            deleted_at_utc: operation
                .payload
                .deleted_at_utc
                .clone()
                .unwrap_or_else(|| operation.mutation_ts_utc.clone()),
            op_id: operation.op_id.clone(),
            client_id: operation.client_id.clone(),
            local_seq: operation.local_seq,
            base_version: operation.base_version.clone(),
        };
        db.put_sync_tombstone(&key, &tombstone)?;
    } else if db.has_sync_tombstone(&key)? {
        db.delete_sync_tombstone(&key)?;
    }
    Ok(changed)
}

fn validate_entity(db: &InventoryDb, entity: &CatalogEntity) -> CommandResult<()> {
    match entity {
        CatalogEntity::Part(part) => validate_part(part),
        CatalogEntity::StorageArea(area) => validate_storage_area(area),
        CatalogEntity::StorageContainer(container) => {
            validate_storage_container(container)?;
            if db.find_storage_area(&container.area_uuid)?.is_none() {
                return Err("Storage container references a missing area.".to_string());
            }
            Ok(())
        }
        CatalogEntity::StockPlacement(placement) => {
            if db.find_part(&placement.part_uuid)?.is_none() {
                return Err("Stock placement references a missing part.".to_string());
            }
            let container = db
                .find_storage_container(&placement.container_uuid)?
                .ok_or_else(|| "Stock placement references a missing container.".to_string())?;
            validate_stock_placement(placement, &container)
        }
    }
}

fn put_entity(db: &InventoryDb, entity: &CatalogEntity) -> CommandResult<bool> {
    match entity {
        CatalogEntity::Part(part) => {
            let changed = db.find_part(&part.entry_uuid)?.as_ref() != Some(part);
            db.put_part(part)?;
            Ok(changed)
        }
        CatalogEntity::StorageArea(area) => {
            let changed = db.find_storage_area(&area.area_uuid)?.as_ref() != Some(area);
            db.put_storage_area(area)?;
            Ok(changed)
        }
        CatalogEntity::StorageContainer(container) => {
            let changed = db
                .find_storage_container(&container.container_uuid)?
                .as_ref()
                != Some(container);
            db.put_storage_container(container)?;
            Ok(changed)
        }
        CatalogEntity::StockPlacement(placement) => {
            let changed =
                db.find_stock_placement(&placement.placement_uuid)?.as_ref() != Some(placement);
            db.put_stock_placement(placement)?;
            Ok(changed)
        }
    }
}

fn delete_entity_if_safe(
    db: &InventoryDb,
    entity_type: CatalogEntityType,
    entity_uuid: &str,
) -> CommandResult<bool> {
    match entity_type {
        CatalogEntityType::StockPlacement => {
            let Some(placement) = db.find_stock_placement(entity_uuid)? else {
                return Ok(false);
            };
            db.delete_stock_placement(&placement)?;
            Ok(true)
        }
        CatalogEntityType::Part => {
            let Some(part) = db.find_part(entity_uuid)? else {
                return Ok(false);
            };
            if !db.load_stock_placements_for_part(entity_uuid)?.is_empty() {
                return Err("Part delete would orphan stock placements.".to_string());
            }
            db.delete_part(&part)?;
            Ok(true)
        }
        CatalogEntityType::StorageContainer => {
            let Some(container) = db.find_storage_container(entity_uuid)? else {
                return Ok(false);
            };
            if !db
                .load_stock_placements_for_container(entity_uuid)?
                .is_empty()
            {
                return Err("Container delete would orphan stock placements.".to_string());
            }
            db.delete_storage_container(&container)?;
            Ok(true)
        }
        CatalogEntityType::StorageArea => {
            let Some(area) = db.find_storage_area(entity_uuid)? else {
                return Ok(false);
            };
            if db
                .load_storage_containers()?
                .iter()
                .any(|container| container.area_uuid == entity_uuid)
            {
                return Err("Area delete would orphan storage containers.".to_string());
            }
            db.delete_storage_area(&area)?;
            Ok(true)
        }
    }
}

fn current_entity_timestamp(
    db: &InventoryDb,
    entity_type: CatalogEntityType,
    entity_uuid: &str,
) -> CommandResult<Option<String>> {
    match entity_type {
        CatalogEntityType::Part => Ok(db.find_part(entity_uuid)?.map(|value| value.updated_at)),
        CatalogEntityType::StorageArea => Ok(db
            .find_storage_area(entity_uuid)?
            .map(|value| value.updated_at)),
        CatalogEntityType::StorageContainer => Ok(db
            .find_storage_container(entity_uuid)?
            .map(|value| value.updated_at)),
        CatalogEntityType::StockPlacement => Ok(db
            .find_stock_placement(entity_uuid)?
            .map(|value| value.updated_at)),
    }
}

fn current_entity_state(
    db: &InventoryDb,
    entity_type: CatalogEntityType,
    entity_uuid: &str,
) -> CommandResult<Option<CatalogEntityState>> {
    db.sync_entry_state(&entity_state_key(entity_type, entity_uuid))
}

fn record_entity_state(
    db: &InventoryDb,
    operation: &CatalogOperationEnvelope,
) -> CommandResult<()> {
    let state = CatalogEntityState {
        entity_type: operation.entity_type,
        entity_uuid: operation.entity_id.clone(),
        last_op_id: operation.op_id.clone(),
        mutation_ts_utc: operation.mutation_ts_utc.clone(),
        deleted: operation.operation_type == CatalogOperationType::Delete,
        base_version: operation.base_version.clone(),
        changed_fields: operation.payload.changed_fields.clone(),
        source_client_id: operation.client_id.clone(),
        source_local_seq: operation.local_seq,
        updated_at_utc: now_timestamp(),
    };
    db.put_sync_entry_state(
        &entity_state_key(operation.entity_type, &operation.entity_id),
        &state,
    )?;
    Ok(())
}

fn mark_operation_applied(
    db: &InventoryDb,
    operation: &CatalogOperationEnvelope,
) -> CommandResult<()> {
    let marker = CatalogAppliedMarker {
        op_id: operation.op_id.clone(),
        client_id: operation.client_id.clone(),
        local_seq: operation.local_seq,
        checksum: operation.checksum.clone(),
        applied_at_utc: now_timestamp(),
    };
    db.put_sync_applied_marker(&operation.op_id, &marker)?;
    db.put_sync_client_seq_marker(&operation.client_id, operation.local_seq, &operation.op_id)?;
    Ok(())
}

fn record_conflict(
    db: &InventoryDb,
    operation: &CatalogOperationEnvelope,
    current: Option<&CatalogEntityState>,
    reason: &str,
) -> CommandResult<()> {
    let record = CatalogConflictRecord {
        conflict_id: Uuid::new_v4().simple().to_string(),
        entity_type: operation.entity_type,
        entity_uuid: operation.entity_id.clone(),
        incoming_op_id: operation.op_id.clone(),
        incoming_client_id: operation.client_id.clone(),
        incoming_local_seq: operation.local_seq,
        current_op_id: current.map(|state| state.last_op_id.clone()),
        reason: reason.to_string(),
        detected_at_utc: now_timestamp(),
    };
    db.put_sync_conflict_record(&record.conflict_id, &record)?;
    Ok(())
}

fn compare_operations(
    left: &CatalogOperationEnvelope,
    right: &CatalogOperationEnvelope,
) -> std::cmp::Ordering {
    left.mutation_ts_utc
        .cmp(&right.mutation_ts_utc)
        .then_with(|| left.client_id.cmp(&right.client_id))
        .then_with(|| left.local_seq.cmp(&right.local_seq))
        .then_with(|| left.op_id.cmp(&right.op_id))
}

fn operation_wins_state(incoming: &CatalogOperationEnvelope, current: &CatalogEntityState) -> bool {
    (
        incoming.mutation_ts_utc.as_str(),
        incoming.client_id.as_str(),
        incoming.local_seq,
        incoming.op_id.as_str(),
    ) > (
        current.mutation_ts_utc.as_str(),
        current.source_client_id.as_str(),
        current.source_local_seq,
        current.last_op_id.as_str(),
    )
}

fn operation_apply_rank(operation: &CatalogOperationEnvelope) -> u8 {
    match operation.operation_type {
        CatalogOperationType::Upsert => operation.entity_type.upsert_rank(),
        CatalogOperationType::Delete => operation.entity_type.delete_rank(),
    }
}

fn push_pending_local_operations(
    db: &InventoryDb,
    paths: &CatalogSharedSyncPaths,
) -> CommandResult<usize> {
    let mut pushed = 0usize;
    let mut watermarks = sync_watermarks(db)?;
    db.scan_sync_outbox_records::<CatalogOperationEnvelope, _>(
        None,
        usize::MAX,
        |_, operation| {
            validate_operation(&operation)?;
            let current = watermarks
                .get(&operation.client_id)
                .copied()
                .unwrap_or_default();
            if operation.local_seq <= current {
                return Ok(true);
            }
            write_operation_file(paths, &operation)?;
            db.set_sync_watermark(&operation.client_id, operation.local_seq)?;
            watermarks.insert(operation.client_id.clone(), operation.local_seq);
            pushed += 1;
            Ok(true)
        },
    )?;
    db.flush();
    Ok(pushed)
}

fn count_pending_local_operations(
    db: &InventoryDb,
    paths: Option<&CatalogSharedSyncPaths>,
) -> CommandResult<usize> {
    let watermarks = sync_watermarks(db)?;
    let mut count = 0usize;
    db.scan_sync_outbox_records::<CatalogOperationEnvelope, _>(
        None,
        usize::MAX,
        |_, operation| {
            let pushed = watermarks
                .get(&operation.client_id)
                .is_some_and(|watermark| operation.local_seq <= *watermark)
                || paths
                    .map(|paths| operation_file_matches(paths, &operation))
                    .unwrap_or(false);
            if !pushed {
                count += 1;
            }
            Ok(true)
        },
    )?;
    Ok(count)
}

fn write_operation_file(
    paths: &CatalogSharedSyncPaths,
    operation: &CatalogOperationEnvelope,
) -> CommandResult<PathBuf> {
    validate_operation(operation)?;
    let final_path = operation_file_path(paths, &operation.client_id, operation.local_seq)?;
    if final_path.exists() {
        let existing = read_operation_file(&final_path, &operation.client_id, operation.local_seq)?;
        if existing.checksum == operation.checksum && existing.op_id == operation.op_id {
            return Ok(final_path);
        }
        return Err(
            "Existing catalog operation file has the same client and sequence with different content."
                .to_string(),
        );
    }
    let parent = final_path
        .parent()
        .ok_or_else(|| "Catalog operation path has no parent.".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temp_path = parent.join(format!(
        "{}.tmp-{}-{}",
        operation_file_name(operation.local_seq),
        process::id(),
        Uuid::new_v4().simple()
    ));
    let bytes = canonical_json_bytes(operation)?;
    let result = (|| -> CommandResult<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp_path)
            .map_err(|error| error.to_string())?;
        file.write_all(&bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        drop(file);
        fs::rename(&temp_path, &final_path).map_err(|error| error.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result.map(|_| final_path)
}

fn read_operation_file(
    path: &Path,
    expected_client_id: &str,
    expected_local_seq: u64,
) -> CommandResult<CatalogOperationEnvelope> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let operation: CatalogOperationEnvelope =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if operation.client_id != expected_client_id || operation.local_seq != expected_local_seq {
        return Err("Catalog operation path identity does not match its payload.".to_string());
    }
    validate_operation(&operation)?;
    Ok(operation)
}

fn validate_operation(operation: &CatalogOperationEnvelope) -> CommandResult<()> {
    if operation.schema_version != CATALOG_SYNC_SCHEMA_VERSION {
        return Err(format!(
            "Unsupported Lab catalog sync schema version {}.",
            operation.schema_version
        ));
    }
    validate_path_segment("client id", &operation.client_id)?;
    validate_path_segment("operation id", &operation.op_id)?;
    if operation.local_seq == 0 {
        return Err("Catalog operation sequence must be greater than zero.".to_string());
    }
    if operation.entity_id.trim().is_empty() || operation.entity_id.contains(':') {
        return Err("Catalog operation entity identity is invalid.".to_string());
    }
    match operation.operation_type {
        CatalogOperationType::Upsert => {
            let entity = operation
                .payload
                .entity
                .as_ref()
                .ok_or_else(|| "Catalog upsert operation is missing its entity.".to_string())?;
            if entity.entity_type() != operation.entity_type
                || entity.entity_id() != operation.entity_id
            {
                return Err(
                    "Catalog operation entity payload does not match its envelope.".to_string(),
                );
            }
            if operation.payload.deleted_at_utc.is_some() {
                return Err(
                    "Catalog upsert operation cannot include a delete timestamp.".to_string(),
                );
            }
        }
        CatalogOperationType::Delete => {
            if operation.payload.entity.is_some() || operation.payload.deleted_at_utc.is_none() {
                return Err("Catalog delete operation payload is invalid.".to_string());
            }
        }
    }
    let expected_checksum = canonical_checksum(operation)?;
    if operation.checksum != expected_checksum {
        return Err("Catalog operation checksum does not match canonical content.".to_string());
    }
    verify_canonical_bytes(
        "catalog.sync.operation.v2",
        &canonical_bytes_without_checksum_or_auth(operation)?,
        operation.auth.as_deref(),
    )?;
    Ok(())
}

fn operation_file_matches(
    paths: &CatalogSharedSyncPaths,
    operation: &CatalogOperationEnvelope,
) -> bool {
    operation_file_path(paths, &operation.client_id, operation.local_seq)
        .ok()
        .filter(|path| path.exists())
        .and_then(|path| read_operation_file(&path, &operation.client_id, operation.local_seq).ok())
        .is_some_and(|existing| {
            existing.op_id == operation.op_id && existing.checksum == operation.checksum
        })
}

fn scan_operation_files(
    db: &InventoryDb,
    paths: &CatalogSharedSyncPaths,
    watermarks: &HashMap<String, u64>,
) -> CommandResult<(Vec<CatalogOperationEnvelope>, usize)> {
    let mut operations = Vec::new();
    let mut corrupt_count = 0usize;
    for client_entry in fs::read_dir(&paths.ops_dir).map_err(|error| error.to_string())? {
        let client_entry = client_entry.map_err(|error| error.to_string())?;
        if !client_entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir()
        {
            continue;
        }
        let client_id = client_entry.file_name().to_string_lossy().into_owned();
        if validate_path_segment("client id", &client_id).is_err() {
            continue;
        }
        let watermark = watermarks.get(&client_id).copied().unwrap_or_default();
        for file_entry in fs::read_dir(client_entry.path()).map_err(|error| error.to_string())? {
            let file_entry = file_entry.map_err(|error| error.to_string())?;
            if !file_entry
                .file_type()
                .map_err(|error| error.to_string())?
                .is_file()
            {
                continue;
            }
            let file_name = file_entry.file_name().to_string_lossy().into_owned();
            let Some(local_seq) = parse_operation_file_name(&file_name) else {
                continue;
            };
            if local_seq <= watermark {
                continue;
            }
            match read_operation_file(&file_entry.path(), &client_id, local_seq) {
                Ok(operation) => operations.push(operation),
                Err(error) => {
                    corrupt_count += 1;
                    record_corrupt_remote(db, &file_entry.path(), error)?;
                }
            }
        }
    }
    operations.sort_by(|left, right| {
        left.client_id
            .cmp(&right.client_id)
            .then_with(|| left.local_seq.cmp(&right.local_seq))
    });
    Ok((operations, corrupt_count))
}

fn record_corrupt_remote(db: &InventoryDb, path: &Path, detail: String) -> CommandResult<()> {
    let content_sha256 = fs::read(path)
        .ok()
        .map(|bytes| format!("{CHECKSUM_PREFIX}{}", sha256_hex(&bytes)));
    let record = CatalogCorruptRemoteRecord {
        record_id: Uuid::new_v4().simple().to_string(),
        path: path.to_string_lossy().into_owned(),
        detail,
        detected_at_utc: now_timestamp(),
        content_sha256,
    };
    db.put_sync_corrupt_record(&record.record_id, &record)?;
    Ok(())
}

fn publish_snapshot(
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
    write_json_atomic(&snapshot_path, &snapshot, true)?;
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
    manifest.checksum = canonical_checksum(&manifest)?;
    manifest.auth = sign_canonical_bytes(
        "catalog.sync.manifest.v2",
        &canonical_bytes_without_checksum_or_auth(&manifest)?,
    )
    .map_err(|error| error.to_string())?;
    write_json_atomic(&paths.manifest_path, &manifest, false)?;
    db.set_last_snapshot_id(&manifest.snapshot_id)?;
    db.flush();
    Ok(true)
}

fn build_snapshot(db: &InventoryDb) -> CommandResult<CatalogSnapshot> {
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
    snapshot.checksum = canonical_checksum(&snapshot)?;
    snapshot.auth = sign_canonical_bytes(
        "catalog.sync.snapshot.v2",
        &canonical_bytes_without_checksum_or_auth(&snapshot)?,
    )
    .map_err(|error| error.to_string())?;
    Ok(snapshot)
}

fn apply_latest_snapshot_if_safe(
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

fn validate_snapshot_entities(snapshot: &CatalogSnapshot) -> CommandResult<()> {
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

fn read_manifest(paths: &CatalogSharedSyncPaths) -> CommandResult<Option<CatalogManifest>> {
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
    if manifest.checksum != canonical_checksum(&manifest)? {
        return Err("Shared Lab catalog manifest checksum is invalid.".to_string());
    }
    verify_canonical_bytes(
        "catalog.sync.manifest.v2",
        &canonical_bytes_without_checksum_or_auth(&manifest)?,
        manifest.auth.as_deref(),
    )?;
    Ok(Some(manifest))
}

fn read_snapshot(
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
    if snapshot.checksum != manifest.snapshot_checksum
        || snapshot.checksum != canonical_checksum(&snapshot)?
    {
        return Err("Shared Lab catalog snapshot checksum is invalid.".to_string());
    }
    verify_canonical_bytes(
        "catalog.sync.snapshot.v2",
        &canonical_bytes_without_checksum_or_auth(&snapshot)?,
        snapshot.auth.as_deref(),
    )?;
    Ok(snapshot)
}

fn read_cutover_marker(
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
    if marker.checksum != canonical_checksum(&marker)? {
        return Err("Lab catalog cutover marker checksum is invalid.".to_string());
    }
    verify_canonical_bytes(
        "catalog.sync.cutover.v2",
        &canonical_bytes_without_checksum_or_auth(&marker)?,
        marker.auth.as_deref(),
    )?;
    Ok(Some(marker))
}

fn catalog_cutover_ready(paths: &CatalogSharedSyncPaths) -> CommandResult<bool> {
    Ok(read_cutover_marker(paths)?.is_some() && ensure_legacy_stream_blocked(paths).is_ok())
}

fn ensure_legacy_stream_blocked(paths: &CatalogSharedSyncPaths) -> CommandResult<()> {
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

fn legacy_stream_state(paths: &CatalogSharedSyncPaths) -> CommandResult<String> {
    if paths.legacy_inventory_path.is_dir() {
        Ok("legacy_directory".to_string())
    } else if paths.legacy_inventory_path.is_file() {
        match ensure_legacy_stream_blocked(paths) {
            Ok(()) => Ok("blocked_file".to_string()),
            Err(_) => Ok("unexpected_file".to_string()),
        }
    } else {
        Ok("missing".to_string())
    }
}

fn catalog_fingerprint(db: &InventoryDb) -> CommandResult<String> {
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
    let bytes = canonical_json_bytes(&FingerprintSource {
        schema_version: CATALOG_SCHEMA_VERSION,
        parts,
        storage_areas,
        storage_containers,
        stock_placements,
    })?;
    Ok(format!("{CHECKSUM_PREFIX}{}", sha256_hex(&bytes)))
}

fn sync_watermarks(db: &InventoryDb) -> CommandResult<HashMap<String, u64>> {
    let mut watermarks = HashMap::new();
    db.scan_sync_watermarks(usize::MAX, |client_id, local_seq| {
        watermarks.insert(client_id, local_seq);
        Ok(true)
    })?;
    Ok(watermarks)
}

fn entity_state_key(entity_type: CatalogEntityType, entity_uuid: &str) -> String {
    format!("{}--{}", entity_type.as_str(), entity_uuid.trim())
}

fn resolve_shared_root() -> PathBuf {
    env::var_os(SHARED_ROOT_ENV)
        .and_then(|value| {
            let path = value.to_string_lossy().trim().to_string();
            (!path.is_empty()).then_some(PathBuf::from(path))
        })
        .unwrap_or_else(|| default_shared_root(ModuleId::TeLabComponents))
}

fn shared_sync_enabled() -> bool {
    match env::var_os(SHARED_SYNC_ENABLED_ENV) {
        None => true,
        Some(value) => {
            let normalized = value.to_string_lossy().trim().to_ascii_lowercase();
            !matches!(normalized.as_str(), "0" | "false" | "no" | "off")
        }
    }
}

fn build_status(
    db: &InventoryDb,
    paths: &CatalogSharedSyncPaths,
    available: bool,
    pending_count: usize,
    corrupt_count: usize,
    mut message: String,
) -> InventorySharedStatus {
    if pending_count > 0 && !message.contains("Pending local") {
        message.push_str(&format!(" Pending local changes: {pending_count}."));
    }
    if corrupt_count > 0 && !message.contains("corrupt") {
        message.push_str(&format!(" Corrupt remote files ignored: {corrupt_count}."));
    }
    InventorySharedStatus {
        available,
        can_modify: true,
        enabled: true,
        has_local_only_changes: Some(pending_count > 0),
        message,
        mutation_mode: if available && pending_count == 0 {
            "shared"
        } else {
            "local"
        }
        .to_string(),
        revision: db.sync_revision().ok().map(|value| value.to_string()),
        last_snapshot_id: db.last_snapshot_id().ok().flatten(),
        shared_root_path: Some(paths.shared_root.to_string_lossy().into_owned()),
    }
}

fn disabled_status(db: Option<&InventoryDb>, message: impl Into<String>) -> InventorySharedStatus {
    InventorySharedStatus {
        available: false,
        can_modify: true,
        enabled: false,
        has_local_only_changes: db
            .map(|db| count_pending_local_operations(db, None).unwrap_or(0) > 0),
        message: message.into(),
        mutation_mode: "local".to_string(),
        revision: db.and_then(|db| db.sync_revision().ok().map(|value| value.to_string())),
        last_snapshot_id: db.and_then(|db| db.last_snapshot_id().ok().flatten()),
        shared_root_path: None,
    }
}

fn ensure_catalog_layout(paths: &CatalogSharedSyncPaths) -> CommandResult<()> {
    fs::create_dir_all(&paths.ops_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&paths.snapshots_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&paths.locks_dir).map_err(|error| error.to_string())?;
    fs::create_dir_all(&paths.backups_dir).map_err(|error| error.to_string())?;
    Ok(())
}

fn write_new_file(path: &Path, bytes: &[u8]) -> CommandResult<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| error.to_string())?;
    file.write_all(bytes).map_err(|error| error.to_string())?;
    file.sync_all().map_err(|error| error.to_string())?;
    Ok(())
}

fn write_json_atomic<T: Serialize>(path: &Path, value: &T, create_new: bool) -> CommandResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| "JSON path has no parent directory.".to_string())?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let temp_path = parent.join(format!(
        ".{}.tmp-{}-{}",
        path.file_name()
            .map(|name| name.to_string_lossy())
            .unwrap_or_default(),
        process::id(),
        Uuid::new_v4().simple()
    ));
    let bytes = canonical_json_bytes(value)?;
    let result = (|| -> CommandResult<()> {
        write_new_file(&temp_path, &bytes)?;
        if create_new && path.exists() {
            return Err(format!("File already exists: {}", path.display()));
        }
        if !create_new && path.exists() {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
        fs::rename(&temp_path, path).map_err(|error| error.to_string())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

fn operation_file_path(
    paths: &CatalogSharedSyncPaths,
    client_id: &str,
    local_seq: u64,
) -> CommandResult<PathBuf> {
    validate_path_segment("client id", client_id)?;
    Ok(paths
        .ops_dir
        .join(client_id)
        .join(operation_file_name(local_seq)))
}

fn operation_file_name(local_seq: u64) -> String {
    format!("{local_seq:0LOCAL_SEQ_WIDTH$}{OP_FILE_SUFFIX}")
}

fn parse_operation_file_name(file_name: &str) -> Option<u64> {
    let sequence = file_name.strip_suffix(OP_FILE_SUFFIX)?;
    if sequence.len() != LOCAL_SEQ_WIDTH || !sequence.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    sequence.parse().ok().filter(|value| *value > 0)
}

fn validate_path_segment(label: &str, value: &str) -> CommandResult<()> {
    if value.trim().is_empty()
        || value.contains(['/', '\\', ':'])
        || value.chars().any(char::is_control)
    {
        return Err(format!("Catalog sync {label} is not a safe path segment."));
    }
    Ok(())
}

fn nonempty_timestamp(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        now_timestamp()
    } else {
        value.to_string()
    }
}

fn canonical_checksum<T: Serialize>(value: &T) -> CommandResult<String> {
    Ok(format!(
        "{CHECKSUM_PREFIX}{}",
        sha256_hex(&canonical_bytes_without_checksum_or_auth(value)?)
    ))
}

fn canonical_json_bytes<T: Serialize>(value: &T) -> CommandResult<Vec<u8>> {
    let value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    serde_json::to_vec(&canonicalize_json_value(value)).map_err(|error| error.to_string())
}

fn canonical_bytes_without_checksum_or_auth<T: Serialize>(value: &T) -> CommandResult<Vec<u8>> {
    let mut value = serde_json::to_value(value).map_err(|error| error.to_string())?;
    if let Value::Object(object) = &mut value {
        object.remove("checksum");
        object.remove("auth");
    }
    serde_json::to_vec(&canonicalize_json_value(value)).map_err(|error| error.to_string())
}

fn canonicalize_json_value(value: Value) -> Value {
    match value {
        Value::Array(values) => {
            Value::Array(values.into_iter().map(canonicalize_json_value).collect())
        }
        Value::Object(object) => {
            let mut keys = object.keys().cloned().collect::<Vec<_>>();
            keys.sort();
            let mut sorted = Map::new();
            for key in keys {
                if let Some(value) = object.get(&key) {
                    sorted.insert(key, canonicalize_json_value(value.clone()));
                }
            }
            Value::Object(sorted)
        }
        value => value,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::te_lab_components::catalog_model::{
        create_part, create_stock_placement, create_storage_area, create_storage_container,
        normalize_part_input, normalize_stock_placement_input, normalize_storage_area_input,
        normalize_storage_container_input, PartInput, StockPlacementInput, StorageAreaInput,
        StorageContainerInput,
    };
    use std::{env, fs, path::PathBuf};

    #[test]
    fn cutover_quarantines_and_blocks_the_legacy_v1_stream() {
        let db = test_db("catalog-cutover-db");
        ensure_catalog_initialized(&db).unwrap();
        let root = unique_test_dir("catalog-cutover-root");
        let legacy = root.join("shared").join("inventory");
        fs::create_dir_all(&legacy).unwrap();
        fs::write(legacy.join("manifest.json"), b"legacy").unwrap();

        let preview = preview_shared_cutover_with_root(&db, root.clone()).unwrap();
        assert_eq!(preview.legacy_stream_state, "legacy_directory");
        let result = commit_shared_cutover_with_root(
            CatalogSharedCutoverCommitInput {
                local_fingerprint: preview.local_fingerprint,
                confirmed: true,
            },
            &db,
            root.clone(),
        )
        .unwrap();

        assert!(!result.noop);
        assert!(root.join("shared").join("inventory").is_file());
        assert!(result
            .legacy_backup_path
            .as_ref()
            .is_some_and(|path| Path::new(path).join("manifest.json").exists()));
        assert!(root
            .join("shared")
            .join("catalog-v2")
            .join("cutover.json")
            .exists());
    }

    #[test]
    fn two_databases_sync_every_catalog_entity_and_dependency_safe_deletes() {
        let first = test_db("catalog-sync-first");
        let second = test_db("catalog-sync-second");
        ensure_catalog_initialized(&first).unwrap();
        ensure_catalog_initialized(&second).unwrap();
        let root = unique_test_dir("catalog-sync-root");
        fs::create_dir_all(&root).unwrap();
        let (part, area, container, placement) = seed_catalog(&first);
        queue_upsert_operation(&first, area.clone().into(), Vec::new(), None).unwrap();
        queue_upsert_operation(&first, container.clone().into(), Vec::new(), None).unwrap();
        queue_upsert_operation(&first, part.clone().into(), Vec::new(), None).unwrap();
        queue_upsert_operation(&first, placement.clone().into(), Vec::new(), None).unwrap();
        let preview = preview_shared_cutover_with_root(&first, root.clone()).unwrap();
        commit_shared_cutover_with_root(
            CatalogSharedCutoverCommitInput {
                local_fingerprint: preview.local_fingerprint,
                confirmed: true,
            },
            &first,
            root.clone(),
        )
        .unwrap();

        run_shared_sync_with_root(&second, root.clone()).unwrap();
        assert_eq!(second.load_parts().unwrap(), vec![part.clone()]);
        assert_eq!(second.load_storage_areas().unwrap(), vec![area.clone()]);
        assert_eq!(
            second.load_storage_containers().unwrap(),
            vec![container.clone()]
        );
        assert_eq!(
            second.load_stock_placements().unwrap(),
            vec![placement.clone()]
        );

        first.delete_stock_placement(&placement).unwrap();
        queue_delete_operation(
            &first,
            CatalogEntityType::StockPlacement,
            &placement.placement_uuid,
            now_timestamp(),
            Some(placement.updated_at.clone()),
        )
        .unwrap();
        first.delete_part(&part).unwrap();
        queue_delete_operation(
            &first,
            CatalogEntityType::Part,
            &part.entry_uuid,
            now_timestamp(),
            Some(part.updated_at.clone()),
        )
        .unwrap();
        first.delete_storage_container(&container).unwrap();
        queue_delete_operation(
            &first,
            CatalogEntityType::StorageContainer,
            &container.container_uuid,
            now_timestamp(),
            Some(container.updated_at.clone()),
        )
        .unwrap();
        first.delete_storage_area(&area).unwrap();
        queue_delete_operation(
            &first,
            CatalogEntityType::StorageArea,
            &area.area_uuid,
            now_timestamp(),
            Some(area.updated_at.clone()),
        )
        .unwrap();
        publish_pending_local_changes_with_root(&first, root.clone()).unwrap();
        run_shared_sync_with_root(&second, root).unwrap();

        assert!(second.load_parts().unwrap().is_empty());
        assert!(second.load_storage_areas().unwrap().is_empty());
        assert!(second.load_storage_containers().unwrap().is_empty());
        assert!(second.load_stock_placements().unwrap().is_empty());
    }

    #[test]
    fn simultaneous_quantity_edits_converge_without_additive_merge() {
        let first = test_db("catalog-conflict-first");
        let second = test_db("catalog-conflict-second");
        ensure_catalog_initialized(&first).unwrap();
        ensure_catalog_initialized(&second).unwrap();
        let root = unique_test_dir("catalog-conflict-root");
        fs::create_dir_all(&root).unwrap();
        let (part, area, container, placement) = seed_catalog(&first);
        queue_upsert_operation(&first, area.into(), Vec::new(), None).unwrap();
        queue_upsert_operation(&first, container.into(), Vec::new(), None).unwrap();
        queue_upsert_operation(&first, part.into(), Vec::new(), None).unwrap();
        queue_upsert_operation(&first, placement.clone().into(), Vec::new(), None).unwrap();
        let preview = preview_shared_cutover_with_root(&first, root.clone()).unwrap();
        commit_shared_cutover_with_root(
            CatalogSharedCutoverCommitInput {
                local_fingerprint: preview.local_fingerprint,
                confirmed: true,
            },
            &first,
            root.clone(),
        )
        .unwrap();
        run_shared_sync_with_root(&second, root.clone()).unwrap();

        let base_version = placement.updated_at.clone();
        let mutation_timestamp = "2026-07-25T12:00:00.000Z".to_string();
        let mut first_edit = first
            .find_stock_placement(&placement.placement_uuid)
            .unwrap()
            .unwrap();
        first_edit.quantity = 11.0;
        first_edit.updated_at = mutation_timestamp.clone();
        first.put_stock_placement(&first_edit).unwrap();
        queue_upsert_operation(
            &first,
            first_edit.into(),
            vec!["quantity".to_string()],
            Some(base_version.clone()),
        )
        .unwrap();

        let mut second_edit = second
            .find_stock_placement(&placement.placement_uuid)
            .unwrap()
            .unwrap();
        second_edit.quantity = 29.0;
        second_edit.updated_at = mutation_timestamp;
        second.put_stock_placement(&second_edit).unwrap();
        queue_upsert_operation(
            &second,
            second_edit.into(),
            vec!["quantity".to_string()],
            Some(base_version),
        )
        .unwrap();

        publish_pending_local_changes_with_root(&first, root.clone()).unwrap();
        publish_pending_local_changes_with_root(&second, root.clone()).unwrap();
        run_shared_sync_with_root(&first, root.clone()).unwrap();
        run_shared_sync_with_root(&second, root.clone()).unwrap();
        run_shared_sync_with_root(&first, root.clone()).unwrap();
        run_shared_sync_with_root(&second, root).unwrap();

        let first_quantity = first
            .find_stock_placement(&placement.placement_uuid)
            .unwrap()
            .unwrap()
            .quantity;
        let second_quantity = second
            .find_stock_placement(&placement.placement_uuid)
            .unwrap()
            .unwrap()
            .quantity;
        assert_eq!(first_quantity, second_quantity);
        assert!(matches!(first_quantity, 11.0 | 29.0));
        assert_ne!(first_quantity, 40.0);
        assert!(conflict_count(&first) + conflict_count(&second) > 0);
    }

    fn seed_catalog(db: &InventoryDb) -> (Part, StorageArea, StorageContainer, StockPlacement) {
        let part = create_part(
            1,
            normalize_part_input(PartInput {
                internal_part_number: "LAB-001".to_string(),
                description: "1 kOhm resistor".to_string(),
                default_unit_of_measure: "pcs".to_string(),
                ..PartInput::default()
            }),
        );
        let area = create_storage_area(normalize_storage_area_input(StorageAreaInput {
            name: "Main Lab".to_string(),
            area_type: "lab".to_string(),
            ..StorageAreaInput::default()
        }));
        let container =
            create_storage_container(normalize_storage_container_input(StorageContainerInput {
                area_uuid: area.area_uuid.clone(),
                name: "Cabinet 1".to_string(),
                container_type: "cabinet".to_string(),
                grid_enabled: true,
                row_count: Some(4),
                column_count: Some(4),
                ..StorageContainerInput::default()
            }));
        let placement = create_stock_placement(normalize_stock_placement_input(
            StockPlacementInput {
                part_uuid: part.entry_uuid.clone(),
                container_uuid: container.container_uuid.clone(),
                row_index: Some(1),
                column_index: Some(2),
                quantity: 10.0,
                unit_of_measure: "pcs".to_string(),
                ..StockPlacementInput::default()
            },
            Some("pcs"),
        ));
        db.put_storage_area(&area).unwrap();
        db.put_storage_container(&container).unwrap();
        db.put_part(&part).unwrap();
        db.put_stock_placement(&placement).unwrap();
        db.set_next_entry_id(2).unwrap();
        (part, area, container, placement)
    }

    fn conflict_count(db: &InventoryDb) -> usize {
        let mut count = 0usize;
        db.scan_sync_conflict_records::<CatalogConflictRecord, _>(usize::MAX, |_, _| {
            count += 1;
            Ok(true)
        })
        .unwrap();
        count
    }

    fn test_db(prefix: &str) -> InventoryDb {
        let root = unique_test_dir(prefix);
        fs::create_dir_all(&root).unwrap();
        InventoryDb::open_at(root.join("te-lab-components.feox")).unwrap()
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
    }

    /// Owner one-shot: seed demo parts into local Lab DB if empty, cut over product
    /// shared root to catalog-v2, and publish the initial shared snapshot.
    ///
    ///   set SEED_LAB_SHARED=1
    ///   cargo test seed_live_lab_shared_demo_catalog -- --ignored --nocapture
    #[test]
    #[ignore = "live shared Lab cutover; set SEED_LAB_SHARED=1"]
    fn seed_live_lab_shared_demo_catalog() {
        use crate::modules::te_lab_components::catalog_demo_seed::seed_demo_catalog_if_empty;
        use crate::platform::DEFAULT_LAB_COMPONENTS_SHARED_ROOT;

        if env::var("SEED_LAB_SHARED").ok().as_deref() != Some("1") {
            eprintln!("skip: set SEED_LAB_SHARED=1 to seed shared Lab catalog");
            return;
        }

        let local = env::var("LOCALAPPDATA").expect("LOCALAPPDATA");
        let lab_path = PathBuf::from(local)
            .join("com.inventory.management")
            .join("te-lab-components.feox");
        assert!(
            lab_path.is_file(),
            "local Lab DB missing at {}",
            lab_path.display()
        );

        let shared_root = PathBuf::from(DEFAULT_LAB_COMPONENTS_SHARED_ROOT);
        assert!(
            shared_root.is_dir(),
            "shared root missing at {}",
            shared_root.display()
        );

        let db = InventoryDb::open_at(lab_path).expect("open local Lab DB");
        ensure_catalog_initialized(&db).expect("init catalog v2");
        seed_demo_catalog_if_empty(&db).expect("seed demo parts when empty");
        repair_demo_counted_placements(&db);
        queue_all_catalog_entities(&db);

        let part_count = db.load_parts().expect("load parts").len();
        let placement_count = db.load_stock_placements().expect("load placements").len();
        assert!(
            part_count >= 6,
            "expected at least the 6 demo parts, got {part_count}"
        );
        eprintln!("local catalog: {part_count} parts, {placement_count} placements");

        let preview = preview_shared_cutover_with_root(&db, shared_root.clone()).expect("preview");
        eprintln!(
            "cutover preview: blocking={} catalog_v2_initialized={} legacy={} parts={} areas={} containers={} placements={}",
            preview.blocking,
            preview.catalog_v2_initialized,
            preview.legacy_stream_state,
            preview.part_count,
            preview.area_count,
            preview.container_count,
            preview.placement_count
        );
        assert!(
            !preview.blocking,
            "shared cutover blocked: root available={} legacy={}",
            preview.shared_root_available, preview.legacy_stream_state
        );

        let commit = commit_shared_cutover_with_root(
            CatalogSharedCutoverCommitInput {
                local_fingerprint: preview.local_fingerprint.clone(),
                confirmed: true,
            },
            &db,
            shared_root.clone(),
        )
        .expect("commit cutover");
        eprintln!(
            "cutover commit: noop={} catalog_root={} message={}",
            commit.noop, commit.catalog_root_path, commit.message
        );

        let sync = run_shared_sync_with_root(&db, shared_root).expect("shared sync");
        eprintln!(
            "shared sync: available={} enabled={} mode={} message={}",
            sync.shared.available,
            sync.shared.enabled,
            sync.shared.mutation_mode,
            sync.shared.message
        );
        assert!(
            sync.shared.available && sync.shared.enabled,
            "expected shared ready after cutover"
        );
        assert!(
            PathBuf::from(DEFAULT_LAB_COMPONENTS_SHARED_ROOT)
                .join("shared")
                .join("catalog-v2")
                .join("cutover.json")
                .is_file(),
            "expected catalog-v2 cutover marker on shared root"
        );
    }

    /// Make installed 0.1.0 openable after newer catalog-v2 Lab writes.
    ///
    /// 0.1.0 has **no** catalog sync types. Its inventory-style recovery panics on
    /// catalog tombstones/entry-states that use `entityUuid` instead of `entryUuid`.
    /// This strips catalog sync metadata (keeps part/placement data for newer builds).
    ///
    ///   set REPAIR_LAB_OUTBOX=1
    ///   cargo test repair_live_lab_outbox_for_legacy_install -- --ignored --nocapture
    #[test]
    #[ignore = "live Lab outbox repair for 0.1.0; set REPAIR_LAB_OUTBOX=1"]
    fn repair_live_lab_outbox_for_legacy_install() {
        if env::var("REPAIR_LAB_OUTBOX").ok().as_deref() != Some("1") {
            eprintln!("skip: set REPAIR_LAB_OUTBOX=1 to strip Lab outbox for legacy 0.1.0");
            return;
        }

        let local = env::var("LOCALAPPDATA").expect("LOCALAPPDATA");
        let lab_path = PathBuf::from(local)
            .join("com.inventory.management")
            .join("te-lab-components.feox");
        assert!(
            lab_path.is_file(),
            "missing Lab DB at {}",
            lab_path.display()
        );

        let file_size = std::fs::metadata(&lab_path)
            .expect("metadata")
            .len()
            .max(64 * 1024 * 1024);
        let db =
            InventoryDb::open_at_with_size(lab_path.clone(), file_size).expect("open local Lab DB");
        let schema = db.schema_version().expect("schema");
        eprintln!("Lab DB {} schema_version={schema:?}", lab_path.display());

        let mut rows: Vec<(u64, String)> = Vec::new();
        db.scan_sync_outbox_raw(None, usize::MAX, |local_seq, value| {
            let preview = String::from_utf8_lossy(value);
            let op = if preview.contains("catalog.entity.upsert") {
                "catalog.entity.upsert".to_string()
            } else if preview.contains("catalog.entity.delete") {
                "catalog.entity.delete".to_string()
            } else if preview.contains("inventory.entry") {
                "inventory.entry.*".to_string()
            } else {
                format!("other/unknown ({} bytes)", value.len())
            };
            rows.push((local_seq, op));
            Ok(true)
        })
        .expect("scan outbox");

        eprintln!("outbox rows before repair: {}", rows.len());
        for (seq, op) in &rows {
            eprintln!("  seq={seq} op={op}");
        }

        let mut deleted_outbox = 0usize;
        for (seq, op) in &rows {
            // Remove everything that installed 0.1.0 inventory-style recovery cannot parse,
            // and catalog ops that re-poison after mixed dev use of the same AppData path.
            if op.starts_with("catalog.entity") || op.starts_with("other") {
                db.delete_sync_outbox_record(*seq)
                    .unwrap_or_else(|error| panic!("delete outbox seq {seq}: {error}"));
                deleted_outbox += 1;
            }
        }

        // Catalog entity states/tombstones use entityUuid — 0.1.0 expects entryUuid and panics.
        let mut deleted_states = 0usize;
        let mut state_keys: Vec<String> = Vec::new();
        db.scan_sync_range(SyncKeyspace::EntryState, usize::MAX, |key, value| {
            let preview = String::from_utf8_lossy(value);
            if preview.contains("entityUuid") || !preview.contains("entryUuid") {
                state_keys.push(String::from_utf8_lossy(key).into_owned());
            }
            Ok(true)
        })
        .expect("scan entry states");
        for key in &state_keys {
            let entity = key
                .strip_prefix("sync:entry_state:")
                .unwrap_or(key.as_str());
            db.delete_sync_entry_state(entity)
                .unwrap_or_else(|error| panic!("delete entry state {entity}: {error}"));
            deleted_states += 1;
        }

        let mut deleted_tombstones = 0usize;
        let mut tombstone_keys: Vec<String> = Vec::new();
        db.scan_sync_range(SyncKeyspace::Tombstone, usize::MAX, |key, value| {
            let preview = String::from_utf8_lossy(value);
            if preview.contains("entityUuid") || !preview.contains("entryUuid") {
                tombstone_keys.push(String::from_utf8_lossy(key).into_owned());
            }
            Ok(true)
        })
        .expect("scan tombstones");
        for key in &tombstone_keys {
            let entity = key.strip_prefix("sync:tombstone:").unwrap_or(key.as_str());
            db.delete_sync_tombstone(entity)
                .unwrap_or_else(|error| panic!("delete tombstone {entity}: {error}"));
            deleted_tombstones += 1;
        }

        db.flush();

        let mut remaining = 0usize;
        db.scan_sync_outbox_raw(None, usize::MAX, |_, _| {
            remaining += 1;
            Ok(true)
        })
        .expect("rescan");
        eprintln!(
            "deleted_outbox={deleted_outbox} deleted_entry_states={deleted_states} deleted_tombstones={deleted_tombstones} remaining_outbox={remaining}"
        );
        assert_eq!(remaining, 0, "expected catalog outbox cleared");
        assert_eq!(deleted_states, state_keys.len());
        assert_eq!(deleted_tombstones, tombstone_keys.len());
    }

    /// Re-bootstrap the six demo parts after accidental local/shared deletion.
    ///
    ///   set RESEED_LAB_DEMO=1
    ///   cargo test reseed_live_lab_demo_catalog -- --ignored --nocapture
    #[test]
    #[ignore = "live reseed Lab demo catalog; set RESEED_LAB_DEMO=1"]
    fn reseed_live_lab_demo_catalog() {
        use crate::modules::te_lab_components::catalog_demo_seed::reseed_demo_catalog;
        use crate::platform::DEFAULT_LAB_COMPONENTS_SHARED_ROOT;

        if env::var("RESEED_LAB_DEMO").ok().as_deref() != Some("1") {
            eprintln!("skip: set RESEED_LAB_DEMO=1 to reseed Lab demo catalog");
            return;
        }

        let local = env::var("LOCALAPPDATA").expect("LOCALAPPDATA");
        let lab_path = PathBuf::from(local)
            .join("com.inventory.management")
            .join("te-lab-components.feox");
        let shared_root = PathBuf::from(DEFAULT_LAB_COMPONENTS_SHARED_ROOT);
        let db = InventoryDb::open_at(lab_path).expect("open local Lab DB");
        ensure_catalog_initialized(&db).expect("init catalog v2");

        let before = db.load_parts().expect("parts").len();
        reseed_demo_catalog(&db).expect("reseed demo catalog");
        repair_demo_counted_placements(&db);
        queue_all_catalog_entities(&db);
        let after = db.load_parts().expect("parts").len();
        eprintln!("reseeded demo catalog: before={before} after={after}");
        assert_eq!(after, 6);

        let sync = publish_pending_local_changes_with_root(&db, shared_root.clone())
            .expect("publish demo catalog");
        let again = run_shared_sync_with_root(&db, shared_root).expect("shared sync");
        eprintln!(
            "publish: available={} enabled={} message={}",
            again.shared.available, again.shared.enabled, again.shared.message
        );
        assert!(sync.shared.enabled || again.shared.enabled);
    }

    fn repair_demo_counted_placements(db: &InventoryDb) {
        for mut placement in db.load_stock_placements().expect("load placements") {
            if placement.count_state == "counted" && placement.last_counted_at.is_none() {
                placement.last_counted_at = Some(now_timestamp());
                if placement.last_counted_by.trim().is_empty() {
                    placement.last_counted_by = "demo-seed".to_string();
                }
                db.put_stock_placement(&placement)
                    .expect("repair counted placement timestamp");
            }
        }
    }

    fn queue_all_catalog_entities(db: &InventoryDb) {
        for area in db.load_storage_areas().expect("areas") {
            queue_upsert_operation(db, area.into(), Vec::new(), None).expect("queue area");
        }
        for container in db.load_storage_containers().expect("containers") {
            queue_upsert_operation(db, container.into(), Vec::new(), None)
                .expect("queue container");
        }
        for part in db.load_parts().expect("parts") {
            queue_upsert_operation(db, part.into(), Vec::new(), None).expect("queue part");
        }
        for placement in db.load_stock_placements().expect("placements") {
            queue_upsert_operation(db, placement.into(), Vec::new(), None)
                .expect("queue placement");
        }
    }
}
