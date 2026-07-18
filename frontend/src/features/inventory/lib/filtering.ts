import type { FilterState, InventoryEntry, InventoryScope } from "@/features/inventory/types";

import { deriveCalibrationHealth, getLocalDateString, isValidDateOnly } from "./calibrationHealth";

export const DEFAULT_FILTERS: FilterState = {
  assetNumber: "",
  manufacturer: "",
  model: "",
  description: "",
  location: "",
  calibrationRequirement: "all",
  calibrationHealth: "all",
  dueWindow: "all",
};

export const INVENTORY_GLOBAL_SEARCH_FIELDS = [
  "assetNumber",
  "serialNumber",
  "manufacturer",
  "model",
  "description",
  "projectName",
  "location",
  "assignedTo",
  "links",
  "notes",
  "lifecycleStatus",
  "workingStatus",
  "condition",
  "calibrationRequirement",
  "lastCalibratedAt",
  "calibrationDueAt",
  "certificateRef",
  "calibrationVendor",
  "calibrationNotes",
  "verifiedBy",
] as const satisfies readonly (keyof InventoryEntry)[];

export type InventoryGlobalSearchField = (typeof INVENTORY_GLOBAL_SEARCH_FIELDS)[number];

export function hasActiveFilters(filters: FilterState): boolean {
  return (
    filters.assetNumber.trim().length > 0 ||
    filters.manufacturer.trim().length > 0 ||
    filters.model.trim().length > 0 ||
    filters.description.trim().length > 0 ||
    filters.location.trim().length > 0 ||
    filters.calibrationRequirement !== "all" ||
    filters.calibrationHealth !== "all" ||
    filters.dueWindow !== "all"
  );
}

export function filterEntries(
  entries: InventoryEntry[],
  scope: InventoryScope,
  query: string,
  filters: FilterState,
  localDate = getLocalDateString(),
): InventoryEntry[] {
  const normalizedQuery = query.trim().toLowerCase();
  const assetNumberFilter = filters.assetNumber.trim().toLowerCase();
  const manufacturerFilter = filters.manufacturer.trim().toLowerCase();
  const modelFilter = filters.model.trim().toLowerCase();
  const descriptionFilter = filters.description.trim().toLowerCase();
  const locationFilter = filters.location.trim().toLowerCase();

  return entries.filter((entry) => {
    if (scope === "inventory" && entry.archived) {
      return false;
    }
    if (scope === "archive" && !entry.archived) {
      return false;
    }

    if (
      filters.calibrationRequirement !== "all" &&
      entry.calibrationRequirement !== filters.calibrationRequirement
    ) {
      return false;
    }
    const health = deriveCalibrationHealth(entry, localDate);
    if (filters.calibrationHealth !== "all" && health !== filters.calibrationHealth) {
      return false;
    }
    if (!matchesDueWindow(entry, filters.dueWindow, localDate, health)) {
      return false;
    }

    const fieldFiltersMatch =
      includesNormalizedFilter(entry.assetNumber, assetNumberFilter) &&
      includesNormalizedFilter(entry.manufacturer, manufacturerFilter) &&
      includesNormalizedFilter(entry.model, modelFilter) &&
      includesNormalizedFilter(entry.description, descriptionFilter) &&
      includesNormalizedFilter(entry.location, locationFilter);

    if (!fieldFiltersMatch) {
      return false;
    }

    if (!normalizedQuery) {
      return true;
    }

    return entryMatchesQuery(entry, normalizedQuery);
  });
}

export function getEntrySearchValues(entry: InventoryEntry): Array<string | undefined> {
  return [
    ...INVENTORY_GLOBAL_SEARCH_FIELDS.map((field) => entry[field] as string | undefined),
    entry.outToCalibration ? "out to calibration" : undefined,
    entry.calibrationIntervalMonths?.toString(),
  ];
}

function matchesDueWindow(
  entry: InventoryEntry,
  dueWindow: FilterState["dueWindow"],
  localDate: string,
  health: ReturnType<typeof deriveCalibrationHealth>,
): boolean {
  if (dueWindow === "all") {
    return true;
  }
  if (dueWindow === "overdue") {
    return health === "overdue";
  }
  if (dueWindow === "missing") {
    return health === "missing_due";
  }
  if (!entry.calibrationDueAt || !isValidDateOnly(entry.calibrationDueAt) || !isValidDateOnly(localDate)) {
    return false;
  }

  const days = dueWindow === "next30" ? 30 : dueWindow === "next60" ? 60 : 90;
  const due = Date.parse(`${entry.calibrationDueAt}T00:00:00Z`);
  const today = Date.parse(`${localDate}T00:00:00Z`);
  return due >= today && due <= today + days * 86_400_000;
}

function includesNormalizedFilter(value: string, normalizedFilter: string): boolean {
  if (!normalizedFilter) {
    return true;
  }
  return value.toLowerCase().includes(normalizedFilter);
}

function entryMatchesQuery(entry: InventoryEntry, normalizedQuery: string): boolean {
  return getEntrySearchValues(entry).some((value) => includesNormalizedQuery(value, normalizedQuery));
}

function includesNormalizedQuery(value: string | undefined, normalizedQuery: string): boolean {
  return value?.toLowerCase().includes(normalizedQuery) ?? false;
}
