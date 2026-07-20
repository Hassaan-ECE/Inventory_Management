import { useState } from "react";

import { DeleteConfirmationDialog } from "@/modules/te-lab-components/components/shell/DeleteConfirmationDialog";
import { EmptyResults } from "@/modules/te-lab-components/components/EmptyResults";
import { InventoryHeader } from "@/modules/te-lab-components/components/InventoryHeader";
import { EntryContextMenu, type EntryContextAction } from "@/modules/te-lab-components/components/EntryContextMenu";
import { EntryDialog } from "@/modules/te-lab-components/components/EntryDialog";
import { InventoryTable } from "@/modules/te-lab-components/components/InventoryTable";
import { SearchCard } from "@/modules/te-lab-components/components/SearchCard";
import { StatusStrip } from "@/modules/te-lab-components/components/StatusStrip";
import { buildDefaultStatusMessage } from "@/modules/te-lab-components/components/shell/helpers";
import { useDesktopInventory } from "@/modules/te-lab-components/components/shell/useDesktopInventory";
import { useDesktopUpdates } from "@/modules/te-lab-components/components/shell/useDesktopUpdates";
import { useInventoryEntryMutations } from "@/modules/te-lab-components/components/shell/useInventoryEntryMutations";
import { useInventoryExportActions } from "@/modules/te-lab-components/components/shell/useInventoryExportActions";
import { useInventoryExternalActions } from "@/modules/te-lab-components/components/shell/useInventoryExternalActions";
import { useInventoryPreferences } from "@/modules/te-lab-components/components/shell/useInventoryPreferences";
import { useInventoryViewModel } from "@/modules/te-lab-components/components/shell/useInventoryViewModel";
import { useStatusAnnouncer } from "@/modules/te-lab-components/components/shell/useStatusAnnouncer";
import { DEFAULT_FILTERS, getVisibleDataColumnCount } from "@/modules/te-lab-components/lib";
import { INVENTORY_COLUMNS } from "@/modules/te-lab-components/types";
import type { ColumnKey, FilterState, InventoryScope, SortState } from "@/modules/te-lab-components/types";
import type { DesktopModuleViewProps } from "@/platform/modules/types";

interface ContextMenuState {
  entryId: string;
  x: number;
  y: number;
}

export function TeLabComponentsView({
  active,
  activeModuleId,
  onModuleChange,
  onThemeToggle,
  theme,
}: DesktopModuleViewProps) {
  const { announceStatus, statusOverride } = useStatusAnnouncer();
  const {
    dataSource,
    entries,
    isLoading,
    scheduleDesktopSync,
    setEntries,
    setSharedStatus,
    sharedStatus,
  } = useDesktopInventory({
    announceStatus,
    active,
  });
  const { handleUpdateAction, updateState } = useDesktopUpdates({ active, announceStatus });
  const { colorRows, columnVisibility, setColorRows, setColumnVisibility } = useInventoryPreferences();
  const [scope, setScope] = useState<InventoryScope>("inventory");
  const [query, setQuery] = useState("");
  const [filters, setFilters] = useState<FilterState>(DEFAULT_FILTERS);
  const [filtersOpen, setFiltersOpen] = useState(false);
  const [sortState, setSortState] = useState<SortState>({ column: "manufacturer", direction: "asc" });
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const {
    counts,
    displayEntries,
    entriesById,
    resultsLabel,
    visibleColumns,
  } = useInventoryViewModel({
    columnVisibility,
    entries,
    filters,
    isLoading,
    query,
    scope,
    sortState,
  });
  const canModifyEntries = dataSource !== "desktop" || sharedStatus.canModify;
  const {
    cancelDeleteEntry,
    closeDialog,
    dialogState,
    handleAddEntry,
    handleArchiveChange,
    handleConfirmDeleteEntry,
    handleOpenEntry,
    handleRequestDeleteEntry,
    handleSaveEntry,
    handleToggleVerified,
    pendingDeleteEntryId,
  } = useInventoryEntryMutations({
    announceStatus,
    canModifyEntries,
    dataSource,
    entriesById,
    onDialogOpen: () => setContextMenu(null),
    scheduleDesktopSync,
    setEntries,
    setSharedStatus,
    sharedStatus,
  });
  const { handleExportExcel, handleExportHtml } = useInventoryExportActions({ announceStatus });
  const { handleOpenEntryLink, handleOpenExternalLink, handleSearchOnline } = useInventoryExternalActions({
    announceStatus,
    entriesById,
  });
  const statusMessage = isLoading
    ? "Loading TE Lab Components Inventory database..."
    : statusOverride ?? buildDefaultStatusMessage(counts.total, counts.verified, dataSource, sharedStatus);
  const dialogEntry = dialogState?.mode === "edit" ? entriesById.get(dialogState.entryId ?? "") ?? null : null;
  const contextEntry = contextMenu ? entriesById.get(contextMenu.entryId) ?? null : null;
  const pendingDeleteEntry = pendingDeleteEntryId ? entriesById.get(pendingDeleteEntryId) ?? null : null;

  function handleFilterChange(field: keyof FilterState, value: string): void {
    setFilters((current) => ({ ...current, [field]: value }));
  }

  function handleClearFilters(): void {
    setFilters(DEFAULT_FILTERS);
  }

  function handleSortChange(column: ColumnKey): void {
    setSortState((current) => ({
      column,
      direction: current.column === column && current.direction === "asc" ? "desc" : "asc",
    }));
  }

  function handleOpenContextMenu(entryId: string, clientX: number, clientY: number): void {
    const menuWidth = 240;
    const entry = entriesById.get(entryId);
    const menuHeight = entry?.links.trim() ? 212 : 172;
    const maxX = typeof window === "undefined" ? clientX : Math.max(12, window.innerWidth - menuWidth - 12);
    const maxY = typeof window === "undefined" ? clientY : Math.max(12, window.innerHeight - menuHeight - 12);

    setContextMenu({
      entryId,
      x: Math.min(clientX, maxX),
      y: Math.min(clientY, maxY),
    });
  }

  async function handleContextAction(action: EntryContextAction): Promise<void> {
    const entryId = contextMenu?.entryId;
    setContextMenu(null);

    if (!entryId) {
      return;
    }

    switch (action) {
      case "open":
        handleOpenEntry(entryId);
        return;
      case "open-link":
        await handleOpenEntryLink(entryId);
        return;
      case "search-online":
        await handleSearchOnline(entryId);
        return;
      case "archive-toggle": {
        const entry = entriesById.get(entryId);
        if (!entry) {
          return;
        }
        await handleArchiveChange(entryId, !entry.archived);
        return;
      }
      case "delete":
        handleRequestDeleteEntry(entryId);
        return;
    }
  }

  function handleToggleColumn(columnKey: ColumnKey): void {
    setColumnVisibility((current) => {
      const nextValue = !current[columnKey];
      const visibleDataColumns = getVisibleDataColumnCount(current);

      if (!nextValue && columnKey !== "verified" && visibleDataColumns === 1) {
        return current;
      }

      return { ...current, [columnKey]: nextValue };
    });
  }

  if (!active) {
    return null;
  }

  return (
    <>
      <InventoryHeader
        activeModuleId={activeModuleId}
        archiveCount={counts.archive}
        canModifyEntries={canModifyEntries}
        inventoryCount={counts.inventory}
        onAddEntry={handleAddEntry}
        onExportExcel={() => {
          void handleExportExcel();
        }}
        onExportHtml={handleExportHtml}
        onModuleChange={onModuleChange}
        onScopeChange={setScope}
        onThemeToggle={onThemeToggle}
        onUpdateAction={() => {
          void handleUpdateAction();
        }}
        scope={scope}
        sharedStatus={sharedStatus}
        theme={theme}
        updateState={updateState}
      />

      <div className="flex min-h-0 flex-1 overflow-hidden px-2 py-1.5 sm:px-3">
        <div className="flex min-h-0 w-full flex-1 flex-col gap-2 overflow-hidden">
          <SearchCard
            colorRows={colorRows}
            columns={INVENTORY_COLUMNS}
            columnVisibility={columnVisibility}
            filters={filters}
            filtersOpen={filtersOpen}
            onColorRowsChange={setColorRows}
            onFilterChange={handleFilterChange}
            onFiltersClear={handleClearFilters}
            onFiltersToggle={() => setFiltersOpen((current) => !current)}
            onQueryChange={setQuery}
            onToggleColumn={handleToggleColumn}
            query={query}
            scope={scope}
          />

          <div className="min-h-0 flex-1 overflow-hidden">
            {isLoading ? (
              <section className="flex h-full min-h-0 flex-1 items-center justify-center rounded-xl border border-border/70 bg-card/80 shadow-sm">
                <div className="text-sm text-muted-foreground">Loading TE Lab Components Inventory database...</div>
              </section>
            ) : displayEntries.length > 0 ? (
              <InventoryTable
                activeEntryId={contextMenu?.entryId ?? dialogEntry?.id ?? null}
                canModifyEntries={canModifyEntries}
                colorRows={colorRows}
                columns={visibleColumns}
                onOpenContextMenu={handleOpenContextMenu}
                onOpenEntry={handleOpenEntry}
                onOpenExternalLink={(url) => {
                  void handleOpenExternalLink(url);
                }}
                onSortChange={handleSortChange}
                onToggleVerified={(entryId) => {
                  void handleToggleVerified(entryId);
                }}
                entries={displayEntries}
                sortState={sortState}
              />
            ) : (
              <EmptyResults query={query} scope={scope} onAddEntry={handleAddEntry} />
            )}
          </div>
        </div>
      </div>

      <StatusStrip counts={counts} message={statusMessage} resultsLabel={resultsLabel} />

      {contextMenu && contextEntry ? (
        <EntryContextMenu
          canModifyEntries={canModifyEntries}
          position={{ x: contextMenu.x, y: contextMenu.y }}
          entry={contextEntry}
          scope={scope}
          onAction={(action) => {
            void handleContextAction(action);
          }}
          onClose={() => setContextMenu(null)}
        />
      ) : null}

      {dialogState ? (
        <EntryDialog
          key={`${dialogState.mode}-${dialogState.entryId ?? scope}`}
          defaultArchived={scope === "archive"}
          mode={dialogState.mode}
          readOnly={dataSource === "desktop" && !canModifyEntries}
          entry={dialogEntry}
          onClose={closeDialog}
          onSave={handleSaveEntry}
        />
      ) : null}

      {pendingDeleteEntry ? (
        <DeleteConfirmationDialog
          entry={pendingDeleteEntry}
          onCancel={cancelDeleteEntry}
          onConfirm={() => {
            void handleConfirmDeleteEntry(pendingDeleteEntry.id);
          }}
        />
      ) : null}
    </>
  );
}
