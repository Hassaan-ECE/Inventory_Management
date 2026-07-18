import { describe, expect, it } from "vitest";

import { deriveCalibrationHealth } from "@/features/inventory/lib/calibrationHealth";
import type {
  CalibrationRequirement,
  InventoryEntry,
} from "@/features/inventory/types";

const LOCAL_DATE = "2026-07-13";

describe("deriveCalibrationHealth", () => {
  it.each([
    ["reference_only", true, undefined, "not_applicable"],
    ["not_required", true, undefined, "not_applicable"],
    ["unknown", true, undefined, "unknown"],
    ["required", true, undefined, "out_to_cal"],
    ["required", false, undefined, "missing_due"],
    ["required", false, "invalid", "missing_due"],
  ] as const)(
    "applies precedence for requirement %s, out-to-cal %s, and due date %s",
    (calibrationRequirement, outToCalibration, calibrationDueAt, expected) => {
      const entry = buildEntry({
        calibrationDueAt,
        calibrationRequirement,
        outToCalibration,
      });

      expect(deriveCalibrationHealth(entry, LOCAL_DATE)).toBe(expected);
    },
  );

  it.each([
    ["2026-07-12", "overdue"],
    ["2026-07-13", "due_soon"],
    ["2026-08-12", "due_soon"],
    ["2026-08-13", "current"],
  ] as const)("classifies the due-date boundary %s as %s", (calibrationDueAt, expected) => {
    expect(deriveCalibrationHealth(buildEntry({ calibrationDueAt }), LOCAL_DATE)).toBe(expected);
  });

  it("excludes archived entries before evaluating calibration state", () => {
    expect(
      deriveCalibrationHealth(
        buildEntry({ archived: true, calibrationDueAt: "2026-07-12", outToCalibration: true }),
        LOCAL_DATE,
      ),
    ).toBeNull();
  });

  it("does not use an interval as a due date", () => {
    expect(
      deriveCalibrationHealth(
        buildEntry({ calibrationDueAt: undefined, calibrationIntervalMonths: 12 }),
        LOCAL_DATE,
      ),
    ).toBe("missing_due");
  });

  it("uses the caller-provided due-soon window inclusively", () => {
    const entry = buildEntry({ calibrationDueAt: "2026-07-28" });

    expect(deriveCalibrationHealth(entry, LOCAL_DATE, 15)).toBe("due_soon");
    expect(deriveCalibrationHealth(entry, LOCAL_DATE, 14)).toBe("current");
  });
});

function buildEntry(
  overrides: Partial<InventoryEntry> & {
    calibrationRequirement?: CalibrationRequirement;
  } = {},
): InventoryEntry {
  return {
    id: "te-health-1",
    assetNumber: "TE-HEALTH-1",
    serialNumber: "SN-HEALTH-1",
    qty: 1,
    manufacturer: "Fluke",
    model: "87V",
    description: "Industrial multimeter",
    projectName: "Electrical Bench",
    location: "Cabinet E1",
    assignedTo: "",
    links: "",
    notes: "",
    lifecycleStatus: "active",
    workingStatus: "working",
    condition: "good",
    calibrationRequirement: "required",
    outToCalibration: false,
    archived: false,
    manualEntry: true,
    picturePath: "",
    createdAt: "2026-07-01T00:00:00Z",
    updatedAt: "2026-07-01T00:00:00Z",
    entryUuid: "00000000-0000-4000-8000-000000000001",
    ...overrides,
  };
}
