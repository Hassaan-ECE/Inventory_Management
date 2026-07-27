use crate::modules::te_lab_components::{
    catalog_migration::ensure_catalog_initialized,
    catalog_model::{
        create_part, create_stock_placement, create_storage_area, create_storage_container,
        indistinguishable_placement_key, normalize_part_input, normalize_stock_placement_input,
        normalize_storage_area_input, normalize_storage_container_input, normalized_lookup,
        normalized_part_number, timestamp_now, update_part, update_stock_placement,
        update_storage_area, update_storage_container, validate_part_input,
        validate_stock_placement_input, validate_storage_area_input,
        validate_storage_container_input, CatalogDeleteResult, CatalogMutationResult, Part,
        PartInput, StockCountInput, StockMoveInput, StockPlacement, StockPlacementInput,
        StorageArea, StorageAreaInput, StorageContainer, StorageContainerInput,
    },
    catalog_sync::{self, CatalogEntityType},
    model::{CommandResult, InventorySharedStatus},
    store::InventoryDb,
};

pub(crate) fn create_part_in_store(
    input: PartInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<Part>> {
    require_catalog_ready(db)?;
    let input = normalize_part_input(input);
    validate_part_input(&input)?;
    ensure_unique_part_number(db, &input.internal_part_number, None)?;

    let id = db.next_entry_id()?;
    let part = create_part(id, input);
    let sync_backup = db.backup_sync_state()?;
    db.put_part(&part)?;
    if let Err(error) = db.set_next_entry_id(id + 1) {
        let _ = db.delete_part(&part);
        db.flush();
        return Err(error);
    }
    if let Err(error) =
        catalog_sync::queue_upsert_operation(db, part.clone().into(), Vec::new(), None)
    {
        let _ = db.delete_part(&part);
        let _ = db.set_next_entry_id(id);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(mutation_result(
        part,
        "Part added to the Lab Components catalog.",
        db,
    ))
}

pub(crate) fn update_part_in_store(
    part_id: &str,
    input: PartInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<Part>> {
    require_catalog_ready(db)?;
    let input = normalize_part_input(input);
    validate_part_input(&input)?;
    let existing = db
        .find_part(part_id)?
        .ok_or_else(|| "The selected part could not be found.".to_string())?;
    ensure_unique_part_number(
        db,
        &input.internal_part_number,
        Some(existing.entry_uuid.as_str()),
    )?;
    let part = update_part(existing.clone(), input);
    let changed_fields = catalog_sync::changed_fields(&existing, &part);
    if changed_fields.is_empty() {
        return Ok(mutation_result(
            existing,
            "Part was already up to date.",
            db,
        ));
    }
    let sync_backup = db.backup_sync_state()?;
    db.put_part(&part)?;
    if let Err(error) = catalog_sync::queue_upsert_operation(
        db,
        part.clone().into(),
        changed_fields,
        Some(existing.updated_at.clone()),
    ) {
        let _ = db.put_part(&existing);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(mutation_result(
        part,
        "Part updated in the Lab Components catalog.",
        db,
    ))
}

pub(crate) fn delete_part_in_store(
    part_id: &str,
    db: &InventoryDb,
) -> CommandResult<CatalogDeleteResult> {
    require_catalog_ready(db)?;
    let part = db
        .find_part(part_id)?
        .ok_or_else(|| "The selected part could not be found.".to_string())?;
    if !part.archived {
        return Err(
            "Archive the part before deleting it permanently.".to_string(),
        );
    }

    // Permanent delete removes remaining stock placements with the part so the
    // archive "Delete Part" confirm does not strand on leftover placement rows.
    let placements = db.load_stock_placements_for_part(&part.entry_uuid)?;
    let sync_backup = db.backup_sync_state()?;
    let deleted_at = timestamp_now();

    for placement in &placements {
        db.delete_stock_placement(placement)?;
        if let Err(error) = catalog_sync::queue_delete_operation(
            db,
            CatalogEntityType::StockPlacement,
            &placement.placement_uuid,
            deleted_at.clone(),
            Some(placement.updated_at.clone()),
        ) {
            for restored in &placements {
                let _ = db.put_stock_placement(restored);
            }
            let _ = db.restore_sync_state(sync_backup);
            db.flush();
            return Err(error);
        }
    }

    db.delete_part(&part)?;
    if let Err(error) = catalog_sync::queue_delete_operation(
        db,
        CatalogEntityType::Part,
        &part.entry_uuid,
        deleted_at,
        Some(part.updated_at.clone()),
    ) {
        let _ = db.put_part(&part);
        for restored in &placements {
            let _ = db.put_stock_placement(restored);
        }
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    let message = if placements.is_empty() {
        "Part deleted from the Lab Components catalog.".to_string()
    } else {
        format!(
            "Part deleted from the Lab Components catalog ({} stock placement{} removed).",
            placements.len(),
            if placements.len() == 1 { "" } else { "s" }
        )
    };
    Ok(delete_result(part.entry_uuid, &message, db))
}

pub(crate) fn create_storage_area_in_store(
    input: StorageAreaInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<StorageArea>> {
    require_catalog_ready(db)?;
    let input = normalize_storage_area_input(input);
    validate_storage_area_input(&input)?;
    ensure_unique_area_name(db, &input.name, None, input.archived)?;
    let area = create_storage_area(input);
    let sync_backup = db.backup_sync_state()?;
    db.put_storage_area(&area)?;
    if let Err(error) =
        catalog_sync::queue_upsert_operation(db, area.clone().into(), Vec::new(), None)
    {
        let _ = db.delete_storage_area(&area);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(mutation_result(area, "Storage area added.", db))
}

pub(crate) fn update_storage_area_in_store(
    area_uuid: &str,
    input: StorageAreaInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<StorageArea>> {
    require_catalog_ready(db)?;
    let input = normalize_storage_area_input(input);
    validate_storage_area_input(&input)?;
    let existing = db
        .find_storage_area(area_uuid)?
        .ok_or_else(|| "The selected storage area could not be found.".to_string())?;
    ensure_unique_area_name(db, &input.name, Some(&existing.area_uuid), input.archived)?;
    let area = update_storage_area(existing.clone(), input);
    let changed_fields = catalog_sync::changed_fields(&existing, &area);
    if changed_fields.is_empty() {
        return Ok(mutation_result(
            existing,
            "Storage area was already up to date.",
            db,
        ));
    }
    let sync_backup = db.backup_sync_state()?;
    db.put_storage_area(&area)?;
    if let Err(error) = catalog_sync::queue_upsert_operation(
        db,
        area.clone().into(),
        changed_fields,
        Some(existing.updated_at.clone()),
    ) {
        let _ = db.put_storage_area(&existing);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(mutation_result(area, "Storage area updated.", db))
}

pub(crate) fn delete_storage_area_in_store(
    area_uuid: &str,
    db: &InventoryDb,
) -> CommandResult<CatalogDeleteResult> {
    require_catalog_ready(db)?;
    let area = db
        .find_storage_area(area_uuid)?
        .ok_or_else(|| "The selected storage area could not be found.".to_string())?;
    if db
        .load_storage_containers()?
        .iter()
        .any(|container| container.area_uuid == area.area_uuid)
    {
        return Err("Archive the area or remove its containers before deleting it.".to_string());
    }
    let sync_backup = db.backup_sync_state()?;
    db.delete_storage_area(&area)?;
    if let Err(error) = catalog_sync::queue_delete_operation(
        db,
        CatalogEntityType::StorageArea,
        &area.area_uuid,
        timestamp_now(),
        Some(area.updated_at.clone()),
    ) {
        let _ = db.put_storage_area(&area);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(delete_result(area.area_uuid, "Storage area deleted.", db))
}

pub(crate) fn create_storage_container_in_store(
    input: StorageContainerInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<StorageContainer>> {
    require_catalog_ready(db)?;
    let input = normalize_storage_container_input(input);
    validate_storage_container_input(&input)?;
    let area = require_area(db, &input.area_uuid)?;
    if area.archived && !input.archived {
        return Err("Restore the storage area before adding an active container.".to_string());
    }
    ensure_unique_container_name(db, &input.area_uuid, &input.name, None, input.archived)?;
    let container = create_storage_container(input);
    let sync_backup = db.backup_sync_state()?;
    db.put_storage_container(&container)?;
    if let Err(error) =
        catalog_sync::queue_upsert_operation(db, container.clone().into(), Vec::new(), None)
    {
        let _ = db.delete_storage_container(&container);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(mutation_result(container, "Storage container added.", db))
}

pub(crate) fn update_storage_container_in_store(
    container_uuid: &str,
    input: StorageContainerInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<StorageContainer>> {
    require_catalog_ready(db)?;
    let input = normalize_storage_container_input(input);
    validate_storage_container_input(&input)?;
    let existing = require_container(db, container_uuid)?;
    let area = require_area(db, &input.area_uuid)?;
    if area.archived && !input.archived {
        return Err("Restore the storage area before using an active container.".to_string());
    }
    ensure_unique_container_name(
        db,
        &input.area_uuid,
        &input.name,
        Some(&existing.container_uuid),
        input.archived,
    )?;
    let container = update_storage_container(existing.clone(), input);
    for placement in db.load_stock_placements_for_container(&existing.container_uuid)? {
        if placement.archived {
            continue;
        }
        let mut placement_input = placement_input(&placement);
        placement_input.container_uuid = container.container_uuid.clone();
        validate_stock_placement_input(&placement_input, &container).map_err(|error| {
            format!(
                "The grid cannot be resized because placement {} would become invalid: {error}",
                placement.placement_uuid
            )
        })?;
    }
    let changed_fields = catalog_sync::changed_fields(&existing, &container);
    if changed_fields.is_empty() {
        return Ok(mutation_result(
            existing,
            "Storage container was already up to date.",
            db,
        ));
    }
    let sync_backup = db.backup_sync_state()?;
    db.put_storage_container(&container)?;
    if let Err(error) = catalog_sync::queue_upsert_operation(
        db,
        container.clone().into(),
        changed_fields,
        Some(existing.updated_at.clone()),
    ) {
        let _ = db.put_storage_container(&existing);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(mutation_result(container, "Storage container updated.", db))
}

pub(crate) fn delete_storage_container_in_store(
    container_uuid: &str,
    db: &InventoryDb,
) -> CommandResult<CatalogDeleteResult> {
    require_catalog_ready(db)?;
    let container = require_container(db, container_uuid)?;
    if !db
        .load_stock_placements_for_container(&container.container_uuid)?
        .is_empty()
    {
        return Err(
            "Archive the container or remove its stock placements before deleting it.".to_string(),
        );
    }
    let sync_backup = db.backup_sync_state()?;
    db.delete_storage_container(&container)?;
    if let Err(error) = catalog_sync::queue_delete_operation(
        db,
        CatalogEntityType::StorageContainer,
        &container.container_uuid,
        timestamp_now(),
        Some(container.updated_at.clone()),
    ) {
        let _ = db.put_storage_container(&container);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(delete_result(
        container.container_uuid,
        "Storage container deleted.",
        db,
    ))
}

pub(crate) fn create_stock_placement_in_store(
    input: StockPlacementInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<StockPlacement>> {
    require_catalog_ready(db)?;
    let part = require_part(db, &input.part_uuid)?;
    if part.archived {
        return Err("Restore the part before adding stock.".to_string());
    }
    let container = require_active_destination_container(db, &input.container_uuid)?;
    let input = normalize_stock_placement_input(input, Some(&part.default_unit_of_measure));
    validate_stock_placement_input(&input, &container)?;
    let placement = create_stock_placement(input);
    ensure_distinct_placement(db, &placement, None)?;
    let sync_backup = db.backup_sync_state()?;
    db.put_stock_placement(&placement)?;
    if let Err(error) =
        catalog_sync::queue_upsert_operation(db, placement.clone().into(), Vec::new(), None)
    {
        let _ = db.delete_stock_placement(&placement);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(mutation_result(placement, "Stock placement added.", db))
}

pub(crate) fn update_stock_placement_in_store(
    placement_uuid: &str,
    input: StockPlacementInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<StockPlacement>> {
    require_catalog_ready(db)?;
    let existing = require_placement(db, placement_uuid)?;
    let part = require_part(db, &input.part_uuid)?;
    let container = require_container(db, &input.container_uuid)?;
    let input = normalize_stock_placement_input(input, Some(&part.default_unit_of_measure));
    if !input.archived {
        require_active_part_and_destination(db, &part, &container)?;
    }
    validate_stock_placement_input(&input, &container)?;
    let placement = update_stock_placement(existing.clone(), input);
    ensure_distinct_placement(db, &placement, Some(&existing.placement_uuid))?;
    let changed_fields = catalog_sync::changed_fields(&existing, &placement);
    if changed_fields.is_empty() {
        return Ok(mutation_result(
            existing,
            "Stock placement was already up to date.",
            db,
        ));
    }
    let sync_backup = db.backup_sync_state()?;
    db.put_stock_placement(&placement)?;
    if let Err(error) = catalog_sync::queue_upsert_operation(
        db,
        placement.clone().into(),
        changed_fields,
        Some(existing.updated_at.clone()),
    ) {
        let _ = db.put_stock_placement(&existing);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(mutation_result(placement, "Stock placement updated.", db))
}

pub(crate) fn delete_stock_placement_in_store(
    placement_uuid: &str,
    db: &InventoryDb,
) -> CommandResult<CatalogDeleteResult> {
    require_catalog_ready(db)?;
    let placement = require_placement(db, placement_uuid)?;
    let sync_backup = db.backup_sync_state()?;
    db.delete_stock_placement(&placement)?;
    if let Err(error) = catalog_sync::queue_delete_operation(
        db,
        CatalogEntityType::StockPlacement,
        &placement.placement_uuid,
        timestamp_now(),
        Some(placement.updated_at.clone()),
    ) {
        let _ = db.put_stock_placement(&placement);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(delete_result(
        placement.placement_uuid,
        "Stock placement deleted.",
        db,
    ))
}

pub(crate) fn count_stock_in_store(
    placement_uuid: &str,
    input: StockCountInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<StockPlacement>> {
    require_catalog_ready(db)?;
    if input.counted_by.trim().is_empty() {
        return Err("Enter who performed the physical count.".to_string());
    }
    let existing = require_placement(db, placement_uuid)?;
    let part = require_part(db, &existing.part_uuid)?;
    let container = require_container(db, &existing.container_uuid)?;
    let mut next_input = placement_input(&existing);
    next_input.quantity = input.quantity;
    next_input.count_state = "counted".to_string();
    next_input.last_counted_at = Some(timestamp_now());
    next_input.last_counted_by = input.counted_by;
    let next_input =
        normalize_stock_placement_input(next_input, Some(&part.default_unit_of_measure));
    validate_stock_placement_input(&next_input, &container)?;
    let placement = update_stock_placement(existing.clone(), next_input);
    let changed_fields = catalog_sync::changed_fields(&existing, &placement);
    let sync_backup = db.backup_sync_state()?;
    db.put_stock_placement(&placement)?;
    if let Err(error) = catalog_sync::queue_upsert_operation(
        db,
        placement.clone().into(),
        changed_fields,
        Some(existing.updated_at.clone()),
    ) {
        let _ = db.put_stock_placement(&existing);
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(error);
    }
    db.flush();
    Ok(mutation_result(placement, "Physical count recorded.", db))
}

pub(crate) fn move_stock_in_store(
    input: StockMoveInput,
    db: &InventoryDb,
) -> CommandResult<CatalogMutationResult<Vec<StockPlacement>>> {
    require_catalog_ready(db)?;
    if !input.quantity.is_finite() || input.quantity <= 0.0 {
        return Err("Move quantity must be greater than zero.".to_string());
    }
    let source = require_placement(db, &input.source_placement_uuid)?;
    if source.archived {
        return Err("Restore the source placement before moving stock.".to_string());
    }
    if input.quantity > source.quantity {
        return Err("Move quantity cannot exceed the source placement quantity.".to_string());
    }
    let part = require_part(db, &source.part_uuid)?;
    if part.archived {
        return Err("Restore the part before moving stock.".to_string());
    }

    let (destination_existing, destination_input) =
        match (input.destination_placement_uuid, input.destination) {
            (Some(destination_uuid), None) => {
                if destination_uuid == source.placement_uuid {
                    return Err("Select a different destination placement.".to_string());
                }
                let destination = require_placement(db, &destination_uuid)?;
                if destination.archived {
                    return Err(
                        "Restore the destination placement before moving stock.".to_string()
                    );
                }
                if destination.part_uuid != source.part_uuid {
                    return Err(
                        "Stock can only move between placements for the same part.".to_string()
                    );
                }
                if destination.unit_of_measure != source.unit_of_measure {
                    return Err("Stock move units must match the source placement.".to_string());
                }
                let container =
                    require_active_destination_container(db, &destination.container_uuid)?;
                let mut next_input = placement_input(&destination);
                next_input.quantity += input.quantity;
                validate_stock_placement_input(&next_input, &container)?;
                (Some(destination), next_input)
            }
            (None, Some(mut destination_input)) => {
                destination_input.part_uuid = source.part_uuid.clone();
                let requested_unit = destination_input.unit_of_measure.trim();
                if !requested_unit.is_empty()
                    && normalized_lookup(requested_unit)
                        != normalized_lookup(&source.unit_of_measure)
                {
                    return Err("Stock move units must match the source placement.".to_string());
                }
                destination_input.unit_of_measure = source.unit_of_measure.clone();
                let container =
                    require_active_destination_container(db, &destination_input.container_uuid)?;
                let destination_input = normalize_stock_placement_input(
                    destination_input,
                    Some(&part.default_unit_of_measure),
                );
                validate_stock_placement_input(&destination_input, &container)?;
                let candidate = create_stock_placement(destination_input.clone());
                let matching = find_distinct_placement(db, &candidate, None)?;
                if let Some(existing) = matching {
                    let mut merged_input = placement_input(&existing);
                    merged_input.quantity += input.quantity;
                    validate_stock_placement_input(&merged_input, &container)?;
                    (Some(existing), merged_input)
                } else {
                    let mut new_input = destination_input;
                    new_input.quantity = input.quantity;
                    (None, new_input)
                }
            }
            _ => {
                return Err(
                    "Provide exactly one destination placement or new destination location."
                        .to_string(),
                )
            }
        };

    let destination = match destination_existing.clone() {
        Some(existing) => update_stock_placement(existing, destination_input),
        None => create_stock_placement(destination_input),
    };
    let mut source_input = placement_input(&source);
    source_input.quantity -= input.quantity;
    if source_input.quantity == 0.0 {
        source_input.archived = true;
    }
    let source_container = require_container(db, &source.container_uuid)?;
    validate_stock_placement_input(&source_input, &source_container)?;
    let updated_source = update_stock_placement(source.clone(), source_input);

    let destination_before = destination_existing.clone();
    let destination_changed_fields = destination_before
        .as_ref()
        .map(|existing| catalog_sync::changed_fields(existing, &destination))
        .unwrap_or_default();
    let destination_base_version = destination_before
        .as_ref()
        .map(|existing| existing.updated_at.clone());
    let source_changed_fields = catalog_sync::changed_fields(&source, &updated_source);
    let sync_backup = db.backup_sync_state()?;
    db.put_stock_placement(&destination)?;
    if let Err(error) = db.put_stock_placement(&updated_source) {
        if let Some(existing) = destination_before {
            let _ = db.put_stock_placement(&existing);
        } else {
            let _ = db.delete_stock_placement(&destination);
        }
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(format!("Stock move rolled back: {error}"));
    }
    let queue_result = (|| -> CommandResult<()> {
        catalog_sync::queue_upsert_operation(
            db,
            destination.clone().into(),
            destination_changed_fields,
            destination_base_version,
        )?;
        catalog_sync::queue_upsert_operation(
            db,
            updated_source.clone().into(),
            source_changed_fields,
            Some(source.updated_at.clone()),
        )?;
        Ok(())
    })();
    if let Err(error) = queue_result {
        let _ = db.put_stock_placement(&source);
        if let Some(existing) = destination_existing {
            let _ = db.put_stock_placement(&existing);
        } else {
            let _ = db.delete_stock_placement(&destination);
        }
        let _ = db.restore_sync_state(sync_backup);
        db.flush();
        return Err(format!("Stock move rolled back: {error}"));
    }
    db.flush();

    Ok(mutation_result(
        vec![updated_source, destination],
        "Stock moved between placements.",
        db,
    ))
}

fn require_catalog_ready(db: &InventoryDb) -> CommandResult<()> {
    ensure_catalog_initialized(db)
}

fn require_part(db: &InventoryDb, part_uuid: &str) -> CommandResult<Part> {
    db.find_part(part_uuid)?
        .ok_or_else(|| "The selected part could not be found.".to_string())
}

fn require_area(db: &InventoryDb, area_uuid: &str) -> CommandResult<StorageArea> {
    db.find_storage_area(area_uuid)?
        .ok_or_else(|| "The selected storage area could not be found.".to_string())
}

fn require_container(db: &InventoryDb, container_uuid: &str) -> CommandResult<StorageContainer> {
    db.find_storage_container(container_uuid)?
        .ok_or_else(|| "The selected storage container could not be found.".to_string())
}

fn require_placement(db: &InventoryDb, placement_uuid: &str) -> CommandResult<StockPlacement> {
    db.find_stock_placement(placement_uuid)?
        .ok_or_else(|| "The selected stock placement could not be found.".to_string())
}

fn require_active_destination_container(
    db: &InventoryDb,
    container_uuid: &str,
) -> CommandResult<StorageContainer> {
    let container = require_container(db, container_uuid)?;
    require_active_destination(db, &container)?;
    Ok(container)
}

fn require_active_part_and_destination(
    db: &InventoryDb,
    part: &Part,
    container: &StorageContainer,
) -> CommandResult<()> {
    if part.archived {
        return Err("Restore the part before changing active stock.".to_string());
    }
    require_active_destination(db, container)
}

fn require_active_destination(db: &InventoryDb, container: &StorageContainer) -> CommandResult<()> {
    if container.archived {
        return Err("Restore the storage container before adding stock to it.".to_string());
    }
    let area = require_area(db, &container.area_uuid)?;
    if area.archived {
        return Err("Restore the storage area before adding stock to it.".to_string());
    }
    Ok(())
}

fn ensure_unique_part_number(
    db: &InventoryDb,
    internal_part_number: &str,
    current_uuid: Option<&str>,
) -> CommandResult<()> {
    if normalized_part_number(internal_part_number).is_empty() {
        return Ok(());
    }
    if let Some(existing) = db.find_part_by_internal_number(internal_part_number)? {
        if current_uuid != Some(existing.entry_uuid.as_str()) {
            return Err("Internal part number must be unique.".to_string());
        }
    }
    Ok(())
}

fn ensure_unique_area_name(
    db: &InventoryDb,
    name: &str,
    current_uuid: Option<&str>,
    archived: bool,
) -> CommandResult<()> {
    if archived {
        return Ok(());
    }
    if let Some(existing) = db.find_storage_area_by_name(name)? {
        if current_uuid != Some(existing.area_uuid.as_str()) {
            return Err("Active storage area names must be unique.".to_string());
        }
    }
    Ok(())
}

fn ensure_unique_container_name(
    db: &InventoryDb,
    area_uuid: &str,
    name: &str,
    current_uuid: Option<&str>,
    archived: bool,
) -> CommandResult<()> {
    if archived {
        return Ok(());
    }
    if let Some(existing) = db.find_storage_container_by_name(area_uuid, name)? {
        if current_uuid != Some(existing.container_uuid.as_str()) {
            return Err("Active container names must be unique within a storage area.".to_string());
        }
    }
    Ok(())
}

fn ensure_distinct_placement(
    db: &InventoryDb,
    placement: &StockPlacement,
    current_uuid: Option<&str>,
) -> CommandResult<()> {
    if let Some(existing) = find_distinct_placement(db, placement, current_uuid)? {
        return Err(format!(
            "An indistinguishable placement already exists ({}). Adjust that placement instead.",
            existing.placement_uuid
        ));
    }
    Ok(())
}

fn find_distinct_placement(
    db: &InventoryDb,
    placement: &StockPlacement,
    current_uuid: Option<&str>,
) -> CommandResult<Option<StockPlacement>> {
    let key = indistinguishable_placement_key(placement);
    Ok(db
        .load_stock_placements_for_part(&placement.part_uuid)?
        .into_iter()
        .find(|existing| {
            !existing.archived
                && current_uuid != Some(existing.placement_uuid.as_str())
                && indistinguishable_placement_key(existing) == key
        }))
}

fn placement_input(placement: &StockPlacement) -> StockPlacementInput {
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

fn mutation_result<T>(
    value: T,
    message: impl Into<String>,
    db: &InventoryDb,
) -> CatalogMutationResult<T> {
    CatalogMutationResult {
        value,
        message: message.into(),
        mutation_mode: "local".to_string(),
        shared: local_shared_status(db),
    }
}

fn delete_result(
    entity_uuid: String,
    message: impl Into<String>,
    db: &InventoryDb,
) -> CatalogDeleteResult {
    CatalogDeleteResult {
        entity_uuid,
        message: message.into(),
        mutation_mode: "local".to_string(),
        shared: local_shared_status(db),
    }
}

fn local_shared_status(db: &InventoryDb) -> InventorySharedStatus {
    catalog_sync::queued_local_status(db)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{env, fs, path::PathBuf};
    use uuid::Uuid;

    #[test]
    fn active_names_are_unique_but_archived_names_can_be_reused() {
        let db = test_db();
        create_storage_area_in_store(area_input("Main Lab", false), &db).unwrap();
        assert!(
            create_storage_area_in_store(area_input(" main   lab ", false), &db)
                .unwrap_err()
                .contains("unique")
        );
        create_storage_area_in_store(area_input("Main Lab", true), &db).unwrap();
    }

    #[test]
    fn grid_resize_is_blocked_when_an_active_placement_would_fall_outside() {
        let db = test_db();
        let part = create_part_in_store(part_input("LAB-1"), &db)
            .unwrap()
            .value;
        let area = create_storage_area_in_store(area_input("Main Lab", false), &db)
            .unwrap()
            .value;
        let container = create_storage_container_in_store(
            grid_container_input(&area.area_uuid, "Cabinet", 4, 4),
            &db,
        )
        .unwrap()
        .value;
        create_stock_placement_in_store(
            StockPlacementInput {
                part_uuid: part.entry_uuid,
                container_uuid: container.container_uuid.clone(),
                row_index: Some(3),
                column_index: Some(3),
                quantity: 10.0,
                unit_of_measure: "pcs".to_string(),
                ..StockPlacementInput::default()
            },
            &db,
        )
        .unwrap();

        assert!(update_storage_container_in_store(
            &container.container_uuid,
            grid_container_input(&area.area_uuid, "Cabinet", 3, 3),
            &db,
        )
        .unwrap_err()
        .contains("would become invalid"));
    }

    #[test]
    fn move_preserves_total_and_merges_matching_destination() {
        let db = test_db();
        let part = create_part_in_store(part_input("LAB-2"), &db)
            .unwrap()
            .value;
        let area = create_storage_area_in_store(area_input("Main Lab", false), &db)
            .unwrap()
            .value;
        let container = create_storage_container_in_store(
            grid_container_input(&area.area_uuid, "Cabinet", 2, 2),
            &db,
        )
        .unwrap()
        .value;
        let source = create_stock_placement_in_store(
            placement_input_for(&part.entry_uuid, &container.container_uuid, 0, 0, 10.0),
            &db,
        )
        .unwrap()
        .value;
        let destination = create_stock_placement_in_store(
            placement_input_for(&part.entry_uuid, &container.container_uuid, 0, 1, 2.0),
            &db,
        )
        .unwrap()
        .value;

        let moved = move_stock_in_store(
            StockMoveInput {
                source_placement_uuid: source.placement_uuid.clone(),
                destination_placement_uuid: Some(destination.placement_uuid.clone()),
                destination: None,
                quantity: 4.0,
            },
            &db,
        )
        .unwrap();

        assert_eq!(moved.value[0].quantity, 6.0);
        assert_eq!(moved.value[1].quantity, 6.0);
        let total = db
            .load_stock_placements_for_part(&part.entry_uuid)
            .unwrap()
            .iter()
            .map(|placement| placement.quantity)
            .sum::<f64>();
        assert_eq!(total, 12.0);
    }

    fn part_input(internal_part_number: &str) -> PartInput {
        PartInput {
            internal_part_number: internal_part_number.to_string(),
            description: "Test component".to_string(),
            default_unit_of_measure: "pcs".to_string(),
            ..PartInput::default()
        }
    }

    fn area_input(name: &str, archived: bool) -> StorageAreaInput {
        StorageAreaInput {
            name: name.to_string(),
            area_type: "lab".to_string(),
            archived,
            ..StorageAreaInput::default()
        }
    }

    fn grid_container_input(
        area_uuid: &str,
        name: &str,
        rows: u32,
        columns: u32,
    ) -> StorageContainerInput {
        StorageContainerInput {
            area_uuid: area_uuid.to_string(),
            name: name.to_string(),
            container_type: "cabinet".to_string(),
            grid_enabled: true,
            row_count: Some(rows),
            column_count: Some(columns),
            ..StorageContainerInput::default()
        }
    }

    fn placement_input_for(
        part_uuid: &str,
        container_uuid: &str,
        row_index: u32,
        column_index: u32,
        quantity: f64,
    ) -> StockPlacementInput {
        StockPlacementInput {
            part_uuid: part_uuid.to_string(),
            container_uuid: container_uuid.to_string(),
            row_index: Some(row_index),
            column_index: Some(column_index),
            quantity,
            unit_of_measure: "pcs".to_string(),
            ..StockPlacementInput::default()
        }
    }

    fn test_db() -> InventoryDb {
        let root = unique_test_dir("catalog-mutations");
        fs::create_dir_all(&root).unwrap();
        InventoryDb::open_at(root.join("te-lab-components.feox")).unwrap()
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
    }
}
