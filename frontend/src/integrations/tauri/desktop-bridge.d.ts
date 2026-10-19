import type {
  CalibrationRosterCommitInput,
  CalibrationRosterCommitResult,
  CalibrationRosterPreviewReport,
  ExcelExportResult,
  InventoryDeleteMutationResult,
  InventoryEntryEditContext,
  InventoryEntryInput,
  InventoryEntryMutationResult,
  InventorySyncResult as TeInventorySyncResult,
} from "@/modules/te-test-equipment/types";
import type {
  CatalogDeleteResult,
  CatalogMutationResult,
  CatalogSyncResult,
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
import type { ModuleId } from "@/platform/modules/types";
import type { UpdateState } from "@/platform/updates/updateState";

/** The inventories this bridge serves; EVCS Storage has its own bridge (`storageBridge.ts`). */
export type ImplementedInventoryModuleId = "te-test-equipment" | "te-lab-components";

type TeModuleId = "te-test-equipment";

export type InventorySyncResult<
  M extends ImplementedInventoryModuleId = TeModuleId,
> = M extends TeModuleId ? TeInventorySyncResult : CatalogSyncResult;

export interface InventorySharedChangedPayload {
  systemId: ModuleId;
}

export interface InventoryDesktopBridge {
  isDesktop: boolean;
  activateInventorySync: (moduleId: ImplementedInventoryModuleId) => Promise<string>;
  loadInventory: <M extends ImplementedInventoryModuleId>(moduleId: M) => Promise<InventorySyncResult<M>>;
  syncInventory: <M extends ImplementedInventoryModuleId>(
    moduleId: M,
    sessionId: string,
  ) => Promise<InventorySyncResult<M> | null>;
  deactivateInventorySync: (moduleId: ImplementedInventoryModuleId, sessionId: string) => Promise<boolean>;
  toggleVerifiedEntry: (
    moduleId: TeModuleId,
    entryId: string,
    nextVerified: boolean,
  ) => Promise<InventoryEntryMutationResult>;
  createEntry: (moduleId: TeModuleId, input: InventoryEntryInput) => Promise<InventoryEntryMutationResult>;
  updateEntry: (
    moduleId: TeModuleId,
    entryId: string,
    input: InventoryEntryInput,
    editContext?: InventoryEntryEditContext,
  ) => Promise<InventoryEntryMutationResult>;
  setArchivedEntry: (
    moduleId: TeModuleId,
    entryId: string,
    archived: boolean,
  ) => Promise<InventoryEntryMutationResult>;
  deleteEntry: (moduleId: TeModuleId, entryId: string) => Promise<InventoryDeleteMutationResult>;
  createLabPart?: (input: PartInput) => Promise<CatalogMutationResult<Part>>;
  updateLabPart?: (partId: string, input: PartInput) => Promise<CatalogMutationResult<Part>>;
  createLabSimpleComponent?: (input: SimpleComponentInput) => Promise<CatalogMutationResult<SimpleComponentValue>>;
  updateLabSimpleComponent?: (
    partId: string,
    input: SimpleComponentInput,
  ) => Promise<CatalogMutationResult<SimpleComponentValue>>;
  deleteLabPart?: (partId: string) => Promise<CatalogDeleteResult>;
  createLabStorageArea?: (input: StorageAreaInput) => Promise<CatalogMutationResult<StorageArea>>;
  updateLabStorageArea?: (areaUuid: string, input: StorageAreaInput) => Promise<CatalogMutationResult<StorageArea>>;
  deleteLabStorageArea?: (areaUuid: string) => Promise<CatalogDeleteResult>;
  createLabStorageContainer?: (input: StorageContainerInput) => Promise<CatalogMutationResult<StorageContainer>>;
  updateLabStorageContainer?: (
    containerUuid: string,
    input: StorageContainerInput,
  ) => Promise<CatalogMutationResult<StorageContainer>>;
  deleteLabStorageContainer?: (containerUuid: string) => Promise<CatalogDeleteResult>;
  createLabStockPlacement?: (input: StockPlacementInput) => Promise<CatalogMutationResult<StockPlacement>>;
  updateLabStockPlacement?: (
    placementUuid: string,
    input: StockPlacementInput,
  ) => Promise<CatalogMutationResult<StockPlacement>>;
  deleteLabStockPlacement?: (placementUuid: string) => Promise<CatalogDeleteResult>;
  moveLabStock?: (input: StockMoveInput) => Promise<CatalogMutationResult<StockPlacement[]>>;
  countLabStock?: (placementUuid: string, input: StockCountInput) => Promise<CatalogMutationResult<StockPlacement>>;
  openExternal?: (url: string) => Promise<boolean>;
  openPath?: (path: string) => Promise<boolean>;
  loadPicturePreview?: (path: string) => Promise<string | null>;
  pickPicturePath?: () => Promise<string | null>;
  pickCalibrationRosterFile?: (moduleId: TeModuleId) => Promise<string | null>;
  previewCalibrationRoster?: (moduleId: TeModuleId, path: string) => Promise<CalibrationRosterPreviewReport>;
  commitCalibrationRoster?: (
    moduleId: TeModuleId,
    input: CalibrationRosterCommitInput,
  ) => Promise<CalibrationRosterCommitResult>;
  exportExcel?: (moduleId: ImplementedInventoryModuleId) => Promise<ExcelExportResult>;
  exportLabOrderRequest?: (lines: LabOrderRequestLineInput[]) => Promise<ExcelExportResult>;
  checkForUpdate?: () => Promise<UpdateState>;
  downloadUpdate?: () => Promise<UpdateState>;
  installUpdate?: () => Promise<UpdateState>;
  onSharedInventoryChanged?: (callback: (payload: InventorySharedChangedPayload) => void) => () => void;
  onUpdateStateChanged?: (callback: (state: UpdateState) => void) => () => void;
}

declare global {
  interface Window {
    inventoryDesktop?: InventoryDesktopBridge;
  }
}
