//! TE Test Equipment calibration roster import: pick, preview, and commit.

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use super::{parse_module_id, schedule_te_shared_publish};
use crate::{
    inventory_stores::InventoryStores,
    modules::te_test_equipment::{
        calibration_roster_import::{
            self, CalibrationRosterCommitInput, CalibrationRosterCommitResult,
            CalibrationRosterPreviewReport, CALIBRATION_ROSTER_FILE_EXTENSIONS,
        },
        model::CommandResult,
    },
    platform::{runtime::shared_sync::SharedSyncCoordinator, ModuleId},
};

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

fn require_te_module(module: ModuleId) -> CommandResult<()> {
    if module == ModuleId::TeTestEquipment {
        Ok(())
    } else {
        Err("Import is only available for TE Test Equipment.".to_string())
    }
}
