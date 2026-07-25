import { useDeferredValue, useMemo } from "react";

import {
  buildResultsLabel,
  filterEntries,
  getColumnsForWorkspace,
  getInventoryCounts,
  getLocalDateString,
  getVisibleColumns,
  sortEntries,
} from "@/modules/te-test-equipment/lib";
import type {
  ColumnKey,
  FilterState,
  InventoryEntry,
  InventoryScope,
  SortState,
  TeTestEquipmentWorkspace,
} from "@/modules/te-test-equipment/types";

interface UseInventoryViewModelOptions {
  columnVisibility: Record<ColumnKey, boolean>;
  entries: InventoryEntry[];
  filters: FilterState;
  isLoading: boolean;
  query: string;
  scope: InventoryScope;
  sortState: SortState | null;
  workspace: TeTestEquipmentWorkspace;
}

const LOADING_ENTRIES: InventoryEntry[] = [];

export function useInventoryViewModel({
  columnVisibility,
  entries,
  filters,
  isLoading,
  query,
  scope,
  sortState,
  workspace,
}: UseInventoryViewModelOptions) {
  const sourceEntries = isLoading ? LOADING_ENTRIES : entries;
  const localDate = getLocalDateString();
  const deferredQuery = useDeferredValue(query);
  const deferredFilters = useDeferredValue(filters);
  const filteredEntries = useMemo(
    () => filterEntries(sourceEntries, scope, deferredQuery, deferredFilters, localDate),
    [deferredFilters, deferredQuery, localDate, scope, sourceEntries],
  );
  const sortedEntries = useMemo(() => sortEntries(filteredEntries, sortState, localDate), [filteredEntries, localDate, sortState]);
  const counts = useMemo(() => getInventoryCounts(sourceEntries, localDate), [localDate, sourceEntries]);
  const calibrationTrackedEntries = useMemo(
    () => sourceEntries.filter((entry) => (
      entry.calibrationRequirement === "required" &&
      (scope === "archive" ? entry.archived : !entry.archived)
    )),
    [scope, sourceEntries],
  );
  const statusCounts = useMemo(
    () => getInventoryCounts(workspace === "calibration" ? calibrationTrackedEntries : sourceEntries, localDate),
    [calibrationTrackedEntries, localDate, sourceEntries, workspace],
  );
  const workspaceColumns = useMemo(() => getColumnsForWorkspace(workspace), [workspace]);
  const visibleColumns = useMemo(
    () => getVisibleColumns(columnVisibility, workspaceColumns),
    [columnVisibility, workspaceColumns],
  );
  const entriesById = useMemo(() => {
    const map = new Map<string, InventoryEntry>();
    for (const entry of sourceEntries) {
      map.set(entry.id, entry);
    }
    return map;
  }, [sourceEntries]);

  return {
    counts,
    deferredFilters,
    deferredQuery,
    displayCount: sortedEntries.length,
    displayEntries: sortedEntries,
    entriesById,
    resultsLabel: isLoading
      ? "Loading inventory entries..."
      : buildResultsLabel(sortedEntries.length, scope, deferredQuery, deferredFilters, workspace),
    statusCounts,
    trackedCount: calibrationTrackedEntries.length,
    visibleColumns,
    localDate,
  };
}
