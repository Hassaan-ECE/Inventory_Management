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
] as const;

export type StorageColumn = typeof STORAGE_COLUMNS[number]["key"];
export type StorageInput = Record<Exclude<StorageColumn, "qty">, string> & { qty: number | null };
export interface StorageEntry extends StorageInput {
  id: string;
  entryUuid: string;
  createdAt: string;
  updatedAt: string;
  archived: boolean;
}
export interface StorageSharedStatus {
  available: boolean;
  enabled: boolean;
  canModify: boolean;
  hasLocalOnlyChanges: boolean | null;
  message: string;
  mutationMode: string;
}
export interface StorageSnapshot {
  entries: StorageEntry[];
  shared: StorageSharedStatus;
}
export interface StorageMutation { entry: StorageEntry; shared: StorageSharedStatus; message: string }
export interface StorageBridge {
  load: () => Promise<StorageSnapshot>;
  activate: () => Promise<string>;
  deactivate: (sessionId: string) => Promise<unknown>;
  sync: (sessionId: string) => Promise<StorageSnapshot | null>;
  subscribe: (callback: () => void) => Promise<() => void>;
  save: (input: StorageInput, original: StorageEntry | null) => Promise<StorageMutation>;
  remove: (entry: StorageEntry) => Promise<{ shared: StorageSharedStatus; message: string }>;
}
export function entryInput(entry?: StorageEntry | null): StorageInput {
  return {
    pn: entry?.pn ?? "", pr: entry?.pr ?? "", po: entry?.po ?? "",
    manufacturer: entry?.manufacturer ?? "", model: entry?.model ?? "",
    description: entry?.description ?? "", qty: entry?.qty ?? null,
    location: entry?.location ?? "", notes: entry?.notes ?? "",
  };
}
