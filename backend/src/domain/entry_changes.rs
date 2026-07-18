use crate::model::{now_timestamp, InventoryEntry, InventoryEntryInput};

pub(crate) fn entry_base_version(entry: &InventoryEntry) -> Option<String> {
    (!entry.updated_at.is_empty()).then(|| entry.updated_at.clone())
}

pub(crate) fn normalize_changed_entry_fields(fields: Vec<String>) -> Vec<String> {
    let mut normalized = fields
        .into_iter()
        .map(|field| normalize_changed_entry_field(&field))
        .filter(|field| !field.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

pub(crate) fn update_entry_from_input_fields(
    mut entry: InventoryEntry,
    input: &InventoryEntryInput,
    changed_fields: &[String],
) -> InventoryEntry {
    for field in changed_fields {
        match field.as_str() {
            "asset_number" => entry.asset_number.clone_from(&input.asset_number),
            "serial_number" => entry.serial_number.clone_from(&input.serial_number),
            "qty" => entry.qty = input.qty,
            "manufacturer" => entry.manufacturer.clone_from(&input.manufacturer),
            "model" => entry.model.clone_from(&input.model),
            "description" => entry.description.clone_from(&input.description),
            "project_name" => entry.project_name.clone_from(&input.project_name),
            "location" => entry.location.clone_from(&input.location),
            "assigned_to" => entry.assigned_to.clone_from(&input.assigned_to),
            "links" => entry.links.clone_from(&input.links),
            "notes" => entry.notes.clone_from(&input.notes),
            "lifecycle_status" => entry.lifecycle_status.clone_from(&input.lifecycle_status),
            "working_status" => entry.working_status.clone_from(&input.working_status),
            "condition" => entry.condition.clone_from(&input.condition),
            "calibration_requirement" => {
                entry.calibration_requirement = input.calibration_requirement
            }
            "out_to_calibration" => entry.out_to_calibration = input.out_to_calibration,
            "last_calibrated_at" => entry
                .last_calibrated_at
                .clone_from(&input.last_calibrated_at),
            "calibration_due_at" => entry
                .calibration_due_at
                .clone_from(&input.calibration_due_at),
            "calibration_interval_months" => {
                entry.calibration_interval_months = input.calibration_interval_months
            }
            "certificate_ref" => entry.certificate_ref.clone_from(&input.certificate_ref),
            "calibration_vendor" => entry
                .calibration_vendor
                .clone_from(&input.calibration_vendor),
            "calibration_notes" => entry.calibration_notes.clone_from(&input.calibration_notes),
            "verified_at" => entry.verified_at.clone_from(&input.verified_at),
            "verified_by" => entry.verified_by.clone_from(&input.verified_by),
            "archived" => entry.archived = input.archived,
            "picture_path" => entry.picture_path = input.picture_path.clone().unwrap_or_default(),
            _ => {}
        }
    }

    entry.updated_at = now_timestamp();
    entry
}

pub(crate) fn changed_entry_fields(before: &InventoryEntry, after: &InventoryEntry) -> Vec<String> {
    let mut fields = Vec::new();

    if before.asset_number != after.asset_number {
        fields.push("asset_number".to_string());
    }
    if before.serial_number != after.serial_number {
        fields.push("serial_number".to_string());
    }
    if before.qty != after.qty {
        fields.push("qty".to_string());
    }
    if before.manufacturer != after.manufacturer {
        fields.push("manufacturer".to_string());
    }
    if before.model != after.model {
        fields.push("model".to_string());
    }
    if before.description != after.description {
        fields.push("description".to_string());
    }
    if before.project_name != after.project_name {
        fields.push("project_name".to_string());
    }
    if before.location != after.location {
        fields.push("location".to_string());
    }
    if before.assigned_to != after.assigned_to {
        fields.push("assigned_to".to_string());
    }
    if before.links != after.links {
        fields.push("links".to_string());
    }
    if before.notes != after.notes {
        fields.push("notes".to_string());
    }
    if before.lifecycle_status != after.lifecycle_status {
        fields.push("lifecycle_status".to_string());
    }
    if before.working_status != after.working_status {
        fields.push("working_status".to_string());
    }
    if before.condition != after.condition {
        fields.push("condition".to_string());
    }
    if before.calibration_requirement != after.calibration_requirement {
        fields.push("calibration_requirement".to_string());
    }
    if before.out_to_calibration != after.out_to_calibration {
        fields.push("out_to_calibration".to_string());
    }
    if before.last_calibrated_at != after.last_calibrated_at {
        fields.push("last_calibrated_at".to_string());
    }
    if before.calibration_due_at != after.calibration_due_at {
        fields.push("calibration_due_at".to_string());
    }
    if before.calibration_interval_months != after.calibration_interval_months {
        fields.push("calibration_interval_months".to_string());
    }
    if before.certificate_ref != after.certificate_ref {
        fields.push("certificate_ref".to_string());
    }
    if before.calibration_vendor != after.calibration_vendor {
        fields.push("calibration_vendor".to_string());
    }
    if before.calibration_notes != after.calibration_notes {
        fields.push("calibration_notes".to_string());
    }
    if before.verified_at != after.verified_at {
        fields.push("verified_at".to_string());
    }
    if before.verified_by != after.verified_by {
        fields.push("verified_by".to_string());
    }
    if before.archived != after.archived {
        fields.push("archived".to_string());
    }
    if before.picture_path != after.picture_path {
        fields.push("picture_path".to_string());
    }

    fields
}

fn normalize_changed_entry_field(field: &str) -> String {
    match field.trim() {
        "assetNumber" => "asset_number".to_string(),
        "serialNumber" => "serial_number".to_string(),
        "projectName" => "project_name".to_string(),
        "assignedTo" => "assigned_to".to_string(),
        "lifecycleStatus" => "lifecycle_status".to_string(),
        "workingStatus" => "working_status".to_string(),
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
        "picturePath" => "picture_path".to_string(),
        "databaseId" | "database_id" | "entryUuid" | "entry_uuid" | "createdAt" | "created_at"
        | "updatedAt" | "updated_at" | "importProvenance" | "import_provenance" | "id" => {
            String::new()
        }
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        create_entry_from_input, normalize_entry_input, CalibrationRequirement, ImportProvenance,
    };

    #[test]
    fn calibration_and_verification_fields_are_selective_and_aliases_are_canonicalized() {
        let before = create_entry_from_input(
            1,
            normalize_entry_input(InventoryEntryInput {
                description: "Meter".to_string(),
                ..InventoryEntryInput::default()
            }),
        );
        let mut after = before.clone();
        after.calibration_requirement = CalibrationRequirement::Required;
        after.calibration_due_at = Some("2027-07-13".to_string());
        after.verified_at = Some("2026-07-13T12:00:00Z".to_string());
        after.verified_by = Some("Taylor".to_string());

        assert_eq!(
            changed_entry_fields(&before, &after),
            vec![
                "calibration_requirement".to_string(),
                "calibration_due_at".to_string(),
                "verified_at".to_string(),
                "verified_by".to_string(),
            ]
        );
        assert_eq!(
            normalize_changed_entry_fields(vec![
                "calibrationRequirement".to_string(),
                "calibrationDueAt".to_string(),
                "verifiedAt".to_string(),
                "verifiedBy".to_string(),
            ]),
            vec![
                "calibration_due_at".to_string(),
                "calibration_requirement".to_string(),
                "verified_at".to_string(),
                "verified_by".to_string(),
            ]
        );
    }

    #[test]
    fn field_scoped_update_preserves_import_provenance() {
        let mut before = create_entry_from_input(
            1,
            normalize_entry_input(InventoryEntryInput {
                description: "Meter".to_string(),
                ..InventoryEntryInput::default()
            }),
        );
        before.import_provenance = Some(ImportProvenance {
            batch_id: "sha256:batch".to_string(),
            source_filename: "synthetic.csv".to_string(),
            source_sheet: None,
            source_row: 2,
            original_id: None,
            original_asset_number: Some("TE-1".to_string()),
            original_serial_number: None,
        });
        let input = normalize_entry_input(InventoryEntryInput {
            description: "Edited meter".to_string(),
            ..InventoryEntryInput::default()
        });

        let after =
            update_entry_from_input_fields(before.clone(), &input, &["description".to_string()]);

        assert_eq!(after.description, "Edited meter");
        assert_eq!(after.import_provenance, before.import_provenance);
    }
}
