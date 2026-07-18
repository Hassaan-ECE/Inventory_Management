// SYNTHETIC IMPORT CONTRACT TESTS ONLY. No lab inventory data belongs in this file.
#![allow(dead_code)]

#[path = "../src/domain/entry_changes.rs"]
pub(crate) mod entry_changes_impl;
#[path = "../src/domain/model.rs"]
pub(crate) mod model;
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

#[path = "../src/integrations/inventory_import/mod.rs"]
pub(crate) mod inventory_import;

use std::{
    fs,
    path::{Path, PathBuf},
};

use inventory_import::{
    commit_import_from_store, commit_import_with_test_failure_after, preview_import_from_path,
    ImportClassification, ImportColumnTreatment, ImportCommitInput,
};
use model::{CalibrationRequirement, InventoryEntryInput};
use store::InventoryDb;
use uuid::Uuid;

const LIVE_EXPORT_HEADERS: [&str; 22] = [
    "Asset Number",
    "Serial Number",
    "Manufacturer",
    "Model",
    "Description",
    "Location",
    "Assigned To",
    "Lifecycle",
    "Working",
    "Condition",
    "Cal Status",
    "Last Cal Date",
    "Cal Due Date",
    "Cal Vendor",
    "Cal Cost",
    "Ownership",
    "Rental Vendor",
    "Rental Cost/Mo",
    "Verified",
    "Blue Dot",
    "Est. Age (Yrs)",
    "Notes",
];

const TIMELESS_VERIFICATION_ISSUE: &str =
    "Timeless verified flag ignored; re-verification required";

#[test]
fn csv_preview_accounts_for_every_column_and_row_and_maps_calibration() {
    let fixture = SyntheticFixture::csv(
        "insert.csv",
        "Asset Number,Serial Number,Manufacturer,Model,Description,Calibration Status,Calibration Due,Calibration Interval Months,Certificate,Calibration Vendor,Verified At,Verified By\n\
         0007,SN-7,Acme,M7,SYNTHETIC meter,calibrated,2027-07-13,12,CERT-SYNTH,Vendor X,2026-07-13T12:00:00Z,Operator X\n\
         0008,SN-8,Acme,M8,SYNTHETIC reference,reference_only,,,,,,\n\
         ,,,,,,,,,,,\n",
    );
    let db = test_db("csv-preview");

    let report = preview_import_from_path(&fixture.path, &db).unwrap();

    assert_eq!(report.selected_sheet, "CSV");
    assert_eq!(report.total_rows, 3);
    assert_eq!(report.inserted, 2);
    assert_eq!(report.ignored, 1);
    assert_eq!(
        report.total_rows,
        report.inserted + report.matched + report.conflicted + report.rejected + report.ignored
    );
    assert_eq!(report.columns.len(), 12);
    assert!(report
        .columns
        .iter()
        .all(|column| !column.original_header.is_empty()));
    assert!(report
        .columns
        .iter()
        .all(|column| !column.reason.is_empty()));
    assert_eq!(report.row_outcomes[0].source_row, 2);
    assert_eq!(
        report.row_outcomes[0].original_asset_number.as_deref(),
        Some("0007")
    );
    assert_eq!(
        report.row_outcomes[0].classification,
        ImportClassification::Inserted
    );
    assert_eq!(
        report.row_outcomes[2].classification,
        ImportClassification::Ignored
    );
    assert!(!report.batch_id.is_empty());
    assert!(!report.source_fingerprint.is_empty());
    assert!(!report.reconciliation_basis.is_empty());
    let wire = serde_json::to_value(&report).unwrap();
    assert_eq!(wire["sourceFilename"], "insert.csv");
    assert!(wire["columns"].as_array().unwrap().iter().all(|column| {
        matches!(
            column["treatment"].as_str(),
            Some("mapped" | "intentionally_ignored" | "unknown")
        )
    }));
}

#[test]
fn xlsx_preview_matches_csv_and_invalid_xls_is_an_explicit_error() {
    let root = unique_test_dir("xlsx-parity");
    fs::create_dir_all(&root).unwrap();
    let xlsx = root.join("SYNTHETIC.xlsx");
    let mut workbook = rust_xlsxwriter::Workbook::new();
    {
        let hidden = workbook.add_worksheet();
        hidden.set_name("Hidden staging").unwrap();
        hidden.write_string(0, 0, "Asset").unwrap();
        hidden.write_string(1, 0, "MUST-NOT-SELECT").unwrap();
        hidden.set_hidden(true);
    }
    {
        let sheet = workbook.add_worksheet();
        sheet.set_name("Inventory").unwrap();
        sheet.set_active(true);
        sheet.write_string(0, 0, "Asset Number").unwrap();
        sheet.write_string(0, 1, "Description").unwrap();
        sheet.write_string(0, 2, "Calibration Due").unwrap();
        sheet.write_string(1, 0, "0009").unwrap();
        sheet.write_string(1, 1, "SYNTHETIC scope").unwrap();
        sheet.write_string(1, 2, "2027-08-01").unwrap();
    }
    workbook.save(&xlsx).unwrap();
    let db = test_db("xlsx-preview");

    let report = preview_import_from_path(&xlsx, &db).unwrap();

    assert_eq!(report.selected_sheet, "Inventory");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.inserted, 1);
    assert_eq!(
        report.row_outcomes[0].original_asset_number.as_deref(),
        Some("0009")
    );

    let invalid_xls = root.join("SYNTHETIC-invalid.xls");
    fs::write(&invalid_xls, b"SYNTHETIC NOT AN XLS WORKBOOK").unwrap();
    let error = preview_import_from_path(&invalid_xls, &db).unwrap_err();
    assert!(error.contains(".xls"));
    assert!(error.to_lowercase().contains("workbook"));
}

#[test]
fn xlsx_prefers_inventory_case_insensitively_and_preserves_actual_sheet_name() {
    let root = unique_test_dir("inventory-sheet-preference");
    fs::create_dir_all(&root).unwrap();
    let xlsx = root.join("three-sheets.xlsx");
    let mut workbook = rust_xlsxwriter::Workbook::new();
    {
        let sheet = workbook.add_worksheet();
        write_xlsx_row(sheet, 0, &["Issue"]);
        write_xlsx_row(sheet, 1, &["SYNTHETIC supporting issue"]);
        sheet.set_name("Import Issues").unwrap();
    }
    {
        let sheet = workbook.add_worksheet();
        write_xlsx_row(sheet, 0, &["Asset", "Description"]);
        write_xlsx_row(sheet, 1, &["SHEET-1", "SYNTHETIC preferred inventory row"]);
        sheet.set_name("iNvEnToRy").unwrap();
    }
    {
        let sheet = workbook.add_worksheet();
        write_xlsx_row(sheet, 0, &["Metric", "Value"]);
        write_xlsx_row(sheet, 1, &["SYNTHETIC total", "1"]);
        sheet.set_name("Export Summary").unwrap();
    }
    workbook.save(&xlsx).unwrap();
    let db = test_db("inventory-sheet-preference");

    let report = preview_import_from_path(&xlsx, &db).unwrap();

    assert_eq!(report.selected_sheet, "iNvEnToRy");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.inserted, 1);
    let committed = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    assert_eq!(committed.inserted, 1);
    let entries = db.load_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0]
            .import_provenance
            .as_ref()
            .and_then(|provenance| provenance.source_sheet.as_deref()),
        Some("iNvEnToRy")
    );
}

#[test]
fn live_22_column_shape_maps_ignores_and_commits_idempotently() {
    let root = unique_test_dir("live-shape");
    fs::create_dir_all(&root).unwrap();
    let xlsx = root.join("live-shape.xlsx");
    write_live_shape_workbook(
        &xlsx,
        [
            "LIVE-22",
            "LIVE-SERIAL-22",
            "Synthetic Maker",
            "Synthetic Model",
            "SYNTHETIC calibrated item",
            "Synthetic Lab",
            "Synthetic Operator",
            "active",
            "working",
            "good",
            "calibrated",
            "2026-01-02",
            "2027-01-02",
            "Synthetic Cal Vendor",
            "125.00",
            "owned",
            "Synthetic Rental Vendor",
            "75.00",
            "true",
            "yes",
            "7",
            "SYNTHETIC notes",
        ],
    );
    let db = test_db("live-shape");

    let report = preview_import_from_path(&xlsx, &db).unwrap();

    assert_eq!(report.mapping_version, "te-test-equipment-v2");
    assert_eq!(report.selected_sheet, "Inventory");
    assert_eq!(report.total_rows, 1);
    assert_eq!(report.inserted, 1);
    assert!(!report.blocking);
    assert_eq!(
        report
            .columns
            .iter()
            .map(|column| column.original_header.as_str())
            .collect::<Vec<_>>(),
        LIVE_EXPORT_HEADERS.to_vec()
    );
    assert_eq!(
        report
            .columns
            .iter()
            .filter(|column| column.treatment == ImportColumnTreatment::Mapped)
            .count(),
        16
    );
    assert_eq!(
        report
            .columns
            .iter()
            .filter(|column| column.treatment == ImportColumnTreatment::IntentionallyIgnored)
            .count(),
        6
    );
    assert!(report
        .columns
        .iter()
        .all(|column| column.treatment != ImportColumnTreatment::Unknown));
    assert_eq!(
        report
            .columns
            .iter()
            .find(|column| column.original_header == "Cal Due Date")
            .and_then(|column| column.normalized_target.as_deref()),
        Some("calibration_due_at")
    );
    assert_eq!(
        report
            .columns
            .iter()
            .find(|column| column.original_header == "Cal Vendor")
            .and_then(|column| column.normalized_target.as_deref()),
        Some("calibration_vendor")
    );
    assert!(report.row_outcomes[0]
        .issues
        .iter()
        .any(|issue| issue == TIMELESS_VERIFICATION_ISSUE));

    let committed = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id.clone(),
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    assert_eq!(committed.inserted, 1);
    assert_eq!(outbox_count(&db), 1);
    let entries = db.load_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].calibration_requirement,
        CalibrationRequirement::Required
    );
    assert_eq!(entries[0].last_calibrated_at.as_deref(), Some("2026-01-02"));
    assert_eq!(entries[0].calibration_due_at.as_deref(), Some("2027-01-02"));
    assert_eq!(
        entries[0].calibration_vendor.as_deref(),
        Some("Synthetic Cal Vendor")
    );
    assert_eq!(entries[0].verified_at, None);
    assert_eq!(
        entries[0]
            .import_provenance
            .as_ref()
            .and_then(|provenance| provenance.source_sheet.as_deref()),
        Some("Inventory")
    );

    let repeated = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    assert_eq!(repeated.inserted, 0);
    assert_eq!(repeated.noop, 1);
    assert!(!repeated.entries_changed);
    assert_eq!(outbox_count(&db), 1);

    let matched_xlsx = root.join("live-shape-matched.xlsx");
    write_live_shape_workbook(
        &matched_xlsx,
        [
            "LIVE-22",
            "LIVE-SERIAL-22",
            "Synthetic Maker",
            "Synthetic Model",
            "SYNTHETIC matched no-op",
            "Synthetic Lab",
            "Synthetic Operator",
            "active",
            "working",
            "good",
            "calibrated",
            "2026-01-02",
            "2027-01-02",
            "Synthetic Cal Vendor",
            "125.00",
            "owned",
            "Synthetic Rental Vendor",
            "75.00",
            "false",
            "yes",
            "7",
            "SYNTHETIC notes",
        ],
    );
    let matched_report = preview_import_from_path(&matched_xlsx, &db).unwrap();
    assert_eq!(matched_report.matched, 1);
    assert!(!matched_report.blocking);
    let matched_commit = commit_import_from_store(
        ImportCommitInput {
            batch_id: matched_report.batch_id.clone(),
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    assert_eq!(matched_commit.matched, 1);
    assert_eq!(matched_commit.noop, 1);
    assert!(!matched_commit.entries_changed);
    let matched_repeat = commit_import_from_store(
        ImportCommitInput {
            batch_id: matched_report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    assert_eq!(matched_repeat.noop, 1);
    assert_eq!(db.load_entries().unwrap().len(), 1);
    assert_eq!(outbox_count(&db), 1);
}

#[test]
fn overwide_csv_and_unheaded_xlsx_cells_are_reported_and_rejected_without_data_loss() {
    let csv = SyntheticFixture::csv(
        "overwide.csv",
        "Asset,Description\nOVER-CSV,SYNTHETIC row,SYNTHETIC csv overflow\n",
    );
    let csv_db = test_db("overwide-csv");

    let csv_report = preview_import_from_path(&csv.path, &csv_db).unwrap();

    assert_eq!(csv_report.total_rows, 1);
    assert_eq!(csv_report.rejected, 1);
    assert!(csv_report.blocking);
    assert_eq!(
        csv_report.total_rows,
        csv_report.inserted
            + csv_report.matched
            + csv_report.conflicted
            + csv_report.rejected
            + csv_report.ignored
    );
    assert!(csv_report
        .columns
        .iter()
        .any(|column| column.original_header == "__unheaded_column_3"));
    assert_eq!(
        csv_report.row_outcomes[0].raw_values["__unheaded_column_3"],
        "SYNTHETIC csv overflow"
    );
    assert!(csv_report.row_outcomes[0]
        .issues
        .iter()
        .any(|issue| issue.contains("declared header width") && issue.contains("column 3")));

    let root = unique_test_dir("unheaded-xlsx");
    fs::create_dir_all(&root).unwrap();
    let xlsx = root.join("unheaded.xlsx");
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("Inventory").unwrap();
    sheet.write_string(0, 0, "Asset").unwrap();
    sheet.write_string(0, 1, "Description").unwrap();
    sheet.write_string(1, 0, "OVER-XLSX").unwrap();
    sheet.write_string(1, 1, "SYNTHETIC row").unwrap();
    sheet.write_string(1, 2, "SYNTHETIC xlsx overflow").unwrap();
    workbook.save(&xlsx).unwrap();
    let xlsx_db = test_db("unheaded-xlsx");

    let xlsx_report = preview_import_from_path(&xlsx, &xlsx_db).unwrap();

    assert_eq!(xlsx_report.total_rows, 1);
    assert_eq!(xlsx_report.rejected, 1);
    assert!(xlsx_report.blocking);
    assert_eq!(
        xlsx_report.total_rows,
        xlsx_report.inserted
            + xlsx_report.matched
            + xlsx_report.conflicted
            + xlsx_report.rejected
            + xlsx_report.ignored
    );
    assert!(xlsx_report
        .columns
        .iter()
        .any(|column| column.original_header == "__unheaded_column_3"));
    assert_eq!(
        xlsx_report.row_outcomes[0].raw_values["__unheaded_column_3"],
        "SYNTHETIC xlsx overflow"
    );
    assert!(xlsx_report.row_outcomes[0]
        .issues
        .iter()
        .any(|issue| issue.contains("unheaded") && issue.contains("column 3")));
}

#[test]
fn multiple_visible_usable_workbook_sheets_without_inventory_are_rejected_as_ambiguous() {
    let root = unique_test_dir("ambiguous-workbook");
    fs::create_dir_all(&root).unwrap();
    let xlsx = root.join("ambiguous.xlsx");
    let mut workbook = rust_xlsxwriter::Workbook::new();
    for sheet_name in ["Primary", "Secondary"] {
        let sheet = workbook.add_worksheet();
        sheet.set_name(sheet_name).unwrap();
        sheet.write_string(0, 0, "Asset").unwrap();
        sheet
            .write_string(1, 0, format!("SYNTHETIC-{sheet_name}"))
            .unwrap();
    }
    workbook.save(&xlsx).unwrap();
    let db = test_db("ambiguous-workbook");

    let error = preview_import_from_path(&xlsx, &db).unwrap_err();

    assert!(error
        .to_lowercase()
        .contains("multiple visible usable sheets"));
    assert!(error.contains("Primary"));
    assert!(error.contains("Secondary"));
    assert!(db.load_entries().unwrap().is_empty());
}

#[test]
fn batch_identity_is_stable_for_same_content_sheet_and_mapping_after_filename_rename() {
    let first = SyntheticFixture::csv(
        "first-name.csv",
        "Asset,Description\nSTABLE-1,SYNTHETIC stable batch\n",
    );
    let renamed = first.path.with_file_name("renamed-source.csv");
    fs::copy(&first.path, &renamed).unwrap();
    let db = test_db("stable-batch");

    let first_report = preview_import_from_path(&first.path, &db).unwrap();
    let renamed_report = preview_import_from_path(&renamed, &db).unwrap();

    assert_eq!(first_report.batch_id, renamed_report.batch_id);
    assert_eq!(
        first_report.source_fingerprint,
        renamed_report.source_fingerprint
    );
    assert_eq!(first_report.selected_sheet, renamed_report.selected_sheet);
    assert_ne!(first_report.source_filename, renamed_report.source_filename);
}

#[test]
fn csv_true_blank_physical_row_is_counted_and_intentional_ignore_is_reported() {
    let fixture = SyntheticFixture::csv(
        "physical-rows.csv",
        "Asset,Description,Age\nPHYS-1,SYNTHETIC first,7\n\nPHYS-2,SYNTHETIC second,8\nPHYS-3,\"SYNTHETIC line one\n\nline three\",9\n",
    );
    let db = test_db("physical-rows");

    let report = preview_import_from_path(&fixture.path, &db).unwrap();

    assert_eq!(report.total_rows, 4);
    assert_eq!(report.inserted, 3);
    assert_eq!(report.ignored, 1);
    assert_eq!(report.row_outcomes[1].source_row, 3);
    assert_eq!(
        report.row_outcomes[1].classification,
        ImportClassification::Ignored
    );
    let age = report
        .columns
        .iter()
        .find(|column| column.original_header == "Age")
        .unwrap();
    assert_eq!(
        age.treatment,
        inventory_import::ImportColumnTreatment::IntentionallyIgnored
    );
    assert_eq!(age.nonblank_count, 3);
}

#[test]
fn reconciliation_uses_only_unique_asset_or_serial_and_detects_disagreement_and_duplicates() {
    let db = test_db("matching");
    let asset_entry = api::mutations::create_entry_in_store(
        input("A-1", "S-1", "Acme", "Shared", "SYNTHETIC asset"),
        &db,
    )
    .unwrap()
    .entry;
    let serial_entry = api::mutations::create_entry_in_store(
        input("A-2", "S-2", "Acme", "Shared", "SYNTHETIC serial"),
        &db,
    )
    .unwrap()
    .entry;
    let _duplicate = api::mutations::create_entry_in_store(
        input("A-2", "S-3", "Acme", "Shared", "SYNTHETIC duplicate asset"),
        &db,
    )
    .unwrap();

    let asset_only = SyntheticFixture::csv("asset.csv", "Asset,Description\nA-1,SYNTHETIC match\n");
    let report = preview_import_from_path(&asset_only.path, &db).unwrap();
    assert_eq!(report.matched, 1);
    assert_eq!(
        report.row_outcomes[0].candidate_entry_uuid.as_deref(),
        Some(asset_entry.entry_uuid.as_str())
    );

    let serial_only = SyntheticFixture::csv(
        "serial.csv",
        "Serial,Description\nS-2,SYNTHETIC serial match\n",
    );
    let report = preview_import_from_path(&serial_only.path, &db).unwrap();
    assert_eq!(report.matched, 1);
    assert_eq!(
        report.row_outcomes[0].candidate_entry_uuid.as_deref(),
        Some(serial_entry.entry_uuid.as_str())
    );

    api::mutations::create_entry_in_store(
        input("A-3", "S-2", "Acme", "Shared", "SYNTHETIC duplicate serial"),
        &db,
    )
    .unwrap();
    let duplicate_serial = SyntheticFixture::csv(
        "duplicate-serial.csv",
        "Serial,Description\nS-2,SYNTHETIC duplicate serial\n",
    );
    assert_eq!(
        preview_import_from_path(&duplicate_serial.path, &db)
            .unwrap()
            .conflicted,
        1
    );

    let both_same = SyntheticFixture::csv(
        "both-same.csv",
        "Asset,Serial,Description\nA-1,S-1,SYNTHETIC same\n",
    );
    assert_eq!(
        preview_import_from_path(&both_same.path, &db)
            .unwrap()
            .matched,
        1
    );

    let disagreement = SyntheticFixture::csv(
        "disagree.csv",
        "Asset,Serial,Description\nA-1,S-2,SYNTHETIC disagreement\n",
    );
    assert_eq!(
        preview_import_from_path(&disagreement.path, &db)
            .unwrap()
            .conflicted,
        1
    );

    let duplicate = SyntheticFixture::csv(
        "duplicate.csv",
        "Asset,Description\nA-2,SYNTHETIC duplicate\n",
    );
    assert_eq!(
        preview_import_from_path(&duplicate.path, &db)
            .unwrap()
            .conflicted,
        1
    );

    let maker_model = SyntheticFixture::csv(
        "maker-model.csv",
        "Manufacturer,Model,Description\nAcme,Shared,SYNTHETIC no auto match\n",
    );
    assert_eq!(
        preview_import_from_path(&maker_model.path, &db)
            .unwrap()
            .inserted,
        1
    );
}

#[test]
fn preview_reconciles_all_five_row_classifications_in_one_batch() {
    let db = test_db("five-way-reconciliation");
    api::mutations::create_entry_in_store(
        input(
            "MATCH-FIVE",
            "MATCH-FIVE-SERIAL",
            "Synthetic Maker",
            "M1",
            "SYNTHETIC match candidate",
        ),
        &db,
    )
    .unwrap();
    for serial in ["DUP-FIVE-1", "DUP-FIVE-2"] {
        api::mutations::create_entry_in_store(
            input(
                "DUP-FIVE",
                serial,
                "Synthetic Maker",
                "M2",
                "SYNTHETIC duplicate candidate",
            ),
            &db,
        )
        .unwrap();
    }
    let fixture = SyntheticFixture::csv(
        "five-way.csv",
        "Asset,Serial,Description,Calibration Due\n\
         INSERT-FIVE,INSERT-FIVE-SERIAL,SYNTHETIC inserted,2027-01-01\n\
         MATCH-FIVE,,SYNTHETIC matched,\n\
         DUP-FIVE,,SYNTHETIC conflicted,\n\
         REJECT-FIVE,REJECT-FIVE-SERIAL,SYNTHETIC rejected,not-a-date\n\
         ,,,\n",
    );

    let report = preview_import_from_path(&fixture.path, &db).unwrap();

    assert_eq!(report.total_rows, 5);
    assert_eq!(report.inserted, 1);
    assert_eq!(report.matched, 1);
    assert_eq!(report.conflicted, 1);
    assert_eq!(report.rejected, 1);
    assert_eq!(report.ignored, 1);
    assert_eq!(
        report.total_rows,
        report.inserted + report.matched + report.conflicted + report.rejected + report.ignored
    );
    assert_eq!(
        report
            .row_outcomes
            .iter()
            .map(|row| row.classification)
            .collect::<Vec<_>>(),
        vec![
            ImportClassification::Inserted,
            ImportClassification::Matched,
            ImportClassification::Conflicted,
            ImportClassification::Rejected,
            ImportClassification::Ignored,
        ]
    );
    assert!(report.blocking);
}

#[test]
fn source_batch_identity_collisions_conflict_every_participant_and_block_commit() {
    let fixture = SyntheticFixture::csv(
        "source-collisions.csv",
        "Asset,Serial,Manufacturer,Model,Description\n\
         DUP-A,AS-1,Maker,Shared,SYNTHETIC duplicate asset one\n\
          dup-a ,AS-2,Maker,Shared,SYNTHETIC duplicate asset two\n\
         BS-1,DUP-S,Maker,Shared,SYNTHETIC duplicate serial one\n\
         BS-2, dup-s ,Maker,Shared,SYNTHETIC duplicate serial two\n\
         GRAPH-A,GRAPH-S1,Maker,Shared,SYNTHETIC graph one\n\
         GRAPH-B,GRAPH-S2,Maker,Shared,SYNTHETIC graph two\n\
          graph-a , graph-s2 ,Maker,Shared,SYNTHETIC graph bridge\n\
         UNIQUE-1,UNIQUE-S1,Same Maker,Same Model,SYNTHETIC unique one\n\
         UNIQUE-2,UNIQUE-S2,Same Maker,Same Model,SYNTHETIC unique two\n",
    );
    let db = test_db("source-collisions");

    let report = preview_import_from_path(&fixture.path, &db).unwrap();

    assert_eq!(report.total_rows, 9);
    assert_eq!(report.conflicted, 7);
    assert_eq!(report.inserted, 2);
    assert!(report.blocking);
    assert_eq!(
        report.total_rows,
        report.inserted + report.matched + report.conflicted + report.rejected + report.ignored
    );
    assert!(report.row_outcomes[..7]
        .iter()
        .all(|row| row.classification == ImportClassification::Conflicted));
    assert!(report.row_outcomes[..7].iter().all(|row| row
        .issues
        .iter()
        .any(|issue| issue.to_lowercase().contains("source batch"))));
    assert!(report.row_outcomes[7..]
        .iter()
        .all(|row| row.classification == ImportClassification::Inserted));

    let error = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap_err();
    assert!(error.to_lowercase().contains("block"));
    assert!(db.load_entries().unwrap().is_empty());
    assert_eq!(outbox_count(&db), 0);
}

#[test]
fn preview_rejects_unsafe_values_without_silently_dropping_unknown_data() {
    let fixture = SyntheticFixture::csv(
        "unsafe.csv",
        "Asset,Description,Calibration Status,Calibration Requirement,Out To Cal,Last Calibrated,Calibration Due,Archived,Verified,Unknown Column\n\
         A-10,SYNTHETIC ambiguous,calibrated,required,false,01/02/2026,2027-01-01,false,,\n\
         A-11,SYNTHETIC impossible,calibrated,required,false,2027-02-01,2027-01-01,false,,\n\
         A-12,SYNTHETIC contradiction,out_to_cal,reference_only,false,2026-01-01,2027-01-01,false,,\n\
         A-13,SYNTHETIC bool,unknown,unknown,false,,,maybe,,\n\
         A-14,SYNTHETIC timeless,unknown,unknown,false,,,false,true,\n\
         A-15,SYNTHETIC unknown,unknown,unknown,false,,,false,,must-not-drop\n",
    );
    let db = test_db("unsafe");

    let report = preview_import_from_path(&fixture.path, &db).unwrap();

    assert_eq!(report.total_rows, 6);
    assert_eq!(report.inserted, 1);
    assert_eq!(report.rejected, 5);
    assert!(report.blocking);
    assert!(report.row_outcomes.iter().all(|row| !row.issues.is_empty()));
    assert_eq!(
        report.row_outcomes[4].classification,
        ImportClassification::Inserted
    );
    assert!(report.row_outcomes[4]
        .issues
        .iter()
        .any(|issue| issue == TIMELESS_VERIFICATION_ISSUE));
    let unknown = report
        .columns
        .iter()
        .find(|column| column.original_header == "Unknown Column")
        .unwrap();
    assert_eq!(unknown.nonblank_count, 1);
    assert!(report.row_outcomes[5]
        .raw_values
        .values()
        .any(|value| value == "must-not-drop"));
}

#[test]
fn duplicate_source_headers_retain_each_raw_cell_with_distinct_context() {
    let fixture = SyntheticFixture::csv(
        "duplicate-headers.csv",
        "Asset,Description,Extra,Extra\nDUP-H,SYNTHETIC duplicate header,first,second\n",
    );
    let db = test_db("duplicate-headers");

    let report = preview_import_from_path(&fixture.path, &db).unwrap();

    assert_eq!(report.columns.len(), 4);
    assert_eq!(report.rejected, 1);
    assert_eq!(
        report.row_outcomes[0].raw_values["Extra [column 3]"],
        "first"
    );
    assert_eq!(
        report.row_outcomes[0].raw_values["Extra [column 4]"],
        "second"
    );
}

#[test]
fn calibration_legacy_mappings_and_explicit_due_interval_are_preserved_on_commit() {
    let fixture = SyntheticFixture::csv(
        "calibration.csv",
        "Asset,Description,Calibration Status,Calibration Due,Calibration Interval Months\n\
         CAL-1,SYNTHETIC calibrated,calibrated,2027-01-01,12\n\
         CAL-2,SYNTHETIC out,out_to_cal,,\n\
         CAL-3,SYNTHETIC ref,reference_only,,\n\
         CAL-4,SYNTHETIC none,not_required,,\n\
         CAL-5,SYNTHETIC unknown,unknown,,\n\
         CAL-6,SYNTHETIC calibrated missing due,calibrated,,\n",
    );
    let db = test_db("calibration");
    let report = preview_import_from_path(&fixture.path, &db).unwrap();
    assert_eq!(report.inserted, 6);

    let result = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    assert_eq!(result.inserted, 6);
    let mut entries = db.load_entries().unwrap();
    entries.sort_by(|left, right| left.asset_number.cmp(&right.asset_number));
    assert_eq!(
        entries[0].calibration_requirement,
        CalibrationRequirement::Required
    );
    assert!(!entries[0].out_to_calibration);
    assert_eq!(entries[0].calibration_due_at.as_deref(), Some("2027-01-01"));
    assert_eq!(entries[0].calibration_interval_months, Some(12));
    assert_eq!(
        entries[1].calibration_requirement,
        CalibrationRequirement::Required
    );
    assert!(entries[1].out_to_calibration);
    assert_eq!(
        entries[2].calibration_requirement,
        CalibrationRequirement::ReferenceOnly
    );
    assert_eq!(
        entries[3].calibration_requirement,
        CalibrationRequirement::NotRequired
    );
    assert_eq!(
        entries[4].calibration_requirement,
        CalibrationRequirement::Unknown
    );
    assert_eq!(
        entries[5].calibration_requirement,
        CalibrationRequirement::Required
    );
    assert_eq!(entries[5].calibration_due_at, None);
}

#[test]
fn snake_case_live_calibration_aliases_map_due_date_and_vendor() {
    let fixture = SyntheticFixture::csv(
        "snake-case-live-aliases.csv",
        "Asset,Description,cal_status,last_cal_date,cal_due_date,cal_vendor\n\
         SNAKE-1,SYNTHETIC snake aliases,calibrated,2026-02-03,2027-02-03,Synthetic Vendor\n",
    );
    let db = test_db("snake-case-live-aliases");

    let report = preview_import_from_path(&fixture.path, &db).unwrap();

    assert_eq!(report.inserted, 1);
    assert!(!report.blocking);
    assert_eq!(
        report
            .columns
            .iter()
            .find(|column| column.original_header == "cal_due_date")
            .and_then(|column| column.normalized_target.as_deref()),
        Some("calibration_due_at")
    );
    assert_eq!(
        report
            .columns
            .iter()
            .find(|column| column.original_header == "cal_vendor")
            .and_then(|column| column.normalized_target.as_deref()),
        Some("calibration_vendor")
    );
    commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    let entries = db.load_entries().unwrap();
    assert_eq!(entries[0].last_calibrated_at.as_deref(), Some("2026-02-03"));
    assert_eq!(entries[0].calibration_due_at.as_deref(), Some("2027-02-03"));
    assert_eq!(
        entries[0].calibration_vendor.as_deref(),
        Some("Synthetic Vendor")
    );
}

#[test]
fn invalid_verification_boolean_and_timestamp_remain_rejected() {
    let fixture = SyntheticFixture::csv(
        "invalid-verification.csv",
        "Asset,Description,Verified,Verified At\n\
         VERIFY-1,SYNTHETIC invalid flag,maybe,\n\
         VERIFY-2,SYNTHETIC invalid timestamp,true,not-a-timestamp\n",
    );
    let db = test_db("invalid-verification");

    let report = preview_import_from_path(&fixture.path, &db).unwrap();

    assert_eq!(report.total_rows, 2);
    assert_eq!(report.rejected, 2);
    assert!(report.blocking);
    assert!(report.row_outcomes[0]
        .issues
        .iter()
        .any(|issue| issue.contains("Unrecognized verified boolean")));
    assert!(report.row_outcomes[1]
        .issues
        .iter()
        .any(|issue| issue == "Verified at must be a valid RFC 3339 timestamp."));
}

#[test]
fn matched_only_commit_is_a_durable_noop_with_no_entry_or_outbox_change() {
    let db = test_db("matched-only");
    let existing = api::mutations::create_entry_in_store(
        input(
            "MATCH-1",
            "MATCH-SERIAL",
            "Acme",
            "M1",
            "SYNTHETIC existing",
        ),
        &db,
    )
    .unwrap();
    let entries_before = db.load_entries().unwrap().len();
    let outbox_before = outbox_count(&db);
    let fixture = SyntheticFixture::csv(
        "matched-only.csv",
        "Asset,Description\nMATCH-1,SYNTHETIC incoming no-op\n",
    );
    let report = preview_import_from_path(&fixture.path, &db).unwrap();
    assert_eq!(report.matched, 1);

    let committed = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id.clone(),
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    assert_eq!(committed.inserted, 0);
    assert_eq!(committed.matched, 1);
    assert_eq!(committed.noop, 1);
    assert!(!committed.entries_changed);
    assert_eq!(db.load_entries().unwrap().len(), entries_before);
    assert_eq!(outbox_count(&db), outbox_before);
    assert_eq!(
        db.load_entries().unwrap()[0].entry_uuid,
        existing.entry.entry_uuid
    );

    let repeated = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    assert_eq!(repeated.inserted, 0);
    assert_eq!(repeated.noop, 1);
    assert!(!repeated.entries_changed);
    assert_eq!(outbox_count(&db), outbox_before);
}

#[test]
fn commit_is_confirmed_blocking_stale_and_idempotent_with_provenance_and_one_outbox_per_insert() {
    let fixture = SyntheticFixture::csv(
        "guarded.csv",
        "ID,Asset,Serial,Description\nlegacy-1,NEW-1,NS-1,SYNTHETIC new one\nlegacy-2,NEW-2,NS-2,SYNTHETIC new two\n",
    );
    let db = test_db("commit");
    let report = preview_import_from_path(&fixture.path, &db).unwrap();

    let error = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id.clone(),
            confirmed: false,
            allow_partial: false,
        },
        &db,
    )
    .unwrap_err();
    assert!(error.to_lowercase().contains("confirm"));

    let result = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id.clone(),
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    assert_eq!(result.inserted, 2);
    assert_eq!(result.remaining, 0);
    assert!(result.entries_changed);
    assert_eq!(result.noop, 0);
    assert_eq!(outbox_count(&db), 2);
    for entry in db.load_entries().unwrap() {
        let provenance = entry.import_provenance.unwrap();
        assert_eq!(provenance.batch_id, report.batch_id);
        assert_eq!(provenance.source_filename, "guarded.csv");
        assert!(!provenance.source_filename.contains('\\'));
        assert!(provenance.source_row == 2 || provenance.source_row == 3);
        assert!(matches!(
            provenance.original_id.as_deref(),
            Some("legacy-1" | "legacy-2")
        ));
        assert!(matches!(
            provenance.original_asset_number.as_deref(),
            Some("NEW-1" | "NEW-2")
        ));
        assert!(matches!(
            provenance.original_serial_number.as_deref(),
            Some("NS-1" | "NS-2")
        ));
        assert!(entry.entry_uuid.len() >= 32);
    }

    let repeated = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    assert_eq!(repeated.noop, 2);
    assert_eq!(repeated.inserted, 0);
    assert!(!repeated.entries_changed);
    assert_eq!(outbox_count(&db), 2);
}

#[test]
fn commit_detects_stale_source_and_reconciliation_basis_and_blocks_bad_preview() {
    let db = test_db("stale");
    let source = SyntheticFixture::csv(
        "stale-source.csv",
        "Asset,Description\nSTALE-1,SYNTHETIC original\n",
    );
    let report = preview_import_from_path(&source.path, &db).unwrap();
    fs::write(
        &source.path,
        "Asset,Description\nSTALE-1,SYNTHETIC changed\n",
    )
    .unwrap();
    let error = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap_err();
    assert!(error.to_lowercase().contains("source"));

    let basis = SyntheticFixture::csv(
        "stale-basis.csv",
        "Asset,Description\nBASIS-1,SYNTHETIC basis\n",
    );
    let report = preview_import_from_path(&basis.path, &db).unwrap();
    api::mutations::create_entry_in_store(input("OTHER", "", "", "", "SYNTHETIC DB change"), &db)
        .unwrap();
    let error = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap_err();
    assert!(error.to_lowercase().contains("reconciliation"));

    let blocked = SyntheticFixture::csv(
        "blocked.csv",
        "Asset,Description,Calibration Due\nBAD-1,SYNTHETIC bad,not-a-date\n",
    );
    let report = preview_import_from_path(&blocked.path, &db).unwrap();
    let error = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap_err();
    assert!(error.to_lowercase().contains("block"));
}

#[test]
fn partial_failure_rerun_completes_exact_remainder_without_duplicate_entry_or_outbox() {
    let fixture = SyntheticFixture::csv(
        "partial.csv",
        "Asset,Description\nPART-1,SYNTHETIC first\nPART-2,SYNTHETIC second\nPART-3,SYNTHETIC third\n",
    );
    let db = test_db("partial");
    let report = preview_import_from_path(&fixture.path, &db).unwrap();
    let input = ImportCommitInput {
        batch_id: report.batch_id.clone(),
        confirmed: true,
        allow_partial: false,
    };

    let error = commit_import_with_test_failure_after(input.clone(), &db, 1).unwrap_err();
    assert!(error.contains("after entry and outbox mutation but before row marker"));
    assert_eq!(db.load_entries().unwrap().len(), 1);
    assert_eq!(outbox_count(&db), 1);
    assert!(!db.has_import_row_marker(&report.batch_id, 2).unwrap());

    let result = commit_import_from_store(input, &db).unwrap();
    assert_eq!(result.inserted, 2);
    assert_eq!(result.remaining, 0);
    assert_eq!(db.load_entries().unwrap().len(), 3);
    assert_eq!(outbox_count(&db), 3);

    let noop = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id,
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    assert_eq!(noop.noop, 3);
    assert_eq!(outbox_count(&db), 3);
}

#[test]
fn live_workbook_dry_run_aggregate_snapshot_if_present() {
    let Some(project_root) = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(Path::to_path_buf)
    else {
        panic!("backend manifest must have a project parent");
    };
    let workbook = project_root.join("data/import/TE_Lab_Equipment_Export.xlsx");
    if !workbook.is_file() {
        println!("live importer aggregate dry run skipped: source absent");
        return;
    }
    let db = test_db("live-workbook-aggregate");

    let report = preview_import_from_path(&workbook, &db).unwrap();

    println!(
        "live importer aggregate: total={} inserted={} matched={} conflicted={} rejected={} ignored={} blocking={}",
        report.total_rows,
        report.inserted,
        report.matched,
        report.conflicted,
        report.rejected,
        report.ignored,
        report.blocking
    );
    assert_eq!(report.selected_sheet, "Inventory");
    assert_eq!(report.mapping_version, "te-test-equipment-v2");
    assert_eq!(report.columns.len(), LIVE_EXPORT_HEADERS.len());
    let expected_columns = [
        (
            "Asset Number",
            Some("asset_number"),
            ImportColumnTreatment::Mapped,
        ),
        (
            "Serial Number",
            Some("serial_number"),
            ImportColumnTreatment::Mapped,
        ),
        (
            "Manufacturer",
            Some("manufacturer"),
            ImportColumnTreatment::Mapped,
        ),
        ("Model", Some("model"), ImportColumnTreatment::Mapped),
        (
            "Description",
            Some("description"),
            ImportColumnTreatment::Mapped,
        ),
        ("Location", Some("location"), ImportColumnTreatment::Mapped),
        (
            "Assigned To",
            Some("assigned_to"),
            ImportColumnTreatment::Mapped,
        ),
        (
            "Lifecycle",
            Some("lifecycle_status"),
            ImportColumnTreatment::Mapped,
        ),
        (
            "Working",
            Some("working_status"),
            ImportColumnTreatment::Mapped,
        ),
        (
            "Condition",
            Some("condition"),
            ImportColumnTreatment::Mapped,
        ),
        (
            "Cal Status",
            Some("calibration_status"),
            ImportColumnTreatment::Mapped,
        ),
        (
            "Last Cal Date",
            Some("last_calibrated_at"),
            ImportColumnTreatment::Mapped,
        ),
        (
            "Cal Due Date",
            Some("calibration_due_at"),
            ImportColumnTreatment::Mapped,
        ),
        (
            "Cal Vendor",
            Some("calibration_vendor"),
            ImportColumnTreatment::Mapped,
        ),
        (
            "Cal Cost",
            None,
            ImportColumnTreatment::IntentionallyIgnored,
        ),
        (
            "Ownership",
            None,
            ImportColumnTreatment::IntentionallyIgnored,
        ),
        (
            "Rental Vendor",
            None,
            ImportColumnTreatment::IntentionallyIgnored,
        ),
        (
            "Rental Cost/Mo",
            None,
            ImportColumnTreatment::IntentionallyIgnored,
        ),
        ("Verified", Some("verified"), ImportColumnTreatment::Mapped),
        (
            "Blue Dot",
            None,
            ImportColumnTreatment::IntentionallyIgnored,
        ),
        (
            "Est. Age (Yrs)",
            None,
            ImportColumnTreatment::IntentionallyIgnored,
        ),
        ("Notes", Some("notes"), ImportColumnTreatment::Mapped),
    ];
    for (header, target, treatment) in expected_columns {
        let column = report
            .columns
            .iter()
            .find(|column| column.original_header == header)
            .unwrap_or_else(|| panic!("missing aggregate column treatment for {header}"));
        assert_eq!(column.normalized_target.as_deref(), target);
        assert_eq!(column.treatment, treatment);
    }
    assert_eq!(
        (
            report.total_rows,
            report.inserted,
            report.matched,
            report.conflicted,
            report.rejected,
            report.ignored,
        ),
        (573, 515, 0, 50, 8, 0)
    );
    assert!(report.blocking);
    assert_eq!(
        report.total_rows,
        report.inserted + report.matched + report.conflicted + report.rejected + report.ignored
    );
}

#[test]
fn partial_commit_imports_clean_rows_and_skips_blocking() {
    let fixture = SyntheticFixture::csv(
        "partial.csv",
        "Asset Number,Serial Number,Manufacturer,Model,Description,Calibration Status\n\
         PART-OK,SN-OK,Acme,M1,SYNTHETIC clean,calibrated\n\
         PART-DUP,SN-A,Acme,M2,SYNTHETIC conflict A,calibrated\n\
         PART-DUP,SN-B,Acme,M3,SYNTHETIC conflict B,calibrated\n\
         BAD,,, , ,not_a_status\n",
    );
    let db = test_db("partial-commit");
    let report = preview_import_from_path(&fixture.path, &db).unwrap();
    assert!(report.blocking);
    assert!(report.inserted >= 1);
    assert!(report.conflicted + report.rejected >= 1);

    let full_err = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id.clone(),
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap_err();
    let full_error = full_err.to_ascii_lowercase();
    assert!(full_error.contains("blocking"));
    assert!(full_error.contains("correct"));
    assert!(!full_error.contains("partial"));

    let result = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id.clone(),
            confirmed: true,
            allow_partial: true,
        },
        &db,
    )
    .unwrap();
    assert_eq!(result.inserted, report.inserted);
    assert!(result.entries_changed);
    assert!(result.message.to_ascii_lowercase().contains("partial"));
    assert_eq!(db.load_entries().unwrap().len(), report.inserted);
}

/// One-shot: set TE_LOAD_LIVE_LOCAL=1 to write a fully clean live export into the real
/// LocalAppData FeOxDB. Close the desktop app first if the DB file is locked.
#[test]
fn load_live_full_batch_into_user_local_appdata_when_requested() {
    if std::env::var("TE_LOAD_LIVE_LOCAL").ok().as_deref() != Some("1") {
        return;
    }
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("project parent")
        .to_path_buf();
    let workbook = project_root.join("data/import/TE_Lab_Equipment_Export.xlsx");
    assert!(
        workbook.is_file(),
        "live workbook missing at {}",
        workbook.display()
    );

    let local_app = PathBuf::from(std::env::var("LOCALAPPDATA").expect("LOCALAPPDATA"))
        .join("com.te.test.equipment.inventory");
    fs::create_dir_all(&local_app).unwrap();
    let db_path = local_app.join("inventory.feox");
    let db = InventoryDb::open_at_with_size(db_path.clone(), 64 * 1024 * 1024)
        .unwrap_or_else(|error| {
            panic!(
                "Could not open {}. Close bun run desktop / the app first, then retry. Error: {error}",
                db_path.display()
            )
        });

    let before = db.load_entries().unwrap().len();
    let report = preview_import_from_path(&workbook, &db).unwrap();
    println!(
        "live load preview: total={} inserted={} matched={} conflicted={} rejected={} ignored={} blocking={} existing_entries={}",
        report.total_rows,
        report.inserted,
        report.matched,
        report.conflicted,
        report.rejected,
        report.ignored,
        report.blocking,
        before
    );
    assert!(report.inserted > 0, "expected clean insertable rows");
    assert!(
        !report.blocking,
        "live cutover is full-batch-only; correct every conflict/rejection before commit"
    );

    let result = commit_import_from_store(
        ImportCommitInput {
            batch_id: report.batch_id.clone(),
            confirmed: true,
            allow_partial: false,
        },
        &db,
    )
    .unwrap();
    db.flush();
    let after = db.load_entries().unwrap().len();
    println!(
        "live load result: inserted={} message={} entries_before={} entries_after={}",
        result.inserted, result.message, before, after
    );
    assert!(result.entries_changed || result.inserted == 0);
    assert_eq!(after, before + result.inserted);
}

fn input(
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
    std::env::temp_dir().join(format!("te-import-{prefix}-{}", Uuid::new_v4().simple()))
}

fn write_xlsx_row(sheet: &mut rust_xlsxwriter::Worksheet, row: u32, values: &[&str]) {
    for (column, value) in values.iter().enumerate() {
        sheet.write_string(row, column as u16, *value).unwrap();
    }
}

fn write_live_shape_workbook(path: &Path, values: [&str; 22]) {
    let mut workbook = rust_xlsxwriter::Workbook::new();
    let sheet = workbook.add_worksheet();
    sheet.set_name("Inventory").unwrap();
    write_xlsx_row(sheet, 0, &LIVE_EXPORT_HEADERS);
    write_xlsx_row(sheet, 1, &values);
    workbook.save(path).unwrap();
}

struct SyntheticFixture {
    path: PathBuf,
}

impl SyntheticFixture {
    fn csv(name: &str, contents: &str) -> Self {
        let root = unique_test_dir("fixture");
        fs::create_dir_all(&root).unwrap();
        let path = root.join(name);
        fs::write(&path, contents).unwrap();
        Self { path }
    }
}
