import type { InventoryCounts, InventoryEntry } from "@/features/inventory/types";

import { deriveCalibrationHealth, getLocalDateString } from "./calibrationHealth";

export function getInventoryCounts(entries: InventoryEntry[], localDate = getLocalDateString()): InventoryCounts {
  let archive = 0;
  let dueSoon = 0;
  let missingDue = 0;
  let outToCal = 0;
  let overdue = 0;
  let verified = 0;

  for (const entry of entries) {
    if (entry.archived) {
      archive += 1;
    }
    if (entry.verifiedAt) {
      verified += 1;
    }
    const health = deriveCalibrationHealth(entry, localDate);
    if (health === "overdue") overdue += 1;
    if (health === "due_soon") dueSoon += 1;
    if (health === "missing_due") missingDue += 1;
    if (health === "out_to_cal") outToCal += 1;
  }

  return {
    inventory: entries.length - archive,
    archive,
    dueSoon,
    missingDue,
    outToCal,
    overdue,
    total: entries.length,
    verified,
  };
}
