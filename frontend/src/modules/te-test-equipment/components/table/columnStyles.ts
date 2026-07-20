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
    default:
      return {};
  }
}
