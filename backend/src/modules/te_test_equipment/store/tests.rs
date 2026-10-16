use super::*;
use std::{env, fs, path::PathBuf};
use uuid::Uuid;

#[test]
fn calibration_verification_and_import_provenance_round_trip_through_feoxdb() {
    let db = test_db();
    let mut entry = test_entry("17", "uuid-17");
    entry.calibration_requirement =
        crate::modules::te_test_equipment::model::CalibrationRequirement::Required;
    entry.out_to_calibration = true;
    entry.last_calibrated_at = Some("2026-01-01".to_string());
    entry.calibration_due_at = Some("2027-01-01".to_string());
    entry.calibration_interval_months = Some(12);
    entry.certificate_ref = Some("CERT-17".to_string());
    entry.calibration_vendor = Some("Vendor 17".to_string());
    entry.calibration_notes = Some("Handle carefully".to_string());
    entry.verified_at = Some("2026-07-13T12:00:00Z".to_string());
    entry.verified_by = Some("Taylor".to_string());
    entry.import_provenance = Some(crate::modules::te_test_equipment::model::ImportProvenance {
        batch_id: "sha256:batch-17".to_string(),
        source_filename: "synthetic.xlsx".to_string(),
        source_sheet: Some("Equipment".to_string()),
        source_row: 18,
        original_id: Some("legacy-17".to_string()),
        original_asset_number: Some("TE-17".to_string()),
        original_serial_number: Some("SN-17".to_string()),
    });

    db.put_entry(&entry).unwrap();
    let loaded = db.find_entry("uuid-17").unwrap().unwrap();

    assert_eq!(loaded, entry);
    assert_eq!(
        loaded
            .import_provenance
            .as_ref()
            .map(|value| value.source_row),
        Some(18)
    );
}

fn test_entry(
    id: &str,
    entry_uuid: &str,
) -> crate::modules::te_test_equipment::model::InventoryEntry {
    crate::modules::te_test_equipment::model::InventoryEntry {
        id: id.to_string(),
        database_id: id.parse::<i64>().ok(),
        entry_uuid: entry_uuid.to_string(),
        asset_number: format!("ME-{id}"),
        serial_number: String::new(),
        qty: Some(1.0),
        manufacturer: "Mitutoyo".to_string(),
        model: String::new(),
        description: "Caliper".to_string(),
        project_name: String::new(),
        location: "Lab".to_string(),
        assigned_to: String::new(),
        links: String::new(),
        notes: String::new(),
        lifecycle_status: "active".to_string(),
        working_status: "unknown".to_string(),
        condition: String::new(),
        calibration_requirement:
            crate::modules::te_test_equipment::model::CalibrationRequirement::Unknown,
        out_to_calibration: false,
        last_calibrated_at: None,
        calibration_due_at: None,
        calibration_interval_months: None,
        certificate_ref: None,
        calibration_vendor: None,
        calibration_notes: None,
        verified_at: None,
        verified_by: None,
        import_provenance: None,
        archived: false,
        manual_entry: false,
        picture_path: String::new(),
        created_at: "2026-01-01T00:00:00Z".to_string(),
        updated_at: "2026-01-01T00:00:00Z".to_string(),
    }
}

fn test_db() -> InventoryDb {
    let root = unique_test_dir("store");
    fs::create_dir_all(&root).unwrap();
    InventoryDb::open_at(root.join("inventory.feox")).unwrap()
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
}
