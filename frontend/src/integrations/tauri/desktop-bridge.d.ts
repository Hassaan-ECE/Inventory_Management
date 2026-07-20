import type {
  InventoryDeleteMutationResult,
  ExcelExportResult,
  InventoryEntry,
  InventoryEntryEditContext,
  InventoryEntryInput,
  InventoryEntryMutationResult,
  InventoryQueryInput,
  InventoryQueryResult,
  InventorySharedStatus,
  ImportCommitInput,
  ImportCommitResult,
  ImportDryRunReport,
  UpdateState,
} from "@/modules/te-test-equipment/types";
import type { ModuleId } from "@/platform/modules/types";

export interface InventorySyncResult {
  dbPath: string;
  entries: InventoryEntry[];
  entriesChanged?: boolean;
  shared: InventorySharedStatus;
}

export interface InventorySharedChangedPayload {
  systemId: ModuleId;
}

declare global {
  interface Window {
    inventoryDesktop?: {
      isDesktop: boolean;
      activateInventorySync: () => Promise<string>;
      loadInventory: () => Promise<InventorySyncResult>;
      queryInventory?: (input: InventoryQueryInput) => Promise<InventoryQueryResult>;
      syncInventory: (sessionId: string) => Promise<InventorySyncResult | null>;
      deactivateInventorySync: (sessionId: string) => Promise<boolean>;
      toggleVerifiedEntry: (entryId: string, nextVerified: boolean) => Promise<InventoryEntryMutationResult>;
      createEntry: (input: InventoryEntryInput) => Promise<InventoryEntryMutationResult>;
      updateEntry: (
        entryId: string,
        input: InventoryEntryInput,
        editContext?: InventoryEntryEditContext,
      ) => Promise<InventoryEntryMutationResult>;
      setArchivedEntry: (entryId: string, archived: boolean) => Promise<InventoryEntryMutationResult>;
      deleteEntry: (entryId: string) => Promise<InventoryDeleteMutationResult>;
      openExternal?: (url: string) => Promise<boolean>;
      openPath?: (path: string) => Promise<boolean>;
      loadPicturePreview?: (path: string) => Promise<string | null>;
      pickPicturePath?: () => Promise<string | null>;
      pickImportFile: () => Promise<string | null>;
      previewImport: (path: string) => Promise<ImportDryRunReport>;
      commitImport: (input: ImportCommitInput) => Promise<ImportCommitResult>;
      exportExcel?: () => Promise<ExcelExportResult>;
      checkForUpdate?: () => Promise<UpdateState>;
      downloadUpdate?: () => Promise<UpdateState>;
      installUpdate?: () => Promise<UpdateState>;
      onSharedInventoryChanged?: (callback: (payload: InventorySharedChangedPayload) => void) => () => void;
      onUpdateStateChanged?: (callback: (state: UpdateState) => void) => () => void;
    };
  }
}
