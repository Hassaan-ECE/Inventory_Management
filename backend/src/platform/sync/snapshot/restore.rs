//! Owner one-shot (2026-10-15): republish an authoritative TE snapshot built from a
//! known-good older snapshot plus any newer operation files still on the share.
//!
//! Watermarks are raised to dominate the current manifest so every installed client
//! accepts the snapshot on its next sync. The tool never pushes operations, never
//! compacts operation files, and never prunes snapshots. It only writes one new
//! snapshot file, a manifest backup, and the manifest.
//!
//!   $env:RESTORE_TE_SHARED_ROOT = '<module root that contains shared\inventory>'
//!   $env:RESTORE_TE_SOURCE_SNAPSHOT = 'snapshot-<id>.snapshot.json'
//!   $env:RESTORE_TE_CONFIRM = 'publish'   # omit for a dry run
//!   cargo test --manifest-path backend/Cargo.toml --lib restore_authoritative_te_snapshot -- --ignored --nocapture
//!
//! Rehearse what an installed client does with the result (uses the unchanged sync loop):
//!
//!   $env:RESTORE_TE_SHARED_ROOT = '<rehearsal module root>'
//!   $env:RESTORE_TE_CLIENT_DB = '<copy of a client inventory.feox>'
//!   cargo test --manifest-path backend/Cargo.toml --lib rehearse_client_sync_after_restore -- --ignored --nocapture

use std::{collections::HashMap, env, fs, path::PathBuf};

use uuid::Uuid;

use crate::{
    modules::te_test_equipment::model::InventoryEntry as TeEntry,
    platform::{common::now_timestamp, store::ModuleStore},
};

use super::super::{apply::pull_remote_operations, SharedSyncPaths};
use super::{
    apply::replace_from_snapshot,
    io::{
        read_manifest, read_verified_snapshot, sign_manifest_for_configured_trust,
        snapshot_checksum, watermarks_to_map, write_manifest, write_new_json_file,
    },
    lock::SnapshotLock,
    publish::{build_snapshot, manifest_for_snapshot},
    types::{SharedInventorySnapshot, SNAPSHOT_FILE_SUFFIX},
};

fn required_env(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| panic!("set {name}"))
}

fn temp_dir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("te-restore-{label}-{}", Uuid::new_v4().simple()));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
#[ignore = "owner one-shot; writes the live shared manifest when RESTORE_TE_CONFIRM=publish"]
fn restore_authoritative_te_snapshot() {
    let paths = SharedSyncPaths::from_shared_root(required_env("RESTORE_TE_SHARED_ROOT"));
    let source_file = required_env("RESTORE_TE_SOURCE_SNAPSHOT");
    let publish = env::var("RESTORE_TE_CONFIRM").ok().as_deref() == Some("publish");

    let current = read_manifest::<TeEntry>(&paths)
        .expect("read manifest")
        .expect("manifest exists");
    eprintln!(
        "current manifest: {} ({} entries, app {}, source {})",
        current.snapshot_id, current.entry_count, current.app_version, current.source_client_id
    );

    let bytes = fs::read(paths.snapshots_dir.join(&source_file)).expect("read source snapshot");
    let source: SharedInventorySnapshot<TeEntry> =
        serde_json::from_slice(&bytes).expect("parse source snapshot");
    let expected = snapshot_checksum(&source).expect("checksum");
    assert_eq!(
        source.checksum, expected,
        "source snapshot checksum does not verify"
    );
    eprintln!(
        "source snapshot: {} ({} entries, app {}, source {}) checksum ok",
        source.snapshot_id,
        source.entries.len(),
        source.app_version,
        source.source_client_id
    );

    let work = temp_dir("work");
    let db = ModuleStore::<TeEntry>::open_at(work.join("restore.feox")).expect("open work db");
    replace_from_snapshot(&db, &source).expect("load source snapshot");
    let pulled = pull_remote_operations(&db, &paths).expect("pull newer operations");
    eprintln!(
        "applied newer ops from share: entries_changed={} corrupt={}",
        pulled.entries_changed, pulled.corrupt_count
    );

    let mut merged: HashMap<String, u64> = watermarks_to_map(&current.watermarks);
    db.scan_sync_watermarks(usize::MAX, |client_id, local_seq| {
        let slot = merged.entry(client_id).or_default();
        *slot = (*slot).max(local_seq);
        Ok(true)
    })
    .expect("scan watermarks");
    for (client_id, local_seq) in &merged {
        db.set_sync_watermark(client_id, *local_seq)
            .expect("set watermark");
    }
    let mut listed: Vec<_> = merged.iter().collect();
    listed.sort();
    eprintln!("published watermarks: {listed:?}");

    let snapshot = build_snapshot(&db).expect("build snapshot");
    eprintln!(
        "new snapshot: {} ({} entries, {} tombstones, {} entry states)",
        snapshot.snapshot_id,
        snapshot.entries.len(),
        snapshot.tombstones.len(),
        snapshot.entry_states.len()
    );

    if !publish {
        eprintln!("DRY RUN: set RESTORE_TE_CONFIRM=publish to write it.");
        return;
    }

    let _lock = SnapshotLock::try_acquire(&paths)
        .expect("lock")
        .expect("snapshot lock is held by another client; retry shortly");
    let still_current = read_manifest::<TeEntry>(&paths)
        .expect("re-read manifest")
        .expect("manifest");
    assert_eq!(
        still_current.snapshot_id, current.snapshot_id,
        "manifest changed while preparing; rerun"
    );

    let snapshot_file = format!("{}{}", snapshot.snapshot_id, SNAPSHOT_FILE_SUFFIX);
    write_new_json_file(&paths.snapshots_dir.join(&snapshot_file), &snapshot)
        .expect("write snapshot");
    let verified = read_verified_snapshot::<TeEntry>(
        &paths,
        &manifest_for_snapshot(&snapshot, snapshot_file.clone()),
    )
    .expect("verify written snapshot");

    fs::create_dir_all(&paths.backups_dir).expect("backups dir");
    let backup = paths.backups_dir.join(format!(
        "manifest-before-restore-{}.json",
        now_timestamp().replace([':', '.'], "-")
    ));
    fs::copy(&paths.manifest_path, &backup).expect("back up manifest");

    let mut manifest = manifest_for_snapshot(&verified, snapshot_file);
    sign_manifest_for_configured_trust(&mut manifest).expect("sign manifest");
    write_manifest(&paths, &manifest).expect("write manifest");
    eprintln!(
        "PUBLISHED {} ; previous manifest saved to {}",
        manifest.snapshot_id,
        backup.display()
    );
}

#[test]
#[ignore = "rehearsal only; runs the normal sync loop on a copied client database"]
fn rehearse_client_sync_after_restore() {
    let root = required_env("RESTORE_TE_SHARED_ROOT");
    let client_db = PathBuf::from(required_env("RESTORE_TE_CLIENT_DB"));
    let work = temp_dir("client");
    let copy = work.join("inventory.feox");
    fs::copy(&client_db, &copy).expect("copy client db");
    let size = fs::metadata(&copy)
        .expect("metadata")
        .len()
        .max(64 * 1024 * 1024);
    let db = ModuleStore::<TeEntry>::open_at_with_size(copy, size).expect("open client copy");

    let before = db.load_entries().expect("entries").len();
    let result = super::super::apply::run_shared_sync_with_root(&db, &root).expect("sync");
    let after = db.load_entries().expect("entries").len();
    let paths = SharedSyncPaths::from_shared_root(&root);
    let manifest = read_manifest::<TeEntry>(&paths)
        .expect("manifest")
        .expect("manifest");
    eprintln!(
        "client entries before={before} after={after} applied_manifest={} message={:?}",
        db.last_snapshot_id().expect("last snapshot").as_deref() == Some(&manifest.snapshot_id),
        result.shared.message
    );
    let export = work.join("entries.json");
    fs::write(
        &export,
        serde_json::to_vec_pretty(&db.load_entries().expect("entries")).expect("json"),
    )
    .expect("write entries");
    eprintln!("client entries written to {}", export.display());
}
