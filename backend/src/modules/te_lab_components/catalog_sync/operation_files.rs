//! Reading, writing, and validating catalog operation files
//! (`ops\<client id>\<sequence>.op.json`).

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process,
};

use uuid::Uuid;

use super::{
    canonical_json, checksum, signing_bytes,
    types::{CatalogCorruptRemoteRecord, CatalogOperationEnvelope, CatalogOperationType},
    CatalogSharedSyncPaths, CATALOG_SYNC_SCHEMA_VERSION,
};
use crate::{
    modules::te_lab_components::store::InventoryDb,
    platform::{
        common::{db_error, now_timestamp, CommandResult},
        sync::{
            operation_file_name, parse_operation_file_name as parse_sequence, sha256_hex,
            verify_canonical_bytes, write_file_atomically, CHECKSUM_PREFIX,
        },
    },
};

pub(super) fn write_operation_file(
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
    let bytes = canonical_json(operation)?;
    write_file_atomically(&temp_path, &final_path, &bytes, false).map_err(db_error)?;
    Ok(final_path)
}

pub(super) fn read_operation_file(
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

pub(super) fn validate_operation(operation: &CatalogOperationEnvelope) -> CommandResult<()> {
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
    let expected_checksum = checksum(operation)?;
    if operation.checksum != expected_checksum {
        return Err("Catalog operation checksum does not match canonical content.".to_string());
    }
    verify_canonical_bytes(
        "catalog.sync.operation.v2",
        &signing_bytes(operation)?,
        operation.auth.as_deref(),
    )?;
    Ok(())
}

pub(super) fn operation_file_matches(
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

pub(super) fn scan_operation_files(
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

pub(super) fn record_corrupt_remote(
    db: &InventoryDb,
    path: &Path,
    detail: String,
) -> CommandResult<()> {
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

pub(super) fn operation_file_path(
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

/// The sequence number in an operation file name; catalog sequences start at 1.
pub(super) fn parse_operation_file_name(file_name: &str) -> Option<u64> {
    parse_sequence(file_name)
        .ok()
        .filter(|sequence| *sequence > 0)
}

pub(super) fn validate_path_segment(label: &str, value: &str) -> CommandResult<()> {
    if value.trim().is_empty()
        || value.contains(['/', '\\', ':'])
        || value.chars().any(char::is_control)
    {
        return Err(format!("Catalog sync {label} is not a safe path segment."));
    }
    Ok(())
}
