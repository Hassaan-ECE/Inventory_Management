use serde::{Deserialize, Serialize};

use crate::modules::te_lab_components::{
    catalog_migration::ensure_catalog_initialized,
    catalog_model::{
        create_part, create_stock_placement, grid_coordinate_label, normalize_part_input,
        normalize_stock_placement_input, normalized_lookup, update_part, update_stock_placement,
        validate_part_input, validate_stock_placement_input, CatalogMutationResult, Part,
        PartInput, StockPlacement, StockPlacementInput, StorageAreaInput, StorageContainer,
        StorageContainerInput,
    },
    catalog_mutations::{
        create_storage_area_in_store, create_storage_container_in_store, ensure_distinct_placement,
        ensure_unique_part_number,
    },
    catalog_sync::{self, queued_local_status},
    model::{CommandResult, InventorySharedStatus},
    store::InventoryDb,
};

pub(crate) const SIMPLE_AREA_NAME: &str = "TE Lab";
pub(crate) const SIMPLE_CONTAINER_NAME: &str = "Shelf";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub(crate) struct SimpleComponentInput {
    pub part: PartInput,
    pub quantity: u32,
    pub location: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SimpleComponentValue {
    pub part: Part,
    pub placement: Option<StockPlacement>,
}

enum SimpleWorkflowFailurePoint {
    AfterPartWrite,
}

pub(crate) fn create_simple_component_in_store(
    input: SimpleComponentInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<SimpleComponentValue>> {
    save_simple_component_with_failure(None, input, db, None)
}

pub(crate) fn update_simple_component_in_store(
    part_id: &str,
    input: SimpleComponentInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<SimpleComponentValue>> {
    save_simple_component_with_failure(Some(part_id), input, db, None)
}

fn save_simple_component_with_failure(
    part_id: Option<&str>,
    input: SimpleComponentInput,
    db: &InventoryDb,
    failure: Option<SimpleWorkflowFailurePoint>,
) -> CommandResult<CatalogMutationResult<SimpleComponentValue>> {
    ensure_catalog_initialized(db)?;

    let location = normalize_location(&input.location, input.quantity)?;
    let quantity = input.quantity as f64;

    let mut part_input = normalize_part_input(input.part);
    part_input.default_unit_of_measure = "pcs".to_string();
    if part_id.is_none() {
        part_input.mounting_type = "through_hole".to_string();
    }
    validate_part_input(&part_input)?;

    let existing_part = match part_id {
        Some(id) => Some(
            db.find_part(id)?
                .ok_or_else(|| "The selected part could not be found.".to_string())?,
        ),
        None => None,
    };

    ensure_unique_part_number(
        db,
        &part_input.internal_part_number,
        existing_part.as_ref().map(|part| part.entry_uuid.as_str()),
    )?;

    let prior_placements = match &existing_part {
        Some(part) => db.load_stock_placements_for_part(&part.entry_uuid)?,
        None => Vec::new(),
    };
    if existing_part.is_some() {
        refuse_if_review_required(db, &prior_placements)?;
    }

    let active_placement = prior_placements
        .iter()
        .find(|placement| !placement.archived)
        .cloned();

    // Resolve/create TE Lab / Shelf before any part/placement write so create helpers
    // never flush() a durable part without its placement. Leftover empty shelf is OK.
    let shelf_for_write = if location.is_empty() {
        None
    } else if let Some(existing) = &active_placement {
        let existing_container = require_container(db, &existing.container_uuid)?;
        let current_location = placement_short_location(existing, &existing_container);
        if current_location == location {
            let mut next_input = placement_input_from(existing);
            next_input.quantity = quantity;
            next_input.unit_of_measure = "pcs".to_string();
            let next_input = normalize_stock_placement_input(next_input, Some("pcs"));
            validate_stock_placement_input(&next_input, &existing_container)?;
            None
        } else {
            let shelf = ensure_simple_shelf(db)?;
            let mut next_input = placement_input_from(existing);
            next_input.container_uuid = shelf.container_uuid.clone();
            next_input.column_index = None;
            next_input.row_index = None;
            next_input.freeform_position = location.clone();
            next_input.quantity = quantity;
            next_input.unit_of_measure = "pcs".to_string();
            next_input.archived = false;
            let next_input = normalize_stock_placement_input(next_input, Some("pcs"));
            validate_stock_placement_input(&next_input, &shelf)?;
            Some(shelf)
        }
    } else {
        let shelf = ensure_simple_shelf(db)?;
        let placement_input = normalize_stock_placement_input(
            StockPlacementInput {
                part_uuid: String::new(), // filled after part id is known
                container_uuid: shelf.container_uuid.clone(),
                freeform_position: location.clone(),
                quantity,
                unit_of_measure: "pcs".to_string(),
                ..StockPlacementInput::default()
            },
            Some("pcs"),
        );
        // part_uuid empty fails validate — validate container geometry only via a temp part uuid
        let mut check = placement_input;
        check.part_uuid = "pending".to_string();
        validate_stock_placement_input(&check, &shelf)?;
        Some(shelf)
    };

    if location.is_empty() {
        if let Some(existing) = &active_placement {
            let container = require_container(db, &existing.container_uuid)?;
            let mut next_input = placement_input_from(existing);
            next_input.quantity = 0.0;
            next_input.archived = true;
            next_input.unit_of_measure = "pcs".to_string();
            let next_input = normalize_stock_placement_input(next_input, Some("pcs"));
            validate_stock_placement_input(&next_input, &container)?;
        }
    }

    let next_entry_id_before = db.next_entry_id()?;
    let sync_backup = db.backup_sync_state()?;
    let part_before = existing_part.clone();
    let placements_before = prior_placements.clone();

    let mut created_part: Option<Part> = None;
    let mut wrote_part = false;
    let mut placement_written: Option<StockPlacement> = None;

    let result = (|| -> CommandResult<CatalogMutationResult<SimpleComponentValue>> {
        let part = match &existing_part {
            Some(existing) => {
                let updated = update_part(existing.clone(), part_input.clone());
                let changed_fields = catalog_sync::changed_fields(existing, &updated);
                if !changed_fields.is_empty() {
                    db.put_part(&updated)?;
                    wrote_part = true;
                    catalog_sync::queue_upsert_operation(
                        db,
                        updated.clone().into(),
                        changed_fields,
                        Some(existing.updated_at.clone()),
                    )?;
                }
                updated
            }
            None => {
                let id = next_entry_id_before;
                let part = create_part(id, part_input.clone());
                db.put_part(&part)?;
                wrote_part = true;
                created_part = Some(part.clone());
                db.set_next_entry_id(id + 1)?;
                catalog_sync::queue_upsert_operation(
                    db,
                    part.clone().into(),
                    Vec::new(),
                    None,
                )?;
                part
            }
        };

        if matches!(failure, Some(SimpleWorkflowFailurePoint::AfterPartWrite)) {
            return Err("injected simple-workflow failure".to_string());
        }

        let placement = match (&active_placement, location.is_empty()) {
            (None, true) => None,
            (Some(existing), true) => {
                // Archive existing placement (qty 0 + blank location).
                let mut next_input = placement_input_from(existing);
                next_input.quantity = 0.0;
                next_input.archived = true;
                next_input.unit_of_measure = "pcs".to_string();
                let container = require_container(db, &existing.container_uuid)?;
                let next_input = normalize_stock_placement_input(next_input, Some("pcs"));
                validate_stock_placement_input(&next_input, &container)?;
                let archived = update_stock_placement(existing.clone(), next_input);
                let changed_fields = catalog_sync::changed_fields(existing, &archived);
                if !changed_fields.is_empty() {
                    db.put_stock_placement(&archived)?;
                    placement_written = Some(archived.clone());
                    catalog_sync::queue_upsert_operation(
                        db,
                        archived.clone().into(),
                        changed_fields,
                        Some(existing.updated_at.clone()),
                    )?;
                }
                None
            }
            (None, false) => {
                let shelf = shelf_for_write
                    .as_ref()
                    .expect("shelf resolved before part write for non-empty location");
                let placement_input = normalize_stock_placement_input(
                    StockPlacementInput {
                        part_uuid: part.entry_uuid.clone(),
                        container_uuid: shelf.container_uuid.clone(),
                        freeform_position: location.clone(),
                        quantity,
                        unit_of_measure: "pcs".to_string(),
                        ..StockPlacementInput::default()
                    },
                    Some("pcs"),
                );
                validate_stock_placement_input(&placement_input, shelf)?;
                let placement = create_stock_placement(placement_input);
                ensure_distinct_placement(db, &placement, None)?;
                db.put_stock_placement(&placement)?;
                placement_written = Some(placement.clone());
                catalog_sync::queue_upsert_operation(
                    db,
                    placement.clone().into(),
                    Vec::new(),
                    None,
                )?;
                Some(placement)
            }
            (Some(existing), false) => {
                let existing_container = require_container(db, &existing.container_uuid)?;
                let current_location = placement_short_location(existing, &existing_container);
                let next = if current_location == location {
                    let mut next_input = placement_input_from(existing);
                    next_input.quantity = quantity;
                    next_input.unit_of_measure = "pcs".to_string();
                    next_input.archived = false;
                    let next_input = normalize_stock_placement_input(next_input, Some("pcs"));
                    validate_stock_placement_input(&next_input, &existing_container)?;
                    update_stock_placement(existing.clone(), next_input)
                } else {
                    let shelf = shelf_for_write
                        .as_ref()
                        .expect("shelf resolved before part write when relocating");
                    let mut next_input = placement_input_from(existing);
                    next_input.container_uuid = shelf.container_uuid.clone();
                    next_input.column_index = None;
                    next_input.row_index = None;
                    next_input.freeform_position = location.clone();
                    next_input.quantity = quantity;
                    next_input.unit_of_measure = "pcs".to_string();
                    next_input.archived = false;
                    let next_input = normalize_stock_placement_input(next_input, Some("pcs"));
                    validate_stock_placement_input(&next_input, shelf)?;
                    let updated = update_stock_placement(existing.clone(), next_input);
                    ensure_distinct_placement(db, &updated, Some(&existing.placement_uuid))?;
                    updated
                };
                let changed_fields = catalog_sync::changed_fields(existing, &next);
                if !changed_fields.is_empty() {
                    db.put_stock_placement(&next)?;
                    placement_written = Some(next.clone());
                    catalog_sync::queue_upsert_operation(
                        db,
                        next.clone().into(),
                        changed_fields,
                        Some(existing.updated_at.clone()),
                    )?;
                }
                Some(next)
            }
        };

        Ok(mutation_result(
            SimpleComponentValue { part, placement },
            if part_id.is_some() {
                "Simple component updated."
            } else {
                "Simple component created."
            },
            db,
        ))
    })();

    match result {
        Ok(value) => {
            db.flush();
            Ok(value)
        }
        Err(error) => {
            // Restore prior records / indexes / next id / sync after any post-write failure.
            if wrote_part {
                if let Some(existing) = &part_before {
                    let _ = db.put_part(existing);
                } else if let Some(created) = &created_part {
                    let _ = db.delete_part(created);
                }
            }
            let _ = db.set_next_entry_id(next_entry_id_before);

            // Restore all prior placements and remove any newly created ones.
            let prior_uuids: std::collections::HashSet<String> = placements_before
                .iter()
                .map(|placement| placement.placement_uuid.clone())
                .collect();
            if let Some(written) = &placement_written {
                if !prior_uuids.contains(&written.placement_uuid) {
                    let _ = db.delete_stock_placement(written);
                }
            }
            for placement in &placements_before {
                let _ = db.put_stock_placement(placement);
            }

            let _ = db.restore_sync_state(sync_backup);
            db.flush();
            Err(error)
        }
    }
}

fn normalize_location(value: &str, quantity: u32) -> CommandResult<String> {
    let location = value.trim().to_ascii_uppercase();
    if location.is_empty() {
        if quantity == 0 {
            return Ok(location);
        }
        return Err("Location is required when quantity is greater than zero.".to_string());
    }
    if !is_simple_location_code(&location) {
        return Err("Use a shelf code such as A1, B3, or M15.".to_string());
    }
    Ok(location)
}

fn is_simple_location_code(location: &str) -> bool {
    let mut chars = location.chars();
    matches!(chars.next(), Some('A'..='Z'))
        && chars
            .clone()
            .next()
            .is_some_and(|value| ('1'..='9').contains(&value))
        && chars.all(|value| value.is_ascii_digit())
}

fn is_grid_location_code(location: &str) -> bool {
    let bytes = location.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_uppercase() {
        i += 1;
    }
    if i == 0 || i >= bytes.len() {
        return false;
    }
    if !(b'1'..=b'9').contains(&bytes[i]) {
        return false;
    }
    i += 1;
    while i < bytes.len() {
        if !bytes[i].is_ascii_digit() {
            return false;
        }
        i += 1;
    }
    true
}

fn ensure_simple_shelf(db: &InventoryDb) -> CommandResult<StorageContainer> {
    let area = if let Some(area) = db.find_storage_area_by_name(SIMPLE_AREA_NAME)? {
        area
    } else {
        create_storage_area_in_store(
            StorageAreaInput {
                name: SIMPLE_AREA_NAME.to_string(),
                area_type: "lab".to_string(),
                ..StorageAreaInput::default()
            },
            db,
        )?
        .value
    };

    if let Some(container) =
        db.find_storage_container_by_name(&area.area_uuid, SIMPLE_CONTAINER_NAME)?
    {
        return Ok(container);
    }

    Ok(create_storage_container_in_store(
        StorageContainerInput {
            area_uuid: area.area_uuid,
            name: SIMPLE_CONTAINER_NAME.to_string(),
            container_type: "shelf".to_string(),
            grid_enabled: false,
            ..StorageContainerInput::default()
        },
        db,
    )?
    .value)
}

fn refuse_if_review_required(
    db: &InventoryDb,
    placements: &[StockPlacement],
) -> CommandResult<()> {
    if requires_advanced_review(db, placements)? {
        return Err(
            "This component requires advanced location review before simple edits.".to_string(),
        );
    }
    Ok(())
}

fn requires_advanced_review(
    db: &InventoryDb,
    placements: &[StockPlacement],
) -> CommandResult<bool> {
    let active: Vec<&StockPlacement> = placements
        .iter()
        .filter(|placement| !placement.archived)
        .collect();
    if active.is_empty() {
        return Ok(false);
    }
    if active.len() > 1 {
        return Ok(true);
    }
    let only = active[0];
    if normalized_lookup(&only.unit_of_measure) != "pcs" {
        return Ok(true);
    }
    let container = require_container(db, &only.container_uuid)?;
    let is_grid = is_grid_cell(only, &container);
    if only.quantity == 0.0 && !is_grid && only.freeform_position.trim().is_empty() {
        return Ok(false);
    }
    let location = placement_short_location(only, &container);
    if is_grid {
        Ok(!is_grid_location_code(&location) && !is_simple_location_code(&location))
    } else {
        Ok(!is_simple_location_code(&location))
    }
}

fn is_grid_cell(placement: &StockPlacement, container: &StorageContainer) -> bool {
    container.grid_enabled && placement.row_index.is_some() && placement.column_index.is_some()
}

fn placement_short_location(
    placement: &StockPlacement,
    container: &StorageContainer,
) -> String {
    if is_grid_cell(placement, container) {
        return grid_coordinate_label(
            container,
            placement.row_index.unwrap_or(0),
            placement.column_index.unwrap_or(0),
        );
    }
    placement.freeform_position.trim().to_ascii_uppercase()
}

fn require_container(db: &InventoryDb, container_uuid: &str) -> CommandResult<StorageContainer> {
    db.find_storage_container(container_uuid)?
        .ok_or_else(|| "The selected storage container could not be found.".to_string())
}

fn placement_input_from(placement: &StockPlacement) -> StockPlacementInput {
    StockPlacementInput {
        part_uuid: placement.part_uuid.clone(),
        container_uuid: placement.container_uuid.clone(),
        column_index: placement.column_index,
        row_index: placement.row_index,
        freeform_position: placement.freeform_position.clone(),
        quantity: placement.quantity,
        unit_of_measure: placement.unit_of_measure.clone(),
        packaging: placement.packaging.clone(),
        lot_code: placement.lot_code.clone(),
        date_code: placement.date_code.clone(),
        condition: placement.condition.clone(),
        count_state: placement.count_state.clone(),
        last_counted_at: placement.last_counted_at.clone(),
        last_counted_by: placement.last_counted_by.clone(),
        notes: placement.notes.clone(),
        archived: placement.archived,
    }
}

fn mutation_result(
    value: SimpleComponentValue,
    message: impl Into<String>,
    db: &InventoryDb,
) -> CatalogMutationResult<SimpleComponentValue> {
    CatalogMutationResult {
        value,
        message: message.into(),
        mutation_mode: "local".to_string(),
        shared: local_shared_status(db),
    }
}

fn local_shared_status(db: &InventoryDb) -> InventorySharedStatus {
    queued_local_status(db)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::te_lab_components::{
        catalog_migration::ensure_catalog_initialized,
        catalog_model::{StorageAreaInput, StorageContainerInput},
        catalog_mutations::{
            create_part_in_store, create_stock_placement_in_store, create_storage_area_in_store,
            create_storage_container_in_store,
        },
        catalog_sync,
    };
    use std::{env, fs};
    use uuid::Uuid;

    #[test]
    fn creates_simple_part_and_one_piece_placement() {
        let db = ready_catalog_db();
        let result = create_simple_component_in_store(
            simple_input("Capacitor", "100 nF", 40, " b3 "),
            &db,
        )
        .unwrap();

        assert_eq!(result.value.part.subcategory, "Capacitor");
        assert_eq!(result.value.part.mounting_type, "through_hole");
        let placement = result.value.placement.unwrap();
        assert_eq!(placement.quantity, 40.0);
        assert_eq!(placement.unit_of_measure, "pcs");
        assert_eq!(placement.freeform_position, "B3");
        assert_eq!(
            db.load_stock_placements_for_part(&result.value.part.entry_uuid)
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn updates_the_existing_simple_placement_without_replacing_its_uuid() {
        let db = ready_catalog_db();
        let created =
            create_simple_component_in_store(simple_input("Resistor", "1 kΩ", 5, "A1"), &db)
                .unwrap();
        let placement_uuid = created.value.placement.unwrap().placement_uuid;

        let updated = update_simple_component_in_store(
            &created.value.part.entry_uuid,
            simple_input("Resistor", "1 kΩ", 9, "A2"),
            &db,
        )
        .unwrap();

        let placement = updated.value.placement.unwrap();
        assert_eq!(placement.placement_uuid, placement_uuid);
        assert_eq!(placement.freeform_position, "A2");
        assert_eq!(placement.quantity, 9.0);
    }

    #[test]
    fn refuses_to_simplify_multiple_or_non_piece_placements() {
        let (db, part_uuid) = catalog_with_advanced_placements();
        let before = workflow_snapshot(&db);
        let error = update_simple_component_in_store(
            &part_uuid,
            simple_input("Resistor", "1 kΩ", 9, "A2"),
            &db,
        )
        .unwrap_err();

        assert!(error.contains("advanced location review"));
        assert_eq!(workflow_snapshot(&db), before);
    }

    #[test]
    fn restores_part_placement_next_id_and_sync_state_after_second_write_failure() {
        let db = ready_catalog_db();
        let before_next_id = db.next_entry_id().unwrap();
        let error = save_simple_component_with_failure(
            None,
            simple_input("Capacitor", "1 µF", 2, "A1"),
            &db,
            Some(SimpleWorkflowFailurePoint::AfterPartWrite),
        )
        .unwrap_err();

        assert!(error.contains("injected simple-workflow failure"));
        // Part/placement/next-id must roll back. Empty TE Lab/Shelf leftovers (and their
        // sync ops) are allowed when the shelf was resolved before the composite write.
        assert!(db.load_parts().unwrap().is_empty());
        assert!(db.load_stock_placements().unwrap().is_empty());
        assert_eq!(db.next_entry_id().unwrap(), before_next_id);
    }

    #[test]
    fn resolves_shelf_before_part_write_so_failure_leaves_empty_shelf_only() {
        let db = ready_catalog_db();
        let error = save_simple_component_with_failure(
            None,
            simple_input("Capacitor", "1 µF", 2, "A1"),
            &db,
            Some(SimpleWorkflowFailurePoint::AfterPartWrite),
        )
        .unwrap_err();

        assert!(error.contains("injected simple-workflow failure"));
        assert!(db.load_parts().unwrap().is_empty());
        // Shelf must be resolved before put_part so create helpers never flush a bare part.
        assert!(db
            .find_storage_area_by_name(SIMPLE_AREA_NAME)
            .unwrap()
            .is_some());
        let area = db.find_storage_area_by_name(SIMPLE_AREA_NAME).unwrap().unwrap();
        assert!(db
            .find_storage_container_by_name(&area.area_uuid, SIMPLE_CONTAINER_NAME)
            .unwrap()
            .is_some());
    }

    #[test]
    fn restores_composite_after_part_write_failure_when_shelf_already_exists() {
        let db = ready_catalog_db();
        create_simple_component_in_store(simple_input("Resistor", "1 kΩ", 5, "A1"), &db).unwrap();
        let before = workflow_snapshot(&db);

        let error = save_simple_component_with_failure(
            None,
            simple_input("Capacitor", "1 µF", 2, "B3"),
            &db,
            Some(SimpleWorkflowFailurePoint::AfterPartWrite),
        )
        .unwrap_err();

        assert!(error.contains("injected simple-workflow failure"));
        assert_eq!(workflow_snapshot(&db), before);
        assert_eq!(db.load_parts().unwrap().len(), 1);
    }

    #[test]
    fn create_with_zero_quantity_and_blank_location_has_no_placement() {
        let db = ready_catalog_db();
        let result =
            create_simple_component_in_store(simple_input("Resistor", "10 kΩ", 0, ""), &db)
                .unwrap();

        assert!(result.value.placement.is_none());
        assert!(db
            .load_stock_placements_for_part(&result.value.part.entry_uuid)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn update_archives_placement_when_quantity_and_location_cleared() {
        let db = ready_catalog_db();
        let created =
            create_simple_component_in_store(simple_input("Resistor", "1 kΩ", 5, "A1"), &db)
                .unwrap();
        let placement_uuid = created
            .value
            .placement
            .as_ref()
            .unwrap()
            .placement_uuid
            .clone();

        let updated = update_simple_component_in_store(
            &created.value.part.entry_uuid,
            simple_input("Resistor", "1 kΩ", 0, ""),
            &db,
        )
        .unwrap();

        assert!(updated.value.placement.is_none());
        let placements = db
            .load_stock_placements_for_part(&created.value.part.entry_uuid)
            .unwrap();
        assert_eq!(placements.len(), 1);
        assert_eq!(placements[0].placement_uuid, placement_uuid);
        assert!(placements[0].archived);
    }

    #[test]
    fn update_grid_placement_same_location_updates_quantity_only() {
        let db = ready_catalog_db();
        let part = create_part_in_store(
            PartInput {
                category: "Passive".to_string(),
                subcategory: "Resistor".to_string(),
                display_value: "1 kΩ".to_string(),
                mounting_type: "through_hole".to_string(),
                description: "Resistor 1 kΩ".to_string(),
                default_unit_of_measure: "pcs".to_string(),
                part_status: "active".to_string(),
                ..PartInput::default()
            },
            &db,
        )
        .unwrap()
        .value;
        let area = create_storage_area_in_store(
            StorageAreaInput {
                name: "Main Lab".to_string(),
                area_type: "lab".to_string(),
                ..StorageAreaInput::default()
            },
            &db,
        )
        .unwrap()
        .value;
        let container = create_storage_container_in_store(
            StorageContainerInput {
                area_uuid: area.area_uuid,
                name: "Cabinet".to_string(),
                container_type: "cabinet".to_string(),
                grid_enabled: true,
                row_count: Some(4),
                column_count: Some(4),
                ..StorageContainerInput::default()
            },
            &db,
        )
        .unwrap()
        .value;
        let placement = create_stock_placement_in_store(
            StockPlacementInput {
                part_uuid: part.entry_uuid.clone(),
                container_uuid: container.container_uuid.clone(),
                row_index: Some(0),
                column_index: Some(0),
                quantity: 3.0,
                unit_of_measure: "pcs".to_string(),
                ..StockPlacementInput::default()
            },
            &db,
        )
        .unwrap()
        .value;

        let updated = update_simple_component_in_store(
            &part.entry_uuid,
            simple_input("Resistor", "1 kΩ", 12, "A1"),
            &db,
        )
        .unwrap();

        let next = updated.value.placement.unwrap();
        assert_eq!(next.placement_uuid, placement.placement_uuid);
        assert_eq!(next.container_uuid, container.container_uuid);
        assert_eq!(next.row_index, Some(0));
        assert_eq!(next.column_index, Some(0));
        assert_eq!(next.quantity, 12.0);
        assert_eq!(next.unit_of_measure, "pcs");
    }

    fn ready_catalog_db() -> InventoryDb {
        let root = env::temp_dir().join(format!("simple-workflow-{}", Uuid::new_v4().simple()));
        fs::create_dir_all(&root).unwrap();
        let db = InventoryDb::open_at(root.join("te-lab-components.feox")).unwrap();
        ensure_catalog_initialized(&db).unwrap();
        db
    }

    fn simple_input(
        component_type: &str,
        display_value: &str,
        quantity: u32,
        location: &str,
    ) -> SimpleComponentInput {
        SimpleComponentInput {
            part: PartInput {
                category: "Passive".to_string(),
                subcategory: component_type.to_string(),
                display_value: display_value.to_string(),
                mounting_type: "through_hole".to_string(),
                description: format!("{component_type} {display_value}"),
                default_unit_of_measure: "pcs".to_string(),
                part_status: "active".to_string(),
                ..PartInput::default()
            },
            quantity,
            location: location.to_string(),
        }
    }

    fn catalog_with_advanced_placements() -> (InventoryDb, String) {
        let db = ready_catalog_db();
        let part = create_part_in_store(
            PartInput {
                category: "Passive".to_string(),
                subcategory: "Resistor".to_string(),
                display_value: "1 kΩ".to_string(),
                mounting_type: "through_hole".to_string(),
                description: "Resistor 1 kΩ".to_string(),
                default_unit_of_measure: "pcs".to_string(),
                part_status: "active".to_string(),
                ..PartInput::default()
            },
            &db,
        )
        .unwrap()
        .value;
        let area = create_storage_area_in_store(
            StorageAreaInput {
                name: "Advanced Lab".to_string(),
                area_type: "lab".to_string(),
                ..StorageAreaInput::default()
            },
            &db,
        )
        .unwrap()
        .value;
        let container = create_storage_container_in_store(
            StorageContainerInput {
                area_uuid: area.area_uuid,
                name: "Bins".to_string(),
                container_type: "bin".to_string(),
                grid_enabled: false,
                ..StorageContainerInput::default()
            },
            &db,
        )
        .unwrap()
        .value;
        create_stock_placement_in_store(
            StockPlacementInput {
                part_uuid: part.entry_uuid.clone(),
                container_uuid: container.container_uuid.clone(),
                freeform_position: "A1".to_string(),
                quantity: 2.0,
                unit_of_measure: "pcs".to_string(),
                ..StockPlacementInput::default()
            },
            &db,
        )
        .unwrap();
        create_stock_placement_in_store(
            StockPlacementInput {
                part_uuid: part.entry_uuid.clone(),
                container_uuid: container.container_uuid,
                freeform_position: "A2".to_string(),
                quantity: 3.0,
                unit_of_measure: "pcs".to_string(),
                ..StockPlacementInput::default()
            },
            &db,
        )
        .unwrap();
        (db, part.entry_uuid)
    }

    fn workflow_snapshot(db: &InventoryDb) -> String {
        let parts = db.load_parts().unwrap();
        let areas = db.load_storage_areas().unwrap();
        let containers = db.load_storage_containers().unwrap();
        let placements = db.load_stock_placements().unwrap();
        let next_entry_id = db.next_entry_id().unwrap();
        let shared = catalog_sync::queued_local_status(db);
        serde_json::to_string(&(
            parts,
            areas,
            containers,
            placements,
            next_entry_id,
            shared,
        ))
        .unwrap()
    }
}
