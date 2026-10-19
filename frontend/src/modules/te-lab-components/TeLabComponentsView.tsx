import { useMemo, useState } from "react";

import { CatalogHeader } from "@/modules/te-lab-components/catalog/CatalogHeader";
import { CATALOG_COLUMNS } from "@/modules/te-lab-components/catalog/catalogColumns";
import {
  activeCatalogFilterCount,
  catalogFilterOptions,
  EMPTY_CATALOG_FILTERS,
  filterCatalogParts,
} from "@/modules/te-lab-components/catalog/catalogFilters";
import { useCatalogPreferences } from "@/modules/te-lab-components/catalog/catalogPreferences";
import { CatalogSearchCard } from "@/modules/te-lab-components/catalog/CatalogSearchCard";
import { cycleCatalogSortState, sortCatalogParts } from "@/modules/te-lab-components/catalog/catalogSorting";
import { createCatalogLookups } from "@/modules/te-lab-components/catalog/catalogUtils";
import {
  labLocationsAvailable,
  requireLabCatalogBridge,
  requireLabLocationBridge,
} from "@/modules/te-lab-components/catalog/labCatalogBridge";
import { LocationManagerDialog } from "@/modules/te-lab-components/catalog/LocationManagerDialog";
import {
  OrderExportDialog,
  type OrderDraftLine,
} from "@/modules/te-lab-components/catalog/OrderExportDialog";
import { OrderSelectionBar } from "@/modules/te-lab-components/catalog/OrderSelectionBar";
import { PartDialog } from "@/modules/te-lab-components/catalog/PartDialog";
import {
  CountStockDialog,
  MoveStockDialog,
  PlacementDialog,
} from "@/modules/te-lab-components/catalog/PlacementDialogs";
import { PartsTable } from "@/modules/te-lab-components/catalog/PartsTable";
import { useLabCatalog } from "@/modules/te-lab-components/catalog/useLabCatalog";
import { useDesktopUpdates } from "@/platform/updates/useDesktopUpdates";
import { useStatusAnnouncer } from "@/shared/hooks/useStatusAnnouncer";
import { StatusStrip } from "@/modules/te-lab-components/components/StatusStrip";
import type {
  CatalogScope,
  LabOrderRequestLineInput,
  Part,
  PartInput,
  SimpleComponentInput,
  StockCountInput,
  StockMoveInput,
  StockPlacement,
  StockPlacementInput,
  StorageArea,
  StorageAreaInput,
  StorageContainer,
  StorageContainerInput,
} from "@/modules/te-lab-components/types";
import type { DesktopModuleViewProps } from "@/platform/modules/types";
import { Button } from "@/shared/components/ui/button";

interface PlacementDialogState {
  partUuid: string;
  placementUuid?: string;
}

export function TeLabComponentsView({
  active,
  activeViewId,
  onThemeToggle,
  onViewChange,
  theme,
}: DesktopModuleViewProps) {
  const { announceStatus, statusOverride } = useStatusAnnouncer();
  const { handleUpdateAction, updateState } = useDesktopUpdates({ active, announceStatus });
  const { catalog, dataSource, isLoading, lastError, refreshAfterMutation } = useLabCatalog({ active, announceStatus });
  const [scope, setScope] = useState<CatalogScope>("inventory");
  const [query, setQuery] = useState("");
  const {
    colorRows,
    columnVisibility,
    filters,
    setColorRows,
    setColumnVisibility,
    setFilters,
    setSortState,
    sortState,
  } = useCatalogPreferences();
  const [filtersOpen, setFiltersOpen] = useState(false);
  const [partDialogId, setPartDialogId] = useState<string | "new" | null>(null);
  const [placementDialog, setPlacementDialog] = useState<PlacementDialogState | null>(null);
  const [movePlacementUuid, setMovePlacementUuid] = useState<string | null>(null);
  const [countPlacementUuid, setCountPlacementUuid] = useState<string | null>(null);
  const [locationsOpen, setLocationsOpen] = useState(false);
  const [orderSelectionMode, setOrderSelectionMode] = useState(false);
  const [selectedPartIds, setSelectedPartIds] = useState<Set<string>>(() => new Set());
  const [orderDialogOpen, setOrderDialogOpen] = useState(false);
  const [orderDrafts, setOrderDrafts] = useState<Record<string, OrderDraftLine>>({});
  const lookups = useMemo(() => createCatalogLookups(catalog), [catalog]);
  const filterOptions = useMemo(() => catalogFilterOptions(catalog.parts), [catalog.parts]);
  const filteredContainerOptions = catalog.storageContainers.filter(
    (container) => !filters.areaUuid || container.areaUuid === filters.areaUuid,
  );
  const activeFilterCount = activeCatalogFilterCount(filters);
  const hasCatalogFilters = Boolean(query || activeFilterCount > 0);
  const canModify = dataSource !== "desktop" || catalog.shared.canModify;
  const locationsAvailable = dataSource === "desktop" && labLocationsAvailable();
  const activePart = partDialogId && partDialogId !== "new" ? lookups.partsById.get(partDialogId) ?? null : null;
  const placementPart = placementDialog ? lookups.partsById.get(placementDialog.partUuid) ?? null : null;
  const editedPlacement = placementDialog?.placementUuid
    ? catalog.stockPlacements.find((placement) => placement.placementUuid === placementDialog.placementUuid) ?? null
    : null;
  const movePlacement = movePlacementUuid
    ? catalog.stockPlacements.find((placement) => placement.placementUuid === movePlacementUuid) ?? null
    : null;
  const countPlacement = countPlacementUuid
    ? catalog.stockPlacements.find((placement) => placement.placementUuid === countPlacementUuid) ?? null
    : null;

  // Collapse filters / order selection when leaving Lab Components (switcher / other modules).
  // Adjust during render when `active` flips — avoids setState-in-effect lint.
  const [filtersOpenActive, setFiltersOpenActive] = useState(active);
  if (filtersOpenActive !== active) {
    setFiltersOpenActive(active);
    if (!active) {
      if (filtersOpen) {
        setFiltersOpen(false);
      }
      if (orderSelectionMode || selectedPartIds.size > 0 || orderDialogOpen) {
        setOrderSelectionMode(false);
        setSelectedPartIds(new Set());
        setOrderDialogOpen(false);
        setOrderDrafts({});
      }
    }
  }

  const displayParts = useMemo(
    () => sortCatalogParts(filterCatalogParts(catalog, lookups, scope, filters, query), sortState, lookups),
    [catalog, filters, lookups, query, scope, sortState],
  );

  async function savePart(part: Part | null, input: PartInput): Promise<void> {
    const bridge = requireLabCatalogBridge();
    const result = part
      ? await bridge.updateLabPart(part.entryUuid, input)
      : await bridge.createLabPart(input);
    await refreshAfterMutation(result.message);
    setPartDialogId(null);
  }

  async function saveSimplePart(part: Part | null, input: SimpleComponentInput): Promise<void> {
    const bridge = requireLabCatalogBridge();
    const result = part
      ? await bridge.updateLabSimpleComponent(part.entryUuid, input)
      : await bridge.createLabSimpleComponent(input);
    await refreshAfterMutation(result.message);
    setPartDialogId(null);
  }

  async function expandGrid(container: StorageContainer, axis: "row" | "column"): Promise<void> {
    if (!container.gridEnabled || typeof window.inventoryDesktop?.updateLabStorageContainer !== "function") {
      return;
    }
    const nextRows = (container.rowCount ?? 0) + (axis === "row" ? 1 : 0);
    const nextColumns = (container.columnCount ?? 0) + (axis === "column" ? 1 : 0);
    const result = await requireLabLocationBridge().updateLabStorageContainer(container.containerUuid, {
      areaUuid: container.areaUuid,
      name: container.name,
      containerType: container.containerType,
      gridEnabled: true,
      rowCount: nextRows,
      columnCount: nextColumns,
      rowStart: container.rowStart,
      description: container.description,
      archived: container.archived,
    });
    await refreshAfterMutation(result.message);
  }

  async function deletePart(part: Part): Promise<void> {
    if (!part.archived) {
      announceStatus("Archive the part before deleting it permanently.");
      throw new Error("Archive the part before deleting it permanently.");
    }
    try {
      const result = await requireLabCatalogBridge().deleteLabPart(part.entryUuid);
      await refreshAfterMutation(result.message);
      setPartDialogId(null);
    } catch (error) {
      const message = error instanceof Error ? error.message : "Could not delete the part.";
      announceStatus(message);
      throw error instanceof Error ? error : new Error(message);
    }
  }

  async function savePlacement(
    input: StockPlacementInput,
    mergeCandidate: StockPlacement | null,
  ): Promise<void> {
    const bridge = requireLabCatalogBridge();
    if (editedPlacement) {
      const result = await bridge.updateLabStockPlacement(editedPlacement.placementUuid, input);
      await refreshAfterMutation(result.message);
    } else if (mergeCandidate) {
      const result = await bridge.updateLabStockPlacement(
        mergeCandidate.placementUuid,
        mergedPlacementInput(mergeCandidate, input),
      );
      await refreshAfterMutation(
        `Added stock to the existing placement. ${result.message}`.trim(),
      );
    } else {
      const result = await bridge.createLabStockPlacement(input);
      await refreshAfterMutation(result.message);
    }
    setPlacementDialog(null);
  }

  async function deletePlacement(placement: StockPlacement): Promise<void> {
    if (!window.confirm("Delete this stock placement?")) return;
    const result = await requireLabCatalogBridge().deleteLabStockPlacement(placement.placementUuid);
    await refreshAfterMutation(result.message);
  }

  async function moveStock(input: StockMoveInput): Promise<void> {
    const result = await requireLabCatalogBridge().moveLabStock(input);
    await refreshAfterMutation(result.message);
    setMovePlacementUuid(null);
  }

  async function countStock(placement: StockPlacement, input: StockCountInput): Promise<void> {
    const result = await requireLabCatalogBridge().countLabStock(placement.placementUuid, input);
    await refreshAfterMutation(result.message);
    setCountPlacementUuid(null);
  }

  async function exportExcel(): Promise<void> {
    const bridge = window.inventoryDesktop;
    if (!bridge?.exportExcel) {
      announceStatus("Excel export is available in the desktop app.");
      return;
    }
    const result = await bridge.exportExcel("te-lab-components");
    announceStatus(result.canceled ? "Export canceled." : result.error || `Exported Lab catalog to ${result.outputPath ?? "the selected workbook"}.`);
  }

  function clearOrderSelection(): void {
    setOrderSelectionMode(false);
    setSelectedPartIds(new Set());
    setOrderDialogOpen(false);
    setOrderDrafts({});
  }

  function enterOrderSelection(): void {
    setOrderSelectionMode(true);
    setSelectedPartIds(new Set());
    setOrderDialogOpen(false);
    setOrderDrafts({});
  }

  function togglePartSelection(part: Part): void {
    setSelectedPartIds((current) => {
      const next = new Set(current);
      if (next.has(part.entryUuid)) {
        next.delete(part.entryUuid);
      } else {
        next.add(part.entryUuid);
      }
      return next;
    });
  }

  function handleScopeChange(nextScope: CatalogScope): void {
    setScope(nextScope);
    if (orderSelectionMode || selectedPartIds.size > 0 || orderDialogOpen) {
      clearOrderSelection();
    }
  }

  async function exportOrderRequest(lines: LabOrderRequestLineInput[]): Promise<void> {
    const bridge = window.inventoryDesktop;
    if (!bridge?.exportLabOrderRequest) {
      announceStatus("Order export is available in the desktop app.");
      throw new Error("Order export is available in the desktop app.");
    }
    const result = await bridge.exportLabOrderRequest(lines);
    if (result.canceled) {
      announceStatus("Order export canceled.");
      return;
    }
    if (result.error) {
      announceStatus(result.error);
      throw new Error(result.error);
    }
    announceStatus(`Exported order request to ${result.outputPath ?? "the selected workbook"}.`);
    clearOrderSelection();
  }

  async function openExternal(url: string): Promise<void> {
    if (!url) return;
    if (window.inventoryDesktop?.openExternal) {
      await window.inventoryDesktop.openExternal(url);
    } else {
      window.open(url, "_blank", "noopener,noreferrer");
    }
  }

  if (!active) {
    return null;
  }

  return (
    <>
      <CatalogHeader
        activeViewId={activeViewId}
        canModify={canModify}
        counts={catalog.counts}
        onAddPart={() => setPartDialogId("new")}
        onExportExcel={() => {
          void exportExcel();
        }}
        onSelectForOrder={enterOrderSelection}
        onManageLocations={locationsAvailable ? () => setLocationsOpen(true) : undefined}
        onScopeChange={handleScopeChange}
        onThemeToggle={onThemeToggle}
        onUpdateAction={() => {
          void handleUpdateAction();
        }}
        onViewChange={onViewChange}
        scope={scope}
        shared={catalog.shared}
        theme={theme}
        updateState={updateState}
      />

      <div className="flex min-h-0 flex-1 flex-col gap-2 overflow-hidden px-2 py-2 sm:px-3">
        <CatalogSearchCard
          activeFilterCount={activeFilterCount}
          areaOptions={catalog.storageAreas}
          categoryOptions={filterOptions.categories}
          colorRows={colorRows}
          containerOptions={filteredContainerOptions.map((container) => ({
            label: containerOptionLabel(container, lookups),
            value: container.containerUuid,
          }))}
          filters={filters}
          filtersOpen={filtersOpen}
          hasActiveFilters={hasCatalogFilters}
          manufacturerOptions={filterOptions.manufacturers}
          mountingTypeOptions={filterOptions.mountingTypes}
          onClearFilters={() => {
            setQuery("");
            setFilters({ ...EMPTY_CATALOG_FILTERS });
          }}
          onColorRowsChange={setColorRows}
          onFilterChange={(field, value) => {
            setFilters((current) => ({ ...current, [field]: value }));
          }}
          onFiltersToggle={() => setFiltersOpen((current) => !current)}
          onQueryChange={setQuery}
          packageTypeOptions={filterOptions.packageTypes}
          partStatusOptions={filterOptions.partStatuses}
          query={query}
          subcategoryOptions={filterOptions.subcategories}
        />

        {orderSelectionMode ? (
          <OrderSelectionBar
            selectedCount={selectedPartIds.size}
            onCancel={clearOrderSelection}
            onContinue={() => setOrderDialogOpen(true)}
          />
        ) : null}

        <div className="min-h-0 flex-1 overflow-hidden">
          {isLoading ? (
            <section className="flex h-full items-center justify-center rounded-xl border border-border bg-card/70 text-sm text-muted-foreground">Loading Lab Components catalog…</section>
          ) : displayParts.length ? (
            <PartsTable
              catalog={catalog}
              colorRows={colorRows}
              columnVisibility={columnVisibility}
              onOpenExternal={(url) => void openExternal(url)}
              onOpenPart={(part) => setPartDialogId(part.entryUuid)}
              onSortChange={(columnKey) => setSortState((current) => cycleCatalogSortState(current, columnKey))}
              onToggleColumn={(columnKey) => {
                setColumnVisibility((current) => {
                  const nextVisible = !current[columnKey];
                  if (!nextVisible && CATALOG_COLUMNS.filter((column) => current[column.key]).length <= 1) {
                    return current;
                  }
                  return { ...current, [columnKey]: nextVisible };
                });
              }}
              onToggleSelection={togglePartSelection}
              parts={displayParts}
              selectedPartIds={selectedPartIds}
              selectionMode={orderSelectionMode}
              sortState={sortState}
              visibleColumns={columnVisibility}
            />
          ) : (
            <section className="flex h-full flex-col items-center justify-center rounded-xl border border-dashed border-border bg-card/50 p-8 text-center"><h2 className="font-semibold">No catalog parts found</h2><p className="mt-1 text-sm text-muted-foreground">Adjust the search and filters, or add a component.</p>{canModify ? <Button className="mt-4" onClick={() => setPartDialogId("new")}>Add Part</Button> : null}</section>
          )}
        </div>
      </div>

      <StatusStrip
        counts={{ lowStock: catalog.counts.lowStock, noStock: catalog.counts.noStock }}
        message={statusOverride ?? lastError ?? catalog.shared.message}
        warning={catalog.shared.warning}
        resultsLabel={
          isLoading
            ? "Loading catalog…"
            : `${displayParts.length.toLocaleString()} shown`
        }
      />

      {partDialogId ? (
        <PartDialog
          catalog={catalog}
          key={partDialogId}
          onAddPlacement={(part) => setPlacementDialog({ partUuid: part.entryUuid })}
          onClose={() => setPartDialogId(null)}
          onCountPlacement={(placement) => setCountPlacementUuid(placement.placementUuid)}
          onDeletePart={(part) => deletePart(part)}
          onDeletePlacement={(placement) => void deletePlacement(placement)}
          onEditPlacement={(placement) => setPlacementDialog({ partUuid: placement.partUuid, placementUuid: placement.placementUuid })}
          onExpandGrid={(container, axis) => void expandGrid(container, axis)}
          onMovePlacement={(placement) => setMovePlacementUuid(placement.placementUuid)}
          onSaveAdvanced={(input) => savePart(activePart, input)}
          onSaveSimple={(input) => saveSimplePart(activePart, input)}
          part={partDialogId === "new" ? null : activePart}
          readOnly={!canModify}
        />
      ) : null}

      {placementDialog && placementPart ? <PlacementDialog catalog={catalog} key={placementDialog.placementUuid ?? `new-${placementDialog.partUuid}`} onClose={() => setPlacementDialog(null)} onExpandGrid={(container, axis) => void expandGrid(container, axis)} onSave={savePlacement} part={placementPart} placement={editedPlacement} /> : null}
      {movePlacement ? <MoveStockDialog catalog={catalog} key={movePlacement.placementUuid} onClose={() => setMovePlacementUuid(null)} onMove={moveStock} source={movePlacement} /> : null}
      {countPlacement ? <CountStockDialog catalog={catalog} key={countPlacement.placementUuid} onClose={() => setCountPlacementUuid(null)} onCount={(input) => countStock(countPlacement, input)} placement={countPlacement} /> : null}
      {locationsOpen ? (
        <LocationManagerDialog
          catalog={catalog}
          onClose={() => setLocationsOpen(false)}
          onCreateArea={async (input: StorageAreaInput) => {
            const result = await requireLabLocationBridge().createLabStorageArea(input);
            await refreshAfterMutation(result.message);
          }}
          onCreateContainer={async (input: StorageContainerInput) => {
            const result = await requireLabLocationBridge().createLabStorageContainer(input);
            await refreshAfterMutation(result.message);
          }}
          onDeleteArea={async (area: StorageArea) => {
            const result = await requireLabLocationBridge().deleteLabStorageArea(area.areaUuid);
            await refreshAfterMutation(result.message);
          }}
          onDeleteContainer={async (container: StorageContainer) => {
            const result = await requireLabLocationBridge().deleteLabStorageContainer(container.containerUuid);
            await refreshAfterMutation(result.message);
          }}
          onUpdateArea={async (area: StorageArea, input: StorageAreaInput) => {
            const result = await requireLabLocationBridge().updateLabStorageArea(area.areaUuid, input);
            await refreshAfterMutation(result.message);
          }}
          onUpdateContainer={async (container: StorageContainer, input: StorageContainerInput) => {
            const result = await requireLabLocationBridge().updateLabStorageContainer(container.containerUuid, input);
            await refreshAfterMutation(result.message);
          }}
          readOnly={!canModify}
        />
      ) : null}
      {orderDialogOpen ? (
        <OrderExportDialog
          catalog={catalog}
          drafts={orderDrafts}
          selectedPartIds={selectedPartIds}
          onClose={() => setOrderDialogOpen(false)}
          onDraftChange={(partUuid, patch) => {
            setOrderDrafts((current) => {
              const existing = current[partUuid] ?? {
                partUuid,
                requestedQuantity: "",
                note: "",
              };
              return {
                ...current,
                [partUuid]: { ...existing, ...patch, partUuid },
              };
            });
          }}
          onExport={exportOrderRequest}
        />
      ) : null}
    </>
  );
}

function mergedPlacementInput(
  existing: StockPlacement,
  incoming: StockPlacementInput,
): StockPlacementInput {
  return {
    ...incoming,
    quantity: existing.quantity + incoming.quantity,
    packaging: incoming.packaging || existing.packaging,
    countState: existing.countState,
    lastCountedAt: existing.lastCountedAt,
    lastCountedBy: existing.lastCountedBy,
    notes: incoming.notes || existing.notes,
    archived: false,
  };
}

function containerOptionLabel(
  container: StorageContainer,
  lookups: ReturnType<typeof createCatalogLookups>,
): string {
  const area = lookups.areasById.get(container.areaUuid);
  return `${area?.name ?? "Unknown area"} / ${container.name}`;
}
