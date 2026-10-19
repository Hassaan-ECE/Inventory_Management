import {
  partPlacements,
  placementPosition,
  searchablePartText,
  type createCatalogLookups,
} from "@/modules/te-lab-components/catalog/catalogUtils";
import { CATEGORY_TEMPLATES } from "@/modules/te-lab-components/catalog/categoryTemplates";
import type { CatalogScope, CatalogSyncResult, Part, StockStatus } from "@/modules/te-lab-components/types";

type CatalogLookups = ReturnType<typeof createCatalogLookups>;

export interface CatalogFilters {
  areaUuid: string;
  category: string;
  containerUuid: string;
  coordinate: string;
  displayValue: string;
  manufacturer: string;
  manufacturerPartNumber: string;
  mountingType: string;
  packageType: string;
  partStatus: string;
  stockStatus: "" | StockStatus;
  subcategory: string;
}

export const EMPTY_CATALOG_FILTERS: CatalogFilters = {
  areaUuid: "",
  category: "",
  containerUuid: "",
  coordinate: "",
  displayValue: "",
  manufacturer: "",
  manufacturerPartNumber: "",
  mountingType: "",
  packageType: "",
  partStatus: "",
  stockStatus: "",
  subcategory: "",
};

/** How many of the filter panel's filters are set (the search box is not counted). */
export function activeCatalogFilterCount(filters: CatalogFilters): number {
  return (Object.keys(EMPTY_CATALOG_FILTERS) as Array<keyof CatalogFilters>).filter((key) => filters[key]).length;
}

/** The parts in `scope` that match the filters and the search text, in table order. */
export function filterCatalogParts(
  catalog: CatalogSyncResult,
  lookups: CatalogLookups,
  scope: CatalogScope,
  filters: CatalogFilters,
  query: string,
): Part[] {
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const normalizedCoordinate = filters.coordinate.trim().toLocaleLowerCase();
  const contains = (value: string, filter: string) =>
    !filter || value.toLocaleLowerCase().includes(filter.trim().toLocaleLowerCase());
  return catalog.parts
    .filter((part) => part.archived === (scope === "archive"))
    .filter((part) => !filters.category || part.category === filters.category)
    .filter((part) => !filters.subcategory || part.subcategory === filters.subcategory)
    .filter((part) => !filters.manufacturer || part.manufacturer === filters.manufacturer)
    .filter((part) => contains(part.manufacturerPartNumber, filters.manufacturerPartNumber))
    .filter((part) => contains(part.displayValue, filters.displayValue))
    .filter((part) => !filters.mountingType || part.mountingType === filters.mountingType)
    .filter((part) => !filters.packageType || part.packageType === filters.packageType)
    .filter((part) => !filters.partStatus || part.partStatus === filters.partStatus)
    .filter((part) => !filters.stockStatus || lookups.summariesByPartId.get(part.entryUuid)?.stockStatus === filters.stockStatus)
    .filter((part) => {
      if (!filters.areaUuid && !filters.containerUuid && !normalizedCoordinate) return true;
      return partPlacements(catalog, part.entryUuid).some((placement) => {
        const container = lookups.containersById.get(placement.containerUuid);
        if (filters.areaUuid && container?.areaUuid !== filters.areaUuid) return false;
        if (filters.containerUuid && placement.containerUuid !== filters.containerUuid) return false;
        return (
          !normalizedCoordinate ||
          placementPosition(placement, container).toLocaleLowerCase().includes(normalizedCoordinate)
        );
      });
    })
    .filter(
      (part) =>
        !normalizedQuery ||
        searchablePartText(part, partPlacements(catalog, part.entryUuid), lookups).includes(normalizedQuery),
    );
}

/** The choices each filter dropdown offers, from the catalog and the category templates. */
export function catalogFilterOptions(parts: Part[]) {
  return {
    categories: uniqueOptions([...CATEGORY_TEMPLATES.map((template) => template.category), ...parts.map((part) => part.category)]),
    manufacturers: uniqueOptions(parts.map((part) => part.manufacturer)),
    mountingTypes: uniqueOptions(parts.map((part) => part.mountingType)),
    packageTypes: uniqueOptions(parts.map((part) => part.packageType)),
    partStatuses: uniqueOptions(parts.map((part) => part.partStatus)),
    subcategories: uniqueOptions(parts.map((part) => part.subcategory)),
  };
}

function uniqueOptions(values: string[]): string[] {
  return [...new Set(values.map((value) => value.trim()).filter(Boolean))].sort((left, right) => left.localeCompare(right));
}
