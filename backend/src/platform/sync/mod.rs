mod apply;
mod auth;
mod conflicts;
mod identity;
mod operation_file;
mod queue;
mod record;
mod recovery;
mod scanning;
mod shared_paths;
mod snapshot;
mod timestamps;
mod types;

pub(crate) use self::apply::{publish_pending_local_changes, run_shared_sync};
pub(crate) use self::auth::{sign_canonical_bytes, verify_canonical_bytes};
pub(crate) use self::queue::{queue_delete_operation, queue_entry_operation};
pub(crate) use self::record::{MergeRule, SyncRecord};
pub(crate) use self::recovery::{last_local_recovery_message, recover_local_sync_state};
pub(crate) use self::shared_paths::{
    queued_local_status, resolved_shared_sync_paths, shared_inventory_status,
    startup_inventory_status,
};
pub(crate) use self::types::SyncOperationType;

/// Engine internals the sync tests reach into; each test uses a different subset.
#[cfg(test)]
#[allow(clippy::items_after_test_module, unused_imports)]
pub(crate) mod test_support {
    pub(crate) use super::apply::run_shared_sync_with_root;
    pub(crate) use super::auth::set_test_hmac_key;
    pub(crate) use super::conflicts::{
        corrupt_remote_record_id, record_corrupt_remote_file, record_corrupt_remote_files,
    };
    pub(crate) use super::identity::{
        allocate_local_sequence, get_or_create_client_identity, peek_next_local_sequence,
    };
    pub(crate) use super::operation_file::{
        canonical_operation_checksum, canonical_operation_json, operation_file_name,
        operation_file_path, read_operation_file, read_operation_file_for_identity, sha256_hex,
        write_operation_file,
    };
    pub(crate) use super::queue::{
        build_delete_operation, build_entry_operation, fail_next_outbox_write,
    };
    pub(crate) use super::scanning::{scan_operation_files, scan_operation_files_after_watermarks};
    pub(crate) use super::shared_paths::{
        ensure_operation_log_layout, resolve_shared_root, resolve_shared_root_from_env_value,
    };
    pub(crate) use super::snapshot::{
        apply_latest_snapshot_if_safe, maybe_publish_snapshot, SharedInventoryManifest,
        SharedInventorySnapshot, SnapshotApplyReport, SnapshotPublishReport, SnapshotWatermark,
        SNAPSHOT_APPLY_PENDING_KEY,
    };
    pub(crate) use super::types::{
        CorruptRemoteFile, CorruptRemoteReason, OperationScanReport, SharedSyncPaths,
        SharedSyncRunResult, SyncAppliedMarker, SyncClientIdentity, SyncConflictReason,
        SyncConflictRecord, SyncCoreError, SyncCoreErrorKind, SyncCoreResult, SyncEntryState,
        SyncOperationEnvelope, SyncOperationPayload, SyncOperationType, SyncTombstoneRecord,
        SHARED_SYNC_ENABLED_ENV,
    };

    /// The JSON keys an entry state of record type `R` serializes with.
    pub(crate) fn entry_state_keys<R: super::SyncRecord>() -> Vec<String> {
        let state = SyncEntryState::<R> {
            entry_uuid: "entry-1".to_string(),
            last_op_id: "op-1".to_string(),
            mutation_ts_utc: "2026-10-16T00:00:00.000Z".to_string(),
            deleted: false,
            base_version: None,
            changed_fields: Vec::new(),
            source_client_id: "client-1".to_string(),
            source_local_seq: 1,
            operation_type: SyncOperationType::InventoryEntryUpdate,
            updated_at_utc: "2026-10-16T00:00:00.000Z".to_string(),
            field_versions: Default::default(),
            recovery_entry: None,
        };
        let serde_json::Value::Object(fields) = serde_json::to_value(&state).unwrap() else {
            panic!("entry state serializes as an object");
        };
        fields.keys().cloned().collect()
    }
}

use self::types::{
    CorruptRemoteFile, CorruptRemoteReason, OperationScanReport, SharedSyncPaths,
    SharedSyncRunResult, SyncAppliedMarker, SyncClientIdentity, SyncConflictReason,
    SyncConflictRecord, SyncCoreError, SyncCoreErrorKind, SyncCoreResult, SyncEntryState,
    SyncOperationEnvelope, SyncOperationPayload, SyncTombstoneRecord, BOOTSTRAP_COMPLETE_KEY,
    CHECKSUM_PREFIX, LOCAL_SEQ_WIDTH, MAX_LOCAL_SEQ, OP_FILE_SUFFIX, OP_TEMP_MARKER,
    SHARED_SYNC_ENABLED_ENV,
};
