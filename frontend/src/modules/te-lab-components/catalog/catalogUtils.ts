import type {
  CatalogSyncResult,
  Part,
  PartStockSummary,
  StockPlacement,
  StorageArea,
  StorageContainer,
} from "@/modules/te-lab-components/types";

export interface CatalogLookups {
  areasById: Map<string, StorageArea>;
  containersById: Map<string, StorageContainer>;
  partsById: Map<string, Part>;
  summariesByPartId: Map<string, PartStockSummary>;
}

export function createCatalogLookups(catalog: CatalogSyncResult): CatalogLookups {
  return {
    areasById: new Map(catalog.storageAreas.map((area) => [area.areaUuid, area])),
    containersById: new Map(
      catalog.storageContainers.map((container) => [container.containerUuid, container]),
    ),
    partsById: new Map(catalog.parts.map((part) => [part.entryUuid, part])),
    summariesByPartId: new Map(catalog.summaries.map((summary) => [summary.partUuid, summary])),
  };
}

export function columnLabel(columnIndex: number): string {
  let current = Math.max(0, Math.trunc(columnIndex));
  let label = "";
  while (true) {
    label = String.fromCharCode(65 + (current % 26)) + label;
    if (current < 26) {
      return label;
    }
    current = Math.floor(current / 26) - 1;
  }
}

export function gridCoordinateLabel(
  container: Pick<StorageContainer, "rowStart">,
  rowIndex: number,
  columnIndex: number,
): string {
  return `${columnLabel(columnIndex)}${container.rowStart + rowIndex}`;
}

export function placementPosition(placement: StockPlacement, container?: StorageContainer): string {
  if (
    container?.gridEnabled &&
    placement.rowIndex !== null &&
    placement.columnIndex !== null
  ) {
    return gridCoordinateLabel(container, placement.rowIndex, placement.columnIndex);
  }
  return placement.freeformPosition || "Unassigned";
}

export function placementPath(placement: StockPlacement, lookups: CatalogLookups): string {
  const container = lookups.containersById.get(placement.containerUuid);
  const area = container ? lookups.areasById.get(container.areaUuid) : undefined;
  return [area?.name || "Unknown area", container?.name || "Unknown container", placementPosition(placement, container)]
    .filter(Boolean)
    .join(" / ");
}

export function formatQuantity(quantity: number): string {
  return Number.isInteger(quantity)
    ? quantity.toLocaleString()
    : quantity.toLocaleString(undefined, { maximumFractionDigits: 4 });
}

export function formatTotals(summary?: PartStockSummary): string {
  if (!summary || summary.totals.length === 0) {
    return "0";
  }
  return summary.totals
    .map((total) => `${formatQuantity(total.quantity)} ${total.unitOfMeasure}`)
    .join(" + ");
}

export function partPlacements(catalog: CatalogSyncResult, partUuid: string): StockPlacement[] {
  return catalog.stockPlacements.filter(
    (placement) => placement.partUuid === partUuid && !placement.archived,
  );
}

export function indistinguishablePlacementKey(
  placement: Pick<
    StockPlacement,
    | "partUuid"
    | "containerUuid"
    | "columnIndex"
    | "rowIndex"
    | "freeformPosition"
    | "lotCode"
    | "dateCode"
    | "condition"
    | "unitOfMeasure"
  >,
): string {
  return [
    placement.partUuid,
    placement.containerUuid,
    placement.columnIndex ?? "",
    placement.rowIndex ?? "",
    normalizePlacementText(placement.freeformPosition),
    normalizePlacementText(placement.lotCode),
    normalizePlacementText(placement.dateCode),
    normalizePlacementText(placement.condition),
    normalizePlacementText(placement.unitOfMeasure),
  ].join("|");
}

export function searchablePartText(
  part: Part,
  placements: StockPlacement[],
  lookups: CatalogLookups,
): string {
  const attributes = Object.entries(part.attributes).flatMap(([key, attribute]) => [
    key,
    attribute.value,
    attribute.unit,
  ]);
  const locations = placements.flatMap((placement) => [
    placementPath(placement, lookups),
    placement.freeformPosition,
    placement.lotCode,
    placement.dateCode,
    placement.notes,
  ]);
  return [
    part.internalPartNumber,
    part.category,
    part.subcategory,
    part.manufacturer,
    part.manufacturerPartNumber,
    part.displayValue,
    part.mountingType,
    part.packageType,
    part.description,
    part.supplier,
    part.supplierSku,
    part.supplierPackaging,
    part.notes,
    ...attributes,
    ...locations,
  ]
    .join(" ")
    .toLocaleLowerCase();
}

export function partDisplayName(part: Part): string {
  return (
    part.manufacturerPartNumber ||
    part.internalPartNumber ||
    part.displayValue ||
    part.description ||
    `Part ${part.id}`
  );
}

function normalizePlacementText(value: string): string {
  return value.trim().replaceAll(/\s+/g, " ").toLocaleLowerCase();
}
