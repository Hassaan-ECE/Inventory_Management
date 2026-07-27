import { useState } from "react";

import { DeleteConfirmationDialog } from "@/modules/te-test-equipment/components/shell/DeleteConfirmationDialog";
import { CalibrationMembershipDialog } from "@/modules/te-test-equipment/components/calibration/CalibrationMembershipDialog";
import { CalibrationRosterDialog } from "@/modules/te-test-equipment/components/calibration/CalibrationRosterDialog";
import { EmptyResults } from "@/modules/te-test-equipment/components/EmptyResults";
import { InventoryHeader } from "@/modules/te-test-equipment/components/InventoryHeader";
import { EntryContextMenu, type EntryContextAction } from "@/modules/te-test-equipment/components/EntryContextMenu";
import { EntryDialog } from "@/modules/te-test-equipment/components/EntryDialog";
import { InventoryTable } from "@/modules/te-test-equipment/components/InventoryTable";
import { SearchCard } from "@/modules/te-test-equipment/components/SearchCard";
import { StatusStrip } from "@/modules/te-test-equipment/components/StatusStrip";
import { buildDefaultStatusMessage } from "@/modules/te-test-equipment/components/shell/helpers";
import { useDesktopInventory } from "@/modules/te-test-equipment/components/shell/useDesktopInventory";
import { useDesktopUpdates } from "@/modules/te-test-equipment/components/shell/useDesktopUpdates";
import { useInventoryEntryMutations } from "@/modules/te-test-equipment/components/shell/useInventoryEntryMutations";
import { useInventoryExportActions } from "@/modules/te-test-equipment/components/shell/useInventoryExportActions";
import { useInventoryExternalActions } from "@/modules/te-test-equipment/components/shell/useInventoryExternalActions";
import { useInventoryPreferences } from "@/modules/te-test-equipment/components/shell/useInventoryPreferences";
import { useInventoryViewModel } from "@/modules/te-test-equipment/components/shell/useInventoryViewModel";
import { useStatusAnnouncer } from "@/modules/te-test-equipment/components/shell/useStatusAnnouncer";
import {
  cycleSortState,
  getColumnsForWorkspace,
  getDefaultFilters,
  getVisibleDataColumnCount,
} from "@/modules/te-test-equipment/lib";
import type {
  CalibrationRequirement,
  ColumnKey,
  FilterState,
  InventoryScope,
  SortState,
  TeTestEquipmentWorkspace,
} from "@/modules/te-test-equipment/types";
import {
  TE_TEST_EQUIPMENT_CALIBRATION_VIEW_ID,
  type DesktopModuleViewProps,
  type InventoryViewId,
} from "@/platform/modules/types";

interface ContextMenuState {
  entryId: string;
  x: number;
  y: number;
}

interface WorkspaceViewState {
  filters: FilterState;
  filtersOpen: boolean;
  query: string;
  sortState: SortState | null;
}

interface CalibrationMembershipState {
  entryId?: string;
  mode: "add" | "remove";
}

export function TeTestEquipmentView({
  active,
  activeViewId,
  onThemeToggle,
  onViewChange,
  theme,
}: DesktopModuleViewProps) {
  const { announceStatus, statusOverride } = useStatusAnnouncer();
  const workspace: TeTestEquipmentWorkspace = activeViewId === TE_TEST_EQUIPMENT_CALIBRATION_VIEW_ID
    ? "calibration"
    : "equipment";
  const [workspaceViews, setWorkspaceViews] = useState<Record<TeTestEquipmentWorkspace, WorkspaceViewState>>(() => ({
    equipment: {
      filters: { ...getDefaultFilters("equipment") },
      filtersOpen: false,
      query: "",
      sortState: { column: "manufacturer", direction: "asc" },
    },
    calibration: {
      filters: { ...getDefaultFilters("calibration") },
      filtersOpen: false,
      query: "",
      sortState: { column: "calibrationDueAt", direction: "asc" },
    },
  }));
  const {
    dataSource,
    entries,
    isLoading,
    refreshDesktopEntries,
    scheduleDesktopSync,
    setEntries,
    setSharedStatus,
    sharedStatus,
  } = useDesktopInventory({
    announceStatus,
    teActive: active,
  });
  const { handleUpdateAction, updateState } = useDesktopUpdates({ active, announceStatus });
  const { colorRows, columnVisibility, setColorRows, setColumnVisibility } = useInventoryPreferences(workspace);
  const [scope, setScope] = useState<InventoryScope>("inventory");
  const [contextMenu, setContextMenu] = useState<ContextMenuState | null>(null);
  const [membershipDialog, setMembershipDialog] = useState<CalibrationMembershipState | null>(null);
  const [rosterDialogOpen, setRosterDialogOpen] = useState(false);
  const { filters, filtersOpen, query, sortState } = workspaceViews[workspace];
  const workspaceColumns = getColumnsForWorkspace(workspace);

  // Collapse filters when leaving a table (Equipment ↔ Calibration) or the TE module.
  // Adjust during render when workspace/active flips — avoids setState-in-effect lint.
  const [filterCollapseKey, setFilterCollapseKey] = useState(`${workspace}:${active}`);
  const nextFilterCollapseKey = `${workspace}:${active}`;
  if (filterCollapseKey !== nextFilterCollapseKey) {
    setFilterCollapseKey(nextFilterCollapseKey);
    setWorkspaceViews((current) => ({
      equipment: { ...current.equipment, filtersOpen: false },
      calibration: { ...current.calibration, filtersOpen: false },
    }));
  }
  const {
    counts,
    displayEntries,
    entriesById,
    resultsLabel,
    statusCounts,
    visibleColumns,
    localDate,
  } = useInventoryViewModel({
    columnVisibility,
    entries,
    filters,
    isLoading,
    query,
    scope,
    sortState,
    workspace,
  });
  const canModifyEntries = dataSource !== "desktop" || sharedStatus.canModify;
  const {
    cancelDeleteEntry,
    closeDialog,
    dialogState,
    handleAddEntry,
    handleArchiveChange,
    handleCalibrationMembershipChange,
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
    ? "Loading TE Test Equipment Inventory database..."
    : statusOverride ?? buildDefaultStatusMessage(counts.total, counts.verified, dataSource, sharedStatus);
  const dialogEntry = dialogState?.mode === "edit" ? entriesById.get(dialogState.entryId ?? "") ?? null : null;
  const contextEntry = contextMenu ? entriesById.get(contextMenu.entryId) ?? null : null;
  const pendingDeleteEntry = pendingDeleteEntryId ? entriesById.get(pendingDeleteEntryId) ?? null : null;
  const membershipEntry = membershipDialog?.entryId ? entriesById.get(membershipDialog.entryId) ?? null : null;
  const calibrationCandidates = entries.filter((entry) => (
    !entry.archived && entry.calibrationRequirement !== "required"
  ));
  const calibrationRosterAvailable = Boolean(
    dataSource === "desktop"
      && window.inventoryDesktop?.pickCalibrationRosterFile
      && window.inventoryDesktop.previewCalibrationRoster
      && window.inventoryDesktop.commitCalibrationRoster,
  );

  function updateWorkspaceView(patch: Partial<WorkspaceViewState>): void {
    setWorkspaceViews((current) => ({
      ...current,
      [workspace]: { ...current[workspace], ...patch },
    }));
  }

  function handleFilterChange(field: keyof FilterState, value: string): void {
    updateWorkspaceView({ filters: { ...filters, [field]: value } });
  }

  function handleClearFilters(): void {
    updateWorkspaceView({ filters: { ...getDefaultFilters(workspace) } });
  }

  function handleSortChange(column: ColumnKey): void {
    updateWorkspaceView({ sortState: cycleSortState(sortState, column) });
  }

  function handleViewChange(nextViewId: InventoryViewId): void {
    setContextMenu(null);
    setMembershipDialog(null);
    onViewChange(nextViewId);
  }

  function handleOpenContextMenu(entryId: string, clientX: number, clientY: number): void {
    const menuWidth = 240;
    const entry = entriesById.get(entryId);
    const menuHeight = entry?.links.trim() ? 252 : 212;
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
        handleOpenEntry(entryId, workspace);
        return;
      case "open-link":
        await handleOpenEntryLink(entryId);
        return;
      case "search-online":
        await handleSearchOnline(entryId);
        return;
      case "calibration-add":
        await handleCalibrationMembershipChange([entryId], "required");
        return;
      case "calibration-remove":
        setMembershipDialog({ entryId, mode: "remove" });
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
      const visibleDataColumns = getVisibleDataColumnCount(current, workspaceColumns);

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
        activeViewId={activeViewId}
        archiveCount={counts.archive}
        calibrationRosterAvailable={calibrationRosterAvailable}
        canModifyEntries={canModifyEntries}
        inventoryCount={counts.inventory}
        onAddEntry={() => {
          if (workspace === "calibration") {
            setMembershipDialog({ mode: "add" });
          } else {
            handleAddEntry({ defaultSection: "equipment" });
          }
        }}
        onExportExcel={() => {
          void handleExportExcel();
        }}
        onExportHtml={handleExportHtml}
        onInitializeCalibrationRoster={() => setRosterDialogOpen(true)}
        onScopeChange={setScope}
        onThemeToggle={onThemeToggle}
        onUpdateAction={() => {
          void handleUpdateAction();
        }}
        onViewChange={handleViewChange}
        scope={scope}
        sharedStatus={sharedStatus}
        theme={theme}
        updateState={updateState}
        workspace={workspace}
      />

      <div className="flex min-h-0 flex-1 overflow-hidden px-2 py-1.5 sm:px-3">
        <div className="flex min-h-0 w-full flex-1 flex-col gap-2 overflow-hidden">
          <SearchCard
            colorRows={colorRows}
            filters={filters}
            filtersOpen={filtersOpen}
            onColorRowsChange={setColorRows}
            onFilterChange={handleFilterChange}
            onFiltersClear={handleClearFilters}
            onFiltersToggle={() => updateWorkspaceView({ filtersOpen: !filtersOpen })}
            onQueryChange={(nextQuery) => updateWorkspaceView({ query: nextQuery })}
            query={query}
            scope={scope}
            workspace={workspace}
          />

          <div className="min-h-0 flex-1 overflow-hidden">
            {isLoading ? (
              <section className="flex h-full min-h-0 flex-1 items-center justify-center rounded-xl border border-border/70 bg-card/80 shadow-sm">
                <div className="text-sm text-muted-foreground">Loading TE Test Equipment Inventory database...</div>
              </section>
            ) : displayEntries.length > 0 ? (
              <InventoryTable
                activeEntryId={contextMenu?.entryId ?? dialogEntry?.id ?? null}
                allColumns={workspaceColumns}
                canModifyEntries={canModifyEntries}
                colorRows={colorRows}
                columnVisibility={columnVisibility}
                columns={visibleColumns}
                onOpenContextMenu={handleOpenContextMenu}
                onOpenEntry={(entryId) => handleOpenEntry(entryId, workspace)}
                onOpenExternalLink={(url) => {
                  void handleOpenExternalLink(url);
                }}
                onSortChange={handleSortChange}
                onToggleColumn={handleToggleColumn}
                onToggleVerified={(entryId) => {
                  void handleToggleVerified(entryId);
                }}
                entries={displayEntries}
                sortState={sortState}
                localDate={localDate}
                workspace={workspace}
              />
            ) : (
              <EmptyResults
                query={query}
                scope={scope}
                workspace={workspace}
                onAddEntry={() => {
                  if (workspace === "calibration") {
                    setMembershipDialog({ mode: "add" });
                  } else {
                    handleAddEntry({ defaultSection: "equipment" });
                  }
                }}
              />
            )}
          </div>
        </div>
      </div>

      <StatusStrip
        counts={statusCounts}
        message={statusMessage}
        resultsLabel={resultsLabel}
        workspace={workspace}
      />

      {contextMenu && contextEntry ? (
        <EntryContextMenu
          canModifyEntries={canModifyEntries}
          position={{ x: contextMenu.x, y: contextMenu.y }}
          entry={contextEntry}
          scope={scope}
          workspace={workspace}
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
          defaultCalibrationRequirement={dialogState.defaultCalibrationRequirement}
          defaultSection={dialogState.defaultSection ?? workspace}
          mode={dialogState.mode}
          readOnly={dataSource === "desktop" && !canModifyEntries}
          entry={dialogEntry}
          onClose={closeDialog}
          onDelete={
            dialogState.mode === "edit" && dialogState.entryId
              ? () => {
                  const entryId = dialogState.entryId!;
                  closeDialog();
                  handleRequestDeleteEntry(entryId);
                }
              : undefined
          }
          onSave={handleSaveEntry}
        />
      ) : null}

      {membershipDialog ? (
        <CalibrationMembershipDialog
          candidates={calibrationCandidates}
          entry={membershipEntry}
          mode={membershipDialog.mode}
          readOnly={!canModifyEntries}
          onAdd={(entryIds) => handleCalibrationMembershipChange(entryIds, "required")}
          onClose={() => setMembershipDialog(null)}
          onCreateNew={() => {
            setMembershipDialog(null);
            handleAddEntry({
              defaultCalibrationRequirement: "required",
              defaultSection: "equipment",
            });
          }}
          onRemove={(requirement: Exclude<CalibrationRequirement, "required">) => (
            membershipEntry
              ? handleCalibrationMembershipChange([membershipEntry.id], requirement)
              : false
          )}
        />
      ) : null}

      {rosterDialogOpen ? (
        <CalibrationRosterDialog
          onClose={() => setRosterDialogOpen(false)}
          onCommitted={async (result) => {
            setScope("inventory");
            setWorkspaceViews((current) => ({
              ...current,
              calibration: {
                ...current.calibration,
                filters: { ...getDefaultFilters("calibration") },
                query: "",
              },
            }));
            handleViewChange(TE_TEST_EQUIPMENT_CALIBRATION_VIEW_ID);
            if (result.entriesChanged) {
              await refreshDesktopEntries({ preserveEntriesOnError: true });
              scheduleDesktopSync();
            }
            announceStatus(result.message);
          }}
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
