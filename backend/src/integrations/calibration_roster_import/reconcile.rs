use std::collections::{BTreeSet, HashMap, HashSet};

use chrono::NaiveDate;

use crate::{
    model::{CalibrationRequirement, CommandResult, InventoryEntry, InventoryEntryInput},
    store::InventoryDb,
};

use super::{
    parser::{hex_digest, ParsedRosterFields, ParsedRosterRow, ParsedRosterSource},
    types::{
        CalibrationRosterAbsentEntry, CalibrationRosterCandidateContext,
        CalibrationRosterClassification, CalibrationRosterCounts, CalibrationRosterFieldChange,
        CalibrationRosterPreviewReport, CalibrationRosterRowOutcome, CalibrationRosterSemanticFlag,
    },
    CALIBRATION_ROSTER_MAPPING_VERSION,
};

pub(super) fn reconcile_roster_source(
    source: &ParsedRosterSource,
    db: &InventoryDb,
) -> CommandResult<CalibrationRosterPreviewReport> {
    let entries = db.load_entries()?;
    let asset_index = identity_index(&entries, |entry| &entry.asset_number);
    let serial_index = identity_index(&entries, |entry| &entry.serial_number);
    let mut built_rows = source
        .rows
        .iter()
        .map(|row| build_row(row, &entries, &asset_index, &serial_index))
        .collect::<Vec<_>>();
    mark_source_duplicates(&mut built_rows);

    let row_outcomes = built_rows
        .iter()
        .map(|row| row.outcome.clone())
        .collect::<Vec<_>>();
    let unambiguous_workbook_entries = built_rows
        .iter()
        .filter(|row| row.outcome.classification == CalibrationRosterClassification::MatchedUpdate)
        .filter_map(|row| row.target_entry_uuid.clone())
        .collect::<HashSet<_>>();
    let current_required_absent = entries
        .iter()
        .filter(|entry| is_active_required(entry))
        .filter(|entry| !unambiguous_workbook_entries.contains(&entry.entry_uuid))
        .map(absent_entry)
        .collect::<Vec<_>>();
    let counts = count_outcomes(&row_outcomes, current_required_absent.len());
    let prospective_required_count = prospective_required_count(&built_rows, &entries);
    let blocking = row_outcomes.iter().any(|row| row.requires_review);

    Ok(CalibrationRosterPreviewReport {
        batch_id: batch_id(source),
        source_fingerprint: source.fingerprint.clone(),
        source_filename: source.filename.clone(),
        mapping_version: CALIBRATION_ROSTER_MAPPING_VERSION.to_string(),
        contributing_sheets: source.contributing_sheets.clone(),
        total_source_rows: row_outcomes.len(),
        counts,
        row_outcomes,
        current_required_absent,
        prospective_required_count,
        reconciliation_basis: reconciliation_basis(&entries),
        blocking,
    })
}

struct BuiltRosterRow {
    outcome: CalibrationRosterRowOutcome,
    normalized_asset: Option<String>,
    normalized_serial: Option<String>,
    target_entry_uuid: Option<String>,
}

#[derive(Clone)]
struct SemanticProposal {
    requirement: CalibrationRequirement,
    out_to_calibration: bool,
    last_calibrated_at: Option<String>,
    calibration_due_at: Option<String>,
    calibration_vendor: Option<String>,
    certificate_ref: Option<String>,
    calibration_notes: Option<String>,
    flags: Vec<CalibrationRosterSemanticFlag>,
    issues: Vec<String>,
    requires_review: bool,
    invalid: bool,
}

fn build_row(
    row: &ParsedRosterRow,
    entries: &[InventoryEntry],
    asset_index: &HashMap<String, Vec<usize>>,
    serial_index: &HashMap<String, Vec<usize>>,
) -> BuiltRosterRow {
    let mut issues = Vec::new();
    let mut ignored_identity_placeholders = Vec::new();
    let obvious_label_row = is_obvious_label_row(&row.fields);
    let normalized_asset = (!obvious_label_row)
        .then(|| {
            normalized_source_identity(
                "Asset number",
                row.fields.asset_number.as_deref(),
                &mut ignored_identity_placeholders,
            )
        })
        .flatten();
    let normalized_serial = (!obvious_label_row)
        .then(|| {
            normalized_source_identity(
                "Serial number",
                row.fields.serial_number.as_deref(),
                &mut ignored_identity_placeholders,
            )
        })
        .flatten();
    let has_description_context = [
        row.fields.description.as_deref(),
        row.fields.manufacturer.as_deref(),
        row.fields.model.as_deref(),
    ]
    .into_iter()
    .flatten()
    .any(has_alphanumeric);
    if obvious_label_row
        || (normalized_asset.is_none() && normalized_serial.is_none() && !has_description_context)
    {
        issues.push(if obvious_label_row {
            "Row is an obvious workbook label or separator, not equipment.".to_string()
        } else {
            "Row has no usable equipment identity or description context.".to_string()
        });
        return BuiltRosterRow {
            outcome: CalibrationRosterRowOutcome {
                source_sheet: row.source_sheet.clone(),
                source_row: row.source_row,
                classification: CalibrationRosterClassification::IgnoredJunk,
                issues,
                ignored_identity_placeholders,
                asset_number: row.fields.asset_number.clone(),
                serial_number: row.fields.serial_number.clone(),
                manufacturer: row.fields.manufacturer.clone(),
                model: row.fields.model.clone(),
                description: row.fields.description.clone(),
                candidate_entry_uuid: None,
                candidate_entries: Vec::new(),
                proposed_requirement: CalibrationRequirement::Unknown,
                proposed_out_to_calibration: false,
                proposed_input: None,
                changes: Vec::new(),
                semantic_flags: Vec::new(),
                requires_review: true,
            },
            normalized_asset,
            normalized_serial,
            target_entry_uuid: None,
        };
    }

    let proposal = semantic_proposal(&row.fields);
    issues.extend(proposal.issues.clone());
    let asset_candidates = normalized_asset
        .as_ref()
        .and_then(|identity| asset_index.get(identity))
        .cloned()
        .unwrap_or_default();
    let serial_candidates = normalized_serial
        .as_ref()
        .and_then(|identity| serial_index.get(identity))
        .cloned()
        .unwrap_or_default();
    let asset_as_serial_candidates = normalized_asset
        .as_ref()
        .and_then(|identity| serial_index.get(identity))
        .cloned()
        .unwrap_or_default();
    let serial_as_asset_candidates = normalized_serial
        .as_ref()
        .and_then(|identity| asset_index.get(identity))
        .cloned()
        .unwrap_or_default();
    let has_identity_candidates = !asset_candidates.is_empty()
        || !serial_candidates.is_empty()
        || !asset_as_serial_candidates.is_empty()
        || !serial_as_asset_candidates.is_empty();
    let review_hint_candidates = if has_identity_candidates {
        Vec::new()
    } else {
        review_hint_candidate_indexes(&row.fields, entries)
    };
    let candidate_indexes = asset_candidates
        .iter()
        .chain(serial_candidates.iter())
        .chain(asset_as_serial_candidates.iter())
        .chain(serial_as_asset_candidates.iter())
        .chain(review_hint_candidates.iter())
        .copied()
        .collect::<BTreeSet<_>>();
    let candidate_entries = candidate_indexes
        .iter()
        .map(|index| candidate_context(&entries[*index]))
        .collect::<Vec<_>>();

    let database_duplicate = asset_candidates.len() > 1 || serial_candidates.len() > 1;
    if asset_candidates.len() > 1 {
        issues.push("Asset number matches more than one current equipment entry.".to_string());
    }
    if serial_candidates.len() > 1 {
        issues.push("Serial number matches more than one current equipment entry.".to_string());
    }
    let cross_field_identity_match =
        !asset_as_serial_candidates.is_empty() || !serial_as_asset_candidates.is_empty();
    if !asset_as_serial_candidates.is_empty() {
        issues.push(
            "Asset-number value matches a current serial number; review for swapped identity columns."
                .to_string(),
        );
    }
    if !serial_as_asset_candidates.is_empty() {
        issues.push(
            "Serial-number value matches a current asset number; review for swapped identity columns."
                .to_string(),
        );
    }
    if !review_hint_candidates.is_empty() {
        issues.push(
            "Manufacturer, model, or description produced review hints only; identifiers did not auto-match."
                .to_string(),
        );
    }
    let asset_match = (asset_candidates.len() == 1)
        .then(|| asset_candidates.first().copied())
        .flatten();
    let serial_match = (serial_candidates.len() == 1)
        .then(|| serial_candidates.first().copied())
        .flatten();
    let identifiers_disagree =
        matches!((asset_match, serial_match), (Some(asset), Some(serial)) if asset != serial);
    if identifiers_disagree {
        issues.push(
            "Asset number and serial number resolve to different equipment entries.".to_string(),
        );
    }
    let target_index = if database_duplicate || identifiers_disagree || cross_field_identity_match {
        None
    } else {
        asset_match.or(serial_match)
    };
    let classification = if database_duplicate
        || identifiers_disagree
        || cross_field_identity_match
        || proposal.invalid
    {
        CalibrationRosterClassification::ConflictReviewRequired
    } else if target_index.is_some() {
        CalibrationRosterClassification::MatchedUpdate
    } else {
        CalibrationRosterClassification::CreateCandidate
    };
    if target_index.is_none() && classification == CalibrationRosterClassification::CreateCandidate
    {
        issues.push(
            "No unique asset-number or serial-number match was found; manufacturer, model, and description were not used for automatic matching."
                .to_string(),
        );
    }

    let proposed_input = proposed_input(
        row,
        &proposal,
        normalized_asset.is_some(),
        normalized_serial.is_some(),
    );
    let changes = target_index
        .map(|index| proposed_changes(&entries[index], &proposal))
        .unwrap_or_default();
    let destructive_change = changes.iter().any(|change| change.destructive);
    let requires_review = classification != CalibrationRosterClassification::MatchedUpdate
        || proposal.requires_review
        || destructive_change;
    let target_entry_uuid = target_index.map(|index| entries[index].entry_uuid.clone());

    BuiltRosterRow {
        outcome: CalibrationRosterRowOutcome {
            source_sheet: row.source_sheet.clone(),
            source_row: row.source_row,
            classification,
            issues,
            ignored_identity_placeholders,
            asset_number: row.fields.asset_number.clone(),
            serial_number: row.fields.serial_number.clone(),
            manufacturer: row.fields.manufacturer.clone(),
            model: row.fields.model.clone(),
            description: row.fields.description.clone(),
            candidate_entry_uuid: target_entry_uuid.clone(),
            candidate_entries,
            proposed_requirement: proposal.requirement,
            proposed_out_to_calibration: proposal.out_to_calibration,
            proposed_input: Some(proposed_input),
            changes,
            semantic_flags: proposal.flags,
            requires_review,
        },
        normalized_asset,
        normalized_serial,
        target_entry_uuid,
    }
}

fn semantic_proposal(fields: &ParsedRosterFields) -> SemanticProposal {
    let last_raw = fields.last_calibrated_at.as_deref().unwrap_or_default();
    let due_raw = fields.calibration_due_at.as_deref().unwrap_or_default();
    let status = fields.status.as_deref().unwrap_or_default();
    let condition = fields.condition.as_deref().unwrap_or_default();
    let comments = fields.comments.as_deref().unwrap_or_default();
    let combined = [last_raw, due_raw, status, condition, comments]
        .join(" ")
        .to_ascii_lowercase();
    let reference_only = contains_any(&combined, &["ref only", "reference only", "no cal needed"]);
    let inactive_or_scrapped = contains_any(
        &combined,
        &["inactive", "scrap", "scrapped", "scraped", "will not recal"],
    );
    let failed_calibration = combined.contains("failed calibration");
    let needs_calibration = contains_any(
        &combined,
        &["needs cal", "needs calibrated", "need calibration"],
    ) || due_raw.trim() == "??";
    let out_to_calibration = last_raw.trim().eq_ignore_ascii_case("accura")
        || due_raw.to_ascii_lowercase().contains("accura has")
        || comments.to_ascii_lowercase().contains("accura has");
    let requirement = if inactive_or_scrapped {
        CalibrationRequirement::NotRequired
    } else if reference_only {
        CalibrationRequirement::ReferenceOnly
    } else {
        CalibrationRequirement::Required
    };

    let mut issues = Vec::new();
    let (last_calibrated_at, invalid_last) = parse_semantic_date(
        "Last calibration date",
        fields.last_calibrated_at.as_deref(),
        &mut issues,
    );
    let (calibration_due_at, invalid_due) = parse_semantic_date(
        "Calibration due date",
        fields.calibration_due_at.as_deref(),
        &mut issues,
    );
    let invalid_order = matches!(
        (
            last_calibrated_at.as_deref().and_then(parse_date),
            calibration_due_at.as_deref().and_then(parse_date),
        ),
        (Some(last), Some(due)) if due < last
    );
    if invalid_order {
        issues.push("Calibration due date is before the last calibration date.".to_string());
    }

    let mut flags = Vec::new();
    if reference_only {
        flags.push(CalibrationRosterSemanticFlag::ReferenceOnly);
    }
    if needs_calibration
        || (requirement == CalibrationRequirement::Required && calibration_due_at.is_none())
    {
        flags.push(CalibrationRosterSemanticFlag::NeedsCalibration);
    }
    if out_to_calibration {
        flags.push(CalibrationRosterSemanticFlag::OutToCalibration);
    }
    if failed_calibration {
        flags.push(CalibrationRosterSemanticFlag::FailedCalibration);
    }
    if inactive_or_scrapped {
        flags.push(CalibrationRosterSemanticFlag::InactiveOrScrapped);
    }
    if flags.is_empty() && (last_calibrated_at.is_some() || calibration_due_at.is_some()) {
        flags.push(CalibrationRosterSemanticFlag::NormalDated);
    }
    let invalid = invalid_last || invalid_due || invalid_order;
    if invalid {
        flags.push(CalibrationRosterSemanticFlag::Unclear);
    }
    let requires_review = inactive_or_scrapped || invalid;
    let calibration_vendor = fields
        .calibration_vendor
        .clone()
        .or_else(|| out_to_calibration.then(|| "ACCURA".to_string()));

    SemanticProposal {
        requirement,
        out_to_calibration,
        last_calibrated_at,
        calibration_due_at,
        calibration_vendor,
        certificate_ref: fields.certificate_ref.clone(),
        calibration_notes: fields.comments.clone(),
        flags,
        issues,
        requires_review,
        invalid,
    }
}

fn parse_semantic_date(
    label: &str,
    value: Option<&str>,
    issues: &mut Vec<String>,
) -> (Option<String>, bool) {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return (None, false);
    };
    if parse_date(value).is_some() {
        return (Some(value.to_string()), false);
    }
    let semantic = value.to_ascii_lowercase();
    if contains_any(
        &semantic,
        &[
            "ref only",
            "no cal needed",
            "needs cal",
            "accura",
            "inactive",
            "unknown",
            "n/a",
            "??",
        ],
    ) {
        return (None, false);
    }
    issues.push(format!(
        "{label} value '{value}' is not a recognized date or semantic token."
    ));
    (None, true)
}

fn proposed_input(
    row: &ParsedRosterRow,
    proposal: &SemanticProposal,
    keep_asset: bool,
    keep_serial: bool,
) -> InventoryEntryInput {
    InventoryEntryInput {
        asset_number: keep_asset
            .then(|| row.fields.asset_number.clone())
            .flatten()
            .unwrap_or_default(),
        serial_number: keep_serial
            .then(|| row.fields.serial_number.clone())
            .flatten()
            .unwrap_or_default(),
        manufacturer: row.fields.manufacturer.clone().unwrap_or_default(),
        model: row.fields.model.clone().unwrap_or_default(),
        description: row.fields.description.clone().unwrap_or_default(),
        location: row.fields.location.clone().unwrap_or_default(),
        assigned_to: row.fields.assigned_to.clone().unwrap_or_default(),
        lifecycle_status: "active".to_string(),
        working_status: "unknown".to_string(),
        condition: row.fields.condition.clone().unwrap_or_default(),
        calibration_requirement: proposal.requirement,
        out_to_calibration: proposal.out_to_calibration,
        last_calibrated_at: proposal.last_calibrated_at.clone(),
        calibration_due_at: proposal.calibration_due_at.clone(),
        certificate_ref: proposal.certificate_ref.clone(),
        calibration_vendor: proposal.calibration_vendor.clone(),
        calibration_notes: proposal.calibration_notes.clone(),
        archived: false,
        ..InventoryEntryInput::default()
    }
}

fn proposed_changes(
    entry: &InventoryEntry,
    proposal: &SemanticProposal,
) -> Vec<CalibrationRosterFieldChange> {
    let mut changes = Vec::new();
    push_change(
        &mut changes,
        "calibrationRequirement",
        Some(requirement_value(entry.calibration_requirement)),
        Some(requirement_value(proposal.requirement)),
        entry.calibration_requirement == CalibrationRequirement::Required
            && proposal.requirement != CalibrationRequirement::Required,
    );
    push_change(
        &mut changes,
        "outToCalibration",
        Some(entry.out_to_calibration.to_string()),
        Some(proposal.out_to_calibration.to_string()),
        entry.out_to_calibration && !proposal.out_to_calibration,
    );
    push_optional_change(
        &mut changes,
        "lastCalibratedAt",
        entry.last_calibrated_at.clone(),
        proposal.last_calibrated_at.clone(),
    );
    push_optional_change(
        &mut changes,
        "calibrationDueAt",
        entry.calibration_due_at.clone(),
        proposal.calibration_due_at.clone(),
    );
    if let Some(vendor) = &proposal.calibration_vendor {
        push_change(
            &mut changes,
            "calibrationVendor",
            entry.calibration_vendor.clone(),
            Some(vendor.clone()),
            false,
        );
    }
    if let Some(certificate_ref) = &proposal.certificate_ref {
        push_change(
            &mut changes,
            "certificateRef",
            entry.certificate_ref.clone(),
            Some(certificate_ref.clone()),
            false,
        );
    }
    if let Some(source_notes) = proposal
        .calibration_notes
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let after = append_note(entry.calibration_notes.as_deref(), source_notes);
        push_change(
            &mut changes,
            "calibrationNotes",
            entry.calibration_notes.clone(),
            Some(after),
            false,
        );
    }
    push_change(
        &mut changes,
        "verifiedAt",
        entry.verified_at.clone(),
        Some("Set at commit time".to_string()),
        false,
    );
    push_change(
        &mut changes,
        "verifiedBy",
        entry.verified_by.clone(),
        Some("Confirmed roster attribution".to_string()),
        false,
    );
    changes
}

fn push_optional_change(
    changes: &mut Vec<CalibrationRosterFieldChange>,
    field: &str,
    before: Option<String>,
    after: Option<String>,
) {
    let destructive = before.is_some() && after.is_none();
    push_change(changes, field, before, after, destructive);
}

fn push_change(
    changes: &mut Vec<CalibrationRosterFieldChange>,
    field: &str,
    before: Option<String>,
    after: Option<String>,
    destructive: bool,
) {
    if before != after {
        changes.push(CalibrationRosterFieldChange {
            field: field.to_string(),
            before,
            after,
            destructive,
        });
    }
}

fn mark_source_duplicates(rows: &mut [BuiltRosterRow]) {
    let mut duplicate_indexes = HashSet::new();
    let mut target_groups: HashMap<String, Vec<usize>> = HashMap::new();
    let mut asset_groups: HashMap<String, Vec<usize>> = HashMap::new();
    let mut serial_groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (index, row) in rows.iter().enumerate() {
        if row.outcome.classification == CalibrationRosterClassification::IgnoredJunk {
            continue;
        }
        if let Some(target) = &row.target_entry_uuid {
            target_groups.entry(target.clone()).or_default().push(index);
        }
        if let Some(asset) = &row.normalized_asset {
            asset_groups.entry(asset.clone()).or_default().push(index);
        }
        if let Some(serial) = &row.normalized_serial {
            serial_groups.entry(serial.clone()).or_default().push(index);
        }
    }
    for indexes in target_groups
        .values()
        .chain(asset_groups.values())
        .chain(serial_groups.values())
        .filter(|indexes| indexes.len() > 1)
    {
        duplicate_indexes.extend(indexes.iter().copied());
    }
    for index in duplicate_indexes {
        let row = &mut rows[index];
        row.outcome.classification = CalibrationRosterClassification::DuplicateSourceRow;
        row.outcome.requires_review = true;
        row.outcome.issues.push(
            "Another source row resolves to the same equipment identity or target; choose one approved row and ignore the competing row."
                .to_string(),
        );
    }
}

fn identity_index(
    entries: &[InventoryEntry],
    field: impl Fn(&InventoryEntry) -> &String,
) -> HashMap<String, Vec<usize>> {
    let mut index = HashMap::new();
    for (entry_index, entry) in entries.iter().enumerate() {
        if let Some(identity) = normalize_identity(field(entry)) {
            index
                .entry(identity)
                .or_insert_with(Vec::new)
                .push(entry_index);
        }
    }
    index
}

fn review_hint_candidate_indexes(
    fields: &ParsedRosterFields,
    entries: &[InventoryEntry],
) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| review_hint_matches(fields, entry).then_some(index))
        .collect()
}

fn review_hint_matches(fields: &ParsedRosterFields, entry: &InventoryEntry) -> bool {
    let explicit_manufacturer_model_match = match (
        normalized_hint(fields.manufacturer.as_deref()),
        normalized_hint(fields.model.as_deref()),
    ) {
        (Some(manufacturer), Some(model)) => {
            normalized_hint(Some(&entry.manufacturer)).as_deref() == Some(manufacturer.as_str())
                && normalized_hint(Some(&entry.model)).as_deref() == Some(model.as_str())
        }
        _ => false,
    };
    let exact_description_match =
        normalized_hint(fields.description.as_deref()).is_some_and(|description| {
            normalized_hint(Some(&entry.description)).as_deref() == Some(description.as_str())
        });
    let description_manufacturer_model_match = match (
        normalized_hint(fields.description.as_deref()),
        normalized_hint(Some(&entry.manufacturer)),
        normalized_hint(Some(&entry.model)),
    ) {
        (Some(description), Some(manufacturer), Some(model))
            if manufacturer.len() >= 3 && model.len() >= 3 =>
        {
            description.contains(&manufacturer) && description.contains(&model)
        }
        _ => false,
    };
    explicit_manufacturer_model_match
        || exact_description_match
        || description_manufacturer_model_match
}

fn normalized_hint(value: Option<&str>) -> Option<String> {
    let normalized = value?
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    (!normalized.is_empty()).then_some(normalized)
}

fn is_obvious_label_row(fields: &ParsedRosterFields) -> bool {
    let combined = [
        fields.asset_number.as_deref(),
        fields.serial_number.as_deref(),
        fields.manufacturer.as_deref(),
        fields.model.as_deref(),
        fields.description.as_deref(),
        fields.status.as_deref(),
        fields.condition.as_deref(),
        fields.comments.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(" ")
    .to_ascii_lowercase();
    contains_any(
        &combined,
        &[
            "newly added to list below",
            "newly added below",
            "ck list found",
            "check list found",
        ],
    )
}

fn has_alphanumeric(value: &str) -> bool {
    value.chars().any(|character| character.is_alphanumeric())
}

fn normalized_source_identity(
    label: &str,
    value: Option<&str>,
    ignored_placeholders: &mut Vec<String>,
) -> Option<String> {
    let value = value.map(str::trim).filter(|value| !value.is_empty())?;
    if is_placeholder_identity(value) {
        ignored_placeholders.push(format!(
            "{label} placeholder '{value}' was ignored for matching."
        ));
        None
    } else {
        normalize_identity(value)
    }
}

pub(super) fn normalize_identity(value: &str) -> Option<String> {
    let normalized = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect::<String>();
    (!normalized.is_empty() && !is_placeholder_identity(&normalized)).then_some(normalized)
}

fn is_placeholder_identity(value: &str) -> bool {
    let normalized = value
        .chars()
        .filter(|character| character.is_ascii_alphanumeric())
        .flat_map(char::to_uppercase)
        .collect::<String>();
    matches!(
        normalized.as_str(),
        "NSN" | "NA" | "NONE" | "UNKNOWN" | "TBD"
    ) || (normalized.len() >= 3 && normalized.chars().all(|character| character == 'X'))
}

fn candidate_context(entry: &InventoryEntry) -> CalibrationRosterCandidateContext {
    CalibrationRosterCandidateContext {
        entry_uuid: entry.entry_uuid.clone(),
        id: entry.id.clone(),
        asset_number: entry.asset_number.clone(),
        serial_number: entry.serial_number.clone(),
        manufacturer: entry.manufacturer.clone(),
        model: entry.model.clone(),
        description: entry.description.clone(),
    }
}

fn absent_entry(entry: &InventoryEntry) -> CalibrationRosterAbsentEntry {
    CalibrationRosterAbsentEntry {
        entry: candidate_context(entry),
        calibration_due_at: entry.calibration_due_at.clone(),
        calibration_vendor: entry.calibration_vendor.clone(),
        calibration_notes: entry.calibration_notes.clone(),
    }
}

fn is_active_required(entry: &InventoryEntry) -> bool {
    !entry.archived
        && entry.lifecycle_status == "active"
        && entry.calibration_requirement == CalibrationRequirement::Required
}

fn count_outcomes(
    rows: &[CalibrationRosterRowOutcome],
    absent_count: usize,
) -> CalibrationRosterCounts {
    let mut counts = CalibrationRosterCounts {
        current_required_absent: absent_count,
        ..CalibrationRosterCounts::default()
    };
    for row in rows {
        match row.classification {
            CalibrationRosterClassification::MatchedUpdate => counts.matched_updates += 1,
            CalibrationRosterClassification::CreateCandidate => counts.create_candidates += 1,
            CalibrationRosterClassification::ConflictReviewRequired => counts.conflicts += 1,
            CalibrationRosterClassification::DuplicateSourceRow => {
                counts.duplicate_source_rows += 1
            }
            CalibrationRosterClassification::IgnoredJunk => counts.ignored_junk += 1,
        }
    }
    counts
}

fn prospective_required_count(rows: &[BuiltRosterRow], entries: &[InventoryEntry]) -> usize {
    let archived_targets = entries
        .iter()
        .filter(|entry| entry.archived || entry.lifecycle_status != "active")
        .map(|entry| entry.entry_uuid.as_str())
        .collect::<HashSet<_>>();
    let mut required = BTreeSet::new();
    for row in rows {
        if row.outcome.classification == CalibrationRosterClassification::IgnoredJunk
            || row.outcome.classification == CalibrationRosterClassification::DuplicateSourceRow
            || row.outcome.proposed_requirement != CalibrationRequirement::Required
        {
            continue;
        }
        if let Some(target) = row.target_entry_uuid.as_deref() {
            if !archived_targets.contains(target) {
                required.insert(format!("entry:{target}"));
            }
        } else {
            required.insert(format!(
                "source:{}:{}",
                row.outcome.source_sheet, row.outcome.source_row
            ));
        }
    }
    required.len()
}

fn reconciliation_basis(entries: &[InventoryEntry]) -> String {
    let mut material = entries
        .iter()
        .map(|entry| {
            format!(
                "{entry_uuid}\0{asset_number}\0{serial_number}\0{requirement:?}\0{out_to_calibration}\0{last_calibrated_at}\0{calibration_due_at}\0{calibration_vendor}\0{certificate_ref}\0{calibration_notes}\0{verified_at}\0{verified_by}\0{lifecycle_status}\0{archived}\0{updated_at}",
                entry_uuid = entry.entry_uuid,
                asset_number = normalize_identity(&entry.asset_number).unwrap_or_default(),
                serial_number = normalize_identity(&entry.serial_number).unwrap_or_default(),
                requirement = entry.calibration_requirement,
                out_to_calibration = entry.out_to_calibration,
                last_calibrated_at = entry.last_calibrated_at.as_deref().unwrap_or_default(),
                calibration_due_at = entry.calibration_due_at.as_deref().unwrap_or_default(),
                calibration_vendor = entry.calibration_vendor.as_deref().unwrap_or_default(),
                certificate_ref = entry.certificate_ref.as_deref().unwrap_or_default(),
                calibration_notes = entry.calibration_notes.as_deref().unwrap_or_default(),
                verified_at = entry.verified_at.as_deref().unwrap_or_default(),
                verified_by = entry.verified_by.as_deref().unwrap_or_default(),
                lifecycle_status = entry.lifecycle_status,
                archived = entry.archived,
                updated_at = entry.updated_at,
            )
        })
        .collect::<Vec<_>>();
    material.sort();
    format!("basis-{}", hex_digest(material.join("\n").as_bytes()))
}

fn batch_id(source: &ParsedRosterSource) -> String {
    let material = format!(
        "{}\0{}",
        source.fingerprint, CALIBRATION_ROSTER_MAPPING_VERSION
    );
    format!("calibration-batch-{}", hex_digest(material.as_bytes()))
}

fn append_note(existing: Option<&str>, source_note: &str) -> String {
    let source_note = source_note.trim();
    let Some(existing) = existing.map(str::trim).filter(|value| !value.is_empty()) else {
        return source_note.to_string();
    };
    if existing.contains(source_note) {
        existing.to_string()
    } else {
        format!("{existing}\n\nCalibration roster: {source_note}")
    }
}

fn requirement_value(requirement: CalibrationRequirement) -> String {
    match requirement {
        CalibrationRequirement::Required => "required",
        CalibrationRequirement::ReferenceOnly => "reference_only",
        CalibrationRequirement::NotRequired => "not_required",
        CalibrationRequirement::Unknown => "unknown",
    }
    .to_string()
}

fn parse_date(value: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}
