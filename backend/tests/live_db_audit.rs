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

use std::{collections::BTreeMap, path::PathBuf};

use calamine::{open_workbook_auto, Reader};
use model::CalibrationRequirement;
use store::InventoryDb;

#[test]
fn inspect_legacy_headers_only() {
    let path = PathBuf::from(
        std::env::var("TE_LEGACY_AUDIT_XLSX").expect("TE_LEGACY_AUDIT_XLSX must be set"),
    );
    let header_rows = [8usize, 2, 7, 0, 5, 6];
    let mut workbook = open_workbook_auto(path).unwrap();
    let sheet_names = workbook.sheet_names().to_vec();
    assert_eq!(sheet_names.len(), header_rows.len());

    for (sheet_name, header_row) in sheet_names.iter().zip(header_rows) {
        let range = workbook.worksheet_range(sheet_name).unwrap();
        let headers = range
            .rows()
            .nth(header_row)
            .unwrap()
            .iter()
            .map(ToString::to_string)
            .filter(|value| !value.trim().is_empty())
            .collect::<Vec<_>>();
        println!(
            "legacy_header: sheet={sheet_name:?} row={} headers={headers:?}",
            header_row + 1
        );
    }
}

#[test]
fn audit_live_database_snapshot_aggregates_only() {
    let path = PathBuf::from(
        std::env::var("TE_INVENTORY_AUDIT_DB").expect("TE_INVENTORY_AUDIT_DB must be set"),
    );
    let file_size = std::fs::metadata(&path).unwrap().len();
    let db = InventoryDb::open_at_with_size(path, file_size).unwrap();
    let entries = db.load_entries().unwrap();

    let mut lifecycle = BTreeMap::<String, usize>::new();
    let mut requirement = [0usize; 4];
    let mut assets = BTreeMap::<String, usize>::new();
    let mut serials = BTreeMap::<String, usize>::new();
    let mut archived = 0usize;
    let mut manual = 0usize;
    let mut imported = 0usize;
    let mut blank_asset = 0usize;
    let mut blank_serial = 0usize;
    let mut blank_both = 0usize;

    for entry in &entries {
        *lifecycle.entry(entry.lifecycle_status.clone()).or_default() += 1;
        requirement[match entry.calibration_requirement {
            CalibrationRequirement::Required => 0,
            CalibrationRequirement::ReferenceOnly => 1,
            CalibrationRequirement::NotRequired => 2,
            CalibrationRequirement::Unknown => 3,
        }] += 1;
        archived += usize::from(entry.archived);
        manual += usize::from(entry.manual_entry);
        imported += usize::from(entry.import_provenance.is_some());
        let asset = entry.asset_number.trim().to_ascii_lowercase();
        let serial = entry.serial_number.trim().to_ascii_lowercase();
        blank_asset += usize::from(asset.is_empty());
        blank_serial += usize::from(serial.is_empty());
        blank_both += usize::from(asset.is_empty() && serial.is_empty());
        if !asset.is_empty() {
            *assets.entry(asset).or_default() += 1;
        }
        if !serial.is_empty() {
            *serials.entry(serial).or_default() += 1;
        }
    }

    let duplicate_asset_keys = assets.values().filter(|count| **count > 1).count();
    let duplicate_asset_rows = assets.values().filter(|count| **count > 1).sum::<usize>();
    let duplicate_serial_keys = serials.values().filter(|count| **count > 1).count();
    let duplicate_serial_rows = serials.values().filter(|count| **count > 1).sum::<usize>();

    println!(
        "live_db: total={} active={} archived={} imported={} manual={} blank_asset={} blank_serial={} blank_both={}",
        entries.len(),
        entries.len() - archived,
        archived,
        imported,
        manual,
        blank_asset,
        blank_serial,
        blank_both
    );
    println!(
        "requirements: required={} reference_only={} not_required={} unknown={}",
        requirement[0], requirement[1], requirement[2], requirement[3]
    );
    println!("lifecycle: {lifecycle:?}");
    println!(
        "duplicate_identity: asset_keys={} asset_rows={} serial_keys={} serial_rows={}",
        duplicate_asset_keys,
        duplicate_asset_rows,
        duplicate_serial_keys,
        duplicate_serial_rows
    );
    assert!(!entries.is_empty());
}

/// Opt-in: empty temp DB pulls from the live product shared root (read path; should not publish
/// while ops < 1000 and snapshot age < 24h).
#[test]
fn pull_live_shared_root_into_empty_temp_db_when_requested() {
    if std::env::var("TE_LIVE_SHARED_PULL").ok().as_deref() != Some("1") {
        return;
    }

    let shared_root = PathBuf::from(
        std::env::var("INVENTORY_MANAGEMENT_SHARED_ROOT").unwrap_or_else(|_| {
            r"S:\Engineering\Public\Syed_Hassaan_Shah\Inventory_Management_App\modules\TE_Test_Equipment"
                .to_string()
        }),
    );
    assert!(
        shared_root.exists(),
        "shared root missing: {}",
        shared_root.display()
    );

    let op_count_before = std::fs::read_dir(shared_root.join("shared").join("inventory").join("ops"))
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .flat_map(|client| std::fs::read_dir(client.path()).into_iter().flatten())
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".op.json"))
        })
        .count();

    let temp = std::env::temp_dir().join(format!(
        "te-live-shared-pull-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    std::fs::create_dir_all(&temp).unwrap();
    let db_path = temp.join("inventory.feox");
    let db = InventoryDb::open_at_with_size(db_path, 64 * 1024 * 1024).unwrap();
    assert!(db.load_entries().unwrap().is_empty());

    let result = sync::test_support::run_shared_sync_with_root(&db, &shared_root).unwrap();
    let entries = db.load_entries().unwrap();
    println!(
        "live_shared_pull: available={} enabled={} entries={} message={}",
        result.shared.available,
        result.shared.enabled,
        entries.len(),
        result.shared.message
    );
    assert!(result.shared.available, "shared root should be available");
    assert_eq!(
        entries.len(),
        op_count_before,
        "fresh client should hydrate one entry per durable op (ops={op_count_before})"
    );

    let op_count_after = std::fs::read_dir(shared_root.join("shared").join("inventory").join("ops"))
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.ok())
        .flat_map(|client| std::fs::read_dir(client.path()).into_iter().flatten())
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".op.json"))
        })
        .count();
    assert_eq!(
        op_count_after, op_count_before,
        "pull-only client must not write extra operation files"
    );

    let _ = std::fs::remove_dir_all(temp);
}
