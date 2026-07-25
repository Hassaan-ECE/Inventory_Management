import { act } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { TE_LAB_COMPONENTS_MODULE_ID } from "@/modules/te-lab-components/moduleId";
import type {
  PartInput as LabPartInput,
  StockPlacementInput as LabStockPlacementInput,
} from "@/modules/te-lab-components/types";
import { TE_TEST_EQUIPMENT_MODULE_ID } from "@/modules/te-test-equipment/moduleId";
import type { InventoryEntryInput } from "@/modules/te-test-equipment/types";

describe("tauri inventory bridge", () => {
  beforeEach(() => {
    delete window.inventoryDesktop;
  });

  it("does not register the desktop bridge in a normal browser session", async () => {
    const tauriGlobal = globalThis as { isTauri?: boolean };
    const originalIsTauri = tauriGlobal.isTauri;
    tauriGlobal.isTauri = false;
    delete window.inventoryDesktop;
    vi.resetModules();

    try {
      await import("@/integrations/tauri/tauriInventoryBridge");
      expect(window.inventoryDesktop).toBeUndefined();
    } finally {
      if (originalIsTauri === undefined) {
        Reflect.deleteProperty(tauriGlobal, "isTauri");
      } else {
        tauriGlobal.isTauri = originalIsTauri;
      }
    }
  }, 15_000);

  it("registers and cleans up Tauri shared inventory change events", async () => {
    type SharedInventoryChangedEvent = {
      event: string;
      id: number;
      payload: unknown;
    };

    const sharedChangeHandlerRef: {
      current: ((event: SharedInventoryChangedEvent) => void) | null;
    } = { current: null };
    const unlisten = vi.fn();
    const listen = vi.fn((_eventName: string, handler: (event: SharedInventoryChangedEvent) => void) => {
      sharedChangeHandlerRef.current = handler;
      return Promise.resolve(unlisten);
    });
    const invoke = vi.fn();

    vi.resetModules();
    vi.doMock("@tauri-apps/api/core", () => ({
      convertFileSrc: (path: string) => `asset://${path}`,
      invoke,
      isTauri: () => true,
    }));
    vi.doMock("@tauri-apps/api/event", () => ({ listen }));
    mockWindowStatePersistence();

    try {
      await import("@/integrations/tauri/tauriInventoryBridge");
      const callback = vi.fn();
      const desktopBridge = Reflect.get(window, "inventoryDesktop") as NonNullable<Window["inventoryDesktop"]> | undefined;
      const cleanup = desktopBridge?.onSharedInventoryChanged?.(callback);

      expect(cleanup).toEqual(expect.any(Function));
      expect(listen).toHaveBeenCalledWith("inventory:shared-changed", expect.any(Function));

      sharedChangeHandlerRef.current?.({ event: "inventory:shared-changed", id: 1, payload: null });
      sharedChangeHandlerRef.current?.({
        event: "inventory:shared-changed",
        id: 2,
        payload: { systemId: "me-storage" },
      });
      expect(callback).not.toHaveBeenCalled();

      sharedChangeHandlerRef.current?.({
        event: "inventory:shared-changed",
        id: 3,
        payload: { systemId: "te-test-equipment" },
      });
      expect(callback).toHaveBeenCalledTimes(1);
      expect(callback).toHaveBeenCalledWith({ systemId: "te-test-equipment" });

      sharedChangeHandlerRef.current?.({
        event: "inventory:shared-changed",
        id: 4,
        payload: { systemId: "te-lab-components" },
      });
      expect(callback).toHaveBeenCalledTimes(2);
      expect(callback).toHaveBeenLastCalledWith({ systemId: "te-lab-components" });

      await flushAsyncWork();
      cleanup?.();

      expect(unlisten).toHaveBeenCalledTimes(1);
    } finally {
      vi.doUnmock("@tauri-apps/api/core");
      vi.doUnmock("@tauri-apps/api/event");
      vi.doUnmock("@/integrations/tauri/windowState");
      vi.resetModules();
    }
  });

  it("invokes tokenized sync commands and treats null sync as a stale no-op", async () => {
    const invoke = vi.fn((command: string) => {
      if (command === "activate_inventory_sync") return Promise.resolve("session-1");
      if (command === "sync_inventory") return Promise.resolve(null);
      if (command === "deactivate_inventory_sync") return Promise.resolve(true);
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const desktopBridge = await registerDesktopBridge(invoke);

    await expect(desktopBridge.activateInventorySync(TE_TEST_EQUIPMENT_MODULE_ID)).resolves.toBe("session-1");
    await expect(desktopBridge.syncInventory(TE_TEST_EQUIPMENT_MODULE_ID, "session-1")).resolves.toBeNull();
    await expect(desktopBridge.deactivateInventorySync(TE_TEST_EQUIPMENT_MODULE_ID, "session-1")).resolves.toBe(true);

    expect(invoke).toHaveBeenNthCalledWith(1, "activate_inventory_sync", {
      moduleId: TE_TEST_EQUIPMENT_MODULE_ID,
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "sync_inventory", {
      moduleId: TE_TEST_EQUIPMENT_MODULE_ID,
      sessionId: "session-1",
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "deactivate_inventory_sync", {
      moduleId: TE_TEST_EQUIPMENT_MODULE_ID,
      sessionId: "session-1",
    });
  });

  it("parses Lab catalog payloads and scopes catalog, placement, migration, and export invocations", async () => {
    const part = validLabCatalogPart();
    const placement = validLabStockPlacement();
    const invoke = vi.fn((command: string) => {
      switch (command) {
        case "load_inventory":
          return Promise.resolve(validLabCatalogSyncPayload(part, placement));
        case "create_lab_part":
          return Promise.resolve({
            value: part,
            message: "Part added.",
            mutationMode: "shared",
            shared: validLabSharedStatus(),
          });
        case "create_lab_stock_placement":
          return Promise.resolve({
            value: placement,
            message: "Placement added.",
            mutationMode: "shared",
            shared: validLabSharedStatus(),
          });
        case "preview_lab_catalog_migration":
          return Promise.resolve(validLabMigrationPreview());
        case "export_excel":
          return Promise.resolve({
            canceled: false,
            outputPath: "D:/exports/TE_Lab_Components_Catalog_Export.xlsx",
          });
        default:
          return Promise.reject(new Error(`Unexpected command: ${command}`));
      }
    });
    const desktopBridge = await registerDesktopBridge(invoke);
    const partInput = validLabPartInput();
    const placementInput = validLabStockPlacementInput();

    const loaded = await desktopBridge.loadInventory(TE_LAB_COMPONENTS_MODULE_ID);
    await expect(
      desktopBridge.createLabPart?.(partInput),
    ).resolves.toMatchObject({
      value: { entryUuid: "lab-part-1", manufacturerPartNumber: "GHR-04V-S" },
      mutationMode: "shared",
    });
    await expect(
      desktopBridge.createLabStockPlacement?.(placementInput),
    ).resolves.toMatchObject({ value: { placementUuid: "lab-placement-1", quantity: 12 } });
    await expect(desktopBridge.previewLabCatalogMigration?.()).resolves.toMatchObject({
      sourceFingerprint: "sha256:legacy",
      proposedParts: 1,
      blocking: false,
    });
    await expect(
      desktopBridge.exportExcel?.(TE_LAB_COMPONENTS_MODULE_ID),
    ).resolves.toEqual({
      canceled: false,
      error: undefined,
      outputPath: "D:/exports/TE_Lab_Components_Catalog_Export.xlsx",
    });

    expect(loaded).toMatchObject({
      dbPath: "te-lab-components.feox",
      parts: [
        {
          entryUuid: "lab-part-1",
          manufacturerPartNumber: "GHR-04V-S",
          defaultUnitOfMeasure: "pcs",
        },
      ],
      stockPlacements: [{ placementUuid: "lab-placement-1", quantity: 12 }],
      counts: { activeParts: 1, totalParts: 1 },
      shared: { enabled: true, mutationMode: "shared" },
    });
    expect(loaded.parts[0]).not.toHaveProperty("calibrationRequirement");
    expect(loaded.parts[0]).not.toHaveProperty("outToCalibration");

    expect(invoke).toHaveBeenNthCalledWith(1, "load_inventory", {
      moduleId: TE_LAB_COMPONENTS_MODULE_ID,
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "create_lab_part", { input: partInput });
    expect(invoke).toHaveBeenNthCalledWith(3, "create_lab_stock_placement", { input: placementInput });
    expect(invoke).toHaveBeenNthCalledWith(4, "preview_lab_catalog_migration");
    expect(invoke).toHaveBeenNthCalledWith(5, "export_excel", {
      moduleId: TE_LAB_COMPONENTS_MODULE_ID,
    });
  });

  it("runs Tauri shared inventory cleanup after pending listener registration resolves", async () => {
    const deferredUnlisten = createDeferred<() => void>();
    const unlisten = vi.fn();
    const listen = vi.fn(() => deferredUnlisten.promise);

    vi.resetModules();
    vi.doMock("@tauri-apps/api/core", () => ({
      convertFileSrc: (path: string) => `asset://${path}`,
      invoke: vi.fn(),
      isTauri: () => true,
    }));
    vi.doMock("@tauri-apps/api/event", () => ({ listen }));
    mockWindowStatePersistence();

    try {
      await import("@/integrations/tauri/tauriInventoryBridge");
      const desktopBridge = Reflect.get(window, "inventoryDesktop") as NonNullable<Window["inventoryDesktop"]> | undefined;
      const cleanup = desktopBridge?.onSharedInventoryChanged?.(() => undefined);

      cleanup?.();
      expect(unlisten).not.toHaveBeenCalled();

      await act(async () => {
        deferredUnlisten.resolve(unlisten);
        await deferredUnlisten.promise;
      });

      expect(unlisten).toHaveBeenCalledTimes(1);
    } finally {
      vi.doUnmock("@tauri-apps/api/core");
      vi.doUnmock("@tauri-apps/api/event");
      vi.doUnmock("@/integrations/tauri/windowState");
      vi.resetModules();
    }
  });

  it("rejects malformed Tauri inventory payloads before they reach React state", async () => {
    const invoke = vi.fn().mockResolvedValue({
      dbPath: "inventory.feox",
      entries: [{ description: "missing id" }],
      shared: { message: "ready" },
    });

    vi.resetModules();
    vi.doMock("@tauri-apps/api/core", () => ({
      convertFileSrc: (path: string) => `asset://${path}`,
      invoke,
      isTauri: () => true,
    }));
    vi.doMock("@tauri-apps/api/event", () => ({
      listen: vi.fn(() => Promise.resolve(() => undefined)),
    }));
    mockWindowStatePersistence();

    try {
      await import("@/integrations/tauri/tauriInventoryBridge");
      const desktopBridge = Reflect.get(window, "inventoryDesktop") as NonNullable<Window["inventoryDesktop"]> | undefined;

      await expect(desktopBridge?.loadInventory(TE_TEST_EQUIPMENT_MODULE_ID)).rejects.toThrow("Invalid inventory entry");
    } finally {
      vi.doUnmock("@tauri-apps/api/core");
      vi.doUnmock("@tauri-apps/api/event");
      vi.doUnmock("@/integrations/tauri/windowState");
      vi.resetModules();
    }
  });

  it("rejects malformed Tauri query, mutation, delete, and export payloads", async () => {
    const invoke = vi.fn((command: string) => {
      switch (command) {
        case "query_inventory":
          return Promise.resolve({
            counts: {},
            dbPath: "inventory.feox",
            entries: "not-an-array",
            shared: { message: "ready" },
            totalFiltered: 0,
          });
        case "delete_entry":
          return Promise.resolve({ message: "missing entry id" });
        case "export_excel":
          return Promise.resolve({ outputPath: 42 });
        default:
          return Promise.resolve({
            entry: { description: "missing id" },
            message: "saved",
            shared: { message: "ready" },
          });
      }
    });

    vi.resetModules();
    vi.doMock("@tauri-apps/api/core", () => ({
      convertFileSrc: (path: string) => `asset://${path}`,
      invoke,
      isTauri: () => true,
    }));
    vi.doMock("@tauri-apps/api/event", () => ({
      listen: vi.fn(() => Promise.resolve(() => undefined)),
    }));
    mockWindowStatePersistence();

    try {
      await import("@/integrations/tauri/tauriInventoryBridge");
      const desktopBridge = Reflect.get(window, "inventoryDesktop") as NonNullable<Window["inventoryDesktop"]> | undefined;

      await expect(
        desktopBridge?.queryInventory?.(TE_TEST_EQUIPMENT_MODULE_ID, {
          filters: {
            assetNumber: "",
            description: "",
            location: "",
            manufacturer: "",
            model: "",
            calibrationRequirement: "all",
            calibrationHealth: "all",
            dueWindow: "all",
          },
          query: "",
          scope: "inventory",
          sort: { column: "manufacturer", direction: "asc" },
        }),
      ).rejects.toThrow("Invalid inventory entries");
      await expect(desktopBridge?.createEntry(TE_TEST_EQUIPMENT_MODULE_ID, validEntryInput())).rejects.toThrow(
        "Invalid inventory entry",
      );
      await expect(
        desktopBridge?.updateEntry(TE_TEST_EQUIPMENT_MODULE_ID, "1", validEntryInput()),
      ).rejects.toThrow("Invalid inventory entry");
      await expect(
        desktopBridge?.toggleVerifiedEntry(TE_TEST_EQUIPMENT_MODULE_ID, "1", true),
      ).rejects.toThrow("Invalid inventory entry");
      await expect(
        desktopBridge?.setArchivedEntry(TE_TEST_EQUIPMENT_MODULE_ID, "1", true),
      ).rejects.toThrow("Invalid inventory entry");
      await expect(desktopBridge?.deleteEntry(TE_TEST_EQUIPMENT_MODULE_ID, "1")).rejects.toThrow("missing entry id");
      await expect(desktopBridge?.exportExcel?.(TE_TEST_EQUIPMENT_MODULE_ID)).rejects.toThrow("Excel export payload");
    } finally {
      vi.doUnmock("@tauri-apps/api/core");
      vi.doUnmock("@tauri-apps/api/event");
      vi.doUnmock("@/integrations/tauri/windowState");
      vi.resetModules();
    }
  });

  it("normalizes malformed but recoverable Tauri inventory fields", async () => {
    const invoke = vi.fn().mockResolvedValue({
      dbPath: "inventory.feox",
      entries: [
        {
          id: "1",
          archived: false,
          description: "Caliper",
          lifecycleStatus: "bad",
          qty: "not-a-number",
          verifiedInSurvey: true,
          workingStatus: "also-bad",
        },
      ],
      shared: { message: "ready", mutationMode: "shared" },
    });

    vi.resetModules();
    vi.doMock("@tauri-apps/api/core", () => ({
      convertFileSrc: (path: string) => `asset://${path}`,
      invoke,
      isTauri: () => true,
    }));
    vi.doMock("@tauri-apps/api/event", () => ({
      listen: vi.fn(() => Promise.resolve(() => undefined)),
    }));
    mockWindowStatePersistence();

    try {
      await import("@/integrations/tauri/tauriInventoryBridge");
      const desktopBridge = Reflect.get(window, "inventoryDesktop") as NonNullable<Window["inventoryDesktop"]> | undefined;

      await expect(desktopBridge?.loadInventory(TE_TEST_EQUIPMENT_MODULE_ID)).resolves.toMatchObject({
        entries: [
          {
            id: "1",
            lifecycleStatus: "active",
            qty: null,
            workingStatus: "unknown",
          },
        ],
        shared: { canModify: true, mutationMode: "shared" },
      });
    } finally {
      vi.doUnmock("@tauri-apps/api/core");
      vi.doUnmock("@tauri-apps/api/event");
      vi.doUnmock("@/integrations/tauri/windowState");
      vi.resetModules();
    }
  });

  it("normalizes malformed shared status values at the bridge boundary", async () => {
    const invoke = vi.fn().mockResolvedValue({
      dbPath: "inventory.feox",
      entries: [validBridgeEntry()],
      shared: "not-a-shared-status",
    });

    vi.resetModules();
    vi.doMock("@tauri-apps/api/core", () => ({
      convertFileSrc: (path: string) => `asset://${path}`,
      invoke,
      isTauri: () => true,
    }));
    vi.doMock("@tauri-apps/api/event", () => ({
      listen: vi.fn(() => Promise.resolve(() => undefined)),
    }));
    mockWindowStatePersistence();

    try {
      await import("@/integrations/tauri/tauriInventoryBridge");
      const desktopBridge = Reflect.get(window, "inventoryDesktop") as NonNullable<Window["inventoryDesktop"]> | undefined;

      await expect(desktopBridge?.loadInventory(TE_TEST_EQUIPMENT_MODULE_ID)).resolves.toMatchObject({
        entries: [{ id: "1" }],
        shared: {
          available: false,
          canModify: true,
          enabled: false,
          message: "Shared sync status unavailable.",
          mutationMode: "local",
        },
      });
    } finally {
      vi.doUnmock("@tauri-apps/api/core");
      vi.doUnmock("@tauri-apps/api/event");
      vi.doUnmock("@/integrations/tauri/windowState");
      vi.resetModules();
    }
  });

  it("defaults missing legacy calibration fields without inventing verification", async () => {
    const desktopBridge = await registerDesktopBridge(vi.fn().mockResolvedValue({
      dbPath: "inventory.feox",
      entries: [validBridgeEntry()],
      shared: { message: "ready" },
    }));

    await expect(desktopBridge.loadInventory(TE_TEST_EQUIPMENT_MODULE_ID)).resolves.toMatchObject({
      entries: [{ calibrationRequirement: "unknown", outToCalibration: false }],
    });
    const result = await desktopBridge.loadInventory(TE_TEST_EQUIPMENT_MODULE_ID);
    expect(result.entries[0]?.verifiedAt).toBeUndefined();
  });

  it.each([
    ["calibration requirement", { calibrationRequirement: "sometimes" }],
    ["null calibration requirement", { calibrationRequirement: null }],
    ["out-to-calibration flag", { outToCalibration: "yes" }],
    ["null out-to-calibration flag", { outToCalibration: null }],
    ["last-calibrated date", { lastCalibratedAt: "2026-02-30" }],
    ["due date", { calibrationDueAt: "07/13/2026" }],
    ["interval", { calibrationIntervalMonths: 0 }],
    ["verification timestamp", { verifiedAt: "yesterday" }],
    ["provenance", { importProvenance: { batchId: "batch", sourceFilename: "file.csv", sourceRow: 0 } }],
  ])("rejects a malformed present %s", async (_label, malformedField) => {
    const desktopBridge = await registerDesktopBridge(vi.fn().mockResolvedValue({
      dbPath: "inventory.feox",
      entries: [validBridgeEntry(malformedField)],
      shared: { message: "ready" },
    }));

    await expect(desktopBridge.loadInventory(TE_TEST_EQUIPMENT_MODULE_ID)).rejects.toThrow("Invalid inventory entry");
  });

  it("parses complete calibration and import provenance fields", async () => {
    const desktopBridge = await registerDesktopBridge(vi.fn().mockResolvedValue({
      dbPath: "inventory.feox",
      entries: [validBridgeEntry({
        calibrationRequirement: "required",
        outToCalibration: true,
        lastCalibratedAt: "2026-01-13",
        calibrationDueAt: "2027-01-13",
        calibrationIntervalMonths: 12,
        certificateRef: "CERT-1",
        calibrationVendor: "Acme Calibration",
        calibrationNotes: "Return to intake",
        verifiedAt: "2026-07-13T12:00:00Z",
        verifiedBy: "Avery",
        importProvenance: {
          batchId: "sha256:batch",
          sourceFilename: "inventory.xlsx",
          sourceSheet: "Equipment",
          sourceRow: 12,
          originalId: "88",
          originalAssetNumber: "TE-88",
          originalSerialNumber: "SN-88",
        },
      })],
      shared: { message: "ready" },
    }));

    await expect(desktopBridge.loadInventory(TE_TEST_EQUIPMENT_MODULE_ID)).resolves.toMatchObject({
      entries: [{
        calibrationRequirement: "required",
        outToCalibration: true,
        calibrationDueAt: "2027-01-13",
        importProvenance: { sourceRow: 12, sourceSheet: "Equipment" },
        verifiedAt: "2026-07-13T12:00:00Z",
      }],
    });
  });

  it("invokes picker, preview, and commit commands and validates import accounting", async () => {
    const report = validImportReport();
    const invoke = vi.fn((command: string) => {
      if (command === "pick_import_file") return Promise.resolve("C:/imports/equipment.csv");
      if (command === "preview_import") return Promise.resolve(report);
      if (command === "commit_import") {
        return Promise.resolve({
          batchId: report.batchId,
          inserted: 1,
          matched: 0,
          conflicted: 0,
          rejected: 0,
          ignored: 0,
          remaining: 0,
          noop: 0,
          entriesChanged: true,
          message: "Import committed.",
        });
      }
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const desktopBridge = await registerDesktopBridge(invoke);

    await expect(desktopBridge.pickImportFile?.(TE_TEST_EQUIPMENT_MODULE_ID)).resolves.toBe(
      "C:/imports/equipment.csv",
    );
    await expect(
      desktopBridge.previewImport?.(TE_TEST_EQUIPMENT_MODULE_ID, "C:/imports/equipment.csv"),
    ).resolves.toEqual(report);
    await expect(
      desktopBridge.commitImport?.(TE_TEST_EQUIPMENT_MODULE_ID, { batchId: report.batchId, confirmed: true }),
    ).resolves.toMatchObject({
      entriesChanged: true,
      noop: 0,
    });
    expect(invoke).toHaveBeenNthCalledWith(1, "pick_import_file", {
      moduleId: TE_TEST_EQUIPMENT_MODULE_ID,
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "preview_import", {
      moduleId: TE_TEST_EQUIPMENT_MODULE_ID,
      path: "C:/imports/equipment.csv",
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "commit_import", {
      input: { batchId: report.batchId, confirmed: true },
      moduleId: TE_TEST_EQUIPMENT_MODULE_ID,
    });
  });

  it.each([
    ["mismatched total", { totalRows: 2 }],
    ["row classification", { rowOutcomes: [{ ...validImportReport().rowOutcomes[0], classification: "other" }] }],
    ["column accounting", { columns: [{ ...validImportReport().columns[0], nonblankCount: -1 }] }],
    ["blocking flag", { blocking: "no" }],
  ])("rejects malformed import preview %s", async (_label, override) => {
    const desktopBridge = await registerDesktopBridge(vi.fn().mockResolvedValue({ ...validImportReport(), ...override }));

    await expect(
      desktopBridge.previewImport?.(TE_TEST_EQUIPMENT_MODULE_ID, "C:/imports/equipment.csv"),
    ).rejects.toThrow("Invalid import dry-run report");
  });

  it("rejects malformed numeric noop and entriesChanged commit fields", async () => {
    const desktopBridge = await registerDesktopBridge(vi.fn().mockResolvedValue({
      batchId: "batch-1",
      inserted: 0,
      matched: 0,
      conflicted: 0,
      rejected: 0,
      ignored: 0,
      remaining: 0,
      noop: false,
      entriesChanged: 1,
      message: "bad",
    }));

    await expect(
      desktopBridge.commitImport?.(TE_TEST_EQUIPMENT_MODULE_ID, { batchId: "batch-1", confirmed: true }),
    ).rejects.toThrow("Invalid import commit result");
  });
});

async function registerDesktopBridge(invoke: ReturnType<typeof vi.fn>): Promise<NonNullable<Window["inventoryDesktop"]>> {
  vi.resetModules();
  vi.doMock("@tauri-apps/api/core", () => ({
    convertFileSrc: (path: string) => `asset://${path}`,
    invoke,
    isTauri: () => true,
  }));
  vi.doMock("@tauri-apps/api/event", () => ({
    listen: vi.fn(() => Promise.resolve(() => undefined)),
  }));
  mockWindowStatePersistence();
  await import("@/integrations/tauri/tauriInventoryBridge");
  const bridge = Reflect.get(window, "inventoryDesktop") as NonNullable<Window["inventoryDesktop"]> | undefined;
  if (!bridge) throw new Error("Desktop bridge did not register.");
  return bridge;
}

function validImportReport() {
  return {
    batchId: "sha256:batch-1",
    sourceFingerprint: "sha256:source-1",
    sourceFilename: "equipment.csv",
    selectedSheet: "equipment.csv",
    mappingVersion: "te-test-equipment-v1",
    totalRows: 1,
    inserted: 1,
    matched: 0,
    conflicted: 0,
    rejected: 0,
    ignored: 0,
    columns: [
      {
        originalHeader: "Asset Number",
        normalizedTarget: "assetNumber",
        treatment: "mapped",
        nonblankCount: 1,
        reason: "Mapped by header alias.",
      },
    ],
    rowOutcomes: [
      {
        sourceRow: 2,
        classification: "inserted",
        issues: [],
        originalId: "88",
        originalAssetNumber: "TE-88",
        originalSerialNumber: "SN-88",
        candidateEntryUuid: null,
        rawValues: { "Asset Number": "TE-88" },
      },
    ],
    blocking: false,
    reconciliationBasis: "inventory-revision-1",
  } as const;
}

function mockWindowStatePersistence() {
  const installWindowStatePersistence = vi.fn();
  const saveCurrentWindowState = vi.fn().mockResolvedValue(undefined);
  vi.doMock("@/integrations/tauri/windowState", () => ({
    installWindowStatePersistence,
    saveCurrentWindowState,
  }));

  return { installWindowStatePersistence, saveCurrentWindowState };
}

function validBridgeEntry(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    id: "1",
    archived: false,
    description: "Caliper",
    lifecycleStatus: "active",
    qty: 1,
    workingStatus: "working",
    ...overrides,
  };
}

function validEntryInput(): InventoryEntryInput {
  return {
    archived: false,
    assetNumber: "",
    assignedTo: "",
    condition: "",
    calibrationRequirement: "unknown",
    outToCalibration: false,
    description: "Caliper",
    lifecycleStatus: "active",
    links: "",
    location: "",
    manufacturer: "",
    model: "",
    notes: "",
    picturePath: "",
    projectName: "",
    qty: 1,
    serialNumber: "",
    workingStatus: "working",
  };
}

function validLabCatalogPart(): Record<string, unknown> {
  return {
    id: "lab-1",
    databaseId: 1,
    entryUuid: "lab-part-1",
    internalPartNumber: "LAB-JST-1",
    category: "Connector",
    subcategory: "Housing",
    manufacturer: "JST",
    manufacturerPartNumber: "GHR-04V-S",
    displayValue: "4 position",
    mountingType: "wire",
    packageType: "GH housing",
    description: "Bench connector",
    attributes: { positions: { value: "4", unit: "" } },
    supplier: "DigiKey",
    supplierSku: "455-1165-ND",
    supplierPackaging: "bag",
    productUrl: "https://example.test/product",
    datasheetUrl: "https://example.test/datasheet.pdf",
    defaultUnitOfMeasure: "pcs",
    reorderPoint: 5,
    targetQuantity: 20,
    partStatus: "active",
    picturePath: "",
    notes: "",
    archived: false,
    legacy: {
      serialNumber: "",
      projectName: "Harnesses",
      assignedTo: "",
      lifecycleStatus: "active",
      workingStatus: "working",
      condition: "Good",
      verifiedInSurvey: true,
      manualEntry: true,
    },
    createdAt: "2026-07-20T12:00:00.000Z",
    updatedAt: "2026-07-20T12:00:00.000Z",
  };
}

function validLabPartInput(): LabPartInput {
  return {
    internalPartNumber: "LAB-JST-1",
    category: "Connector",
    subcategory: "Housing",
    manufacturer: "JST",
    manufacturerPartNumber: "GHR-04V-S",
    displayValue: "4 position",
    mountingType: "wire",
    packageType: "GH housing",
    description: "Bench connector",
    attributes: { positions: { value: "4", unit: "" } },
    supplier: "DigiKey",
    supplierSku: "455-1165-ND",
    supplierPackaging: "bag",
    productUrl: "https://example.test/product",
    datasheetUrl: "https://example.test/datasheet.pdf",
    defaultUnitOfMeasure: "pcs",
    reorderPoint: 5,
    targetQuantity: 20,
    partStatus: "active",
    picturePath: "",
    notes: "",
    archived: false,
  };
}

function validLabStockPlacement(): Record<string, unknown> {
  return {
    placementUuid: "lab-placement-1",
    partUuid: "lab-part-1",
    containerUuid: "lab-container-1",
    columnIndex: 2,
    rowIndex: 6,
    freeformPosition: "",
    quantity: 12,
    unitOfMeasure: "pcs",
    packaging: "bag",
    lotCode: "",
    dateCode: "",
    condition: "new",
    countState: "counted",
    lastCountedAt: "2026-07-20T12:00:00.000Z",
    lastCountedBy: "Avery",
    notes: "",
    archived: false,
    createdAt: "2026-07-20T12:00:00.000Z",
    updatedAt: "2026-07-20T12:00:00.000Z",
  };
}

function validLabStockPlacementInput(): LabStockPlacementInput {
  return {
    partUuid: "lab-part-1",
    containerUuid: "lab-container-1",
    columnIndex: 2,
    rowIndex: 6,
    freeformPosition: "",
    quantity: 12,
    unitOfMeasure: "pcs",
    packaging: "bag",
    lotCode: "",
    dateCode: "",
    condition: "new",
    countState: "uncounted",
    lastCountedAt: null,
    lastCountedBy: "",
    notes: "",
    archived: false,
  };
}

function validLabCatalogSyncPayload(part: Record<string, unknown>, placement: Record<string, unknown>): Record<string, unknown> {
  return {
    dbPath: "te-lab-components.feox",
    parts: [part],
    storageAreas: [{ areaUuid: "lab-area-1", name: "Main Lab", areaType: "lab", owner: "TE", description: "", archived: false, createdAt: "2026-07-20T12:00:00.000Z", updatedAt: "2026-07-20T12:00:00.000Z" }],
    storageContainers: [{ containerUuid: "lab-container-1", areaUuid: "lab-area-1", name: "Cabinet 1", containerType: "cabinet", gridEnabled: true, rowCount: 8, columnCount: 8, rowStart: 1, origin: "top_left", description: "", archived: false, createdAt: "2026-07-20T12:00:00.000Z", updatedAt: "2026-07-20T12:00:00.000Z" }],
    stockPlacements: [placement],
    summaries: [{ partUuid: "lab-part-1", totals: [{ unitOfMeasure: "pcs", quantity: 12 }], stockStatus: "in_stock" }],
    counts: { activeParts: 1, archivedParts: 0, totalParts: 1, noStock: 0, lowStock: 0, unitReview: 0 },
    migration: { schemaVersion: 2, required: false, legacyEntryCount: 0, catalogInitialized: true, message: "ready" },
    shared: validLabSharedStatus(),
  };
}

function validLabMigrationPreview(): Record<string, unknown> {
  return {
    mappingVersion: "te-lab-components-catalog-v2",
    sourceFingerprint: "sha256:legacy",
    sourceSchemaVersion: 1,
    targetSchemaVersion: 2,
    legacyRows: 1,
    proposedParts: 1,
    proposedPlacements: 1,
    archivedParts: 0,
    blankPositiveQuantityLocations: 0,
    generatedPartUuids: 0,
    duplicateInternalPartNumbers: [],
    likelyMpnDuplicates: [],
    invalidRows: [],
    warnings: ["Units require review."],
    blocking: false,
  };
}

function validLabSharedStatus(): Record<string, unknown> {
  return {
    available: true,
    canModify: true,
    enabled: true,
    message: "",
    mutationMode: "shared",
  };
}

function createDeferred<T>(): {
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

async function flushAsyncWork(): Promise<void> {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}
