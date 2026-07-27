//! One-shot local demo seed for empty Lab catalog databases.
//!
//! Seeds only when catalog v2 is ready and there are no parts yet. Intended for
//! layout/review sessions after wiping throwaway local data — not a production import.

use crate::modules::te_lab_components::{
    catalog_model::{
        create_part, create_stock_placement, create_storage_area, create_storage_container,
        normalize_part_input, normalize_stock_placement_input, normalize_storage_area_input,
        normalize_storage_container_input, ComponentAttributeValue, ComponentAttributes, PartInput,
        StockPlacementInput, StorageAreaInput, StorageContainerInput,
    },
    model::{now_timestamp, CommandResult},
    store::InventoryDb,
};

pub(crate) fn seed_demo_catalog_if_empty(db: &InventoryDb) -> CommandResult<()> {
    if !db.load_parts()?.is_empty() {
        return Ok(());
    }
    seed_demo_catalog(db)
}

/// Replace catalog content with the six-part demo set (local repair / shared re-bootstrap).
/// Maintenance-only (ignored live tests); not used by the running app.
#[cfg(test)]
pub(crate) fn reseed_demo_catalog(db: &InventoryDb) -> CommandResult<()> {
    for placement in db.load_stock_placements()? {
        db.delete_stock_placement(&placement)?;
    }
    for part in db.load_parts()? {
        db.delete_part(&part)?;
    }
    for container in db.load_storage_containers()? {
        db.delete_storage_container(&container)?;
    }
    for area in db.load_storage_areas()? {
        db.delete_storage_area(&area)?;
    }
    db.flush();
    seed_demo_catalog(db)
}

fn seed_demo_catalog(db: &InventoryDb) -> CommandResult<()> {
    let area = create_storage_area(normalize_storage_area_input(StorageAreaInput {
        name: "Main Lab".to_string(),
        area_type: "lab".to_string(),
        owner: "TE".to_string(),
        description: "Main component storage".to_string(),
        ..StorageAreaInput::default()
    }));
    let desk = create_storage_area(normalize_storage_area_input(StorageAreaInput {
        name: "Desk — Alex".to_string(),
        area_type: "desk".to_string(),
        owner: "Alex".to_string(),
        description: "Desk-side organizer".to_string(),
        ..StorageAreaInput::default()
    }));
    let cabinet =
        create_storage_container(normalize_storage_container_input(StorageContainerInput {
            area_uuid: area.area_uuid.clone(),
            name: "Component Cabinet 1".to_string(),
            container_type: "cabinet".to_string(),
            grid_enabled: true,
            row_count: Some(8),
            column_count: Some(28),
            description: "Excel-style drawer grid".to_string(),
            ..StorageContainerInput::default()
        }));
    let organizer =
        create_storage_container(normalize_storage_container_input(StorageContainerInput {
            area_uuid: desk.area_uuid.clone(),
            name: "Organizer 1".to_string(),
            container_type: "organizer".to_string(),
            grid_enabled: true,
            row_count: Some(6),
            column_count: Some(6),
            ..StorageContainerInput::default()
        }));

    db.put_storage_area(&area)?;
    db.put_storage_area(&desk)?;
    db.put_storage_container(&cabinet)?;
    db.put_storage_container(&organizer)?;

    let mut next_id = 1_i64;
    let resistor = put_part(
        db,
        &mut next_id,
        PartInput {
            internal_part_number: "LAB-R-1K0".to_string(),
            category: "Passive".to_string(),
            subcategory: "Resistor".to_string(),
            manufacturer: "Stackpole Electronics".to_string(),
            manufacturer_part_number: "CF14JT1K00".to_string(),
            display_value: "1 kΩ".to_string(),
            mounting_type: "through_hole".to_string(),
            package_type: "axial".to_string(),
            description: "Carbon film resistor, 1/4 W".to_string(),
            attributes: attrs(&[
                ("tolerance", "5", "%"),
                ("powerRating", "0.25", "W"),
            ]),
            supplier: "DigiKey".to_string(),
            supplier_sku: "CF14JT1K00CT-ND".to_string(),
            supplier_packaging: "cut_tape".to_string(),
            product_url: "https://www.digikey.com/en/products/detail/stackpole-electronics-inc/CF14JT1K00/1741314".to_string(),
            default_unit_of_measure: "pcs".to_string(),
            reorder_point: Some(50.0),
            target_quantity: Some(250.0),
            part_status: "active".to_string(),
            ..PartInput::default()
        },
    )?;
    let capacitor = put_part(
        db,
        &mut next_id,
        PartInput {
            internal_part_number: "LAB-C-100N".to_string(),
            category: "Passive".to_string(),
            subcategory: "Capacitor".to_string(),
            manufacturer: "KEMET".to_string(),
            manufacturer_part_number: "C315C104M5U5TA".to_string(),
            display_value: "100 nF".to_string(),
            mounting_type: "through_hole".to_string(),
            package_type: "radial".to_string(),
            description: "MLCC ceramic, general purpose".to_string(),
            attributes: attrs(&[("capacitance", "100", "nF"), ("ratedVoltage", "50", "V")]),
            supplier: "DigiKey".to_string(),
            supplier_sku: "399-4151-ND".to_string(),
            supplier_packaging: "cut_tape".to_string(),
            default_unit_of_measure: "pcs".to_string(),
            reorder_point: Some(25.0),
            target_quantity: Some(100.0),
            part_status: "active".to_string(),
            ..PartInput::default()
        },
    )?;
    let _transistor = put_part(
        db,
        &mut next_id,
        PartInput {
            internal_part_number: "LAB-Q-3904".to_string(),
            category: "Semiconductor".to_string(),
            subcategory: "BJT".to_string(),
            manufacturer: "onsemi".to_string(),
            manufacturer_part_number: "2N3904BU".to_string(),
            display_value: "2N3904".to_string(),
            mounting_type: "through_hole".to_string(),
            package_type: "TO-92".to_string(),
            description: "NPN general-purpose switching transistor".to_string(),
            attributes: attrs(&[("deviceType", "NPN", ""), ("voltageRating", "40", "V")]),
            supplier: "DigiKey".to_string(),
            supplier_sku: "2N3904BU-ND".to_string(),
            supplier_packaging: "bag".to_string(),
            default_unit_of_measure: "pcs".to_string(),
            part_status: "active".to_string(),
            ..PartInput::default()
        },
    )?;
    let led = put_part(
        db,
        &mut next_id,
        PartInput {
            internal_part_number: "LAB-D-LED-RED".to_string(),
            category: "Optoelectronics".to_string(),
            subcategory: "LED".to_string(),
            manufacturer: "Kingbright".to_string(),
            manufacturer_part_number: "WP7113ID".to_string(),
            display_value: "Red 5 mm".to_string(),
            mounting_type: "through_hole".to_string(),
            package_type: "T-1 3/4".to_string(),
            description: "High-efficiency red indicator LED".to_string(),
            attributes: attrs(&[("color", "Red", ""), ("forwardVoltage", "2.0", "V")]),
            supplier: "DigiKey".to_string(),
            supplier_sku: "754-1264-ND".to_string(),
            supplier_packaging: "bag".to_string(),
            default_unit_of_measure: "pcs".to_string(),
            reorder_point: Some(20.0),
            target_quantity: Some(100.0),
            part_status: "active".to_string(),
            ..PartInput::default()
        },
    )?;
    let mcu = put_part(
        db,
        &mut next_id,
        PartInput {
            internal_part_number: "LAB-U-STM32".to_string(),
            category: "Semiconductor".to_string(),
            subcategory: "MCU".to_string(),
            manufacturer: "STMicroelectronics".to_string(),
            manufacturer_part_number: "STM32F103C8T6".to_string(),
            display_value: "STM32F103C8".to_string(),
            mounting_type: "smd".to_string(),
            package_type: "LQFP-48".to_string(),
            description: "ARM Cortex-M3 MCU, 64 KB flash".to_string(),
            attributes: attrs(&[("flash", "64", "KB"), ("core", "Cortex-M3", "")]),
            supplier: "Mouser".to_string(),
            supplier_sku: "511-STM32F103C8T6".to_string(),
            supplier_packaging: "tray".to_string(),
            default_unit_of_measure: "pcs".to_string(),
            reorder_point: Some(2.0),
            target_quantity: Some(10.0),
            part_status: "active".to_string(),
            ..PartInput::default()
        },
    )?;
    let connector = put_part(
        db,
        &mut next_id,
        PartInput {
            internal_part_number: "LAB-J-USB-C".to_string(),
            category: "Connector".to_string(),
            subcategory: "USB".to_string(),
            manufacturer: "Amphenol ICC".to_string(),
            manufacturer_part_number: "12401598E4#2A".to_string(),
            display_value: "USB-C receptacle".to_string(),
            mounting_type: "smd".to_string(),
            package_type: "SMT".to_string(),
            description: "USB Type-C 16-pin mid-mount receptacle".to_string(),
            attributes: attrs(&[("gender", "Receptacle", ""), ("ports", "1", "")]),
            supplier: "DigiKey".to_string(),
            supplier_sku: "609-5247-1-ND".to_string(),
            supplier_packaging: "reel_cut".to_string(),
            default_unit_of_measure: "pcs".to_string(),
            reorder_point: Some(5.0),
            target_quantity: Some(25.0),
            part_status: "active".to_string(),
            ..PartInput::default()
        },
    )?;

    put_placement(
        db,
        DemoPlacementSpec {
            part_uuid: &resistor.entry_uuid,
            container_uuid: &cabinet.container_uuid,
            column_index: Some(2),
            row_index: Some(6),
            quantity: 180.0,
            packaging: "bag",
            count_state: "counted",
        },
    )?;
    put_placement(
        db,
        DemoPlacementSpec {
            part_uuid: &resistor.entry_uuid,
            container_uuid: &organizer.container_uuid,
            column_index: Some(1),
            row_index: Some(3),
            quantity: 25.0,
            packaging: "bag",
            count_state: "uncounted",
        },
    )?;
    put_placement(
        db,
        DemoPlacementSpec {
            part_uuid: &capacitor.entry_uuid,
            container_uuid: &cabinet.container_uuid,
            column_index: Some(4),
            row_index: Some(2),
            quantity: 40.0,
            packaging: "bag",
            count_state: "uncounted",
        },
    )?;
    put_placement(
        db,
        DemoPlacementSpec {
            part_uuid: &led.entry_uuid,
            container_uuid: &cabinet.container_uuid,
            column_index: Some(8),
            row_index: Some(1),
            quantity: 12.0,
            packaging: "bag",
            count_state: "counted",
        },
    )?;
    put_placement(
        db,
        DemoPlacementSpec {
            part_uuid: &mcu.entry_uuid,
            container_uuid: &cabinet.container_uuid,
            column_index: Some(12),
            row_index: Some(3),
            quantity: 6.0,
            packaging: "tray",
            count_state: "counted",
        },
    )?;
    put_placement(
        db,
        DemoPlacementSpec {
            part_uuid: &connector.entry_uuid,
            container_uuid: &cabinet.container_uuid,
            column_index: Some(15),
            row_index: Some(5),
            quantity: 3.0,
            packaging: "reel_cut",
            count_state: "uncounted",
        },
    )?;

    db.set_next_entry_id(next_id)?;
    db.flush();
    Ok(())
}

fn put_part(
    db: &InventoryDb,
    next_id: &mut i64,
    input: PartInput,
) -> CommandResult<crate::modules::te_lab_components::catalog_model::Part> {
    let part = create_part(*next_id, normalize_part_input(input));
    db.put_part(&part)?;
    *next_id += 1;
    Ok(part)
}

struct DemoPlacementSpec<'a> {
    part_uuid: &'a str,
    container_uuid: &'a str,
    column_index: Option<u32>,
    row_index: Option<u32>,
    quantity: f64,
    packaging: &'a str,
    count_state: &'a str,
}

fn put_placement(db: &InventoryDb, spec: DemoPlacementSpec<'_>) -> CommandResult<()> {
    let counted = spec.count_state == "counted";
    let placement = create_stock_placement(normalize_stock_placement_input(
        StockPlacementInput {
            part_uuid: spec.part_uuid.to_string(),
            container_uuid: spec.container_uuid.to_string(),
            column_index: spec.column_index,
            row_index: spec.row_index,
            quantity: spec.quantity,
            unit_of_measure: "pcs".to_string(),
            packaging: spec.packaging.to_string(),
            condition: "new".to_string(),
            count_state: spec.count_state.to_string(),
            last_counted_at: counted.then(now_timestamp),
            last_counted_by: if counted {
                "demo-seed".to_string()
            } else {
                String::new()
            },
            ..StockPlacementInput::default()
        },
        Some("pcs"),
    ));
    db.put_stock_placement(&placement)?;
    Ok(())
}

fn attrs(entries: &[(&str, &str, &str)]) -> ComponentAttributes {
    let mut attributes = ComponentAttributes::new();
    for (key, value, unit) in entries {
        attributes.insert(
            (*key).to_string(),
            ComponentAttributeValue {
                value: (*value).to_string(),
                unit: (*unit).to_string(),
            },
        );
    }
    attributes
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs};
    use uuid::Uuid;

    use crate::modules::te_lab_components::catalog_migration::ensure_catalog_initialized;

    #[test]
    fn seeds_six_demo_parts_once_on_empty_catalog() {
        let db = test_db();
        ensure_catalog_initialized(&db).unwrap();
        seed_demo_catalog_if_empty(&db).unwrap();
        assert_eq!(db.load_parts().unwrap().len(), 6);
        assert_eq!(db.load_stock_placements().unwrap().len(), 6);
        assert_eq!(db.load_storage_areas().unwrap().len(), 2);

        // Second call is a no-op when parts already exist.
        seed_demo_catalog_if_empty(&db).unwrap();
        assert_eq!(db.load_parts().unwrap().len(), 6);
    }

    fn test_db() -> InventoryDb {
        let root = env::temp_dir().join(format!("lab-demo-seed-{}", Uuid::new_v4().simple()));
        fs::create_dir_all(&root).unwrap();
        InventoryDb::open_at(root.join("te-lab-components.feox")).unwrap()
    }
}
