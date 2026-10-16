//! The footer warning: problems the last sync found, plus the health of the local database.

use serde::{Deserialize, Serialize};

use crate::platform::{
    common::{CommandResult, SyncWarning},
    store::ModuleStore,
};

const FINDINGS_KEY: &str = "meta:last_sync_findings";
/// Warn while there is still plenty of room to act on it.
const NEARLY_FULL_PERCENT: u8 = 80;

/// What the last full sync found that the user should know about. Kept in the database so
/// the warning stays up between syncs instead of clearing on the next local change.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SyncFindings {
    /// Shared files that could not be read and were skipped.
    pub skipped_files: usize,
    /// Of those, files written by a newer schema than this version understands.
    pub newer_schema_files: usize,
    /// A newer app version that a teammate published a snapshot with.
    pub newer_team_version: Option<String>,
}

pub(crate) fn record_sync_findings<R>(
    db: &ModuleStore<R>,
    findings: &SyncFindings,
) -> CommandResult<()> {
    let bytes = serde_json::to_vec(findings).map_err(|error| error.to_string())?;
    db.put_sync_value(FINDINGS_KEY, &bytes)
}

/// The most important warning for this inventory, if any.
pub(crate) fn sync_warning<R>(db: &ModuleStore<R>) -> Option<SyncWarning> {
    let health = db.database_health();
    if health.write_failures > 0 {
        return Some(SyncWarning::LocalSaveFailing);
    }
    if health.percent_used >= NEARLY_FULL_PERCENT {
        return Some(SyncWarning::DatabaseNearlyFull {
            percent_used: health.percent_used,
        });
    }
    let findings = db
        .get_sync_value(FINDINGS_KEY)
        .ok()
        .flatten()
        .and_then(|bytes| serde_json::from_slice::<SyncFindings>(&bytes).ok())
        .unwrap_or_default();
    if findings.newer_team_version.is_some() || findings.newer_schema_files > 0 {
        return Some(SyncWarning::UpdateAvailable {
            version: findings.newer_team_version,
        });
    }
    (findings.skipped_files > 0).then_some(SyncWarning::SkippedFiles {
        count: findings.skipped_files,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::te_storage::store::InventoryDb;
    use std::env;
    use uuid::Uuid;

    fn test_db() -> InventoryDb {
        let path = env::temp_dir()
            .join(format!("sync-warning-{}", Uuid::new_v4().simple()))
            .join("warning.feox");
        InventoryDb::open_at(path).unwrap()
    }

    #[test]
    fn a_clean_sync_has_no_warning() {
        let db = test_db();
        record_sync_findings(&db, &SyncFindings::default()).unwrap();
        assert_eq!(sync_warning(&db), None);
    }

    #[test]
    fn newer_files_ask_for_an_update_before_reporting_skipped_files() {
        let db = test_db();
        record_sync_findings(
            &db,
            &SyncFindings {
                skipped_files: 3,
                newer_schema_files: 1,
                newer_team_version: None,
            },
        )
        .unwrap();
        assert_eq!(
            sync_warning(&db),
            Some(SyncWarning::UpdateAvailable { version: None })
        );

        record_sync_findings(
            &db,
            &SyncFindings {
                skipped_files: 2,
                ..SyncFindings::default()
            },
        )
        .unwrap();
        assert_eq!(
            sync_warning(&db),
            Some(SyncWarning::SkippedFiles { count: 2 })
        );
    }
}
