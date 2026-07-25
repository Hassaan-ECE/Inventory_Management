import { useEffect, useMemo, useState } from "react";
import { SlidersHorizontalIcon, XIcon } from "lucide-react";

import type { InventoryDesktopBridge } from "@/integrations/tauri/desktop-bridge";
import { CatalogHeader } from "@/modules/te-lab-components/catalog/CatalogHeader";
import {
  CATALOG_COLUMNS,
  type CatalogColumnKey,
} from "@/modules/te-lab-components/catalog/catalogColumns";
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
import { useStatusAnnouncer } from "@/modules/te-lab-components/components/shell/useStatusAnnouncer";
import type {
  CatalogMigrationPreview,
  CatalogScope,
  CatalogSharedCutoverPreview,
  Part,
  PartInput,
  StockCountInput,
  StockMoveInput,
  StockPlacement,
  StockPlacementInput,
  StockStatus,
  StorageArea,
  StorageAreaInput,
  StorageContainer,
  StorageContainerInput,
} from "@/modules/te-lab-components/types";
import type { DesktopModuleViewProps } from "@/platform/modules/types";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";

const COLUMN_VISIBILITY_KEY = "teLabComponents.catalog.v2.columnVisibility";
const FILTER_PREFERENCES_KEY = "teLabComponents.catalog.v2.filters";
const SELECT_CLASS = "h-9 rounded-lg border border-input bg-background px-3 text-sm outline-none focus:border-ring focus:ring-[3px] focus:ring-ring/18 dark:bg-input/30";

type CatalogSortKey =
  | "category"
  | "manufacturerPartNumber"
  | "displayValue"
  | "manufacturer"
  | "stockStatus"
  | "updatedAt";

interface CatalogFilters {
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
  sortBy: CatalogSortKey;
  stockStatus: "" | StockStatus;
  subcategory: string;
}

const EMPTY_CATALOG_FILTERS: CatalogFilters = {
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
  sortBy: "category",
  stockStatus: "",
  subcategory: "",
};

const CATALOG_SORT_KEYS: CatalogSortKey[] = [
  "category",
  "manufacturerPartNumber",
  "displayValue",
  "manufacturer",
  "stockStatus",
  "updatedAt",
];

interface PlacementDialogState {
  partUuid: string;
  placementUuid?: string;
}

type LabCatalogBridgeMethod =
  | "createLabPart"
  | "updateLabPart"
  | "deleteLabPart"
  | "createLabStorageArea"
  | "updateLabStorageArea"
  | "deleteLabStorageArea"
  | "createLabStorageContainer"
  | "updateLabStorageContainer"
  | "deleteLabStorageContainer"
  | "createLabStockPlacement"
  | "updateLabStockPlacement"
  | "deleteLabStockPlacement"
  | "moveLabStock"
  | "countLabStock"
  | "previewLabCatalogMigration"
  | "commitLabCatalogMigration"
  | "previewLabSharedCutover"
  | "commitLabSharedCutover";

type LabCatalogBridge = InventoryDesktopBridge & Required<Pick<InventoryDesktopBridge, LabCatalogBridgeMethod>>;

const LAB_CATALOG_BRIDGE_METHODS: LabCatalogBridgeMethod[] = [
  "createLabPart",
  "updateLabPart",
  "deleteLabPart",
  "createLabStorageArea",
  "updateLabStorageArea",
  "deleteLabStorageArea",
  "createLabStorageContainer",
  "updateLabStorageContainer",
  "deleteLabStorageContainer",
  "createLabStockPlacement",
  "updateLabStockPlacement",
  "deleteLabStockPlacement",
  "moveLabStock",
  "countLabStock",
  "previewLabCatalogMigration",
  "commitLabCatalogMigration",
  "previewLabSharedCutover",
  "commitLabSharedCutover",
];

export function TeLabComponentsView({
  active,
  activeModuleId,
  onModuleChange,
  onThemeToggle,
  theme,
}: DesktopModuleViewProps) {
  const { announceStatus, statusOverride } = useStatusAnnouncer();
  const { catalog, dataSource, isLoading, lastError, refreshAfterMutation, refreshCatalog } = useLabCatalog({ active, announceStatus });
  const [scope, setScope] = useState<CatalogScope>("inventory");
  const [query, setQuery] = useState("");
  const [filters, setFilters] = useState<CatalogFilters>(readFilterPreferences);
  const [columnVisibility, setColumnVisibility] = useState<Record<CatalogColumnKey, boolean>>(readColumnVisibility);
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
  const advancedFilterCount = [
    filters.containerUuid,
    filters.coordinate,
    filters.displayValue,
    filters.manufacturer,
    filters.manufacturerPartNumber,
    filters.mountingType,
    filters.packageType,
    filters.partStatus,
    filters.subcategory,
  ].filter(Boolean).length;
  const hasCatalogFilters = Boolean(
    query ||
      filters.areaUuid ||
      filters.category ||
      filters.containerUuid ||
      filters.coordinate ||
      filters.displayValue ||
      filters.manufacturer ||
      filters.manufacturerPartNumber ||
      filters.mountingType ||
      filters.packageType ||
      filters.partStatus ||
      filters.stockStatus ||
      filters.subcategory,
  );
  const canModify = !catalog.migration.required && (dataSource !== "desktop" || catalog.shared.canModify);
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

  useEffect(() => {
    localStorage.setItem(COLUMN_VISIBILITY_KEY, JSON.stringify(columnVisibility));
  }, [columnVisibility]);

  useEffect(() => {
    localStorage.setItem(FILTER_PREFERENCES_KEY, JSON.stringify(filters));
  }, [filters]);

  const displayParts = useMemo(() => {
    const normalizedQuery = query.trim().toLocaleLowerCase();
    return catalog.parts
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
      .filter((part) => !normalizedQuery || searchablePartText(part, partPlacements(catalog, part.entryUuid), lookups).includes(normalizedQuery))
      .sort((left, right) => compareCatalogParts(left, right, filters.sortBy, lookups));
  }, [catalog, filters, lookups, query, scope]);

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

  async function savePart(part: Part | null, input: PartInput): Promise<void> {
    const bridge = requireBridge();
    const result = part
      ? await bridge.updateLabPart(part.entryUuid, input)
      : await bridge.createLabPart(input);
    await refreshAfterMutation(result.message);
    setPartDialogId(null);
  }

  async function deletePart(part: Part): Promise<void> {
    if (!window.confirm(`Delete ${part.manufacturerPartNumber || part.displayValue || part.id}? Parts with placements must be archived instead.`)) return;
    const result = await requireBridge().deleteLabPart(part.entryUuid);
    await refreshAfterMutation(result.message);
    setPartDialogId(null);
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

  async function exportWorkbook(): Promise<void> {
    const bridge = window.inventoryDesktop;
    if (!bridge?.exportExcel) {
      announceStatus("Excel export is available in the desktop app.");
      return;
    }
    const result = await bridge.exportExcel("te-lab-components");
    announceStatus(result.canceled ? "Export canceled." : result.error || `Exported Lab catalog to ${result.outputPath ?? "the selected workbook"}.`);
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
        activeModuleId={activeModuleId}
        canModify={canModify}
        counts={catalog.counts}
        onAddPart={() => setPartDialogId("new")}
        onExport={() => void exportWorkbook()}
        onLocations={() => setLocationsOpen(true)}
        onModuleChange={onModuleChange}
        onScopeChange={setScope}
        onSharedSetup={() => setSharedCutoverOpen(true)}
        onThemeToggle={onThemeToggle}
        scope={scope}
        shared={catalog.shared}
        theme={theme}
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
          <section className="shrink-0 rounded-xl border border-border bg-card/75 p-2.5 shadow-sm">
            <div className="flex flex-wrap items-center gap-2">
              <Input aria-label="Search Lab components" className="min-w-56 flex-1" placeholder="Search MPN, value, attributes, supplier, or location…" value={query} onChange={(event) => setQuery(event.target.value)} />
              <label className="text-xs font-medium"><span className="sr-only">Filter category</span><select aria-label="Filter category" className={SELECT_CLASS} value={filters.category} onChange={(event) => setFilters((current) => ({ ...current, category: event.target.value }))}><option value="">All categories</option>{filterOptions.categories.map((category) => <option key={category} value={category}>{category}</option>)}</select></label>
              <label className="text-xs font-medium"><span className="sr-only">Filter stock status</span><select aria-label="Filter stock status" className={SELECT_CLASS} value={filters.stockStatus} onChange={(event) => setFilters((current) => ({ ...current, stockStatus: event.target.value as CatalogFilters["stockStatus"] }))}><option value="">All stock states</option>{["in_stock", "low_stock", "no_stock", "unit_review", "mixed_units", "archived"].map((status) => <option key={status} value={status}>{status.replaceAll("_", " ")}</option>)}</select></label>
              <label className="text-xs font-medium"><span className="sr-only">Filter storage area</span><select aria-label="Filter storage area" className={SELECT_CLASS} value={filters.areaUuid} onChange={(event) => setFilters((current) => ({ ...current, areaUuid: event.target.value, containerUuid: "" }))}><option value="">All areas</option>{catalog.storageAreas.map((area) => <option key={area.areaUuid} value={area.areaUuid}>{area.name}</option>)}</select></label>
              <label className="text-xs font-medium"><span className="sr-only">Sort catalog</span><select aria-label="Sort catalog" className={SELECT_CLASS} value={filters.sortBy} onChange={(event) => setFilters((current) => ({ ...current, sortBy: event.target.value as CatalogSortKey }))}><option value="category">Sort: Category</option><option value="manufacturerPartNumber">Sort: MPN</option><option value="displayValue">Sort: Value</option><option value="manufacturer">Sort: Manufacturer</option><option value="stockStatus">Sort: Stock status</option><option value="updatedAt">Sort: Recently updated</option></select></label>
              <details className="relative">
                <summary className="flex h-9 cursor-pointer list-none items-center gap-2 rounded-lg border border-input bg-background px-3 text-sm font-medium hover:bg-accent/50"><SlidersHorizontalIcon className="size-3.5" /> Columns</summary>
                <div className="absolute right-0 z-40 mt-2 grid w-64 gap-1 rounded-xl border border-border bg-popover p-2 shadow-xl">
                  {CATALOG_COLUMNS.map((column) => <label className="flex items-center gap-2 rounded-md px-2 py-1.5 text-xs hover:bg-accent" key={column.key}><input checked={columnVisibility[column.key]} onChange={(event) => setColumnVisibility((current) => ({ ...current, [column.key]: event.target.checked }))} type="checkbox" /> {column.label}</label>)}
                </div>
              </details>
              {hasCatalogFilters ? <Button aria-label="Clear catalog filters" onClick={() => { setQuery(""); setFilters((current) => ({ ...EMPTY_CATALOG_FILTERS, sortBy: current.sortBy })); }} size="sm" variant="ghost"><XIcon className="size-3.5" /> Clear</Button> : null}
            </div>
            <details className="mt-2 rounded-lg border border-border/70 bg-muted/20 px-2.5 py-1.5">
              <summary className="cursor-pointer text-xs font-semibold text-muted-foreground">
                More Filters{advancedFilterCount ? ` (${advancedFilterCount})` : ""}
              </summary>
              <div className="mt-2 grid gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
                <FilterSelect label="Filter subcategory" value={filters.subcategory} onChange={(value) => setFilters((current) => ({ ...current, subcategory: value }))} options={filterOptions.subcategories} placeholder="All subcategories" />
                <FilterSelect label="Filter manufacturer" value={filters.manufacturer} onChange={(value) => setFilters((current) => ({ ...current, manufacturer: value }))} options={filterOptions.manufacturers} placeholder="All manufacturers" />
                <FilterInput label="Filter manufacturer part number" value={filters.manufacturerPartNumber} onChange={(value) => setFilters((current) => ({ ...current, manufacturerPartNumber: value }))} placeholder="MPN contains…" />
                <FilterInput label="Filter value or label" value={filters.displayValue} onChange={(value) => setFilters((current) => ({ ...current, displayValue: value }))} placeholder="Value contains…" />
                <FilterSelect label="Filter mounting type" value={filters.mountingType} onChange={(value) => setFilters((current) => ({ ...current, mountingType: value }))} options={filterOptions.mountingTypes} placeholder="All mounting types" />
                <FilterSelect label="Filter package type" value={filters.packageType} onChange={(value) => setFilters((current) => ({ ...current, packageType: value }))} options={filterOptions.packageTypes} placeholder="All packages" />
                <FilterSelect label="Filter storage container" value={filters.containerUuid} onChange={(value) => setFilters((current) => ({ ...current, containerUuid: value }))} options={filteredContainerOptions.map((container) => ({ label: containerOptionLabel(container, lookups), value: container.containerUuid }))} placeholder="All containers" />
                <FilterInput label="Filter bin coordinate" value={filters.coordinate} onChange={(value) => setFilters((current) => ({ ...current, coordinate: value }))} placeholder="C7, AA1…" />
                <FilterSelect label="Filter part status" value={filters.partStatus} onChange={(value) => setFilters((current) => ({ ...current, partStatus: value }))} options={filterOptions.partStatuses} placeholder="All part states" />
              </div>
            </details>
          </section>

          <div className="min-h-0 flex-1 overflow-hidden">
            {isLoading ? (
              <section className="flex h-full items-center justify-center rounded-xl border border-border bg-card/70 text-sm text-muted-foreground">Loading Lab Components catalog…</section>
            ) : displayParts.length ? (
              <PartsTable catalog={catalog} onOpenExternal={(url) => void openExternal(url)} onOpenPart={(part) => setPartDialogId(part.entryUuid)} parts={displayParts} visibleColumns={columnVisibility} />
            ) : (
              <section className="flex h-full flex-col items-center justify-center rounded-xl border border-dashed border-border bg-card/50 p-8 text-center"><h2 className="font-semibold">No catalog parts found</h2><p className="mt-1 text-sm text-muted-foreground">Adjust the search and filters, or add a generalized electronic part.</p>{canModify ? <Button className="mt-4" onClick={() => setPartDialogId("new")}>Add Part</Button> : null}</section>
            )}
          </div>
        </div>
      )}

      <footer className="flex shrink-0 flex-wrap items-center gap-x-4 gap-y-1 border-t border-border bg-muted/25 px-3 py-1.5 text-[11px] text-muted-foreground sm:px-5">
        <span>{displayParts.length.toLocaleString()} shown</span>
        <span>{catalog.counts.noStock} no stock</span>
        <span>{catalog.counts.lowStock} low stock</span>
        <span>{catalog.counts.unitReview} unit review</span>
        <span className="ml-auto max-w-[55vw] truncate" role="status">{statusOverride ?? lastError ?? catalog.shared.message}</span>
      </footer>

      {partDialogId ? (
        <PartDialog
          catalog={catalog}
          key={partDialogId}
          onAddPlacement={(part) => setPlacementDialog({ partUuid: part.entryUuid })}
          onClose={() => setPartDialogId(null)}
          onCountPlacement={(placement) => setCountPlacementUuid(placement.placementUuid)}
          onDeletePart={(part) => void deletePart(part)}
          onDeletePlacement={(placement) => void deletePlacement(placement)}
          onEditPlacement={(placement) => setPlacementDialog({ partUuid: placement.partUuid, placementUuid: placement.placementUuid })}
          onMovePlacement={(placement) => setMovePlacementUuid(placement.placementUuid)}
          onSave={(input) => savePart(activePart, input)}
          part={partDialogId === "new" ? null : activePart}
          readOnly={!canModify}
        />
      ) : null}

      {placementDialog && placementPart ? <PlacementDialog catalog={catalog} key={placementDialog.placementUuid ?? `new-${placementDialog.partUuid}`} onClose={() => setPlacementDialog(null)} onSave={savePlacement} part={placementPart} placement={editedPlacement} /> : null}
      {movePlacement ? <MoveStockDialog catalog={catalog} key={movePlacement.placementUuid} onClose={() => setMovePlacementUuid(null)} onMove={moveStock} source={movePlacement} /> : null}
      {countPlacement ? <CountStockDialog catalog={catalog} key={countPlacement.placementUuid} onClose={() => setCountPlacementUuid(null)} onCount={(input) => countStock(countPlacement, input)} placement={countPlacement} /> : null}
      {locationsOpen ? <LocationManagerDialog
        catalog={catalog}
        onClose={() => setLocationsOpen(false)}
        onCreateArea={async (input: StorageAreaInput) => { const result = await requireBridge().createLabStorageArea(input); await refreshAfterMutation(result.message); }}
        onCreateContainer={async (input: StorageContainerInput) => { const result = await requireBridge().createLabStorageContainer(input); await refreshAfterMutation(result.message); }}
        onDeleteArea={async (area: StorageArea) => { const result = await requireBridge().deleteLabStorageArea(area.areaUuid); await refreshAfterMutation(result.message); }}
        onDeleteContainer={async (container: StorageContainer) => { const result = await requireBridge().deleteLabStorageContainer(container.containerUuid); await refreshAfterMutation(result.message); }}
        onUpdateArea={async (area: StorageArea, input: StorageAreaInput) => { const result = await requireBridge().updateLabStorageArea(area.areaUuid, input); await refreshAfterMutation(result.message); }}
        onUpdateContainer={async (container: StorageContainer, input: StorageContainerInput) => { const result = await requireBridge().updateLabStorageContainer(container.containerUuid, input); await refreshAfterMutation(result.message); }}
        readOnly={!canModify}
      /> : null}
      {sharedCutoverOpen ? <SharedCutoverDialog
        onClose={() => setSharedCutoverOpen(false)}
        onCommit={async (preview: CatalogSharedCutoverPreview) => { const result = await requireBridge().commitLabSharedCutover({ localFingerprint: preview.localFingerprint, confirmed: true }); await refreshCatalog(); return result; }}
        onPreview={() => requireBridge().previewLabSharedCutover()}
      /> : null}
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

function readFilterPreferences(): CatalogFilters {
  try {
    const saved = JSON.parse(localStorage.getItem(FILTER_PREFERENCES_KEY) ?? "{}") as Partial<CatalogFilters>;
    return {
      ...EMPTY_CATALOG_FILTERS,
      ...saved,
      sortBy: CATALOG_SORT_KEYS.includes(saved.sortBy as CatalogSortKey)
        ? (saved.sortBy as CatalogSortKey)
        : EMPTY_CATALOG_FILTERS.sortBy,
    };
  } catch {
    return { ...EMPTY_CATALOG_FILTERS };
  }
}

function compareCatalogParts(
  left: Part,
  right: Part,
  sortBy: CatalogSortKey,
  lookups: ReturnType<typeof createCatalogLookups>,
): number {
  const comparison = (() => {
    switch (sortBy) {
      case "manufacturerPartNumber":
        return left.manufacturerPartNumber.localeCompare(right.manufacturerPartNumber);
      case "displayValue":
        return left.displayValue.localeCompare(right.displayValue);
      case "manufacturer":
        return left.manufacturer.localeCompare(right.manufacturer);
      case "stockStatus":
        return (lookups.summariesByPartId.get(left.entryUuid)?.stockStatus ?? "no_stock").localeCompare(
          lookups.summariesByPartId.get(right.entryUuid)?.stockStatus ?? "no_stock",
        );
      case "updatedAt":
        return right.updatedAt.localeCompare(left.updatedAt);
      case "category":
        return left.category.localeCompare(right.category);
    }
  })();
  return (
    comparison ||
    left.category.localeCompare(right.category) ||
    left.manufacturerPartNumber.localeCompare(right.manufacturerPartNumber) ||
    left.displayValue.localeCompare(right.displayValue) ||
    left.entryUuid.localeCompare(right.entryUuid)
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

function FilterInput({
  label,
  onChange,
  placeholder,
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  placeholder: string;
  value: string;
}) {
  return (
    <Input
      aria-label={label}
      className="h-8 text-xs"
      onChange={(event) => onChange(event.target.value)}
      placeholder={placeholder}
      value={value}
    />
  );
}

function FilterSelect({
  label,
  onChange,
  options,
  placeholder,
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  options: string[] | Array<{ label: string; value: string }>;
  placeholder: string;
  value: string;
}) {
  return (
    <select
      aria-label={label}
      className={`${SELECT_CLASS} h-8 text-xs`}
      onChange={(event) => onChange(event.target.value)}
      value={value}
    >
      <option value="">{placeholder}</option>
      {options.map((option) => {
        const value = typeof option === "string" ? option : option.value;
        const optionLabel = typeof option === "string" ? option.replaceAll("_", " ") : option.label;
        return <option key={value} value={value}>{optionLabel}</option>;
      })}
    </select>
  );
}
