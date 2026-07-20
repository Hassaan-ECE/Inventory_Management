import type {
  ExcelExportResult,
  ImportCommitInput,
  ImportCommitResult,
  ImportDryRunReport,
  InventoryDeleteMutationResult as TeInventoryDeleteMutationResult,
  InventoryEntry as TeInventoryEntry,
  InventoryEntryEditContext as TeInventoryEntryEditContext,
  InventoryEntryInput as TeInventoryEntryInput,
  InventoryEntryMutationResult as TeInventoryEntryMutationResult,
  InventoryQueryInput as TeInventoryQueryInput,
  InventoryQueryResult as TeInventoryQueryResult,
  InventorySharedStatus as TeInventorySharedStatus,
  UpdateState,
} from "@/modules/te-test-equipment/types";
import type {
  InventoryDeleteMutationResult as LabInventoryDeleteMutationResult,
  InventoryEntry as LabInventoryEntry,
  InventoryEntryEditContext as LabInventoryEntryEditContext,
  InventoryEntryInput as LabInventoryEntryInput,
  InventoryEntryMutationResult as LabInventoryEntryMutationResult,
  InventoryQueryInput as LabInventoryQueryInput,
  InventoryQueryResult as LabInventoryQueryResult,
  InventorySharedStatus as LabInventorySharedStatus,
} from "@/modules/te-lab-components/types";
import type { ModuleId } from "@/platform/modules/types";

export type ImplementedInventoryModuleId = "te-test-equipment" | "te-lab-components";

interface InventoryModuleContract {
  "te-test-equipment": {
    deleteMutationResult: TeInventoryDeleteMutationResult;
    editContext: TeInventoryEntryEditContext;
    entry: TeInventoryEntry;
    entryInput: TeInventoryEntryInput;
    entryMutationResult: TeInventoryEntryMutationResult;
    queryInput: TeInventoryQueryInput;
    queryResult: TeInventoryQueryResult;
    sharedStatus: TeInventorySharedStatus;
  };
  "te-lab-components": {
    deleteMutationResult: LabInventoryDeleteMutationResult;
    editContext: LabInventoryEntryEditContext;
    entry: LabInventoryEntry;
    entryInput: LabInventoryEntryInput;
    entryMutationResult: LabInventoryEntryMutationResult;
    queryInput: LabInventoryQueryInput;
    queryResult: LabInventoryQueryResult;
    sharedStatus: LabInventorySharedStatus;
  };
}

export type InventoryEntryFor<M extends ImplementedInventoryModuleId> = InventoryModuleContract[M]["entry"];
export type InventoryEntryInputFor<M extends ImplementedInventoryModuleId> = InventoryModuleContract[M]["entryInput"];
export type InventoryEntryEditContextFor<M extends ImplementedInventoryModuleId> =
  InventoryModuleContract[M]["editContext"];
export type InventoryEntryMutationResultFor<M extends ImplementedInventoryModuleId> =
  InventoryModuleContract[M]["entryMutationResult"];
export type InventoryDeleteMutationResultFor<M extends ImplementedInventoryModuleId> =
  InventoryModuleContract[M]["deleteMutationResult"];
export type InventoryQueryInputFor<M extends ImplementedInventoryModuleId> = InventoryModuleContract[M]["queryInput"];
export type InventoryQueryResultFor<M extends ImplementedInventoryModuleId> = InventoryModuleContract[M]["queryResult"];

export interface InventorySyncResult<
  M extends ImplementedInventoryModuleId = "te-test-equipment",
> {
  dbPath: string;
  entries: InventoryEntryFor<M>[];
  entriesChanged?: boolean;
  shared: InventoryModuleContract[M]["sharedStatus"];
}

export interface InventorySharedChangedPayload {
  systemId: ModuleId;
}

export interface InventoryDesktopBridge {
  isDesktop: boolean;
  activateInventorySync: (moduleId: ImplementedInventoryModuleId) => Promise<string>;
  loadInventory: <M extends ImplementedInventoryModuleId>(moduleId: M) => Promise<InventorySyncResult<M>>;
  queryInventory?: <M extends ImplementedInventoryModuleId>(
    moduleId: M,
    input: InventoryQueryInputFor<M>,
  ) => Promise<InventoryQueryResultFor<M>>;
  syncInventory: <M extends ImplementedInventoryModuleId>(
    moduleId: M,
    sessionId: string,
  ) => Promise<InventorySyncResult<M> | null>;
  deactivateInventorySync: (moduleId: ImplementedInventoryModuleId, sessionId: string) => Promise<boolean>;
  toggleVerifiedEntry: <M extends ImplementedInventoryModuleId>(
    moduleId: M,
    entryId: string,
    nextVerified: boolean,
  ) => Promise<InventoryEntryMutationResultFor<M>>;
  createEntry: <M extends ImplementedInventoryModuleId>(
    moduleId: M,
    input: InventoryEntryInputFor<M>,
  ) => Promise<InventoryEntryMutationResultFor<M>>;
  updateEntry: <M extends ImplementedInventoryModuleId>(
    moduleId: M,
    entryId: string,
    input: InventoryEntryInputFor<M>,
    editContext?: InventoryEntryEditContextFor<M>,
  ) => Promise<InventoryEntryMutationResultFor<M>>;
  setArchivedEntry: <M extends ImplementedInventoryModuleId>(
    moduleId: M,
    entryId: string,
    archived: boolean,
  ) => Promise<InventoryEntryMutationResultFor<M>>;
  deleteEntry: <M extends ImplementedInventoryModuleId>(
    moduleId: M,
    entryId: string,
  ) => Promise<InventoryDeleteMutationResultFor<M>>;
  openExternal?: (url: string) => Promise<boolean>;
  openPath?: (path: string) => Promise<boolean>;
  loadPicturePreview?: (path: string) => Promise<string | null>;
  pickPicturePath?: () => Promise<string | null>;
  pickImportFile: (moduleId: "te-test-equipment") => Promise<string | null>;
  previewImport: (moduleId: "te-test-equipment", path: string) => Promise<ImportDryRunReport>;
  commitImport: (moduleId: "te-test-equipment", input: ImportCommitInput) => Promise<ImportCommitResult>;
  exportExcel?: (moduleId: ImplementedInventoryModuleId) => Promise<ExcelExportResult>;
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
