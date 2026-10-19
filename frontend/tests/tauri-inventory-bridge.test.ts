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

    try {
      await import("@/integrations/tauri/tauriInventoryBridge");
      const callback = vi.fn();
      const desktopBridge = Reflect.get(window, "inventoryDesktop") as NonNullable<Window["inventoryDesktop"]> | undefined;
      const cleanup = desktopBridge?.onSharedInventoryChanged?.(callback);

      expect(cleanup).toEqual(expect.any(Function));
      expect(listen).toHaveBeenCalledWith("inventory:shared-changed", expect.any(Function));

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

  it("parses Lab catalog payloads and scopes catalog, placement, and export invocations", async () => {
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
    expect(invoke).toHaveBeenNthCalledWith(4, "export_excel", {
      moduleId: TE_LAB_COMPONENTS_MODULE_ID,
    });
  });

  it("exports selected Lab order lines with the exact payload", async () => {
    const invoke = vi.fn().mockResolvedValue({
      canceled: false,
      outputPath: "C:/tmp/TE_Lab_Components_Order_Request_2026-08-17.xlsx",
    });
    const bridge = await registerDesktopBridge(invoke);
    const lines = [{ partUuid: "part-1", requestedQuantity: 25, note: "" }];

    await bridge.exportLabOrderRequest?.(lines);

    expect(invoke).toHaveBeenCalledWith("export_lab_order_request", { lines });
  });

  it("invokes and parses simple Lab component create and update commands", async () => {
    const invoke = vi
      .fn()
      .mockResolvedValueOnce(validSimpleComponentMutation())
      .mockResolvedValueOnce(validSimpleComponentMutation());
    const bridge = await registerDesktopBridge(invoke);
    const input = validSimpleComponentInput();

    await expect(bridge.createLabSimpleComponent?.(input)).resolves.toMatchObject({
      value: {
        part: { entryUuid: "lab-part-1", manufacturerPartNumber: "GHR-04V-S" },
        placement: { placementUuid: "lab-placement-1", quantity: 12 },
      },
      message: "Simple component saved.",
      mutationMode: "shared",
    });
    await expect(bridge.updateLabSimpleComponent?.("part-1", input)).resolves.toMatchObject({
      value: {
        part: { entryUuid: "lab-part-1" },
        placement: { placementUuid: "lab-placement-1" },
      },
      mutationMode: "shared",
    });

    expect(invoke).toHaveBeenNthCalledWith(1, "create_lab_simple_component", { input });
    expect(invoke).toHaveBeenNthCalledWith(2, "update_lab_simple_component", {
      input,
      partId: "part-1",
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
      vi.resetModules();
    }
  });

  it("invokes and parses calibration roster pick, preview, and commit commands", async () => {
    const invoke = vi.fn((command: string) => {
      if (command === "pick_calibration_roster_file") return Promise.resolve("C:/imports/calibration.xlsx");
      if (command === "preview_calibration_roster") return Promise.resolve(validCalibrationRosterReport());
      if (command === "commit_calibration_roster") {
        return Promise.resolve({
          batchId: "calibration-batch-1",
          updated: 1,
          created: 1,
          reset: 2,
          ignored: 1,
          noop: 0,
          finalRequired: 2,
          entriesChanged: true,
          message: "Calibration roster applied.",
        });
      }
      return Promise.reject(new Error(`Unexpected command: ${command}`));
    });
    const desktopBridge = await registerDesktopBridge(invoke);
    const commitInput = {
      batchId: "calibration-batch-1",
      confirmed: true,
      replaceActiveRequiredRoster: true,
      verificationAttribution: "Calibration roster cutover",
      resolutions: [],
    };

    await expect(desktopBridge.pickCalibrationRosterFile?.(TE_TEST_EQUIPMENT_MODULE_ID)).resolves.toBe(
      "C:/imports/calibration.xlsx",
    );
    await expect(
      desktopBridge.previewCalibrationRoster?.(TE_TEST_EQUIPMENT_MODULE_ID, "C:/imports/calibration.xlsx"),
    ).resolves.toMatchObject({ counts: { matchedUpdates: 1 }, totalSourceRows: 1 });
    await expect(
      desktopBridge.commitCalibrationRoster?.(TE_TEST_EQUIPMENT_MODULE_ID, commitInput),
    ).resolves.toMatchObject({ created: 1, entriesChanged: true, finalRequired: 2 });

    expect(invoke).toHaveBeenNthCalledWith(1, "pick_calibration_roster_file", {
      moduleId: TE_TEST_EQUIPMENT_MODULE_ID,
    });
    expect(invoke).toHaveBeenNthCalledWith(2, "preview_calibration_roster", {
      moduleId: TE_TEST_EQUIPMENT_MODULE_ID,
      path: "C:/imports/calibration.xlsx",
    });
    expect(invoke).toHaveBeenNthCalledWith(3, "commit_calibration_roster", {
      input: commitInput,
      moduleId: TE_TEST_EQUIPMENT_MODULE_ID,
    });
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
  await import("@/integrations/tauri/tauriInventoryBridge");
  const bridge = Reflect.get(window, "inventoryDesktop") as NonNullable<Window["inventoryDesktop"]> | undefined;
  if (!bridge) throw new Error("Desktop bridge did not register.");
  return bridge;
}

function validCalibrationRosterReport() {
  return {
    batchId: "calibration-batch-1",
    sourceFingerprint: "sha256-calibration-source",
    sourceFilename: "calibration.xlsx",
    mappingVersion: "te-calibration-roster-v1",
    contributingSheets: ["May"],
    totalSourceRows: 1,
    counts: {
      matchedUpdates: 1,
      createCandidates: 0,
      conflicts: 0,
      duplicateSourceRows: 0,
      ignoredJunk: 0,
      currentRequiredAbsent: 0,
    },
    rowOutcomes: [
      {
        sourceSheet: "May",
        sourceRow: 4,
        classification: "matched_update",
        issues: [],
        ignoredIdentityPlaceholders: ["Serial placeholder NSN ignored."],
        assetNumber: "TE-1",
        serialNumber: "NSN",
        manufacturer: "Maker",
        model: "Model",
        description: "Meter",
        candidateEntryUuid: "entry-1",
        candidateEntries: [
          {
            entryUuid: "entry-1",
            id: "1",
            assetNumber: "TE-1",
            serialNumber: "SER-1",
            manufacturer: "Maker",
            model: "Model",
            description: "Meter",
          },
        ],
        proposedRequirement: "required",
        proposedOutToCalibration: false,
        proposedInput: {
          ...validEntryInput(),
          assetNumber: "TE-1",
          serialNumber: "",
          calibrationRequirement: "required",
          lastCalibratedAt: "2026-05-31",
          calibrationDueAt: "2027-05-31",
        },
        changes: [
          {
            field: "calibrationDueAt",
            before: "2026-05-31",
            after: "2027-05-31",
            destructive: false,
          },
        ],
        semanticFlags: ["normal_dated"],
        requiresReview: false,
      },
    ],
    currentRequiredAbsent: [],
    prospectiveRequiredCount: 1,
    reconciliationBasis: "basis-1",
    blocking: false,
  } as const;
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

function validSimpleComponentInput(): {
  part: LabPartInput;
  quantity: number;
  location: string;
} {
  return {
    part: validLabPartInput(),
    quantity: 12,
    location: "Shelf A",
  };
}

function validSimpleComponentMutation(): Record<string, unknown> {
  return {
    value: {
      part: validLabCatalogPart(),
      placement: validLabStockPlacement(),
    },
    message: "Simple component saved.",
    mutationMode: "shared",
    shared: validLabSharedStatus(),
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
    shared: validLabSharedStatus(),
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
