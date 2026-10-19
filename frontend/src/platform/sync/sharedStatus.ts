import type { InventorySharedStatus } from "@/integrations/tauri/generated/platform/InventorySharedStatus";
import type { SyncWarning } from "@/integrations/tauri/generated/platform/SyncWarning";

/** Shared-folder sync status every inventory receives with loads, syncs, and mutations. */
export type { InventorySharedStatus, SyncWarning };

export function syncWarningsMatch(left?: SyncWarning | null, right?: SyncWarning | null): boolean {
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
