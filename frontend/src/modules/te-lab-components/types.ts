export type InventoryScope = "inventory" | "archive";
export type ThemeMode = "light" | "dark";
export type SortDirection = "asc" | "desc";
export type LifecycleStatus = "active" | "repair" | "scrapped" | "missing" | "rental";
export type WorkingStatus = "working" | "limited" | "not_working" | "unknown";

export const LIFECYCLE_OPTIONS = ["active", "repair", "scrapped", "missing", "rental"] as const satisfies readonly LifecycleStatus[];
export const WORKING_STATUS_OPTIONS = ["unknown", "working", "limited", "not_working"] as const satisfies readonly WorkingStatus[];

export interface InventoryEntry {
  id: string;
  databaseId?: number;
  assetNumber: string;
  serialNumber?: string;
  qty: number | null;
  manufacturer: string;
  model: string;
  description: string;
  projectName: string;
  location: string;
  assignedTo?: string;
  links: string;
  notes: string;
  lifecycleStatus: LifecycleStatus;
  workingStatus: WorkingStatus;
  condition?: string;
  verifiedInSurvey: boolean;
  archived: boolean;
  createdAt?: string;
  updatedAt: string;
  entryUuid?: string;
  manualEntry?: boolean;
  picturePath?: string;
}

export interface InventoryEntryInput {
  assetNumber: string;
  serialNumber: string;
  qty: number | null;
  manufacturer: string;
  model: string;
  description: string;
  projectName: string;
  location: string;
  assignedTo: string;
  links: string;
  notes: string;
  lifecycleStatus: LifecycleStatus;
  workingStatus: WorkingStatus;
  condition: string;
  verifiedInSurvey: boolean;
  archived: boolean;
  picturePath?: string;
}

export interface InventoryEntryEditContext {
  baseVersion?: string;
  changedFields: string[];
}

export interface InventorySharedStatus {
  available: boolean;
  canModify: boolean;
  enabled: boolean;
  hasLocalOnlyChanges?: boolean;
  message: string;
  mutationMode?: "shared" | "local";
  revision?: string;
  lastSnapshotId?: string;
  sharedRootPath?: string;
}

export type CatalogScope = "inventory" | "archive";
export type StockStatus =
  | "archived"
  | "no_stock"
  | "unit_review"
  | "low_stock"
  | "in_stock"
  | "mixed_units";

export interface ComponentAttributeValue {
  value: string;
  unit: string;
}

export type ComponentAttributes = Record<string, ComponentAttributeValue>;

export interface LegacyPartFields {
  serialNumber: string;
  projectName: string;
  assignedTo: string;
  lifecycleStatus: string;
  workingStatus: string;
  condition: string;
  verifiedInSurvey: boolean;
  manualEntry: boolean;
}

export interface Part {
  id: string;
  databaseId?: number;
  entryUuid: string;
  internalPartNumber: string;
  category: string;
  subcategory: string;
  manufacturer: string;
  manufacturerPartNumber: string;
  displayValue: string;
  mountingType: string;
  packageType: string;
  description: string;
  attributes: ComponentAttributes;
  supplier: string;
  supplierSku: string;
  supplierPackaging: string;
  productUrl: string;
  datasheetUrl: string;
  defaultUnitOfMeasure: string;
  reorderPoint: number | null;
  targetQuantity: number | null;
  partStatus: string;
  picturePath: string;
  notes: string;
  archived: boolean;
  legacy: LegacyPartFields;
  createdAt: string;
  updatedAt: string;
}

export interface PartInput {
  internalPartNumber: string;
  category: string;
  subcategory: string;
  manufacturer: string;
  manufacturerPartNumber: string;
  displayValue: string;
  mountingType: string;
  packageType: string;
  description: string;
  attributes: ComponentAttributes;
  supplier: string;
  supplierSku: string;
  supplierPackaging: string;
  productUrl: string;
  datasheetUrl: string;
  defaultUnitOfMeasure: string;
  reorderPoint: number | null;
  targetQuantity: number | null;
  partStatus: string;
  picturePath?: string;
  notes: string;
  archived: boolean;
}

export interface StorageArea {
  areaUuid: string;
  name: string;
  areaType: string;
  owner: string;
  description: string;
  archived: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface StorageAreaInput {
  name: string;
  areaType: string;
  owner: string;
  description: string;
  archived: boolean;
}

export interface StorageContainer {
  containerUuid: string;
  areaUuid: string;
  name: string;
  containerType: string;
  gridEnabled: boolean;
  rowCount: number | null;
  columnCount: number | null;
  rowStart: number;
  origin: string;
  description: string;
  archived: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface StorageContainerInput {
  areaUuid: string;
  name: string;
  containerType: string;
  gridEnabled: boolean;
  rowCount: number | null;
  columnCount: number | null;
  rowStart: number | null;
  description: string;
  archived: boolean;
}

export interface StockPlacement {
  placementUuid: string;
  partUuid: string;
  containerUuid: string;
  columnIndex: number | null;
  rowIndex: number | null;
  freeformPosition: string;
  quantity: number;
  unitOfMeasure: string;
  packaging: string;
  lotCode: string;
  dateCode: string;
  condition: string;
  countState: string;
  lastCountedAt: string | null;
  lastCountedBy: string;
  notes: string;
  archived: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface StockPlacementInput {
  partUuid: string;
  containerUuid: string;
  columnIndex: number | null;
  rowIndex: number | null;
  freeformPosition: string;
  quantity: number;
  unitOfMeasure: string;
  packaging: string;
  lotCode: string;
  dateCode: string;
  condition: string;
  countState: string;
  lastCountedAt: string | null;
  lastCountedBy: string;
  notes: string;
  archived: boolean;
}

export interface StockMoveInput {
  sourcePlacementUuid: string;
  destinationPlacementUuid: string | null;
  destination: StockPlacementInput | null;
  quantity: number;
}

export interface StockCountInput {
  quantity: number;
  countedBy: string;
}

export interface QuantityTotal {
  unitOfMeasure: string;
  quantity: number;
}

export interface PartStockSummary {
  partUuid: string;
  totals: QuantityTotal[];
  stockStatus: StockStatus;
}

export interface CatalogCounts {
  activeParts: number;
  archivedParts: number;
  totalParts: number;
  noStock: number;
  lowStock: number;
  unitReview: number;
}

export interface CatalogMigrationStatus {
  schemaVersion: number | null;
  required: boolean;
  legacyEntryCount: number;
  catalogInitialized: boolean;
  message: string;
}

export interface CatalogSyncResult {
  dbPath: string;
  parts: Part[];
  storageAreas: StorageArea[];
  storageContainers: StorageContainer[];
  stockPlacements: StockPlacement[];
  summaries: PartStockSummary[];
  counts: CatalogCounts;
  migration: CatalogMigrationStatus;
  entriesChanged?: boolean;
  shared: InventorySharedStatus;
}

export interface CatalogMutationResult<T> {
  value: T;
  message: string;
  mutationMode: InventoryMutationMode;
  shared: InventorySharedStatus;
}

export interface SimpleComponentInput {
  part: PartInput;
  quantity: number;
  location: string;
}

export interface SimpleComponentValue {
  part: Part;
  placement: StockPlacement | null;
}

export interface CatalogDeleteResult {
  entityUuid: string;
  message: string;
  mutationMode: InventoryMutationMode;
  shared: InventorySharedStatus;
}

export interface MigrationDuplicateGroup {
  key: string;
  entryUuids: string[];
  entryIds: string[];
}

export interface CatalogMigrationPreview {
  mappingVersion: string;
  sourceFingerprint: string;
  sourceSchemaVersion: number | null;
  targetSchemaVersion: number;
  legacyRows: number;
  proposedParts: number;
  proposedPlacements: number;
  archivedParts: number;
  blankPositiveQuantityLocations: number;
  generatedPartUuids: number;
  duplicateInternalPartNumbers: MigrationDuplicateGroup[];
  likelyMpnDuplicates: MigrationDuplicateGroup[];
  invalidRows: string[];
  warnings: string[];
  blocking: boolean;
}

export interface CatalogMigrationCommitInput {
  sourceFingerprint: string;
  confirmed: boolean;
}

export interface CatalogMigrationCommitResult {
  sourceFingerprint: string;
  partsCreated: number;
  placementsCreated: number;
  areasCreated: number;
  containersCreated: number;
  noop: boolean;
  message: string;
}

export interface CatalogSharedCutoverPreview {
  localFingerprint: string;
  sharedRootPath: string;
  sharedRootAvailable: boolean;
  catalogV2Initialized: boolean;
  legacyStreamState: string;
  partCount: number;
  areaCount: number;
  containerCount: number;
  placementCount: number;
  warnings: string[];
  blocking: boolean;
}

export interface CatalogSharedCutoverCommitInput {
  localFingerprint: string;
  confirmed: boolean;
}

export interface CatalogSharedCutoverCommitResult {
  localFingerprint: string;
  legacyBackupPath: string | null;
  catalogRootPath: string;
  noop: boolean;
  message: string;
}

export interface InventoryCounts {
  archive: number;
  inventory: number;
  total: number;
  verified: number;
}

export type InventoryMutationMode = "shared" | "local";

export interface InventoryEntryMutationResult {
  entry: InventoryEntry;
  message: string;
  mutationMode: InventoryMutationMode;
  shared?: InventorySharedStatus;
}

export interface InventoryDeleteMutationResult {
  entryId: string;
  message: string;
  mutationMode: InventoryMutationMode;
  shared?: InventorySharedStatus;
}

export interface ExcelExportResult {
  canceled: boolean;
  error?: string;
  outputPath?: string;
}

export interface LabOrderRequestLineInput {
  partUuid: string;
  requestedQuantity: number;
  note: string;
}

export type UpdateStatus =
  | "idle"
  | "checking"
  | "available"
  | "not-available"
  | "downloading"
  | "ready"
  | "installing"
  | "error";

export interface UpdateState {
  available: boolean;
  currentVersion: string;
  downloadPhase?: "copying" | "verifying" | "ready";
  downloadProgress?: number;
  downloadedInstallerPath?: string;
  error?: string;
  installLogPath?: string;
  installerPid?: number;
  latestVersion?: string;
  notes?: string;
  publishedAt?: string;
  status: UpdateStatus;
}

export interface FilterState {
  assetNumber: string;
  manufacturer: string;
  model: string;
  description: string;
  location: string;
}

export interface InventoryQueryInput {
  filters: FilterState;
  limit?: number;
  offset?: number;
  query: string;
  scope: InventoryScope;
  sort: SortState;
}

export interface InventoryQueryResult {
  counts: InventoryCounts;
  dbPath: string;
  entries: InventoryEntry[];
  shared: InventorySharedStatus;
  totalFiltered: number;
}

export interface ColumnConfig {
  key:
    | "verified"
    | "assetNumber"
    | "qty"
    | "manufacturer"
    | "model"
    | "description"
    | "projectName"
    | "location"
    | "links";
  label: string;
  defaultVisible: boolean;
  sortable: boolean;
  align?: "left" | "center";
}

export type ColumnKey = ColumnConfig["key"];

export interface SortState {
  column: ColumnKey;
  direction: SortDirection;
}

export const INVENTORY_COLUMNS = [
  { key: "verified", label: "Verified", defaultVisible: true, sortable: true, align: "center" },
  { key: "assetNumber", label: "Asset #", defaultVisible: false, sortable: true },
  { key: "qty", label: "Qty", defaultVisible: true, sortable: true, align: "center" },
  { key: "manufacturer", label: "Manufacturer", defaultVisible: true, sortable: true },
  { key: "model", label: "Model", defaultVisible: true, sortable: true },
  { key: "description", label: "Description", defaultVisible: true, sortable: true },
  { key: "projectName", label: "Project", defaultVisible: false, sortable: true },
  { key: "location", label: "Location", defaultVisible: true, sortable: true },
  { key: "links", label: "Links", defaultVisible: true, sortable: true },
] as const satisfies readonly ColumnConfig[];
