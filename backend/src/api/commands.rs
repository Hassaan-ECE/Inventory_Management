use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use super::mutations::{
    create_entry_in_store, delete_entry_in_store, set_archived_entry_in_store,
    toggle_verified_entry_in_store, update_entry_in_store,
};
use crate::{
    inventory_import::{
        self, ImportCommitInput, ImportCommitResult, ImportDryRunReport, IMPORT_FILE_EXTENSIONS,
    },
    inventory_stores::InventoryStores,
    model::{
        CommandResult, InventoryEntryEditContext, InventoryEntryInput, InventoryQueryInput,
        InventoryQueryResult, InventorySharedStatus, InventorySyncResult,
    },
    modules::te_lab_components::{
        model as lab_model, mutations as lab_mutations, query as lab_query,
        store::InventoryDb as LabInventoryDb, sync as lab_sync,
    },
    platform::ModuleId,
    query::{get_inventory_counts, query_entries},
    shared_sync::SharedSyncCoordinator,
    shared_watcher::{self, SharedSyncWatcher},
    store::InventoryDb,
    sync,
};

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
        ModuleId::TeTestEquipment => command_value(load_inventory_from_store_with_status(
            stores.te_test_equipment(),
            coordinator.background_status::<InventorySharedStatus>(module)?,
        )?),
        ModuleId::TeLabComponents => command_value(load_lab_inventory_from_store_with_status(
            stores.te_lab_components(),
            coordinator.background_status::<lab_model::InventorySharedStatus>(module)?,
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
        ModuleId::TeTestEquipment => {
            let input = parse_command_input::<InventoryQueryInput>(input, "TE inventory query")?;
            command_value(query_inventory_from_store_with_status(
                input,
                stores.te_test_equipment(),
                coordinator.background_status::<InventorySharedStatus>(module)?,
            )?)
        }
        ModuleId::TeLabComponents => {
            let input = parse_command_input::<lab_model::InventoryQueryInput>(
                input,
                "Lab inventory query",
            )?;
            command_value(query_lab_inventory_from_store_with_status(
                input,
                stores.te_lab_components(),
                coordinator.background_status::<lab_model::InventorySharedStatus>(module)?,
            )?)
        }
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
                let paths = sync::resolved_shared_sync_paths();
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
            let (result, entries, db_path) = tauri::async_runtime::spawn_blocking(move || {
                let result = task_coordinator
                    .run_exclusive(module, "shared sync", || lab_sync::run_shared_sync(&db))?;
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
                let paths = lab_sync::resolved_shared_sync_paths();
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

            Ok(Some(command_value(lab_model::InventorySyncResult {
                db_path,
                entries,
                entries_changed: Some(result.entries_changed),
                shared: result.shared,
            })?))
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
        ModuleId::TeLabComponents => {
            let input = parse_command_input::<lab_model::InventoryEntryInput>(input, "Lab entry")?;
            let coordinator = coordinator.inner().clone();
            let db = stores.te_lab_components();
            let result = coordinator.run_exclusive(module, "inventory create", || {
                lab_mutations::create_entry_in_store(input, db)
            })?;
            schedule_lab_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
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
        ModuleId::TeLabComponents => {
            let input = parse_command_input::<lab_model::InventoryEntryInput>(input, "Lab entry")?;
            let edit_context = parse_optional_command_input::<lab_model::InventoryEntryEditContext>(
                edit_context,
                "Lab edit context",
            )?;
            let coordinator = coordinator.inner().clone();
            let db = stores.te_lab_components();
            let result = coordinator.run_exclusive(module, "inventory update", || {
                lab_mutations::update_entry_in_store(&entry_id, input, edit_context, db)
            })?;
            schedule_lab_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
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
        ModuleId::TeTestEquipment => {
            let coordinator = coordinator.inner().clone();
            let db = stores.te_test_equipment();
            let result = coordinator.run_exclusive(module, "inventory verify", || {
                toggle_verified_entry_in_store(&entry_id, next_verified, db)
            })?;
            schedule_te_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
        ModuleId::TeLabComponents => {
            let coordinator = coordinator.inner().clone();
            let db = stores.te_lab_components();
            let result = coordinator.run_exclusive(module, "inventory verify", || {
                lab_mutations::toggle_verified_entry_in_store(&entry_id, next_verified, db)
            })?;
            schedule_lab_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
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
        ModuleId::TeTestEquipment => {
            let coordinator = coordinator.inner().clone();
            let db = stores.te_test_equipment();
            let result = coordinator.run_exclusive(module, "inventory archive", || {
                set_archived_entry_in_store(&entry_id, archived, db)
            })?;
            schedule_te_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
        ModuleId::TeLabComponents => {
            let coordinator = coordinator.inner().clone();
            let db = stores.te_lab_components();
            let result = coordinator.run_exclusive(module, "inventory archive", || {
                lab_mutations::set_archived_entry_in_store(&entry_id, archived, db)
            })?;
            schedule_lab_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
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
        ModuleId::TeTestEquipment => {
            let coordinator = coordinator.inner().clone();
            let db = stores.te_test_equipment();
            let result = coordinator.run_exclusive(module, "inventory delete", || {
                delete_entry_in_store(&entry_id, db)
            })?;
            schedule_te_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
        ModuleId::TeLabComponents => {
            let coordinator = coordinator.inner().clone();
            let db = stores.te_lab_components();
            let result = coordinator.run_exclusive(module, "inventory delete", || {
                lab_mutations::delete_entry_in_store(&entry_id, db)
            })?;
            schedule_lab_shared_publish(app, db.clone(), coordinator);
            command_value(result)
        }
    }
}

#[tauri::command]
pub(crate) async fn pick_import_file(
    app: AppHandle,
    module_id: String,
) -> CommandResult<Option<String>> {
    require_te_module(parse_module_id(&module_id)?)?;
    let selected = app
        .dialog()
        .file()
        .set_title("Select TE Test Equipment Import")
        .add_filter("Inventory data", IMPORT_FILE_EXTENSIONS)
        .blocking_pick_file();

    selected
        .map(|file_path| {
            file_path
                .simplified()
                .into_path()
                .map(|path| path.to_string_lossy().to_string())
                .map_err(|error| format!("Could not read the selected import path: {error}"))
        })
        .transpose()
}

#[tauri::command]
pub(crate) fn preview_import(
    module_id: String,
    path: String,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<ImportDryRunReport> {
    let module = parse_module_id(&module_id)?;
    require_te_module(module)?;
    coordinator.run_exclusive(module, "inventory import preview", || {
        inventory_import::preview_import_from_path(
            std::path::Path::new(&path),
            stores.te_test_equipment(),
        )
    })
}

#[tauri::command]
pub(crate) fn commit_import(
    app: AppHandle,
    module_id: String,
    input: ImportCommitInput,
    coordinator: State<'_, SharedSyncCoordinator>,
    stores: State<'_, InventoryStores>,
) -> CommandResult<ImportCommitResult> {
    let module = parse_module_id(&module_id)?;
    require_te_module(module)?;
    validate_v0_1_import_policy(&input)?;
    let coordinator = coordinator.inner().clone();
    let db = stores.te_test_equipment();
    let result = coordinator.run_exclusive(module, "inventory import commit", || {
        inventory_import::commit_import_from_store(input, db)
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

fn validate_v0_1_import_policy(input: &ImportCommitInput) -> CommandResult<()> {
    if input.allow_partial {
        return Err(
            "v0.1 desktop import is full-batch-only; partial commit is reserved for internal tests."
                .to_string(),
        );
    }
    Ok(())
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
        sync::startup_inventory_status(message)
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
    latest_background_status: Option<lab_model::InventorySharedStatus>,
) -> CommandResult<lab_model::InventorySyncResult> {
    let shared = latest_background_status.unwrap_or_else(|| {
        let message = lab_sync::last_local_recovery_message(db).unwrap_or_else(|| {
            "FeOxDB Lab Components store ready. Shared sync starting.".to_string()
        });
        lab_sync::startup_inventory_status(message)
    });
    let entries = db.load_entries()?;

    Ok(lab_model::InventorySyncResult {
        db_path: db.db_path_string(),
        entries,
        entries_changed: Some(true),
        shared,
    })
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

fn query_lab_inventory_from_store_with_status(
    input: lab_model::InventoryQueryInput,
    db: &LabInventoryDb,
    latest_background_status: Option<lab_model::InventorySharedStatus>,
) -> CommandResult<lab_model::InventoryQueryResult> {
    let all_entries = db.load_entries()?;
    let counts = lab_query::get_inventory_counts(&all_entries);
    let (entries, total_filtered) = lab_query::query_entries(&all_entries, input);
    let shared = latest_background_status.unwrap_or_else(|| {
        lab_sync::shared_inventory_status(db, "FeOxDB Lab Components store ready.")
    });

    Ok(lab_model::InventoryQueryResult {
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

fn schedule_lab_shared_publish(
    app: AppHandle,
    db: LabInventoryDb,
    coordinator: SharedSyncCoordinator,
) {
    let module = ModuleId::TeLabComponents;
    drop(tauri::async_runtime::spawn_blocking(move || {
        let status = match coordinator.run_exclusive(module, "shared publish", || {
            lab_sync::publish_pending_local_changes(&db)
        }) {
            Ok(result) => result.shared,
            Err(error) => lab_sync::shared_inventory_status(
                &db,
                format!("Background shared publish failed: {error}"),
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
    use crate::model::{create_entry_from_input, InventoryEntryInput};
    use crate::sync::test_support::SyncOperationEnvelope;
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

    #[test]
    fn desktop_import_policy_rejects_partial_commit_requests() {
        let partial = ImportCommitInput {
            batch_id: "batch-partial".to_string(),
            confirmed: true,
            allow_partial: true,
        };
        let error = validate_v0_1_import_policy(&partial).unwrap_err();
        assert!(error.contains("full-batch-only"));

        let full = ImportCommitInput {
            batch_id: "batch-full".to_string(),
            confirmed: true,
            allow_partial: false,
        };
        assert_eq!(validate_v0_1_import_policy(&full), Ok(()));
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
        db.scan_sync_outbox_records::<SyncOperationEnvelope, _>(None, usize::MAX, |_, _| {
            count += 1;
            Ok(true)
        })
        .unwrap();
        count
    }

    fn unique_test_dir(prefix: &str) -> PathBuf {
        env::temp_dir().join(format!("{prefix}-{}", Uuid::new_v4().simple()))
    }
}
