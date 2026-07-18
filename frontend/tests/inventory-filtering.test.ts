import { describe, expect, it } from "vitest";

import { MOCK_INVENTORY } from "@/features/inventory/data/mockInventory";
import {
  DEFAULT_FILTERS,
  buildResultsLabel,
  filterEntries,
  formatLinkLabel,
  getInventoryCounts,
  cycleSortState,
  sortEntries,
} from "@/features/inventory/lib";
import type { InventoryEntry } from "@/features/inventory/types";

const LOCAL_DATE = "2026-07-13";

describe("inventory helpers", () => {
  it("filters inventory entries by query and scope", () => {
    const results = filterEntries(MOCK_INVENTORY, "inventory", "reorder threshold", DEFAULT_FILTERS);

    expect(results).toHaveLength(1);
    expect(results[0]?.id).toBe("te-1006");
  });

  it("combines field filters with global search", () => {
    const results = filterEntries(MOCK_INVENTORY, "inventory", "fixture", {
      ...DEFAULT_FILTERS,
      location: "tool crib",
    });

    expect(results).toHaveLength(1);
    expect(results[0]?.id).toBe("te-1003");
  });

  it("searches assigned user and condition fields", () => {
    const entries = MOCK_INVENTORY.map((entry) => {
      if (entry.id === "te-1002") {
        return { ...entry, assignedTo: "Avery Morgan" };
      }
      if (entry.id === "te-1004") {
        return { ...entry, condition: "Calibration due" };
      }
      return entry;
    });

    expect(filterEntries(entries, "inventory", "avery", DEFAULT_FILTERS).map((entry) => entry.id)).toEqual([
      "te-1002",
    ]);
    expect(filterEntries(entries, "inventory", "calibration due", DEFAULT_FILTERS).map((entry) => entry.id)).toEqual([
      "te-1004",
    ]);
  });

  it("builds result labels that match the source behavior", () => {
    const activeLocationFilter = { ...DEFAULT_FILTERS, location: "archive" };

    expect(buildResultsLabel(10, "inventory", "", DEFAULT_FILTERS)).toBe("Showing all 10 entries");
    expect(buildResultsLabel(0, "archive", "", DEFAULT_FILTERS)).toBe("No archived entries yet");
    expect(buildResultsLabel(3, "inventory", "", activeLocationFilter)).toBe("Showing 3 filtered entries");
    expect(buildResultsLabel(2, "archive", "", activeLocationFilter)).toBe("Showing 2 filtered archived entries");
    expect(buildResultsLabel(1, "inventory", "fluke", DEFAULT_FILTERS)).toBe('1 result for "fluke"');
    expect(buildResultsLabel(0, "archive", "bridgeport", DEFAULT_FILTERS)).toBe('No archived results for "bridgeport"');
    expect(buildResultsLabel(2, "archive", "active", activeLocationFilter)).toBe(
      '2 archived results for "active" after column filters',
    );
  });

  it("keeps blank values at the bottom when sorting", () => {
    const results = filterEntries(MOCK_INVENTORY, "inventory", "", DEFAULT_FILTERS);
    const ascendingQuantity = sortEntries(results, { column: "qty", direction: "asc" });
    const descendingQuantity = sortEntries(results, { column: "qty", direction: "desc" });
    const ascendingAssetNumber = sortEntries(results, { column: "assetNumber", direction: "asc" });
    const descendingAssetNumber = sortEntries(results, { column: "assetNumber", direction: "desc" });

    expect(ascendingQuantity.at(-1)?.id).toBe("te-1007");
    expect(descendingQuantity.at(-1)?.id).toBe("te-1007");
    expect(ascendingAssetNumber.at(-1)?.id).toBe("te-1004");
    expect(descendingAssetNumber.at(-1)?.id).toBe("te-1004");
  });

  it("cycles column sort through ascending, descending, then off", () => {
    expect(cycleSortState(null, "model")).toEqual({ column: "model", direction: "asc" });
    expect(cycleSortState({ column: "model", direction: "asc" }, "model")).toEqual({
      column: "model",
      direction: "desc",
    });
    expect(cycleSortState({ column: "model", direction: "desc" }, "model")).toBeNull();
    expect(cycleSortState({ column: "model", direction: "desc" }, "location")).toEqual({
      column: "location",
      direction: "asc",
    });
  });

  it("leaves filtered order unchanged when sort is cleared", () => {
    const results = filterEntries(MOCK_INVENTORY, "inventory", "", DEFAULT_FILTERS);
    expect(sortEntries(results, null).map((entry) => entry.id)).toEqual(results.map((entry) => entry.id));
  });

  it("formats long links into compact table labels", () => {
    expect(
      formatLinkLabel(
        "https://www.cejn.com/en-us/products/thermal-control/?filters=null%3D1191&mtm_campaign=Semicon-Campaign",
      ),
    ).toBe("www.cejn.com/en-us/products/thermal-control");
  });

  it("counts verification and active calibration health from the seeded dataset", () => {
    expect(getInventoryCounts(MOCK_INVENTORY, LOCAL_DATE)).toEqual({
      archive: 4,
      dueSoon: 2,
      inventory: 10,
      missingDue: 1,
      outToCal: 1,
      overdue: 1,
      total: 14,
      verified: 8,
    });
  });

  it("filters by requirement, derived health, and deterministic due windows after scope", () => {
    const entries = [
      calibrationEntry("overdue", "2026-07-12"),
      calibrationEntry("next-30", "2026-08-12"),
      calibrationEntry("next-60", "2026-09-11"),
      calibrationEntry("next-90", "2026-10-11"),
      calibrationEntry("missing", undefined),
      calibrationEntry("archived-overdue", "2026-07-01", { archived: true }),
      calibrationEntry("reference", undefined, { calibrationRequirement: "reference_only" }),
    ];

    expect(
      filterEntries(
        entries,
        "inventory",
        "",
        { ...DEFAULT_FILTERS, calibrationRequirement: "reference_only" },
        LOCAL_DATE,
      ).map((entry) => entry.id),
    ).toEqual(["reference"]);
    expect(
      filterEntries(
        entries,
        "inventory",
        "",
        { ...DEFAULT_FILTERS, calibrationHealth: "overdue" },
        LOCAL_DATE,
      ).map((entry) => entry.id),
    ).toEqual(["overdue"]);
    expect(
      filterEntries(
        entries,
        "archive",
        "",
        { ...DEFAULT_FILTERS, calibrationHealth: "overdue" },
        LOCAL_DATE,
      ),
    ).toEqual([]);
    expect(
      filterEntries(
        entries,
        "inventory",
        "",
        { ...DEFAULT_FILTERS, dueWindow: "next60" },
        LOCAL_DATE,
      ).map((entry) => entry.id),
    ).toEqual(["next-30", "next-60"]);
    expect(
      filterEntries(
        entries,
        "inventory",
        "",
        { ...DEFAULT_FILTERS, dueWindow: "missing" },
        LOCAL_DATE,
      ).map((entry) => entry.id),
    ).toEqual(["missing"]);
  });

  it("searches useful calibration text", () => {
    const entries = [
      calibrationEntry("certificate", "2026-08-13", {
        calibrationNotes: "Return through metrology intake",
        calibrationVendor: "Acme Calibration",
        certificateRef: "CERT-2026-88",
      }),
    ];

    expect(filterEntries(entries, "inventory", "cert-2026", DEFAULT_FILTERS, LOCAL_DATE)).toHaveLength(1);
    expect(filterEntries(entries, "inventory", "acme calibration", DEFAULT_FILTERS, LOCAL_DATE)).toHaveLength(1);
    expect(filterEntries(entries, "inventory", "metrology intake", DEFAULT_FILTERS, LOCAL_DATE)).toHaveLength(1);
  });

  it("sorts calibration and verification fields with blanks last in both directions and stable ties", () => {
    const entries = [
      calibrationEntry("blank-a", undefined),
      calibrationEntry("later", "2026-09-01", { verifiedAt: "2026-07-12T10:00:00Z" }),
      calibrationEntry("earlier-a", "2026-08-01", { verifiedAt: "2026-07-11T10:00:00Z" }),
      calibrationEntry("earlier-b", "2026-08-01", { verifiedAt: "2026-07-11T10:00:00Z" }),
      calibrationEntry("blank-b", undefined),
    ];

    expect(sortEntries(entries, { column: "calibrationDueAt", direction: "asc" }, LOCAL_DATE).map(({ id }) => id)).toEqual([
      "earlier-a",
      "earlier-b",
      "later",
      "blank-a",
      "blank-b",
    ]);
    expect(sortEntries(entries, { column: "calibrationDueAt", direction: "desc" }, LOCAL_DATE).map(({ id }) => id)).toEqual([
      "later",
      "earlier-a",
      "earlier-b",
      "blank-a",
      "blank-b",
    ]);
    expect(sortEntries(entries, { column: "verified", direction: "desc" }, LOCAL_DATE).slice(-2).map(({ id }) => id)).toEqual([
      "blank-a",
      "blank-b",
    ]);
    expect(sortEntries(entries, { column: "calibrationHealth", direction: "asc" }, LOCAL_DATE)[0]?.id).toBe("later");
  });
});

function calibrationEntry(
  id: string,
  calibrationDueAt?: string,
  overrides: Partial<InventoryEntry> = {},
): InventoryEntry {
  return {
    ...MOCK_INVENTORY[0]!,
    id,
    assetNumber: id,
    archived: false,
    calibrationDueAt,
    calibrationRequirement: "required",
    outToCalibration: false,
    verifiedAt: undefined,
    ...overrides,
  };
}
