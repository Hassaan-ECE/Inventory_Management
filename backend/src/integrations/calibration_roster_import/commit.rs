use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    api::mutations::{create_imported_entry_in_store, update_entry_in_store},
    model::{
        normalize_entry_input, now_timestamp, validate_entry_input, CalibrationRequirement,
        CommandResult, ImportProvenance, InventoryEntry, InventoryEntryEditContext,
        InventoryEntryInput,
    },
    store::InventoryDb,
};

use super::{
    parser::parse_roster_source,
    reconcile::{normalize_identity, reconcile_roster_source},
    types::{
        CalibrationRosterClassification, CalibrationRosterCommitInput,
        CalibrationRosterCommitResult, CalibrationRosterPreviewReport, CalibrationRosterResolution,
        CalibrationRosterResolutionAction, CalibrationRosterRowOutcome,
    },
    StoredCalibrationRosterPreview, CALIBRATION_ROSTER_MAPPING_VERSION,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalibrationRosterCommitState {
    commit_signature: String,
    verified_at: String,
    verification_attribution: String,
    reset_entry_uuids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CalibrationRosterOperationMarker {
    action: String,
    entry_uuid: Option<String>,
}

#[derive(Clone)]
enum PlannedRowAction {
    Existing {
        row: Box<CalibrationRosterRowOutcome>,
        target_entry_uuid: String,
    },
    Create {
        row: Box<CalibrationRosterRowOutcome>,
        entry_uuid: String,
        input: Box<InventoryEntryInput>,
    },
    Ignore {
        row: Box<CalibrationRosterRowOutcome>,
    },
}

struct CommitPlan {
    actions: Vec<PlannedRowAction>,
    approved_entry_uuids: HashSet<String>,
}

pub(crate) fn commit_calibration_roster_from_store(
    input: CalibrationRosterCommitInput,
    db: &InventoryDb,
) -> CommandResult<CalibrationRosterCommitResult> {
    commit_calibration_roster(input, db, None)
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn commit_calibration_roster_with_test_failure_after(
    input: CalibrationRosterCommitInput,
    db: &InventoryDb,
    completed_mutations: usize,
) -> CommandResult<CalibrationRosterCommitResult> {
    commit_calibration_roster(input, db, Some(completed_mutations))
}

fn commit_calibration_roster(
    mut input: CalibrationRosterCommitInput,
    db: &InventoryDb,
    inject_after_mutations_before_marker: Option<usize>,
) -> CommandResult<CalibrationRosterCommitResult> {
    if !input.confirmed {
        return Err("Calibration roster commit requires explicit confirmation.".to_string());
    }
    if !input.replace_active_required_roster {
        return Err(
            "Calibration roster commit requires confirmation that the workbook replaces the active required roster."
                .to_string(),
        );
    }
    input.verification_attribution = input.verification_attribution.trim().to_string();
    if input.verification_attribution.is_empty() {
        return Err("Calibration roster verification attribution is required.".to_string());
    }
    if let Some(completed) =
        db.completed_calibration_roster::<CalibrationRosterCommitResult>(&input.batch_id)?
    {
        return Ok(CalibrationRosterCommitResult {
            batch_id: completed.batch_id,
            updated: 0,
            created: 0,
            reset: 0,
            ignored: 0,
            noop: completed.updated
                + completed.created
                + completed.reset
                + completed.ignored
                + completed.noop,
            final_required: completed.final_required,
            entries_changed: false,
            message: "Calibration roster batch was already completed; no entries or sync operations were added."
                .to_string(),
        });
    }

    let stored = db
        .calibration_roster_preview::<StoredCalibrationRosterPreview>(&input.batch_id)?
        .ok_or_else(|| {
            "Calibration roster preview was not found. Preview the workbook again.".to_string()
        })?;
    if stored.report.batch_id != input.batch_id {
        return Err(
            "Stored calibration roster batch does not match the requested batch.".to_string(),
        );
    }
    if stored.report.mapping_version != CALIBRATION_ROSTER_MAPPING_VERSION {
        return Err(
            "Calibration roster mapping changed after preview; preview it again.".to_string(),
        );
    }
    let parsed = parse_roster_source(std::path::Path::new(&stored.source_path))?;
    if parsed.fingerprint != stored.report.source_fingerprint {
        return Err(
            "Calibration roster workbook changed after preview; preview it again.".to_string(),
        );
    }
    let signature = commit_signature(&input)?;
    let existing_state =
        db.calibration_roster_commit_state::<CalibrationRosterCommitState>(&input.batch_id)?;
    let state = if let Some(state) = existing_state {
        if state.commit_signature != signature
            || state.verification_attribution != input.verification_attribution
        {
            return Err(
                "Calibration roster commit was already started with different resolutions or attribution."
                    .to_string(),
            );
        }
        state
    } else {
        let fresh_report = reconcile_roster_source(&parsed, db)?;
        if fresh_report.batch_id != stored.report.batch_id
            || fresh_report.reconciliation_basis != stored.report.reconciliation_basis
            || fresh_report.row_outcomes != stored.report.row_outcomes
        {
            return Err(
                "Calibration roster reconciliation basis changed after preview; preview it again."
                    .to_string(),
            );
        }
        let entries = db.load_entries()?;
        let plan = build_commit_plan(&stored.report, &input, &entries)?;
        let mut reset_entry_uuids = entries
            .iter()
            .filter(|entry| is_active_required(entry))
            .filter(|entry| !plan.approved_entry_uuids.contains(&entry.entry_uuid))
            .map(|entry| entry.entry_uuid.clone())
            .collect::<Vec<_>>();
        reset_entry_uuids.sort();
        let state = CalibrationRosterCommitState {
            commit_signature: signature,
            verified_at: now_timestamp(),
            verification_attribution: input.verification_attribution.clone(),
            reset_entry_uuids,
        };
        db.put_calibration_roster_commit_state(&input.batch_id, &state)?;
        state
    };

    let current_entries = db.load_entries()?;
    let plan = build_commit_plan(&stored.report, &input, &current_entries)?;
    let mut updated = 0usize;
    let mut created = 0usize;
    let mut reset = 0usize;
    let mut ignored = 0usize;
    let mut noop = 0usize;
    let mut mutation_count = 0usize;

    for action in &plan.actions {
        let row = action.row();
        let operation_id = row_operation_id(&row.source_sheet, row.source_row);
        if db.has_calibration_roster_operation_marker(&input.batch_id, &operation_id)? {
            noop += 1;
            continue;
        }
        match action {
            PlannedRowAction::Existing {
                row,
                target_entry_uuid,
            } => {
                let existing = db.find_entry(target_entry_uuid)?.ok_or_else(|| {
                    format!(
                        "Selected calibration roster target no longer exists for {} row {}.",
                        row.source_sheet, row.source_row
                    )
                })?;
                let (entry_input, changed_fields, has_changes) = matched_update_input(
                    &existing,
                    row,
                    &state.verified_at,
                    &state.verification_attribution,
                )?;
                if has_changes {
                    update_entry_in_store(
                        target_entry_uuid,
                        entry_input,
                        Some(InventoryEntryEditContext {
                            base_version: Some(existing.updated_at.clone()),
                            changed_fields,
                        }),
                        db,
                    )?;
                    updated += 1;
                    mutation_count += 1;
                    maybe_inject_failure(
                        inject_after_mutations_before_marker,
                        mutation_count,
                        row,
                    )?;
                } else {
                    noop += 1;
                }
                db.mark_calibration_roster_operation_complete(
                    &input.batch_id,
                    &operation_id,
                    &CalibrationRosterOperationMarker {
                        action: "use_existing".to_string(),
                        entry_uuid: Some(target_entry_uuid.clone()),
                    },
                )?;
            }
            PlannedRowAction::Create {
                row,
                entry_uuid,
                input: create_input,
            } => {
                let provenance = row_provenance(&input.batch_id, &stored.report, row);
                if let Some(existing) = db.find_entry(entry_uuid)? {
                    if existing.import_provenance.as_ref() != Some(&provenance) {
                        return Err(format!(
                            "Deterministic calibration roster UUID collision at {} row {}.",
                            row.source_sheet, row.source_row
                        ));
                    }
                    noop += 1;
                } else {
                    let mut create_input = create_input.as_ref().clone();
                    create_input.verified_at = Some(state.verified_at.clone());
                    create_input.verified_by = Some(state.verification_attribution.clone());
                    create_input.archived = false;
                    create_imported_entry_in_store(
                        create_input,
                        entry_uuid.clone(),
                        provenance,
                        db,
                    )?;
                    created += 1;
                    mutation_count += 1;
                    maybe_inject_failure(
                        inject_after_mutations_before_marker,
                        mutation_count,
                        row,
                    )?;
                }
                db.mark_calibration_roster_operation_complete(
                    &input.batch_id,
                    &operation_id,
                    &CalibrationRosterOperationMarker {
                        action: "create".to_string(),
                        entry_uuid: Some(entry_uuid.clone()),
                    },
                )?;
            }
            PlannedRowAction::Ignore { row: _ } => {
                ignored += 1;
                db.mark_calibration_roster_operation_complete(
                    &input.batch_id,
                    &operation_id,
                    &CalibrationRosterOperationMarker {
                        action: "ignore".to_string(),
                        entry_uuid: None,
                    },
                )?;
            }
        }
    }

    for entry_uuid in &state.reset_entry_uuids {
        let operation_id = reset_operation_id(entry_uuid);
        if db.has_calibration_roster_operation_marker(&input.batch_id, &operation_id)? {
            noop += 1;
            continue;
        }
        let existing = db.find_entry(entry_uuid)?.ok_or_else(|| {
            format!("Calibration roster reset target '{entry_uuid}' no longer exists.")
        })?;
        let mut reset_input = entry_to_input(&existing);
        reset_input.calibration_requirement = CalibrationRequirement::Unknown;
        reset_input.out_to_calibration = false;
        let has_changes = existing.calibration_requirement != CalibrationRequirement::Unknown
            || existing.out_to_calibration;
        if has_changes {
            update_entry_in_store(
                entry_uuid,
                reset_input,
                Some(InventoryEntryEditContext {
                    base_version: Some(existing.updated_at.clone()),
                    changed_fields: vec![
                        "calibrationRequirement".to_string(),
                        "outToCalibration".to_string(),
                    ],
                }),
                db,
            )?;
            reset += 1;
            mutation_count += 1;
            if inject_after_mutations_before_marker == Some(mutation_count) {
                return Err(format!(
                    "injected calibration roster failure after reset mutation but before marker for entry {entry_uuid}"
                ));
            }
        } else {
            noop += 1;
        }
        db.mark_calibration_roster_operation_complete(
            &input.batch_id,
            &operation_id,
            &CalibrationRosterOperationMarker {
                action: "reset_absent_required".to_string(),
                entry_uuid: Some(entry_uuid.clone()),
            },
        )?;
    }

    let final_required = db
        .load_entries()?
        .iter()
        .filter(|entry| is_active_required(entry))
        .count();
    let entries_changed = updated + created + reset > 0;
    let result = CalibrationRosterCommitResult {
        batch_id: input.batch_id.clone(),
        updated,
        created,
        reset,
        ignored,
        noop,
        final_required,
        entries_changed,
        message: format!(
            "Calibration roster applied: {updated} updated, {created} created, {reset} removed from the active required roster, {ignored} ignored."
        ),
    };
    db.mark_calibration_roster_completed(&input.batch_id, &result)?;
    Ok(result)
}

impl PlannedRowAction {
    fn row(&self) -> &CalibrationRosterRowOutcome {
        match self {
            Self::Existing { row, .. } | Self::Create { row, .. } | Self::Ignore { row } => {
                row.as_ref()
            }
        }
    }
}

fn build_commit_plan(
    report: &CalibrationRosterPreviewReport,
    input: &CalibrationRosterCommitInput,
    entries: &[InventoryEntry],
) -> CommandResult<CommitPlan> {
    let entries_by_uuid = entries
        .iter()
        .map(|entry| (entry.entry_uuid.as_str(), entry))
        .collect::<HashMap<_, _>>();
    let mut resolutions = HashMap::new();
    for resolution in &input.resolutions {
        let key = row_key(&resolution.source_sheet, resolution.source_row);
        if resolutions.insert(key.clone(), resolution).is_some() {
            return Err(format!(
                "Calibration roster has more than one resolution for {} row {}.",
                resolution.source_sheet, resolution.source_row
            ));
        }
    }

    let known_rows = report
        .row_outcomes
        .iter()
        .map(|row| row_key(&row.source_sheet, row.source_row))
        .collect::<HashSet<_>>();
    if let Some(unknown) = resolutions.keys().find(|key| !known_rows.contains(*key)) {
        return Err(format!(
            "Calibration roster resolution references unknown source row '{unknown}'."
        ));
    }

    let mut actions = Vec::with_capacity(report.row_outcomes.len());
    let mut approved_entry_uuids = HashSet::new();
    for row in &report.row_outcomes {
        let key = row_key(&row.source_sheet, row.source_row);
        let resolution = resolutions.get(&key).copied();
        let resolution_required = row.requires_review
            || row.classification != CalibrationRosterClassification::MatchedUpdate;
        if resolution_required && resolution.is_none() {
            return Err(format!(
                "Calibration roster row {} in '{}' requires an explicit confirmed resolution.",
                row.source_row, row.source_sheet
            ));
        }
        if let Some(resolution) = resolution {
            if !resolution.confirmed {
                return Err(format!(
                    "Calibration roster row {} in '{}' has not been explicitly confirmed.",
                    row.source_row, row.source_sheet
                ));
            }
            let action = resolved_action(row, resolution, &entries_by_uuid, report)?;
            if let Some(entry_uuid) = action.approved_entry_uuid() {
                if !approved_entry_uuids.insert(entry_uuid.to_string()) {
                    return Err(format!(
                        "More than one calibration roster row targets equipment entry '{entry_uuid}'."
                    ));
                }
            }
            actions.push(action);
            continue;
        }

        let target_entry_uuid = row.candidate_entry_uuid.clone().ok_or_else(|| {
            format!(
                "Matched calibration roster row {} in '{}' has no target entry.",
                row.source_row, row.source_sheet
            )
        })?;
        if !entries_by_uuid.contains_key(target_entry_uuid.as_str()) {
            return Err(format!(
                "Matched calibration roster target '{target_entry_uuid}' no longer exists."
            ));
        }
        approved_entry_uuids.insert(target_entry_uuid.clone());
        actions.push(PlannedRowAction::Existing {
            row: Box::new(row.clone()),
            target_entry_uuid,
        });
    }

    validate_create_identity_collisions(&actions, entries)?;
    Ok(CommitPlan {
        actions,
        approved_entry_uuids,
    })
}

fn resolved_action(
    row: &CalibrationRosterRowOutcome,
    resolution: &CalibrationRosterResolution,
    entries_by_uuid: &HashMap<&str, &InventoryEntry>,
    report: &CalibrationRosterPreviewReport,
) -> CommandResult<PlannedRowAction> {
    match resolution.action {
        CalibrationRosterResolutionAction::Ignore => Ok(PlannedRowAction::Ignore {
            row: Box::new(row.clone()),
        }),
        CalibrationRosterResolutionAction::UseExisting => {
            let target_entry_uuid = resolution
                .target_entry_uuid
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    "Use-existing calibration roster resolution requires a target entry."
                        .to_string()
                })?;
            if !row
                .candidate_entries
                .iter()
                .any(|candidate| candidate.entry_uuid == target_entry_uuid)
            {
                return Err(format!(
                    "Selected target '{target_entry_uuid}' is not a preview candidate for {} row {}.",
                    row.source_sheet, row.source_row
                ));
            }
            if !entries_by_uuid.contains_key(target_entry_uuid) {
                return Err(format!(
                    "Selected calibration roster target '{target_entry_uuid}' no longer exists."
                ));
            }
            Ok(PlannedRowAction::Existing {
                row: Box::new(row.clone()),
                target_entry_uuid: target_entry_uuid.to_string(),
            })
        }
        CalibrationRosterResolutionAction::Create => {
            if row.classification == CalibrationRosterClassification::IgnoredJunk {
                return Err("Ignored junk rows cannot create equipment entries.".to_string());
            }
            let mut create_input = resolution.create_input.clone().ok_or_else(|| {
                format!(
                    "Create resolution for {} row {} requires reviewed equipment input.",
                    row.source_sheet, row.source_row
                )
            })?;
            create_input.archived = false;
            create_input.verified_at = None;
            create_input.verified_by = None;
            create_input = normalize_entry_input(create_input);
            validate_entry_input(&create_input)?;
            Ok(PlannedRowAction::Create {
                row: Box::new(row.clone()),
                entry_uuid: deterministic_entry_uuid(
                    &report.batch_id,
                    &row.source_sheet,
                    row.source_row,
                ),
                input: Box::new(create_input),
            })
        }
    }
}

impl PlannedRowAction {
    fn approved_entry_uuid(&self) -> Option<&str> {
        match self {
            Self::Existing {
                target_entry_uuid, ..
            } => Some(target_entry_uuid),
            Self::Create { entry_uuid, .. } => Some(entry_uuid),
            Self::Ignore { .. } => None,
        }
    }
}

fn validate_create_identity_collisions(
    actions: &[PlannedRowAction],
    entries: &[InventoryEntry],
) -> CommandResult<()> {
    for action in actions {
        let PlannedRowAction::Create {
            row,
            entry_uuid,
            input,
        } = action
        else {
            continue;
        };
        for entry in entries {
            if entry.entry_uuid == *entry_uuid {
                continue;
            }
            let asset_collision = normalize_identity(&input.asset_number).is_some_and(|identity| {
                normalize_identity(&entry.asset_number).as_deref() == Some(identity.as_str())
            });
            let serial_collision =
                normalize_identity(&input.serial_number).is_some_and(|identity| {
                    normalize_identity(&entry.serial_number).as_deref() == Some(identity.as_str())
                });
            if asset_collision || serial_collision {
                return Err(format!(
                    "Reviewed create for {} row {} now matches existing equipment '{}'; choose the existing entry instead.",
                    row.source_sheet, row.source_row, entry.entry_uuid
                ));
            }
        }
    }
    Ok(())
}

fn matched_update_input(
    existing: &InventoryEntry,
    row: &CalibrationRosterRowOutcome,
    verified_at: &str,
    verification_attribution: &str,
) -> CommandResult<(InventoryEntryInput, Vec<String>, bool)> {
    let proposal = row.proposed_input.as_ref().ok_or_else(|| {
        format!(
            "Calibration roster row {} in '{}' has no proposed calibration input.",
            row.source_row, row.source_sheet
        )
    })?;
    let mut input = entry_to_input(existing);
    input.calibration_requirement = proposal.calibration_requirement;
    input.out_to_calibration = proposal.out_to_calibration;
    input
        .last_calibrated_at
        .clone_from(&proposal.last_calibrated_at);
    input
        .calibration_due_at
        .clone_from(&proposal.calibration_due_at);
    let mut changed_fields = vec![
        "calibrationRequirement".to_string(),
        "outToCalibration".to_string(),
        "lastCalibratedAt".to_string(),
        "calibrationDueAt".to_string(),
        "verifiedAt".to_string(),
        "verifiedBy".to_string(),
    ];
    if proposal.calibration_vendor.is_some() {
        input
            .calibration_vendor
            .clone_from(&proposal.calibration_vendor);
        changed_fields.push("calibrationVendor".to_string());
    }
    if proposal.certificate_ref.is_some() {
        input.certificate_ref.clone_from(&proposal.certificate_ref);
        changed_fields.push("certificateRef".to_string());
    }
    if let Some(source_note) = proposal
        .calibration_notes
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        input.calibration_notes = Some(append_note(
            existing.calibration_notes.as_deref(),
            source_note,
        ));
        changed_fields.push("calibrationNotes".to_string());
    }
    input.verified_at = Some(verified_at.to_string());
    input.verified_by = Some(verification_attribution.to_string());
    input = normalize_entry_input(input);
    validate_entry_input(&input)?;
    let has_changes = existing.calibration_requirement != input.calibration_requirement
        || existing.out_to_calibration != input.out_to_calibration
        || existing.last_calibrated_at != input.last_calibrated_at
        || existing.calibration_due_at != input.calibration_due_at
        || (proposal.calibration_vendor.is_some()
            && existing.calibration_vendor != input.calibration_vendor)
        || (proposal.certificate_ref.is_some()
            && existing.certificate_ref != input.certificate_ref)
        || (proposal.calibration_notes.is_some()
            && existing.calibration_notes != input.calibration_notes)
        || existing.verified_at != input.verified_at
        || existing.verified_by != input.verified_by;
    Ok((input, changed_fields, has_changes))
}

fn entry_to_input(entry: &InventoryEntry) -> InventoryEntryInput {
    InventoryEntryInput {
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
    }
}

fn row_provenance(
    batch_id: &str,
    report: &CalibrationRosterPreviewReport,
    row: &CalibrationRosterRowOutcome,
) -> ImportProvenance {
    ImportProvenance {
        batch_id: batch_id.to_string(),
        source_filename: report.source_filename.clone(),
        source_sheet: Some(row.source_sheet.clone()),
        source_row: row.source_row,
        original_id: None,
        original_asset_number: row.asset_number.clone(),
        original_serial_number: row.serial_number.clone(),
    }
}

fn is_active_required(entry: &InventoryEntry) -> bool {
    !entry.archived
        && entry.lifecycle_status == "active"
        && entry.calibration_requirement == CalibrationRequirement::Required
}

fn commit_signature(input: &CalibrationRosterCommitInput) -> CommandResult<String> {
    let mut canonical = input.clone();
    canonical.resolutions.sort_by(|left, right| {
        (&left.source_sheet, left.source_row).cmp(&(&right.source_sheet, right.source_row))
    });
    let bytes = serde_json::to_vec(&canonical)
        .map_err(|error| format!("Could not encode calibration roster commit: {error}"))?;
    Ok(format!("sha256-{}", digest(&bytes)))
}

fn deterministic_entry_uuid(batch_id: &str, sheet: &str, source_row: u64) -> String {
    let digest = Sha256::digest(format!("{batch_id}\0{sheet}\0{source_row}").as_bytes());
    digest[..16]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn row_operation_id(sheet: &str, source_row: u64) -> String {
    format!(
        "row-{}",
        digest(format!("{sheet}\0{source_row}").as_bytes())
    )
}

fn reset_operation_id(entry_uuid: &str) -> String {
    format!("reset-{}", digest(entry_uuid.as_bytes()))
}

fn row_key(sheet: &str, source_row: u64) -> String {
    format!("{sheet}\0{source_row}")
}

fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
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

fn maybe_inject_failure(
    inject_after_mutations_before_marker: Option<usize>,
    mutation_count: usize,
    row: &CalibrationRosterRowOutcome,
) -> CommandResult<()> {
    if inject_after_mutations_before_marker == Some(mutation_count) {
        return Err(format!(
            "injected calibration roster failure after mutation but before marker at {} row {}",
            row.source_sheet, row.source_row
        ));
    }
    Ok(())
}
