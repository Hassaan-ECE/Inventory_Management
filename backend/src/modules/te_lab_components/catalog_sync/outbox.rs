//! This computer's catalog changes: queueing them as operations, pushing them to the share,
//! and counting what has not reached it yet.

use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use super::{
    apply::{mark_operation_applied, record_entity_state, validate_entity},
    checksum, entity_state_key,
    operation_files::{operation_file_matches, validate_operation, write_operation_file},
    signing_bytes, sync_watermarks,
    types::{
        CatalogEntity, CatalogEntityType, CatalogOperationEnvelope, CatalogOperationPayload,
        CatalogOperationType, CatalogTombstone,
    },
    CatalogSharedSyncPaths, CATALOG_SYNC_SCHEMA_VERSION,
};
use crate::{
    modules::te_lab_components::{catalog_model::CATALOG_SCHEMA_VERSION, store::InventoryDb},
    platform::{
        common::{now_timestamp, CommandResult},
        sync::sign_canonical_bytes,
    },
};

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
    db.ensure_catalog_schema()?;
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
    db.ensure_catalog_schema()?;
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

pub(super) fn build_operation(
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
    operation.checksum = checksum(&operation)?;
    operation.auth = sign_canonical_bytes("catalog.sync.operation.v2", &signing_bytes(&operation)?)
        .map_err(|error| error.to_string())?;
    validate_operation(&operation)?;
    Ok(operation)
}

pub(super) fn persist_local_operation(
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

pub(super) fn push_pending_local_operations(
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

pub(super) fn count_pending_local_operations(
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

pub(super) fn nonempty_timestamp(value: &str) -> String {
    let value = value.trim();
    if value.is_empty() {
        now_timestamp()
    } else {
        value.to_string()
    }
}
