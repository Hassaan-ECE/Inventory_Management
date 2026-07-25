use serde::{Deserialize, Serialize};

use crate::model::{CalibrationRequirement, InventoryEntryInput};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CalibrationRosterClassification {
    MatchedUpdate,
    CreateCandidate,
    ConflictReviewRequired,
    DuplicateSourceRow,
    IgnoredJunk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CalibrationRosterSemanticFlag {
    NormalDated,
    ReferenceOnly,
    NeedsCalibration,
    OutToCalibration,
    FailedCalibration,
    InactiveOrScrapped,
    Unclear,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationRosterCandidateContext {
    pub entry_uuid: String,
    pub id: String,
    pub asset_number: String,
    pub serial_number: String,
    pub manufacturer: String,
    pub model: String,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationRosterFieldChange {
    pub field: String,
    pub before: Option<String>,
    pub after: Option<String>,
    pub destructive: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationRosterRowOutcome {
    pub source_sheet: String,
    pub source_row: u64,
    pub classification: CalibrationRosterClassification,
    pub issues: Vec<String>,
    pub ignored_identity_placeholders: Vec<String>,
    pub asset_number: Option<String>,
    pub serial_number: Option<String>,
    pub manufacturer: Option<String>,
    pub model: Option<String>,
    pub description: Option<String>,
    pub candidate_entry_uuid: Option<String>,
    pub candidate_entries: Vec<CalibrationRosterCandidateContext>,
    pub proposed_requirement: CalibrationRequirement,
    pub proposed_out_to_calibration: bool,
    pub proposed_input: Option<InventoryEntryInput>,
    pub changes: Vec<CalibrationRosterFieldChange>,
    pub semantic_flags: Vec<CalibrationRosterSemanticFlag>,
    pub requires_review: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationRosterAbsentEntry {
    pub entry: CalibrationRosterCandidateContext,
    pub calibration_due_at: Option<String>,
    pub calibration_vendor: Option<String>,
    pub calibration_notes: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationRosterCounts {
    pub matched_updates: usize,
    pub create_candidates: usize,
    pub conflicts: usize,
    pub duplicate_source_rows: usize,
    pub ignored_junk: usize,
    pub current_required_absent: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationRosterPreviewReport {
    pub batch_id: String,
    pub source_fingerprint: String,
    pub source_filename: String,
    pub mapping_version: String,
    pub contributing_sheets: Vec<String>,
    pub total_source_rows: usize,
    pub counts: CalibrationRosterCounts,
    pub row_outcomes: Vec<CalibrationRosterRowOutcome>,
    pub current_required_absent: Vec<CalibrationRosterAbsentEntry>,
    pub prospective_required_count: usize,
    pub reconciliation_basis: String,
    pub blocking: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CalibrationRosterResolutionAction {
    UseExisting,
    Create,
    Ignore,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationRosterResolution {
    pub source_sheet: String,
    pub source_row: u64,
    pub action: CalibrationRosterResolutionAction,
    pub target_entry_uuid: Option<String>,
    pub create_input: Option<InventoryEntryInput>,
    #[serde(default)]
    pub confirmed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationRosterCommitInput {
    pub batch_id: String,
    pub confirmed: bool,
    pub replace_active_required_roster: bool,
    pub verification_attribution: String,
    #[serde(default)]
    pub resolutions: Vec<CalibrationRosterResolution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CalibrationRosterCommitResult {
    pub batch_id: String,
    pub updated: usize,
    pub created: usize,
    pub reset: usize,
    pub ignored: usize,
    pub noop: usize,
    pub final_required: usize,
    pub entries_changed: bool,
    pub message: String,
}
