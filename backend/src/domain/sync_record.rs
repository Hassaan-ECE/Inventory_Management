//! How TE Test Equipment entries plug into the shared sync engine.

use super::model::{validate_inventory_entry, CommandResult, InventoryEntry};
use crate::platform::sync::{MergeRule, SyncRecord};

impl SyncRecord for InventoryEntry {
    const LABEL: &'static str = "TE";
    const SHARED_ROOT_ENV: &'static str = "INVENTORY_MANAGEMENT_SHARED_ROOT";
    const SYNC_SCHEMA_VERSION: u16 = 2;
    const ENTITY_TYPE: &'static str = "inventory_entry";
    const OPERATION_AUTH_DOMAIN: &'static str = "sync.operation.v1";
    const MERGE_RULE: MergeRule = MergeRule::DisjointFieldsFromSameBase;
    const ENTRY_FIELDS: &'static [&'static str] = &[
        "asset_number",
        "serial_number",
        "qty",
        "manufacturer",
        "model",
        "description",
        "project_name",
        "location",
        "assigned_to",
        "links",
        "notes",
        "lifecycle_status",
        "working_status",
        "condition",
        "calibration_requirement",
        "out_to_calibration",
        "last_calibrated_at",
        "calibration_due_at",
        "calibration_interval_months",
        "certificate_ref",
        "calibration_vendor",
        "calibration_notes",
        "verified_at",
        "verified_by",
        "archived",
        "picture_path",
    ];

    fn validate(&self) -> CommandResult<()> {
        validate_inventory_entry(self)
    }

    fn normalize_changed_fields(fields: &[String]) -> Vec<String> {
        let mut normalized = fields
            .iter()
            .flat_map(|field| {
                if field.trim() == "verifiedInSurvey" {
                    vec!["verified_at".to_string(), "verified_by".to_string()]
                } else {
                    vec![normalize_changed_field(field)]
                }
            })
            .filter(|field| !field.is_empty())
            .collect::<Vec<_>>();
        normalized.sort();
        normalized.dedup();
        normalized
    }

    fn copy_fields(&mut self, source: &Self, fields: &[String]) {
        for field in fields {
            match field.as_str() {
                "asset_number" => self.asset_number = source.asset_number.clone(),
                "serial_number" => self.serial_number = source.serial_number.clone(),
                "qty" => self.qty = source.qty,
                "manufacturer" => self.manufacturer = source.manufacturer.clone(),
                "model" => self.model = source.model.clone(),
                "description" => self.description = source.description.clone(),
                "project_name" => self.project_name = source.project_name.clone(),
                "location" => self.location = source.location.clone(),
                "assigned_to" => self.assigned_to = source.assigned_to.clone(),
                "links" => self.links = source.links.clone(),
                "notes" => self.notes = source.notes.clone(),
                "lifecycle_status" => self.lifecycle_status = source.lifecycle_status.clone(),
                "working_status" => self.working_status = source.working_status.clone(),
                "condition" => self.condition = source.condition.clone(),
                "calibration_requirement" => {
                    self.calibration_requirement = source.calibration_requirement
                }
                "out_to_calibration" => self.out_to_calibration = source.out_to_calibration,
                "last_calibrated_at" => self
                    .last_calibrated_at
                    .clone_from(&source.last_calibrated_at),
                "calibration_due_at" => self
                    .calibration_due_at
                    .clone_from(&source.calibration_due_at),
                "calibration_interval_months" => {
                    self.calibration_interval_months = source.calibration_interval_months
                }
                "certificate_ref" => self.certificate_ref.clone_from(&source.certificate_ref),
                "calibration_vendor" => self
                    .calibration_vendor
                    .clone_from(&source.calibration_vendor),
                "calibration_notes" => self.calibration_notes.clone_from(&source.calibration_notes),
                "verified_at" => self.verified_at.clone_from(&source.verified_at),
                "verified_by" => self.verified_by.clone_from(&source.verified_by),
                "archived" => self.archived = source.archived,
                "picture_path" => self.picture_path = source.picture_path.clone(),
                _ => {}
            }
        }
    }

    fn set_updated_at(&mut self, updated_at: String) {
        self.updated_at = updated_at;
    }

    fn take_local_id_from(&mut self, existing: &Self) {
        self.id.clone_from(&existing.id);
        self.database_id = existing.database_id;
    }

    fn set_local_id(&mut self, id: i64) {
        self.id = id.to_string();
        self.database_id = Some(id);
    }
}

/// Frontend (camelCase) and stored (snake_case) names map to the snake_case names
/// `copy_fields` uses; identity and bookkeeping fields map to "" and are dropped.
fn normalize_changed_field(field: &str) -> String {
    match field.trim() {
        "calibrationRequirement" => "calibration_requirement".to_string(),
        "outToCalibration" => "out_to_calibration".to_string(),
        "lastCalibratedAt" => "last_calibrated_at".to_string(),
        "calibrationDueAt" => "calibration_due_at".to_string(),
        "calibrationIntervalMonths" => "calibration_interval_months".to_string(),
        "certificateRef" => "certificate_ref".to_string(),
        "calibrationVendor" => "calibration_vendor".to_string(),
        "calibrationNotes" => "calibration_notes".to_string(),
        "verifiedAt" | "verifiedInSurvey" => "verified_at".to_string(),
        "verifiedBy" => "verified_by".to_string(),
        "projectName" => "project_name".to_string(),
        "workingStatus" => "working_status".to_string(),
        "lifecycleStatus" => "lifecycle_status".to_string(),
        "picturePath" => "picture_path".to_string(),
        "assetNumber" => "asset_number".to_string(),
        "serialNumber" => "serial_number".to_string(),
        "assignedTo" => "assigned_to".to_string(),
        "databaseId" | "database_id" | "entryUuid" | "entry_uuid" | "createdAt" | "created_at"
        | "updatedAt" | "updated_at" | "importProvenance" | "import_provenance" | "id" => {
            String::new()
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::InventoryEntry;
    use crate::platform::sync::test_support::entry_state_keys;

    /// Snapshots carry entry states and are checksummed by re-serializing them, so a TE
    /// entry state must keep exactly the fields 0.1.4 wrote or 0.1.4 clients reject the
    /// snapshot.
    #[test]
    fn te_entry_states_serialize_only_the_0_1_4_fields() {
        let mut keys = entry_state_keys::<InventoryEntry>();
        keys.sort();
        assert_eq!(
            keys,
            [
                "changedFields",
                "deleted",
                "entryUuid",
                "lastOpId",
                "mutationTsUtc",
                "operationType",
                "sourceClientId",
                "sourceLocalSeq",
                "updatedAtUtc",
            ]
        );
    }
}
