import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { APP_VERSION } from "@/app/branding";
import type {
  InventorySharedChangedPayload,
  InventorySyncResult,
} from "@/integrations/tauri/desktop-bridge";
import type {
  CatalogMigrationPreview,
  CatalogSyncResult,
  InventorySharedStatus as LabInventorySharedStatus,
  Part,
  StockPlacement,
} from "@/modules/te-lab-components/types";
import { InventoryShell } from "@/shell/InventoryShell";

import {
  CONNECTED_SHARED_STATUS,
  TEST_DB_PATH,
  buildDesktopSyncResult,
  buildTestEntry,
  createDeferred,
  createDesktopBridge,
  flushAsyncWork,
} from "./inventory-shell/helpers";

const LAB_DB_PATH = "C:/Users/Test/AppData/Local/com.inventory.management/te-lab-components.feox";
const LAB_SHARED_STATUS: LabInventorySharedStatus = {
  available: true,
  canModify: true,
  enabled: true,
  message: "",
  mutationMode: "shared",
};

describe("TE Lab Components catalog shell integration", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
    document.documentElement.classList.remove("dark");
    delete window.inventoryDesktop;
    vi.spyOn(document, "hasFocus").mockReturnValue(true);
  });

  it("switches Lab to TE and back with independent sessions and cached catalog parts", async () => {
    localStorage.setItem("inventory.activeSystem", "te-lab-components");
    const user = userEvent.setup();
    const labPart = buildLabPart({
      description: "Cached Lab catalog part",
      manufacturerPartNumber: "CACHED-LAB-701",
    });
    const teEntry = buildTestEntry({ description: "Cached TE row" });
    const firstLabCatalog = buildLabCatalog([labPart]);
    const secondLabLoad = createDeferred<InventorySyncResult<"te-lab-components">>();
    const sharedCallbacks: Array<(payload: InventorySharedChangedPayload) => void> = [];
    const unsubscribeCallbacks = [vi.fn(), vi.fn(), vi.fn()];
    let labActivations = 0;
    let teActivations = 0;
    let labLoads = 0;

    const activateInventorySync = vi.fn(async (moduleId: "te-test-equipment" | "te-lab-components") => {
      if (moduleId === "te-lab-components") {
        labActivations += 1;
        return `lab-session-${labActivations}`;
      }
      teActivations += 1;
      return `te-session-${teActivations}`;
    });
    const deactivateInventorySync = vi.fn().mockResolvedValue(true);
    const loadInventory = vi.fn((moduleId: "te-test-equipment" | "te-lab-components") => {
      if (moduleId === "te-test-equipment") {
        return Promise.resolve(buildDesktopSyncResult(CONNECTED_SHARED_STATUS, [teEntry]));
      }
      labLoads += 1;
      return labLoads === 1 ? Promise.resolve(firstLabCatalog) : secondLabLoad.promise;
    });
    const syncInventory = vi.fn(async (moduleId: "te-test-equipment" | "te-lab-components") =>
      moduleId === "te-test-equipment"
        ? { dbPath: TEST_DB_PATH, entries: [], entriesChanged: false, shared: CONNECTED_SHARED_STATUS }
        : { ...firstLabCatalog, entriesChanged: false },
    );
    const onSharedInventoryChanged = vi.fn((callback: (payload: InventorySharedChangedPayload) => void) => {
      const index = sharedCallbacks.length;
      sharedCallbacks.push(callback);
      return unsubscribeCallbacks[index] ?? (() => undefined);
    });

    window.inventoryDesktop = createDesktopBridge({
      activateInventorySync,
      deactivateInventorySync,
      loadInventory,
      onSharedInventoryChanged,
      syncInventory,
    });

    render(<InventoryShell />);

    expect(await screen.findByText("CACHED-LAB-701")).toBeInTheDocument();
    await waitFor(() => expect(syncInventory).toHaveBeenCalledWith("te-lab-components", "lab-session-1"));
    expect(activateInventorySync).toHaveBeenNthCalledWith(1, "te-lab-components");
    expect(document.title).toBe(`Inventory Management — TE Lab Components v${APP_VERSION}`);

    await switchInventory(user, "TE Test Equipment");

    expect(await screen.findByText("Cached TE row")).toBeInTheDocument();
    await waitFor(() => expect(deactivateInventorySync).toHaveBeenCalledWith("te-lab-components", "lab-session-1"));
    await waitFor(() => expect(syncInventory).toHaveBeenCalledWith("te-test-equipment", "te-session-1"));
    expect(unsubscribeCallbacks[0]).toHaveBeenCalledTimes(1);

    syncInventory.mockClear();
    act(() => sharedCallbacks[1]?.({ systemId: "te-lab-components" }));
    await flushAsyncWork();
    expect(syncInventory).not.toHaveBeenCalled();

    act(() => sharedCallbacks[1]?.({ systemId: "te-test-equipment" }));
    await waitFor(() => expect(syncInventory).toHaveBeenCalledWith("te-test-equipment", "te-session-1"));

    await switchInventory(user, "TE Lab Components");

    expect(screen.getByText("CACHED-LAB-701")).toBeInTheDocument();
    await waitFor(() => expect(deactivateInventorySync).toHaveBeenCalledWith("te-test-equipment", "te-session-1"));
    expect(loadInventory.mock.calls.filter(([moduleId]) => moduleId === "te-lab-components")).toHaveLength(2);
    expect(unsubscribeCallbacks[1]).toHaveBeenCalledTimes(1);

    await act(async () => {
      secondLabLoad.resolve(firstLabCatalog);
      await secondLabLoad.promise;
      await Promise.resolve();
    });

    await waitFor(() => expect(syncInventory).toHaveBeenCalledWith("te-lab-components", "lab-session-2"));
    expect(document.title).toBe(`Inventory Management — TE Lab Components v${APP_VERSION}`);
  });

  it("shows generalized part columns and editor fields without equipment calibration controls", async () => {
    localStorage.setItem("inventory.activeSystem", "te-lab-components");
    const user = userEvent.setup();

    render(<InventoryShell />);

    expect(screen.getByRole("columnheader", { name: "Stock Status" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Category" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Subcategory" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Manufacturer Part #" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Total Quantity" })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: "Locations" })).toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: /Verified/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Import" })).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Add Part" }));

    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByRole("heading", { name: "Add Catalog Part" })).toBeInTheDocument();
    expect(within(dialog).getByLabelText("Manufacturer part number")).toBeInTheDocument();
    expect(within(dialog).getByLabelText("Value / label")).toBeInTheDocument();
    expect(within(dialog).getByLabelText("Package type")).toBeInTheDocument();
    expect(within(dialog).getByLabelText("Default unit of measure")).toBeInTheDocument();
    expect(within(dialog).queryByText(/Calibration requirement/i)).not.toBeInTheDocument();

    const category = within(dialog).getByLabelText("Category");
    await user.clear(category);
    await user.type(category, "Optoelectronics");
    expect(category).toHaveValue("Optoelectronics");
    await user.click(within(dialog).getByRole("button", { name: "Add custom attribute" }));
    expect(within(dialog).getByLabelText("Attribute 1 key")).toBeInTheDocument();
  });

  it("selects shared bins with Excel labels, occupied warnings, and keyboard navigation", async () => {
    localStorage.setItem("inventory.activeSystem", "te-lab-components");
    const user = userEvent.setup();

    render(<InventoryShell />);

    await user.dblClick(screen.getByText("CF14JT1K00"));
    const partDialog = screen.getByRole("dialog");
    await user.click(within(partDialog).getByRole("button", { name: "Add Placement" }));

    const dialogs = screen.getAllByRole("dialog");
    const placementDialog = dialogs[dialogs.length - 1];
    expect(within(placementDialog).getByRole("button", { name: /Main Lab \/ Component Cabinet 1 \/ AA1; empty/i })).toBeInTheDocument();

    const occupiedCell = within(placementDialog).getByRole("button", { name: /Main Lab \/ Component Cabinet 1 \/ C7;/i });
    await user.click(occupiedCell);
    expect(within(placementDialog).getByText("Occupied bin — sharing is allowed")).toBeInTheDocument();
    await user.keyboard("{ArrowRight}");
    expect(within(placementDialog).getByRole("button", { name: /Main Lab \/ Component Cabinet 1 \/ D7; empty/i })).toHaveFocus();
  });

  it("filters by detailed catalog fields, coordinates, and persists sorting", async () => {
    localStorage.setItem("inventory.activeSystem", "te-lab-components");
    const user = userEvent.setup();

    render(<InventoryShell />);

    await user.click(screen.getByRole("button", { name: "Show filters" }));
    await user.click(screen.getByLabelText("Filter subcategory"));
    await user.click(screen.getByRole("option", { name: "BJT" }));

    expect(screen.getByText("2N3904BU")).toBeInTheDocument();
    expect(screen.queryByText("CF14JT1K00")).not.toBeInTheDocument();

    await user.click(screen.getByLabelText("Filter subcategory"));
    await user.click(screen.getByRole("option", { name: "All subcategories" }));
    await user.type(screen.getByLabelText("Filter bin coordinate"), "C7");

    expect(screen.getByText("CF14JT1K00")).toBeInTheDocument();
    expect(screen.getByText("C315C104M5U5TA")).toBeInTheDocument();
    expect(screen.queryByText("2N3904BU")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Sort by Manufacturer" }));
    await waitFor(() => {
      expect(JSON.parse(localStorage.getItem("teLabComponents.catalog.v2.filters") ?? "{}")).toMatchObject({
        coordinate: "C7",
      });
      expect(JSON.parse(localStorage.getItem("teLabComponents.catalog.v2.sort") ?? "null")).toMatchObject({
        column: "manufacturer",
        direction: "asc",
      });
    });
  });

  it("adds stock to an indistinguishable placement instead of creating a duplicate", async () => {
    localStorage.setItem("inventory.activeSystem", "te-lab-components");
    const user = userEvent.setup();
    const part = buildLabPart({
      entryUuid: "lab-part-merge",
      manufacturerPartNumber: "MERGE-1",
    });
    const existingPlacement: StockPlacement = {
      placementUuid: "lab-placement-merge",
      partUuid: part.entryUuid,
      containerUuid: "lab-container-main",
      columnIndex: 2,
      rowIndex: 6,
      freeformPosition: "",
      quantity: 10,
      unitOfMeasure: "pcs",
      packaging: "bag",
      lotCode: "",
      dateCode: "",
      condition: "new",
      countState: "counted",
      lastCountedAt: "2026-07-20T10:00:00.000Z",
      lastCountedBy: "Alex",
      notes: "Existing placement",
      archived: false,
      createdAt: "2026-07-20T10:00:00.000Z",
      updatedAt: "2026-07-20T10:00:00.000Z",
    };
    const catalog = buildLabCatalog([part], undefined, {
      stockPlacements: [existingPlacement],
      summaries: [{
        partUuid: part.entryUuid,
        totals: [{ unitOfMeasure: "pcs", quantity: 10 }],
        stockStatus: "in_stock",
      }],
      counts: {
        activeParts: 1,
        archivedParts: 0,
        totalParts: 1,
        noStock: 0,
        lowStock: 0,
        unitReview: 0,
      },
    });
    const updateLabStockPlacement = vi.fn().mockResolvedValue({
      value: { ...existingPlacement, quantity: 15 },
      message: "Stock placement updated.",
      mutationMode: "shared",
      shared: LAB_SHARED_STATUS,
    });
    const createLabStockPlacement = vi.fn();
    window.inventoryDesktop = createDesktopBridge({
      createLabStockPlacement,
      loadInventory: vi.fn().mockResolvedValue(catalog),
      syncInventory: vi.fn().mockResolvedValue({ ...catalog, entriesChanged: false }),
      updateLabStockPlacement,
    });

    render(<InventoryShell />);

    await user.dblClick(await screen.findByText("MERGE-1"));
    await user.click(within(screen.getByRole("dialog")).getByRole("button", { name: "Add Placement" }));
    const dialogs = screen.getAllByRole("dialog");
    const placementDialog = dialogs[dialogs.length - 1];
    await user.click(within(placementDialog).getByRole("button", { name: /Main Lab \/ Cabinet A \/ C7;/i }));
    await user.selectOptions(within(placementDialog).getByLabelText("Condition"), "new");
    const quantity = within(placementDialog).getByLabelText("Quantity");
    await user.clear(quantity);
    await user.type(quantity, "5");

    expect(within(placementDialog).getByText("Matching placement already exists")).toBeInTheDocument();
    await user.click(within(placementDialog).getByRole("button", { name: "Add to Existing Placement" }));

    await waitFor(() => expect(updateLabStockPlacement).toHaveBeenCalledWith(
      existingPlacement.placementUuid,
      expect.objectContaining({
        quantity: 15,
        countState: "counted",
        lastCountedBy: "Alex",
      }),
    ));
    expect(createLabStockPlacement).not.toHaveBeenCalled();
  });

  it("requires a reviewed dry-run fingerprint before committing legacy migration", async () => {
    localStorage.setItem("inventory.activeSystem", "te-lab-components");
    const user = userEvent.setup();
    const migrationCatalog = buildLabCatalog([], undefined, {
      migration: {
        schemaVersion: 1,
        required: true,
        legacyEntryCount: 3,
        catalogInitialized: false,
        message: "Legacy Lab Components data requires a reviewed catalog migration.",
      },
      shared: { ...LAB_SHARED_STATUS, enabled: false, available: false, mutationMode: "local" },
    });
    const readyCatalog = buildLabCatalog([]);
    const preview = buildMigrationPreview();
    const loadInventory = vi.fn().mockResolvedValueOnce(migrationCatalog).mockResolvedValue(readyCatalog);
    const previewLabCatalogMigration = vi.fn().mockResolvedValue(preview);
    const commitLabCatalogMigration = vi.fn().mockResolvedValue({
      sourceFingerprint: preview.sourceFingerprint,
      partsCreated: 3,
      placementsCreated: 2,
      areasCreated: 1,
      containersCreated: 1,
      noop: false,
      message: "Migration committed.",
    });
    window.inventoryDesktop = createDesktopBridge({
      loadInventory,
      previewLabCatalogMigration,
      commitLabCatalogMigration,
    });

    render(<InventoryShell />);

    expect(await screen.findByRole("heading", { name: "Legacy Lab data requires reviewed migration" })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Run Dry-Run" }));
    expect(await screen.findByText("Every migrated quantity uses unit unknown.")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Commit Migration" })).toBeDisabled();
    await user.click(screen.getByLabelText(/I reviewed this exact dry-run fingerprint/i));
    await user.click(screen.getByRole("button", { name: "Commit Migration" }));

    await waitFor(() => expect(commitLabCatalogMigration).toHaveBeenCalledWith({
      sourceFingerprint: preview.sourceFingerprint,
      confirmed: true,
    }));
  });
});

function buildLabPart(overrides: Partial<Part> = {}): Part {
  return {
    id: "lab-701",
    databaseId: 701,
    entryUuid: "lab-part-701",
    internalPartNumber: "LAB-701",
    category: "Connector",
    subcategory: "Header",
    manufacturer: "Lab Maker",
    manufacturerPartNumber: "LC-701",
    displayValue: "4 position",
    mountingType: "through_hole",
    packageType: "2.54 mm header",
    description: "Lab component",
    attributes: { positions: { value: "4", unit: "" } },
    supplier: "DigiKey",
    supplierSku: "LAB-701-ND",
    supplierPackaging: "bag",
    productUrl: "",
    datasheetUrl: "",
    defaultUnitOfMeasure: "pcs",
    reorderPoint: 5,
    targetQuantity: 25,
    partStatus: "active",
    picturePath: "",
    notes: "",
    archived: false,
    legacy: {
      serialNumber: "",
      projectName: "Bench Controls",
      assignedTo: "",
      lifecycleStatus: "active",
      workingStatus: "working",
      condition: "",
      verifiedInSurvey: true,
      manualEntry: true,
    },
    createdAt: "2026-07-20T10:00:00.000Z",
    updatedAt: "2026-07-20T10:00:00.000Z",
    ...overrides,
  };
}

function buildLabCatalog(
  parts: Part[],
  entriesChanged?: boolean,
  overrides: Partial<CatalogSyncResult> = {},
): CatalogSyncResult {
  return {
    dbPath: LAB_DB_PATH,
    parts,
    storageAreas: [{
      areaUuid: "lab-area-main",
      name: "Main Lab",
      areaType: "lab",
      owner: "TE",
      description: "",
      archived: false,
      createdAt: "2026-07-20T10:00:00.000Z",
      updatedAt: "2026-07-20T10:00:00.000Z",
    }],
    storageContainers: [{
      containerUuid: "lab-container-main",
      areaUuid: "lab-area-main",
      name: "Cabinet A",
      containerType: "cabinet",
      gridEnabled: true,
      rowCount: 8,
      columnCount: 8,
      rowStart: 1,
      origin: "top_left",
      description: "",
      archived: false,
      createdAt: "2026-07-20T10:00:00.000Z",
      updatedAt: "2026-07-20T10:00:00.000Z",
    }],
    stockPlacements: [],
    summaries: parts.map((part) => ({ partUuid: part.entryUuid, totals: [], stockStatus: "no_stock" })),
    counts: {
      activeParts: parts.filter((part) => !part.archived).length,
      archivedParts: parts.filter((part) => part.archived).length,
      totalParts: parts.length,
      noStock: parts.filter((part) => !part.archived).length,
      lowStock: 0,
      unitReview: 0,
    },
    migration: {
      schemaVersion: 2,
      required: false,
      legacyEntryCount: 0,
      catalogInitialized: true,
      message: "Lab catalog ready.",
    },
    entriesChanged,
    shared: LAB_SHARED_STATUS,
    ...overrides,
  };
}

function buildMigrationPreview(): CatalogMigrationPreview {
  return {
    mappingVersion: "te-lab-components-catalog-v2",
    sourceFingerprint: "sha256:migration-review",
    sourceSchemaVersion: 1,
    targetSchemaVersion: 2,
    legacyRows: 3,
    proposedParts: 3,
    proposedPlacements: 2,
    archivedParts: 0,
    blankPositiveQuantityLocations: 1,
    generatedPartUuids: 0,
    duplicateInternalPartNumbers: [],
    likelyMpnDuplicates: [],
    invalidRows: [],
    warnings: ["Every migrated quantity uses unit unknown."],
    blocking: false,
  };
}

async function switchInventory(
  user: ReturnType<typeof userEvent.setup>,
  label: "TE Lab Components" | "TE Test Equipment",
): Promise<void> {
  await user.click(screen.getByRole("button", { name: "Switch inventory system" }));
  await user.click(screen.getByRole("option", { name: label }));
}
