use std::collections::BTreeMap;

use crate::platform::{
    common::{now_timestamp, CommandResult},
    store::ModuleStore,
};

use super::{
    operation_file::{operation_file_path, sha256_hex},
    timestamps::compare_timestamp_text,
    types::SyncFieldVersion,
    CorruptRemoteFile, CorruptRemoteReason, SharedSyncPaths, SyncConflictReason,
    SyncConflictRecord, SyncEntryState, SyncOperationEnvelope, SyncOperationType, SyncRecord,
    SyncTombstoneRecord,
};

pub(crate) fn record_corrupt_remote_file<R: SyncRecord>(
    db: &ModuleStore<R>,
    corrupt: &CorruptRemoteFile,
) -> CommandResult<()> {
    put_corrupt_remote_file(db, corrupt)?;
    db.flush();
    Ok(())
}

pub(crate) fn record_corrupt_remote_files<R: SyncRecord>(
    db: &ModuleStore<R>,
    corrupt_files: &[CorruptRemoteFile],
) -> CommandResult<usize> {
    for corrupt in corrupt_files {
        put_corrupt_remote_file(db, corrupt)?;
    }

    if !corrupt_files.is_empty() {
        db.flush();
    }

    Ok(corrupt_files.len())
}

pub(super) fn current_entry_state<R: SyncRecord>(
    db: &ModuleStore<R>,
    entry_uuid: &str,
) -> CommandResult<Option<SyncEntryState<R>>> {
    if let Some(state) = db.sync_entry_state::<SyncEntryState<R>>(entry_uuid)? {
        return Ok(Some(state));
    }

    if let Some(tombstone) = db.sync_tombstone::<SyncTombstoneRecord>(entry_uuid)? {
        return Ok(Some(SyncEntryState {
            entry_uuid: tombstone.entry_uuid,
            last_op_id: tombstone.op_id,
            mutation_ts_utc: tombstone.deleted_at_utc,
            deleted: true,
            base_version: tombstone.base_version,
            changed_fields: Vec::new(),
            source_client_id: tombstone.client_id,
            source_local_seq: tombstone.local_seq,
            operation_type: SyncOperationType::InventoryEntryDelete,
            updated_at_utc: now_timestamp(),
            field_versions: BTreeMap::new(),
            recovery_entry: None,
        }));
    }

    let Some(entry) = db.find_entry(entry_uuid)? else {
        return Ok(None);
    };

    Ok(Some(SyncEntryState {
        entry_uuid: entry.entry_uuid().to_string(),
        last_op_id: String::new(),
        mutation_ts_utc: entry.updated_at().to_string(),
        deleted: false,
        base_version: None,
        changed_fields: Vec::new(),
        source_client_id: "local".to_string(),
        source_local_seq: 0,
        operation_type: SyncOperationType::InventoryEntryUpdate,
        updated_at_utc: now_timestamp(),
        field_versions: BTreeMap::new(),
        recovery_entry: None,
    }))
}

pub(super) fn operation_wins_state<R: SyncRecord>(
    operation: &SyncOperationEnvelope<R>,
    state: &SyncEntryState<R>,
) -> bool {
    let timestamp_order =
        compare_timestamp_text(&operation.mutation_ts_utc, &state.mutation_ts_utc);
    timestamp_order.is_gt()
        || (timestamp_order.is_eq() && operation.op_id.as_str() > state.last_op_id.as_str())
}

/// Same ordering as [`operation_wins_state`], for comparing two recorded states.
pub(super) fn state_wins_state<R: SyncRecord>(
    incoming: &SyncEntryState<R>,
    current: &SyncEntryState<R>,
) -> bool {
    let timestamp_order =
        compare_timestamp_text(&incoming.mutation_ts_utc, &current.mutation_ts_utc);
    timestamp_order.is_gt()
        || (timestamp_order.is_eq() && incoming.last_op_id.as_str() > current.last_op_id.as_str())
}

pub(super) fn record_entry_state_for_operation<R: SyncRecord>(
    db: &ModuleStore<R>,
    operation: &SyncOperationEnvelope<R>,
) -> CommandResult<()> {
    let (field_versions, recovery_entry) = if R::MERGE_RULE.keeps_field_versions() {
        (
            field_versions_after(db, operation)?,
            operation.payload.entry.clone(),
        )
    } else {
        (BTreeMap::new(), None)
    };
    let state = SyncEntryState {
        entry_uuid: operation.entity_id.clone(),
        last_op_id: operation.op_id.clone(),
        mutation_ts_utc: operation.mutation_ts_utc.clone(),
        deleted: operation.operation_type == SyncOperationType::InventoryEntryDelete,
        base_version: operation.base_version.clone(),
        changed_fields: operation.payload.changed_fields.clone(),
        source_client_id: operation.client_id.clone(),
        source_local_seq: operation.local_seq,
        operation_type: operation.operation_type,
        updated_at_utc: now_timestamp(),
        field_versions,
        recovery_entry,
    };
    db.put_sync_entry_state(&operation.entity_id, &state)?;
    Ok(())
}

/// The entry's field versions once `operation` is applied: a create (or the first operation
/// with no versions yet) stamps every field, an update stamps the fields it reports changing,
/// and a delete keeps them as they were.
fn field_versions_after<R: SyncRecord>(
    db: &ModuleStore<R>,
    operation: &SyncOperationEnvelope<R>,
) -> CommandResult<BTreeMap<String, SyncFieldVersion>> {
    let prior = db.sync_entry_state::<SyncEntryState<R>>(&operation.entity_id)?;
    let mut field_versions = prior.map(|state| state.field_versions).unwrap_or_default();
    if operation.operation_type == SyncOperationType::InventoryEntryDelete {
        return Ok(field_versions);
    }
    let fields = if operation.operation_type == SyncOperationType::InventoryEntryCreate
        || field_versions.is_empty()
    {
        R::ENTRY_FIELDS
            .iter()
            .map(|field| field.to_string())
            .collect()
    } else {
        operation.payload.changed_fields.clone()
    };
    for field in fields {
        field_versions.insert(
            field,
            SyncFieldVersion {
                mutation_ts_utc: operation.mutation_ts_utc.clone(),
                op_id: operation.op_id.clone(),
            },
        );
    }
    Ok(field_versions)
}

pub(super) fn record_stale_operation_conflict<R: SyncRecord>(
    db: &ModuleStore<R>,
    operation: &SyncOperationEnvelope<R>,
    current_state: &SyncEntryState<R>,
) -> CommandResult<()> {
    let conflict_id = stale_conflict_id(operation, current_state);
    let record = SyncConflictRecord {
        conflict_id: conflict_id.clone(),
        entry_uuid: operation.entity_id.clone(),
        incoming_op_id: operation.op_id.clone(),
        incoming_client_id: operation.client_id.clone(),
        incoming_local_seq: operation.local_seq,
        incoming_mutation_ts_utc: operation.mutation_ts_utc.clone(),
        current_op_id: current_state.last_op_id.clone(),
        current_client_id: current_state.source_client_id.clone(),
        current_local_seq: current_state.source_local_seq,
        current_mutation_ts_utc: current_state.mutation_ts_utc.clone(),
        reason: SyncConflictReason::StaleIncomingOperation,
        detected_at_utc: now_timestamp(),
    };
    db.put_sync_conflict_record(&conflict_id, &record)?;
    Ok(())
}

fn stale_conflict_id<R: SyncRecord>(
    operation: &SyncOperationEnvelope<R>,
    current_state: &SyncEntryState<R>,
) -> String {
    let source = format!(
        "stale:{}:{}:{}:{}",
        operation.entity_id,
        operation.op_id,
        current_state.last_op_id,
        current_state.mutation_ts_utc
    );
    sha256_hex(source.as_bytes())
}

pub(super) fn record_operation_file_conflict<R: SyncRecord>(
    db: &ModuleStore<R>,
    paths: &SharedSyncPaths,
    operation: &SyncOperationEnvelope<R>,
    detail: String,
) -> CommandResult<()> {
    let path = operation_file_path(paths, &operation.client_id, operation.local_seq)
        .map(|path| path.to_string_lossy().into_owned())
        .unwrap_or_else(|_| format!("{}:{}", operation.client_id, operation.local_seq));
    let corrupt = CorruptRemoteFile {
        path,
        reason: CorruptRemoteReason::DuplicateSequenceDifferentChecksum,
        detail,
        detected_at_utc: now_timestamp(),
        content_sha256: Some(operation.checksum.clone()),
    };
    record_corrupt_remote_file(db, &corrupt)
}

pub(super) fn sync_core_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn put_corrupt_remote_file<R: SyncRecord>(
    db: &ModuleStore<R>,
    corrupt: &CorruptRemoteFile,
) -> CommandResult<()> {
    db.put_sync_corrupt_record(&corrupt_remote_record_id(corrupt), corrupt)?;
    Ok(())
}

pub(crate) fn corrupt_remote_record_id(corrupt: &CorruptRemoteFile) -> String {
    let source = corrupt
        .content_sha256
        .as_deref()
        .unwrap_or(&corrupt.path)
        .as_bytes();
    sha256_hex(source)
}
