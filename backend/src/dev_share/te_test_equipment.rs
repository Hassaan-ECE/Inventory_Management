//! TE Test Equipment fixtures: every lifecycle and working status, every calibration state,
//! verified, archived, and deleted entries, empty and maximum-length text, unicode, duplicate
//! asset numbers, and two computers editing the same entries.

use std::{collections::BTreeMap, path::Path};

use chrono::{Days, Local, NaiveDate};
use serde_json::Value;

use super::{require_synced, text_of_length, without_local_ids, Computer, Team};
use crate::{
    modules::te_test_equipment::{
        model::{
            entry_to_input, now_timestamp, CalibrationRequirement, CommandResult, InventoryEntry,
            InventoryEntryEditContext, InventoryEntryInput,
        },
        mutations::{
            create_entry_in_store, delete_entry_in_store, set_archived_entry_in_store,
            toggle_verified_entry_in_store, update_entry_in_store,
        },
    },
    platform::sync::test_support::run_shared_sync_with_root,
};

/// Model names the timeline looks entries up by.
const MERGED_EDIT: &str = "N6705C";
const DISPUTED_EDIT: &str = "34465A";
const TO_VERIFY: &str = "E36313A";
const TO_ARCHIVE: &str = "TDS2024";
const TO_DELETE: &str = "Fixture to delete";

pub(super) fn write(team: &Team, root: &Path) -> CommandResult<()> {
    let Team {
        alice, bob, carol, ..
    } = team;
    for input in starting_entries(Local::now().date_naive()) {
        create_entry_in_store(input, &alice.te)?;
    }
    // Alice's first sync publishes the share's first snapshot; Bob and Carol start from it.
    for computer in team.everyone() {
        sync(computer, root)?;
    }

    // Both edit one entry from the same version, different fields: TE merges them.
    edit(alice, MERGED_EDIT, "location", |input| {
        input.location = "Bench 2".into()
    })?;
    edit(bob, MERGED_EDIT, "notes", |input| {
        input.notes = "Firmware updated to A.03.12 on Bob's computer.".into()
    })?;
    // Both change the same field: the later operation wins.
    edit(alice, DISPUTED_EDIT, "location", |input| {
        input.location = "Bench 3 (Alice)".into()
    })?;
    edit(bob, DISPUTED_EDIT, "location", |input| {
        input.location = "Bench 7 (Bob, later)".into()
    })?;

    toggle_verified_entry_in_store(&find(bob, TO_VERIFY)?.entry_uuid, true, &bob.te)?;
    set_archived_entry_in_store(&find(bob, TO_ARCHIVE)?.entry_uuid, true, &bob.te)?;
    let doomed = find(carol, TO_DELETE)?.entry_uuid;
    set_archived_entry_in_store(&doomed, true, &carol.te)?;
    delete_entry_in_store(&doomed, &carol.te)?;
    create_entry_in_store(
        InventoryEntryInput {
            asset_number: "TE-2001".into(),
            description: "Added on Carol's computer after the first snapshot".into(),
            ..equipment("Tektronix", "MSO44", "")
        },
        &carol.te,
    )?;

    // Two rounds, so every computer has applied every other computer's operations.
    for _ in 0..2 {
        for computer in team.everyone() {
            sync(computer, root)?;
        }
    }
    if comparable(alice) != comparable(bob) || comparable(bob) != comparable(carol) {
        return Err("TE fixture computers did not converge.".to_string());
    }
    Ok(())
}

fn starting_entries(today: NaiveDate) -> Vec<InventoryEntryInput> {
    let date = |offset_days: i64| {
        let day = if offset_days >= 0 {
            today.checked_add_days(Days::new(offset_days as u64))
        } else {
            today.checked_sub_days(Days::new(offset_days.unsigned_abs()))
        };
        day.map(|day| day.format("%Y-%m-%d").to_string())
    };
    let calibrated = |input: InventoryEntryInput, last: i64, due: i64| InventoryEntryInput {
        calibration_requirement: CalibrationRequirement::Required,
        last_calibrated_at: date(last),
        calibration_due_at: date(due),
        calibration_interval_months: Some(12),
        ..input
    };

    vec![
        InventoryEntryInput {
            asset_number: "TE-1001".into(),
            serial_number: "MY58001234".into(),
            project_name: "EVCS Gen 3".into(),
            assigned_to: "Alice".into(),
            condition: "Good".into(),
            links: "https://www.keysight.com/us/en/product/DSOX3024T.html".into(),
            notes: "Calibration current. Probe set stays with the scope.".into(),
            certificate_ref: Some("CERT-2026-0412".into()),
            calibration_vendor: Some("Keysight Service".into()),
            calibration_notes: Some("Annual; includes probe compensation.".into()),
            verified_at: Some(now_timestamp()),
            verified_by: Some("Alice".into()),
            picture_path: Some("https://example.com/fixtures/oscilloscope.jpg".into()),
            ..calibrated(
                equipment("Keysight", "DSOX3024T", "Oscilloscope: calibration current"),
                -60,
                305,
            )
        },
        InventoryEntryInput {
            asset_number: "TE-1002".into(),
            calibration_vendor: Some("Transcat".into()),
            ..calibrated(
                equipment("Fluke", "87V", "Multimeter: calibration due soon"),
                -353,
                12,
            )
        },
        InventoryEntryInput {
            asset_number: "TE-1003".into(),
            ..calibrated(
                equipment("Rigol", "DP832", "Power supply: calibration overdue"),
                -385,
                -20,
            )
        },
        InventoryEntryInput {
            asset_number: "TE-1004".into(),
            calibration_requirement: CalibrationRequirement::Required,
            ..equipment(
                "Tektronix",
                "AFG31022",
                "Function generator: calibration required, no due date",
            )
        },
        InventoryEntryInput {
            asset_number: "TE-1005".into(),
            out_to_calibration: true,
            calibration_vendor: Some("Tektronix Service".into()),
            location: "Out to vendor".into(),
            ..calibrated(
                equipment("Keithley", "2450", "SourceMeter: out to calibration"),
                -365,
                0,
            )
        },
        InventoryEntryInput {
            calibration_requirement: CalibrationRequirement::ReferenceOnly,
            ..equipment("Fluke", "5522A", "Multifunction calibrator: reference only")
        },
        InventoryEntryInput {
            calibration_requirement: CalibrationRequirement::NotRequired,
            qty: Some(12.0),
            ..equipment("Pomona", "6754", "BNC cable kit: calibration not required")
        },
        InventoryEntryInput {
            working_status: "unknown".into(),
            ..equipment(
                "Agilent",
                "34401A",
                "Bench multimeter: calibration requirement not reviewed",
            )
        },
        InventoryEntryInput {
            lifecycle_status: "repair".into(),
            working_status: "not_working".into(),
            condition: "Fan noise; sent for repair".into(),
            ..equipment("Chroma", "61604", "AC source: in repair, not working")
        },
        InventoryEntryInput {
            lifecycle_status: "scrapped".into(),
            working_status: "not_working".into(),
            condition: "Display failed".into(),
            ..equipment("HP", "3478A", "Multimeter: scrapped")
        },
        InventoryEntryInput {
            lifecycle_status: "missing".into(),
            working_status: "unknown".into(),
            location: String::new(),
            notes: "Last seen on Bench 4.".into(),
            ..equipment("Fluke", "1587 FC", "Insulation tester: missing")
        },
        InventoryEntryInput {
            lifecycle_status: "rental".into(),
            working_status: "limited".into(),
            assigned_to: "Electro Rent".into(),
            notes: "Channel 3 is limited to 10 A.".into(),
            ..equipment("Chroma", "17020", "Battery cycler: rental, limited")
        },
        InventoryEntryInput {
            model: "Model-only entry".into(),
            ..InventoryEntryInput::default()
        },
        InventoryEntryInput {
            qty: Some(0.0),
            ..equipment("Generic", "Banana lead set", "Quantity zero")
        },
        InventoryEntryInput {
            qty: Some(2.5),
            ..equipment("Generic", "Ferrite clamp", "Fractional quantity")
        },
        InventoryEntryInput {
            qty: Some(1_000_000.0),
            ..equipment("Generic", "Cable tie", "Largest allowed quantity")
        },
        InventoryEntryInput {
            qty: None,
            ..equipment("Generic", "Probe tip kit", "No quantity")
        },
        longest_entry(date(-1), date(400)),
        InventoryEntryInput {
            asset_number: "TE-Ω-001".into(),
            project_name: "Prüfstand Süd".into(),
            location: "Labor Süd / 実験室 2".into(),
            notes: "Line one\nLine two\n\tIndented line\nEmoji 🔧⚡ and RTL اختبار".into(),
            ..equipment(
                "Rohde & Schwarz",
                "ZNB8 – Vektor-Netzwerkanalysator",
                "Unicode: Ω µ ± ° — 测试设备",
            )
        },
        InventoryEntryInput {
            asset_number: "TE-DUP-0001".into(),
            serial_number: "DUP-A".into(),
            ..equipment("Fluke", "117", "Duplicate asset number (1 of 2)")
        },
        InventoryEntryInput {
            asset_number: "TE-DUP-0001".into(),
            serial_number: "DUP-B".into(),
            ..equipment("Fluke", "117", "Duplicate asset number (2 of 2)")
        },
        InventoryEntryInput {
            links: "mailto:calibration@example.com".into(),
            picture_path: Some(r"C:\Fixtures\Pictures\missing-picture.jpg".into()),
            ..equipment(
                "Tektronix",
                "P6139B",
                "Mail link and a picture file that does not exist",
            )
        },
        equipment("Keysight", TO_VERIFY, "Verified later on Bob's computer"),
        equipment("Tektronix", TO_ARCHIVE, "Archived later on Bob's computer"),
        equipment("Generic", TO_DELETE, "Deleted later on Carol's computer"),
        equipment(
            "Keysight",
            MERGED_EDIT,
            "Alice and Bob edit different fields; both edits are kept",
        ),
        equipment(
            "Keysight",
            DISPUTED_EDIT,
            "Alice and Bob edit the location; Bob's later edit wins",
        ),
    ]
}

/// Every text field at its length limit, and the longest calibration interval.
fn longest_entry(last: Option<String>, due: Option<String>) -> InventoryEntryInput {
    let standard = |label: &str| text_of_length(label, 512);
    InventoryEntryInput {
        asset_number: standard("Longest asset number "),
        serial_number: standard("Longest serial number "),
        qty: Some(1.0),
        manufacturer: standard("Longest manufacturer "),
        model: standard("Longest model "),
        description: text_of_length("Longest description ", 4_000),
        project_name: standard("Longest project "),
        location: standard("Longest location "),
        assigned_to: standard("Longest assignee "),
        links: format!("https://example.com/{}", "a".repeat(4_000 - 20)),
        notes: text_of_length("Longest notes ", 8_000),
        lifecycle_status: "active".into(),
        working_status: "working".into(),
        condition: standard("Longest condition "),
        calibration_requirement: CalibrationRequirement::Required,
        out_to_calibration: false,
        last_calibrated_at: last,
        calibration_due_at: due,
        calibration_interval_months: Some(1_200),
        certificate_ref: Some(standard("Longest certificate ")),
        calibration_vendor: Some(standard("Longest vendor ")),
        calibration_notes: Some(text_of_length("Longest calibration notes ", 8_000)),
        verified_at: Some(now_timestamp()),
        verified_by: Some(standard("Longest verifier ")),
        archived: false,
        picture_path: Some(format!(r"C:\Fixtures\{}.png", "p".repeat(2_048 - 16))),
    }
}

fn equipment(manufacturer: &str, model: &str, description: &str) -> InventoryEntryInput {
    InventoryEntryInput {
        manufacturer: manufacturer.into(),
        model: model.into(),
        description: description.into(),
        qty: Some(1.0),
        location: "TE Lab".into(),
        lifecycle_status: "active".into(),
        working_status: "working".into(),
        ..InventoryEntryInput::default()
    }
}

fn find(computer: &Computer, model: &str) -> CommandResult<InventoryEntry> {
    computer
        .te
        .load_entries()?
        .into_iter()
        .find(|entry| entry.model == model)
        .ok_or_else(|| format!("No TE fixture with model {model}."))
}

/// Edits one field from the version this computer last saw, as the edit dialog does.
fn edit(
    computer: &Computer,
    model: &str,
    field: &str,
    change: impl FnOnce(&mut InventoryEntryInput),
) -> CommandResult<()> {
    let entry = find(computer, model)?;
    let mut input = entry_to_input(&entry);
    change(&mut input);
    update_entry_in_store(
        &entry.entry_uuid,
        input,
        Some(InventoryEntryEditContext {
            base_version: Some(entry.updated_at.clone()),
            changed_fields: vec![field.to_string()],
        }),
        &computer.te,
    )?;
    Ok(())
}

fn sync(computer: &Computer, root: &Path) -> CommandResult<()> {
    require_synced(
        &run_shared_sync_with_root(&computer.te, root)?.shared,
        "TE Test Equipment",
    )
}

fn comparable(computer: &Computer) -> BTreeMap<String, Value> {
    computer
        .te
        .load_entries()
        .unwrap_or_default()
        .iter()
        .map(|entry| (entry.entry_uuid.clone(), without_local_ids(entry)))
        .collect()
}

#[cfg(test)]
pub(super) fn assert_matches_share(newcomer: &Computer, root: &Path, expected_entries: usize) {
    use crate::modules::te_test_equipment::model::{derive_calibration_health, CalibrationHealth};
    use crate::platform::sync::SyncRecord;
    use std::collections::BTreeSet;

    sync(newcomer, root).unwrap();
    let entries = newcomer.te.load_entries().unwrap();
    assert_eq!(entries.len(), expected_entries);

    let by_model = |model: &str| entries.iter().find(|entry| entry.model == model).unwrap();
    assert_eq!(by_model(MERGED_EDIT).location, "Bench 2");
    assert!(by_model(MERGED_EDIT).notes.contains("Bob's computer"));
    assert_eq!(by_model(DISPUTED_EDIT).location, "Bench 7 (Bob, later)");
    assert!(by_model(TO_VERIFY).verified_at.is_some());
    assert!(by_model(TO_ARCHIVE).archived);
    assert!(entries.iter().all(|entry| entry.model != TO_DELETE));

    let lifecycle: BTreeSet<_> = entries
        .iter()
        .map(|e| e.lifecycle_status.as_str())
        .collect();
    let working: BTreeSet<_> = entries.iter().map(|e| e.working_status.as_str()).collect();
    assert_eq!(
        lifecycle,
        BTreeSet::from(["active", "missing", "rental", "repair", "scrapped"])
    );
    assert_eq!(
        working,
        BTreeSet::from(["limited", "not_working", "unknown", "working"])
    );
    let today = Local::now().date_naive();
    let health: BTreeSet<_> = entries
        .iter()
        .filter_map(|entry| derive_calibration_health(entry, today, 30))
        .map(|health| format!("{health:?}"))
        .collect();
    let every_health = [
        CalibrationHealth::MissingDue,
        CalibrationHealth::Overdue,
        CalibrationHealth::DueSoon,
        CalibrationHealth::Current,
        CalibrationHealth::NotApplicable,
        CalibrationHealth::Unknown,
        CalibrationHealth::OutToCal,
    ]
    .map(|health| format!("{health:?}"));
    assert_eq!(health, BTreeSet::from(every_health));

    let records: Vec<Value> = entries.iter().map(without_local_ids).collect();
    let fields: Vec<String> = InventoryEntry::ENTRY_FIELDS
        .iter()
        .map(|field| snake_to_camel(field))
        .collect();
    super::assert_every_field_is_set("TE Test Equipment", &records, &fields);
}

#[cfg(test)]
pub(super) fn snake_to_camel(field: &str) -> String {
    let mut parts = field.split('_');
    let first = parts.next().unwrap_or_default().to_string();
    parts.fold(first, |mut name, part| {
        let mut chars = part.chars();
        if let Some(initial) = chars.next() {
            name.extend(initial.to_uppercase());
            name.push_str(chars.as_str());
        }
        name
    })
}
