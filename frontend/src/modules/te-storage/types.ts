import type { InventoryEntry } from "@/integrations/tauri/generated/te-storage/InventoryEntry";
import type { InventoryEntryInput } from "@/integrations/tauri/generated/te-storage/InventoryEntryInput";
import type { InventorySharedStatus } from "@/platform/sync/sharedStatus";

/** EVCS Storage records and inputs, generated from the Rust types. */
export type StorageEntry = InventoryEntry;
export type StorageInput = InventoryEntryInput;

export const STORAGE_COLUMNS = [
  { key: "pn", label: "PN #", width: 130 },
  { key: "pr", label: "PR #", width: 130 },
  { key: "po", label: "PO #", width: 130 },
  { key: "manufacturer", label: "Manufacturer", width: 170 },
  { key: "model", label: "Model", width: 160 },
  { key: "description", label: "Description", width: 240 },
  { key: "qty", label: "Qty", width: 80 },
  { key: "location", label: "Location", width: 150 },
  { key: "notes", label: "Notes", width: 240 },
] as const satisfies readonly { key: keyof StorageInput; label: string; width: number }[];

export type StorageColumn = typeof STORAGE_COLUMNS[number]["key"];
export interface StorageSnapshot {
  entries: StorageEntry[];
  shared: InventorySharedStatus;
}
export interface StorageMutation { entry: StorageEntry; shared: InventorySharedStatus; message: string }
export interface StorageBridge {
  load: () => Promise<StorageSnapshot>;
  activate: () => Promise<string>;
  deactivate: (sessionId: string) => Promise<unknown>;
  sync: (sessionId: string) => Promise<StorageSnapshot | null>;
  subscribe: (callback: () => void) => Promise<() => void>;
  save: (input: StorageInput, original: StorageEntry | null) => Promise<StorageMutation>;
  remove: (entry: StorageEntry) => Promise<{ shared: InventorySharedStatus; message: string }>;
}
export function entryInput(entry?: StorageEntry | null): StorageInput {
  return {
    pn: entry?.pn ?? "", pr: entry?.pr ?? "", po: entry?.po ?? "",
    manufacturer: entry?.manufacturer ?? "", model: entry?.model ?? "",
    description: entry?.description ?? "", qty: entry?.qty ?? null,
    location: entry?.location ?? "", notes: entry?.notes ?? "",
  };
}
