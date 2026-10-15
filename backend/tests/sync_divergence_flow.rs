//! Regression coverage for the 2026-10-15 TE divergence: a client whose snapshots did not
//! cover another client's newest operation had its own operation files compacted away,
//! every other client refused its snapshots, and a stale client later overwrote the
//! shared manifest. These tests replay that sequence against the current sync loop.

#[path = "support/backend.rs"]
mod backend;
pub(crate) use backend::{model, platform, store, sync};
#[path = "support/sync_fixtures.rs"]
mod sync_fixtures;

use std::{fs, path::Path};

use serde_json::Value;
use store::InventoryDb;
use sync::test_support::{maybe_publish_snapshot, run_shared_sync_with_root, SharedSyncPaths};
use sync::{queue_entry_operation, SyncOperationType};
use sync_fixtures::{existing_shared_root, manifest_path, sample_entry, test_db};

fn create_entry(db: &InventoryDb, id: &str, entry_uuid: &str, description: &str) {
    let entry = sample_entry(id, entry_uuid, description);
    db.put_entry(&entry).unwrap();
    db.set_next_entry_id(id.parse::<i64>().unwrap() + 1)
        .unwrap();
    queue_entry_operation(
        db,
        SyncOperationType::InventoryEntryCreate,
        entry,
        Vec::new(),
        None,
    )
    .unwrap();
    db.flush();
}

fn has_entry(db: &InventoryDb, entry_uuid: &str) -> bool {
    db.find_entry(entry_uuid).unwrap().is_some()
}

fn client_ops_dir(shared_root: &Path, db: &InventoryDb) -> std::path::PathBuf {
    let client_id = db.get_or_create_client_id().unwrap();
    SharedSyncPaths::from_shared_root(shared_root)
        .ops_dir
        .join(client_id)
}

fn read_manifest_json(shared_root: &Path) -> Value {
    serde_json::from_slice(&fs::read(manifest_path(shared_root)).unwrap()).unwrap()
}

fn set_manifest_field(shared_root: &Path, field: &str, value: &str) {
    let mut manifest = read_manifest_json(shared_root);
    manifest[field] = Value::String(value.to_string());
    fs::write(
        manifest_path(shared_root),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

fn manifest_watermark(shared_root: &Path, db: &InventoryDb) -> u64 {
    let client_id = db.get_or_create_client_id().unwrap();
    read_manifest_json(shared_root)["watermarks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|watermark| watermark["clientId"] == client_id.as_str())
        .and_then(|watermark| watermark["localSeq"].as_u64())
        .unwrap_or(0)
}

/// Runs `sync` while another client's operation files are invisible, the way Randy's
/// 0.1.1 could not read one of the other clients' operations.
fn sync_without_seeing(shared_root: &Path, db: &InventoryDb, hidden: &InventoryDb) {
    let dir = client_ops_dir(shared_root, hidden);
    let parked = dir.with_extension("parked");
    fs::rename(&dir, &parked).unwrap();
    let result = run_shared_sync_with_root(db, shared_root);
    fs::rename(&parked, &dir).unwrap();
    result.unwrap();
}

fn three_clients(prefix: &str) -> (std::path::PathBuf, InventoryDb, InventoryDb, InventoryDb) {
    let shared_root = existing_shared_root(&format!("{prefix}-root"));
    let a = test_db(&format!("{prefix}-a"));
    let b = test_db(&format!("{prefix}-b"));
    let r = test_db(&format!("{prefix}-r"));
    for db in [&a, &b, &r] {
        run_shared_sync_with_root(db, &shared_root).unwrap();
    }
    (shared_root, a, b, r)
}

#[test]
fn snapshot_missing_a_newer_op_is_merged_instead_of_stranding_its_publisher() {
    let (shared_root, a, b, r) = three_clients("divergence-stranded");

    create_entry(&b, "1", "entry-from-b", "Created on B");
    run_shared_sync_with_root(&b, &shared_root).unwrap();

    create_entry(&r, "2", "entry-from-r", "Created on R");
    sync_without_seeing(&shared_root, &r, &b);

    // R publishes a snapshot that does not cover B's op, then compacts R's own op file.
    set_manifest_field(&shared_root, "createdAtUtc", "2020-01-01T00:00:00.000Z");
    sync_without_seeing(&shared_root, &r, &b);
    assert!(manifest_watermark(&shared_root, &r) >= 1);
    assert!(
        !client_ops_dir(&shared_root, &r).exists()
            || fs::read_dir(client_ops_dir(&shared_root, &r))
                .unwrap()
                .count()
                == 0,
        "R's op file should have been compacted"
    );

    // Before 0.1.4, B refused R's snapshot forever and never saw R's entry.
    run_shared_sync_with_root(&b, &shared_root).unwrap();
    assert!(has_entry(&b, "entry-from-r"), "B must receive R's work");
    assert!(has_entry(&b, "entry-from-b"), "B must keep its own work");

    run_shared_sync_with_root(&a, &shared_root).unwrap();
    assert!(has_entry(&a, "entry-from-r"));
    assert!(has_entry(&a, "entry-from-b"));

    run_shared_sync_with_root(&r, &shared_root).unwrap();
    assert!(
        has_entry(&r, "entry-from-b"),
        "R catches up once it can read B"
    );
}

#[test]
fn client_behind_the_shared_snapshot_cannot_publish_over_it() {
    let (shared_root, a, b, r) = three_clients("divergence-stale-publish");

    create_entry(&r, "1", "entry-from-r", "Created on R");
    run_shared_sync_with_root(&r, &shared_root).unwrap();
    set_manifest_field(&shared_root, "createdAtUtc", "2020-01-01T00:00:00.000Z");
    run_shared_sync_with_root(&r, &shared_root).unwrap();
    let published = read_manifest_json(&shared_root)["snapshotId"].clone();

    // Leave an op file on the share and age the manifest, so the normal triggers fire.
    create_entry(&b, "2", "entry-from-b", "Created on B");
    run_shared_sync_with_root(&b, &shared_root).unwrap();
    set_manifest_field(&shared_root, "createdAtUtc", "2020-01-01T00:00:00.000Z");

    // A has not merged R's snapshot yet; publishing directly must be refused.
    let report =
        maybe_publish_snapshot(&a, &SharedSyncPaths::from_shared_root(&shared_root)).unwrap();
    assert!(!report.snapshot_published);
    assert_eq!(read_manifest_json(&shared_root)["snapshotId"], published);
}

#[test]
fn holder_of_work_lost_from_the_share_republishes_it() {
    let (shared_root, a, b, r) = three_clients("divergence-lost-work");
    let stale_manifest = fs::read(manifest_path(&shared_root)).unwrap();

    create_entry(&r, "1", "entry-from-r-1", "R one");
    create_entry(&r, "2", "entry-from-r-2", "R two");
    run_shared_sync_with_root(&r, &shared_root).unwrap();
    set_manifest_field(&shared_root, "createdAtUtc", "2020-01-01T00:00:00.000Z");
    run_shared_sync_with_root(&r, &shared_root).unwrap();

    // A stale client (pre-0.1.4) put back a manifest that knows nothing of R's work,
    // after R's snapshot had already compacted R's op files.
    fs::write(manifest_path(&shared_root), &stale_manifest).unwrap();
    run_shared_sync_with_root(&b, &shared_root).unwrap();
    assert!(!has_entry(&b, "entry-from-r-1"));

    // R notices its work is no longer on the share and republishes it.
    run_shared_sync_with_root(&r, &shared_root).unwrap();
    assert_eq!(manifest_watermark(&shared_root, &r), 2);

    run_shared_sync_with_root(&b, &shared_root).unwrap();
    run_shared_sync_with_root(&a, &shared_root).unwrap();
    for db in [&a, &b] {
        assert!(has_entry(db, "entry-from-r-1"));
        assert!(has_entry(db, "entry-from-r-2"));
    }
}

#[test]
fn newer_team_version_stops_snapshot_publishing_and_asks_for_update() {
    let (shared_root, _a, b, _r) = three_clients("divergence-version-gate");
    set_manifest_field(&shared_root, "appVersion", "99.0.0");
    set_manifest_field(&shared_root, "createdAtUtc", "2020-01-01T00:00:00.000Z");
    let before = read_manifest_json(&shared_root)["snapshotId"].clone();

    create_entry(&b, "1", "entry-from-b", "Created on B");
    let result = run_shared_sync_with_root(&b, &shared_root).unwrap();

    assert_eq!(read_manifest_json(&shared_root)["snapshotId"], before);
    assert!(result
        .shared
        .message
        .contains("99.0.0 is in use on the team"));
    assert!(
        fs::read_dir(client_ops_dir(&shared_root, &b))
            .unwrap()
            .count()
            > 0,
        "edits still reach the share as operations"
    );
}
