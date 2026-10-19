//! TE Lab Components catalog commands: parts, the simple one-dialog component, storage areas
//! and containers, stock placements, moves, and counts.

use serde_json::Value;
use tauri::{AppHandle, State};

use super::command_value;
use crate::{
    inventory_stores::InventoryStores,
    modules::te_lab_components::{
        catalog_model as lab_catalog_model, catalog_mutations as lab_catalog_mutations,
        catalog_sync as lab_catalog_sync, simple_workflow as lab_simple_workflow,
        store::InventoryDb as LabInventoryDb,
    },
    modules::te_test_equipment::model::CommandResult,
    platform::{runtime::shared_sync::SharedSyncCoordinator, runtime::shared_watcher, ModuleId},
};

#[tauri::command]
pub(crate) fn create_lab_part(
    app: AppHandle,
    input: lab_catalog_model::PartInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "catalog part create", || {
            lab_catalog_mutations::create_part_in_store(input, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn update_lab_part(
    app: AppHandle,
    part_id: String,
    input: lab_catalog_model::PartInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "catalog part update", || {
            lab_catalog_mutations::update_part_in_store(&part_id, input, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn create_lab_simple_component(
    app: AppHandle,
    input: lab_simple_workflow::SimpleComponentInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "simple component create", || {
            lab_simple_workflow::create_simple_component_in_store(input, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn update_lab_simple_component(
    app: AppHandle,
    part_id: String,
    input: lab_simple_workflow::SimpleComponentInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "simple component update", || {
            lab_simple_workflow::update_simple_component_in_store(&part_id, input, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn delete_lab_part(
    app: AppHandle,
    part_id: String,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "catalog part delete", || {
            lab_catalog_mutations::delete_part_in_store(&part_id, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn create_lab_storage_area(
    app: AppHandle,
    input: lab_catalog_model::StorageAreaInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "storage area create", || {
            lab_catalog_mutations::create_storage_area_in_store(input, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn update_lab_storage_area(
    app: AppHandle,
    area_uuid: String,
    input: lab_catalog_model::StorageAreaInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "storage area update", || {
            lab_catalog_mutations::update_storage_area_in_store(&area_uuid, input, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn delete_lab_storage_area(
    app: AppHandle,
    area_uuid: String,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "storage area delete", || {
            lab_catalog_mutations::delete_storage_area_in_store(&area_uuid, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn create_lab_storage_container(
    app: AppHandle,
    input: lab_catalog_model::StorageContainerInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result = coordinator.run_exclusive(
        ModuleId::TeLabComponents,
        "storage container create",
        || lab_catalog_mutations::create_storage_container_in_store(input, db),
    )?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn update_lab_storage_container(
    app: AppHandle,
    container_uuid: String,
    input: lab_catalog_model::StorageContainerInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result = coordinator.run_exclusive(
        ModuleId::TeLabComponents,
        "storage container update",
        || lab_catalog_mutations::update_storage_container_in_store(&container_uuid, input, db),
    )?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn delete_lab_storage_container(
    app: AppHandle,
    container_uuid: String,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result = coordinator.run_exclusive(
        ModuleId::TeLabComponents,
        "storage container delete",
        || lab_catalog_mutations::delete_storage_container_in_store(&container_uuid, db),
    )?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn create_lab_stock_placement(
    app: AppHandle,
    input: lab_catalog_model::StockPlacementInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "placement create", || {
            lab_catalog_mutations::create_stock_placement_in_store(input, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn update_lab_stock_placement(
    app: AppHandle,
    placement_uuid: String,
    input: lab_catalog_model::StockPlacementInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "placement update", || {
            lab_catalog_mutations::update_stock_placement_in_store(&placement_uuid, input, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn delete_lab_stock_placement(
    app: AppHandle,
    placement_uuid: String,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "placement delete", || {
            lab_catalog_mutations::delete_stock_placement_in_store(&placement_uuid, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn move_lab_stock(
    app: AppHandle,
    input: lab_catalog_model::StockMoveInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result = coordinator.run_exclusive(ModuleId::TeLabComponents, "stock move", || {
        lab_catalog_mutations::move_stock_in_store(input, db)
    })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

#[tauri::command]
pub(crate) fn count_lab_stock(
    app: AppHandle,
    placement_uuid: String,
    input: lab_catalog_model::StockCountInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let coordinator = coordinator.inner().clone();
    let db = stores.te_lab_components();
    let result =
        coordinator.run_exclusive(ModuleId::TeLabComponents, "physical stock count", || {
            lab_catalog_mutations::count_stock_in_store(&placement_uuid, input, db)
        })?;
    schedule_lab_catalog_shared_publish(app, db.clone(), coordinator);
    command_value(result)
}

fn schedule_lab_catalog_shared_publish(
    app: AppHandle,
    db: LabInventoryDb,
    coordinator: SharedSyncCoordinator,
) {
    let module = ModuleId::TeLabComponents;
    drop(tauri::async_runtime::spawn_blocking(move || {
        let status = match coordinator.run_exclusive(module, "shared catalog publish", || {
            lab_catalog_sync::publish_pending_local_changes(&db)
        }) {
            Ok(result) => result.shared,
            Err(error) => lab_catalog_sync::shared_catalog_status(
                &db,
                format!("Background Lab catalog publish failed: {error}"),
            ),
        };
        let _ = coordinator.set_background_status(module, status);
        db.flush();
        shared_watcher::emit_module_shared_inventory_changed(&app, module);
    }));
}
