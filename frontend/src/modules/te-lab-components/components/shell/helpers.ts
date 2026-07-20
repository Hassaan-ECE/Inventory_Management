import { APP_VERSION } from "@/app/branding";
import { buildDefaultColumnVisibility, mergeColumnVisibility } from "@/modules/te-lab-components/lib";
import type {
  ColumnKey,
  InventoryEntry,
  InventoryEntryInput,
  InventorySharedStatus,
  UpdateState,
} from "@/modules/te-lab-components/types";

export { THEME_STORAGE_KEY, readTheme } from "@/platform/ui/theme";
export const COLOR_ROWS_STORAGE_KEY = "teLabComponentsInventory.colorRows";
export const COLUMN_VISIBILITY_STORAGE_KEY = "teLabComponentsInventory.columnVisibility";
export const UPDATE_CHECK_INTERVAL_MS = 5 * 60_000;

export const MOCK_SHARED_STATUS: InventorySharedStatus = {
  available: true,
  canModify: true,
  enabled: false,
  message: "",
  mutationMode: "shared",
};

export const DESKTOP_SHARED_PENDING_STATUS: InventorySharedStatus = {
  available: false,
  canModify: false,
  enabled: false,
  message: "Checking shared workspace...",
  mutationMode: "local",
};

export function buildIdleUpdateState(): UpdateState {
  return {
    available: false,
    currentVersion: APP_VERSION,
    status: "idle",
  };
}

export function chooseFreshUpdateState(current: UpdateState, next: UpdateState): UpdateState {
  if (current.latestVersion && current.latestVersion === next.latestVersion) {
    return getUpdateStatusRank(current.status) > getUpdateStatusRank(next.status) ? current : next;
  }

  return next;
}

function getUpdateStatusRank(status: UpdateState["status"]): number {
  switch (status) {
    case "idle":
      return 0;
    case "checking":
      return 1;
    case "not-available":
      return 2;
    case "available":
      return 3;
    case "downloading":
      return 4;
    case "ready":
      return 5;
    case "installing":
      return 6;
    case "error":
      return 7;
    default:
      return 0;
  }
}

export function sharedStatusesMatch(left: InventorySharedStatus, right: InventorySharedStatus): boolean {
  return (
    left.available === right.available &&
    left.canModify === right.canModify &&
    left.enabled === right.enabled &&
    left.hasLocalOnlyChanges === right.hasLocalOnlyChanges &&
    left.message === right.message &&
    left.mutationMode === right.mutationMode &&
    left.revision === right.revision &&
    left.lastSnapshotId === right.lastSnapshotId &&
    left.sharedRootPath === right.sharedRootPath
  );
}

export function normalizeSharedStatus(status: InventorySharedStatus): InventorySharedStatus {
  return status;
}

export function hasDesktopBridge(): boolean {
  return typeof window !== "undefined" && Boolean(window.inventoryDesktop?.isDesktop);
}

/**
 * Transient status only. Mode is already shown by the Shared/Local pill, so suppress
 * healthy idle sync chatter like "Shared operation sync ready."
 */
export function buildDefaultStatusMessage(
  _totalCount: number,
  _verifiedCount: number,
  dataSource: "desktop" | "mock",
  sharedStatus: InventorySharedStatus,
): string {
  if (dataSource !== "desktop" || !sharedStatus.enabled) {
    return "";
  }

  const message = sharedStatus.message.trim();
  if (!message || isRoutineSharedStatusMessage(message)) {
    return "";
  }
  return message;
}

/** True for idle/healthy sync strings that duplicate the Shared pill. */
export function isRoutineSharedStatusMessage(message: string): boolean {
  const normalized = message.trim().replace(/\s+/g, " ");
  if (!normalized) {
    return true;
  }

  const lower = normalized.toLowerCase();
  // Keep anything that needs operator attention or confirms a real action.
  const keepHints = [
    "unavailable",
    "pending local",
    "published",
    "corrupt",
    "failed",
    "disabled",
    "error",
    "permission",
    "queued",
    "could not",
    "denied",
  ];
  if (keepHints.some((hint) => lower.includes(hint))) {
    return false;
  }

  if (/^shared operation sync ready\.?( snapshot refreshed\.)?$/i.test(normalized)) {
    return true;
  }
  if (/^feoxdb local store ready/i.test(normalized)) {
    return true;
  }
  if (/^shared sync (enabled|starting)/i.test(normalized)) {
    return true;
  }

  return false;
}

export function buildLocalCreatedEntry(input: InventoryEntryInput): InventoryEntry {
  const timestamp = new Date().toISOString();

  return {
    id: `local-${Date.now()}`,
    entryUuid: "",
    assetNumber: input.assetNumber,
    serialNumber: input.serialNumber,
    qty: input.qty,
    manufacturer: input.manufacturer,
    model: input.model,
    description: input.description,
    projectName: input.projectName,
    location: input.location,
    assignedTo: input.assignedTo,
    links: input.links,
    notes: input.notes,
    lifecycleStatus: input.lifecycleStatus,
    workingStatus: input.workingStatus,
    condition: input.condition,
    verifiedInSurvey: input.verifiedInSurvey,
    archived: input.archived,
    manualEntry: true,
    picturePath: input.picturePath ?? "",
    createdAt: timestamp,
    updatedAt: timestamp,
  };
}

export function buildLocalUpdatedEntry(existingEntry: InventoryEntry, input: InventoryEntryInput): InventoryEntry {
  return {
    ...existingEntry,
    assetNumber: input.assetNumber,
    serialNumber: input.serialNumber,
    qty: input.qty,
    manufacturer: input.manufacturer,
    model: input.model,
    description: input.description,
    projectName: input.projectName,
    location: input.location,
    assignedTo: input.assignedTo,
    links: input.links,
    notes: input.notes,
    lifecycleStatus: input.lifecycleStatus,
    workingStatus: input.workingStatus,
    condition: input.condition,
    verifiedInSurvey: input.verifiedInSurvey,
    archived: input.archived,
    picturePath: input.picturePath ?? "",
    updatedAt: new Date().toISOString(),
  };
}

export function readColorRows(): boolean {
  if (typeof window === "undefined") {
    return true;
  }

  const storedValue = window.localStorage.getItem(COLOR_ROWS_STORAGE_KEY);
  return storedValue == null ? true : storedValue === "true";
}

export function readColumnVisibility(): Record<ColumnKey, boolean> {
  if (typeof window === "undefined") {
    return buildDefaultColumnVisibility();
  }

  const storedValue = window.localStorage.getItem(COLUMN_VISIBILITY_STORAGE_KEY);
  if (!storedValue) {
    return buildDefaultColumnVisibility();
  }

  try {
    return mergeColumnVisibility(JSON.parse(storedValue) as Partial<Record<ColumnKey, boolean>>);
  } catch {
    return buildDefaultColumnVisibility();
  }
}
