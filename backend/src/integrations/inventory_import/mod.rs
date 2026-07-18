mod commit;
mod mapping;
mod parser;
mod reconcile;

use std::{collections::BTreeMap, path::Path};

use serde::{Deserialize, Serialize};

use crate::{model::CommandResult, store::InventoryDb};

pub(crate) use commit::commit_import_from_store;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use commit::commit_import_with_test_failure_after;

pub(crate) const IMPORT_FILE_EXTENSIONS: &[&str] = &["csv", "xlsx", "xls"];
pub(crate) const IMPORT_MAPPING_VERSION: &str = "te-test-equipment-v2";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImportClassification {
    Inserted,
    Matched,
    Conflicted,
    Rejected,
    Ignored,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ImportColumnTreatment {
    Mapped,
    IntentionallyIgnored,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportColumnReport {
    pub original_header: String,
    pub normalized_target: Option<String>,
    pub treatment: ImportColumnTreatment,
    pub nonblank_count: usize,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportRowOutcome {
    pub source_row: u64,
    pub classification: ImportClassification,
    pub issues: Vec<String>,
    pub original_id: Option<String>,
    pub original_asset_number: Option<String>,
    pub original_serial_number: Option<String>,
    pub candidate_entry_uuid: Option<String>,
    pub raw_values: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportDryRunReport {
    pub batch_id: String,
    pub source_fingerprint: String,
    pub source_filename: String,
    pub selected_sheet: String,
    pub mapping_version: String,
    pub total_rows: usize,
    pub inserted: usize,
    pub matched: usize,
    pub conflicted: usize,
    pub rejected: usize,
    pub ignored: usize,
    pub columns: Vec<ImportColumnReport>,
    pub row_outcomes: Vec<ImportRowOutcome>,
    pub blocking: bool,
    pub reconciliation_basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportCommitInput {
    pub batch_id: String,
    pub confirmed: bool,
    /// Internal test capability. The v0.1 desktop command rejects partial requests.
    #[serde(default)]
    pub allow_partial: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportCommitResult {
    pub batch_id: String,
    pub inserted: usize,
    pub matched: usize,
    pub conflicted: usize,
    pub rejected: usize,
    pub ignored: usize,
    pub remaining: usize,
    pub noop: usize,
    pub entries_changed: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredImportPreview {
    report: ImportDryRunReport,
    source_path: String,
}

pub(crate) fn preview_import_from_path(
    path: &Path,
    db: &InventoryDb,
) -> CommandResult<ImportDryRunReport> {
    let parsed = parser::parse_source(path)?;
    let reconciled = reconcile::reconcile_source(&parsed, db, None)?;
    let report = reconciled.report;
    let stored = StoredImportPreview {
        report: report.clone(),
        source_path: path.to_string_lossy().into_owned(),
    };
    db.put_import_preview(&report.batch_id, &stored)?;
    Ok(report)
}
