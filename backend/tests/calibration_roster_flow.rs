#![allow(dead_code)]

#[path = "../src/domain/entry_changes.rs"]
pub(crate) mod entry_changes_impl;
#[path = "../src/domain/model.rs"]
pub(crate) mod model;
#[allow(dead_code, unused_imports)]
#[path = "../src/platform/mod.rs"]
pub(crate) mod platform;
#[path = "../src/storage/mod.rs"]
pub(crate) mod store;
#[allow(dead_code, unused_imports)]
#[path = "../src/sync/mod.rs"]
pub(crate) mod sync;

pub(crate) mod domain {
    pub(crate) use crate::entry_changes_impl as entry_changes;
}

#[path = "../src/api/mutations.rs"]
pub(crate) mod mutations_impl;
pub(crate) mod api {
    pub(crate) use crate::mutations_impl as mutations;
}

#[path = "../src/integrations/calibration_roster_import/mod.rs"]
pub(crate) mod calibration_roster_import;

use std::{collections::BTreeMap, fs, path::PathBuf};

use calibration_roster_import::{
    commit_calibration_roster_from_store, commit_calibration_roster_with_test_failure_after,
    preview_calibration_roster_from_path, CalibrationRosterClassification,
    CalibrationRosterCommitInput, CalibrationRosterResolution, CalibrationRosterResolutionAction,
    CalibrationRosterRowOutcome, CalibrationRosterSemanticFlag,
};
use model::{CalibrationRequirement, ImportProvenance, InventoryEntryInput};
use rust_xlsxwriter::{ExcelDateTime, Format, Workbook, Worksheet};
use store::InventoryDb;
use uuid::Uuid;

#[test]
fn copied_calibration_roster_preview_aggregates_only_when_requested() {
    if std::env::var("IM015_CALIBRATION_PREVIEW").ok().as_deref() != Some("1") {
        return;
    }

    let workbook_path = PathBuf::from(
        std::env::var("TE_CALIBRATION_ROSTER_XLSX")
            .expect("TE_CALIBRATION_ROSTER_XLSX must point to a copied workbook"),
    );
    let db_path = PathBuf::from(
        std::env::var("TE_CALIBRATION_ROSTER_DB_COPY")
            .expect("TE_CALIBRATION_ROSTER_DB_COPY must point to a copied database"),
    );
    let file_size = fs::metadata(&db_path).unwrap().len();
    let db = InventoryDb::open_at_with_size(db_path, file_size).unwrap();
    let entries_before = db.load_entries().unwrap();
    let entry_count_before = entries_before.len();
    let outbox_count_before = outbox_count(&db);

    let report = preview_calibration_roster_from_path(&workbook_path, &db).unwrap();
    let mut classifications_by_sheet = BTreeMap::<String, [usize; 5]>::new();
    let mut create_identity_shapes = [0usize; 4];
    let mut create_hint_shapes = [0usize; 3];
    for row in &report.row_outcomes {
        let counts = classifications_by_sheet
            .entry(row.source_sheet.clone())
            .or_default();
        counts[match row.classification {
            CalibrationRosterClassification::MatchedUpdate => 0,
            CalibrationRosterClassification::CreateCandidate => 1,
            CalibrationRosterClassification::ConflictReviewRequired => 2,
            CalibrationRosterClassification::DuplicateSourceRow => 3,
            CalibrationRosterClassification::IgnoredJunk => 4,
        }] += 1;
        if row.classification == CalibrationRosterClassification::CreateCandidate {
            create_identity_shapes[match (row.asset_number.is_some(), row.serial_number.is_some()) {
                (true, true) => 0,
                (true, false) => 1,
                (false, true) => 2,
                (false, false) => 3,
            }] += 1;
            create_hint_shapes[match row.candidate_entries.len() {
                0 => 0,
                1 => 1,
                _ => 2,
            }] += 1;
        }
    }

    println!(
        "calibration_roster_preview: sheets={} source_rows={} matched_updates={} create_candidates={} conflicts={} duplicate_source_rows={} ignored_junk={} current_required_absent={} prospective_required={} blocking={}",
        report.contributing_sheets.len(),
        report.total_source_rows,
        report.counts.matched_updates,
        report.counts.create_candidates,
        report.counts.conflicts,
        report.counts.duplicate_source_rows,
        report.counts.ignored_junk,
        report.counts.current_required_absent,
        report.prospective_required_count,
        report.blocking,
    );
    println!(
        "calibration_roster_preview_by_sheet: [matched, create, conflict, duplicate, junk]={classifications_by_sheet:?}"
    );
    println!(
        "calibration_roster_create_identity_shapes: both={} asset_only={} serial_only={} neither={}",
        create_identity_shapes[0],
        create_identity_shapes[1],
        create_identity_shapes[2],
        create_identity_shapes[3],
    );
    println!(
        "calibration_roster_create_hint_shapes: none={} unique={} multiple={}",
        create_hint_shapes[0], create_hint_shapes[1], create_hint_shapes[2],
    );
    assert_eq!(
        report.total_source_rows,
        report.counts.matched_updates
            + report.counts.create_candidates
            + report.counts.conflicts
            + report.counts.duplicate_source_rows
            + report.counts.ignored_junk
    );
    assert_eq!(db.load_entries().unwrap().len(), entry_count_before);
    assert_eq!(outbox_count(&db), outbox_count_before);
}

#[test]
fn preview_merges_nonfirst_headers_and_classifies_semantics_and_identity_conflicts() {
    let db = test_db("preview");
    let matched_entry = create_entry(
        &db,
        equipment_input(
            "MATCH-1",
            "SER-1",
            "Existing Maker",
            "M1",
            "Existing matched",
        ),
    );
    create_entry(
        &db,
        equipment_input("ASSET-A", "SER-A", "Maker A", "A", "Asset candidate"),
    );
    create_entry(
        &db,
        equipment_input("ASSET-B", "SER-B", "Maker B", "B", "Serial candidate"),
    );
    create_entry(
        &db,
        equipment_input("DB-DUP", "DB-S1", "Maker D", "D1", "Duplicate one"),
    );
    create_entry(
        &db,
        equipment_input("DB-DUP", "DB-S2", "Maker D", "D2", "Duplicate two"),
    );
    create_entry(
        &db,
        equipment_input("SOURCE-DUP", "SOURCE-S", "Maker S", "S", "Source duplicate"),
    );
    let swapped_entry = create_entry(
        &db,
        equipment_input(
            "SWAP-ASSET",
            "SWAP-SERIAL",
            "Maker Swap",
            "SW",
            "Swapped identity candidate",
        ),
    );
    let path = unique_test_dir("preview-workbook").join("calibration.xlsx");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    write_preview_workbook(&path);

    let report = preview_calibration_roster_from_path(&path, &db).unwrap();

    assert_eq!(
        report.contributing_sheets,
        vec!["May Roster", "October Roster"]
    );
    assert_eq!(report.total_source_rows, 12);
    assert_eq!(report.counts.matched_updates, 1);
    assert_eq!(report.counts.create_candidates, 3);
    assert_eq!(report.counts.conflicts, 3);
    assert_eq!(report.counts.duplicate_source_rows, 2);
    assert_eq!(report.counts.ignored_junk, 3);
    assert!(report.blocking);

    let matched = row(&report, "May Roster", 4);
    assert_eq!(
        matched.classification,
        CalibrationRosterClassification::MatchedUpdate
    );
    assert!(matched
        .ignored_identity_placeholders
        .iter()
        .any(|issue| issue.contains("NSN")));
    assert!(matched
        .semantic_flags
        .contains(&CalibrationRosterSemanticFlag::FailedCalibration));
    assert_eq!(
        matched
            .proposed_input
            .as_ref()
            .and_then(|input| input.last_calibrated_at.as_deref()),
        Some("2026-05-31")
    );
    assert_eq!(
        matched
            .proposed_input
            .as_ref()
            .and_then(|input| input.calibration_due_at.as_deref()),
        Some("2027-05-31")
    );

    let reference = row(&report, "May Roster", 6);
    assert_eq!(
        reference.proposed_requirement,
        CalibrationRequirement::ReferenceOnly
    );
    assert!(reference
        .semantic_flags
        .contains(&CalibrationRosterSemanticFlag::ReferenceOnly));

    let out = row(&report, "May Roster", 7);
    assert!(out.proposed_out_to_calibration);
    assert!(out
        .semantic_flags
        .contains(&CalibrationRosterSemanticFlag::OutToCalibration));
    assert!(out
        .ignored_identity_placeholders
        .iter()
        .any(|issue| issue.contains("xxxxxxxx")));

    let disagreement = row(&report, "October Roster", 5);
    assert_eq!(
        disagreement.classification,
        CalibrationRosterClassification::ConflictReviewRequired
    );
    assert_eq!(disagreement.candidate_entries.len(), 2);
    let manufacturer_hint_only = row(&report, "October Roster", 8);
    assert_eq!(
        manufacturer_hint_only.classification,
        CalibrationRosterClassification::CreateCandidate
    );
    assert!(manufacturer_hint_only.candidate_entry_uuid.is_none());
    assert_eq!(
        manufacturer_hint_only
            .candidate_entries
            .iter()
            .map(|candidate| candidate.entry_uuid.as_str())
            .collect::<Vec<_>>(),
        vec![matched_entry.entry_uuid.as_str()]
    );
    assert!(manufacturer_hint_only
        .issues
        .iter()
        .any(|issue| issue.to_ascii_lowercase().contains("review hints")));

    let swapped = row(&report, "October Roster", 9);
    assert_eq!(
        swapped.classification,
        CalibrationRosterClassification::ConflictReviewRequired
    );
    assert!(swapped.candidate_entry_uuid.is_none());
    assert_eq!(
        swapped
            .candidate_entries
            .iter()
            .map(|candidate| candidate.entry_uuid.as_str())
            .collect::<Vec<_>>(),
        vec![swapped_entry.entry_uuid.as_str()]
    );
    assert!(swapped
        .issues
        .iter()
        .any(|issue| issue.to_ascii_lowercase().contains("swapped")));
}

#[test]
fn commit_accepts_explicit_existing_resolution_for_hint_only_create_candidate() {
    let db = test_db("hint-existing");
    let existing = create_entry(
        &db,
        equipment_input(
            "HINT-ASSET",
            "HINT-SERIAL",
            "Hint Maker",
            "HM1",
            "Existing hinted equipment",
        ),
    );
    let path = hint_only_workbook();
    let report = preview_calibration_roster_from_path(&path, &db).unwrap();
    let hinted = report.row_outcomes.first().unwrap();

    assert_eq!(
        hinted.classification,
        CalibrationRosterClassification::CreateCandidate
    );
    assert!(hinted.candidate_entry_uuid.is_none());
    assert_eq!(
        hinted
            .candidate_entries
            .iter()
            .map(|candidate| candidate.entry_uuid.as_str())
            .collect::<Vec<_>>(),
        vec![existing.entry_uuid.as_str()]
    );

    let result = commit_calibration_roster_from_store(
        CalibrationRosterCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            replace_active_required_roster: true,
            verification_attribution: "Synthetic hinted resolution".to_string(),
            resolutions: vec![CalibrationRosterResolution {
                source_sheet: hinted.source_sheet.clone(),
                source_row: hinted.source_row,
                action: CalibrationRosterResolutionAction::UseExisting,
                target_entry_uuid: Some(existing.entry_uuid.clone()),
                create_input: None,
                confirmed: true,
            }],
        },
        &db,
    )
    .unwrap();

    assert_eq!(result.updated, 1);
    assert_eq!(result.created, 0);
    let updated = db.find_entry(&existing.entry_uuid).unwrap().unwrap();
    assert_eq!(
        updated.calibration_requirement,
        CalibrationRequirement::Required
    );
    assert_eq!(
        updated.verified_by.as_deref(),
        Some("Synthetic hinted resolution")
    );
}

#[test]
fn commit_updates_only_calibration_fields_creates_reviewed_rows_and_resets_absent_required() {
    let db = test_db("commit");
    let matched = create_entry(
        &db,
        InventoryEntryInput {
            calibration_requirement: CalibrationRequirement::Required,
            out_to_calibration: true,
            last_calibrated_at: Some("2025-01-01".to_string()),
            calibration_due_at: Some("2026-01-01".to_string()),
            calibration_vendor: Some("Old Vendor".to_string()),
            calibration_notes: Some("Historical note".to_string()),
            verified_at: Some("2025-01-02T00:00:00Z".to_string()),
            verified_by: Some("Prior reviewer".to_string()),
            location: "Original location".to_string(),
            ..equipment_input(
                "MATCH-1",
                "SER-1",
                "Existing Maker",
                "M1",
                "Original description",
            )
        },
    );
    let mut matched_entry = matched.clone();
    matched_entry.import_provenance = Some(ImportProvenance {
        batch_id: "batch-original-123456".to_string(),
        source_filename: "original.xlsx".to_string(),
        source_sheet: Some("Inventory".to_string()),
        source_row: 2,
        original_id: None,
        original_asset_number: Some("MATCH-1".to_string()),
        original_serial_number: Some("SER-1".to_string()),
    });
    db.put_entry(&matched_entry).unwrap();
    let absent = create_entry(
        &db,
        InventoryEntryInput {
            calibration_requirement: CalibrationRequirement::Required,
            out_to_calibration: true,
            last_calibrated_at: Some("2024-01-01".to_string()),
            calibration_due_at: Some("2025-01-01".to_string()),
            calibration_vendor: Some("Preserved Vendor".to_string()),
            calibration_notes: Some("Preserved history".to_string()),
            verified_at: Some("2025-01-02T00:00:00Z".to_string()),
            verified_by: Some("Preserved reviewer".to_string()),
            ..equipment_input("ABSENT-1", "ABSENT-S", "Maker", "A", "Absent required")
        },
    );
    let archived = create_entry(
        &db,
        InventoryEntryInput {
            calibration_requirement: CalibrationRequirement::Required,
            archived: true,
            ..equipment_input("ARCHIVED-1", "ARCH-S", "Maker", "R", "Archived required")
        },
    );
    let reference = create_entry(
        &db,
        InventoryEntryInput {
            calibration_requirement: CalibrationRequirement::ReferenceOnly,
            ..equipment_input("REFERENCE-1", "REF-S", "Maker", "R", "Reference only")
        },
    );
    let outbox_before = outbox_count(&db);
    let path = simple_cutover_workbook("commit");
    let report = preview_calibration_roster_from_path(&path, &db).unwrap();
    let create_row = report
        .row_outcomes
        .iter()
        .find(|row| row.classification == CalibrationRosterClassification::CreateCandidate)
        .unwrap();
    let junk_row = report
        .row_outcomes
        .iter()
        .find(|row| row.classification == CalibrationRosterClassification::IgnoredJunk)
        .unwrap();
    let matched_row = report
        .row_outcomes
        .iter()
        .find(|row| row.candidate_entry_uuid.as_deref() == Some(matched_entry.entry_uuid.as_str()))
        .unwrap();
    let commit_input = CalibrationRosterCommitInput {
        batch_id: report.batch_id.clone(),
        confirmed: true,
        replace_active_required_roster: true,
        verification_attribution: "Synthetic calibration roster cutover".to_string(),
        resolutions: vec![
            use_existing_resolution(matched_row),
            create_resolution(create_row),
            ignore_resolution(junk_row),
        ],
    };

    let result = commit_calibration_roster_from_store(commit_input.clone(), &db).unwrap();

    assert_eq!(result.updated, 1);
    assert_eq!(result.created, 1);
    assert_eq!(result.reset, 1);
    assert_eq!(result.ignored, 1);
    assert!(result.entries_changed);
    assert_eq!(outbox_count(&db), outbox_before + 3);

    let entries = db.load_entries().unwrap();
    let matched_after = find_asset(&entries, "MATCH-1");
    assert_eq!(matched_after.description, "Original description");
    assert_eq!(matched_after.location, "Original location");
    assert_eq!(matched_after.manufacturer, "Existing Maker");
    assert_eq!(
        matched_after.last_calibrated_at.as_deref(),
        Some("2026-05-31")
    );
    assert_eq!(
        matched_after.calibration_due_at.as_deref(),
        Some("2027-05-31")
    );
    assert!(!matched_after.out_to_calibration);
    assert_eq!(
        matched_after.verified_by.as_deref(),
        Some("Synthetic calibration roster cutover")
    );
    assert!(matched_after
        .verified_at
        .as_deref()
        .is_some_and(|value| value.contains('T')));
    assert_eq!(
        matched_after.import_provenance,
        matched_entry.import_provenance
    );
    assert!(matched_after
        .calibration_notes
        .as_deref()
        .is_some_and(
            |notes| notes.contains("Historical note") && notes.contains("Workbook comment")
        ));

    let created = find_asset(&entries, "NEW-1");
    assert_eq!(
        created.calibration_requirement,
        CalibrationRequirement::Required
    );
    assert_eq!(
        created.verified_by.as_deref(),
        Some("Synthetic calibration roster cutover")
    );
    assert_eq!(
        created
            .import_provenance
            .as_ref()
            .and_then(|provenance| provenance.source_sheet.as_deref()),
        Some("May Roster")
    );
    let absent_after = find_uuid(&entries, &absent.entry_uuid);
    assert_eq!(
        absent_after.calibration_requirement,
        CalibrationRequirement::Unknown
    );
    assert!(!absent_after.out_to_calibration);
    assert_eq!(absent_after.calibration_due_at, absent.calibration_due_at);
    assert_eq!(absent_after.calibration_vendor, absent.calibration_vendor);
    assert_eq!(absent_after.calibration_notes, absent.calibration_notes);
    assert_eq!(absent_after.verified_at, absent.verified_at);
    assert_eq!(
        find_uuid(&entries, &archived.entry_uuid).calibration_requirement,
        CalibrationRequirement::Required
    );
    assert_eq!(
        find_uuid(&entries, &reference.entry_uuid).calibration_requirement,
        CalibrationRequirement::ReferenceOnly
    );

    let outbox_after = outbox_count(&db);
    let repeated = commit_calibration_roster_from_store(commit_input, &db).unwrap();
    assert!(!repeated.entries_changed);
    assert_eq!(repeated.created, 0);
    assert_eq!(outbox_count(&db), outbox_after);
}

#[test]
fn commit_rejects_stale_workbook_and_inventory_basis() {
    let workbook_db = test_db("stale-workbook");
    create_entry(
        &workbook_db,
        equipment_input("MATCH-1", "SER-1", "Maker", "M", "Matched"),
    );
    let workbook_path = simple_cutover_workbook("stale-workbook");
    let report = preview_calibration_roster_from_path(&workbook_path, &workbook_db).unwrap();
    fs::write(&workbook_path, b"changed after preview").unwrap();
    let error =
        commit_calibration_roster_from_store(commit_input_for_report(&report), &workbook_db)
            .unwrap_err();
    assert!(error.to_ascii_lowercase().contains("workbook"));

    let basis_db = test_db("stale-basis");
    create_entry(
        &basis_db,
        equipment_input("MATCH-1", "SER-1", "Maker", "M", "Matched"),
    );
    let basis_path = simple_cutover_workbook("stale-basis");
    let report = preview_calibration_roster_from_path(&basis_path, &basis_db).unwrap();
    create_entry(
        &basis_db,
        equipment_input("LATE-1", "LATE-S", "Maker", "L", "Late database row"),
    );
    let error = commit_calibration_roster_from_store(commit_input_for_report(&report), &basis_db)
        .unwrap_err();
    assert!(error.to_ascii_lowercase().contains("basis"));
}

#[test]
fn interrupted_commit_resumes_without_duplicate_entries_or_outbox_operations() {
    let db = test_db("resume");
    create_entry(
        &db,
        InventoryEntryInput {
            calibration_requirement: CalibrationRequirement::Required,
            ..equipment_input("MATCH-1", "SER-1", "Maker", "M", "Matched")
        },
    );
    create_entry(
        &db,
        InventoryEntryInput {
            calibration_requirement: CalibrationRequirement::Required,
            ..equipment_input("ABSENT-1", "ABSENT-S", "Maker", "A", "Absent")
        },
    );
    let outbox_before = outbox_count(&db);
    let path = simple_cutover_workbook("resume");
    let report = preview_calibration_roster_from_path(&path, &db).unwrap();
    let input = commit_input_for_report(&report);

    let error =
        commit_calibration_roster_with_test_failure_after(input.clone(), &db, 1).unwrap_err();
    assert!(error.contains("before marker"));
    assert_eq!(outbox_count(&db), outbox_before + 1);

    let result = commit_calibration_roster_from_store(input.clone(), &db).unwrap();
    assert!(result.entries_changed);
    assert_eq!(
        db.load_entries()
            .unwrap()
            .iter()
            .filter(|entry| entry.asset_number == "NEW-1")
            .count(),
        1
    );
    assert_eq!(outbox_count(&db), outbox_before + 3);

    let repeated = commit_calibration_roster_from_store(input, &db).unwrap();
    assert!(!repeated.entries_changed);
    assert_eq!(outbox_count(&db), outbox_before + 3);
}

fn write_preview_workbook(path: &PathBuf) {
    let mut workbook = Workbook::new();
    {
        let sheet = workbook.add_worksheet();
        sheet.set_name("May Roster").unwrap();
        sheet.write_string(0, 0, "Synthetic title").unwrap();
        sheet.write_string(1, 0, "Synthetic subtitle").unwrap();
        write_row(
            sheet,
            2,
            &[
                "Check",
                "Asset Number",
                "Serial Number",
                "Description",
                "New Cal Date",
                "Cal Due Date",
                "Status",
                "Location",
                "OWNER",
                "COMMENTS",
            ],
        );
        write_row(
            sheet,
            3,
            &[
                "√",
                "MATCH-1",
                "NSN",
                "Workbook description",
                "",
                "",
                "ACTIVE",
                "Workbook location",
                "Reviewer",
                "FAILED CALIBRATION - synthetic",
            ],
        );
        write_excel_date(sheet, 3, 4, 2026, 5, 31);
        write_excel_date(sheet, 3, 5, 2027, 5, 31);
        write_row(sheet, 4, &["Ck List FOUND"]);
        write_row(
            sheet,
            5,
            &[
                "√",
                "REF-NEW",
                "REF-S",
                "Synthetic reference",
                "REF ONLY",
                "NO CAL NEEDED",
            ],
        );
        write_row(
            sheet,
            6,
            &[
                "√",
                "OUT-NEW",
                "xxxxxxxx",
                "Synthetic out to calibration",
                "ACCURA",
                "Accura Has",
                "ACTIVE",
                "Lab",
                "Owner",
                "ACCURA Has",
            ],
        );
        write_row(sheet, 7, &["", "Newly ADDED To List Below"]);
        write_row(sheet, 8, &["", "", "", "`"]);
    }
    {
        let sheet = workbook.add_worksheet();
        sheet.set_name("October Roster").unwrap();
        sheet.write_string(0, 0, "Synthetic October title").unwrap();
        sheet.write_string(1, 0, "Legend").unwrap();
        write_row(
            sheet,
            2,
            &[
                "Check",
                "Blue Dot#",
                "Asset Numbers",
                "Serial Number",
                "Manufacturer",
                "Model",
                "Description",
                "Last Calibration Date",
                "Calibration Due Date",
                "Last Calibrated by",
                "Location",
                "Assigned to",
                "Condition",
            ],
        );
        write_row(
            sheet,
            3,
            &[
                "",
                "1",
                "ASSET-A",
                "SER-B",
                "Maker",
                "X",
                "Identifier disagreement",
                "2026-01-01",
                "2027-01-01",
                "Vendor",
                "Lab",
                "Owner",
                "working",
            ],
        );
        write_row(
            sheet,
            4,
            &[
                "",
                "2",
                "DB-DUP",
                "",
                "Maker D",
                "D",
                "Database duplicate",
                "2026-01-01",
                "2027-01-01",
            ],
        );
        write_row(
            sheet,
            5,
            &[
                "",
                "3",
                "SOURCE-DUP",
                "SOURCE-S",
                "Maker S",
                "S",
                "Source duplicate one",
                "2026-01-01",
                "2027-01-01",
            ],
        );
        write_row(
            sheet,
            6,
            &[
                "",
                "4",
                "SOURCE-DUP",
                "SOURCE-S",
                "Maker S",
                "S",
                "Source duplicate two",
                "2026-01-01",
                "2027-01-01",
            ],
        );
        write_row(
            sheet,
            7,
            &[
                "",
                "5",
                "",
                "",
                "Existing Maker",
                "M1",
                "Manufacturer hint only",
                "2026-01-01",
                "2027-01-01",
            ],
        );
        write_row(
            sheet,
            8,
            &[
                "",
                "6",
                "SWAP-SERIAL",
                "SWAP-ASSET",
                "Maker Swap",
                "SW",
                "Swapped identity columns",
                "2026-01-01",
                "2027-01-01",
            ],
        );
    }
    workbook.save(path).unwrap();
}

fn simple_cutover_workbook(prefix: &str) -> PathBuf {
    let root = unique_test_dir(prefix);
    fs::create_dir_all(&root).unwrap();
    let path = root.join("calibration.xlsx");
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("May Roster").unwrap();
    sheet.write_string(0, 0, "Synthetic title").unwrap();
    write_row(
        sheet,
        1,
        &[
            "Check",
            "Asset Number",
            "Serial Number",
            "Description",
            "New Cal Date",
            "Cal Due Date",
            "Status",
            "Location",
            "OWNER",
            "COMMENTS",
        ],
    );
    write_row(
        sheet,
        2,
        &[
            "√",
            "MATCH-1",
            "SER-1",
            "Workbook description must not replace equipment",
            "",
            "",
            "ACTIVE",
            "Workbook location",
            "Workbook owner",
            "Workbook comment",
        ],
    );
    write_excel_date(sheet, 2, 4, 2026, 5, 31);
    write_excel_date(sheet, 2, 5, 2027, 5, 31);
    write_row(
        sheet,
        3,
        &[
            "√",
            "NEW-1",
            "NEW-S",
            "Reviewed new equipment",
            "2026-06-01",
            "2027-06-01",
            "ACTIVE",
            "Lab",
            "Owner",
            "New calibration note",
        ],
    );
    write_row(sheet, 4, &["Ck List FOUND"]);
    workbook.save(&path).unwrap();
    path
}

fn hint_only_workbook() -> PathBuf {
    let root = unique_test_dir("hint-only");
    fs::create_dir_all(&root).unwrap();
    let path = root.join("calibration.xlsx");
    let mut workbook = Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("October Roster").unwrap();
    write_row(
        sheet,
        0,
        &[
            "Asset Numbers",
            "Serial Number",
            "Manufacturer",
            "Model",
            "Description",
            "Last Calibration Date",
            "Calibration Due Date",
        ],
    );
    write_row(
        sheet,
        1,
        &[
            "",
            "",
            "Hint Maker",
            "HM1",
            "Workbook hinted equipment",
            "2026-01-01",
            "2027-01-01",
        ],
    );
    workbook.save(&path).unwrap();
    path
}

fn commit_input_for_report(
    report: &calibration_roster_import::CalibrationRosterPreviewReport,
) -> CalibrationRosterCommitInput {
    let resolutions = report
        .row_outcomes
        .iter()
        .filter_map(|row| match row.classification {
            CalibrationRosterClassification::CreateCandidate => Some(create_resolution(row)),
            CalibrationRosterClassification::IgnoredJunk => Some(ignore_resolution(row)),
            _ if row.requires_review => Some(CalibrationRosterResolution {
                source_sheet: row.source_sheet.clone(),
                source_row: row.source_row,
                action: CalibrationRosterResolutionAction::UseExisting,
                target_entry_uuid: row.candidate_entry_uuid.clone(),
                create_input: None,
                confirmed: true,
            }),
            _ => None,
        })
        .collect();
    CalibrationRosterCommitInput {
        batch_id: report.batch_id.clone(),
        confirmed: true,
        replace_active_required_roster: true,
        verification_attribution: "Synthetic calibration roster cutover".to_string(),
        resolutions,
    }
}

fn create_resolution(row: &CalibrationRosterRowOutcome) -> CalibrationRosterResolution {
    CalibrationRosterResolution {
        source_sheet: row.source_sheet.clone(),
        source_row: row.source_row,
        action: CalibrationRosterResolutionAction::Create,
        target_entry_uuid: None,
        create_input: row.proposed_input.clone(),
        confirmed: true,
    }
}

fn ignore_resolution(row: &CalibrationRosterRowOutcome) -> CalibrationRosterResolution {
    CalibrationRosterResolution {
        source_sheet: row.source_sheet.clone(),
        source_row: row.source_row,
        action: CalibrationRosterResolutionAction::Ignore,
        target_entry_uuid: None,
        create_input: None,
        confirmed: true,
    }
}

fn use_existing_resolution(row: &CalibrationRosterRowOutcome) -> CalibrationRosterResolution {
    CalibrationRosterResolution {
        source_sheet: row.source_sheet.clone(),
        source_row: row.source_row,
        action: CalibrationRosterResolutionAction::UseExisting,
        target_entry_uuid: row.candidate_entry_uuid.clone(),
        create_input: None,
        confirmed: true,
    }
}

fn row<'a>(
    report: &'a calibration_roster_import::CalibrationRosterPreviewReport,
    sheet: &str,
    source_row: u64,
) -> &'a CalibrationRosterRowOutcome {
    report
        .row_outcomes
        .iter()
        .find(|row| row.source_sheet == sheet && row.source_row == source_row)
        .unwrap()
}

fn create_entry(db: &InventoryDb, input: InventoryEntryInput) -> model::InventoryEntry {
    api::mutations::create_entry_in_store(input, db)
        .unwrap()
        .entry
}

fn equipment_input(
    asset: &str,
    serial: &str,
    manufacturer: &str,
    model: &str,
    description: &str,
) -> InventoryEntryInput {
    InventoryEntryInput {
        asset_number: asset.to_string(),
        serial_number: serial.to_string(),
        manufacturer: manufacturer.to_string(),
        model: model.to_string(),
        description: description.to_string(),
        lifecycle_status: "active".to_string(),
        working_status: "unknown".to_string(),
        ..InventoryEntryInput::default()
    }
}

fn find_asset<'a>(entries: &'a [model::InventoryEntry], asset: &str) -> &'a model::InventoryEntry {
    entries
        .iter()
        .find(|entry| entry.asset_number == asset)
        .unwrap()
}

fn find_uuid<'a>(
    entries: &'a [model::InventoryEntry],
    entry_uuid: &str,
) -> &'a model::InventoryEntry {
    entries
        .iter()
        .find(|entry| entry.entry_uuid == entry_uuid)
        .unwrap()
}

fn outbox_count(db: &InventoryDb) -> usize {
    let mut count = 0;
    db.scan_sync_outbox_records::<serde_json::Value, _>(None, usize::MAX, |_, _| {
        count += 1;
        Ok(true)
    })
    .unwrap();
    count
}

fn test_db(prefix: &str) -> InventoryDb {
    let root = unique_test_dir(prefix);
    fs::create_dir_all(&root).unwrap();
    InventoryDb::open_at(root.join("inventory.feox")).unwrap()
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "te-calibration-roster-{prefix}-{}",
        Uuid::new_v4().simple()
    ))
}

fn write_row(sheet: &mut Worksheet, row: u32, values: &[&str]) {
    for (column, value) in values.iter().enumerate() {
        sheet.write_string(row, column as u16, *value).unwrap();
    }
}

fn write_excel_date(sheet: &mut Worksheet, row: u32, column: u16, year: u16, month: u8, day: u8) {
    let date = ExcelDateTime::from_ymd(year, month, day).unwrap();
    let format = Format::new().set_num_format("yyyy-mm-dd");
    sheet
        .write_datetime_with_format(row, column, &date, &format)
        .unwrap();
}

#[test]
fn apply_owner_calibration_roster_cutover_when_requested() {
    if std::env::var("IM015_CALIBRATION_COMMIT").ok().as_deref() != Some("1") {
        return;
    }

    let workbook_path = PathBuf::from(
        std::env::var("TE_CALIBRATION_ROSTER_XLSX")
            .expect("TE_CALIBRATION_ROSTER_XLSX must point to the calibration workbook"),
    );
    let db_path = PathBuf::from(
        std::env::var("TE_CALIBRATION_ROSTER_DB_COPY")
            .expect("TE_CALIBRATION_ROSTER_DB_COPY must point to the target database file"),
    );
    let file_size = fs::metadata(&db_path).unwrap().len();
    let db = InventoryDb::open_at_with_size(db_path, file_size).unwrap();

    let report = preview_calibration_roster_from_path(&workbook_path, &db).unwrap();
    println!(
        "preview: matched={} create={} conflicts={} dups={} junk={} absent={} blocking={}",
        report.counts.matched_updates,
        report.counts.create_candidates,
        report.counts.conflicts,
        report.counts.duplicate_source_rows,
        report.counts.ignored_junk,
        report.counts.current_required_absent,
        report.blocking
    );

    let mut seen_targets = std::collections::BTreeSet::<String>::new();
    for row in &report.row_outcomes {
        if row.classification == CalibrationRosterClassification::MatchedUpdate {
            if let Some(target) = row.candidate_entry_uuid.as_ref() {
                seen_targets.insert(target.clone());
            }
        }
    }
    let mut resolutions = Vec::new();
    for row in &report.row_outcomes {
        match row.classification {
            CalibrationRosterClassification::MatchedUpdate => {
                // Some matched rows still require_review (semantics / notes) and need confirmation.
                if row.requires_review {
                    if let Some(target) = row.candidate_entry_uuid.clone() {
                        if seen_targets.insert(target.clone()) {
                            resolutions.push(CalibrationRosterResolution {
                                source_sheet: row.source_sheet.clone(),
                                source_row: row.source_row,
                                action: CalibrationRosterResolutionAction::UseExisting,
                                target_entry_uuid: Some(target),
                                create_input: None,
                                confirmed: true,
                            });
                        } else {
                            resolutions.push(ignore_resolution(row));
                        }
                    } else {
                        resolutions.push(ignore_resolution(row));
                    }
                }
            }
            CalibrationRosterClassification::IgnoredJunk => {
                resolutions.push(ignore_resolution(row));
            }
            CalibrationRosterClassification::DuplicateSourceRow => {
                if let Some(target) = row
                    .candidate_entry_uuid
                    .clone()
                    .or_else(|| row.candidate_entries.first().map(|c| c.entry_uuid.clone()))
                {
                    if seen_targets.insert(target.clone()) {
                        resolutions.push(CalibrationRosterResolution {
                            source_sheet: row.source_sheet.clone(),
                            source_row: row.source_row,
                            action: CalibrationRosterResolutionAction::UseExisting,
                            target_entry_uuid: Some(target),
                            create_input: None,
                            confirmed: true,
                        });
                    } else {
                        resolutions.push(ignore_resolution(row));
                    }
                } else {
                    resolutions.push(ignore_resolution(row));
                }
            }
            CalibrationRosterClassification::CreateCandidate => {
                if row.candidate_entries.len() == 1 {
                    let target = row.candidate_entries[0].entry_uuid.clone();
                    if seen_targets.insert(target.clone()) {
                        resolutions.push(CalibrationRosterResolution {
                            source_sheet: row.source_sheet.clone(),
                            source_row: row.source_row,
                            action: CalibrationRosterResolutionAction::UseExisting,
                            target_entry_uuid: Some(target),
                            create_input: None,
                            confirmed: true,
                        });
                    } else {
                        resolutions.push(ignore_resolution(row));
                    }
                } else {
                    // Unmatched workbook rows (including multi-hint noise) stay out of inventory/calibration.
                    resolutions.push(ignore_resolution(row));
                }
            }
            CalibrationRosterClassification::ConflictReviewRequired => {
                let target = resolve_conflict_target(row);
                if let Some(target) = target {
                    if seen_targets.insert(target.clone()) {
                        resolutions.push(CalibrationRosterResolution {
                            source_sheet: row.source_sheet.clone(),
                            source_row: row.source_row,
                            action: CalibrationRosterResolutionAction::UseExisting,
                            target_entry_uuid: Some(target),
                            create_input: None,
                            confirmed: true,
                        });
                    } else {
                        resolutions.push(ignore_resolution(row));
                    }
                } else {
                    resolutions.push(ignore_resolution(row));
                }
            }
        }
    }

    let input = CalibrationRosterCommitInput {
        batch_id: report.batch_id.clone(),
        confirmed: true,
        replace_active_required_roster: true,
        verification_attribution: "Calibration tracking workbook cutover 2026-07-27".to_string(),
        resolutions,
    };

    let result = commit_calibration_roster_from_store(input, &db).unwrap();
    println!(
        "commit: updated={} created={} reset={} ignored={} noop={} final_required={} changed={} message={}",
        result.updated,
        result.created,
        result.reset,
        result.ignored,
        result.noop,
        result.final_required,
        result.entries_changed,
        result.message
    );

    let entries = db.load_entries().unwrap();
    let required: Vec<_> = entries
        .iter()
        .filter(|e| !e.archived && e.calibration_requirement == CalibrationRequirement::Required)
        .collect();
    let required_verified = required.iter().filter(|e| e.verified_at.is_some()).count();
    let required_pending = required.len() - required_verified;
    println!(
        "post: total={} active_required={} required_verified={} required_pending={}",
        entries.len(),
        required.len(),
        required_verified,
        required_pending
    );
    assert!(result.entries_changed, "cutover should mutate entries");
    assert_eq!(result.created, 0, "owner asked not to invent missing inventory rows");
    if required_pending > 0 {
        for entry in &required {
            if entry.verified_at.is_none() {
                println!(
                    "pending required: asset={} serial={} desc={} uuid={}",
                    entry.asset_number, entry.serial_number, entry.description, entry.entry_uuid
                );
            }
        }
    }
    // Prefer verified for all approved workbook matches; report any leftovers for owner review.
    assert!(required_verified >= 78, "expected at least the matched workbook set verified");
}

fn resolve_conflict_target(row: &CalibrationRosterRowOutcome) -> Option<String> {
    let asset = row.asset_number.as_deref().unwrap_or("").trim().to_ascii_lowercase();
    let serial = row.serial_number.as_deref().unwrap_or("").trim().to_ascii_lowercase();
    if !asset.is_empty() {
        let asset_matches: Vec<_> = row
            .candidate_entries
            .iter()
            .filter(|c| c.asset_number.trim().to_ascii_lowercase() == asset)
            .collect();
        if asset_matches.len() == 1 {
            return Some(asset_matches[0].entry_uuid.clone());
        }
        if asset_matches.len() > 1 && !serial.is_empty() {
            if let Some(hit) = asset_matches.iter().find(|c| {
                c.serial_number.trim().to_ascii_lowercase() == serial
            }) {
                return Some(hit.entry_uuid.clone());
            }
            // Prefer the cleaner-looking serial (no leading +) among asset matches.
            if let Some(hit) = asset_matches.iter().find(|c| {
                let s = c.serial_number.trim();
                !s.is_empty() && !s.starts_with('+')
            }) {
                return Some(hit.entry_uuid.clone());
            }
            return Some(asset_matches[0].entry_uuid.clone());
        }
    }
    if !serial.is_empty() {
        let serial_matches: Vec<_> = row
            .candidate_entries
            .iter()
            .filter(|c| c.serial_number.trim().to_ascii_lowercase() == serial)
            .collect();
        if serial_matches.len() == 1 {
            return Some(serial_matches[0].entry_uuid.clone());
        }
    }
    None
}

#[test]
fn clear_cutover_verification_attribution_when_requested() {
    if std::env::var("IM015_CLEAR_CUTOVER_VERIFIED_BY").ok().as_deref() != Some("1") {
        return;
    }
    let db_path = PathBuf::from(
        std::env::var("TE_CALIBRATION_ROSTER_DB_COPY")
            .expect("TE_CALIBRATION_ROSTER_DB_COPY must point to the target database file"),
    );
    let file_size = fs::metadata(&db_path).unwrap().len();
    let db = InventoryDb::open_at_with_size(db_path, file_size).unwrap();
    let marker = "Calibration tracking workbook cutover";
    let mut cleared = 0usize;
    for mut entry in db.load_entries().unwrap() {
        let Some(verified_by) = entry.verified_by.as_ref() else {
            continue;
        };
        if !verified_by.contains(marker) {
            continue;
        }
        entry.verified_by = None;
        db.put_entry(&entry).unwrap();
        cleared += 1;
    }
    db.flush();
    println!("cleared verified_by on {cleared} entries");
    assert!(cleared > 0, "expected to clear at least one cutover attribution");
}
