import { useEffect, useState } from "react";

import { CATALOG_COLUMNS, type CatalogColumnKey } from "@/modules/te-lab-components/catalog/catalogColumns";
import { EMPTY_CATALOG_FILTERS, type CatalogFilters } from "@/modules/te-lab-components/catalog/catalogFilters";
import type { CatalogSortState } from "@/modules/te-lab-components/catalog/catalogSorting";

// v5: five-column simple workspace defaults (Component Type, Value, Quantity, Location).
const COLUMN_VISIBILITY_KEY = "teLabComponents.catalog.v5.columnVisibility";
const COLOR_ROWS_KEY = "teLabComponents.catalog.v1.colorRows";
const FILTER_PREFERENCES_KEY = "teLabComponents.catalog.v2.filters";
const SORT_PREFERENCES_KEY = "teLabComponents.catalog.v2.sort";
const DEFAULT_SORT_STATE: CatalogSortState = { column: "subcategory", direction: "asc" };

/** The catalog view settings this computer remembers: columns, row colors, filters, and sort. */
export function useCatalogPreferences() {
  const [filters, setFilters] = useState<CatalogFilters>(readFilterPreferences);
  const [sortState, setSortState] = useState<CatalogSortState | null>(readSortPreferences);
  const [columnVisibility, setColumnVisibility] = useState<Record<CatalogColumnKey, boolean>>(readColumnVisibility);
  const [colorRows, setColorRows] = useState<boolean>(readColorRows);

  useEffect(() => {
    localStorage.setItem(COLUMN_VISIBILITY_KEY, JSON.stringify(columnVisibility));
  }, [columnVisibility]);

  useEffect(() => {
    localStorage.setItem(COLOR_ROWS_KEY, JSON.stringify(colorRows));
  }, [colorRows]);

  useEffect(() => {
    localStorage.setItem(FILTER_PREFERENCES_KEY, JSON.stringify(filters));
  }, [filters]);

  useEffect(() => {
    localStorage.setItem(SORT_PREFERENCES_KEY, JSON.stringify(sortState));
  }, [sortState]);

  return {
    colorRows,
    columnVisibility,
    filters,
    setColorRows,
    setColumnVisibility,
    setFilters,
    setSortState,
    sortState,
  };
}

function readColumnVisibility(): Record<CatalogColumnKey, boolean> {
  const defaults = Object.fromEntries(CATALOG_COLUMNS.map((column) => [column.key, column.defaultVisible])) as Record<CatalogColumnKey, boolean>;
  try {
    const saved = JSON.parse(localStorage.getItem(COLUMN_VISIBILITY_KEY) ?? "{}") as Partial<Record<CatalogColumnKey, boolean>>;
    return { ...defaults, ...saved };
  } catch {
    return defaults;
  }
}

function readColorRows(): boolean {
  try {
    return JSON.parse(localStorage.getItem(COLOR_ROWS_KEY) ?? "true") === true;
  } catch {
    return true;
  }
}

function readFilterPreferences(): CatalogFilters {
  try {
    const saved = JSON.parse(localStorage.getItem(FILTER_PREFERENCES_KEY) ?? "{}") as Partial<CatalogFilters> & {
      sortBy?: string;
    };
    const filterFields = { ...saved };
    delete filterFields.sortBy;
    return {
      ...EMPTY_CATALOG_FILTERS,
      ...filterFields,
    };
  } catch {
    return { ...EMPTY_CATALOG_FILTERS };
  }
}

function readSortPreferences(): CatalogSortState | null {
  try {
    const raw = localStorage.getItem(SORT_PREFERENCES_KEY);
    if (!raw) {
      // Migrate legacy sortBy preference once.
      const legacy = JSON.parse(localStorage.getItem(FILTER_PREFERENCES_KEY) ?? "{}") as { sortBy?: string };
      if (legacy.sortBy && CATALOG_COLUMNS.some((column) => column.key === legacy.sortBy && column.sortable)) {
        return { column: legacy.sortBy as CatalogColumnKey, direction: "asc" };
      }
      return DEFAULT_SORT_STATE;
    }
    const saved = JSON.parse(raw) as CatalogSortState | null;
    if (!saved) {
      return null;
    }
    if (
      !CATALOG_COLUMNS.some((column) => column.key === saved.column && column.sortable) ||
      (saved.direction !== "asc" && saved.direction !== "desc")
    ) {
      return DEFAULT_SORT_STATE;
    }
    return saved;
  } catch {
    return DEFAULT_SORT_STATE;
  }
}
