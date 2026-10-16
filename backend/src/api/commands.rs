use crate::modules::te_storage::{
    commands as room_commands, model as room_model, mutations as room_mutations,
};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::modules::te_test_equipment::mutations::{
    create_entry_in_store, delete_entry_in_store, set_archived_entry_in_store,
    toggle_verified_entry_in_store, update_entry_in_store,
};
use crate::{
    inventory_stores::InventoryStores,
    modules::te_lab_components::{
        catalog_model as lab_catalog_model, catalog_mutations as lab_catalog_mutations,
        catalog_query as lab_catalog_query, catalog_sync as lab_catalog_sync,
        simple_workflow as lab_simple_workflow, store::InventoryDb as LabInventoryDb,
    },
    modules::te_test_equipment::{
        calibration_roster_import::{
            self, CalibrationRosterCommitInput, CalibrationRosterCommitResult,
            CalibrationRosterPreviewReport, CALIBRATION_ROSTER_FILE_EXTENSIONS,
        },
        model::{
            CommandResult, InventoryEntryEditContext, InventoryEntryInput, InventoryQueryInput,
            InventoryQueryResult, InventorySharedStatus, InventorySyncResult,
        },
        query::{get_inventory_counts, query_entries},
        store::InventoryDb,
    },
    platform::{
        runtime::shared_sync::SharedSyncCoordinator,
        runtime::shared_watcher::{self, SharedSyncWatcher},
        sync, ModuleId,
    },
};

const LAB_USES_CATALOG_COMMANDS: &str =
    "TE Lab Components changes go through the Lab catalog commands.";
const WATCHER_DEGRADED_MESSAGE: &str =
    "File watch unavailable; scheduled synchronization remains active.";

#[tauri::command]
pub(crate) fn load_inventory(
    module_id: String,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let module = parse_module_id(&module_id)?;
    match module {
        ModuleId::TeStorage => coordinator.run_exclusive(module, "EVCS Storage load", || {
            command_value(room_commands::load(
                stores.te_storage(),
                coordinator.background_status(module)?,
            )?)
        }),

        ModuleId::TeTestEquipment => command_value(load_inventory_from_store_with_status(
            stores.te_test_equipment(),
            coordinator.background_status::<InventorySharedStatus>(module)?,
        )?),
        ModuleId::TeLabComponents => command_value(load_lab_inventory_from_store_with_status(
            stores.te_lab_components(),
            coordinator.background_status::<InventorySharedStatus>(module)?,
        )?),
    }
}

#[tauri::command]
pub(crate) fn query_inventory(
    module_id: String,
    input: Value,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let module = parse_module_id(&module_id)?;
    match module {
        ModuleId::TeStorage => Err("This operation is not supported for EVCS Storage.".into()),

        ModuleId::TeTestEquipment => {
            let input = parse_command_input::<InventoryQueryInput>(input, "TE inventory query")?;
            command_value(query_inventory_from_store_with_status(
                input,
                stores.te_test_equipment(),
                coordinator.background_status::<InventorySharedStatus>(module)?,
            )?)
        }
        ModuleId::TeLabComponents => Err(LAB_USES_CATALOG_COMMANDS.into()),
    }
}

#[tauri::command]
pub(crate) fn activate_inventory_sync(
    module_id: String,
    watcher: State<'_, SharedSyncWatcher>,
) -> CommandResult<String> {
    watcher.activate_session_for(parse_module_id(&module_id)?)
}

#[tauri::command]
pub(crate) async fn sync_inventory(
    app: AppHandle,
    module_id: String,
    session_id: String,
    coordinator: State<'_, SharedSyncCoordinator>,
    watcher: State<'_, SharedSyncWatcher>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Option<Value>> {
    let module = parse_module_id(&module_id)?;
    if !watcher.begin_sync_for(module, &session_id)? {
        return Ok(None);
    }

    match module {
        ModuleId::TeStorage => {
            room_commands::synchronize(app, session_id, &coordinator, &watcher, stores.te_storage())
                .await?
                .map(command_value)
                .transpose()
        }

        ModuleId::TeTestEquipment => {
            let coordinator = coordinator.inner().clone();
            let task_coordinator = coordinator.clone();
            let db = stores.te_test_equipment().clone();
            let (result, entries, db_path) = tauri::async_runtime::spawn_blocking(move || {
                let result = task_coordinator
                    .run_exclusive(module, "shared sync", || sync::run_shared_sync(&db))?;
                let entries = if result.entries_changed {
                    db.load_entries()?
                } else {
                    Vec::new()
                };

                Ok::<_, String>((result, entries, db.db_path_string()))
            })
            .await
            .map_err(|error| format!("Shared sync task failed: {error}"))??;

            let mut result = result;
            let completion = if result.shared.enabled && result.shared.available {
                let paths = sync::resolved_shared_sync_paths::<
                    crate::modules::te_test_equipment::model::InventoryEntry,
                >();
                watcher.complete_sync_for(app, module, &session_id, &paths.ops_dir)?
            } else {
                watcher.complete_sync_without_watcher_for(module, &session_id)?
            };
            if !completion.current {
                return Ok(None);
            }
            if completion.watcher_degradation_started {
                result.shared.message = WATCHER_DEGRADED_MESSAGE.to_string();
            }
            coordinator.set_background_status(module, result.shared.clone())?;

            Ok(Some(command_value(InventorySyncResult {
                db_path,
                entries,
                entries_changed: Some(result.entries_changed),
                shared: result.shared,
            })?))
        }
        ModuleId::TeLabComponents => {
            let coordinator = coordinator.inner().clone();
            let task_coordinator = coordinator.clone();
            let db = stores.te_lab_components().clone();
            let result = tauri::async_runtime::spawn_blocking(move || {
                let result =
                    task_coordinator.run_exclusive(module, "shared catalog sync", || {
                        lab_catalog_sync::run_shared_sync(&db)
                    })?;
                Ok::<_, String>(result)
            })
            .await
            .map_err(|error| format!("Shared sync task failed: {error}"))??;

            let mut result = result;
            let completion = if result.shared.enabled && result.shared.available {
                let paths = lab_catalog_sync::resolved_catalog_shared_sync_paths();
                watcher.complete_sync_for(app, module, &session_id, &paths.ops_dir)?
            } else {
                watcher.complete_sync_without_watcher_for(module, &session_id)?
            };
            if !completion.current {
                return Ok(None);
            }
            if completion.watcher_degradation_started {
                result.shared.message = WATCHER_DEGRADED_MESSAGE.to_string();
            }
            coordinator.set_background_status(module, result.shared.clone())?;

            Ok(Some(command_value(
                lab_catalog_query::load_catalog_from_store(
                    stores.te_lab_components(),
                    result.shared,
                    Some(result.entries_changed),
                )?,
            )?))
        }
    }
}

#[tauri::command]
pub(crate) fn deactivate_inventory_sync(
    module_id: String,
    session_id: String,
    watcher: State<'_, SharedSyncWatcher>,
) -> CommandResult<bool> {
    watcher.deactivate_session_for(parse_module_id(&module_id)?, &session_id)
}

#[tauri::command]
pub(crate) fn create_entry(
    app: AppHandle,
    module_id: String,
    input: Value,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let module = parse_module_id(&module_id)?;
    match module {
        ModuleId::TeStorage => {
            let input = parse_command_input::<room_model::InventoryEntryInput>(
                input,
                "EVCS Storage entry",
            )?;
            let coordinator = coordinator.inner().clone();
            let db = stores.te_storage();
            let result = coordinator.run_exclusive(module, "EVCS Storage create", || {
                room_mutations::create_entry_in_store(input, db)
            })?;
            room_commands::schedule_publish(app, db.clone(), coordinator);
            command_value(result)
        }

        ModuleId::TeTestEquipment => {
            let input = parse_command_input::<InventoryEntryInput>(input, "TE entry")?;
            let coordinator = coordinator.inner().clone();
            let db = stores.te_test_equipment();
            let result = coordinator.run_exclusive(module, "inventory create", || {
                create_entry_in_store(input, db)
            })?;
            schedule_te_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
        ModuleId::TeLabComponents => Err(LAB_USES_CATALOG_COMMANDS.into()),
    }
}

#[tauri::command]
pub(crate) fn update_entry(
    app: AppHandle,
    module_id: String,
    entry_id: String,
    input: Value,
    edit_context: Option<Value>,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let module = parse_module_id(&module_id)?;
    match module {
        ModuleId::TeStorage => {
            let input = parse_command_input::<room_model::InventoryEntryInput>(
                input,
                "EVCS Storage entry",
            )?;
            let edit_context = parse_optional_command_input::<room_model::InventoryEntryEditContext>(
                edit_context,
                "EVCS Storage edit context",
            )?;
            let coordinator = coordinator.inner().clone();
            let db = stores.te_storage();
            let result = coordinator.run_exclusive(module, "EVCS Storage update", || {
                room_mutations::update_entry_in_store(&entry_id, input, edit_context, db)
            })?;
            room_commands::schedule_publish(app, db.clone(), coordinator);
            command_value(result)
        }

        ModuleId::TeTestEquipment => {
            let input = parse_command_input::<InventoryEntryInput>(input, "TE entry")?;
            let edit_context = parse_optional_command_input::<InventoryEntryEditContext>(
                edit_context,
                "TE edit context",
            )?;
            let coordinator = coordinator.inner().clone();
            let db = stores.te_test_equipment();
            let result = coordinator.run_exclusive(module, "inventory update", || {
                update_entry_in_store(&entry_id, input, edit_context, db)
            })?;
            schedule_te_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
        ModuleId::TeLabComponents => Err(LAB_USES_CATALOG_COMMANDS.into()),
    }
}

#[tauri::command]
pub(crate) fn toggle_verified_entry(
    app: AppHandle,
    module_id: String,
    entry_id: String,
    next_verified: bool,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let module = parse_module_id(&module_id)?;
    match module {
        ModuleId::TeStorage => Err("This operation is not supported for EVCS Storage.".into()),

        ModuleId::TeTestEquipment => {
            let coordinator = coordinator.inner().clone();
            let db = stores.te_test_equipment();
            let result = coordinator.run_exclusive(module, "inventory verify", || {
                toggle_verified_entry_in_store(&entry_id, next_verified, db)
            })?;
            schedule_te_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
        ModuleId::TeLabComponents => Err(LAB_USES_CATALOG_COMMANDS.into()),
    }
}

#[tauri::command]
pub(crate) fn set_archived_entry(
    app: AppHandle,
    module_id: String,
    entry_id: String,
    archived: bool,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let module = parse_module_id(&module_id)?;
    match module {
        ModuleId::TeStorage => Err("This operation is not supported for EVCS Storage.".into()),

        ModuleId::TeTestEquipment => {
            let coordinator = coordinator.inner().clone();
            let db = stores.te_test_equipment();
            let result = coordinator.run_exclusive(module, "inventory archive", || {
                set_archived_entry_in_store(&entry_id, archived, db)
            })?;
            schedule_te_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
        ModuleId::TeLabComponents => Err(LAB_USES_CATALOG_COMMANDS.into()),
    }
}

#[tauri::command]
pub(crate) fn delete_entry(
    app: AppHandle,
    module_id: String,
    entry_id: String,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<Value> {
    let module = parse_module_id(&module_id)?;
    match module {
        ModuleId::TeStorage => {
            let coordinator = coordinator.inner().clone();
            let db = stores.te_storage();
            let result = coordinator.run_exclusive(module, "EVCS Storage delete", || {
                room_mutations::delete_entry_in_store(&entry_id, db)
            })?;
            room_commands::schedule_publish(app, db.clone(), coordinator);
            command_value(result)
        }

        ModuleId::TeTestEquipment => {
            let coordinator = coordinator.inner().clone();
            let db = stores.te_test_equipment();
            let result = coordinator.run_exclusive(module, "inventory delete", || {
                delete_entry_in_store(&entry_id, db)
            })?;
            schedule_te_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
        ModuleId::TeLabComponents => Err(LAB_USES_CATALOG_COMMANDS.into()),
    }
}

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

#[tauri::command]
pub(crate) async fn pick_calibration_roster_file(
    app: AppHandle,
    module_id: String,
) -> CommandResult<Option<String>> {
    require_te_module(parse_module_id(&module_id)?)?;
    let selected = app
        .dialog()
        .file()
        .set_title("Select TE Calibration Roster Workbook")
        .add_filter("Calibration roster", CALIBRATION_ROSTER_FILE_EXTENSIONS)
        .blocking_pick_file();

    selected
        .map(|file_path| {
            file_path
                .simplified()
                .into_path()
                .map(|path| path.to_string_lossy().to_string())
                .map_err(|error| format!("Could not read the selected roster path: {error}"))
        })
        .transpose()
}

#[tauri::command]
pub(crate) fn preview_calibration_roster(
    module_id: String,
    path: String,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<CalibrationRosterPreviewReport> {
    let module = parse_module_id(&module_id)?;
    require_te_module(module)?;
    coordinator.run_exclusive(module, "calibration roster preview", || {
        calibration_roster_import::preview_calibration_roster_from_path(
            std::path::Path::new(&path),
            stores.te_test_equipment(),
        )
    })
}

#[tauri::command]
pub(crate) fn commit_calibration_roster(
    app: AppHandle,
    module_id: String,
    input: CalibrationRosterCommitInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<CalibrationRosterCommitResult> {
    let module = parse_module_id(&module_id)?;
    require_te_module(module)?;
    let coordinator = coordinator.inner().clone();
    let db = stores.te_test_equipment();
    let result = coordinator.run_exclusive(module, "calibration roster commit", || {
        calibration_roster_import::commit_calibration_roster_from_store(input, db)
    })?;
    if result.entries_changed {
        schedule_te_shared_publish(app, db.clone(), coordinator);
    }
    Ok(result)
}

fn parse_module_id(value: &str) -> CommandResult<ModuleId> {
    ModuleId::parse(value).ok_or_else(|| format!("Unknown inventory module id: {value}"))
}

fn require_te_module(module: ModuleId) -> CommandResult<()> {
    if module == ModuleId::TeTestEquipment {
        Ok(())
    } else {
        Err("Import is only available for TE Test Equipment.".to_string())
    }
}

fn parse_command_input<T: DeserializeOwned>(value: Value, label: &str) -> CommandResult<T> {
    serde_json::from_value(value).map_err(|error| format!("Invalid {label}: {error}"))
}

fn parse_optional_command_input<T: DeserializeOwned>(
    value: Option<Value>,
    label: &str,
) -> CommandResult<Option<T>> {
    value
        .map(|value| parse_command_input(value, label))
        .transpose()
}

fn command_value<T: Serialize>(value: T) -> CommandResult<Value> {
    serde_json::to_value(value)
        .map_err(|error| format!("Could not serialize command result: {error}"))
}

#[cfg(test)]
fn load_inventory_from_store(db: &InventoryDb) -> CommandResult<InventorySyncResult> {
    load_inventory_from_store_with_status(db, None)
}

fn load_inventory_from_store_with_status(
    db: &InventoryDb,
    latest_background_status: Option<InventorySharedStatus>,
) -> CommandResult<InventorySyncResult> {
    let shared = latest_background_status.unwrap_or_else(|| {
        let message = sync::last_local_recovery_message(db)
            .unwrap_or_else(|| "FeOxDB local store ready. Shared sync starting.".to_string());
        sync::startup_inventory_status::<crate::modules::te_test_equipment::model::InventoryEntry>(
            message,
        )
    });
    let entries = db.load_entries()?;

    Ok(InventorySyncResult {
        db_path: db.db_path_string(),
        entries,
        entries_changed: Some(true),
        shared,
    })
}

fn load_lab_inventory_from_store_with_status(
    db: &LabInventoryDb,
    latest_background_status: Option<InventorySharedStatus>,
) -> CommandResult<lab_catalog_model::CatalogSyncResult> {
    let shared = latest_background_status.unwrap_or_else(|| {
        lab_catalog_sync::startup_catalog_status(
            "FeOxDB Lab Components catalog ready. Shared sync starting.",
        )
    });
    lab_catalog_query::load_catalog_from_store(db, shared, Some(true))
}

fn query_inventory_from_store_with_status(
    input: InventoryQueryInput,
    db: &InventoryDb,
    latest_background_status: Option<InventorySharedStatus>,
) -> CommandResult<InventoryQueryResult> {
    let all_entries = db.load_entries()?;
    let counts = get_inventory_counts(&all_entries);
    let (entries, total_filtered) = query_entries(&all_entries, input);
    let shared = latest_background_status
        .unwrap_or_else(|| sync::shared_inventory_status(db, "FeOxDB local store ready."));

    Ok(InventoryQueryResult {
        counts,
        db_path: db.db_path_string(),
        entries,
        shared,
        total_filtered,
    })
}

fn schedule_te_shared_publish(app: AppHandle, db: InventoryDb, coordinator: SharedSyncCoordinator) {
    let module = ModuleId::TeTestEquipment;
    drop(tauri::async_runtime::spawn_blocking(move || {
        let status = match coordinator.run_exclusive(module, "shared publish", || {
            sync::publish_pending_local_changes(&db)
        }) {
            Ok(result) => result.shared,
            Err(error) => sync::shared_inventory_status(
                &db,
                format!("Background shared publish failed: {error}"),
            ),
        };
        let _ = coordinator.set_background_status(module, status);
        db.flush();
        shared_watcher::emit_module_shared_inventory_changed(&app, module);
    }));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modules::te_test_equipment::model::{create_entry_from_input, InventoryEntryInput};
    use crate::platform::sync::test_support::SyncOperationEnvelope;
    use std::{env, fs, path::PathBuf};
    use uuid::Uuid;

    #[test]
    fn load_inventory_returns_local_entries_without_shared_sync_bootstrap() {
        let db = test_db();
        let entry = create_entry_from_input(1, test_input("Startup local"));
        db.put_entry(&entry).unwrap();
        db.set_next_entry_id(2).unwrap();
        db.flush();

        let loaded = load_inventory_from_store(&db).unwrap();

        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].description, "Startup local");
        // Load does not bootstrap/publish; existing rows stay local until shared sync runs.
        assert_eq!(outbox_count(&db), 0);
        assert!(loaded.shared.enabled);
    }

    #[test]
    fn load_and_query_surface_latest_background_publish_status() {
        let db = test_db();
        let status = InventorySharedStatus {
            available: false,
            can_modify: true,
            enabled: true,
            has_local_only_changes: Some(true),
            last_snapshot_id: None,
            message: "Background shared publish failed: permission denied".to_string(),
            mutation_mode: "local".to_string(),
            revision: Some("7".to_string()),
            shared_root_path: Some("S:\\TE\\Test_Equipment".to_string()),
            warning: None,
        };

        let loaded = load_inventory_from_store_with_status(&db, Some(status.clone())).unwrap();
        let queried = query_inventory_from_store_with_status(
            InventoryQueryInput::default(),
            &db,
            Some(status),
        )
        .unwrap();

        assert_eq!(
            loaded.shared.message,
            "Background shared publish failed: permission denied"
        );
        assert_eq!(
            queried.shared.message,
            "Background shared publish failed: permission denied"
        );
        assert_eq!(queried.shared.has_local_only_changes, Some(true));
    }

    fn test_input(description: &str) -> InventoryEntryInput {
        InventoryEntryInput {
            description: description.to_string(),
            lifecycle_status: "active".to_string(),
            working_status: "unknown".to_string(),
            ..InventoryEntryInput::default()
        }
    }

    fn test_db() -> InventoryDb {
        let root = unique_test_dir("commands");
        fs::create_dir_all(&root).unwrap();
        InventoryDb::open_at(root.join("inventory.feox")).unwrap()
    }

    fn outbox_count(db: &InventoryDb) -> usize {
        let mut count = 0;
        db.scan_sync_outbox_records::<SyncOperationEnvelope<crate::modules::te_test_equipment::model::InventoryEntry>, _>(
            None,
            usize::MAX,
            |_, _| {
                count += 1;
                Ok(true)
            },
        )
        .unwrap();
        count
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
    }
}
