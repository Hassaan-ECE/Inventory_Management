use std::path::Path;

use chrono::{DateTime, Days, NaiveDate, SecondsFormat, Utc};
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;
use uuid::Uuid;

pub(crate) const MAX_QUERY_LIMIT: usize = 10_000;
const MAX_QUANTITY: f64 = 1_000_000.0;
const STANDARD_TEXT_LIMIT: usize = 512;
const LONG_TEXT_LIMIT: usize = 4_000;
const NOTES_TEXT_LIMIT: usize = 8_000;
const PATH_TEXT_LIMIT: usize = 2_048;
const MAX_CALIBRATION_INTERVAL_MONTHS: u16 = 1_200;
const IMAGE_PATH_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "bmp", "tif", "tiff"];
const LINK_PROTOCOLS: &[&str] = &["http", "https", "mailto"];
const PICTURE_URL_PROTOCOLS: &[&str] = &["http", "https"];
pub(crate) type CommandResult<T> = Result<T, String>;
pub(crate) const LEGACY_VERIFIED_APPROXIMATION_LABEL: &str =
    "Legacy verified flag — timestamp approximated from updatedAt";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CalibrationRequirement {
    Required,
    ReferenceOnly,
    NotRequired,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CalibrationHealth {
    MissingDue,
    Overdue,
    DueSoon,
    Current,
    NotApplicable,
    Unknown,
    OutToCal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ImportProvenance {
    pub batch_id: String,
    pub source_filename: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_sheet: Option<String>,
    pub source_row: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_asset_number: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub original_serial_number: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventoryEntry {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database_id: Option<i64>,
    #[serde(default)]
    pub entry_uuid: String,
    #[serde(default)]
    pub asset_number: String,
    #[serde(default)]
    pub serial_number: String,
    pub qty: Option<f64>,
    #[serde(default)]
    pub manufacturer: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub project_name: String,
    #[serde(default)]
    pub location: String,
    #[serde(default)]
    pub assigned_to: String,
    #[serde(default)]
    pub links: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default = "default_lifecycle_status")]
    pub lifecycle_status: String,
    #[serde(default = "default_working_status")]
    pub working_status: String,
    #[serde(default)]
    pub condition: String,
    #[serde(default)]
    pub calibration_requirement: CalibrationRequirement,
    #[serde(default)]
    pub out_to_calibration: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_calibrated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_due_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_interval_months: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_vendor: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verified_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_provenance: Option<ImportProvenance>,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub manual_entry: bool,
    #[serde(default)]
    pub picture_path: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
struct InventoryEntryWire {
    id: String,
    database_id: Option<i64>,
    entry_uuid: String,
    asset_number: String,
    serial_number: String,
    qty: Option<f64>,
    manufacturer: String,
    model: String,
    description: String,
    project_name: String,
    location: String,
    assigned_to: String,
    links: String,
    notes: String,
    lifecycle_status: String,
    working_status: String,
    condition: String,
    calibration_requirement: CalibrationRequirement,
    out_to_calibration: bool,
    last_calibrated_at: Option<String>,
    calibration_due_at: Option<String>,
    calibration_interval_months: Option<u16>,
    certificate_ref: Option<String>,
    calibration_vendor: Option<String>,
    calibration_notes: Option<String>,
    verified_at: Option<String>,
    verified_by: Option<String>,
    import_provenance: Option<ImportProvenance>,
    verified_in_survey: bool,
    archived: bool,
    manual_entry: bool,
    picture_path: String,
    created_at: String,
    updated_at: String,
}

impl<'de> Deserialize<'de> for InventoryEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::from_wire(InventoryEntryWire::deserialize(
            deserializer,
        )?))
    }
}

impl InventoryEntry {
    fn from_wire(wire: InventoryEntryWire) -> Self {
        let mut verified_at = normalize_optional_string(wire.verified_at);
        let mut verified_by = normalize_optional_string(wire.verified_by);
        if verified_at.is_none() && verified_by.is_none() && wire.verified_in_survey {
            let updated_at = wire.updated_at.trim();
            if DateTime::parse_from_rfc3339(updated_at).is_ok() {
                verified_at = Some(updated_at.to_string());
                verified_by = Some(LEGACY_VERIFIED_APPROXIMATION_LABEL.to_string());
            }
        }

        Self {
            id: wire.id,
            database_id: wire.database_id,
            entry_uuid: wire.entry_uuid,
            asset_number: wire.asset_number,
            serial_number: wire.serial_number,
            qty: wire.qty,
            manufacturer: wire.manufacturer,
            model: wire.model,
            description: wire.description,
            project_name: wire.project_name,
            location: wire.location,
            assigned_to: wire.assigned_to,
            links: wire.links,
            notes: wire.notes,
            lifecycle_status: if wire.lifecycle_status.is_empty() {
                default_lifecycle_status()
            } else {
                wire.lifecycle_status
            },
            working_status: if wire.working_status.is_empty() {
                default_working_status()
            } else {
                wire.working_status
            },
            condition: wire.condition,
            calibration_requirement: wire.calibration_requirement,
            out_to_calibration: wire.out_to_calibration,
            last_calibrated_at: normalize_optional_string(wire.last_calibrated_at),
            calibration_due_at: normalize_optional_string(wire.calibration_due_at),
            calibration_interval_months: wire.calibration_interval_months,
            certificate_ref: normalize_optional_string(wire.certificate_ref),
            calibration_vendor: normalize_optional_string(wire.calibration_vendor),
            calibration_notes: normalize_optional_string(wire.calibration_notes),
            verified_at,
            verified_by,
            import_provenance: wire.import_provenance,
            archived: wire.archived,
            manual_entry: wire.manual_entry,
            picture_path: wire.picture_path,
            created_at: wire.created_at,
            updated_at: wire.updated_at,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct InventoryEntryInput {
    pub asset_number: String,
    pub serial_number: String,
    pub qty: Option<f64>,
    pub manufacturer: String,
    pub model: String,
    pub description: String,
    pub project_name: String,
    pub location: String,
    pub assigned_to: String,
    pub links: String,
    pub notes: String,
    pub lifecycle_status: String,
    pub working_status: String,
    pub condition: String,
    pub calibration_requirement: CalibrationRequirement,
    pub out_to_calibration: bool,
    pub last_calibrated_at: Option<String>,
    pub calibration_due_at: Option<String>,
    pub calibration_interval_months: Option<u16>,
    pub certificate_ref: Option<String>,
    pub calibration_vendor: Option<String>,
    pub calibration_notes: Option<String>,
    pub verified_at: Option<String>,
    pub verified_by: Option<String>,
    pub archived: bool,
    pub picture_path: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct InventoryEntryEditContext {
    pub base_version: Option<String>,
    pub changed_fields: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct FilterState {
    pub asset_number: String,
    pub manufacturer: String,
    pub model: String,
    pub description: String,
    pub location: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct SortState {
    pub column: String,
    pub direction: String,
}

impl Default for SortState {
    fn default() -> Self {
        Self {
            column: "manufacturer".to_string(),
            direction: "asc".to_string(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct InventoryQueryInput {
    pub filters: FilterState,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub query: String,
    pub scope: String,
    pub sort: SortState,
}

impl Default for InventoryQueryInput {
    fn default() -> Self {
        Self {
            filters: FilterState::default(),
            limit: Some(MAX_QUERY_LIMIT),
            offset: Some(0),
            query: String::new(),
            scope: "inventory".to_string(),
            sort: SortState::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventoryCounts {
    pub archive: usize,
    pub inventory: usize,
    pub total: usize,
    pub verified: usize,
}

#[derive(Debug, Clone, Serialize)]
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
    pub sync_interval_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventorySyncResult {
    pub db_path: String,
    pub entries: Vec<InventoryEntry>,
    pub entries_changed: Option<bool>,
    pub shared: InventorySharedStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventoryQueryResult {
    pub counts: InventoryCounts,
    pub db_path: String,
    pub entries: Vec<InventoryEntry>,
    pub shared: InventorySharedStatus,
    pub total_filtered: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventoryEntryMutationResult {
    pub entry: InventoryEntry,
    pub message: String,
    pub mutation_mode: String,
    pub shared: InventorySharedStatus,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct InventoryDeleteMutationResult {
    pub entry_id: String,
    pub message: String,
    pub mutation_mode: String,
    pub shared: InventorySharedStatus,
}

pub(crate) fn default_lifecycle_status() -> String {
    "active".to_string()
}

pub(crate) fn default_working_status() -> String {
    "unknown".to_string()
}

pub(crate) fn db_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

pub(crate) fn numeric_id(id: &str) -> i64 {
    id.parse::<i64>().unwrap_or(0)
}

pub(crate) fn now_timestamp() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

pub(crate) fn normalize_entry_input(input: InventoryEntryInput) -> InventoryEntryInput {
    InventoryEntryInput {
        asset_number: input.asset_number.trim().to_string(),
        serial_number: input.serial_number.trim().to_string(),
        qty: input.qty,
        manufacturer: input.manufacturer.trim().to_string(),
        model: input.model.trim().to_string(),
        description: input.description.trim().to_string(),
        project_name: input.project_name.trim().to_string(),
        location: input.location.trim().to_string(),
        assigned_to: input.assigned_to.trim().to_string(),
        links: input.links.trim().to_string(),
        notes: input.notes.trim().to_string(),
        lifecycle_status: normalize_enum(
            input.lifecycle_status,
            &["active", "repair", "scrapped", "missing", "rental"],
            "active",
        ),
        working_status: normalize_enum(
            input.working_status,
            &["unknown", "working", "limited", "not_working"],
            "unknown",
        ),
        condition: input.condition.trim().to_string(),
        calibration_requirement: input.calibration_requirement,
        out_to_calibration: input.out_to_calibration,
        last_calibrated_at: normalize_optional_string(input.last_calibrated_at),
        calibration_due_at: normalize_optional_string(input.calibration_due_at),
        calibration_interval_months: input.calibration_interval_months,
        certificate_ref: normalize_optional_string(input.certificate_ref),
        calibration_vendor: normalize_optional_string(input.calibration_vendor),
        calibration_notes: normalize_optional_string(input.calibration_notes),
        verified_at: normalize_optional_string(input.verified_at),
        verified_by: normalize_optional_string(input.verified_by),
        archived: input.archived,
        picture_path: input.picture_path.map(|path| path.trim().to_string()),
    }
}

pub(crate) fn validate_entry_input(input: &InventoryEntryInput) -> CommandResult<()> {
    let has_identity = !input.asset_number.is_empty()
        || !input.serial_number.is_empty()
        || !input.manufacturer.is_empty()
        || !input.model.is_empty()
        || !input.description.is_empty();

    if !has_identity {
        return Err(
            "Provide at least an asset number, serial number, manufacturer, model, or description before saving."
                .to_string(),
        );
    }

    if let Some(qty) = input.qty {
        if !qty.is_finite() || !(0.0..=MAX_QUANTITY).contains(&qty) {
            return Err(format!(
                "Quantity must be a number between 0 and {MAX_QUANTITY}."
            ));
        }
    }

    validate_text_length("Asset number", &input.asset_number, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Serial number", &input.serial_number, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Manufacturer", &input.manufacturer, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Model", &input.model, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Description", &input.description, LONG_TEXT_LIMIT)?;
    validate_text_length("Project name", &input.project_name, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Location", &input.location, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Assigned to", &input.assigned_to, STANDARD_TEXT_LIMIT)?;
    validate_text_length("Links", &input.links, LONG_TEXT_LIMIT)?;
    validate_links(&input.links)?;
    validate_text_length("Notes", &input.notes, NOTES_TEXT_LIMIT)?;
    validate_text_length("Condition", &input.condition, STANDARD_TEXT_LIMIT)?;
    let last_calibrated_at =
        validate_optional_date("Last calibrated date", input.last_calibrated_at.as_deref())?;
    let calibration_due_at =
        validate_optional_date("Calibration due date", input.calibration_due_at.as_deref())?;
    if let (Some(last), Some(due)) = (last_calibrated_at, calibration_due_at) {
        if due < last {
            return Err(
                "Calibration due date must be on or after the last calibrated date.".to_string(),
            );
        }
    }
    if let Some(interval) = input.calibration_interval_months {
        if interval == 0 || interval > MAX_CALIBRATION_INTERVAL_MONTHS {
            return Err(format!(
                "Calibration interval must be between 1 and {MAX_CALIBRATION_INTERVAL_MONTHS} months."
            ));
        }
    }
    validate_optional_text_length(
        "Certificate reference",
        input.certificate_ref.as_deref(),
        STANDARD_TEXT_LIMIT,
    )?;
    validate_optional_text_length(
        "Calibration vendor",
        input.calibration_vendor.as_deref(),
        STANDARD_TEXT_LIMIT,
    )?;
    validate_optional_text_length(
        "Calibration notes",
        input.calibration_notes.as_deref(),
        NOTES_TEXT_LIMIT,
    )?;
    if let Some(verified_at) = input.verified_at.as_deref() {
        DateTime::parse_from_rfc3339(verified_at)
            .map_err(|_| "Verified at must be a valid RFC 3339 timestamp.".to_string())?;
    } else if input.verified_by.is_some() {
        return Err("Verified at is required when a verifier is provided.".to_string());
    }
    validate_optional_text_length(
        "Verified by",
        input.verified_by.as_deref(),
        STANDARD_TEXT_LIMIT,
    )?;
    if let Some(picture_path) = &input.picture_path {
        validate_text_length("Picture path", picture_path, PATH_TEXT_LIMIT)?;
        validate_picture_path(picture_path)?;
    }

    Ok(())
}

pub(crate) fn create_entry_from_input(id: i64, input: InventoryEntryInput) -> InventoryEntry {
    let timestamp = now_timestamp();
    InventoryEntry {
        id: id.to_string(),
        database_id: Some(id),
        entry_uuid: Uuid::new_v4().simple().to_string(),
        asset_number: input.asset_number,
        serial_number: input.serial_number,
        qty: input.qty,
        manufacturer: input.manufacturer,
        model: input.model,
        description: input.description,
        project_name: input.project_name,
        location: input.location,
        assigned_to: input.assigned_to,
        links: input.links,
        notes: input.notes,
        lifecycle_status: input.lifecycle_status,
        working_status: input.working_status,
        condition: input.condition,
        calibration_requirement: input.calibration_requirement,
        out_to_calibration: input.out_to_calibration,
        last_calibrated_at: input.last_calibrated_at,
        calibration_due_at: input.calibration_due_at,
        calibration_interval_months: input.calibration_interval_months,
        certificate_ref: input.certificate_ref,
        calibration_vendor: input.calibration_vendor,
        calibration_notes: input.calibration_notes,
        verified_at: input.verified_at,
        verified_by: input.verified_by,
        import_provenance: None,
        archived: input.archived,
        manual_entry: true,
        picture_path: input.picture_path.unwrap_or_default(),
        created_at: timestamp.clone(),
        updated_at: timestamp,
    }
}

pub(crate) fn update_entry_from_input(
    mut entry: InventoryEntry,
    input: InventoryEntryInput,
) -> InventoryEntry {
    entry.asset_number = input.asset_number;
    entry.serial_number = input.serial_number;
    entry.qty = input.qty;
    entry.manufacturer = input.manufacturer;
    entry.model = input.model;
    entry.description = input.description;
    entry.project_name = input.project_name;
    entry.location = input.location;
    entry.assigned_to = input.assigned_to;
    entry.links = input.links;
    entry.notes = input.notes;
    entry.lifecycle_status = input.lifecycle_status;
    entry.working_status = input.working_status;
    entry.condition = input.condition;
    entry.calibration_requirement = input.calibration_requirement;
    entry.out_to_calibration = input.out_to_calibration;
    entry.last_calibrated_at = input.last_calibrated_at;
    entry.calibration_due_at = input.calibration_due_at;
    entry.calibration_interval_months = input.calibration_interval_months;
    entry.certificate_ref = input.certificate_ref;
    entry.calibration_vendor = input.calibration_vendor;
    entry.calibration_notes = input.calibration_notes;
    entry.verified_at = input.verified_at;
    entry.verified_by = input.verified_by;
    entry.archived = input.archived;
    entry.picture_path = input.picture_path.unwrap_or_default();
    entry.updated_at = now_timestamp();
    entry
}

impl CalibrationRequirement {
    pub(crate) const fn display_label(self) -> &'static str {
        match self {
            Self::Required => "Required",
            Self::ReferenceOnly => "Reference only",
            Self::NotRequired => "Not required",
            Self::Unknown => "Unknown",
        }
    }
}

impl CalibrationHealth {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::MissingDue => "missing_due",
            Self::Overdue => "overdue",
            Self::DueSoon => "due_soon",
            Self::Current => "current",
            Self::NotApplicable => "not_applicable",
            Self::Unknown => "unknown",
            Self::OutToCal => "out_to_cal",
        }
    }
}

pub(crate) fn derive_calibration_health(
    entry: &InventoryEntry,
    local_date: NaiveDate,
    due_soon_days: u64,
) -> Option<CalibrationHealth> {
    if entry.archived {
        return None;
    }

    match entry.calibration_requirement {
        CalibrationRequirement::ReferenceOnly | CalibrationRequirement::NotRequired => {
            return Some(CalibrationHealth::NotApplicable);
        }
        CalibrationRequirement::Unknown => return Some(CalibrationHealth::Unknown),
        CalibrationRequirement::Required => {}
    }
    if entry.out_to_calibration {
        return Some(CalibrationHealth::OutToCal);
    }

    let Some(due_date) = entry.calibration_due_at.as_deref().and_then(parse_date) else {
        return Some(CalibrationHealth::MissingDue);
    };
    if due_date < local_date {
        return Some(CalibrationHealth::Overdue);
    }
    let due_soon_limit = local_date.checked_add_days(Days::new(due_soon_days));
    if due_soon_limit.is_some_and(|limit| due_date <= limit) {
        Some(CalibrationHealth::DueSoon)
    } else {
        Some(CalibrationHealth::Current)
    }
}

pub(crate) fn validate_inventory_entry(entry: &InventoryEntry) -> CommandResult<()> {
    validate_entry_input(&InventoryEntryInput {
        asset_number: entry.asset_number.clone(),
        serial_number: entry.serial_number.clone(),
        qty: entry.qty,
        manufacturer: entry.manufacturer.clone(),
        model: entry.model.clone(),
        description: entry.description.clone(),
        project_name: entry.project_name.clone(),
        location: entry.location.clone(),
        assigned_to: entry.assigned_to.clone(),
        links: entry.links.clone(),
        notes: entry.notes.clone(),
        lifecycle_status: entry.lifecycle_status.clone(),
        working_status: entry.working_status.clone(),
        condition: entry.condition.clone(),
        calibration_requirement: entry.calibration_requirement,
        out_to_calibration: entry.out_to_calibration,
        last_calibrated_at: entry.last_calibrated_at.clone(),
        calibration_due_at: entry.calibration_due_at.clone(),
        calibration_interval_months: entry.calibration_interval_months,
        certificate_ref: entry.certificate_ref.clone(),
        calibration_vendor: entry.calibration_vendor.clone(),
        calibration_notes: entry.calibration_notes.clone(),
        verified_at: entry.verified_at.clone(),
        verified_by: entry.verified_by.clone(),
        archived: entry.archived,
        picture_path: Some(entry.picture_path.clone()),
    })
}

fn normalize_enum(value: String, allowed: &[&str], fallback: &str) -> String {
    let trimmed = value.trim();
    if allowed.contains(&trimmed) {
        trimmed.to_string()
    } else {
        fallback.to_string()
    }
}

fn validate_text_length(field_name: &str, value: &str, max_chars: usize) -> CommandResult<()> {
    if value.chars().count() > max_chars {
        return Err(format!(
            "{field_name} must be {max_chars} characters or fewer."
        ));
    }

    Ok(())
}

fn normalize_optional_string(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_string())
    })
}

fn validate_optional_text_length(
    field_name: &str,
    value: Option<&str>,
    max_chars: usize,
) -> CommandResult<()> {
    if let Some(value) = value {
        validate_text_length(field_name, value, max_chars)?;
    }
    Ok(())
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn validate_optional_date(
    field_name: &str,
    value: Option<&str>,
) -> CommandResult<Option<NaiveDate>> {
    value
        .map(|value| {
            parse_date(value)
                .ok_or_else(|| format!("{field_name} must be a valid date in YYYY-MM-DD format."))
        })
        .transpose()
}

fn validate_links(value: &str) -> CommandResult<()> {
    if value.trim().is_empty() {
        return Ok(());
    }
    if looks_like_windows_path(value) {
        return Err("Links must be http, https, or mailto URLs.".to_string());
    }

    let url = parse_url_or_implicit_https(value)
        .ok_or_else(|| "Links must be http, https, or mailto URLs.".to_string())?;
    if LINK_PROTOCOLS.contains(&url.scheme()) {
        Ok(())
    } else {
        Err("Links must be http, https, or mailto URLs.".to_string())
    }
}

fn validate_picture_path(value: &str) -> CommandResult<()> {
    let value = value.trim();
    if value.is_empty() {
        return Ok(());
    }
    if value.starts_with("data:image/") {
        return Ok(());
    }
    if looks_like_windows_path(value) || Path::new(value).is_absolute() {
        return validate_picture_file_extension(value);
    }
    if let Ok(url) = Url::parse(value) {
        if PICTURE_URL_PROTOCOLS.contains(&url.scheme()) {
            return Ok(());
        }
        return Err(
            "Picture path must be an absolute image path or an http/https image URL.".to_string(),
        );
    }

    Err("Picture path must be an absolute image path or an http/https image URL.".to_string())
}

fn parse_url_or_implicit_https(value: &str) -> Option<Url> {
    Url::parse(value)
        .or_else(|_| Url::parse(&format!("https://{value}")))
        .ok()
}

fn looks_like_windows_path(value: &str) -> bool {
    let bytes = value.as_bytes();
    value.starts_with(r"\\")
        || (bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && matches!(bytes[2], b'\\' | b'/'))
}

fn validate_picture_file_extension(value: &str) -> CommandResult<()> {
    let extension = Path::new(value)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default();
    if IMAGE_PATH_EXTENSIONS
        .iter()
        .any(|allowed| extension.eq_ignore_ascii_case(allowed))
    {
        Ok(())
    } else {
        Err("Picture path must use a supported image extension.".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_identity_is_invalid() {
        let input = normalize_entry_input(InventoryEntryInput::default());

        assert!(validate_entry_input(&input).is_err());
    }

    #[test]
    fn normalize_trims_text_and_defaults_enums() {
        let input = normalize_entry_input(InventoryEntryInput {
            manufacturer: "  Mitutoyo ".to_string(),
            lifecycle_status: "bad".to_string(),
            working_status: "also_bad".to_string(),
            picture_path: Some(" C:\\Pictures\\part.jpg ".to_string()),
            ..InventoryEntryInput::default()
        });

        assert_eq!(input.manufacturer, "Mitutoyo");
        assert_eq!(input.lifecycle_status, "active");
        assert_eq!(input.working_status, "unknown");
        assert_eq!(
            input.picture_path.as_deref(),
            Some("C:\\Pictures\\part.jpg")
        );
    }

    #[test]
    fn validation_rejects_unreasonable_quantities() {
        let mut input = normalize_entry_input(InventoryEntryInput {
            description: "Caliper".to_string(),
            qty: Some(-1.0),
            ..InventoryEntryInput::default()
        });
        assert!(validate_entry_input(&input)
            .unwrap_err()
            .contains("Quantity"));

        input.qty = Some(f64::NAN);
        assert!(validate_entry_input(&input)
            .unwrap_err()
            .contains("Quantity"));

        input.qty = Some(MAX_QUANTITY + 1.0);
        assert!(validate_entry_input(&input)
            .unwrap_err()
            .contains("Quantity"));
    }

    #[test]
    fn validation_rejects_overlong_text_fields() {
        let input = normalize_entry_input(InventoryEntryInput {
            description: "x".repeat(LONG_TEXT_LIMIT + 1),
            manufacturer: "Mitutoyo".to_string(),
            ..InventoryEntryInput::default()
        });

        assert!(validate_entry_input(&input)
            .unwrap_err()
            .contains("Description"));
    }

    #[test]
    fn validation_rejects_unsafe_link_values() {
        let valid = normalize_entry_input(InventoryEntryInput {
            description: "Caliper".to_string(),
            links: "https://example.com/manual".to_string(),
            ..InventoryEntryInput::default()
        });
        assert!(validate_entry_input(&valid).is_ok());

        let invalid = normalize_entry_input(InventoryEntryInput {
            description: "Caliper".to_string(),
            links: "C:\\Pictures\\part.jpg".to_string(),
            ..InventoryEntryInput::default()
        });
        assert!(validate_entry_input(&invalid)
            .unwrap_err()
            .contains("Links"));
    }

    #[test]
    fn validation_rejects_unsafe_picture_paths() {
        let valid = normalize_entry_input(InventoryEntryInput {
            description: "Caliper".to_string(),
            picture_path: Some("C:\\Pictures\\part.png".to_string()),
            ..InventoryEntryInput::default()
        });
        assert!(validate_entry_input(&valid).is_ok());

        let invalid = normalize_entry_input(InventoryEntryInput {
            description: "Caliper".to_string(),
            picture_path: Some("relative\\part.txt".to_string()),
            ..InventoryEntryInput::default()
        });
        assert!(validate_entry_input(&invalid)
            .unwrap_err()
            .contains("Picture path"));
    }

    #[test]
    fn calibration_health_obeys_precedence_and_due_boundaries() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 13).unwrap();
        let mut entry = create_entry_from_input(
            1,
            normalize_entry_input(InventoryEntryInput {
                description: "Reference meter".to_string(),
                calibration_requirement: CalibrationRequirement::Required,
                calibration_due_at: Some("2026-07-12".to_string()),
                ..InventoryEntryInput::default()
            }),
        );

        assert_eq!(
            derive_calibration_health(&entry, today, 30),
            Some(CalibrationHealth::Overdue)
        );
        entry.calibration_due_at = Some("2026-07-13".to_string());
        assert_eq!(
            derive_calibration_health(&entry, today, 30),
            Some(CalibrationHealth::DueSoon)
        );
        entry.calibration_due_at = Some("2026-08-12".to_string());
        assert_eq!(
            derive_calibration_health(&entry, today, 30),
            Some(CalibrationHealth::DueSoon)
        );
        entry.calibration_due_at = Some("2026-08-13".to_string());
        assert_eq!(
            derive_calibration_health(&entry, today, 30),
            Some(CalibrationHealth::Current)
        );
        entry.calibration_due_at = None;
        assert_eq!(
            derive_calibration_health(&entry, today, 30),
            Some(CalibrationHealth::MissingDue)
        );

        entry.out_to_calibration = true;
        assert_eq!(
            derive_calibration_health(&entry, today, 30),
            Some(CalibrationHealth::OutToCal)
        );
        entry.calibration_requirement = CalibrationRequirement::ReferenceOnly;
        assert_eq!(
            derive_calibration_health(&entry, today, 30),
            Some(CalibrationHealth::NotApplicable)
        );
        entry.calibration_requirement = CalibrationRequirement::Unknown;
        assert_eq!(
            derive_calibration_health(&entry, today, 30),
            Some(CalibrationHealth::Unknown)
        );
        entry.archived = true;
        assert_eq!(derive_calibration_health(&entry, today, 30), None);
    }

    #[test]
    fn calibration_normalization_and_validation_preserve_explicit_due_authority() {
        let input = normalize_entry_input(InventoryEntryInput {
            description: "Oscilloscope".to_string(),
            calibration_requirement: CalibrationRequirement::Required,
            last_calibrated_at: Some(" 2026-07-10 ".to_string()),
            calibration_due_at: Some(" 2026-07-09 ".to_string()),
            calibration_interval_months: Some(12),
            certificate_ref: Some("  CERT-123 ".to_string()),
            calibration_vendor: Some("   ".to_string()),
            calibration_notes: Some("  vendor note  ".to_string()),
            verified_at: Some(" 2026-07-13T12:00:00Z ".to_string()),
            verified_by: Some("  Avery  ".to_string()),
            ..InventoryEntryInput::default()
        });

        assert_eq!(input.last_calibrated_at.as_deref(), Some("2026-07-10"));
        assert_eq!(input.calibration_due_at.as_deref(), Some("2026-07-09"));
        assert_eq!(input.certificate_ref.as_deref(), Some("CERT-123"));
        assert_eq!(input.calibration_vendor, None);
        assert_eq!(input.calibration_notes.as_deref(), Some("vendor note"));
        assert_eq!(input.verified_at.as_deref(), Some("2026-07-13T12:00:00Z"));
        assert_eq!(input.verified_by.as_deref(), Some("Avery"));
        assert!(validate_entry_input(&input)
            .unwrap_err()
            .contains("due date"));

        let mut invalid_date = input.clone();
        invalid_date.calibration_due_at = Some("2026-02-30".to_string());
        assert!(validate_entry_input(&invalid_date)
            .unwrap_err()
            .contains("Calibration due date"));

        let mut zero_interval = input.clone();
        zero_interval.last_calibrated_at = None;
        zero_interval.calibration_due_at = None;
        zero_interval.calibration_interval_months = Some(0);
        assert!(validate_entry_input(&zero_interval)
            .unwrap_err()
            .contains("interval"));

        let mut invalid_verified_at = zero_interval;
        invalid_verified_at.calibration_interval_months = Some(12);
        invalid_verified_at.verified_at = Some("yesterday".to_string());
        assert!(validate_entry_input(&invalid_verified_at)
            .unwrap_err()
            .contains("Verified at"));
    }

    #[test]
    fn legacy_verified_true_uses_valid_updated_at_with_explicit_approximation_label() {
        let legacy = serde_json::json!({
            "id": "1",
            "entryUuid": "legacy-1",
            "description": "Legacy meter",
            "verifiedInSurvey": true,
            "updatedAt": "2026-07-13T12:00:00Z"
        });

        let decoded: InventoryEntry = serde_json::from_value(legacy).unwrap();

        assert_eq!(decoded.verified_at.as_deref(), Some("2026-07-13T12:00:00Z"));
        assert_eq!(
            decoded.verified_by.as_deref(),
            Some(LEGACY_VERIFIED_APPROXIMATION_LABEL)
        );
        let serialized = serde_json::to_value(&decoded).unwrap();
        assert!(serialized.get("verifiedInSurvey").is_none());
        assert_eq!(
            serialized
                .get("verifiedBy")
                .and_then(|value| value.as_str()),
            Some(LEGACY_VERIFIED_APPROXIMATION_LABEL)
        );
        let round_tripped: InventoryEntry = serde_json::from_value(serialized).unwrap();
        assert_eq!(
            round_tripped.verified_at.as_deref(),
            Some("2026-07-13T12:00:00Z")
        );
        assert_eq!(
            round_tripped.verified_by.as_deref(),
            Some(LEGACY_VERIFIED_APPROXIMATION_LABEL)
        );
    }

    #[test]
    fn legacy_verified_missing_or_false_maps_to_no_verification() {
        for legacy in [
            serde_json::json!({
                "id": "1",
                "entryUuid": "legacy-missing",
                "description": "Legacy meter",
                "updatedAt": "2026-07-13T12:00:00Z"
            }),
            serde_json::json!({
                "id": "2",
                "entryUuid": "legacy-false",
                "description": "Legacy meter",
                "verifiedInSurvey": false,
                "updatedAt": "2026-07-13T12:00:00Z"
            }),
        ] {
            let decoded: InventoryEntry = serde_json::from_value(legacy).unwrap();
            assert_eq!(decoded.verified_at, None);
            assert_eq!(decoded.verified_by, None);
        }
    }

    #[test]
    fn legacy_verified_true_without_valid_updated_at_does_not_invent_timestamp() {
        for updated_at in [None, Some("not-a-timestamp")] {
            let mut legacy = serde_json::json!({
                "id": "1",
                "entryUuid": "legacy-1",
                "description": "Legacy meter",
                "verifiedInSurvey": true
            });
            if let Some(updated_at) = updated_at {
                legacy["updatedAt"] = serde_json::Value::String(updated_at.to_string());
            }

            let decoded: InventoryEntry = serde_json::from_value(legacy).unwrap();

            assert_eq!(decoded.verified_at, None);
            assert_eq!(decoded.verified_by, None);
        }
    }

    #[test]
    fn explicit_verification_fields_win_over_legacy_flag() {
        let mixed = serde_json::json!({
            "id": "1",
            "entryUuid": "legacy-1",
            "description": "Legacy meter",
            "verifiedInSurvey": true,
            "updatedAt": "2026-07-13T12:00:00Z",
            "verifiedAt": "2026-07-14T13:00:00Z",
            "verifiedBy": "Taylor"
        });

        let decoded: InventoryEntry = serde_json::from_value(mixed).unwrap();

        assert_eq!(decoded.verified_at.as_deref(), Some("2026-07-14T13:00:00Z"));
        assert_eq!(decoded.verified_by.as_deref(), Some("Taylor"));
    }

    #[test]
    fn import_provenance_round_trips_and_normal_edits_preserve_it() {
        let encoded = serde_json::json!({
            "id": "8",
            "entryUuid": "imported-8",
            "description": "Imported meter",
            "importProvenance": {
                "batchId": "sha256:batch",
                "sourceFilename": "inventory.xlsx",
                "sourceSheet": "Equipment",
                "sourceRow": 12,
                "originalId": "legacy-88",
                "originalAssetNumber": " TE-008 ",
                "originalSerialNumber": "SN-008"
            }
        });
        let entry: InventoryEntry = serde_json::from_value(encoded).unwrap();

        let updated = update_entry_from_input(
            entry.clone(),
            normalize_entry_input(InventoryEntryInput {
                description: "Edited meter".to_string(),
                ..InventoryEntryInput::default()
            }),
        );

        assert_eq!(updated.import_provenance, entry.import_provenance);
        assert_eq!(
            updated
                .import_provenance
                .as_ref()
                .and_then(|value| value.source_sheet.as_deref()),
            Some("Equipment")
        );
        assert_eq!(
            serde_json::to_value(&updated).unwrap()["importProvenance"]["sourceRow"],
            12
        );
    }
}
