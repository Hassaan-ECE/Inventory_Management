//! Commands every inventory answers, routed by `module_id`: load, the sync session
//! (activate, sync, deactivate), and the entry commands TE Test Equipment and EVCS Storage use.

use crate::modules::te_storage::{
    commands as room_commands, model as room_model, mutations as room_mutations,
};
use serde_json::Value;
use tauri::{AppHandle, State};

use super::{
    command_value, parse_command_input, parse_module_id, parse_optional_command_input,
    schedule_te_shared_publish,
};
use crate::modules::te_test_equipment::mutations::{
    create_entry_in_store, delete_entry_in_store, set_archived_entry_in_store,
    toggle_verified_entry_in_store, update_entry_in_store,
};
use crate::{
    inventory_stores::InventoryStores,
    modules::te_lab_components::{
        catalog_model as lab_catalog_model, catalog_query as lab_catalog_query,
        catalog_sync as lab_catalog_sync, store::InventoryDb as LabInventoryDb,
    },
    modules::te_test_equipment::{
        model::{
            CommandResult, InventoryEntryEditContext, InventoryEntryInput, InventorySharedStatus,
            InventorySyncResult,
        },
        store::InventoryDb,
    },
    platform::{
        runtime::shared_sync::SharedSyncCoordinator, runtime::shared_watcher::SharedSyncWatcher,
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
    fn load_surfaces_latest_background_publish_status() {
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

        let loaded = load_inventory_from_store_with_status(&db, Some(status)).unwrap();

        assert_eq!(
            loaded.shared.message,
            "Background shared publish failed: permission denied"
        );
        assert_eq!(loaded.shared.has_local_only_changes, Some(true));
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
