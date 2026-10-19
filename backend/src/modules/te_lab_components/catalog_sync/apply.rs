//! Pulling other computers' catalog operations and applying the newest one per entity.

use std::collections::HashMap;

use uuid::Uuid;

use super::{
    entity_state_key,
    operation_files::{scan_operation_files, validate_operation},
    sync_watermarks,
    types::{
        CatalogAppliedMarker, CatalogConflictRecord, CatalogEntity, CatalogEntityState,
        CatalogEntityType, CatalogOperationEnvelope, CatalogOperationType, CatalogTombstone,
    },
    CatalogSharedSyncPaths,
};
use crate::{
    modules::te_lab_components::{
        catalog_model::{
            validate_part, validate_stock_placement, validate_storage_area,
            validate_storage_container,
        },
        store::InventoryDb,
    },
    platform::common::{now_timestamp, CommandResult},
};

#[derive(Debug, Clone, Copy, Default)]
pub(super) struct PullReport {
    pub(super) entries_changed: bool,
    pub(super) corrupt_count: usize,
    pub(super) quarantined_count: usize,
}

pub(super) fn pull_remote_operations(
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

pub(super) fn apply_remote_candidate(
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

pub(super) fn validate_entity(db: &InventoryDb, entity: &CatalogEntity) -> CommandResult<()> {
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

pub(super) fn put_entity(db: &InventoryDb, entity: &CatalogEntity) -> CommandResult<bool> {
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

pub(super) fn delete_entity_if_safe(
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

pub(super) fn current_entity_timestamp(
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

pub(super) fn current_entity_state(
    db: &InventoryDb,
    entity_type: CatalogEntityType,
    entity_uuid: &str,
) -> CommandResult<Option<CatalogEntityState>> {
    db.sync_entry_state(&entity_state_key(entity_type, entity_uuid))
}

pub(super) fn record_entity_state(
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

pub(super) fn mark_operation_applied(
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

pub(super) fn record_conflict(
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

pub(super) fn compare_operations(
    left: &CatalogOperationEnvelope,
    right: &CatalogOperationEnvelope,
) -> std::cmp::Ordering {
    left.mutation_ts_utc
        .cmp(&right.mutation_ts_utc)
        .then_with(|| left.client_id.cmp(&right.client_id))
        .then_with(|| left.local_seq.cmp(&right.local_seq))
        .then_with(|| left.op_id.cmp(&right.op_id))
}

pub(super) fn operation_wins_state(
    incoming: &CatalogOperationEnvelope,
    current: &CatalogEntityState,
) -> bool {
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

pub(super) fn operation_apply_rank(operation: &CatalogOperationEnvelope) -> u8 {
    match operation.operation_type {
        CatalogOperationType::Upsert => operation.entity_type.upsert_rank(),
        CatalogOperationType::Delete => operation.entity_type.delete_rank(),
    }
}
