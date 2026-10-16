/** Shared-folder sync status every inventory receives with loads, syncs, and mutations. */
export interface InventorySharedStatus {
  available: boolean;
  canModify: boolean;
  enabled: boolean;
  hasLocalOnlyChanges?: boolean;
  message: string;
  mutationMode?: "shared" | "local";
  revision?: string;
  lastSnapshotId?: string;
  sharedRootPath?: string;
  warning?: SyncWarning;
}

/** A problem the footer flags beside the status message (mirrors the backend `SyncWarning`). */
export type SyncWarning =
  | { kind: "localSaveFailing" }
  | { kind: "databaseNearlyFull"; percentUsed: number }
  | { kind: "updateAvailable"; version: string | null }
  | { kind: "skippedFiles"; count: number };

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function optionalString(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function finiteNumber(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

/** Unknown or malformed warnings are dropped rather than failing the whole response. */
function parseSyncWarning(value: unknown): SyncWarning | undefined {
  if (!isRecord(value)) return undefined;
  switch (value.kind) {
    case "localSaveFailing":
      return { kind: "localSaveFailing" };
    case "databaseNearlyFull": {
      const percentUsed = finiteNumber(value.percentUsed);
      return percentUsed === null ? undefined : { kind: "databaseNearlyFull", percentUsed };
    }
    case "updateAvailable":
      return { kind: "updateAvailable", version: optionalString(value.version) ?? null };
    case "skippedFiles": {
      const count = finiteNumber(value.count);
      return count === null ? undefined : { kind: "skippedFiles", count };
    }
    default:
      return undefined;
  }
}

export function parseSharedStatus(value: unknown): InventorySharedStatus {
  const record = isRecord(value) ? value : {};
  return {
    available: record.available === true,
    canModify: record.canModify !== false,
    enabled: record.enabled === true,
    hasLocalOnlyChanges: typeof record.hasLocalOnlyChanges === "boolean" ? record.hasLocalOnlyChanges : undefined,
    lastSnapshotId: optionalString(record.lastSnapshotId),
    message: optionalString(record.message) ?? "Shared sync status unavailable.",
    mutationMode: record.mutationMode === "shared" ? "shared" : "local",
    revision: optionalString(record.revision),
    sharedRootPath: optionalString(record.sharedRootPath),
    warning: parseSyncWarning(record.warning),
  };
}

export function syncWarningsMatch(left?: SyncWarning, right?: SyncWarning): boolean {
  return JSON.stringify(left ?? null) === JSON.stringify(right ?? null);
}

/** Short footer label and the longer explanation shown on hover. */
export function describeSyncWarning(warning: SyncWarning): { label: string; detail: string } {
  switch (warning.kind) {
    case "localSaveFailing":
      return {
        label: "Changes not saving",
        detail: "This computer could not write to its local database, so recent changes may be lost when the app closes. Tell the app owner before closing the app.",
      };
    case "databaseNearlyFull":
      return {
        label: `Local database ${warning.percentUsed}% full`,
        detail: "The local database is nearly full. Tell the app owner so it can be cleaned up before changes stop saving.",
      };
    case "updateAvailable":
      return {
        label: "Update needed",
        detail: warning.version
          ? `A teammate is using Inventory Management ${warning.version}. Install the update to keep syncing everything.`
          : "Some shared changes come from a newer version of Inventory Management. Install the update to read them.",
      };
    case "skippedFiles":
      return {
        label: `${warning.count} shared ${warning.count === 1 ? "file" : "files"} skipped`,
        detail: "Some files in the shared folder could not be read and were skipped. Changes in them are not shown here.",
      };
  }
}
