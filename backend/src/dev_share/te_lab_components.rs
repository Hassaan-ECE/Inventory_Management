//! TE Lab Components fixtures: storage areas and containers of every type (grid and
//! freeform, archived), parts made through the simple workflow and the full part dialog,
//! every stock status (in stock, low, none, unit review, mixed units, archived), every count
//! state, maximum-length text, unicode, and two computers moving, counting, editing, and
//! deleting after the first snapshot.

use std::{collections::BTreeMap, path::Path};

use serde_json::Value;

use super::{require_synced, text_of_length, without_local_ids, Computer, Team};
use crate::{
    modules::te_lab_components::{
        catalog_model::{
            ComponentAttributeValue, ComponentAttributes, Part, PartInput, StockCountInput,
            StockMoveInput, StockPlacementInput, StorageAreaInput, StorageContainer,
            StorageContainerInput,
        },
        catalog_mutations::{
            count_stock_in_store, create_part_in_store, create_stock_placement_in_store,
            create_storage_area_in_store, create_storage_container_in_store, delete_part_in_store,
            move_stock_in_store, update_part_in_store, update_stock_placement_in_store,
        },
        catalog_sync::run_shared_sync_with_root,
        simple_workflow::{create_simple_component_in_store, SimpleComponentInput},
        store::InventoryDb,
    },
    platform::common::{now_timestamp, CommandResult},
};

/// Internal part numbers the timeline looks parts up by.
const RESISTOR: &str = "R-10K-0.25W";
const OP_AMP: &str = "IC-LM358";
const TO_DELETE: &str = "DEL-001";
const ADDED_LATER: &str = "C-22P-ADDED";

pub(super) fn write(team: &Team, root: &Path) -> CommandResult<()> {
    let Team {
        alice, bob, carol, ..
    } = team;
    add_simple_components(&carol.lab)?;
    let containers = add_storage(&carol.lab)?;
    add_advanced_parts(&carol.lab, &containers)?;
    // Carol's first sync publishes the share's first catalog snapshot.
    for computer in [carol, alice, bob] {
        sync(computer, root)?;
    }

    // Alice and Bob both edit the op amp; Lab keeps the newest whole part.
    edit_part(alice, OP_AMP, |input| {
        input.notes = "Alice: use the SOIC version for new boards.".into()
    })?;
    edit_part(bob, OP_AMP, |input| {
        input.supplier = "Mouser (Bob, later)".into()
    })?;
    let resistor_bin = placement_of(alice, RESISTOR)?;
    move_stock_in_store(
        StockMoveInput {
            source_placement_uuid: resistor_bin.clone(),
            destination: Some(StockPlacementInput {
                container_uuid: shelf(&alice.lab)?.container_uuid,
                column_index: Some(0),
                row_index: Some(1),
                quantity: 0.0,
                count_state: "uncounted".into(),
                ..StockPlacementInput::default()
            }),
            quantity: 50.0,
            ..StockMoveInput::default()
        },
        &alice.lab,
    )?;
    count_stock_in_store(
        &placement_of(bob, OP_AMP)?,
        StockCountInput {
            quantity: 15.0,
            counted_by: "Bob".into(),
        },
        &bob.lab,
    )?;
    let doomed = find_part(bob, TO_DELETE)?;
    edit_part(bob, TO_DELETE, |input| input.archived = true)?;
    delete_part_in_store(&doomed.entry_uuid, &bob.lab)?;
    create_simple_component_in_store(
        simple(
            ADDED_LATER,
            "Ceramic capacitor",
            "22pF",
            "Added on Alice's computer after the first snapshot",
            100,
            "B1",
        ),
        &alice.lab,
    )?;

    for _ in 0..2 {
        for computer in team.everyone() {
            sync(computer, root)?;
        }
    }
    if comparable(alice) != comparable(bob) || comparable(bob) != comparable(carol) {
        return Err("Lab fixture computers did not converge.".to_string());
    }
    Ok(())
}

/// The everyday path: one dialog, a quantity, and a shelf code. The first one creates the
/// "TE Lab" area and its "Shelf" grid.
fn add_simple_components(db: &InventoryDb) -> CommandResult<()> {
    let resistor = SimpleComponentInput {
        part: PartInput {
            subcategory: "Carbon film".into(),
            manufacturer: "Yageo".into(),
            manufacturer_part_number: "CFR-25JB-52-10K".into(),
            package_type: "Axial".into(),
            attributes: attributes(&[
                ("resistance", "10", "kΩ"),
                ("tolerance", "5", "%"),
                ("power", "0.25", "W"),
            ]),
            ..simple(RESISTOR, "Resistor", "10kΩ", "10 kΩ ±5% 1/4 W", 0, "").part
        },
        quantity: 250,
        location: "A1".into(),
    };
    let op_amp = SimpleComponentInput {
        part: PartInput {
            reorder_point: Some(20.0),
            target_quantity: Some(50.0),
            supplier: "Digi-Key".into(),
            ..simple(OP_AMP, "IC", "LM358", "Dual op amp: low stock", 0, "").part
        },
        quantity: 12,
        location: "C4".into(),
    };
    for input in [
        resistor,
        op_amp,
        simple(
            "C-100N-50V",
            "Capacitor",
            "100nF",
            "Ceramic capacitor: no stock, no location",
            0,
            "",
        ),
        simple(
            "R-4K7-Ω",
            "Widerstand",
            "4,7 kΩ",
            "Unicode: Ω µ ± ° — 电阻 — مقاومة 🔧",
            40,
            "D7",
        ),
        simple(
            "EDGE-Z20",
            "Connector",
            "2-pin header",
            "Stored in the last shelf bin",
            8,
            "Z20",
        ),
        simple(
            TO_DELETE,
            "Fuse",
            "1A",
            "Deleted later on Bob's computer",
            5,
            "E5",
        ),
    ] {
        create_simple_component_in_store(input, db)?;
    }
    Ok(())
}

struct Containers {
    tray_rack: StorageContainer,
    overflow_box: StorageContainer,
    fridge: StorageContainer,
}

/// Areas and containers of every type, beyond the simple workflow's shelf.
fn add_storage(db: &InventoryDb) -> CommandResult<Containers> {
    let te_lab = shelf(db)?.area_uuid;
    let area = |name: &str, area_type: &str, owner: &str, archived: bool| {
        create_storage_area_in_store(
            StorageAreaInput {
                name: name.into(),
                area_type: area_type.into(),
                owner: owner.into(),
                description: format!("{name} ({area_type})"),
                archived,
            },
            db,
        )
        .map(|result| result.value.area_uuid)
    };
    let cold_storage = area("Cold Storage", "room", "Facilities", false)?;
    let lab_cart = area("Lab Cart", "mobile", "Bob", false)?;
    let old_bench = area("Old Bench", "desk", "", true)?;
    area("Shared Closet", "other", "Everyone", false)?;

    let container = |area_uuid: &str,
                     name: &str,
                     container_type: &str,
                     grid: Option<(u32, u32, u32)>,
                     archived: bool| {
        create_storage_container_in_store(
            StorageContainerInput {
                area_uuid: area_uuid.into(),
                name: name.into(),
                container_type: container_type.into(),
                grid_enabled: grid.is_some(),
                row_count: grid.map(|(rows, _, _)| rows),
                column_count: grid.map(|(_, columns, _)| columns),
                row_start: grid.map(|(_, _, row_start)| row_start),
                description: format!("{name} ({container_type})"),
                archived,
            },
            db,
        )
        .map(|result| result.value)
    };
    // The simple workflow stores into the first active grid by name, so every other grid is
    // named to sort after "Shelf".
    let tray_rack = container(&te_lab, "Tray Rack", "organizer", Some((4, 6, 1)), false)?;
    let overflow_box = container(&te_lab, "Overflow Box", "box", None, false)?;
    let fridge = container(&cold_storage, "Fridge", "cabinet", None, false)?;
    container(
        &lab_cart,
        "Utility Cart Drawers",
        "drawer_bank",
        Some((2, 3, 5)),
        false,
    )?;
    container(&old_bench, "Retired Bin", "other", None, true)?;
    Ok(Containers {
        tray_rack,
        overflow_box,
        fridge,
    })
}

/// The full part dialog and placement dialogs: every part and placement field.
fn add_advanced_parts(db: &InventoryDb, containers: &Containers) -> CommandResult<()> {
    let at = |container: &StorageContainer, quantity: f64, unit: &str| StockPlacementInput {
        container_uuid: container.container_uuid.clone(),
        quantity,
        unit_of_measure: unit.into(),
        count_state: "uncounted".into(),
        ..StockPlacementInput::default()
    };

    let mcu = create_part_in_store(
        PartInput {
            internal_part_number: "IC-STM32G431".into(),
            category: "IC".into(),
            subcategory: "Microcontroller".into(),
            manufacturer: "STMicroelectronics".into(),
            manufacturer_part_number: "STM32G431CBT6".into(),
            display_value: "STM32G431".into(),
            mounting_type: "smd".into(),
            package_type: "LQFP-48".into(),
            description: "Every part field filled".into(),
            attributes: attributes(&[("core", "Cortex-M4", ""), ("flash", "128", "KB")]),
            supplier: "Digi-Key".into(),
            supplier_sku: "497-STM32G431CBT6-ND".into(),
            supplier_packaging: "Tray".into(),
            product_url: "https://www.st.com/en/microcontrollers-microprocessors/stm32g431cb.html"
                .into(),
            datasheet_url: "https://www.st.com/resource/en/datasheet/stm32g431cb.pdf".into(),
            default_unit_of_measure: "pcs".into(),
            reorder_point: Some(5.0),
            target_quantity: Some(20.0),
            part_status: "active".into(),
            picture_path: Some("https://example.com/fixtures/stm32.jpg".into()),
            notes: "Keep in the anti-static tray.".into(),
            archived: false,
        },
        db,
    )?
    .value;
    create_stock_placement_in_store(
        StockPlacementInput {
            part_uuid: mcu.entry_uuid.clone(),
            column_index: Some(1),
            row_index: Some(1),
            count_state: "legacy_verified".into(),
            ..at(&containers.tray_rack, 8.0, "pcs")
        },
        db,
    )?;
    let old_bag = create_stock_placement_in_store(
        StockPlacementInput {
            part_uuid: mcu.entry_uuid.clone(),
            freeform_position: "Old bag".into(),
            notes: "Archived placement".into(),
            ..at(&containers.overflow_box, 2.0, "pcs")
        },
        db,
    )?
    .value;
    update_stock_placement_in_store(
        &old_bag.placement_uuid,
        StockPlacementInput {
            part_uuid: mcu.entry_uuid.clone(),
            freeform_position: "Old bag".into(),
            notes: "Archived placement".into(),
            archived: true,
            ..at(&containers.overflow_box, 0.0, "pcs")
        },
        db,
    )?;

    let wire = create_part_in_store(
        part(
            "WIRE-22AWG-RED",
            "Wire",
            "Hookup wire: stock in metres and pieces (mixed units)",
            "m",
        ),
        db,
    )?
    .value;
    for (position, quantity, unit) in [("Spool rack", 12.5, "m"), ("Cut lengths bag", 3.0, "pcs")] {
        create_stock_placement_in_store(
            StockPlacementInput {
                part_uuid: wire.entry_uuid.clone(),
                freeform_position: position.into(),
                ..at(&containers.overflow_box, quantity, unit)
            },
            db,
        )?;
    }

    let unsorted = create_part_in_store(
        part(
            "MISC-UNSORTED",
            "Misc",
            "Unit not decided yet (unit review)",
            "unknown",
        ),
        db,
    )?
    .value;
    create_stock_placement_in_store(
        StockPlacementInput {
            part_uuid: unsorted.entry_uuid,
            freeform_position: "Unsorted tray".into(),
            ..at(&containers.overflow_box, 5.0, "unknown")
        },
        db,
    )?;

    let paste = create_part_in_store(
        part(
            "CHEM-TIM-01",
            "Consumable",
            "Thermal paste: counted, with lot and date codes",
            "g",
        ),
        db,
    )?
    .value;
    create_stock_placement_in_store(
        StockPlacementInput {
            part_uuid: paste.entry_uuid,
            freeform_position: "Top shelf".into(),
            packaging: "Syringe".into(),
            lot_code: "LOT-24-117".into(),
            date_code: "2425".into(),
            condition: "Sealed".into(),
            count_state: "counted".into(),
            last_counted_at: Some(now_timestamp()),
            last_counted_by: "Carol".into(),
            notes: "Keep refrigerated.".into(),
            ..at(&containers.fridge, 30.0, "g")
        },
        db,
    )?;

    // Stock is added before archiving, since archived parts cannot take new stock.
    let retired = create_part_in_store(
        PartInput {
            part_status: "obsolete".into(),
            ..part("IC-NE555-OLD", "IC", "Archived part with stock", "pcs")
        },
        db,
    )?
    .value;
    create_stock_placement_in_store(
        StockPlacementInput {
            part_uuid: retired.entry_uuid.clone(),
            column_index: Some(0),
            row_index: Some(0),
            ..at(&containers.tray_rack, 3.0, "pcs")
        },
        db,
    )?;
    update_part_in_store(
        &retired.entry_uuid,
        PartInput {
            archived: true,
            ..part_input_from(&retired)
        },
        db,
    )?;

    let longest = create_part_in_store(longest_part(), db)?.value;
    create_stock_placement_in_store(
        StockPlacementInput {
            part_uuid: longest.entry_uuid,
            freeform_position: text_of_length("Longest position ", 512),
            packaging: text_of_length("Longest packaging ", 512),
            lot_code: text_of_length("Longest lot ", 512),
            date_code: text_of_length("Longest date code ", 512),
            condition: text_of_length("Longest condition ", 512),
            count_state: "counted".into(),
            last_counted_at: Some(now_timestamp()),
            last_counted_by: text_of_length("Longest counter ", 512),
            notes: text_of_length("Longest placement notes ", 8_000),
            ..at(&containers.overflow_box, 1_000_000_000.0, "pcs")
        },
        db,
    )?;
    Ok(())
}

fn longest_part() -> PartInput {
    let standard = |label: &str| text_of_length(label, 512);
    PartInput {
        internal_part_number: standard("LONGEST-"),
        category: standard("Longest category "),
        subcategory: standard("Longest subcategory "),
        manufacturer: standard("Longest manufacturer "),
        manufacturer_part_number: standard("Longest MPN "),
        display_value: standard("Longest value "),
        mounting_type: standard("Longest mounting "),
        package_type: standard("Longest package "),
        description: text_of_length("Longest description ", 4_000),
        attributes: attributes(&[(
            "longest_attribute",
            &text_of_length("Longest attribute value ", 4_000),
            &standard("Longest unit "),
        )]),
        supplier: standard("Longest supplier "),
        supplier_sku: standard("Longest SKU "),
        supplier_packaging: standard("Longest packaging "),
        product_url: format!("https://example.com/{}", "p".repeat(2_000)),
        datasheet_url: format!("https://example.com/{}.pdf", "d".repeat(2_000)),
        default_unit_of_measure: "pcs".into(),
        reorder_point: Some(0.0),
        target_quantity: Some(1_000_000_000.0),
        part_status: "active".into(),
        picture_path: Some(format!(r"C:\Fixtures\{}.png", "p".repeat(2_048 - 16))),
        notes: text_of_length("Longest notes ", 8_000),
        archived: false,
    }
}

fn simple(
    internal_part_number: &str,
    category: &str,
    value: &str,
    description: &str,
    quantity: u32,
    location: &str,
) -> SimpleComponentInput {
    SimpleComponentInput {
        part: PartInput {
            internal_part_number: internal_part_number.into(),
            category: category.into(),
            display_value: value.into(),
            description: description.into(),
            ..PartInput::default()
        },
        quantity,
        location: location.into(),
    }
}

fn part(internal_part_number: &str, category: &str, description: &str, unit: &str) -> PartInput {
    PartInput {
        internal_part_number: internal_part_number.into(),
        category: category.into(),
        description: description.into(),
        default_unit_of_measure: unit.into(),
        ..PartInput::default()
    }
}

fn attributes(values: &[(&str, &str, &str)]) -> ComponentAttributes {
    values
        .iter()
        .map(|(key, value, unit)| {
            (
                key.to_string(),
                ComponentAttributeValue {
                    value: value.to_string(),
                    unit: unit.to_string(),
                },
            )
        })
        .collect()
}

/// The part dialog's input for an existing part, unchanged.
fn part_input_from(part: &Part) -> PartInput {
    PartInput {
        internal_part_number: part.internal_part_number.clone(),
        category: part.category.clone(),
        subcategory: part.subcategory.clone(),
        manufacturer: part.manufacturer.clone(),
        manufacturer_part_number: part.manufacturer_part_number.clone(),
        display_value: part.display_value.clone(),
        mounting_type: part.mounting_type.clone(),
        package_type: part.package_type.clone(),
        description: part.description.clone(),
        attributes: part.attributes.clone(),
        supplier: part.supplier.clone(),
        supplier_sku: part.supplier_sku.clone(),
        supplier_packaging: part.supplier_packaging.clone(),
        product_url: part.product_url.clone(),
        datasheet_url: part.datasheet_url.clone(),
        default_unit_of_measure: part.default_unit_of_measure.clone(),
        reorder_point: part.reorder_point,
        target_quantity: part.target_quantity,
        part_status: part.part_status.clone(),
        picture_path: Some(part.picture_path.clone()),
        notes: part.notes.clone(),
        archived: part.archived,
    }
}

fn shelf(db: &InventoryDb) -> CommandResult<StorageContainer> {
    db.load_storage_containers()?
        .into_iter()
        .find(|container| container.name == "Shelf")
        .ok_or_else(|| "The simple workflow did not create the Shelf.".to_string())
}

fn find_part(computer: &Computer, internal_part_number: &str) -> CommandResult<Part> {
    computer
        .lab
        .load_parts()?
        .into_iter()
        .find(|part| part.internal_part_number == internal_part_number)
        .ok_or_else(|| format!("No Lab fixture part {internal_part_number}."))
}

fn placement_of(computer: &Computer, internal_part_number: &str) -> CommandResult<String> {
    let part = find_part(computer, internal_part_number)?;
    computer
        .lab
        .load_stock_placements_for_part(&part.entry_uuid)?
        .into_iter()
        .find(|placement| !placement.archived)
        .map(|placement| placement.placement_uuid)
        .ok_or_else(|| format!("Lab fixture part {internal_part_number} has no stock."))
}

fn edit_part(
    computer: &Computer,
    internal_part_number: &str,
    change: impl FnOnce(&mut PartInput),
) -> CommandResult<()> {
    let part = find_part(computer, internal_part_number)?;
    let mut input = part_input_from(&part);
    change(&mut input);
    update_part_in_store(&part.entry_uuid, input, &computer.lab)?;
    Ok(())
}

fn sync(computer: &Computer, root: &Path) -> CommandResult<()> {
    require_synced(
        &run_shared_sync_with_root(&computer.lab, root)?.shared,
        "TE Lab Components",
    )
}

/// Every catalog record on a computer, keyed by type and identity, without local ids.
fn comparable(computer: &Computer) -> BTreeMap<String, Value> {
    let db = &computer.lab;
    let mut records = BTreeMap::new();
    for part in db.load_parts().unwrap_or_default() {
        records.insert(
            format!("part {}", part.entry_uuid),
            without_local_ids(&part),
        );
    }
    for area in db.load_storage_areas().unwrap_or_default() {
        records.insert(format!("area {}", area.area_uuid), without_local_ids(&area));
    }
    for container in db.load_storage_containers().unwrap_or_default() {
        let key = format!("container {}", container.container_uuid);
        records.insert(key, without_local_ids(&container));
    }
    for placement in db.load_stock_placements().unwrap_or_default() {
        let key = format!("placement {}", placement.placement_uuid);
        records.insert(key, without_local_ids(&placement));
    }
    records
}

#[cfg(test)]
pub(super) fn assert_matches_share(
    newcomer: &Computer,
    root: &Path,
    summary: &super::DevShareSummary,
) {
    use crate::modules::te_lab_components::catalog_model::summarize_part_stock;
    use std::collections::BTreeSet;

    sync(newcomer, root).unwrap();
    let db = &newcomer.lab;
    let parts = db.load_parts().unwrap();
    assert_eq!(parts.len(), summary.lab_parts);
    assert_eq!(
        db.load_stock_placements().unwrap().len(),
        summary.lab_placements
    );
    assert!(parts
        .iter()
        .any(|part| part.internal_part_number == ADDED_LATER));
    assert!(parts
        .iter()
        .all(|part| part.internal_part_number != TO_DELETE));
    let op_amp = parts
        .iter()
        .find(|part| part.internal_part_number == OP_AMP)
        .unwrap();
    assert_eq!(op_amp.supplier, "Mouser (Bob, later)");

    let placements = db.load_stock_placements().unwrap();
    let statuses: BTreeSet<String> = parts
        .iter()
        .map(|part| format!("{:?}", summarize_part_stock(part, &placements).stock_status))
        .collect();
    assert_eq!(
        statuses,
        BTreeSet::from(
            [
                "Archived",
                "InStock",
                "LowStock",
                "MixedUnits",
                "NoStock",
                "UnitReview"
            ]
            .map(String::from)
        )
    );
    let count_states: BTreeSet<_> = placements
        .into_iter()
        .map(|placement| placement.count_state)
        .collect();
    assert_eq!(
        count_states,
        BTreeSet::from(["counted", "legacy_verified", "uncounted"].map(String::from))
    );

    let mut by_kind: BTreeMap<&str, Vec<Value>> = BTreeMap::new();
    for (key, record) in comparable(newcomer) {
        let kind = match key.split(' ').next().unwrap() {
            "part" => "part",
            "area" => "storage area",
            "container" => "storage container",
            _ => "stock placement",
        };
        by_kind.entry(kind).or_default().push(record);
    }
    for (kind, records) in by_kind {
        let mut fields: BTreeSet<String> = BTreeSet::new();
        for record in &records {
            if let Value::Object(object) = record {
                fields.extend(object.keys().cloned());
            }
        }
        // `legacy` only holds fields carried over by the removed 0.1 Lab migration; no
        // current dialog sets it.
        fields.remove("legacy");
        let fields: Vec<String> = fields.into_iter().collect();
        super::assert_every_field_is_set(kind, &records, &fields);
    }
}
