use super::super::catalog_model::CATALOG_SCHEMA_VERSION;
use super::*;
use std::{env, fs, path::PathBuf};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct TestSyncRecord {
    label: String,
    local_seq: u64,
}

#[test]
fn sync_keys_do_not_appear_in_catalog_scans() {
    let db = test_db();

    db.set_schema_version(1).unwrap();
    db.set_sync_schema_version(1).unwrap();
    db.set_client_id("client-1").unwrap();
    db.set_device_id("device-1").unwrap();
    db.set_next_local_seq(2).unwrap();
    db.set_sync_watermark("client-1", 1).unwrap();
    db.put_sync_outbox_record(1, &sync_record("pending", 1))
        .unwrap();
    db.put_sync_applied_marker("op-1", &sync_record("applied", 1))
        .unwrap();
    db.put_sync_tombstone("entry-1", &sync_record("deleted", 1))
        .unwrap();
    db.put_sync_corrupt_record("hash-1", &sync_record("corrupt", 1))
        .unwrap();
    db.put_sync_value("meta:test_raw", b"meta").unwrap();
    db.put_sync_value("sync:test_raw", b"sync").unwrap();

    assert!(!db.has_catalog_entities().unwrap());
    assert!(db.load_parts().unwrap().is_empty());
    assert_eq!(
        db.get_sync_value("meta:test_raw").unwrap().as_deref(),
        Some(&b"meta"[..])
    );
    assert!(db.put_sync_value("part:test_raw", b"bad").is_err());
}

#[test]
fn catalog_schema_setup_drops_v1_rows_and_sync_state() {
    let db = test_db();
    db.put_bytes(b"entry:legacy-uuid", b"{}").unwrap();
    db.put_bytes(b"entry_id:7", b"legacy-uuid").unwrap();
    db.put_sync_outbox_record(1, &sync_record("v1-op", 1))
        .unwrap();
    db.set_sync_schema_version(1).unwrap();

    db.ensure_catalog_schema().unwrap();

    assert_eq!(db.schema_version().unwrap(), Some(CATALOG_SCHEMA_VERSION));
    assert_eq!(db.sync_schema_version().unwrap(), Some(2));
    assert!(!db.contains_key(b"entry:legacy-uuid"));
    assert!(!db.contains_key(b"entry_id:7"));
    assert!(db
        .sync_outbox_record::<TestSyncRecord>(1)
        .unwrap()
        .is_none());
}

fn sync_record(label: &str, local_seq: u64) -> TestSyncRecord {
    TestSyncRecord {
        label: label.to_string(),
        local_seq,
    }
}

fn test_db() -> InventoryDb {
    let root = unique_test_dir("store");
    fs::create_dir_all(&root).unwrap();
    InventoryDb::open_at(root.join("inventory.feox")).unwrap()
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
}
