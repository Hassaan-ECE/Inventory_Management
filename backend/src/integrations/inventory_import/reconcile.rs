use std::collections::{BTreeMap, HashMap};

use crate::{
    model::{
        normalize_entry_input, validate_entry_input, CalibrationRequirement, CommandResult,
        InventoryEntryInput,
    },
    store::InventoryDb,
};
use chrono::{DateTime, NaiveDate};

use super::{
    mapping::{map_header, MappingDisposition, SourceTarget},
    parser::{hex_digest, ParsedRow, ParsedSource},
    ImportClassification, ImportColumnReport, ImportColumnTreatment, ImportDryRunReport,
    ImportRowOutcome, IMPORT_MAPPING_VERSION,
};

#[derive(Debug, Clone)]
pub(super) struct ReconciledImport {
    pub report: ImportDryRunReport,
    pub rows: Vec<ReconciledRow>,
}

#[derive(Debug, Clone)]
pub(super) struct ReconciledRow {
    pub source_row: u64,
    pub classification: ImportClassification,
    pub input: Option<InventoryEntryInput>,
    pub original_id: Option<String>,
    pub original_asset_number: Option<String>,
    pub original_serial_number: Option<String>,
}

pub(super) fn reconcile_source(
    source: &ParsedSource,
    db: &InventoryDb,
    exclude_batch_id: Option<&str>,
) -> CommandResult<ReconciledImport> {
    let mappings = source
        .headers
        .iter()
        .map(|header| map_header(header))
        .collect::<Vec<_>>();
    let columns = source
        .headers
        .iter()
        .enumerate()
        .map(|(index, header)| column_report(header, mappings[index], source, index))
        .collect::<Vec<_>>();
    let batch_id = batch_id(source);
    let entries = db.load_entries()?;
    let matching_entries = entries
        .iter()
        .filter(|entry| {
            exclude_batch_id.is_none_or(|batch_id| {
                entry
                    .import_provenance
                    .as_ref()
                    .is_none_or(|provenance| provenance.batch_id != batch_id)
            })
        })
        .cloned()
        .collect::<Vec<_>>();
    let asset_index = identity_index(&matching_entries, |entry| &entry.asset_number);
    let serial_index = identity_index(&matching_entries, |entry| &entry.serial_number);
    let basis = reconciliation_basis(&entries, exclude_batch_id);

    let mut built_rows = source
        .rows
        .iter()
        .map(|row| build_row(source, &mappings, row))
        .collect::<Vec<_>>();
    reconcile_source_identities(&mut built_rows);
    for built in &mut built_rows {
        if built.classification == ImportClassification::Inserted {
            reconcile_identity(built, &asset_index, &serial_index);
        }
    }

    let mut outcomes = Vec::with_capacity(source.rows.len());
    let mut reconciled_rows = Vec::with_capacity(source.rows.len());
    let mut totals = [0usize; 5];
    for (row, built) in source.rows.iter().zip(built_rows) {
        increment_total(&mut totals, built.classification);
        outcomes.push(built.outcome.clone());
        reconciled_rows.push(ReconciledRow {
            source_row: row.source_row,
            classification: built.classification,
            input: built.input,
            original_id: built.outcome.original_id,
            original_asset_number: built.outcome.original_asset_number,
            original_serial_number: built.outcome.original_serial_number,
        });
    }

    let [inserted, matched, conflicted, rejected, ignored] = totals;
    let report = ImportDryRunReport {
        batch_id,
        source_fingerprint: source.fingerprint.clone(),
        source_filename: source.filename.clone(),
        selected_sheet: source.selected_sheet.clone(),
        mapping_version: IMPORT_MAPPING_VERSION.to_string(),
        total_rows: source.rows.len(),
        inserted,
        matched,
        conflicted,
        rejected,
        ignored,
        columns,
        row_outcomes: outcomes,
        blocking: conflicted > 0 || rejected > 0,
        reconciliation_basis: basis,
    };
    debug_assert_eq!(
        report.total_rows,
        report.inserted + report.matched + report.conflicted + report.rejected + report.ignored
    );
    Ok(ReconciledImport {
        report,
        rows: reconciled_rows,
    })
}

fn batch_id(source: &ParsedSource) -> String {
    let material = format!(
        "{}\0{}\0{}",
        source.fingerprint, source.selected_sheet, IMPORT_MAPPING_VERSION
    );
    format!("batch-{}", hex_digest(material.as_bytes()))
}

fn reconciliation_basis(
    entries: &[crate::model::InventoryEntry],
    exclude_batch_id: Option<&str>,
) -> String {
    let mut rows = entries
        .iter()
        .filter(|entry| {
            exclude_batch_id.is_none_or(|batch_id| {
                entry
                    .import_provenance
                    .as_ref()
                    .is_none_or(|provenance| provenance.batch_id != batch_id)
            })
        })
        .map(|entry| {
            format!(
                "{}\0{}\0{}\0{}\0{}",
                entry.entry_uuid,
                normalize_identity(&entry.asset_number),
                normalize_identity(&entry.serial_number),
                entry.updated_at,
                entry.archived
            )
        })
        .collect::<Vec<_>>();
    rows.sort();
    format!("basis-{}", hex_digest(rows.join("\n").as_bytes()))
}

fn column_report(
    header: &str,
    mapping: MappingDisposition,
    source: &ParsedSource,
    index: usize,
) -> ImportColumnReport {
    let nonblank_count = source
        .rows
        .iter()
        .filter(|row| {
            row.values
                .get(index)
                .is_some_and(|value| !value.trim().is_empty())
        })
        .count();
    match mapping {
        MappingDisposition::Mapped(target) => ImportColumnReport {
            original_header: header.to_string(),
            normalized_target: Some(target.name().to_string()),
            treatment: ImportColumnTreatment::Mapped,
            nonblank_count,
            reason: format!("Mapped to {}.", target.name()),
        },
        MappingDisposition::IntentionallyIgnored(reason) => ImportColumnReport {
            original_header: header.to_string(),
            normalized_target: None,
            treatment: ImportColumnTreatment::IntentionallyIgnored,
            nonblank_count,
            reason: reason.to_string(),
        },
        MappingDisposition::Unknown => ImportColumnReport {
            original_header: header.to_string(),
            normalized_target: None,
            treatment: ImportColumnTreatment::Unknown,
            nonblank_count,
            reason: "Unknown source column; any nonblank value rejects its row.".to_string(),
        },
    }
}

struct BuiltRow {
    classification: ImportClassification,
    input: Option<InventoryEntryInput>,
    outcome: ImportRowOutcome,
}

fn build_row(source: &ParsedSource, mappings: &[MappingDisposition], row: &ParsedRow) -> BuiltRow {
    let raw_values = raw_values(source, row);
    let all_blank = row.values.iter().all(|value| value.trim().is_empty());
    if all_blank {
        return BuiltRow {
            classification: ImportClassification::Ignored,
            input: None,
            outcome: ImportRowOutcome {
                source_row: row.source_row,
                classification: ImportClassification::Ignored,
                issues: vec!["Blank source row intentionally ignored.".to_string()],
                original_id: None,
                original_asset_number: None,
                original_serial_number: None,
                candidate_entry_uuid: None,
                raw_values,
            },
        };
    }

    let mut values: HashMap<SourceTarget, Vec<&str>> = HashMap::new();
    let mut issues = Vec::new();
    let mut rejected = false;
    for index in &source.unheaded_columns {
        if row
            .values
            .get(*index)
            .is_some_and(|value| !value.trim().is_empty())
        {
            issues.push(format!(
                "Nonblank value in unheaded column {} exceeds the declared header width.",
                index + 1
            ));
            rejected = true;
        }
    }
    for (index, mapping) in mappings.iter().enumerate() {
        let value = row
            .values
            .get(index)
            .map(String::as_str)
            .unwrap_or_default();
        if value.starts_with("#EXCEL_ERROR:") {
            issues.push(format!(
                "Excel error cell in source column '{}'.",
                source.headers[index]
            ));
            rejected = true;
        }
        match mapping {
            MappingDisposition::Mapped(target) => values.entry(*target).or_default().push(value),
            MappingDisposition::Unknown if !value.trim().is_empty() => {
                issues.push(format!(
                    "Unknown column '{}' contains a nonblank value.",
                    source.headers[index]
                ));
                rejected = true;
            }
            _ => {}
        }
    }

    for (target, target_values) in &values {
        let nonblank = target_values
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        if nonblank.len() > 1 && nonblank.windows(2).any(|pair| pair[0] != pair[1]) {
            issues.push(format!(
                "Multiple columns mapped to {} with different values.",
                target.name()
            ));
            rejected = true;
        }
    }

    let get = |target| first_value(&values, target);
    let original_id = optional_raw(get(SourceTarget::OriginalId));
    let original_asset_number = optional_raw(get(SourceTarget::AssetNumber));
    let original_serial_number = optional_raw(get(SourceTarget::SerialNumber));
    let mut input = InventoryEntryInput {
        asset_number: text(get(SourceTarget::AssetNumber)),
        serial_number: text(get(SourceTarget::SerialNumber)),
        manufacturer: text(get(SourceTarget::Manufacturer)),
        model: text(get(SourceTarget::Model)),
        description: text(get(SourceTarget::Description)),
        project_name: text(get(SourceTarget::ProjectName)),
        location: text(get(SourceTarget::Location)),
        assigned_to: text(get(SourceTarget::AssignedTo)),
        links: text(get(SourceTarget::Links)),
        notes: text(get(SourceTarget::Notes)),
        lifecycle_status: "active".to_string(),
        working_status: "unknown".to_string(),
        condition: text(get(SourceTarget::Condition)),
        certificate_ref: optional_raw(get(SourceTarget::CertificateRef)),
        calibration_vendor: optional_raw(get(SourceTarget::CalibrationVendor)),
        calibration_notes: optional_raw(get(SourceTarget::CalibrationNotes)),
        verified_by: optional_raw(get(SourceTarget::VerifiedBy)),
        picture_path: optional_raw(get(SourceTarget::PicturePath)),
        ..InventoryEntryInput::default()
    };

    parse_quantity(
        get(SourceTarget::Quantity),
        &mut input,
        &mut issues,
        &mut rejected,
    );
    parse_enum(
        "lifecycle status",
        get(SourceTarget::LifecycleStatus),
        &["active", "repair", "scrapped", "missing", "rental"],
        &mut input.lifecycle_status,
        &mut issues,
        &mut rejected,
    );
    parse_enum(
        "working status",
        get(SourceTarget::WorkingStatus),
        &["unknown", "working", "limited", "not_working"],
        &mut input.working_status,
        &mut issues,
        &mut rejected,
    );
    parse_calibration(&values, &mut input, &mut issues, &mut rejected);
    input.last_calibrated_at = parse_date(
        "last calibrated date",
        get(SourceTarget::LastCalibratedAt),
        &mut issues,
        &mut rejected,
    );
    input.calibration_due_at = parse_date(
        "calibration due date",
        get(SourceTarget::CalibrationDueAt),
        &mut issues,
        &mut rejected,
    );
    parse_interval(
        get(SourceTarget::CalibrationIntervalMonths),
        &mut input,
        &mut issues,
        &mut rejected,
    );
    parse_verified(&values, &mut input, &mut issues, &mut rejected);
    parse_bool_field(
        "archived",
        get(SourceTarget::Archived),
        &mut input.archived,
        &mut issues,
        &mut rejected,
    );
    input = normalize_entry_input(input);
    if let Err(error) = validate_entry_input(&input) {
        issues.push(error);
        rejected = true;
    }
    let classification = if rejected {
        ImportClassification::Rejected
    } else {
        ImportClassification::Inserted
    };
    BuiltRow {
        classification,
        input: (!rejected).then_some(input),
        outcome: ImportRowOutcome {
            source_row: row.source_row,
            classification,
            issues,
            original_id,
            original_asset_number,
            original_serial_number,
            candidate_entry_uuid: None,
            raw_values,
        },
    }
}

fn reconcile_source_identities(rows: &mut [BuiltRow]) {
    let mut asset_index: HashMap<String, Vec<usize>> = HashMap::new();
    let mut serial_index: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, row) in rows.iter().enumerate() {
        if row.classification != ImportClassification::Inserted {
            continue;
        }
        let Some(input) = row.input.as_ref() else {
            continue;
        };
        let asset = normalize_identity(&input.asset_number);
        if !asset.is_empty() {
            asset_index.entry(asset).or_default().push(index);
        }
        let serial = normalize_identity(&input.serial_number);
        if !serial.is_empty() {
            serial_index.entry(serial).or_default().push(index);
        }
    }

    let mut duplicate_asset = vec![false; rows.len()];
    for indexes in asset_index.values().filter(|indexes| indexes.len() > 1) {
        for index in indexes {
            duplicate_asset[*index] = true;
        }
    }
    let mut duplicate_serial = vec![false; rows.len()];
    for indexes in serial_index.values().filter(|indexes| indexes.len() > 1) {
        for index in indexes {
            duplicate_serial[*index] = true;
        }
    }

    for index in 0..rows.len() {
        let issue = match (duplicate_asset[index], duplicate_serial[index]) {
            (true, true) => Some(
                "Normalized asset and serial identities are duplicated within the source batch.",
            ),
            (true, false) => {
                Some("Normalized asset identity is duplicated within the source batch.")
            }
            (false, true) => {
                Some("Normalized serial identity is duplicated within the source batch.")
            }
            (false, false) => None,
        };
        if let Some(issue) = issue {
            set_conflict(&mut rows[index], issue);
        }
    }
}

fn reconcile_identity(
    row: &mut BuiltRow,
    asset_index: &HashMap<String, Vec<String>>,
    serial_index: &HashMap<String, Vec<String>>,
) {
    let Some(input) = row.input.as_ref() else {
        return;
    };
    let asset = normalize_identity(&input.asset_number);
    let serial = normalize_identity(&input.serial_number);
    let assets = (!asset.is_empty())
        .then(|| asset_index.get(&asset))
        .flatten();
    let serials = (!serial.is_empty())
        .then(|| serial_index.get(&serial))
        .flatten();
    if assets.is_some_and(|matches| matches.len() > 1)
        || serials.is_some_and(|matches| matches.len() > 1)
    {
        set_conflict(
            row,
            "A normalized asset or serial key has duplicate database candidates.",
        );
        return;
    }
    let asset_match = assets.and_then(|matches| matches.first());
    let serial_match = serials.and_then(|matches| matches.first());
    match (asset_match, serial_match) {
        (Some(asset_uuid), Some(serial_uuid)) if asset_uuid != serial_uuid => {
            set_conflict(
                row,
                "Asset and serial keys resolve to different database entries.",
            );
        }
        (Some(entry_uuid), _) | (_, Some(entry_uuid)) => {
            row.classification = ImportClassification::Matched;
            row.outcome.classification = ImportClassification::Matched;
            row.outcome.candidate_entry_uuid = Some(entry_uuid.clone());
            row.outcome
                .issues
                .push("Unique normalized asset/serial identity matched; v0.1 commit leaves the existing entry unchanged.".to_string());
        }
        (None, None) if asset.is_empty() && serial.is_empty() => {
            row.outcome.issues.push(
                "No asset or serial key was supplied; manufacturer/model were not used for auto-match."
                    .to_string(),
            );
        }
        (None, None) => {}
    }
}

fn set_conflict(row: &mut BuiltRow, issue: &str) {
    row.classification = ImportClassification::Conflicted;
    row.outcome.classification = ImportClassification::Conflicted;
    row.outcome.issues.push(issue.to_string());
    row.input = None;
}

fn identity_index(
    entries: &[crate::model::InventoryEntry],
    select: impl Fn(&crate::model::InventoryEntry) -> &String,
) -> HashMap<String, Vec<String>> {
    let mut index: HashMap<String, Vec<String>> = HashMap::new();
    for entry in entries {
        let key = normalize_identity(select(entry));
        if !key.is_empty() {
            index.entry(key).or_default().push(entry.entry_uuid.clone());
        }
    }
    index
}

fn normalize_identity(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn raw_values(source: &ParsedSource, row: &ParsedRow) -> BTreeMap<String, String> {
    let mut header_counts: HashMap<&str, usize> = HashMap::new();
    for header in &source.headers {
        *header_counts.entry(header.as_str()).or_default() += 1;
    }
    source
        .headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            let key = if header_counts
                .get(header.as_str())
                .copied()
                .unwrap_or_default()
                > 1
            {
                format!("{header} [column {}]", index + 1)
            } else {
                header.clone()
            };
            (key, row.values.get(index).cloned().unwrap_or_default())
        })
        .collect()
}

fn first_value<'a>(
    values: &'a HashMap<SourceTarget, Vec<&'a str>>,
    target: SourceTarget,
) -> Option<&'a str> {
    values
        .get(&target)
        .and_then(|items| items.iter().find(|value| !value.trim().is_empty()).copied())
}

fn optional_raw(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn text(value: Option<&str>) -> String {
    optional_raw(value).unwrap_or_default()
}

fn parse_quantity(
    value: Option<&str>,
    input: &mut InventoryEntryInput,
    issues: &mut Vec<String>,
    rejected: &mut bool,
) {
    let Some(value) = optional_raw(value) else {
        return;
    };
    match value.parse::<f64>() {
        Ok(quantity) => input.qty = Some(quantity),
        Err(_) => reject(issues, rejected, "Quantity is not a valid number."),
    }
}

fn parse_enum(
    label: &str,
    value: Option<&str>,
    allowed: &[&str],
    output: &mut String,
    issues: &mut Vec<String>,
    rejected: &mut bool,
) {
    let Some(value) = optional_raw(value) else {
        return;
    };
    let normalized = value.to_ascii_lowercase().replace([' ', '-'], "_");
    if allowed.contains(&normalized.as_str()) {
        *output = normalized;
    } else {
        reject(
            issues,
            rejected,
            &format!("Unrecognized {label} '{value}'."),
        );
    }
}

fn parse_calibration(
    values: &HashMap<SourceTarget, Vec<&str>>,
    input: &mut InventoryEntryInput,
    issues: &mut Vec<String>,
    rejected: &mut bool,
) {
    let legacy = optional_raw(first_value(values, SourceTarget::CalibrationStatus))
        .map(|value| value.to_ascii_lowercase().replace([' ', '-'], "_"));
    let explicit_requirement =
        optional_raw(first_value(values, SourceTarget::CalibrationRequirement))
            .map(|value| value.to_ascii_lowercase().replace([' ', '-'], "_"));
    let explicit_out = parse_optional_bool(
        "out to calibration",
        first_value(values, SourceTarget::OutToCalibration),
        issues,
        rejected,
    );

    let legacy_mapping = match legacy.as_deref() {
        None | Some("") | Some("unknown") => Some((CalibrationRequirement::Unknown, false, false)),
        Some("calibrated") => Some((CalibrationRequirement::Required, false, true)),
        Some("out_to_cal") | Some("out_to_calibration") => {
            Some((CalibrationRequirement::Required, true, true))
        }
        Some("reference_only") => Some((CalibrationRequirement::ReferenceOnly, false, true)),
        Some("not_required") => Some((CalibrationRequirement::NotRequired, false, true)),
        Some(value) => {
            reject(
                issues,
                rejected,
                &format!("Unrecognized legacy calibration status '{value}'."),
            );
            None
        }
    };
    let requirement = match explicit_requirement.as_deref() {
        None | Some("") => None,
        Some("required") => Some(CalibrationRequirement::Required),
        Some("reference_only") => Some(CalibrationRequirement::ReferenceOnly),
        Some("not_required") => Some(CalibrationRequirement::NotRequired),
        Some("unknown") => Some(CalibrationRequirement::Unknown),
        Some(value) => {
            reject(
                issues,
                rejected,
                &format!("Unrecognized calibration requirement '{value}'."),
            );
            None
        }
    };
    if let Some((legacy_requirement, legacy_out, authoritative)) = legacy_mapping {
        if authoritative
            && (requirement.is_some_and(|value| value != legacy_requirement)
                || explicit_out.is_some_and(|value| value != legacy_out))
        {
            reject(
                issues,
                rejected,
                "Separate calibration requirement/out flag contradicts legacy calibration status.",
            );
        }
        input.calibration_requirement = requirement.unwrap_or(legacy_requirement);
        input.out_to_calibration = explicit_out.unwrap_or(legacy_out);
    } else {
        input.calibration_requirement = requirement.unwrap_or_default();
        input.out_to_calibration = explicit_out.unwrap_or(false);
    }
}

fn parse_date(
    label: &str,
    value: Option<&str>,
    issues: &mut Vec<String>,
    rejected: &mut bool,
) -> Option<String> {
    let value = optional_raw(value)?;
    for format in ["%Y-%m-%d", "%Y/%m/%d", "%d %b %Y", "%d %B %Y"] {
        if let Ok(date) = NaiveDate::parse_from_str(&value, format) {
            return Some(date.format("%Y-%m-%d").to_string());
        }
    }
    reject(
        issues,
        rejected,
        &format!("Invalid or ambiguous {label} '{value}'. Use YYYY-MM-DD."),
    );
    None
}

fn parse_interval(
    value: Option<&str>,
    input: &mut InventoryEntryInput,
    issues: &mut Vec<String>,
    rejected: &mut bool,
) {
    let Some(value) = optional_raw(value) else {
        return;
    };
    match value.parse::<u16>() {
        Ok(months) if months > 0 => input.calibration_interval_months = Some(months),
        _ => reject(
            issues,
            rejected,
            "Calibration interval months must be a positive whole number.",
        ),
    }
}

fn parse_verified(
    values: &HashMap<SourceTarget, Vec<&str>>,
    input: &mut InventoryEntryInput,
    issues: &mut Vec<String>,
    rejected: &mut bool,
) {
    let verified_at = optional_raw(first_value(values, SourceTarget::VerifiedAt));
    if let Some(value) = verified_at {
        if DateTime::parse_from_rfc3339(&value).is_ok() {
            input.verified_at = Some(value);
        } else {
            reject(
                issues,
                rejected,
                "Verified at must be a valid RFC 3339 timestamp.",
            );
        }
    }
    if let Some(flag) = parse_optional_bool(
        "verified",
        first_value(values, SourceTarget::Verified),
        issues,
        rejected,
    ) {
        if flag && input.verified_at.is_none() {
            issues.push("Timeless verified flag ignored; re-verification required".to_string());
        }
    }
}

fn parse_bool_field(
    label: &str,
    value: Option<&str>,
    output: &mut bool,
    issues: &mut Vec<String>,
    rejected: &mut bool,
) {
    if let Some(value) = parse_optional_bool(label, value, issues, rejected) {
        *output = value;
    }
}

fn parse_optional_bool(
    label: &str,
    value: Option<&str>,
    issues: &mut Vec<String>,
    rejected: &mut bool,
) -> Option<bool> {
    let value = optional_raw(value)?;
    match value.to_ascii_lowercase().as_str() {
        "true" | "yes" | "y" | "1" => Some(true),
        "false" | "no" | "n" | "0" => Some(false),
        _ => {
            reject(
                issues,
                rejected,
                &format!("Unrecognized {label} boolean '{value}'."),
            );
            None
        }
    }
}

fn reject(issues: &mut Vec<String>, rejected: &mut bool, issue: &str) {
    issues.push(issue.to_string());
    *rejected = true;
}

fn increment_total(totals: &mut [usize; 5], classification: ImportClassification) {
    totals[match classification {
        ImportClassification::Inserted => 0,
        ImportClassification::Matched => 1,
        ImportClassification::Conflicted => 2,
        ImportClassification::Rejected => 3,
        ImportClassification::Ignored => 4,
    }] += 1;
}
