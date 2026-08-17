pub(crate) mod api;
pub(crate) mod domain;
pub(crate) mod integrations;
mod inventory_stores;
pub(crate) mod modules;
pub(crate) mod platform;
pub(crate) mod runtime;
pub(crate) mod storage;
mod sync;

pub(crate) use api::commands;
pub(crate) use domain::{model, query};
pub(crate) use integrations::{
    calibration_roster_import, deprecated_db_cleanup, export, inventory_import, native,
};
pub(crate) use runtime::{shared_sync, shared_watcher};
pub(crate) use storage as store;

use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let app_paths = platform::InventoryAppPaths::resolve(app.handle())?;
            let stores = inventory_stores::InventoryStores::open(&app_paths)?;
            let te_db = stores.te_test_equipment();
            let lab_db = stores.te_lab_components();
            let _ = deprecated_db_cleanup::quarantine_deprecated_databases_once(&app_paths, te_db);
            // Recovery must never prevent the window from opening: unknown/future outbox
            // op kinds (or mixed catalog vs inventory rows) should not brick launches or
            // block in-app updates from an older install.
            if let Err(error) = sync::recover_local_sync_state(te_db) {
                eprintln!("TE local sync recovery failed (continuing startup): {error}");
            }
            let lab_recovery = if lab_db.schema_version()?
                == Some(modules::te_lab_components::catalog_model::CATALOG_SCHEMA_VERSION)
            {
                modules::te_lab_components::catalog_sync::recover_local_sync_state(lab_db)
            } else {
                modules::te_lab_components::sync::recover_local_sync_state(lab_db).map(|_| ())
            };
            if let Err(error) = lab_recovery {
                eprintln!("Lab local sync recovery failed (continuing startup): {error}");
            }
            app.manage(stores);
            app.manage(shared_sync::SharedSyncCoordinator::new());
            app.manage(shared_watcher::SharedSyncWatcher::new());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_inventory,
            commands::query_inventory,
            commands::activate_inventory_sync,
            commands::sync_inventory,
            commands::deactivate_inventory_sync,
            commands::create_entry,
            commands::update_entry,
            commands::toggle_verified_entry,
            commands::set_archived_entry,
            commands::delete_entry,
            commands::create_lab_part,
            commands::update_lab_part,
            commands::create_lab_simple_component,
            commands::update_lab_simple_component,
            commands::delete_lab_part,
            commands::create_lab_storage_area,
            commands::update_lab_storage_area,
            commands::delete_lab_storage_area,
            commands::create_lab_storage_container,
            commands::update_lab_storage_container,
            commands::delete_lab_storage_container,
            commands::create_lab_stock_placement,
            commands::update_lab_stock_placement,
            commands::delete_lab_stock_placement,
            commands::move_lab_stock,
            commands::count_lab_stock,
            commands::preview_lab_catalog_migration,
            commands::commit_lab_catalog_migration,
            commands::preview_lab_shared_cutover,
            commands::commit_lab_shared_cutover,
            commands::pick_import_file,
            commands::preview_import,
            commands::commit_import,
            commands::pick_calibration_roster_file,
            commands::preview_calibration_roster,
            commands::commit_calibration_roster,
            export::export_excel,
            native::load_picture_preview,
            native::open_external,
            native::open_path,
            native::pick_picture_path
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|app_handle, event| {
        if let RunEvent::Exit = event {
            app_handle
                .state::<inventory_stores::InventoryStores>()
                .flush();
        }
    });
}
