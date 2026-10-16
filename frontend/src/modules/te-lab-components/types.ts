type InventoryScope = "inventory" | "archive";
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

interface ComponentAttributeValue {
  value: string;
  unit: string;
}

export type ComponentAttributes = Record<string, ComponentAttributeValue>;

interface LegacyPartFields {
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

export interface CatalogSyncResult {
  dbPath: string;
  parts: Part[];
  storageAreas: StorageArea[];
  storageContainers: StorageContainer[];
  stockPlacements: StockPlacement[];
  summaries: PartStockSummary[];
  counts: CatalogCounts;
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

export interface InventoryCounts {
  archive: number;
  inventory: number;
  total: number;
  verified: number;
}

type InventoryMutationMode = "shared" | "local";

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

export interface LabOrderRequestLineInput {
  partUuid: string;
  requestedQuantity: number;
  note: string;
}

interface FilterState {
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

interface ColumnConfig {
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

type ColumnKey = ColumnConfig["key"];

interface SortState {
  column: ColumnKey;
  direction: SortDirection;
}
