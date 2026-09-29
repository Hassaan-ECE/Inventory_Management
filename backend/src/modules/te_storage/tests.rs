use super::{
    model::*, mutations::*, store::InventoryDb, sync::test_support::run_shared_sync_with_root,
};
use std::{fs, path::PathBuf};
use uuid::Uuid;

fn root() -> PathBuf {
    std::env::temp_dir().join(format!("te-storage-test-{}", Uuid::new_v4()))
}
fn input() -> InventoryEntryInput {
    InventoryEntryInput {
        pn: "00123".into(),
        pr: "00045".into(),
        po: "00987".into(),
        qty: Some(2.0),
        location: "A1".into(),
        ..Default::default()
    }
}

#[test]
fn empty_start_and_reopen_preserve_identifiers_and_quantity() {
    let path = root().join("te-storage-room.feox");
    {
        let db = InventoryDb::open_at(path.clone()).unwrap();
        assert!(db.load_entries().unwrap().is_empty());
        create_entry_in_store(input(), &db).unwrap();
    }
    let reopened = InventoryDb::open_at(path).unwrap();
    let entries = reopened.load_entries().unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        (&*entries[0].pn, &*entries[0].pr, &*entries[0].po),
        ("00123", "00045", "00987")
    );
    assert_eq!(entries[0].qty, Some(2.0));
}

#[test]
fn two_clients_sync_offline_create_edit_and_delete() {
    let root = root();
    let shared = root.join("shared-room");
    let a = InventoryDb::open_at(root.join("a.feox")).unwrap();
    let b = InventoryDb::open_at(root.join("b.feox")).unwrap();
    let created = create_entry_in_store(input(), &a).unwrap().entry;
    assert!(
        !run_shared_sync_with_root(&a, &shared)
            .unwrap()
            .shared
            .available
    );
    assert_eq!(a.load_entries().unwrap().len(), 1);
    fs::create_dir_all(&shared).unwrap();
    run_shared_sync_with_root(&a, &shared).unwrap();
    run_shared_sync_with_root(&b, &shared).unwrap();
    assert_eq!(b.load_entries().unwrap()[0].pn, "00123");
    let mut edited = input();
    edited.location = "B2".into();
    update_entry_in_store(
        &created.entry_uuid,
        edited,
        Some(InventoryEntryEditContext {
            base_version: Some(created.updated_at),
            changed_fields: vec!["location".into()],
        }),
        &b,
    )
    .unwrap();
    run_shared_sync_with_root(&b, &shared).unwrap();
    run_shared_sync_with_root(&a, &shared).unwrap();
    assert_eq!(a.load_entries().unwrap()[0].location, "B2");
    delete_entry_in_store(&created.entry_uuid, &a).unwrap();
    run_shared_sync_with_root(&a, &shared).unwrap();
    run_shared_sync_with_root(&b, &shared).unwrap();
    assert!(b.load_entries().unwrap().is_empty());
    run_shared_sync_with_root(&a, &shared).unwrap();
    assert!(a.load_entries().unwrap().is_empty());
}

#[test]
fn stale_editor_preserves_unrelated_changes() {
    let db = InventoryDb::open_at(root().join("storage.feox")).unwrap();
    let entry = create_entry_in_store(input(), &db).unwrap().entry;
    let mut location_edit = input();
    location_edit.location = "C3".into();
    update_entry_in_store(&entry.entry_uuid, location_edit, None, &db).unwrap();
    let mut old_form = input();
    old_form.qty = Some(9.0);
    let updated = update_entry_in_store(
        &entry.entry_uuid,
        old_form,
        Some(InventoryEntryEditContext {
            base_version: Some(entry.updated_at),
            changed_fields: vec!["qty".into()],
        }),
        &db,
    )
    .unwrap()
    .entry;
    assert_eq!(updated.location, "C3");
    assert_eq!(updated.qty, Some(9.0));
}

#[test]
fn invalid_input_does_not_create_records() {
    let db = InventoryDb::open_at(root().join("storage.feox")).unwrap();
    assert!(create_entry_in_store(InventoryEntryInput::default(), &db).is_err());
    for qty in [-1.0, f64::NAN, f64::INFINITY, 1_000_001.0] {
        let mut bad = input();
        bad.qty = Some(qty);
        assert!(create_entry_in_store(bad, &db).is_err());
    }
    assert!(db.load_entries().unwrap().is_empty());
}

#[test]
fn unchanged_stale_form_does_not_overwrite_current_fields() {
    let db = InventoryDb::open_at(root().join("storage.feox")).unwrap();
    let entry = create_entry_in_store(input(), &db).unwrap().entry;
    let mut newer = input();
    newer.location = "Z9".into();
    update_entry_in_store(&entry.entry_uuid, newer, None, &db).unwrap();
    let saved = update_entry_in_store(
        &entry.entry_uuid,
        input(),
        Some(InventoryEntryEditContext {
            base_version: Some(entry.updated_at),
            changed_fields: vec![],
        }),
        &db,
    )
    .unwrap();
    assert_eq!(saved.entry.location, "Z9");
}

#[test]
fn merged_peer_fields_survive_startup_recovery() {
    let root = root();
    let shared = root.join("share");
    fs::create_dir_all(&shared).unwrap();
    let a = InventoryDb::open_at(root.join("a.feox")).unwrap();
    let b = InventoryDb::open_at(root.join("b.feox")).unwrap();
    let entry = create_entry_in_store(input(), &a).unwrap().entry;
    run_shared_sync_with_root(&a, &shared).unwrap();
    run_shared_sync_with_root(&b, &shared).unwrap();
    let context = |field: &str| {
        Some(InventoryEntryEditContext {
            base_version: Some(entry.updated_at.clone()),
            changed_fields: vec![field.into()],
        })
    };
    let mut location = input();
    location.location = "B2".into();
    update_entry_in_store(&entry.entry_uuid, location, context("location"), &b).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let mut quantity = input();
    quantity.qty = Some(9.0);
    update_entry_in_store(&entry.entry_uuid, quantity, context("qty"), &a).unwrap();
    run_shared_sync_with_root(&b, &shared).unwrap();
    run_shared_sync_with_root(&a, &shared).unwrap();
    run_shared_sync_with_root(&b, &shared).unwrap();
    assert_eq!(a.load_entries().unwrap()[0].location, "B2");
    super::sync::recover_local_sync_state(&a).unwrap();
    super::sync::recover_local_sync_state(&b).unwrap();
    run_shared_sync_with_root(&a, &shared).unwrap();
    run_shared_sync_with_root(&b, &shared).unwrap();
    for db in [&a, &b] {
        let row = db.load_entries().unwrap().remove(0);
        assert_eq!(row.location, "B2");
        assert_eq!(row.qty, Some(9.0));
    }
}

#[test]
fn repeated_offline_edits_preserve_other_clients_fields() {
    let root = root();
    let shared = root.join("share");
    fs::create_dir_all(&shared).unwrap();
    let a = InventoryDb::open_at(root.join("a.feox")).unwrap();
    let b = InventoryDb::open_at(root.join("b.feox")).unwrap();
    let entry = create_entry_in_store(input(), &a).unwrap().entry;
    run_shared_sync_with_root(&a, &shared).unwrap();
    run_shared_sync_with_root(&b, &shared).unwrap();
    let mut edit = input();
    edit.qty = Some(9.0);
    let first = update_entry_in_store(
        &entry.entry_uuid,
        edit,
        Some(InventoryEntryEditContext {
            base_version: Some(entry.updated_at.clone()),
            changed_fields: vec!["qty".into()],
        }),
        &a,
    )
    .unwrap()
    .entry;
    let mut edit = input();
    edit.qty = Some(9.0);
    edit.notes = "keep these notes".into();
    update_entry_in_store(
        &entry.entry_uuid,
        edit,
        Some(InventoryEntryEditContext {
            base_version: Some(first.updated_at),
            changed_fields: vec!["notes".into()],
        }),
        &a,
    )
    .unwrap();
    std::thread::sleep(std::time::Duration::from_millis(5));
    let mut edit = input();
    edit.location = "Z9".into();
    update_entry_in_store(
        &entry.entry_uuid,
        edit,
        Some(InventoryEntryEditContext {
            base_version: Some(entry.updated_at),
            changed_fields: vec!["location".into()],
        }),
        &b,
    )
    .unwrap();
    run_shared_sync_with_root(&b, &shared).unwrap();
    run_shared_sync_with_root(&a, &shared).unwrap();
    run_shared_sync_with_root(&b, &shared).unwrap();
    for db in [&a, &b] {
        super::sync::recover_local_sync_state(db).unwrap();
        let row = db.load_entries().unwrap().remove(0);
        assert_eq!(row.qty, Some(9.0));
        assert_eq!(row.notes, "keep these notes");
        assert_eq!(row.location, "Z9");
    }
}

#[test]
fn stale_patch_cannot_remove_the_last_identifying_field() {
    let db = InventoryDb::open_at(root().join("storage.feox")).unwrap();
    let original = InventoryEntryInput {
        pn: "P".into(),
        model: "M".into(),
        ..Default::default()
    };
    let entry = create_entry_in_store(original, &db).unwrap().entry;
    update_entry_in_store(
        &entry.entry_uuid,
        InventoryEntryInput {
            model: "M".into(),
            ..Default::default()
        },
        None,
        &db,
    )
    .unwrap();
    let result = update_entry_in_store(
        &entry.entry_uuid,
        InventoryEntryInput {
            pn: "P".into(),
            ..Default::default()
        },
        Some(InventoryEntryEditContext {
            base_version: Some(entry.updated_at),
            changed_fields: vec!["model".into()],
        }),
        &db,
    );
    assert!(result.is_err());
    assert_eq!(db.load_entries().unwrap()[0].model, "M");
}

#[test]
fn failed_mutations_cannot_reappear_through_recovery_or_sync() {
    let root = root();
    let shared = root.join("share");
    fs::create_dir_all(&shared).unwrap();
    let a = InventoryDb::open_at(root.join("a.feox")).unwrap();
    let b = InventoryDb::open_at(root.join("b.feox")).unwrap();
    super::sync::test_support::fail_next_outbox_write();
    assert!(create_entry_in_store(input(), &a).is_err());
    super::sync::recover_local_sync_state(&a).unwrap();
    assert!(a.load_entries().unwrap().is_empty());
    let entry = create_entry_in_store(input(), &a).unwrap().entry;
    super::sync::test_support::fail_next_outbox_write();
    let mut edit = input();
    edit.location = "Should not save".into();
    assert!(update_entry_in_store(&entry.entry_uuid, edit, None, &a).is_err());
    super::sync::recover_local_sync_state(&a).unwrap();
    assert_eq!(a.load_entries().unwrap()[0].location, "A1");
    super::sync::test_support::fail_next_outbox_write();
    assert!(delete_entry_in_store(&entry.entry_uuid, &a).is_err());
    super::sync::recover_local_sync_state(&a).unwrap();
    run_shared_sync_with_root(&a, &shared).unwrap();
    run_shared_sync_with_root(&b, &shared).unwrap();
    assert_eq!(b.load_entries().unwrap().len(), 1);
    assert_eq!(b.load_entries().unwrap()[0].location, "A1");
}
