use crate::model::InventoryEntry;

pub(super) const INVENTORY_COLUMNS: [InventoryColumn; 29] = [
    InventoryColumn::new(
        "Asset Number",
        16.0,
        InventoryField::AssetNumber,
        CellKind::Text,
    ),
    InventoryColumn::new(
        "Serial Number",
        20.0,
        InventoryField::SerialNumber,
        CellKind::Text,
    ),
    InventoryColumn::new("Qty", 9.0, InventoryField::Qty, CellKind::Number),
    InventoryColumn::new(
        "Manufacturer",
        18.0,
        InventoryField::Manufacturer,
        CellKind::Text,
    ),
    InventoryColumn::new("Model", 16.0, InventoryField::Model, CellKind::Text),
    InventoryColumn::new(
        "Description",
        32.0,
        InventoryField::Description,
        CellKind::WrappedText,
    ),
    InventoryColumn::new("Project", 20.0, InventoryField::Project, CellKind::Text),
    InventoryColumn::new("Location", 24.0, InventoryField::Location, CellKind::Text),
    InventoryColumn::new(
        "Assigned To",
        14.0,
        InventoryField::AssignedTo,
        CellKind::Text,
    ),
    InventoryColumn::new(
        "Lifecycle",
        13.0,
        InventoryField::Lifecycle,
        CellKind::Lifecycle,
    ),
    InventoryColumn::new("Working", 13.0, InventoryField::Working, CellKind::Working),
    InventoryColumn::new(
        "Condition",
        22.0,
        InventoryField::Condition,
        CellKind::WrappedText,
    ),
    InventoryColumn::new(
        "Calibration Requirement",
        20.0,
        InventoryField::CalibrationRequirement,
        CellKind::Centered,
    ),
    InventoryColumn::new(
        "Out to Calibration",
        18.0,
        InventoryField::OutToCalibration,
        CellKind::Centered,
    ),
    InventoryColumn::new(
        "Last Calibrated At",
        18.0,
        InventoryField::LastCalibratedAt,
        CellKind::Text,
    ),
    InventoryColumn::new(
        "Calibration Due At",
        18.0,
        InventoryField::CalibrationDueAt,
        CellKind::Text,
    ),
    InventoryColumn::new(
        "Calibration Interval Months",
        20.0,
        InventoryField::CalibrationIntervalMonths,
        CellKind::Number,
    ),
    InventoryColumn::new(
        "Calibration Health",
        18.0,
        InventoryField::CalibrationHealth,
        CellKind::Centered,
    ),
    InventoryColumn::new(
        "Certificate Ref",
        20.0,
        InventoryField::CertificateRef,
        CellKind::Text,
    ),
    InventoryColumn::new(
        "Calibration Vendor",
        22.0,
        InventoryField::CalibrationVendor,
        CellKind::Text,
    ),
    InventoryColumn::new(
        "Calibration Notes",
        36.0,
        InventoryField::CalibrationNotes,
        CellKind::WrappedText,
    ),
    InventoryColumn::new(
        "Verified At",
        22.0,
        InventoryField::VerifiedAt,
        CellKind::Text,
    ),
    InventoryColumn::new(
        "Verified By",
        20.0,
        InventoryField::VerifiedBy,
        CellKind::Text,
    ),
    InventoryColumn::new(
        "Archived",
        12.0,
        InventoryField::Archived,
        CellKind::Centered,
    ),
    InventoryColumn::new(
        "Picture Path",
        34.0,
        InventoryField::PicturePath,
        CellKind::Text,
    ),
    InventoryColumn::new("Links", 28.0, InventoryField::Links, CellKind::WrappedText),
    InventoryColumn::new("Notes", 40.0, InventoryField::Notes, CellKind::WrappedText),
    InventoryColumn::new(
        "Created At",
        22.0,
        InventoryField::CreatedAt,
        CellKind::Text,
    ),
    InventoryColumn::new(
        "Updated At",
        22.0,
        InventoryField::UpdatedAt,
        CellKind::Text,
    ),
];

#[derive(Clone, Copy)]
pub(super) struct InventoryColumn {
    pub(super) header: &'static str,
    pub(super) width: f64,
    pub(super) field: InventoryField,
    pub(super) kind: CellKind,
}

impl InventoryColumn {
    const fn new(header: &'static str, width: f64, field: InventoryField, kind: CellKind) -> Self {
        Self {
            header,
            width,
            field,
            kind,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) enum InventoryField {
    AssetNumber,
    SerialNumber,
    Qty,
    Manufacturer,
    Model,
    Description,
    Project,
    Location,
    AssignedTo,
    Lifecycle,
    Working,
    Condition,
    CalibrationRequirement,
    OutToCalibration,
    LastCalibratedAt,
    CalibrationDueAt,
    CalibrationIntervalMonths,
    CalibrationHealth,
    CertificateRef,
    CalibrationVendor,
    CalibrationNotes,
    VerifiedAt,
    VerifiedBy,
    Archived,
    PicturePath,
    Links,
    Notes,
    CreatedAt,
    UpdatedAt,
}

#[derive(Clone, Copy)]
pub(super) enum CellKind {
    Centered,
    Lifecycle,
    Number,
    Text,
    WrappedText,
    Working,
}

pub(super) fn inventory_text(entry: &InventoryEntry, field: InventoryField) -> &str {
    match field {
        InventoryField::AssetNumber => &entry.asset_number,
        InventoryField::SerialNumber => &entry.serial_number,
        InventoryField::Manufacturer => &entry.manufacturer,
        InventoryField::Model => &entry.model,
        InventoryField::Description => &entry.description,
        InventoryField::Project => &entry.project_name,
        InventoryField::Location => &entry.location,
        InventoryField::AssignedTo => &entry.assigned_to,
        InventoryField::Lifecycle => &entry.lifecycle_status,
        InventoryField::Working => &entry.working_status,
        InventoryField::Condition => &entry.condition,
        InventoryField::CalibrationRequirement => entry.calibration_requirement.display_label(),
        InventoryField::LastCalibratedAt => entry.last_calibrated_at.as_deref().unwrap_or(""),
        InventoryField::CalibrationDueAt => entry.calibration_due_at.as_deref().unwrap_or(""),
        InventoryField::CertificateRef => entry.certificate_ref.as_deref().unwrap_or(""),
        InventoryField::CalibrationVendor => entry.calibration_vendor.as_deref().unwrap_or(""),
        InventoryField::CalibrationNotes => entry.calibration_notes.as_deref().unwrap_or(""),
        InventoryField::VerifiedAt => entry.verified_at.as_deref().unwrap_or(""),
        InventoryField::VerifiedBy => entry.verified_by.as_deref().unwrap_or(""),
        InventoryField::PicturePath => &entry.picture_path,
        InventoryField::Links => &entry.links,
        InventoryField::Notes => &entry.notes,
        InventoryField::CreatedAt => &entry.created_at,
        InventoryField::UpdatedAt => &entry.updated_at,
        InventoryField::Qty
        | InventoryField::OutToCalibration
        | InventoryField::CalibrationIntervalMonths
        | InventoryField::CalibrationHealth
        | InventoryField::Archived => "",
    }
}

pub(super) fn yes_if(value: bool) -> &'static str {
    if value {
        "Yes"
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calibration_requirement_export_uses_user_facing_labels() {
        let reference: InventoryEntry = serde_json::from_value(serde_json::json!({
            "id": "1",
            "entryUuid": "reference",
            "description": "Reference meter",
            "calibrationRequirement": "reference_only"
        }))
        .unwrap();
        let not_required: InventoryEntry = serde_json::from_value(serde_json::json!({
            "id": "2",
            "entryUuid": "not-required",
            "description": "Display only",
            "calibrationRequirement": "not_required"
        }))
        .unwrap();

        assert_eq!(
            inventory_text(&reference, InventoryField::CalibrationRequirement),
            "Reference only"
        );
        assert_eq!(
            inventory_text(&not_required, InventoryField::CalibrationRequirement),
            "Not required"
        );
    }
}
