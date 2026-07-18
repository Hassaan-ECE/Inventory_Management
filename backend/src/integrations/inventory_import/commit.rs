use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    api::mutations::create_imported_entry_in_store,
    model::{CommandResult, ImportProvenance},
    store::InventoryDb,
};

use super::{
    parser::parse_source, reconcile::reconcile_source, ImportClassification, ImportCommitInput,
    ImportCommitResult, StoredImportPreview, IMPORT_MAPPING_VERSION,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ImportRowMarker {
    classification: ImportClassification,
    entry_uuid: Option<String>,
}

pub(crate) fn commit_import_from_store(
    input: ImportCommitInput,
    db: &InventoryDb,
) -> CommandResult<ImportCommitResult> {
    commit_import(input, db, None)
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn commit_import_with_test_failure_after(
    input: ImportCommitInput,
    db: &InventoryDb,
    completed_mutations: usize,
) -> CommandResult<ImportCommitResult> {
    commit_import(input, db, Some(completed_mutations))
}

fn commit_import(
    input: ImportCommitInput,
    db: &InventoryDb,
    inject_after_mutations_before_marker: Option<usize>,
) -> CommandResult<ImportCommitResult> {
    if !input.confirmed {
        return Err("Import commit requires explicit confirmation.".to_string());
    }
    if let Some(completed) = db.completed_import::<ImportCommitResult>(&input.batch_id)? {
        return Ok(ImportCommitResult {
            batch_id: completed.batch_id,
            inserted: 0,
            matched: completed.matched,
            conflicted: completed.conflicted,
            rejected: completed.rejected,
            ignored: completed.ignored,
            remaining: 0,
            noop: completed.inserted + completed.noop,
            entries_changed: false,
            message:
                "Import batch was already completed; no entries or outbox operations were added."
                    .to_string(),
        });
    }

    let stored = db
        .import_preview::<StoredImportPreview>(&input.batch_id)?
        .ok_or_else(|| {
            "Import dry-run batch was not found. Preview the source again.".to_string()
        })?;
    if stored.report.batch_id != input.batch_id {
        return Err("Stored import batch identity does not match the requested batch.".to_string());
    }
    let parsed = parse_source(std::path::Path::new(&stored.source_path))?;
    if parsed.fingerprint != stored.report.source_fingerprint {
        return Err("Import source changed after dry run; preview it again.".to_string());
    }
    if parsed.selected_sheet != stored.report.selected_sheet
        || stored.report.mapping_version != IMPORT_MAPPING_VERSION
    {
        return Err(
            "Import sheet or mapping version changed after dry run; preview it again.".to_string(),
        );
    }
    let reconciled = reconcile_source(&parsed, db, Some(&input.batch_id))?;
    if reconciled.report.batch_id != input.batch_id {
        return Err("Import batch identity is stale; preview the source again.".to_string());
    }
    if reconciled.report.reconciliation_basis != stored.report.reconciliation_basis {
        return Err(
            "Import reconciliation basis changed after dry run; preview it again.".to_string(),
        );
    }
    if reconciled.report.blocking && !input.allow_partial {
        return Err(
            "Import dry run is blocking because conflicted or rejected rows remain. Correct the source and run the dry run again before committing.".to_string(),
        );
    }

    let mut inserted = 0usize;
    let mut mutation_count = 0usize;
    let mut recovered = 0usize;
    let mut skipped_blocking = 0usize;
    for row in &reconciled.rows {
        if db.has_import_row_marker(&input.batch_id, row.source_row)? {
            continue;
        }
        match row.classification {
            ImportClassification::Inserted => {
                let entry_uuid = deterministic_entry_uuid(
                    &input.batch_id,
                    &reconciled.report.selected_sheet,
                    row.source_row,
                );
                let provenance = ImportProvenance {
                    batch_id: input.batch_id.clone(),
                    source_filename: reconciled.report.source_filename.clone(),
                    source_sheet: Some(reconciled.report.selected_sheet.clone()),
                    source_row: row.source_row,
                    original_id: row.original_id.clone(),
                    original_asset_number: row.original_asset_number.clone(),
                    original_serial_number: row.original_serial_number.clone(),
                };
                if let Some(existing) = db.find_import_entry_by_uuid(&entry_uuid)? {
                    if existing.import_provenance.as_ref() != Some(&provenance) {
                        return Err(format!(
                            "Deterministic UUID collision at source row {}.",
                            row.source_row
                        ));
                    }
                    db.mark_import_row_complete(
                        &input.batch_id,
                        row.source_row,
                        &ImportRowMarker {
                            classification: row.classification,
                            entry_uuid: Some(entry_uuid),
                        },
                    )?;
                    recovered += 1;
                    continue;
                }
                let entry_input = row.input.clone().ok_or_else(|| {
                    format!(
                        "Inserted source row {} has no validated input.",
                        row.source_row
                    )
                })?;
                create_imported_entry_in_store(entry_input, entry_uuid.clone(), provenance, db)?;
                inserted += 1;
                mutation_count += 1;
                if inject_after_mutations_before_marker == Some(mutation_count) {
                    return Err(format!(
                        "injected import failure after entry and outbox mutation but before row marker at source row {}",
                        row.source_row
                    ));
                }
                db.mark_import_row_complete(
                    &input.batch_id,
                    row.source_row,
                    &ImportRowMarker {
                        classification: row.classification,
                        entry_uuid: Some(entry_uuid),
                    },
                )?;
            }
            ImportClassification::Matched | ImportClassification::Ignored => {
                db.mark_import_row_complete(
                    &input.batch_id,
                    row.source_row,
                    &ImportRowMarker {
                        classification: row.classification,
                        entry_uuid: None,
                    },
                )?;
            }
            ImportClassification::Conflicted | ImportClassification::Rejected => {
                if input.allow_partial {
                    skipped_blocking += 1;
                    continue;
                }
                return Err("Blocking import row reached commit unexpectedly.".to_string());
            }
        }
    }

    let mut remaining = 0usize;
    for row in &reconciled.rows {
        if !db.has_import_row_marker(&input.batch_id, row.source_row)? {
            remaining += 1;
        }
    }
    let result = ImportCommitResult {
        batch_id: input.batch_id.clone(),
        inserted,
        matched: reconciled.report.matched,
        conflicted: reconciled.report.conflicted,
        rejected: reconciled.report.rejected,
        ignored: reconciled.report.ignored,
        remaining,
        noop: reconciled.report.matched + reconciled.report.ignored + recovered,
        entries_changed: inserted > 0,
        message: if input.allow_partial && skipped_blocking > 0 {
            if inserted > 0 {
                format!(
                    "Partial import: {inserted} new entries written. Skipped {skipped_blocking} conflicted/rejected rows (not imported)."
                )
            } else {
                format!(
                    "Partial import: no new entries. Skipped {skipped_blocking} conflicted/rejected rows."
                )
            }
        } else if inserted > 0 {
            format!(
                "Imported {inserted} new entries. Matched and ignored rows were recorded as no-op rows."
            )
        } else {
            "No new entries were added. Matched and ignored rows were recorded as durable no-op rows."
                .to_string()
        },
    };
    // Full completion only when every row is marked (partial leaves conflicted/rejected unmarked).
    if remaining == 0 {
        db.mark_import_completed(&input.batch_id, &result)?;
    }
    Ok(result)
}

fn deterministic_entry_uuid(batch_id: &str, sheet: &str, source_row: u64) -> String {
    let digest = Sha256::digest(format!("{batch_id}\0{sheet}\0{source_row}").as_bytes());
    digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
