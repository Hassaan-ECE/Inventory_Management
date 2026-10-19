import { convertFileSrc, invoke, isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { check, type DownloadEvent, type Update } from "@tauri-apps/plugin-updater";

import { APP_VERSION } from "@/app/branding";
import type {
  CalibrationRosterCommitInput,
  CalibrationRosterCommitResult,
  CalibrationRosterPreviewReport,
  ExcelExportResult,
  InventoryDeleteMutationResult,
  InventoryEntryEditContext,
  InventoryEntryInput,
  InventoryEntryMutationResult,
} from "@/modules/te-test-equipment/types";
import type {
  CatalogDeleteResult,
  CatalogMutationResult,
  LabOrderRequestLineInput,
  Part,
  PartInput,
  SimpleComponentInput,
  SimpleComponentValue,
  StockCountInput,
  StockMoveInput,
  StockPlacement,
  StockPlacementInput,
  StorageArea,
  StorageAreaInput,
  StorageContainer,
  StorageContainerInput,
} from "@/modules/te-lab-components/types";
import type {
  ImplementedInventoryModuleId,
  InventoryDesktopBridge,
  InventorySharedChangedPayload,
  InventorySyncResult,
} from "@/integrations/tauri/desktop-bridge";
import type { UpdateState } from "@/platform/updates/updateState";

// Every payload type is generated from the Rust type the command returns (see
// integrations/tauri/generated/), and the backend ships in the same build, so responses are
// used as typed rather than re-validated here.

const updateStateListeners = new Set<(state: UpdateState) => void>();
let pendingUpdate: Update | null = null;
let pendingUpdateState: UpdateState | null = null;

if (typeof window !== "undefined" && isTauri()) {
  const bridge: InventoryDesktopBridge = {
    isDesktop: true,
    activateInventorySync: (moduleId: ImplementedInventoryModuleId) =>
      invoke<string>("activate_inventory_sync", { moduleId }),
    loadInventory: <M extends ImplementedInventoryModuleId>(moduleId: M) =>
      invoke<InventorySyncResult<M>>("load_inventory", { moduleId }),
    syncInventory: <M extends ImplementedInventoryModuleId>(moduleId: M, sessionId: string) =>
      invoke<InventorySyncResult<M> | null>("sync_inventory", { moduleId, sessionId }),
    deactivateInventorySync: (moduleId: ImplementedInventoryModuleId, sessionId: string) =>
      invoke<boolean>("deactivate_inventory_sync", { moduleId, sessionId }),
    toggleVerifiedEntry: (moduleId, entryId: string, nextVerified: boolean) =>
      invoke<InventoryEntryMutationResult>("toggle_verified_entry", { entryId, moduleId, nextVerified }),
    createEntry: (moduleId, input: InventoryEntryInput) =>
      invoke<InventoryEntryMutationResult>("create_entry", { input, moduleId }),
    updateEntry: (moduleId, entryId: string, input: InventoryEntryInput, editContext?: InventoryEntryEditContext) =>
      invoke<InventoryEntryMutationResult>("update_entry", { editContext, entryId, input, moduleId }),
    setArchivedEntry: (moduleId, entryId: string, archived: boolean) =>
      invoke<InventoryEntryMutationResult>("set_archived_entry", { archived, entryId, moduleId }),
    deleteEntry: (moduleId, entryId: string) =>
      invoke<InventoryDeleteMutationResult>("delete_entry", { entryId, moduleId }),
    createLabPart: (input: PartInput) => invoke<CatalogMutationResult<Part>>("create_lab_part", { input }),
    updateLabPart: (partId: string, input: PartInput) =>
      invoke<CatalogMutationResult<Part>>("update_lab_part", { input, partId }),
    createLabSimpleComponent: (input: SimpleComponentInput) =>
      invoke<CatalogMutationResult<SimpleComponentValue>>("create_lab_simple_component", { input }),
    updateLabSimpleComponent: (partId: string, input: SimpleComponentInput) =>
      invoke<CatalogMutationResult<SimpleComponentValue>>("update_lab_simple_component", { input, partId }),
    deleteLabPart: (partId: string) => invoke<CatalogDeleteResult>("delete_lab_part", { partId }),
    createLabStorageArea: (input: StorageAreaInput) =>
      invoke<CatalogMutationResult<StorageArea>>("create_lab_storage_area", { input }),
    updateLabStorageArea: (areaUuid: string, input: StorageAreaInput) =>
      invoke<CatalogMutationResult<StorageArea>>("update_lab_storage_area", { areaUuid, input }),
    deleteLabStorageArea: (areaUuid: string) => invoke<CatalogDeleteResult>("delete_lab_storage_area", { areaUuid }),
    createLabStorageContainer: (input: StorageContainerInput) =>
      invoke<CatalogMutationResult<StorageContainer>>("create_lab_storage_container", { input }),
    updateLabStorageContainer: (containerUuid: string, input: StorageContainerInput) =>
      invoke<CatalogMutationResult<StorageContainer>>("update_lab_storage_container", { containerUuid, input }),
    deleteLabStorageContainer: (containerUuid: string) =>
      invoke<CatalogDeleteResult>("delete_lab_storage_container", { containerUuid }),
    createLabStockPlacement: (input: StockPlacementInput) =>
      invoke<CatalogMutationResult<StockPlacement>>("create_lab_stock_placement", { input }),
    updateLabStockPlacement: (placementUuid: string, input: StockPlacementInput) =>
      invoke<CatalogMutationResult<StockPlacement>>("update_lab_stock_placement", { input, placementUuid }),
    deleteLabStockPlacement: (placementUuid: string) =>
      invoke<CatalogDeleteResult>("delete_lab_stock_placement", { placementUuid }),
    moveLabStock: (input: StockMoveInput) =>
      invoke<CatalogMutationResult<StockPlacement[]>>("move_lab_stock", { input }),
    countLabStock: (placementUuid: string, input: StockCountInput) =>
      invoke<CatalogMutationResult<StockPlacement>>("count_lab_stock", { input, placementUuid }),
    openExternal: (url: string) => invoke<boolean>("open_external", { url }),
    openPath: (path: string) => invoke<boolean>("open_path", { path }),
    loadPicturePreview: async (path: string) => {
      const previewPath = await invoke<string | null>("load_picture_preview", { path });
      return previewPath ? convertFileSrc(previewPath) : null;
    },
    pickPicturePath: () => invoke<string | null>("pick_picture_path"),
    pickCalibrationRosterFile: (moduleId) => invoke<string | null>("pick_calibration_roster_file", { moduleId }),
    previewCalibrationRoster: (moduleId, path: string) =>
      invoke<CalibrationRosterPreviewReport>("preview_calibration_roster", { moduleId, path }),
    commitCalibrationRoster: (moduleId, input: CalibrationRosterCommitInput) =>
      invoke<CalibrationRosterCommitResult>("commit_calibration_roster", { input, moduleId }),
    exportExcel: (moduleId: ImplementedInventoryModuleId) => invoke<ExcelExportResult>("export_excel", { moduleId }),
    exportLabOrderRequest: (lines: LabOrderRequestLineInput[]) =>
      invoke<ExcelExportResult>("export_lab_order_request", { lines }),
    checkForUpdate,
    downloadUpdate,
    installUpdate,
    onSharedInventoryChanged: listenToSharedInventoryChanged,
    onUpdateStateChanged: listenToUpdateStateChanged,
  };
  window.inventoryDesktop = bridge;
}

async function checkForUpdate(): Promise<UpdateState> {
  publishUpdateState({
    available: false,
    currentVersion: APP_VERSION,
    status: "checking",
  });

  try {
    const update = await check();
    pendingUpdate?.close().catch(() => undefined);
    pendingUpdate = update;

    if (!update) {
      return publishUpdateState({
        available: false,
        currentVersion: APP_VERSION,
        notes: "Inventory Management is up to date.",
        status: "not-available",
      });
    }

    pendingUpdateState = updateStateFromUpdate(update, "available");
    return publishUpdateState(pendingUpdateState);
  } catch (error) {
    pendingUpdate = null;
    pendingUpdateState = null;
    return publishUpdateState(errorUpdateState(error));
  }
}

async function downloadUpdate(): Promise<UpdateState> {
  let update = pendingUpdate;
  if (!update) {
    const state = await checkForUpdate();
    if (!pendingUpdate || !state.available) {
      return state;
    }
    update = pendingUpdate;
  }

  let totalBytes: number | undefined;
  let downloadedBytes = 0;
  const downloadingState = updateStateFromUpdate(update, "downloading", {
    downloadPhase: "copying",
    downloadProgress: 0,
  });
  pendingUpdateState = downloadingState;
  publishUpdateState(downloadingState);

  try {
    await update.download((event) => {
      const nextState = updateDownloadState(update, event, totalBytes, downloadedBytes);
      if (event.event === "Started") {
        totalBytes = event.data.contentLength;
        downloadedBytes = 0;
      } else if (event.event === "Progress") {
        downloadedBytes += event.data.chunkLength;
      }
      pendingUpdateState = nextState;
      publishUpdateState(nextState);
    });

    pendingUpdateState = updateStateFromUpdate(update, "ready", {
      downloadPhase: "ready",
      downloadProgress: 100,
    });
    return publishUpdateState(pendingUpdateState);
  } catch (error) {
    pendingUpdate = null;
    pendingUpdateState = null;
    return publishUpdateState(errorUpdateState(error));
  }
}

async function installUpdate(): Promise<UpdateState> {
  const update = pendingUpdate;
  if (!update) {
    return publishUpdateState({
      available: false,
      currentVersion: APP_VERSION,
      error: "Download the update before installing it.",
      status: "error",
    });
  }

  const installingState = updateStateFromUpdate(update, "installing");
  publishUpdateState(installingState);

  try {
    await update.install();
    pendingUpdate = null;
    pendingUpdateState = installingState;
    return publishUpdateState(installingState);
  } catch (error) {
    return publishUpdateState(errorUpdateState(error, update));
  }
}

function listenToUpdateStateChanged(callback: (state: UpdateState) => void): () => void {
  updateStateListeners.add(callback);
  if (pendingUpdateState) {
    callback(pendingUpdateState);
  }

  return () => {
    updateStateListeners.delete(callback);
  };
}

function listenToSharedInventoryChanged(
  callback: (payload: InventorySharedChangedPayload) => void,
): () => void {
  let disposed = false;
  let unlisten: UnlistenFn | null = null;

  void listen<InventorySharedChangedPayload>("inventory:shared-changed", (event) => {
    callback(event.payload);
  })
    .then((nextUnlisten) => {
      if (disposed) {
        nextUnlisten();
        return;
      }

      unlisten = nextUnlisten;
    })
    .catch(() => undefined);

  return () => {
    disposed = true;
    unlisten?.();
    unlisten = null;
  };
}

function publishUpdateState(state: UpdateState): UpdateState {
  pendingUpdateState = state;
  for (const listener of updateStateListeners) {
    listener(state);
  }
  return state;
}

function updateStateFromUpdate(
  update: Update,
  status: UpdateState["status"],
  overrides: Partial<UpdateState> = {},
): UpdateState {
  return {
    available: true,
    currentVersion: update.currentVersion || APP_VERSION,
    latestVersion: update.version,
    notes: update.body,
    publishedAt: update.date,
    status,
    ...overrides,
  };
}

function updateDownloadState(
  update: Update,
  event: DownloadEvent,
  previousTotalBytes: number | undefined,
  previousDownloadedBytes: number,
): UpdateState {
  if (event.event === "Started") {
    return updateStateFromUpdate(update, "downloading", {
      downloadPhase: "copying",
      downloadProgress: event.data.contentLength ? 0 : undefined,
    });
  }

  if (event.event === "Finished") {
    return updateStateFromUpdate(update, "downloading", {
      downloadPhase: "verifying",
      downloadProgress: 100,
    });
  }

  const totalBytes = previousTotalBytes;
  const nextDownloadedBytes = previousDownloadedBytes + event.data.chunkLength;
  const downloadProgress =
    totalBytes && totalBytes > 0 ? Math.min(99, Math.round((nextDownloadedBytes / totalBytes) * 100)) : undefined;

  return updateStateFromUpdate(update, "downloading", {
    downloadPhase: "copying",
    downloadProgress,
  });
}

function errorUpdateState(error: unknown, update?: Update): UpdateState {
  const message = error instanceof Error ? error.message : "Update failed.";
  if (update) {
    return updateStateFromUpdate(update, "error", { error: message });
  }

  return {
    available: false,
    currentVersion: APP_VERSION,
    error: message,
    status: "error",
  };
}
