import type {
  FilterState,
  InventoryScope,
  TeTestEquipmentWorkspace,
} from "@/modules/te-test-equipment/types";

import { hasActiveFilters } from "./filtering";

export function buildResultsLabel(
  count: number,
  scope: InventoryScope,
  query: string,
  filters: FilterState,
  workspace: TeTestEquipmentWorkspace = "equipment",
): string {
  const filtersActive = hasActiveFilters(filters, workspace);
  const trimmedQuery = query.trim();

  if (workspace === "calibration") {
    if (!trimmedQuery) {
      if (scope === "archive" && count === 0 && !filtersActive) {
        return "No archived calibration equipment yet";
      }
      if (filtersActive) {
        return scope === "archive"
          ? `Showing ${count} filtered archived calibration equipment`
          : `Showing ${count} filtered calibration equipment`;
      }
      return scope === "archive"
        ? `Showing all ${count} archived calibration equipment`
        : `Showing all ${count} calibration equipment`;
    }

    if (count === 0) {
      return scope === "archive"
        ? `No archived calibration results for "${trimmedQuery}"`
        : `No calibration results for "${trimmedQuery}"`;
    }

    const suffix = filtersActive ? " after calibration filters" : "";
    return scope === "archive"
      ? `${count} archived calibration results for "${trimmedQuery}"${suffix}`
      : `${count} calibration results for "${trimmedQuery}"${suffix}`;
  }

  if (!trimmedQuery) {
    if (scope === "archive" && count === 0 && !filtersActive) {
      return "No archived entries yet";
    }
    if (filtersActive) {
      return scope === "archive" ? `Showing ${count} filtered archived entries` : `Showing ${count} filtered entries`;
    }
    return scope === "archive" ? `Showing all ${count} archived entries` : `Showing all ${count} entries`;
  }

  if (count === 0) {
    return scope === "archive"
      ? `No archived results for "${trimmedQuery}"`
      : `No results for "${trimmedQuery}"`;
  }

  const suffix = filtersActive ? " after column filters" : "";
  const resultWord = count === 1 ? "result" : "results";
  if (scope === "archive") {
    return `${count} archived ${resultWord} for "${trimmedQuery}"${suffix}`;
  }
  return `${count} ${resultWord} for "${trimmedQuery}"${suffix}`;
}
