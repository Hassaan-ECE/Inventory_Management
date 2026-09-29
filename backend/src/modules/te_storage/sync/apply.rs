use std::{collections::HashMap, path::PathBuf};

use crate::modules::te_storage::{
    model::{now_timestamp, numeric_id, validate_inventory_entry, CommandResult, InventoryEntry},
    store::InventoryDb,
};

use super::{
    conflicts::{
        current_entry_state, operation_wins_state, record_corrupt_remote_file,
        record_corrupt_remote_files, record_entry_state_for_operation,
        record_stale_operation_conflict, sync_core_error,
    },
    operation_file::operation_file_path,
    queue::{
        bootstrap_existing_entries_once, count_pending_local_operations,
        push_pending_local_operations,
    },
    scanning::scan_operation_files_after_watermarks,
    shared_paths::{
        build_shared_status, disabled_shared_status, ensure_operation_log_layout,
        prepare_shared_root, shared_sync_enabled,
    },
    snapshot::{apply_latest_snapshot_if_safe, maybe_publish_snapshot},
    timestamps::max_timestamp_text,
    CorruptRemoteFile, CorruptRemoteReason, SharedSyncPaths, SharedSyncRunResult,
    SyncAppliedMarker, SyncOperationEnvelope, SyncOperationType, SyncTombstoneRecord,
};

pub(crate) fn run_shared_sync(db: &InventoryDb) -> CommandResult<SharedSyncRunResult> {
    if !shared_sync_enabled() {
        return Ok(SharedSyncRunResult {
            entries_changed: false,
            shared: disabled_shared_status(
                Some(db),
                "Shared sync is disabled for this process. No shared path was accessed.",
            ),
        });
    }
    let root = prepare_shared_root();
    run_shared_sync_with_root(db, root)
}

pub(crate) fn publish_pending_local_changes(
    db: &InventoryDb,
) -> CommandResult<SharedSyncRunResult> {
    if !shared_sync_enabled() {
        return Ok(SharedSyncRunResult {
            entries_changed: false,
            shared: disabled_shared_status(
                Some(db),
                "Shared sync is disabled for this process. Change saved locally; sync is not a backup.",
            ),
        });
    }
    let root = prepare_shared_root();
    let paths = SharedSyncPaths::from_shared_root(root);

    if !paths.shared_root.exists() {
        let pending_count = count_pending_local_operations(db, None)?;
        return Ok(SharedSyncRunResult {
            entries_changed: false,
            shared: build_shared_status(
                db,
                &paths,
                false,
                pending_count,
                0,
                "Shared workspace unavailable. Saving changes locally.".to_string(),
            ),
        });
    }

    if let Err(error) = ensure_operation_log_layout(&paths) {
        let pending_count = count_pending_local_operations(db, None)?;
        return Ok(SharedSyncRunResult {
            entries_changed: false,
            shared: build_shared_status(
                db,
                &paths,
                false,
                pending_count,
                0,
                format!("Shared workspace unavailable. {error}"),
            ),
        });
    }

    let pushed_count = push_pending_local_operations(db, &paths)?;
    let pending_count = count_pending_local_operations(db, Some(&paths))?;
    let message = if pushed_count > 0 && pending_count == 0 {
        "Local change published to shared sync.".to_string()
    } else if pushed_count > 0 {
        format!("Published {pushed_count} local change(s) to shared sync.")
    } else {
        "Shared operation sync ready.".to_string()
    };

    Ok(SharedSyncRunResult {
        entries_changed: false,
        shared: build_shared_status(db, &paths, true, pending_count, 0, message),
    })
}

pub(crate) fn run_shared_sync_with_root(
    db: &InventoryDb,
    shared_root: impl Into<PathBuf>,
) -> CommandResult<SharedSyncRunResult> {
    let paths = SharedSyncPaths::from_shared_root(shared_root);

    if !paths.shared_root.exists() {
        bootstrap_existing_entries_once(db)?;
        let pending_count = count_pending_local_operations(db, None)?;
        return Ok(SharedSyncRunResult {
            entries_changed: false,
            shared: build_shared_status(
                db,
                &paths,
                false,
                pending_count,
                0,
                "Shared workspace unavailable. Saving changes locally.".to_string(),
            ),
        });
    }

    if let Err(error) = ensure_operation_log_layout(&paths) {
        bootstrap_existing_entries_once(db)?;
        let pending_count = count_pending_local_operations(db, None)?;
        return Ok(SharedSyncRunResult {
            entries_changed: false,
            shared: build_shared_status(
                db,
                &paths,
                false,
                pending_count,
                0,
                format!("Shared workspace unavailable. {error}"),
            ),
        });
    }

    bootstrap_existing_entries_once(db)?;
    let _pushed_count = push_pending_local_operations(db, &paths)?;
    let pending_before_snapshot = count_pending_local_operations(db, Some(&paths))?;
    let snapshot_report = apply_latest_snapshot_if_safe(db, &paths, pending_before_snapshot)?;
    let pull_report = pull_remote_operations(db, &paths)?;
    let publish_report = maybe_publish_snapshot(db, &paths)?;
    let pending_count = count_pending_local_operations(db, Some(&paths))?;
    let corrupt_count =
        pull_report.corrupt_count + snapshot_report.corrupt_count + publish_report.corrupt_count;
    let mut message = if corrupt_count > 0 {
        format!(
            "Shared operation sync ready. Ignored {} corrupt remote file(s).",
            corrupt_count
        )
    } else {
        "Shared operation sync ready.".to_string()
    };
    if publish_report.snapshot_published {
        message.push_str(" Snapshot refreshed.");
    }

    Ok(SharedSyncRunResult {
        entries_changed: snapshot_report.entries_changed || pull_report.entries_changed,
        shared: build_shared_status(db, &paths, true, pending_count, corrupt_count, message),
    })
}

#[derive(Debug, Clone, Copy, Default)]
struct PullReport {
    entries_changed: bool,
    corrupt_count: usize,
}

fn pull_remote_operations(db: &InventoryDb, paths: &SharedSyncPaths) -> CommandResult<PullReport> {
    let watermarks = sync_watermarks(db)?;
    let scan_report =
        scan_operation_files_after_watermarks(paths, &watermarks).map_err(sync_core_error)?;
    let mut corrupt_count = record_corrupt_remote_files(db, &scan_report.corrupt)?;
    let mut entries_changed = false;
    let mut applied_count = 0usize;

    for operation in scan_report.operations {
        if db.has_sync_applied_marker(&operation.op_id)? {
            advance_sync_watermark(db, &operation.client_id, operation.local_seq)?;
            continue;
        }

        if let Some(existing_op_id) =
            db.sync_client_seq_marker::<String>(&operation.client_id, operation.local_seq)?
        {
            if existing_op_id != operation.op_id {
                let corrupt = CorruptRemoteFile {
                    path: operation_file_path(paths, &operation.client_id, operation.local_seq)
                        .map(|path| path.to_string_lossy().into_owned())
                        .unwrap_or_else(|_| {
                            format!("{}:{}", operation.client_id, operation.local_seq)
                        }),
                    reason: CorruptRemoteReason::DuplicateSequenceDifferentChecksum,
                    detail: "Remote operation uses an already-applied client_id/local_seq with a different op_id."
                        .to_string(),
                    detected_at_utc: now_timestamp(),
                    content_sha256: Some(operation.checksum.clone()),
                };
                record_corrupt_remote_file(db, &corrupt)?;
                corrupt_count += 1;
                continue;
            }
        }

        if apply_remote_operation(db, &operation)? {
            entries_changed = true;
        }
        mark_operation_applied(db, &operation)?;
        advance_sync_watermark(db, &operation.client_id, operation.local_seq)?;
        applied_count += 1;
    }

    if entries_changed || applied_count > 0 {
        db.flush();
    }

    Ok(PullReport {
        entries_changed,
        corrupt_count,
    })
}

fn apply_remote_operation(
    db: &InventoryDb,
    operation: &SyncOperationEnvelope,
) -> CommandResult<bool> {
    if let Some(current_state) = current_entry_state(db, &operation.entity_id)? {
        if current_state.last_op_id == operation.op_id {
            return Ok(false);
        }
        if let Some(merged) = try_merge_concurrent_field_update(db, operation, &current_state)? {
            return Ok(merged);
        }
        if !operation_wins_state(operation, &current_state) {
            record_stale_operation_conflict(db, operation, &current_state)?;
            return Ok(false);
        }
    }

    let entries_changed = match operation.operation_type {
        SyncOperationType::InventoryEntryDelete => apply_remote_delete(db, operation),
        SyncOperationType::InventoryEntryCreate
        | SyncOperationType::InventoryEntryUpdate
        | SyncOperationType::InventoryEntryVerify
        | SyncOperationType::InventoryEntryArchive => apply_remote_upsert(db, operation),
    }?;

    record_entry_state_for_operation(db, operation)?;
    if entries_changed {
        db.increment_sync_revision()?;
    }

    Ok(entries_changed)
}

fn sync_watermarks(db: &InventoryDb) -> CommandResult<HashMap<String, u64>> {
    let mut watermarks = HashMap::new();
    db.scan_sync_watermarks(usize::MAX, |client_id, local_seq| {
        watermarks.insert(client_id, local_seq);
        Ok(true)
    })?;
    Ok(watermarks)
}

fn advance_sync_watermark(db: &InventoryDb, client_id: &str, local_seq: u64) -> CommandResult<()> {
    let current = db.sync_watermark(client_id)?.unwrap_or(0);
    if local_seq <= current {
        return Ok(());
    }
    if local_seq == current + 1 {
        db.set_sync_watermark(client_id, local_seq)?;
    }
    Ok(())
}

fn try_merge_concurrent_field_update(
    db: &InventoryDb,
    operation: &SyncOperationEnvelope,
    current_state: &super::SyncEntryState,
) -> CommandResult<Option<bool>> {
    if current_state.deleted || operation.operation_type == SyncOperationType::InventoryEntryDelete
    {
        return Ok(None);
    }
    let Some(incoming) = operation.payload.entry.as_ref() else {
        return Ok(None);
    };
    let Some(current) = db.find_entry(&operation.entity_id)? else {
        return Ok(None);
    };
    validate_inventory_entry(incoming)?;
    let mut state = current_state.clone();
    let mut merged = current.clone();
    let fields = if operation.operation_type == SyncOperationType::InventoryEntryCreate
        || operation.payload.changed_fields.is_empty()
    {
        crate::modules::te_storage::entry_changes::ENTRY_FIELDS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        crate::modules::te_storage::entry_changes::normalize_changed_entry_fields(
            operation.payload.changed_fields.clone(),
        )
    };
    let mut accepted = Vec::new();
    for field in fields {
        let (timestamp, op_id) = state
            .field_versions
            .get(&field)
            .map(|version| (version.mutation_ts_utc.as_str(), version.op_id.as_str()))
            .unwrap_or((&current_state.mutation_ts_utc, &current_state.last_op_id));
        let order =
            super::timestamps::compare_timestamp_text(&operation.mutation_ts_utc, timestamp);
        if order.is_gt() || (order.is_eq() && operation.op_id.as_str() > op_id) {
            state.field_versions.insert(
                field.clone(),
                super::types::SyncFieldVersion {
                    mutation_ts_utc: operation.mutation_ts_utc.clone(),
                    op_id: operation.op_id.clone(),
                },
            );
            accepted.push(field);
        }
    }
    if accepted.is_empty() {
        record_stale_operation_conflict(db, operation, current_state)?;
        return Ok(Some(false));
    }
    apply_changed_fields(&mut merged, incoming, &accepted);
    // Two individually valid edits can remove different identifying fields.
    // Resolve that invariant conflict deterministically to the newer valid row.
    if validate_inventory_entry(&merged).is_err() {
        if operation_wins_state(operation, current_state) {
            let changed = apply_remote_upsert(db, operation)?;
            record_entry_state_for_operation(db, operation)?;
            if changed {
                db.increment_sync_revision()?;
            }
            return Ok(Some(changed));
        }
        record_stale_operation_conflict(db, operation, current_state)?;
        return Ok(Some(false));
    }
    merged.updated_at = max_timestamp_text(&current.updated_at, &operation.mutation_ts_utc);
    if operation_wins_state(operation, current_state) {
        state.last_op_id = operation.op_id.clone();
        state.mutation_ts_utc = operation.mutation_ts_utc.clone();
        state.source_client_id = operation.client_id.clone();
        state.source_local_seq = operation.local_seq;
        state.operation_type = operation.operation_type;
        state.base_version = operation.base_version.clone();
    }
    state.changed_fields = state.field_versions.keys().cloned().collect();
    state.changed_fields.sort();
    state.updated_at_utc = now_timestamp();
    state.recovery_entry = Some(merged.clone());
    let changed = current != merged;
    if changed {
        db.put_entry(&merged)?;
    }
    db.put_sync_entry_state(&operation.entity_id, &state)?;
    if changed {
        db.increment_sync_revision()?;
    }
    Ok(Some(changed))
}

fn apply_changed_fields(target: &mut InventoryEntry, source: &InventoryEntry, fields: &[String]) {
    for field in fields {
        match field.as_str() {
            "pn" => target.pn = source.pn.clone(),
            "pr" => target.pr = source.pr.clone(),
            "qty" => target.qty = source.qty,
            "manufacturer" => target.manufacturer = source.manufacturer.clone(),
            "model" => target.model = source.model.clone(),
            "description" => target.description = source.description.clone(),
            "po" => target.po = source.po.clone(),
            "location" => target.location = source.location.clone(),
            "notes" => target.notes = source.notes.clone(),
            "archived" => target.archived = source.archived,
            _ => {}
        }
    }
}

fn apply_remote_delete(db: &InventoryDb, operation: &SyncOperationEnvelope) -> CommandResult<bool> {
    let entry_uuid = operation
        .payload
        .entry_uuid
        .as_deref()
        .unwrap_or(&operation.entity_id);
    let deleted_at_utc = operation
        .payload
        .deleted_at_utc
        .as_deref()
        .unwrap_or(&operation.mutation_ts_utc);

    let tombstone = SyncTombstoneRecord {
        entry_uuid: entry_uuid.to_string(),
        deleted_at_utc: deleted_at_utc.to_string(),
        op_id: operation.op_id.clone(),
        client_id: operation.client_id.clone(),
        local_seq: operation.local_seq,
        base_version: operation.base_version.clone(),
    };
    db.put_sync_tombstone(entry_uuid, &tombstone)?;

    if let Some(entry) = db.find_entry(entry_uuid)? {
        db.delete_entry(&entry)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

fn apply_remote_upsert(db: &InventoryDb, operation: &SyncOperationEnvelope) -> CommandResult<bool> {
    let Some(entry) = operation.payload.entry.clone() else {
        return Ok(false);
    };
    validate_inventory_entry(&entry)?;

    if db.has_sync_tombstone(&operation.entity_id)? {
        db.delete_sync_tombstone(&operation.entity_id)?;
    }

    let entry = prepare_incoming_entry(db, entry)?;
    let changed = db
        .find_entry(&entry.entry_uuid)?
        .map(|existing| existing.updated_at != entry.updated_at || existing != entry)
        .unwrap_or(true);
    db.put_entry(&entry)?;
    bump_next_entry_id_after_remote_entry(db, &entry)?;
    Ok(changed)
}

fn prepare_incoming_entry(
    db: &InventoryDb,
    mut entry: InventoryEntry,
) -> CommandResult<InventoryEntry> {
    if let Some(existing) = db.find_entry(&entry.entry_uuid)? {
        entry.id = existing.id;
        entry.database_id = existing.database_id;
        return Ok(entry);
    }

    if entry.id.trim().is_empty() || local_id_belongs_to_different_entry(db, &entry)? {
        let local_id = reserve_unused_entry_id(db)?;
        entry.id = local_id.to_string();
        entry.database_id = Some(local_id);
    }

    Ok(entry)
}

fn local_id_belongs_to_different_entry(
    db: &InventoryDb,
    entry: &InventoryEntry,
) -> CommandResult<bool> {
    if entry.id.trim().is_empty() {
        return Ok(false);
    }

    Ok(db
        .find_entry(&entry.id)?
        .map(|existing| existing.entry_uuid != entry.entry_uuid)
        .unwrap_or(false))
}

fn reserve_unused_entry_id(db: &InventoryDb) -> CommandResult<i64> {
    loop {
        let candidate = db.next_entry_id()?;
        let candidate_text = candidate.to_string();
        if db.find_entry(&candidate_text)?.is_none() {
            db.set_next_entry_id(candidate + 1)?;
            return Ok(candidate);
        }
        db.set_next_entry_id(candidate + 1)?;
    }
}

fn bump_next_entry_id_after_remote_entry(
    db: &InventoryDb,
    entry: &InventoryEntry,
) -> CommandResult<()> {
    let entry_id = numeric_id(&entry.id);
    if entry_id > 0 && entry_id >= db.next_entry_id()? {
        db.set_next_entry_id(entry_id + 1)?;
    }
    Ok(())
}

pub(super) fn mark_operation_applied(
    db: &InventoryDb,
    operation: &SyncOperationEnvelope,
) -> CommandResult<()> {
    let marker = SyncAppliedMarker {
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
