import { describe, expect, it } from "vitest";
import { parseGridCoordinate } from "@/modules/te-lab-components/catalog/catalogUtils";
import {
  applySimpleIdentity,
  categoryForComponentType,
  engineeringUnitsFor,
  formatComponentValue,
  isOrderSelectable,
  normalizeShelfLocation,
  projectSimpleStock,
  readComponentValue,
  shelfLocationError,
} from "@/modules/te-lab-components/catalog/simpleComponent";
import type {
  Part,
  PartInput,
  PartStockSummary,
  QuantityTotal,
  StockPlacement,
  StorageContainer,
} from "@/modules/te-lab-components/types";

function placement(overrides: Partial<StockPlacement> = {}): StockPlacement {
  return {
    placementUuid: "placement-1",
    partUuid: "part-1",
    containerUuid: "simple-shelf",
    columnIndex: null,
    rowIndex: null,
    freeformPosition: "A1",
    quantity: 1,
    unitOfMeasure: "pcs",
    packaging: "",
    lotCode: "",
    dateCode: "",
    condition: "new",
    countState: "uncounted",
    lastCountedAt: null,
    lastCountedBy: "",
    notes: "",
    archived: false,
    createdAt: "2026-08-17T12:00:00.000Z",
    updatedAt: "2026-08-17T12:00:00.000Z",
    ...overrides,
  };
}

function summary(totals: QuantityTotal[]): PartStockSummary {
  return {
    partUuid: "part-1",
    totals,
    stockStatus:
      totals.length === 1 && totals[0]?.unitOfMeasure === "pcs"
        ? "in_stock"
        : "unit_review",
  };
}

function part(overrides: Partial<Part> = {}): Part {
  return {
    id: "part-1",
    entryUuid: "part-1",
    internalPartNumber: "LP-0001",
    category: "Passive",
    subcategory: "Capacitor",
    manufacturer: "Murata",
    manufacturerPartNumber: "GRM155R71C104KA88D",
    displayValue: "100 nF",
    mountingType: "SMD",
    packageType: "0402",
    description: "MLCC",
    attributes: {
      value: { value: "100", unit: "nF" },
    },
    supplier: "",
    supplierSku: "",
    supplierPackaging: "",
    productUrl: "",
    datasheetUrl: "",
    defaultUnitOfMeasure: "pcs",
    reorderPoint: null,
    targetQuantity: null,
    partStatus: "active",
    picturePath: "",
    notes: "",
    archived: false,
    legacy: {
      serialNumber: "",
      projectName: "",
      assignedTo: "",
      lifecycleStatus: "active",
      workingStatus: "unknown",
      condition: "",
      verifiedInSurvey: false,
      manualEntry: true,
    },
    createdAt: "2026-08-17T12:00:00.000Z",
    updatedAt: "2026-08-17T12:00:00.000Z",
    ...overrides,
  };
}

function partInput(overrides: Partial<PartInput> = {}): PartInput {
  return {
    internalPartNumber: "LP-0001",
    category: "Passive",
    subcategory: "Capacitor",
    manufacturer: "Murata",
    manufacturerPartNumber: "GRM155R71C104KA88D",
    displayValue: "100 nF",
    mountingType: "SMD",
    packageType: "0402",
    description: "MLCC",
    attributes: {
      value: { value: "100", unit: "nF" },
      tolerance: { value: "10", unit: "%" },
    },
    supplier: "",
    supplierSku: "",
    supplierPackaging: "",
    productUrl: "",
    datasheetUrl: "",
    defaultUnitOfMeasure: "pcs",
    reorderPoint: null,
    targetQuantity: null,
    partStatus: "active",
    notes: "",
    archived: false,
    ...overrides,
  };
}

function container(overrides: Partial<StorageContainer> = {}): StorageContainer {
  return {
    containerUuid: "grid-drawer",
    areaUuid: "area-1",
    name: "Drawer 1",
    containerType: "drawer",
    gridEnabled: true,
    rowCount: 10,
    columnCount: 10,
    rowStart: 1,
    origin: "top-left",
    description: "",
    archived: false,
    createdAt: "2026-08-17T12:00:00.000Z",
    updatedAt: "2026-08-17T12:00:00.000Z",
    ...overrides,
  };
}

describe("simple Lab component projection", () => {
  it("maps controlled component types without changing persisted field names", () => {
    expect(categoryForComponentType("Capacitor")).toBe("Passive");
    expect(categoryForComponentType("MCU")).toBe("Integrated Circuit");
    expect(categoryForComponentType("Legacy Custom Type")).toBeNull();
  });

  it("uses type-aware engineering units and combines display values", () => {
    expect(engineeringUnitsFor("Capacitor")).toEqual(["pF", "nF", "µF", "mF", "F"]);
    expect(formatComponentValue("Capacitor", "100", "nF")).toBe("100 nF");
    expect(formatComponentValue("BJT", "2N3904", "")).toBe("2N3904");
  });

  it("normalizes and validates shelf and grid locations", () => {
    expect(normalizeShelfLocation(" b3 ")).toBe("B3");
    expect(shelfLocationError("m15", 40)).toBeNull();
    expect(shelfLocationError("AA1", 40)).toBeNull();
    expect(shelfLocationError("A-1", 40)).toBe(
      "Use a shelf code such as A1, B3, or M15.",
    );
    expect(shelfLocationError("", 1)).toBe(
      "Location is required when quantity is greater than zero.",
    );
    expect(shelfLocationError("", 0)).toBeNull();
  });

  it("parses Excel-style grid coordinates", () => {
    expect(parseGridCoordinate("B3")).toEqual({ columnIndex: 1, rowIndex: 2 });
    expect(parseGridCoordinate("A1")).toEqual({ columnIndex: 0, rowIndex: 0 });
    expect(parseGridCoordinate("AA1")).toEqual({ columnIndex: 26, rowIndex: 0 });
    expect(parseGridCoordinate("12A")).toBeNull();
  });

  it("projects one pcs placement but flags advanced placement sets", () => {
    expect(
      projectSimpleStock(
        [placement({ quantity: 40, unitOfMeasure: "pcs", freeformPosition: "A1" })],
        new Map(),
      ),
    ).toMatchObject({ kind: "simple", location: "A1", quantity: 40 });
    expect(
      projectSimpleStock(
        [
          placement({ placementUuid: "p1", unitOfMeasure: "pcs" }),
          placement({ placementUuid: "p2", unitOfMeasure: "pcs" }),
        ],
        new Map(),
      ),
    ).toMatchObject({ kind: "review", reasons: ["multiple_placements"] });
  });

  it("allows order selection only for an unambiguous pcs total", () => {
    expect(isOrderSelectable(summary([{ unitOfMeasure: "pcs", quantity: 4 }]))).toBe(
      true,
    );
    expect(isOrderSelectable(summary([{ unitOfMeasure: "m", quantity: 4 }]))).toBe(
      false,
    );
    expect(
      isOrderSelectable(
        summary([
          { unitOfMeasure: "pcs", quantity: 4 },
          { unitOfMeasure: "reel", quantity: 1 },
        ]),
      ),
    ).toBe(false);
  });

  it("reads component value from attributes, then recognized display suffix, else full text", () => {
    expect(readComponentValue(part())).toEqual({ value: "100", unit: "nF" });
    expect(
      readComponentValue(
        part({
          attributes: {},
          displayValue: "4.7 kΩ",
          subcategory: "Resistor",
        }),
      ),
    ).toEqual({ value: "4.7", unit: "kΩ" });
    expect(
      readComponentValue(
        part({
          attributes: {},
          displayValue: "2N3904",
          subcategory: "BJT",
        }),
      ),
    ).toEqual({ value: "2N3904", unit: "" });
  });

  it("applies recognized identity fields and still writes displayValue for unrecognized types", () => {
    const base = partInput({
      category: "Legacy Category",
      subcategory: "Legacy Custom Type",
      displayValue: "old label",
      attributes: {
        tolerance: { value: "10", unit: "%" },
        value: { value: "old", unit: "x" },
      },
    });

    const applied = applySimpleIdentity(base, "Capacitor", "100", "nF");
    expect(applied.category).toBe("Passive");
    expect(applied.subcategory).toBe("Capacitor");
    expect(applied.displayValue).toBe("100 nF");
    expect(applied.attributes.value).toEqual({ value: "100", unit: "nF" });
    expect(applied.attributes.tolerance).toEqual({ value: "10", unit: "%" });
    expect(applied.manufacturer).toBe(base.manufacturer);

    const custom = applySimpleIdentity(base, "Legacy Custom Type", "Header 4-pin", "");
    expect(custom.category).toBe("Legacy Category");
    expect(custom.subcategory).toBe("Legacy Custom Type");
    expect(custom.displayValue).toBe("Header 4-pin");
    // Unrecognized types must not invent attributes.value.
    expect(custom.attributes.value).toEqual({ value: "old", unit: "x" });
    expect(custom.attributes.tolerance).toEqual({ value: "10", unit: "%" });
  });

  it("flags non-whole pcs quantities as review-required", () => {
    expect(
      projectSimpleStock(
        [placement({ quantity: 3.5, unitOfMeasure: "pcs", freeformPosition: "A1" })],
        new Map(),
      ),
    ).toMatchObject({
      kind: "review",
      reasons: ["non_whole_quantity"],
    });
  });

  it("clears stale attributes.value when switching to a non-unit component type", () => {
    const base = partInput({
      attributes: {
        value: { value: "100", unit: "nF" },
        tolerance: { value: "10", unit: "%" },
      },
    });

    const applied = applySimpleIdentity(base, "BJT", "2N3904", "");
    expect(applied.category).toBe("Semiconductor");
    expect(applied.subcategory).toBe("BJT");
    expect(applied.displayValue).toBe("2N3904");
    expect(applied.attributes.value).toBeUndefined();
    expect(applied.attributes).not.toHaveProperty("value");
    expect(applied.attributes.tolerance).toEqual({ value: "10", unit: "%" });
  });

  it("ignores archived placements when projecting simple stock", () => {
    expect(
      projectSimpleStock(
        [
          placement({
            placementUuid: "archived",
            archived: true,
            quantity: 99,
            freeformPosition: "Z9",
          }),
          placement({
            placementUuid: "active",
            quantity: 12,
            freeformPosition: "B2",
          }),
        ],
        new Map(),
      ),
    ).toMatchObject({
      kind: "simple",
      location: "B2",
      placementUuid: "active",
      quantity: 12,
    });
  });

  it("uses placementPosition for grid placements", () => {
    const grid = container();
    const containersById = new Map([[grid.containerUuid, grid]]);
    expect(
      projectSimpleStock(
        [
          placement({
            containerUuid: grid.containerUuid,
            columnIndex: 2,
            rowIndex: 6,
            freeformPosition: "",
            quantity: 5,
          }),
        ],
        containersById,
      ),
    ).toMatchObject({
      kind: "simple",
      location: "C7",
      quantity: 5,
    });
  });

  it("rejects multi-letter freeform codes but accepts real multi-letter grid cells", () => {
    expect(
      projectSimpleStock(
        [
          placement({
            freeformPosition: "AA1",
            columnIndex: null,
            rowIndex: null,
            unitOfMeasure: "pcs",
            quantity: 3,
          }),
        ],
        new Map(),
      ),
    ).toMatchObject({ kind: "review", reasons: ["invalid_location"] });

    expect(
      projectSimpleStock(
        [
          placement({
            freeformPosition: "BIN2",
            columnIndex: null,
            rowIndex: null,
            unitOfMeasure: "pcs",
            quantity: 3,
          }),
        ],
        new Map(),
      ),
    ).toMatchObject({ kind: "review", reasons: ["invalid_location"] });

    const grid = container({ columnCount: 30, rowCount: 10 });
    const containersById = new Map([[grid.containerUuid, grid]]);
    expect(
      projectSimpleStock(
        [
          placement({
            containerUuid: grid.containerUuid,
            columnIndex: 26,
            rowIndex: 0,
            freeformPosition: "",
            unitOfMeasure: "pcs",
            quantity: 7,
          }),
        ],
        containersById,
      ),
    ).toMatchObject({
      kind: "simple",
      location: "AA1",
      quantity: 7,
    });
  });

  it("collects review reasons for non-piece, mixed units, and invalid locations", () => {
    expect(
      projectSimpleStock(
        [placement({ unitOfMeasure: "reel", freeformPosition: "A1" })],
        new Map(),
      ),
    ).toMatchObject({ kind: "review", reasons: ["non_piece_unit"] });

    expect(
      projectSimpleStock(
        [
          placement({
            placementUuid: "p1",
            unitOfMeasure: "pcs",
            freeformPosition: "A1",
          }),
          placement({
            placementUuid: "p2",
            unitOfMeasure: "reel",
            freeformPosition: "B2",
          }),
        ],
        new Map(),
      ),
    ).toMatchObject({
      kind: "review",
      reasons: expect.arrayContaining([
        "multiple_placements",
        "mixed_units",
        "non_piece_unit",
      ]),
    });

    expect(
      projectSimpleStock(
        [
          placement({
            unitOfMeasure: "pcs",
            freeformPosition: "not-a-shelf",
            columnIndex: null,
            rowIndex: null,
          }),
        ],
        new Map(),
      ),
    ).toMatchObject({ kind: "review", reasons: ["invalid_location"] });
  });

  it("treats zero quantity with empty location as simple empty stock", () => {
    expect(projectSimpleStock([], new Map())).toMatchObject({
      kind: "simple",
      location: "",
      placementUuid: null,
      quantity: 0,
    });

    expect(
      projectSimpleStock(
        [
          placement({
            quantity: 0,
            freeformPosition: "",
            unitOfMeasure: "pcs",
          }),
        ],
        new Map(),
      ),
    ).toMatchObject({
      kind: "simple",
      location: "",
      placementUuid: null,
      quantity: 0,
    });
  });

  it("does not treat zero-qty blank non-pcs stock as simple empty", () => {
    expect(
      projectSimpleStock(
        [
          placement({
            quantity: 0,
            freeformPosition: "",
            unitOfMeasure: "reel",
          }),
        ],
        new Map(),
      ),
    ).toMatchObject({ kind: "review", reasons: ["non_piece_unit"] });
  });

  it("projects zero-qty pcs with a valid location and flags blank positive stock", () => {
    expect(
      projectSimpleStock(
        [
          placement({
            quantity: 0,
            freeformPosition: "A1",
            unitOfMeasure: "pcs",
          }),
        ],
        new Map(),
      ),
    ).toMatchObject({
      kind: "simple",
      location: "A1",
      quantity: 0,
    });

    expect(
      projectSimpleStock(
        [
          placement({
            quantity: 5,
            freeformPosition: "",
            unitOfMeasure: "pcs",
            columnIndex: null,
            rowIndex: null,
          }),
        ],
        new Map(),
      ),
    ).toMatchObject({ kind: "review", reasons: ["invalid_location"] });
  });

  it("treats missing summary and empty totals correctly for order selection", () => {
    expect(isOrderSelectable(undefined)).toBe(false);
    expect(isOrderSelectable(summary([]))).toBe(true);
  });
});
