//! EVCS Storage fixtures: every column filled and empty, hidden-column-only rows, identifiers
//! with leading zeros, quantity edge cases, maximum-length text, unicode, duplicate part
//! numbers, a delete, and two computers editing the same entries (merged field by field).

use std::{collections::BTreeMap, path::Path};

use serde_json::Value;

use super::{require_synced, text_of_length, without_local_ids, Computer, Team};
use crate::{
    modules::te_storage::{
        model::{CommandResult, InventoryEntry, InventoryEntryEditContext, InventoryEntryInput},
        mutations::{create_entry_in_store, delete_entry_in_store, update_entry_in_store},
    },
    platform::sync::test_support::run_shared_sync_with_root,
};

/// Model names the timeline looks entries up by.
const MERGED_EDIT: &str = "EVMA253-S merged";
const DISPUTED_EDIT: &str = "EVMA253-S disputed";
const TO_DELETE: &str = "Fixture to delete";

pub(super) fn write(team: &Team, root: &Path) -> CommandResult<()> {
    let Team {
        alice, bob, carol, ..
    } = team;
    for input in starting_entries() {
        create_entry_in_store(input, &bob.storage)?;
    }
    // Bob's first sync publishes the share's first snapshot; Alice and Carol start from it.
    for computer in [bob, alice, carol] {
        sync(computer, root)?;
    }

    // EVCS Storage keeps a version per field: both edits to different fields are kept, and
    // for the same field the later edit wins.
    edit(alice, MERGED_EDIT, "qty", |input| input.qty = Some(3.0))?;
    edit(carol, MERGED_EDIT, "location", |input| {
        input.location = "Eng Lab/E8".into()
    })?;
    edit(alice, DISPUTED_EDIT, "notes", |input| {
        input.notes = "Alice: counted 2 on the shelf".into()
    })?;
    edit(carol, DISPUTED_EDIT, "notes", |input| {
        input.notes = "Carol (later): counted 4 after restock".into()
    })?;

    delete_entry_in_store(&find(alice, TO_DELETE)?.entry_uuid, &alice.storage)?;
    create_entry_in_store(
        InventoryEntryInput {
            notes: "Added on Carol's computer after the first snapshot".into(),
            ..part("00777", "Phoenix Contact", "CHARX SEC-3100")
        },
        &carol.storage,
    )?;

    for _ in 0..2 {
        for computer in team.everyone() {
            sync(computer, root)?;
        }
    }
    if comparable(alice) != comparable(bob) || comparable(bob) != comparable(carol) {
        return Err("EVCS Storage fixture computers did not converge.".to_string());
    }
    Ok(())
}

fn starting_entries() -> Vec<InventoryEntryInput> {
    vec![
        InventoryEntryInput {
            pr: "00045".into(),
            po: "PO-2026-0193".into(),
            description: "Every column filled".into(),
            notes: "Spare for charger bay 2.".into(),
            qty: Some(4.0),
            ..part("EVMA253-S", "Shin EV", "EVMA253-S")
        },
        InventoryEntryInput {
            pr: "00046".into(),
            description: "Only the hidden PR # and Description columns are filled".into(),
            ..InventoryEntryInput::default()
        },
        InventoryEntryInput {
            po: "00987".into(),
            ..part("00123", "", "")
        },
        InventoryEntryInput {
            location: "Eng Lab".into(),
            ..part("WL-24V-RED", "Patlite", "Warning light")
        },
        InventoryEntryInput {
            location: "Eng Lab/E8".into(),
            ..part("CBL-CCS1-5M", "Phoenix Contact", "CCS1 cable, 5 m")
        },
        InventoryEntryInput {
            qty: None,
            ..part("FUSE-30A", "Littelfuse", "No quantity")
        },
        InventoryEntryInput {
            qty: Some(0.0),
            ..part("FUSE-40A", "Littelfuse", "Quantity zero")
        },
        InventoryEntryInput {
            qty: Some(0.5),
            ..part("GREASE-DI", "Dow", "Fractional quantity")
        },
        InventoryEntryInput {
            qty: Some(1_000_000.0),
            ..part("LABEL-QR", "Brady", "Largest allowed quantity")
        },
        InventoryEntryInput {
            pn: text_of_length("Longest PN ", 512),
            pr: text_of_length("Longest PR ", 512),
            po: text_of_length("Longest PO ", 512),
            manufacturer: text_of_length("Longest manufacturer ", 512),
            model: text_of_length("Longest model ", 512),
            description: text_of_length("Longest description ", 4_000),
            location: text_of_length("Longest location ", 512),
            notes: text_of_length("Longest notes ", 8_000),
            qty: Some(1.0),
        },
        InventoryEntryInput {
            location: "Lager Süd / 倉庫".into(),
            notes: "Line one\nLine two\nEmoji 🔌 and RTL اختبار".into(),
            ..part("PN-Ω-01", "Schneider Électrique", "Contacteur µ ±")
        },
        InventoryEntryInput {
            notes: "Duplicate PN (1 of 2)".into(),
            ..part("230071240403000990", "Shin EV", "EVMA253-S")
        },
        InventoryEntryInput {
            notes: "Duplicate PN (2 of 2)".into(),
            ..part("230071240403000990", "Shin EV", "EVMA253-S")
        },
        InventoryEntryInput {
            description: "Deleted later on Alice's computer".into(),
            ..part("DEL-001", "Generic", TO_DELETE)
        },
        InventoryEntryInput {
            description: "Alice edits Qty, Carol edits Location; both are kept".into(),
            ..part("MRG-001", "Shin EV", MERGED_EDIT)
        },
        InventoryEntryInput {
            description: "Alice and Carol edit Notes; Carol's later edit wins".into(),
            ..part("DSP-001", "Shin EV", DISPUTED_EDIT)
        },
    ]
}

fn part(pn: &str, manufacturer: &str, model: &str) -> InventoryEntryInput {
    InventoryEntryInput {
        pn: pn.into(),
        manufacturer: manufacturer.into(),
        model: model.into(),
        location: "TE Storage Room".into(),
        qty: Some(1.0),
        ..InventoryEntryInput::default()
    }
}

fn find(computer: &Computer, model: &str) -> CommandResult<InventoryEntry> {
    computer
        .storage
        .load_entries()?
        .into_iter()
        .find(|entry| entry.model == model)
        .ok_or_else(|| format!("No EVCS Storage fixture with model {model}."))
}

/// Edits one field from the version this computer last saw, as the edit dialog does.
fn edit(
    computer: &Computer,
    model: &str,
    field: &str,
    change: impl FnOnce(&mut InventoryEntryInput),
) -> CommandResult<()> {
    let entry = find(computer, model)?;
    let mut input = InventoryEntryInput {
        pn: entry.pn.clone(),
        pr: entry.pr.clone(),
        po: entry.po.clone(),
        manufacturer: entry.manufacturer.clone(),
        model: entry.model.clone(),
        description: entry.description.clone(),
        location: entry.location.clone(),
        notes: entry.notes.clone(),
        qty: entry.qty,
    };
    change(&mut input);
    update_entry_in_store(
        &entry.entry_uuid,
        input,
        Some(InventoryEntryEditContext {
            base_version: Some(entry.updated_at.clone()),
            changed_fields: vec![field.to_string()],
        }),
        &computer.storage,
    )?;
    Ok(())
}

fn sync(computer: &Computer, root: &Path) -> CommandResult<()> {
    require_synced(
        &run_shared_sync_with_root(&computer.storage, root)?.shared,
        "EVCS Storage",
    )
}

fn comparable(computer: &Computer) -> BTreeMap<String, Value> {
    computer
        .storage
        .load_entries()
        .unwrap_or_default()
        .iter()
        .map(|entry| (entry.entry_uuid.clone(), without_local_ids(entry)))
        .collect()
}

#[cfg(test)]
pub(super) fn assert_matches_share(newcomer: &Computer, root: &Path, expected_entries: usize) {
    use crate::platform::sync::SyncRecord;

    sync(newcomer, root).unwrap();
    let entries = newcomer.storage.load_entries().unwrap();
    assert_eq!(entries.len(), expected_entries);

    let by_model = |model: &str| entries.iter().find(|entry| entry.model == model).unwrap();
    assert_eq!(by_model(MERGED_EDIT).qty, Some(3.0));
    assert_eq!(by_model(MERGED_EDIT).location, "Eng Lab/E8");
    assert!(by_model(DISPUTED_EDIT).notes.starts_with("Carol (later)"));
    assert!(entries.iter().all(|entry| entry.model != TO_DELETE));
    assert!(entries.iter().any(|entry| entry.pn == "00123"));

    let records: Vec<Value> = entries.iter().map(without_local_ids).collect();
    let fields: Vec<String> = InventoryEntry::ENTRY_FIELDS
        .iter()
        .map(|field| super::te_test_equipment::snake_to_camel(field))
        .collect();
    super::assert_every_field_is_set("EVCS Storage", &records, &fields);
}
