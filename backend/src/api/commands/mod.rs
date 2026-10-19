//! Tauri commands. Every command that serves more than one inventory takes a `module_id` and
//! routes to that inventory's store; payloads go out as JSON values.

pub(crate) mod calibration_roster;
pub(crate) mod inventory;
pub(crate) mod lab_catalog;

use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use tauri::AppHandle;

use crate::{
    modules::te_test_equipment::store::InventoryDb,
    platform::{
        common::CommandResult,
        runtime::{shared_sync::SharedSyncCoordinator, shared_watcher},
        sync, ModuleId,
    },
};

fn parse_module_id(value: &str) -> CommandResult<ModuleId> {
    ModuleId::parse(value).ok_or_else(|| format!("Unknown inventory module id: {value}"))
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

/// Publishes queued TE changes in the background after a mutation, then tells the window.
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
