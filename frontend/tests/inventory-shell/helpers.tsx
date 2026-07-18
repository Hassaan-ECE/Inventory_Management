import { act } from "@testing-library/react";
import { vi } from "vitest";

import { getInventoryCounts } from "@/features/inventory/lib";
import type { InventorySyncResult } from "@/integrations/tauri/desktop-bridge";
import type {
  InventoryCounts,
  InventoryEntry,
  InventoryQueryResult,
  InventorySharedStatus,
} from "@/features/inventory/types";

export const CONNECTED_SHARED_STATUS: InventorySharedStatus = {
  available: true,
  canModify: true,
  enabled: true,
  message: "",
  mutationMode: "shared",
  syncIntervalMs: 60_000,
};
export const LOCAL_SHARED_STATUS: InventorySharedStatus = {
  available: false,
  canModify: true,
  enabled: true,
  hasLocalOnlyChanges: true,
  message: "Shared workspace unavailable. Saving changes locally.",
  mutationMode: "local",
  syncIntervalMs: 60_000,
};
export const DISABLED_SHARED_STATUS: InventorySharedStatus = {
  available: true,
  canModify: true,
  enabled: false,
  message: "",
  mutationMode: "local",
};
export const EMPTY_TEST_COUNTS: InventoryCounts = {
  archive: 0,
  dueSoon: 0,
  inventory: 0,
  missingDue: 0,
  outToCal: 0,
  overdue: 0,
  total: 0,
  verified: 0,
};
export const TEST_DB_PATH = "C:/Users/Test/AppData/Local/com.te.test.equipment.inventory/inventory.feox";

export function buildTestEntry(overrides: Partial<InventoryEntry> = {}): InventoryEntry {
  return {
    id: "701",
    assetNumber: "TE-701",
    serialNumber: "",
    qty: 1,
    manufacturer: "Sync Maker",
    model: "SM-701",
    description: "Sync test entry",
    projectName: "Shared",
    location: "Bench 1",
    assignedTo: "",
    links: "",
    notes: "",
    lifecycleStatus: "active",
    workingStatus: "working",
    condition: "",
    calibrationRequirement: "unknown",
    outToCalibration: false,
    archived: false,
    createdAt: "2026-04-26T10:00:00.000Z",
    updatedAt: "2026-04-26T10:00:00.000Z",
    entryUuid: "00000000-0000-4000-8000-000000000701",
    manualEntry: true,
    picturePath: "",
    ...overrides,
  } as InventoryEntry;
}

export function buildDesktopQueryResult(shared: InventorySharedStatus, entries: InventoryEntry[] = []): InventoryQueryResult {
  return {
    counts: entries.length === 0 ? EMPTY_TEST_COUNTS : buildInventoryCounts(entries),
    dbPath: TEST_DB_PATH,
    entries,
    shared,
    totalFiltered: entries.length,
  };
}

export function buildDesktopSyncResult(shared: InventorySharedStatus, entries: InventoryEntry[] = []): InventorySyncResult {
  return {
    dbPath: TEST_DB_PATH,
    entries,
    shared,
  };
}

export function buildInventoryCounts(entries: InventoryEntry[]): InventoryCounts {
  return getInventoryCounts(entries, "2026-07-13");
}

export function createDesktopBridge(
  overrides: Partial<NonNullable<Window["inventoryDesktop"]>>,
): NonNullable<Window["inventoryDesktop"]> {
  return {
    isDesktop: true,
    loadInventory: vi.fn().mockResolvedValue({
      dbPath: TEST_DB_PATH,
      entries: [],
      shared: CONNECTED_SHARED_STATUS,
    }),
    queryInventory: vi.fn().mockResolvedValue(buildDesktopQueryResult(CONNECTED_SHARED_STATUS)),
    syncInventory: vi.fn().mockResolvedValue({
      dbPath: TEST_DB_PATH,
      entries: [],
      entriesChanged: false,
      shared: CONNECTED_SHARED_STATUS,
    }),
    toggleVerifiedEntry: vi.fn(),
    createEntry: vi.fn(),
    updateEntry: vi.fn(),
    setArchivedEntry: vi.fn(),
    deleteEntry: vi.fn(),
    openExternal: vi.fn().mockResolvedValue(true),
    openPath: vi.fn().mockResolvedValue(true),
    pickPicturePath: vi.fn().mockResolvedValue(null),
    pickImportFile: vi.fn().mockResolvedValue(null),
    previewImport: vi.fn(),
    commitImport: vi.fn(),
    exportExcel: vi.fn().mockResolvedValue({ canceled: false, outputPath: "D:/exports/TE_Test_Equipment_Inventory_Export.xlsx" }),
    ...overrides,
  } as NonNullable<Window["inventoryDesktop"]>;
}

export function createDeferred<T>(): {
  promise: Promise<T>;
  reject: (reason?: unknown) => void;
  resolve: (value: T) => void;
} {
  let reject!: (reason?: unknown) => void;
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((promiseResolve, promiseReject) => {
    resolve = promiseResolve;
    reject = promiseReject;
  });

  return { promise, reject, resolve };
}

export async function flushAsyncWork(): Promise<void> {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

export async function delay(ms: number): Promise<void> {
  await act(async () => {
    await new Promise((resolve) => window.setTimeout(resolve, ms));
  });
}
