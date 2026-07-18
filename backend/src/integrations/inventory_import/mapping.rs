#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum SourceTarget {
    OriginalId,
    AssetNumber,
    SerialNumber,
    Quantity,
    Manufacturer,
    Model,
    Description,
    ProjectName,
    Location,
    AssignedTo,
    Links,
    Notes,
    LifecycleStatus,
    WorkingStatus,
    Condition,
    CalibrationStatus,
    CalibrationRequirement,
    OutToCalibration,
    LastCalibratedAt,
    CalibrationDueAt,
    CalibrationIntervalMonths,
    CertificateRef,
    CalibrationVendor,
    CalibrationNotes,
    Verified,
    VerifiedAt,
    VerifiedBy,
    Archived,
    PicturePath,
}

impl SourceTarget {
    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::OriginalId => "original_id",
            Self::AssetNumber => "asset_number",
            Self::SerialNumber => "serial_number",
            Self::Quantity => "qty",
            Self::Manufacturer => "manufacturer",
            Self::Model => "model",
            Self::Description => "description",
            Self::ProjectName => "project_name",
            Self::Location => "location",
            Self::AssignedTo => "assigned_to",
            Self::Links => "links",
            Self::Notes => "notes",
            Self::LifecycleStatus => "lifecycle_status",
            Self::WorkingStatus => "working_status",
            Self::Condition => "condition",
            Self::CalibrationStatus => "calibration_status",
            Self::CalibrationRequirement => "calibration_requirement",
            Self::OutToCalibration => "out_to_calibration",
            Self::LastCalibratedAt => "last_calibrated_at",
            Self::CalibrationDueAt => "calibration_due_at",
            Self::CalibrationIntervalMonths => "calibration_interval_months",
            Self::CertificateRef => "certificate_ref",
            Self::CalibrationVendor => "calibration_vendor",
            Self::CalibrationNotes => "calibration_notes",
            Self::Verified => "verified",
            Self::VerifiedAt => "verified_at",
            Self::VerifiedBy => "verified_by",
            Self::Archived => "archived",
            Self::PicturePath => "picture_path",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MappingDisposition {
    Mapped(SourceTarget),
    IntentionallyIgnored(&'static str),
    Unknown,
}

pub(super) fn map_header(header: &str) -> MappingDisposition {
    let normalized = normalize_header(header);
    let target = match normalized.as_str() {
        "id" | "equipment_id" | "record_id" | "original_id" => SourceTarget::OriginalId,
        "asset" | "asset_number" | "asset_no" | "asset_tag" => SourceTarget::AssetNumber,
        "serial" | "serial_number" | "serial_no" | "serial_num" => SourceTarget::SerialNumber,
        "qty" | "quantity" => SourceTarget::Quantity,
        "manufacturer" | "make" | "maker" => SourceTarget::Manufacturer,
        "model" | "model_number" | "model_no" => SourceTarget::Model,
        "description" | "equipment" | "name" => SourceTarget::Description,
        "project" | "project_name" => SourceTarget::ProjectName,
        "location" | "room" | "area" => SourceTarget::Location,
        "assigned_to" | "assigned_user" | "user" => SourceTarget::AssignedTo,
        "links" | "link" | "url" => SourceTarget::Links,
        "notes" | "general_notes" => SourceTarget::Notes,
        "lifecycle" | "lifecycle_status" | "status" => SourceTarget::LifecycleStatus,
        "working" | "working_status" => SourceTarget::WorkingStatus,
        "condition" => SourceTarget::Condition,
        "calibration_status" | "cal_status" => SourceTarget::CalibrationStatus,
        "calibration_requirement" | "cal_requirement" => SourceTarget::CalibrationRequirement,
        "out_to_calibration" | "out_to_cal" => SourceTarget::OutToCalibration,
        "last_calibrated" | "last_calibrated_at" | "last_cal_date" => {
            SourceTarget::LastCalibratedAt
        }
        "calibration_due" | "calibration_due_at" | "cal_due_date" | "due_date"
        | "next_calibration" => SourceTarget::CalibrationDueAt,
        "calibration_interval_months" | "interval_months" => {
            SourceTarget::CalibrationIntervalMonths
        }
        "certificate" | "certificate_ref" | "certificate_number" | "certificate_link" => {
            SourceTarget::CertificateRef
        }
        "calibration_vendor" | "cal_vendor" | "vendor" => SourceTarget::CalibrationVendor,
        "calibration_notes" | "cal_notes" => SourceTarget::CalibrationNotes,
        "verified" | "verified_in_survey" => SourceTarget::Verified,
        "verified_at" => SourceTarget::VerifiedAt,
        "verified_by" => SourceTarget::VerifiedBy,
        "archived" | "is_archived" => SourceTarget::Archived,
        "picture" | "picture_path" | "image" => SourceTarget::PicturePath,
        "ownership"
        | "cal_cost"
        | "blue_dot"
        | "rental_vendor"
        | "rental_cost_mo"
        | "est_age_yrs"
        | "est_age_years"
        | "estimated_age_years"
        | "age"
        | "age_years"
        | "row_number" => {
            return MappingDisposition::IntentionallyIgnored(
                "Deferred field is explicitly ignored.",
            )
        }
        _ => return MappingDisposition::Unknown,
    };
    MappingDisposition::Mapped(target)
}

pub(super) fn normalize_header(value: &str) -> String {
    let mut normalized = String::new();
    let mut pending_separator = false;
    for character in value.trim().chars() {
        if character.is_ascii_alphanumeric() {
            if pending_separator && !normalized.is_empty() {
                normalized.push('_');
            }
            normalized.push(character.to_ascii_lowercase());
            pending_separator = false;
        } else {
            pending_separator = true;
        }
    }
    normalized
}
