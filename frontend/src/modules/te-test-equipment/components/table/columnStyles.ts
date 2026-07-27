import type { CSSProperties } from "react";

import type { ColumnConfig } from "@/modules/te-test-equipment/types";

/** Fixed-size column: holds its width under `table-fixed` so residual space does not steal from it. */
function fixedRem(rem: number): CSSProperties {
  return { width: `${rem}rem`, minWidth: `${rem}rem`, maxWidth: `${rem}rem` };
}

/**
 * Column widths for the equipment / calibration grids.
 *
 * Non-description columns use locked rem widths so they stay fully sized.
 * Description is the only flexible column and receives leftover horizontal space
 * after those fixed columns are accounted for.
 */
export function getColumnStyle(columnKey: ColumnConfig["key"]): CSSProperties {
  switch (columnKey) {
    case "verified":
      return fixedRem(2.75);
    case "qty":
      return fixedRem(3.5);
    case "assetNumber":
      // Fits tags like VPEQ0001279 + padding.
      return fixedRem(10.5);
    case "serialNumber":
      return fixedRem(9.5);
    case "manufacturer":
      return fixedRem(11);
    case "model":
      return fixedRem(10);
    case "projectName":
      return fixedRem(9);
    case "location":
      return fixedRem(10);
    case "assignedTo":
      return fixedRem(9);
    case "calibrationHealth":
      return fixedRem(8);
    case "lastCalibratedAt":
    case "calibrationDueAt":
      // Compact MM/YY (e.g. 05/27).
      return fixedRem(5);
    case "calibrationIntervalMonths":
      return fixedRem(6.5);
    case "outToCalibration":
      return fixedRem(6);
    case "certificateRef":
      return fixedRem(8.5);
    case "calibrationVendor":
      return fixedRem(9);
    case "calibrationNotes":
      return fixedRem(12);
    case "links":
      return fixedRem(9);
    case "description":
      // Only flexible column: keep a usable floor, then absorb leftover width.
      return { width: "auto", minWidth: "12rem" };
    default:
      return fixedRem(8);
  }
}
