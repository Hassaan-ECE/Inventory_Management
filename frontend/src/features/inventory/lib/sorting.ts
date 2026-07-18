import type { ColumnKey, InventoryEntry, SortState } from "@/features/inventory/types";

import { formatLinkLabel } from "./columns";
import { deriveCalibrationHealth, getLocalDateString } from "./calibrationHealth";

const REQUIREMENT_ORDER = { required: 0, reference_only: 1, not_required: 2, unknown: 3 } as const;
const HEALTH_ORDER = {
  current: 0,
  due_soon: 1,
  overdue: 2,
  missing_due: 3,
  out_to_cal: 4,
  unknown: 5,
  not_applicable: 6,
} as const;

/** Cycle column sort: inactive → asc → desc → inactive. */
export function cycleSortState(current: SortState | null, column: ColumnKey): SortState | null {
  if (!current || current.column !== column) {
    return { column, direction: "asc" };
  }
  if (current.direction === "asc") {
    return { column, direction: "desc" };
  }
  return null;
}

export function sortEntries(
  entries: InventoryEntry[],
  sortState: SortState | null,
  localDate = getLocalDateString(),
): InventoryEntry[] {
  if (!sortState || entries.length <= 1) {
    return entries;
  }

  const multiplier = sortState.direction === "asc" ? 1 : -1;

  return entries.map((entry, index) => ({ entry, index })).sort((leftItem, rightItem) => {
    const leftValue = getSortValue(leftItem.entry, sortState.column, localDate);
    const rightValue = getSortValue(rightItem.entry, sortState.column, localDate);
    const leftBlank = isBlankValue(leftValue);
    const rightBlank = isBlankValue(rightValue);

    if (leftBlank && rightBlank) {
      return leftItem.index - rightItem.index;
    }
    if (leftBlank) {
      return 1;
    }
    if (rightBlank) {
      return -1;
    }
    if (leftValue === undefined || rightValue === undefined) {
      return leftItem.index - rightItem.index;
    }
    if (leftValue < rightValue) {
      return -1 * multiplier;
    }
    if (leftValue > rightValue) {
      return 1 * multiplier;
    }
    return leftItem.index - rightItem.index;
  }).map(({ entry }) => entry);
}

function getSortValue(entry: InventoryEntry, column: ColumnKey, localDate: string): number | string | undefined {
  switch (column) {
    case "verified":
      return entry.verifiedAt;
    case "qty":
      return entry.qty ?? undefined;
    case "assetNumber":
      return entry.assetNumber.trim().toLowerCase();
    case "serialNumber":
      return entry.serialNumber.trim().toLowerCase();
    case "manufacturer":
      return entry.manufacturer.trim().toLowerCase();
    case "model":
      return entry.model.trim().toLowerCase();
    case "description":
      return entry.description.trim().toLowerCase();
    case "projectName":
      return entry.projectName.trim().toLowerCase();
    case "location":
      return entry.location.trim().toLowerCase();
    case "calibrationRequirement":
      return REQUIREMENT_ORDER[entry.calibrationRequirement];
    case "outToCalibration":
      return entry.outToCalibration ? 1 : 0;
    case "calibrationDueAt":
      return entry.calibrationDueAt;
    case "calibrationHealth": {
      const health = deriveCalibrationHealth(entry, localDate);
      return health === null ? undefined : HEALTH_ORDER[health];
    }
    case "links":
      return formatLinkLabel(entry.links).toLowerCase();
  }
}

function isBlankValue(value: number | string | undefined): boolean {
  if (value === undefined) {
    return true;
  }
  if (typeof value === "number") {
    return !Number.isFinite(value);
  }
  return value.trim().length === 0;
}
