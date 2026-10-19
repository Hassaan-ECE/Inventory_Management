import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { InventoryDeleteMutationResult } from "@/integrations/tauri/generated/te-storage/InventoryDeleteMutationResult";
import type { InventoryEntryMutationResult } from "@/integrations/tauri/generated/te-storage/InventoryEntryMutationResult";
import type { InventorySyncResult } from "@/integrations/tauri/generated/te-storage/InventorySyncResult";
import { STORAGE_COLUMNS, type StorageBridge } from "./types";

const moduleId = "te-storage";

// Payload types are generated from the Rust types and the backend ships in the same build,
// so responses are used as typed.
const desktopBridge: StorageBridge = {
  load: () => invoke<InventorySyncResult>("load_inventory", { moduleId }),
  activate: () => invoke<string>("activate_inventory_sync", { moduleId }),
  deactivate: sessionId => invoke("deactivate_inventory_sync", { moduleId, sessionId }),
  sync: sessionId => invoke<InventorySyncResult | null>("sync_inventory", { moduleId, sessionId }),
  subscribe: callback => listen<{ systemId: string }>("inventory:shared-changed", event => {
    if (event.payload.systemId === moduleId) callback();
  }),
  save: async (input, original) => {
    const editContext = original ? {
      baseVersion: original.updatedAt,
      changedFields: STORAGE_COLUMNS.filter(({ key }) => original[key] !== input[key]).map(({ key }) => key),
    } : undefined;
    return invoke<InventoryEntryMutationResult>(original ? "update_entry" : "create_entry", {
      moduleId, input, ...(original ? { entryId: original.entryUuid, editContext } : {}),
    });
  },
  remove: original => invoke<InventoryDeleteMutationResult>("delete_entry", { moduleId, entryId: original.entryUuid }),
};
export function getStorageBridge(): StorageBridge | null { return isTauri() ? desktopBridge : null; }
