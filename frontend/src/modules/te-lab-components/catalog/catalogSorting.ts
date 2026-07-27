import type { CatalogColumnKey } from "@/modules/te-lab-components/catalog/catalogColumns";
import type { CatalogLookups } from "@/modules/te-lab-components/catalog/catalogUtils";
import type { Part, SortDirection } from "@/modules/te-lab-components/types";

export interface CatalogSortState {
  column: CatalogColumnKey;
  direction: SortDirection;
}

/** Cycle column sort: inactive → asc → desc → inactive. */
export function cycleCatalogSortState(
  current: CatalogSortState | null,
  column: CatalogColumnKey,
): CatalogSortState | null {
  if (!current || current.column !== column) {
    return { column, direction: "asc" };
  }
  if (current.direction === "asc") {
    return { column, direction: "desc" };
  }
  return null;
}

export function sortCatalogParts(
  parts: Part[],
  sortState: CatalogSortState | null,
  lookups: CatalogLookups,
): Part[] {
  if (!sortState || parts.length <= 1) {
    return parts;
  }

  const multiplier = sortState.direction === "asc" ? 1 : -1;

  return parts
    .map((part, index) => ({ part, index }))
    .sort((leftItem, rightItem) => {
      const leftValue = getCatalogSortValue(leftItem.part, sortState.column, lookups);
      const rightValue = getCatalogSortValue(rightItem.part, sortState.column, lookups);
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
    .map(({ part }) => part);
}

function getCatalogSortValue(
  part: Part,
  column: CatalogColumnKey,
  lookups: CatalogLookups,
): number | string | undefined {
  switch (column) {
    case "stockStatus":
      return lookups.summariesByPartId.get(part.entryUuid)?.stockStatus ?? "no_stock";
    case "category":
      return part.category.trim().toLocaleLowerCase();
    case "manufacturerPartNumber":
      return part.manufacturerPartNumber.trim().toLocaleLowerCase();
    case "displayValue":
      return part.displayValue.trim().toLocaleLowerCase();
    case "packageType":
      return part.packageType.trim().toLocaleLowerCase();
    case "mountingType":
      return part.mountingType.trim().toLocaleLowerCase();
    case "totals": {
      const totals = lookups.summariesByPartId.get(part.entryUuid)?.totals ?? [];
      return totals.reduce((sum, total) => sum + total.quantity, 0);
    }
    case "locations":
      return undefined;
    case "manufacturer":
      return part.manufacturer.trim().toLocaleLowerCase();
    case "links":
      return undefined;
    case "internalPartNumber":
      return part.internalPartNumber.trim().toLocaleLowerCase();
    case "subcategory":
      return part.subcategory.trim().toLocaleLowerCase();
    case "supplier":
      return part.supplier.trim().toLocaleLowerCase();
    case "supplierSku":
      return part.supplierSku.trim().toLocaleLowerCase();
    case "partStatus":
      return part.partStatus.trim().toLocaleLowerCase();
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
