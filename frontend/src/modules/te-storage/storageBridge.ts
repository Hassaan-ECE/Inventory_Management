import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { STORAGE_COLUMNS, type StorageBridge, type StorageEntry, type StorageMutation, type StorageSharedStatus, type StorageSnapshot } from "./types";

const moduleId = "te-storage";
function object(value: unknown): Record<string, unknown> {
  if (!value || typeof value !== "object" || Array.isArray(value)) throw new Error("Invalid TE Storage response.");
  return value as Record<string, unknown>;
}
function entry(value: unknown): StorageEntry {
  const row = object(value);
  for (const column of STORAGE_COLUMNS) {
    if (column.key === "qty") {
      if (row.qty !== null && (typeof row.qty !== "number" || !Number.isFinite(row.qty) || row.qty < 0 || row.qty > 1_000_000)) throw new Error("Invalid TE Storage quantity.");
    } else if (typeof row[column.key] !== "string") throw new Error(`Invalid TE Storage ${column.label}.`);
  }
  for (const key of ["id", "entryUuid", "createdAt", "updatedAt"]) {
    if (typeof row[key] !== "string" || !row[key]) throw new Error("Invalid TE Storage record identity.");
  }
  if (typeof row.archived !== "boolean") throw new Error("Invalid TE Storage record state.");
  return row as unknown as StorageEntry;
}
function shared(value: unknown): StorageSharedStatus {
  const status = object(value);
  if (["available", "enabled", "canModify"].some(key => typeof status[key] !== "boolean") ||
      typeof status.message !== "string" || typeof status.mutationMode !== "string" ||
      (status.hasLocalOnlyChanges !== null && typeof status.hasLocalOnlyChanges !== "boolean")) {
    throw new Error("Invalid TE Storage sync status.");
  }
  return status as unknown as StorageSharedStatus;
}
export function parseStorageSnapshot(value: unknown): StorageSnapshot {
  const payload = object(value);
  if (!Array.isArray(payload.entries)) throw new Error("Invalid TE Storage inventory.");
  return { entries: payload.entries.map(entry), shared: shared(payload.shared) };
}
const desktopBridge: StorageBridge = {
  load: () => invoke("load_inventory", { moduleId }).then(parseStorageSnapshot),
  activate: async () => {
    const token = await invoke<unknown>("activate_inventory_sync", { moduleId });
    if (typeof token !== "string" || !token) throw new Error("Could not start TE Storage sync.");
    return token;
  },
  deactivate: sessionId => invoke("deactivate_inventory_sync", { moduleId, sessionId }),
  sync: async sessionId => {
    const result = await invoke("sync_inventory", { moduleId, sessionId });
    return result === null ? null : parseStorageSnapshot(result);
  },
  subscribe: callback => listen<{ systemId: string }>("inventory:shared-changed", event => {
    if (event.payload.systemId === moduleId) callback();
  }),
  save: async (input, original) => {
    const editContext = original ? {
      baseVersion: original.updatedAt,
      changedFields: STORAGE_COLUMNS.filter(({ key }) => original[key] !== input[key]).map(({ key }) => key),
    } : undefined;
    const payload = object(await invoke(original ? "update_entry" : "create_entry", {
      moduleId, input, ...(original ? { entryId: original.entryUuid, editContext } : {}),
    }));
    return { entry: entry(payload.entry), shared: shared(payload.shared), message: String(payload.message ?? "Saved.") } satisfies StorageMutation;
  },
  remove: async original => {
    const payload = object(await invoke("delete_entry", { moduleId, entryId: original.entryUuid }));
    return { shared: shared(payload.shared), message: String(payload.message ?? "Item deleted.") };
  },
};
export function getStorageBridge(): StorageBridge | null { return isTauri() ? desktopBridge : null; }
