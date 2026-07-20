import type { ColumnKey, InventoryEntry, SortState } from "@/modules/te-lab-components/types";

import { formatLinkLabel } from "./columns";

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

export function sortEntries(entries: InventoryEntry[], sortState: SortState | null): InventoryEntry[] {
  if (!sortState || entries.length <= 1) {
    return entries;
  }

  const multiplier = sortState.direction === "asc" ? 1 : -1;

  return entries
    .map((entry, index) => ({ entry, index }))
    .sort((leftItem, rightItem) => {
      const leftValue = getSortValue(leftItem.entry, sortState.column);
      const rightValue = getSortValue(rightItem.entry, sortState.column);
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
    })
    .map(({ entry }) => entry);
}

function getSortValue(entry: InventoryEntry, column: ColumnKey): number | string | undefined {
  switch (column) {
    case "verified":
      return entry.verifiedInSurvey ? 1 : 0;
    case "qty":
      return entry.qty ?? undefined;
    case "assetNumber":
      return entry.assetNumber.trim().toLowerCase();
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
