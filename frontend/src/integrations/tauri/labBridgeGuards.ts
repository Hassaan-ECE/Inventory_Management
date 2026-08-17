import { APP_VERSION } from "@/app/branding";
import type { InventorySyncResult } from "@/integrations/tauri/desktop-bridge";
import {
  LIFECYCLE_OPTIONS,
  WORKING_STATUS_OPTIONS,
  type CatalogCounts,
  type CatalogDeleteResult,
  type CatalogMigrationCommitResult,
  type CatalogMigrationPreview,
  type CatalogMigrationStatus,
  type CatalogMutationResult,
  type CatalogSharedCutoverCommitResult,
  type CatalogSharedCutoverPreview,
  type ComponentAttributes,
  type InventoryCounts,
  type InventoryDeleteMutationResult,
  type InventoryEntry,
  type InventoryEntryMutationResult,
  type InventoryQueryResult,
  type InventorySharedStatus,
  type ExcelExportResult,
  type LifecycleStatus,
  type MigrationDuplicateGroup,
  type Part,
  type PartStockSummary,
  type QuantityTotal,
  type SimpleComponentValue,
  type StockPlacement,
  type StockStatus,
  type StorageArea,
  type StorageContainer,
  type UpdateState,
  type UpdateStatus,
  type WorkingStatus,
} from "@/modules/te-lab-components/types";

const UPDATE_STATUSES = new Set<UpdateStatus>([
  "idle",
  "checking",
  "available",
  "not-available",
  "downloading",
  "ready",
  "installing",
  "error",
]);

export function parseLabInventorySyncResult(value: unknown): InventorySyncResult<"te-lab-components"> {
  const record = requireRecord(value, "inventory sync payload");
  return {
    dbPath: optionalString(record.dbPath) ?? "",
    parts: requireArray(record.parts, "catalog parts").map(parsePart),
    storageAreas: requireArray(record.storageAreas, "storage areas").map(parseStorageArea),
    storageContainers: requireArray(record.storageContainers, "storage containers").map(parseStorageContainer),
    stockPlacements: requireArray(record.stockPlacements, "stock placements").map(parseStockPlacement),
    summaries: requireArray(record.summaries, "part stock summaries").map(parsePartStockSummary),
    counts: parseCatalogCounts(record.counts),
    migration: parseCatalogMigrationStatus(record.migration),
    entriesChanged: optionalBoolean(record.entriesChanged),
    shared: parseSharedStatus(record.shared),
  };
}

export function parseLabPartMutationResult(value: unknown): CatalogMutationResult<Part> {
  return parseCatalogMutationResult(value, "part mutation", parsePart);
}

export function parseLabSimpleComponentMutationResult(
  value: unknown,
): CatalogMutationResult<SimpleComponentValue> {
  return parseCatalogMutationResult(value, "simple component mutation", parseSimpleComponentValue);
}

function parseSimpleComponentValue(value: unknown): SimpleComponentValue {
  const record = requireRecord(value, "simple component mutation value");
  if (record.part == null) {
    throw new Error("Invalid simple component mutation: missing part.");
  }
  if (record.placement !== null && !isRecord(record.placement)) {
    throw new Error("Invalid simple component mutation: placement must be object or null.");
  }
  return {
    part: parsePart(record.part),
    placement: record.placement === null ? null : parseStockPlacement(record.placement),
  };
}

export function parseLabStorageAreaMutationResult(value: unknown): CatalogMutationResult<StorageArea> {
  return parseCatalogMutationResult(value, "storage area mutation", parseStorageArea);
}

export function parseLabStorageContainerMutationResult(
  value: unknown,
): CatalogMutationResult<StorageContainer> {
  return parseCatalogMutationResult(value, "storage container mutation", parseStorageContainer);
}

export function parseLabStockPlacementMutationResult(
  value: unknown,
): CatalogMutationResult<StockPlacement> {
  return parseCatalogMutationResult(value, "stock placement mutation", parseStockPlacement);
}

export function parseLabStockMoveMutationResult(value: unknown): CatalogMutationResult<StockPlacement[]> {
  return parseCatalogMutationResult(value, "stock move mutation", (result) =>
    requireArray(result, "stock move placements").map(parseStockPlacement),
  );
}

export function parseLabCatalogDeleteResult(value: unknown): CatalogDeleteResult {
  const record = requireRecord(value, "catalog delete payload");
  const entityUuid = optionalString(record.entityUuid);
  if (!entityUuid) {
    throw new Error("Invalid catalog delete payload: missing entity UUID.");
  }
  return {
    entityUuid,
    message: optionalString(record.message) ?? "Catalog record deleted.",
    mutationMode: record.mutationMode === "shared" ? "shared" : "local",
    shared: parseSharedStatus(record.shared),
  };
}

export function parseLabCatalogMigrationPreview(value: unknown): CatalogMigrationPreview {
  const record = requireRecord(value, "catalog migration preview");
  return {
    mappingVersion: requireString(record.mappingVersion, "migration mapping version"),
    sourceFingerprint: requireString(record.sourceFingerprint, "migration source fingerprint"),
    sourceSchemaVersion: parseNullableFiniteNumber(record.sourceSchemaVersion),
    targetSchemaVersion: requireFiniteNumber(record.targetSchemaVersion, "migration target schema version"),
    legacyRows: requireFiniteNumber(record.legacyRows, "legacy row count"),
    proposedParts: requireFiniteNumber(record.proposedParts, "proposed part count"),
    proposedPlacements: requireFiniteNumber(record.proposedPlacements, "proposed placement count"),
    archivedParts: requireFiniteNumber(record.archivedParts, "archived part count"),
    blankPositiveQuantityLocations: requireFiniteNumber(
      record.blankPositiveQuantityLocations,
      "blank positive-quantity location count",
    ),
    generatedPartUuids: requireFiniteNumber(record.generatedPartUuids, "generated part UUID count"),
    duplicateInternalPartNumbers: requireArray(
      record.duplicateInternalPartNumbers,
      "duplicate internal part-number groups",
    ).map(parseMigrationDuplicateGroup),
    likelyMpnDuplicates: requireArray(record.likelyMpnDuplicates, "likely MPN duplicate groups").map(
      parseMigrationDuplicateGroup,
    ),
    invalidRows: parseStringArray(record.invalidRows, "invalid migration rows"),
    warnings: parseStringArray(record.warnings, "migration warnings"),
    blocking: requireBoolean(record.blocking, "migration blocking flag"),
  };
}

export function parseLabCatalogMigrationCommitResult(value: unknown): CatalogMigrationCommitResult {
  const record = requireRecord(value, "catalog migration commit result");
  return {
    sourceFingerprint: requireString(record.sourceFingerprint, "migration source fingerprint"),
    partsCreated: requireFiniteNumber(record.partsCreated, "created part count"),
    placementsCreated: requireFiniteNumber(record.placementsCreated, "created placement count"),
    areasCreated: requireFiniteNumber(record.areasCreated, "created area count"),
    containersCreated: requireFiniteNumber(record.containersCreated, "created container count"),
    noop: requireBoolean(record.noop, "migration no-op flag"),
    message: requireString(record.message, "migration commit message"),
  };
}

export function parseLabSharedCutoverPreview(value: unknown): CatalogSharedCutoverPreview {
  const record = requireRecord(value, "Lab shared cutover preview");
  return {
    localFingerprint: requireString(record.localFingerprint, "cutover local fingerprint"),
    sharedRootPath: requireString(record.sharedRootPath, "cutover shared root path"),
    sharedRootAvailable: requireBoolean(record.sharedRootAvailable, "shared-root availability"),
    catalogV2Initialized: requireBoolean(record.catalogV2Initialized, "catalog-v2 initialized flag"),
    legacyStreamState: requireString(record.legacyStreamState, "legacy stream state"),
    partCount: requireFiniteNumber(record.partCount, "cutover part count"),
    areaCount: requireFiniteNumber(record.areaCount, "cutover area count"),
    containerCount: requireFiniteNumber(record.containerCount, "cutover container count"),
    placementCount: requireFiniteNumber(record.placementCount, "cutover placement count"),
    warnings: parseStringArray(record.warnings, "cutover warnings"),
    blocking: requireBoolean(record.blocking, "cutover blocking flag"),
  };
}

export function parseLabSharedCutoverCommitResult(value: unknown): CatalogSharedCutoverCommitResult {
  const record = requireRecord(value, "Lab shared cutover commit result");
  return {
    localFingerprint: requireString(record.localFingerprint, "cutover local fingerprint"),
    legacyBackupPath: parseNullableStringValue(record.legacyBackupPath, "legacy backup path"),
    catalogRootPath: requireString(record.catalogRootPath, "catalog root path"),
    noop: requireBoolean(record.noop, "cutover no-op flag"),
    message: requireString(record.message, "cutover commit message"),
  };
}

export function parseLabInventoryQueryResult(value: unknown): InventoryQueryResult {
  const record = requireRecord(value, "inventory query payload");
  return {
    counts: parseCounts(record.counts),
    dbPath: optionalString(record.dbPath) ?? "",
    entries: requireArray(record.entries, "inventory entries").map(parseInventoryEntry),
    shared: parseSharedStatus(record.shared),
    totalFiltered: optionalFiniteNumber(record.totalFiltered) ?? 0,
  };
}

export function parseLabEntryMutationResult(value: unknown): InventoryEntryMutationResult {
  const record = requireRecord(value, "inventory mutation payload");
  return {
    entry: parseInventoryEntry(record.entry),
    message: optionalString(record.message) ?? "Inventory entry saved.",
    mutationMode: record.mutationMode === "shared" ? "shared" : "local",
    shared: parseSharedStatus(record.shared),
  };
}

export function parseLabDeleteMutationResult(value: unknown): InventoryDeleteMutationResult {
  const record = requireRecord(value, "inventory delete payload");
  const entryId = optionalString(record.entryId);
  if (!entryId) {
    throw new Error("Invalid inventory delete payload: missing entry id.");
  }

  return {
    entryId,
    message: optionalString(record.message) ?? "Inventory entry deleted.",
    mutationMode: record.mutationMode === "shared" ? "shared" : "local",
    shared: parseSharedStatus(record.shared),
  };
}

export function parseLabExcelExportResult(value: unknown): ExcelExportResult {
  const record = requireRecord(value, "Excel export payload");
  if (typeof record.canceled !== "boolean") {
    throw new Error("Invalid Excel export payload: missing canceled flag.");
  }

  return {
    canceled: record.canceled,
    error: optionalString(record.error),
    outputPath: optionalString(record.outputPath),
  };
}

export function parseNullableString(value: unknown, label: string): string | null {
  if (value === null || value === undefined) {
    return null;
  }
  if (typeof value !== "string") {
    throw new Error(`Invalid ${label}: expected a string or null.`);
  }
  return value;
}

export function parseBoolean(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") {
    throw new Error(`Invalid ${label}: expected a boolean.`);
  }
  return value;
}

export function parseUpdateState(value: unknown): UpdateState {
  const record = requireRecord(value, "update state");
  const status = UPDATE_STATUSES.has(record.status as UpdateStatus)
    ? (record.status as UpdateStatus)
    : "error";
  return {
    available: record.available === true,
    currentVersion: optionalString(record.currentVersion) ?? APP_VERSION,
    downloadPhase:
      record.downloadPhase === "copying" ||
      record.downloadPhase === "verifying" ||
      record.downloadPhase === "ready"
        ? record.downloadPhase
        : undefined,
    downloadProgress: optionalFiniteNumber(record.downloadProgress),
    downloadedInstallerPath: optionalString(record.downloadedInstallerPath),
    error: optionalString(record.error),
    installLogPath: optionalString(record.installLogPath),
    installerPid: optionalFiniteNumber(record.installerPid),
    latestVersion: optionalString(record.latestVersion),
    notes: optionalString(record.notes),
    publishedAt: optionalString(record.publishedAt),
    status,
  };
}

function parseCatalogMutationResult<T>(
  value: unknown,
  label: string,
  parseValue: (value: unknown) => T,
): CatalogMutationResult<T> {
  const record = requireRecord(value, `${label} payload`);
  return {
    value: parseValue(record.value),
    message: requireString(record.message, `${label} message`),
    mutationMode: record.mutationMode === "shared" ? "shared" : "local",
    shared: parseSharedStatus(record.shared),
  };
}

function parsePart(value: unknown): Part {
  const record = requireRecord(value, "catalog part");
  return {
    id: requireString(record.id, "part id"),
    databaseId: optionalFiniteNumber(record.databaseId),
    entryUuid: requireString(record.entryUuid, "part UUID"),
    internalPartNumber: optionalString(record.internalPartNumber) ?? "",
    category: optionalString(record.category) ?? "",
    subcategory: optionalString(record.subcategory) ?? "",
    manufacturer: optionalString(record.manufacturer) ?? "",
    manufacturerPartNumber: optionalString(record.manufacturerPartNumber) ?? "",
    displayValue: optionalString(record.displayValue) ?? "",
    mountingType: optionalString(record.mountingType) ?? "",
    packageType: optionalString(record.packageType) ?? "",
    description: optionalString(record.description) ?? "",
    attributes: parseComponentAttributes(record.attributes),
    supplier: optionalString(record.supplier) ?? "",
    supplierSku: optionalString(record.supplierSku) ?? "",
    supplierPackaging: optionalString(record.supplierPackaging) ?? "",
    productUrl: optionalString(record.productUrl) ?? "",
    datasheetUrl: optionalString(record.datasheetUrl) ?? "",
    defaultUnitOfMeasure: optionalString(record.defaultUnitOfMeasure) ?? "unknown",
    reorderPoint: parseNullableFiniteNumber(record.reorderPoint),
    targetQuantity: parseNullableFiniteNumber(record.targetQuantity),
    partStatus: optionalString(record.partStatus) ?? "active",
    picturePath: optionalString(record.picturePath) ?? "",
    notes: optionalString(record.notes) ?? "",
    archived: record.archived === true,
    legacy: parseLegacyPartFields(record.legacy),
    createdAt: optionalString(record.createdAt) ?? "",
    updatedAt: optionalString(record.updatedAt) ?? "",
  };
}

function parseStorageArea(value: unknown): StorageArea {
  const record = requireRecord(value, "storage area");
  return {
    areaUuid: requireString(record.areaUuid, "storage area UUID"),
    name: requireString(record.name, "storage area name"),
    areaType: optionalString(record.areaType) ?? "other",
    owner: optionalString(record.owner) ?? "",
    description: optionalString(record.description) ?? "",
    archived: record.archived === true,
    createdAt: optionalString(record.createdAt) ?? "",
    updatedAt: optionalString(record.updatedAt) ?? "",
  };
}

function parseStorageContainer(value: unknown): StorageContainer {
  const record = requireRecord(value, "storage container");
  return {
    containerUuid: requireString(record.containerUuid, "storage container UUID"),
    areaUuid: requireString(record.areaUuid, "storage container area UUID"),
    name: requireString(record.name, "storage container name"),
    containerType: optionalString(record.containerType) ?? "other",
    gridEnabled: requireBoolean(record.gridEnabled, "storage container grid flag"),
    rowCount: parseNullableFiniteNumber(record.rowCount),
    columnCount: parseNullableFiniteNumber(record.columnCount),
    rowStart: requireFiniteNumber(record.rowStart, "storage container row start"),
    origin: optionalString(record.origin) ?? "top_left",
    description: optionalString(record.description) ?? "",
    archived: record.archived === true,
    createdAt: optionalString(record.createdAt) ?? "",
    updatedAt: optionalString(record.updatedAt) ?? "",
  };
}

function parseStockPlacement(value: unknown): StockPlacement {
  const record = requireRecord(value, "stock placement");
  return {
    placementUuid: requireString(record.placementUuid, "stock placement UUID"),
    partUuid: requireString(record.partUuid, "stock placement part UUID"),
    containerUuid: requireString(record.containerUuid, "stock placement container UUID"),
    columnIndex: parseNullableFiniteNumber(record.columnIndex),
    rowIndex: parseNullableFiniteNumber(record.rowIndex),
    freeformPosition: optionalString(record.freeformPosition) ?? "",
    quantity: requireFiniteNumber(record.quantity, "stock placement quantity"),
    unitOfMeasure: requireString(record.unitOfMeasure, "stock placement unit"),
    packaging: optionalString(record.packaging) ?? "",
    lotCode: optionalString(record.lotCode) ?? "",
    dateCode: optionalString(record.dateCode) ?? "",
    condition: optionalString(record.condition) ?? "unknown",
    countState: optionalString(record.countState) ?? "uncounted",
    lastCountedAt: parseNullableStringValue(record.lastCountedAt, "last-counted timestamp"),
    lastCountedBy: optionalString(record.lastCountedBy) ?? "",
    notes: optionalString(record.notes) ?? "",
    archived: record.archived === true,
    createdAt: optionalString(record.createdAt) ?? "",
    updatedAt: optionalString(record.updatedAt) ?? "",
  };
}

function parsePartStockSummary(value: unknown): PartStockSummary {
  const record = requireRecord(value, "part stock summary");
  return {
    partUuid: requireString(record.partUuid, "summary part UUID"),
    totals: requireArray(record.totals, "part quantity totals").map(parseQuantityTotal),
    stockStatus: parseStockStatus(record.stockStatus),
  };
}

function parseQuantityTotal(value: unknown): QuantityTotal {
  const record = requireRecord(value, "quantity total");
  return {
    unitOfMeasure: requireString(record.unitOfMeasure, "quantity total unit"),
    quantity: requireFiniteNumber(record.quantity, "quantity total"),
  };
}

function parseStockStatus(value: unknown): StockStatus {
  const statuses = new Set<StockStatus>([
    "archived",
    "no_stock",
    "unit_review",
    "low_stock",
    "in_stock",
    "mixed_units",
  ]);
  if (!statuses.has(value as StockStatus)) {
    throw new Error("Invalid stock status.");
  }
  return value as StockStatus;
}

function parseCatalogCounts(value: unknown): CatalogCounts {
  const record = requireRecord(value, "catalog counts");
  return {
    activeParts: requireFiniteNumber(record.activeParts, "active part count"),
    archivedParts: requireFiniteNumber(record.archivedParts, "archived part count"),
    totalParts: requireFiniteNumber(record.totalParts, "total part count"),
    noStock: requireFiniteNumber(record.noStock, "no-stock count"),
    lowStock: requireFiniteNumber(record.lowStock, "low-stock count"),
    unitReview: requireFiniteNumber(record.unitReview, "unit-review count"),
  };
}

function parseCatalogMigrationStatus(value: unknown): CatalogMigrationStatus {
  const record = requireRecord(value, "catalog migration status");
  return {
    schemaVersion: parseNullableFiniteNumber(record.schemaVersion),
    required: requireBoolean(record.required, "migration required flag"),
    legacyEntryCount: requireFiniteNumber(record.legacyEntryCount, "legacy entry count"),
    catalogInitialized: requireBoolean(record.catalogInitialized, "catalog initialized flag"),
    message: requireString(record.message, "migration status message"),
  };
}

function parseComponentAttributes(value: unknown): ComponentAttributes {
  const record = requireRecord(value, "component attributes");
  const attributes: ComponentAttributes = {};
  for (const [key, attributeValue] of Object.entries(record)) {
    const attribute = requireRecord(attributeValue, `component attribute ${key}`);
    attributes[key] = {
      value: requireString(attribute.value, `component attribute ${key} value`),
      unit: optionalString(attribute.unit) ?? "",
    };
  }
  return attributes;
}

function parseLegacyPartFields(value: unknown): Part["legacy"] {
  const record = isRecord(value) ? value : {};
  return {
    serialNumber: optionalString(record.serialNumber) ?? "",
    projectName: optionalString(record.projectName) ?? "",
    assignedTo: optionalString(record.assignedTo) ?? "",
    lifecycleStatus: optionalString(record.lifecycleStatus) ?? "",
    workingStatus: optionalString(record.workingStatus) ?? "",
    condition: optionalString(record.condition) ?? "",
    verifiedInSurvey: record.verifiedInSurvey === true,
    manualEntry: record.manualEntry === true,
  };
}

function parseMigrationDuplicateGroup(value: unknown): MigrationDuplicateGroup {
  const record = requireRecord(value, "migration duplicate group");
  return {
    key: requireString(record.key, "migration duplicate key"),
    entryUuids: parseStringArray(record.entryUuids, "migration duplicate entry UUIDs"),
    entryIds: parseStringArray(record.entryIds, "migration duplicate entry IDs"),
  };
}

function parseInventoryEntry(value: unknown): InventoryEntry {
  const record = requireRecord(value, "inventory entry");
  const id = optionalString(record.id);
  if (!id) {
    throw new Error("Invalid inventory entry: missing id.");
  }

  return {
    id,
    archived: record.archived === true,
    assetNumber: optionalString(record.assetNumber) ?? "",
    assignedTo: optionalString(record.assignedTo) ?? "",
    condition: optionalString(record.condition) ?? "",
    createdAt: optionalString(record.createdAt) ?? "",
    databaseId: optionalFiniteNumber(record.databaseId),
    description: optionalString(record.description) ?? "",
    entryUuid: optionalString(record.entryUuid) ?? "",
    lifecycleStatus: parseLifecycleStatus(record.lifecycleStatus),
    links: optionalString(record.links) ?? "",
    location: optionalString(record.location) ?? "",
    manufacturer: optionalString(record.manufacturer) ?? "",
    manualEntry: record.manualEntry === true,
    model: optionalString(record.model) ?? "",
    notes: optionalString(record.notes) ?? "",
    picturePath: optionalString(record.picturePath) ?? "",
    projectName: optionalString(record.projectName) ?? "",
    qty: parseNullableFiniteNumber(record.qty),
    serialNumber: optionalString(record.serialNumber) ?? "",
    updatedAt: optionalString(record.updatedAt) ?? "",
    verifiedInSurvey: record.verifiedInSurvey === true,
    workingStatus: parseWorkingStatus(record.workingStatus),
  };
}

function parseSharedStatus(value: unknown): InventorySharedStatus {
  const record = isRecord(value) ? value : {};
  return {
    available: record.available === true,
    canModify: record.canModify !== false,
    enabled: record.enabled === true,
    hasLocalOnlyChanges: optionalBoolean(record.hasLocalOnlyChanges),
    lastSnapshotId: optionalString(record.lastSnapshotId),
    message: optionalString(record.message) ?? "Shared sync status unavailable.",
    mutationMode: record.mutationMode === "shared" ? "shared" : "local",
    revision: optionalString(record.revision),
    sharedRootPath: optionalString(record.sharedRootPath),
  };
}

function parseCounts(value: unknown): InventoryCounts {
  const record = isRecord(value) ? value : {};
  return {
    archive: optionalFiniteNumber(record.archive) ?? 0,
    inventory: optionalFiniteNumber(record.inventory) ?? 0,
    total: optionalFiniteNumber(record.total) ?? 0,
    verified: optionalFiniteNumber(record.verified) ?? 0,
  };
}

function parseLifecycleStatus(value: unknown): LifecycleStatus {
  return LIFECYCLE_OPTIONS.includes(value as LifecycleStatus) ? (value as LifecycleStatus) : "active";
}

function parseWorkingStatus(value: unknown): WorkingStatus {
  return WORKING_STATUS_OPTIONS.includes(value as WorkingStatus) ? (value as WorkingStatus) : "unknown";
}

function parseNullableFiniteNumber(value: unknown): number | null {
  if (value === null || value === undefined) {
    return null;
  }
  return optionalFiniteNumber(value) ?? null;
}

function optionalFiniteNumber(value: unknown): number | undefined {
  return typeof value === "number" && Number.isFinite(value) ? value : undefined;
}

function requireFiniteNumber(value: unknown, label: string): number {
  const parsed = optionalFiniteNumber(value);
  if (parsed === undefined) {
    throw new Error(`Invalid ${label}: expected a finite number.`);
  }
  return parsed;
}

function requireString(value: unknown, label: string): string {
  if (typeof value !== "string") {
    throw new Error(`Invalid ${label}: expected a string.`);
  }
  return value;
}

function requireBoolean(value: unknown, label: string): boolean {
  if (typeof value !== "boolean") {
    throw new Error(`Invalid ${label}: expected a boolean.`);
  }
  return value;
}

function parseNullableStringValue(value: unknown, label: string): string | null {
  if (value === null || value === undefined) {
    return null;
  }
  return requireString(value, label);
}

function parseStringArray(value: unknown, label: string): string[] {
  return requireArray(value, label).map((item) => requireString(item, label));
}

function optionalString(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

function optionalBoolean(value: unknown): boolean | undefined {
  return typeof value === "boolean" ? value : undefined;
}

function requireArray(value: unknown, label: string): unknown[] {
  if (!Array.isArray(value)) {
    throw new Error(`Invalid ${label}: expected an array.`);
  }
  return value;
}

function requireRecord(value: unknown, label: string): Record<string, unknown> {
  if (!isRecord(value)) {
    throw new Error(`Invalid ${label}: expected an object.`);
  }
  return value;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}
