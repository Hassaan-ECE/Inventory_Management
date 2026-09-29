use tauri::AppHandle;

use super::{model::*, store::InventoryDb, sync};
use crate::{
    platform::ModuleId,
    shared_sync::SharedSyncCoordinator,
    shared_watcher::{self, SharedSyncWatcher},
};

pub(crate) fn load(
    db: &InventoryDb,
    status: Option<InventorySharedStatus>,
) -> CommandResult<InventorySyncResult> {
    Ok(InventorySyncResult {
        db_path: db.db_path_string(),
        entries: db.load_entries()?,
        entries_changed: None,
        shared: status
            .unwrap_or_else(|| sync::shared_inventory_status(db, "Storage Room inventory ready.")),
    })
}

pub(crate) async fn synchronize(
    app: AppHandle,
    session_id: String,
    coordinator: &SharedSyncCoordinator,
    watcher: &SharedSyncWatcher,
    db: &InventoryDb,
) -> CommandResult<Option<InventorySyncResult>> {
    let module = ModuleId::TeStorage;
    let task_coordinator = coordinator.clone();
    let db = db.clone();
    let mut result = tauri::async_runtime::spawn_blocking(move || {
        task_coordinator.run_exclusive(module, "Storage Room sync", || {
            let synced = sync::run_shared_sync(&db)?;
            let mut result = load(&db, Some(synced.shared))?;
            result.entries_changed = Some(synced.entries_changed);
            Ok(result)
        })
    })
    .await
    .map_err(|error| format!("Storage Room sync task failed: {error}"))??;
    let completion = if result.shared.enabled && result.shared.available {
        watcher.complete_sync_for(
            app,
            module,
            &session_id,
            &sync::resolved_shared_sync_paths().ops_dir,
        )?
    } else {
        watcher.complete_sync_without_watcher_for(module, &session_id)?
    };
    if !completion.current {
        return Ok(None);
    }
    if completion.watcher_degradation_started {
        result.shared.message =
            "File watch unavailable; scheduled synchronization remains active.".into();
    }
    coordinator.set_background_status(module, result.shared.clone())?;
    Ok(Some(result))
}

pub(crate) fn schedule_publish(
    app: AppHandle,
    db: InventoryDb,
    coordinator: SharedSyncCoordinator,
) {
    let module = ModuleId::TeStorage;
    drop(tauri::async_runtime::spawn_blocking(move || {
        let status = match coordinator.run_exclusive(module, "Storage Room publish", || {
            sync::publish_pending_local_changes(&db)
        }) {
            Ok(result) => result.shared,
            Err(error) => {
                let mut status = sync::shared_inventory_status(
                    &db,
                    format!("Shared publish failed; changes remain on this computer: {error}"),
                );
                status.available = false;
                status.mutation_mode = "local".into();
                status
            }
        };
        let _ = coordinator.set_background_status(module, status);
        shared_watcher::emit_module_shared_inventory_changed(&app, module);
    }));
}
