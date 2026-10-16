//! Types and helpers every inventory module shares.

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

/// Commands report failures to the frontend as plain messages.
pub(crate) type CommandResult<T> = Result<T, String>;

/// Shared-folder sync state shown in the footer and returned with every load, query, and
/// mutation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventorySharedStatus {
    pub available: bool,
    pub can_modify: bool,
    pub enabled: bool,
    pub has_local_only_changes: Option<bool>,
    pub message: String,
    pub mutation_mode: String,
    pub revision: Option<String>,
    pub last_snapshot_id: Option<String>,
    pub shared_root_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub warning: Option<SyncWarning>,
}

/// A problem the footer flags beside the status message, most serious first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub(crate) enum SyncWarning {
    /// The local database is failing to write, so recent changes may be lost on restart.
    LocalSaveFailing,
    /// The local database file is nearly full.
    DatabaseNearlyFull { percent_used: u8 },
    /// A teammate runs a newer version, or wrote shared files this version cannot read.
    UpdateAvailable { version: Option<String> },
    /// Shared files that could not be read were skipped.
    SkippedFiles { count: usize },
}

pub(crate) fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

/// Display ids are numeric strings; anything else sorts as 0.
pub(crate) fn numeric_id(id: &str) -> i64 {
    id.parse::<i64>().unwrap_or(0)
}

/// UTC RFC 3339 with milliseconds, the format every stored timestamp uses.
pub(crate) fn now_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

/// What a workbook export wrote.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExcelExportStats {
    pub archived_count: usize,
    pub inventory_count: usize,
    pub output_path: String,
    pub total_count: usize,
}
