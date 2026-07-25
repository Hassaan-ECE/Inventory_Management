mod commit;
mod parser;
mod reconcile;
mod types;

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{model::CommandResult, store::InventoryDb};

pub(crate) use commit::commit_calibration_roster_from_store;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use commit::commit_calibration_roster_with_test_failure_after;
#[cfg(test)]
#[allow(unused_imports)]
pub(crate) use types::{
    CalibrationRosterClassification, CalibrationRosterResolution,
    CalibrationRosterResolutionAction, CalibrationRosterRowOutcome, CalibrationRosterSemanticFlag,
};
#[allow(unused_imports)]
pub(crate) use types::{
    CalibrationRosterCommitInput, CalibrationRosterCommitResult, CalibrationRosterPreviewReport,
};

pub(crate) const CALIBRATION_ROSTER_FILE_EXTENSIONS: &[&str] = &["xlsx"];
pub(crate) const CALIBRATION_ROSTER_MAPPING_VERSION: &str = "te-calibration-roster-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct StoredCalibrationRosterPreview {
    report: CalibrationRosterPreviewReport,
    source_path: String,
}

pub(crate) fn preview_calibration_roster_from_path(
    path: &Path,
    db: &InventoryDb,
) -> CommandResult<CalibrationRosterPreviewReport> {
    let parsed = parser::parse_roster_source(path)?;
    let report = reconcile::reconcile_roster_source(&parsed, db)?;
    db.put_calibration_roster_preview(
        &report.batch_id,
        &StoredCalibrationRosterPreview {
            report: report.clone(),
            source_path: path.to_string_lossy().into_owned(),
        },
    )?;
    Ok(report)
}
