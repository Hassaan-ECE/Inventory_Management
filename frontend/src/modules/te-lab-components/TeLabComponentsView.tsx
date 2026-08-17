import { useEffect, useMemo, useState } from "react";

import type { InventoryDesktopBridge } from "@/integrations/tauri/desktop-bridge";
import { CatalogHeader } from "@/modules/te-lab-components/catalog/CatalogHeader";
import {
  CATALOG_COLUMNS,
  type CatalogColumnKey,
} from "@/modules/te-lab-components/catalog/catalogColumns";
import { CatalogSearchCard } from "@/modules/te-lab-components/catalog/CatalogSearchCard";
import {
  EMPTY_CATALOG_FILTERS,
  type CatalogFilters,
} from "@/modules/te-lab-components/catalog/catalogFilters";
import {
  cycleCatalogSortState,
  sortCatalogParts,
  type CatalogSortState,
} from "@/modules/te-lab-components/catalog/catalogSorting";
import {
  createCatalogLookups,
  partPlacements,
  placementPosition,
  searchablePartText,
} from "@/modules/te-lab-components/catalog/catalogUtils";
import { CATEGORY_TEMPLATES } from "@/modules/te-lab-components/catalog/categoryTemplates";
import { LocationManagerDialog } from "@/modules/te-lab-components/catalog/LocationManagerDialog";
import { MigrationPanel } from "@/modules/te-lab-components/catalog/MigrationPanel";
import { PartDialog } from "@/modules/te-lab-components/catalog/PartDialog";
import {
  CountStockDialog,
  MoveStockDialog,
  PlacementDialog,
} from "@/modules/te-lab-components/catalog/PlacementDialogs";
import { PartsTable } from "@/modules/te-lab-components/catalog/PartsTable";
import { SharedCutoverDialog } from "@/modules/te-lab-components/catalog/SharedCutoverDialog";
import { useLabCatalog } from "@/modules/te-lab-components/catalog/useLabCatalog";
import { useDesktopUpdates } from "@/modules/te-lab-components/components/shell/useDesktopUpdates";
import { useStatusAnnouncer } from "@/modules/te-lab-components/components/shell/useStatusAnnouncer";
import { StatusStrip } from "@/modules/te-lab-components/components/StatusStrip";
import type {
  CatalogMigrationPreview,
  CatalogScope,
  CatalogSharedCutoverPreview,
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

// v5: five-column simple workspace defaults (Component Type, Value, Quantity, Location).
const COLUMN_VISIBILITY_KEY = "teLabComponents.catalog.v5.columnVisibility";
const COLOR_ROWS_KEY = "teLabComponents.catalog.v1.colorRows";
const FILTER_PREFERENCES_KEY = "teLabComponents.catalog.v2.filters";
const SORT_PREFERENCES_KEY = "teLabComponents.catalog.v2.sort";
const DEFAULT_SORT_STATE: CatalogSortState = { column: "subcategory", direction: "asc" };

interface PlacementDialogState {
  partUuid: string;
  placementUuid?: string;
}

type LabCatalogBridgeMethod =
  | "createLabPart"
  | "updateLabPart"
  | "createLabSimpleComponent"
  | "updateLabSimpleComponent"
  | "deleteLabPart"
  | "createLabStockPlacement"
  | "updateLabStockPlacement"
  | "deleteLabStockPlacement"
  | "moveLabStock"
  | "countLabStock"
  | "previewLabCatalogMigration"
  | "commitLabCatalogMigration";

type LabCatalogBridge = InventoryDesktopBridge & Required<Pick<InventoryDesktopBridge, LabCatalogBridgeMethod>>;

type LabLocationBridgeMethod =
  | "createLabStorageArea"
  | "updateLabStorageArea"
  | "deleteLabStorageArea"
  | "createLabStorageContainer"
  | "updateLabStorageContainer"
  | "deleteLabStorageContainer";

type LabLocationBridge = InventoryDesktopBridge & Required<Pick<InventoryDesktopBridge, LabLocationBridgeMethod>>;

const LAB_LOCATION_BRIDGE_METHODS: LabLocationBridgeMethod[] = [
  "createLabStorageArea",
  "updateLabStorageArea",
  "deleteLabStorageArea",
  "createLabStorageContainer",
  "updateLabStorageContainer",
  "deleteLabStorageContainer",
];

type LabSharedCutoverBridgeMethod = "previewLabSharedCutover" | "commitLabSharedCutover";

type LabSharedCutoverBridge = InventoryDesktopBridge & Required<Pick<InventoryDesktopBridge, LabSharedCutoverBridgeMethod>>;

const LAB_SHARED_CUTOVER_BRIDGE_METHODS: LabSharedCutoverBridgeMethod[] = [
  "previewLabSharedCutover",
  "commitLabSharedCutover",
];

const LAB_CATALOG_BRIDGE_METHODS: LabCatalogBridgeMethod[] = [
  "createLabPart",
  "updateLabPart",
  "createLabSimpleComponent",
  "updateLabSimpleComponent",
  "deleteLabPart",
  "createLabStockPlacement",
  "updateLabStockPlacement",
  "deleteLabStockPlacement",
  "moveLabStock",
  "countLabStock",
  "previewLabCatalogMigration",
  "commitLabCatalogMigration",
];

export function TeLabComponentsView({
  active,
  activeViewId,
  onThemeToggle,
  onViewChange,
  theme,
}: DesktopModuleViewProps) {
  const { announceStatus, statusOverride } = useStatusAnnouncer();
  const { handleUpdateAction, updateState } = useDesktopUpdates({ active, announceStatus });
  const { catalog, dataSource, isLoading, lastError, refreshAfterMutation, refreshCatalog } = useLabCatalog({ active, announceStatus });
  const [scope, setScope] = useState<CatalogScope>("inventory");
  const [query, setQuery] = useState("");
  const [filters, setFilters] = useState<CatalogFilters>(readFilterPreferences);
  const [filtersOpen, setFiltersOpen] = useState(false);
  const [sortState, setSortState] = useState<CatalogSortState | null>(readSortPreferences);
  const [columnVisibility, setColumnVisibility] = useState<Record<CatalogColumnKey, boolean>>(readColumnVisibility);
  const [colorRows, setColorRows] = useState<boolean>(readColorRows);
  const [partDialogId, setPartDialogId] = useState<string | "new" | null>(null);
  const [placementDialog, setPlacementDialog] = useState<PlacementDialogState | null>(null);
  const [movePlacementUuid, setMovePlacementUuid] = useState<string | null>(null);
  const [countPlacementUuid, setCountPlacementUuid] = useState<string | null>(null);
  const [locationsOpen, setLocationsOpen] = useState(false);
  const [sharedCutoverOpen, setSharedCutoverOpen] = useState(false);
  const lookups = useMemo(() => createCatalogLookups(catalog), [catalog]);
  const filterOptions = useMemo(
    () => ({
      categories: uniqueOptions([
        ...CATEGORY_TEMPLATES.map((template) => template.category),
        ...catalog.parts.map((part) => part.category),
      ]),
      manufacturers: uniqueOptions(catalog.parts.map((part) => part.manufacturer)),
      mountingTypes: uniqueOptions(catalog.parts.map((part) => part.mountingType)),
      packageTypes: uniqueOptions(catalog.parts.map((part) => part.packageType)),
      partStatuses: uniqueOptions(catalog.parts.map((part) => part.partStatus)),
      subcategories: uniqueOptions(catalog.parts.map((part) => part.subcategory)),
    }),
    [catalog.parts],
  );
  const filteredContainerOptions = catalog.storageContainers.filter(
    (container) => !filters.areaUuid || container.areaUuid === filters.areaUuid,
  );
  const activeFilterCount = [
    filters.areaUuid,
    filters.category,
    filters.containerUuid,
    filters.coordinate,
    filters.displayValue,
    filters.manufacturer,
    filters.manufacturerPartNumber,
    filters.mountingType,
    filters.packageType,
    filters.partStatus,
    filters.stockStatus,
    filters.subcategory,
  ].filter(Boolean).length;
  const hasCatalogFilters = Boolean(query || activeFilterCount > 0);
  const canModify = !catalog.migration.required && (dataSource !== "desktop" || catalog.shared.canModify);
  const desktopBridge = window.inventoryDesktop;
  const locationsAvailable = Boolean(
    dataSource === "desktop"
      && desktopBridge?.isDesktop
      && LAB_LOCATION_BRIDGE_METHODS.every((method) => typeof desktopBridge[method] === "function"),
  );
  const sharedCutoverAvailable = Boolean(
    dataSource === "desktop"
      && desktopBridge?.isDesktop
      && catalog.shared.enabled
      && !catalog.shared.available
      && LAB_SHARED_CUTOVER_BRIDGE_METHODS.every((method) => typeof desktopBridge[method] === "function"),
  );
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

  // Collapse filters when leaving Lab Components (switcher / other modules).
  // Adjust during render when `active` flips — avoids setState-in-effect lint.
  const [filtersOpenActive, setFiltersOpenActive] = useState(active);
  if (filtersOpenActive !== active) {
    setFiltersOpenActive(active);
    if (!active && filtersOpen) {
      setFiltersOpen(false);
    }
  }

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

  const displayParts = useMemo(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase();
    const filtered = catalog.parts
      .filter((part) => part.archived === (scope === "archive"))
      .filter((part) => !filters.category || part.category === filters.category)
      .filter((part) => !filters.subcategory || part.subcategory === filters.subcategory)
      .filter((part) => !filters.manufacturer || part.manufacturer === filters.manufacturer)
      .filter(
        (part) =>
          !filters.manufacturerPartNumber ||
          part.manufacturerPartNumber
            .toLocaleLowerCase()
            .includes(filters.manufacturerPartNumber.trim().toLocaleLowerCase()),
      )
      .filter(
        (part) =>
          !filters.displayValue ||
          part.displayValue.toLocaleLowerCase().includes(filters.displayValue.trim().toLocaleLowerCase()),
      )
      .filter((part) => !filters.mountingType || part.mountingType === filters.mountingType)
      .filter((part) => !filters.packageType || part.packageType === filters.packageType)
      .filter((part) => !filters.partStatus || part.partStatus === filters.partStatus)
      .filter((part) => {
        const summary = lookups.summariesByPartId.get(part.entryUuid);
        return !filters.stockStatus || summary?.stockStatus === filters.stockStatus;
      })
      .filter((part) => {
        if (!filters.areaUuid && !filters.containerUuid && !filters.coordinate) return true;
        const normalizedCoordinate = filters.coordinate.trim().toLocaleLowerCase();
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
    return sortCatalogParts(filtered, sortState, lookups);
  }, [catalog, filters, lookups, query, scope, sortState]);

  function requireBridge(): LabCatalogBridge {
    const bridge = window.inventoryDesktop;
    if (
      !bridge?.isDesktop ||
      LAB_CATALOG_BRIDGE_METHODS.some((method) => typeof bridge[method] !== "function")
    ) {
      throw new Error("This desktop build does not expose the Lab catalog-v2 commands.");
    }
    return bridge as LabCatalogBridge;
  }

  function requireLocationBridge(): LabLocationBridge {
    const bridge = window.inventoryDesktop;
    if (
      !bridge?.isDesktop
      || LAB_LOCATION_BRIDGE_METHODS.some((method) => typeof bridge[method] !== "function")
    ) {
      throw new Error("This desktop build does not expose the Lab storage-location commands.");
    }
    return bridge as LabLocationBridge;
  }

  function requireSharedCutoverBridge(): LabSharedCutoverBridge {
    const bridge = window.inventoryDesktop;
    if (
      !bridge?.isDesktop
      || LAB_SHARED_CUTOVER_BRIDGE_METHODS.some((method) => typeof bridge[method] !== "function")
    ) {
      throw new Error("This desktop build does not expose the Lab shared-cutover commands.");
    }
    return bridge as LabSharedCutoverBridge;
  }

  async function savePart(part: Part | null, input: PartInput): Promise<void> {
    const bridge = requireBridge();
    const result = part
      ? await bridge.updateLabPart(part.entryUuid, input)
      : await bridge.createLabPart(input);
    await refreshAfterMutation(result.message);
    setPartDialogId(null);
  }

  async function saveSimplePart(part: Part | null, input: SimpleComponentInput): Promise<void> {
    const bridge = requireBridge();
    const result = part
      ? await bridge.updateLabSimpleComponent(part.entryUuid, input)
      : await bridge.createLabSimpleComponent(input);
    await refreshAfterMutation(result.message);
    setPartDialogId(null);
  }

  async function deletePart(part: Part): Promise<void> {
    if (!part.archived) {
      announceStatus("Archive the part before deleting it permanently.");
      throw new Error("Archive the part before deleting it permanently.");
    }
    try {
      const result = await requireBridge().deleteLabPart(part.entryUuid);
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
    const bridge = requireBridge();
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
    const result = await requireBridge().deleteLabStockPlacement(placement.placementUuid);
    await refreshAfterMutation(result.message);
  }

  async function moveStock(input: StockMoveInput): Promise<void> {
    const result = await requireBridge().moveLabStock(input);
    await refreshAfterMutation(result.message);
    setMovePlacementUuid(null);
  }

  async function countStock(placement: StockPlacement, input: StockCountInput): Promise<void> {
    const result = await requireBridge().countLabStock(placement.placementUuid, input);
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

  function exportHtml(): void {
    announceStatus("HTML export is not implemented yet.");
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
        onExportHtml={exportHtml}
        onManageLocations={locationsAvailable ? () => setLocationsOpen(true) : undefined}
        onOpenSharedCutover={sharedCutoverAvailable ? () => setSharedCutoverOpen(true) : undefined}
        onScopeChange={setScope}
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

      {catalog.migration.required ? (
        <div className="min-h-0 flex-1 overflow-hidden">
          <MigrationPanel
            migration={catalog.migration}
            onCommit={async (preview: CatalogMigrationPreview) => {
              const result = await requireBridge().commitLabCatalogMigration({ sourceFingerprint: preview.sourceFingerprint, confirmed: true });
              await refreshAfterMutation(result.message);
              return result;
            }}
            onPreview={() => requireBridge().previewLabCatalogMigration()}
          />
        </div>
      ) : (
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
                parts={displayParts}
                sortState={sortState}
                visibleColumns={columnVisibility}
              />
            ) : (
              <section className="flex h-full flex-col items-center justify-center rounded-xl border border-dashed border-border bg-card/50 p-8 text-center"><h2 className="font-semibold">No catalog parts found</h2><p className="mt-1 text-sm text-muted-foreground">Adjust the search and filters, or add a component.</p>{canModify ? <Button className="mt-4" onClick={() => setPartDialogId("new")}>Add Part</Button> : null}</section>
            )}
          </div>
        </div>
      )}

      <StatusStrip
        counts={{ lowStock: catalog.counts.lowStock, noStock: catalog.counts.noStock }}
        message={statusOverride ?? lastError ?? catalog.shared.message}
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
          onMovePlacement={(placement) => setMovePlacementUuid(placement.placementUuid)}
          onSaveAdvanced={(input) => savePart(activePart, input)}
          onSaveSimple={(input) => saveSimplePart(activePart, input)}
          part={partDialogId === "new" ? null : activePart}
          readOnly={!canModify}
        />
      ) : null}

      {placementDialog && placementPart ? <PlacementDialog catalog={catalog} key={placementDialog.placementUuid ?? `new-${placementDialog.partUuid}`} onClose={() => setPlacementDialog(null)} onSave={savePlacement} part={placementPart} placement={editedPlacement} /> : null}
      {movePlacement ? <MoveStockDialog catalog={catalog} key={movePlacement.placementUuid} onClose={() => setMovePlacementUuid(null)} onMove={moveStock} source={movePlacement} /> : null}
      {countPlacement ? <CountStockDialog catalog={catalog} key={countPlacement.placementUuid} onClose={() => setCountPlacementUuid(null)} onCount={(input) => countStock(countPlacement, input)} placement={countPlacement} /> : null}
      {locationsOpen ? (
        <LocationManagerDialog
          catalog={catalog}
          onClose={() => setLocationsOpen(false)}
          onCreateArea={async (input: StorageAreaInput) => {
            const result = await requireLocationBridge().createLabStorageArea(input);
            await refreshAfterMutation(result.message);
          }}
          onCreateContainer={async (input: StorageContainerInput) => {
            const result = await requireLocationBridge().createLabStorageContainer(input);
            await refreshAfterMutation(result.message);
          }}
          onDeleteArea={async (area: StorageArea) => {
            const result = await requireLocationBridge().deleteLabStorageArea(area.areaUuid);
            await refreshAfterMutation(result.message);
          }}
          onDeleteContainer={async (container: StorageContainer) => {
            const result = await requireLocationBridge().deleteLabStorageContainer(container.containerUuid);
            await refreshAfterMutation(result.message);
          }}
          onUpdateArea={async (area: StorageArea, input: StorageAreaInput) => {
            const result = await requireLocationBridge().updateLabStorageArea(area.areaUuid, input);
            await refreshAfterMutation(result.message);
          }}
          onUpdateContainer={async (container: StorageContainer, input: StorageContainerInput) => {
            const result = await requireLocationBridge().updateLabStorageContainer(container.containerUuid, input);
            await refreshAfterMutation(result.message);
          }}
          readOnly={!canModify}
        />
      ) : null}
      {sharedCutoverOpen ? (
        <SharedCutoverDialog
          onClose={() => setSharedCutoverOpen(false)}
          onCommit={async (preview: CatalogSharedCutoverPreview) => {
            const result = await requireSharedCutoverBridge().commitLabSharedCutover({
              localFingerprint: preview.localFingerprint,
              confirmed: true,
            });
            await refreshCatalog();
            return result;
          }}
          onPreview={() => requireSharedCutoverBridge().previewLabSharedCutover()}
        />
      ) : null}
    </>
  );
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

function uniqueOptions(values: string[]): string[] {
  return [...new Set(values.map((value) => value.trim()).filter(Boolean))].sort((left, right) =>
    left.localeCompare(right),
  );
}

function containerOptionLabel(
  container: StorageContainer,
  lookups: ReturnType<typeof createCatalogLookups>,
): string {
  const area = lookups.areasById.get(container.areaUuid);
  return `${area?.name ?? "Unknown area"} / ${container.name}`;
}


