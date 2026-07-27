use std::{env, fs, path::PathBuf};

use uuid::Uuid;

use crate::modules::te_lab_components::{
    model::{InventoryEntry, InventoryEntryInput},
    mutations::{
        create_entry_in_store, delete_entry_in_store, toggle_verified_entry_in_store,
        update_entry_in_store,
    },
    store::InventoryDb,
};

use super::test_support::{run_shared_sync_with_root, SyncOperationEnvelope};

#[test]
fn schema_v1_two_database_flow_keeps_lab_fields_isolated() {
    let db_a = test_db("lab-sync-db-a");
    let db_b = test_db("lab-sync-db-b");
    let shared_root = existing_shared_root("lab-sync-root");

    run_shared_sync_with_root(&db_a, &shared_root).unwrap();
    run_shared_sync_with_root(&db_b, &shared_root).unwrap();

    let created = create_entry_in_store(test_input("Created on A"), &db_a).unwrap();
    let entry_uuid = created.entry.entry_uuid.clone();
    let create_operation = outbox_operation(&db_a, 1);
    assert_eq!(create_operation.schema_version, 1);
    let create_json = serde_json::to_value(&create_operation).unwrap();
    let entry_json = &create_json["payload"]["entry"];
    assert!(entry_json.get("verifiedInSurvey").is_some());
    assert!(entry_json.get("calibrationRequirement").is_none());
    assert!(entry_json.get("calibrationDueAt").is_none());

    run_shared_sync_with_root(&db_a, &shared_root).unwrap();
    assert!(
        run_shared_sync_with_root(&db_b, &shared_root)
            .unwrap()
            .entries_changed
    );
    assert_eq!(
        db_b.find_entry(&entry_uuid).unwrap().unwrap().description,
        "Created on A"
    );

    let existing = db_b.find_entry(&entry_uuid).unwrap().unwrap();
    let mut update_input = input_from_entry(&existing);
    update_input.description = "Updated on B".to_string();
    update_entry_in_store(&entry_uuid, update_input, None, &db_b).unwrap();

    run_shared_sync_with_root(&db_b, &shared_root).unwrap();
    assert!(
        run_shared_sync_with_root(&db_a, &shared_root)
            .unwrap()
            .entries_changed
    );
    assert_eq!(
        db_a.find_entry(&entry_uuid).unwrap().unwrap().description,
        "Updated on B"
    );

    toggle_verified_entry_in_store(&entry_uuid, true, &db_a).unwrap();
    run_shared_sync_with_root(&db_a, &shared_root).unwrap();
    assert!(
        run_shared_sync_with_root(&db_b, &shared_root)
            .unwrap()
            .entries_changed
    );
    assert!(
        db_b.find_entry(&entry_uuid)
            .unwrap()
            .unwrap()
            .verified_in_survey
    );

    delete_entry_in_store(&entry_uuid, &db_b).unwrap();
    run_shared_sync_with_root(&db_b, &shared_root).unwrap();
    assert!(
        run_shared_sync_with_root(&db_a, &shared_root)
            .unwrap()
            .entries_changed
    );
    assert!(db_a.find_entry(&entry_uuid).unwrap().is_none());
}

fn test_input(description: &str) -> InventoryEntryInput {
    InventoryEntryInput {
        description: description.to_string(),
        lifecycle_status: "active".to_string(),
        working_status: "unknown".to_string(),
        ..InventoryEntryInput::default()
    }
}

fn input_from_entry(entry: &InventoryEntry) -> InventoryEntryInput {
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
        verified_in_survey: entry.verified_in_survey,
        archived: entry.archived,
        picture_path: Some(entry.picture_path.clone()),
    }
}

fn outbox_operation(db: &InventoryDb, local_seq: u64) -> SyncOperationEnvelope {
    db.sync_outbox_record(local_seq).unwrap().unwrap()
}

/// Live one-shot: strip catalog.entity.* outbox rows from this machine's AppData so
/// installed 0.1.0 can open again. Does not change the 0.1.0 binary.
///
///   set REPAIR_LIVE_APPDATA=1
///   cargo test repair_live_appdata_catalog_outbox -- --ignored --nocapture
#[test]
#[ignore = "live AppData repair; set REPAIR_LIVE_APPDATA=1"]
fn repair_live_appdata_catalog_outbox() {
    if env::var("REPAIR_LIVE_APPDATA").ok().as_deref() != Some("1") {
        eprintln!("skip: set REPAIR_LIVE_APPDATA=1 to run live repair");
        return;
    }

    let local = env::var("LOCALAPPDATA").expect("LOCALAPPDATA");
    let data_dir = PathBuf::from(local).join("com.inventory.management");
    let lab_path = data_dir.join("te-lab-components.feox");
    assert!(lab_path.is_file(), "missing {}", lab_path.display());

    let backup = data_dir.join(format!(
        "te-lab-components.feox.bak-repair-{}",
        Uuid::new_v4().simple()
    ));
    fs::copy(&lab_path, &backup).expect("backup Lab DB");
    eprintln!("backup: {}", backup.display());

    let db = InventoryDb::open_at(lab_path.clone()).expect("open Lab DB");
    let mut remove = Vec::new();
    let mut keep = 0usize;
    db.scan_sync_outbox_raw(None, usize::MAX, |local_seq, value| {
        if serde_json::from_slice::<SyncOperationEnvelope>(value).is_ok() {
            keep += 1;
            eprintln!("keep   seq {local_seq}");
        } else {
            remove.push(local_seq);
            let type_name = serde_json::from_slice::<serde_json::Value>(value)
                .ok()
                .and_then(|value| value.get("type")?.as_str().map(str::to_string))
                .unwrap_or_else(|| "<unknown>".to_string());
            eprintln!("remove seq {local_seq} type={type_name}");
        }
        Ok(true)
    })
    .expect("scan outbox");

    for local_seq in &remove {
        db.delete_sync_outbox_record(*local_seq).expect("delete outbox");
    }
    let mut max_seq = 0u64;
    db.scan_sync_outbox_raw(None, usize::MAX, |local_seq, _| {
        max_seq = max_seq.max(local_seq);
        Ok(true)
    })
    .expect("rescan");
    if max_seq > 0 {
        let next = db.next_local_seq().expect("next_local_seq");
        if next <= max_seq {
            db.set_next_local_seq(max_seq + 1).expect("set next_local_seq");
        }
    }
    db.flush();
    eprintln!(
        "repaired: removed {} row(s), kept {} inventory.entry row(s)",
        remove.len(),
        keep
    );
}

/// Regression: installed 0.1.0 panics when Lab outbox holds catalog.entity.* rows.
/// Inventory-style recovery must skip those and still succeed.
#[test]
fn inventory_style_recovery_skips_catalog_entity_outbox_ops() {
    use crate::modules::te_lab_components::sync::recover_local_sync_state;

    let db = test_db("lab-recovery-catalog-outbox");
    let catalog_op = serde_json::json!({
        "schemaVersion": 1,
        "opId": "catalog-op-1",
        "clientId": "client-a",
        "deviceId": "device-a",
        "localSeq": 1,
        "appVersion": "0.1.0",
        "createdAtUtc": "2026-07-27T00:00:00.000Z",
        "type": "catalog.entity.upsert",
        "entityType": "part",
        "entityId": "part-uuid-1",
        "baseVersion": null,
        "mutationTsUtc": "2026-07-27T00:00:00.000Z",
        "payload": { "entity": null, "changedFields": [], "deletedAtUtc": null },
        "checksum": "deadbeef",
        "auth": null
    });
    db.put_sync_outbox_record(1u64, &catalog_op).unwrap();
    db.set_next_local_seq(1).unwrap();

    let report = recover_local_sync_state(&db).expect("recovery must not fail on catalog outbox ops");
    assert_eq!(report.repaired_outbox_operations, 0);
    // Sequence counter must still advance past the foreign outbox row.
    assert!(db.next_local_seq().unwrap() > 1);
}

fn test_db(prefix: &str) -> InventoryDb {
    let root = unique_test_dir(prefix);
    fs::create_dir_all(&root).unwrap();
    InventoryDb::open_at(root.join("te-lab-components.feox")).unwrap()
}

fn existing_shared_root(prefix: &str) -> PathBuf {
    let root = unique_test_dir(prefix);
    fs::create_dir_all(&root).unwrap();
    root
}

fn unique_test_dir(prefix: &str) -> PathBuf {
    env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
}
