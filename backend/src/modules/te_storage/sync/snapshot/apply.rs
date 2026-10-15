use std::collections::{HashMap, HashSet};

use crate::modules::te_storage::{
    model::CommandResult,
    store::{InventoryDb, SyncKeyspace},
};

use super::super::{
    conflicts::{current_entry_state, state_wins_state},
    SharedSyncPaths, SyncEntryState, SyncTombstoneRecord, BOOTSTRAP_COMPLETE_KEY,
};
use super::{
    io::{corruption_count_from_error, read_manifest, read_verified_snapshot, watermarks_to_map},
    types::{SharedInventorySnapshot, SnapshotApplyReport, SNAPSHOT_APPLY_PENDING_KEY},
};

/// Brings the local store up to the latest shared snapshot.
///
/// An empty store adopts the snapshot wholesale. A store with data merges it entry by
/// entry (newest state wins), so a snapshot that lacks some of this client's knowledge
/// can never strand either side: before 0.1.4 such snapshots were skipped entirely,
/// while their publisher compacted the operation files that held the missing work.
pub(crate) fn apply_latest_snapshot_if_safe(
    db: &InventoryDb,
    paths: &SharedSyncPaths,
    pending_count: usize,
) -> CommandResult<SnapshotApplyReport> {
    if pending_count > 0 {
        return Ok(SnapshotApplyReport::default());
    }

    let manifest = match read_manifest(paths) {
        Ok(Some(manifest)) => manifest,
        Ok(None) => return Ok(SnapshotApplyReport::default()),
        Err(error) => {
            return Ok(SnapshotApplyReport {
                entries_changed: false,
                corrupt_count: corruption_count_from_error(error),
            });
        }
    };

    if db.last_snapshot_id()?.as_deref() == Some(manifest.snapshot_id.as_str()) {
        return Ok(SnapshotApplyReport::default());
    }

    let snapshot = match read_verified_snapshot(paths, &manifest) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            return Ok(SnapshotApplyReport {
                entries_changed: false,
                corrupt_count: corruption_count_from_error(error),
            });
        }
    };

    db.put_sync_value(SNAPSHOT_APPLY_PENDING_KEY, snapshot.snapshot_id.as_bytes())?;
    db.flush();

    let entries_changed = if db.has_entries()? {
        merge_from_snapshot(db, &snapshot)?
    } else {
        replace_from_snapshot(db, &snapshot)?
    };
    db.put_sync_value(BOOTSTRAP_COMPLETE_KEY, snapshot.created_at_utc.as_bytes())?;
    db.set_last_snapshot_id(&snapshot.snapshot_id)?;
    if entries_changed {
        db.increment_sync_revision()?;
    }
    db.delete_sync_value(SNAPSHOT_APPLY_PENDING_KEY)?;
    db.flush();

    Ok(SnapshotApplyReport {
        entries_changed,
        corrupt_count: 0,
    })
}
pub(super) fn replace_from_snapshot(
    db: &InventoryDb,
    snapshot: &SharedInventorySnapshot,
) -> CommandResult<bool> {
    db.set_sync_schema_version(snapshot.sync_schema_version.into())?;
    let entries_changed = db.replace_entries_snapshot(&snapshot.entries)?;

    db.clear_sync_keyspace(SyncKeyspace::Tombstone)?;
    for tombstone in &snapshot.tombstones {
        db.put_sync_tombstone(&tombstone.entry_uuid, tombstone)?;
    }

    db.clear_sync_keyspace(SyncKeyspace::EntryState)?;
    for state in &snapshot.entry_states {
        db.put_sync_entry_state(&state.entry_uuid, state)?;
    }

    db.clear_sync_keyspace(SyncKeyspace::Watermark)?;
    for watermark in &snapshot.watermarks {
        if watermark.local_seq > 0 {
            db.set_sync_watermark(&watermark.client_id, watermark.local_seq)?;
        }
    }

    Ok(entries_changed)
}

/// Merges a snapshot into a non-empty store without losing either side's newer work.
///
/// - Entries the snapshot knows about: the newer entry state wins (same ordering as
///   remote operations).
/// - Entries only this client knows about: kept unless the snapshot already covers the
///   operation that produced them, in which case the snapshot is authoritative and the
///   entry is dropped.
/// - Watermarks: per-client maximum of local and snapshot.
fn merge_from_snapshot(
    db: &InventoryDb,
    snapshot: &SharedInventorySnapshot,
) -> CommandResult<bool> {
    db.set_sync_schema_version(snapshot.sync_schema_version.into())?;
    let snapshot_watermarks = watermarks_to_map(&snapshot.watermarks);
    let entries: HashMap<&str, _> = snapshot
        .entries
        .iter()
        .map(|entry| (entry.entry_uuid.as_str(), entry))
        .collect();
    let tombstones: HashMap<&str, &SyncTombstoneRecord> = snapshot
        .tombstones
        .iter()
        .map(|tombstone| (tombstone.entry_uuid.as_str(), tombstone))
        .collect();

    let mut snapshot_states: HashMap<String, SyncEntryState> = snapshot
        .entry_states
        .iter()
        .map(|state| (state.entry_uuid.clone(), state.clone()))
        .collect();
    for tombstone in &snapshot.tombstones {
        snapshot_states
            .entry(tombstone.entry_uuid.clone())
            .or_insert_with(|| tombstone_state(tombstone));
    }

    let mut changed = false;
    for (entry_uuid, incoming) in &snapshot_states {
        let local = current_entry_state(db, entry_uuid)?;
        let take = match &local {
            None => true,
            Some(local) => {
                local.last_op_id != incoming.last_op_id && state_wins_state(incoming, local)
            }
        };
        if !take {
            continue;
        }

        if incoming.deleted {
            if let Some(existing) = db.find_entry(entry_uuid)? {
                db.delete_entry(&existing)?;
                changed = true;
            }
            let tombstone = tombstones
                .get(entry_uuid.as_str())
                .map(|tombstone| (*tombstone).clone())
                .unwrap_or_else(|| state_tombstone(incoming));
            db.put_sync_tombstone(entry_uuid, &tombstone)?;
        } else {
            let Some(entry) = entries.get(entry_uuid.as_str()) else {
                continue;
            };
            if db.find_entry(entry_uuid)?.as_ref() != Some(*entry) {
                db.put_entry(entry)?;
                changed = true;
            }
            db.delete_sync_tombstone(entry_uuid)?;
        }
        db.put_sync_entry_state(entry_uuid, incoming)?;
    }

    let known: HashSet<&str> = snapshot_states.keys().map(String::as_str).collect();
    let mut local_only = Vec::new();
    db.scan_sync_entry_states::<SyncEntryState, _>(usize::MAX, |entry_uuid, state| {
        if !known.contains(entry_uuid.as_str()) {
            local_only.push(state);
        }
        Ok(true)
    })?;
    for state in local_only {
        let covered = snapshot_watermarks
            .get(&state.source_client_id)
            .is_some_and(|watermark| state.source_local_seq <= *watermark);
        if !covered {
            continue;
        }
        if let Some(existing) = db.find_entry(&state.entry_uuid)? {
            db.delete_entry(&existing)?;
            changed = true;
        }
        db.delete_sync_tombstone(&state.entry_uuid)?;
        db.delete_sync_entry_state(&state.entry_uuid)?;
    }

    for (client_id, local_seq) in snapshot_watermarks {
        if db.sync_watermark(&client_id)?.unwrap_or(0) < local_seq {
            db.set_sync_watermark(&client_id, local_seq)?;
        }
    }

    if changed {
        let max_id = db
            .load_entries()?
            .iter()
            .filter_map(|entry| entry.id.parse::<i64>().ok())
            .max()
            .unwrap_or(0);
        if db.next_entry_id()? <= max_id {
            db.set_next_entry_id(max_id + 1)?;
        }
    }

    Ok(changed)
}

fn tombstone_state(tombstone: &SyncTombstoneRecord) -> SyncEntryState {
    SyncEntryState {
        entry_uuid: tombstone.entry_uuid.clone(),
        last_op_id: tombstone.op_id.clone(),
        mutation_ts_utc: tombstone.deleted_at_utc.clone(),
        deleted: true,
        base_version: tombstone.base_version.clone(),
        changed_fields: Vec::new(),
        source_client_id: tombstone.client_id.clone(),
        source_local_seq: tombstone.local_seq,
        operation_type: super::super::SyncOperationType::InventoryEntryDelete,
        field_versions: Default::default(),
        recovery_entry: None,
        updated_at_utc: tombstone.deleted_at_utc.clone(),
    }
}

fn state_tombstone(state: &SyncEntryState) -> SyncTombstoneRecord {
    SyncTombstoneRecord {
        entry_uuid: state.entry_uuid.clone(),
        deleted_at_utc: state.mutation_ts_utc.clone(),
        op_id: state.last_op_id.clone(),
        client_id: state.source_client_id.clone(),
        local_seq: state.source_local_seq,
        base_version: state.base_version.clone(),
    }
}
