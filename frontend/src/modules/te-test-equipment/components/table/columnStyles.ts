import type { CSSProperties } from "react";

import type { ColumnConfig } from "@/modules/te-test-equipment/types";

export function getColumnStyle(columnKey: ColumnConfig["key"]): CSSProperties {
  switch (columnKey) {
    case "verified":
      return { width: "4.75rem" };
    case "qty":
      return { width: "3.75rem" };
    case "assetNumber":
      // Fits tags like VPEQ0001279 (≈11 chars) + cell padding; longer values still truncate with title tooltip.
      return { width: "10rem", minWidth: "10rem" };
    case "serialNumber":
      return { width: "8rem" };
    case "projectName":
      return { width: "8.5rem" };
    case "calibrationHealth":
      return { width: "9.5rem" };
    case "lastCalibratedAt":
    case "calibrationDueAt":
      return { width: "8.5rem" };
    case "calibrationIntervalMonths":
      return { width: "7rem" };
    case "outToCalibration":
      return { width: "7.5rem" };
    case "certificateRef":
      return { width: "9rem" };
    case "calibrationVendor":
      return { width: "10rem" };
    case "assignedTo":
      return { width: "9rem" };
    case "calibrationNotes":
      return { width: "14rem" };
    default:
      return {};
  }
}
