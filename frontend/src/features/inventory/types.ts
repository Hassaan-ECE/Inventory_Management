export type InventoryScope = "inventory" | "archive";
export type ThemeMode = "light" | "dark";
export type SortDirection = "asc" | "desc";
export type LifecycleStatus = "active" | "repair" | "scrapped" | "missing" | "rental";
export type WorkingStatus = "working" | "limited" | "not_working" | "unknown";
export type CalibrationRequirement = "required" | "reference_only" | "not_required" | "unknown";
export type CalibrationHealth =
  | "missing_due"
  | "overdue"
  | "due_soon"
  | "current"
  | "not_applicable"
  | "unknown"
  | "out_to_cal";
export type DueWindow = "all" | "overdue" | "next30" | "next60" | "next90" | "missing";

export const LIFECYCLE_OPTIONS = ["active", "repair", "scrapped", "missing", "rental"] as const satisfies readonly LifecycleStatus[];
export const WORKING_STATUS_OPTIONS = ["unknown", "working", "limited", "not_working"] as const satisfies readonly WorkingStatus[];
export const CALIBRATION_REQUIREMENT_OPTIONS = [
  "required",
  "reference_only",
  "not_required",
  "unknown",
] as const satisfies readonly CalibrationRequirement[];
export const CALIBRATION_HEALTH_OPTIONS = [
  "missing_due",
  "overdue",
  "due_soon",
  "current",
  "not_applicable",
  "unknown",
  "out_to_cal",
] as const satisfies readonly CalibrationHealth[];

export interface ImportProvenance {
  batchId: string;
  sourceFilename: string;
  sourceSheet?: string;
  sourceRow: number;
  originalId?: string;
  originalAssetNumber?: string;
  originalSerialNumber?: string;
}

export interface InventoryEntry {
  id: string;
  databaseId?: number;
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
  calibrationRequirement: CalibrationRequirement;
  outToCalibration: boolean;
  lastCalibratedAt?: string;
  calibrationDueAt?: string;
  calibrationIntervalMonths?: number;
  certificateRef?: string;
  calibrationVendor?: string;
  calibrationNotes?: string;
  verifiedAt?: string;
  verifiedBy?: string;
  importProvenance?: ImportProvenance;
  archived: boolean;
  createdAt: string;
  updatedAt: string;
  entryUuid: string;
  manualEntry: boolean;
  picturePath: string;
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
  calibrationRequirement: CalibrationRequirement;
  outToCalibration: boolean;
  lastCalibratedAt?: string;
  calibrationDueAt?: string;
  calibrationIntervalMonths?: number;
  certificateRef?: string;
  calibrationVendor?: string;
  calibrationNotes?: string;
  verifiedAt?: string;
  verifiedBy?: string;
  archived: boolean;
  picturePath?: string;
}

export interface InventoryEntryEditContext {
  baseVersion?: string;
  changedFields: string[];
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
  syncIntervalMs?: number;
}

export interface InventoryCounts {
  archive: number;
  dueSoon: number;
  inventory: number;
  missingDue: number;
  outToCal: number;
  overdue: number;
  total: number;
  verified: number;
}

export type ImportClassification = "inserted" | "matched" | "conflicted" | "rejected" | "ignored";
export type ImportColumnTreatment = "mapped" | "intentionally_ignored" | "unknown";

export interface ImportColumnReport {
  originalHeader: string;
  normalizedTarget: string | null;
  treatment: ImportColumnTreatment;
  nonblankCount: number;
  reason: string;
}

export interface ImportRowOutcome {
  sourceRow: number;
  classification: ImportClassification;
  issues: string[];
  originalId: string | null;
  originalAssetNumber: string | null;
  originalSerialNumber: string | null;
  candidateEntryUuid: string | null;
  rawValues: Record<string, string>;
}

export interface ImportDryRunReport {
  batchId: string;
  sourceFingerprint: string;
  sourceFilename: string;
  selectedSheet: string;
  mappingVersion: string;
  totalRows: number;
  inserted: number;
  matched: number;
  conflicted: number;
  rejected: number;
  ignored: number;
  columns: ImportColumnReport[];
  rowOutcomes: ImportRowOutcome[];
  blocking: boolean;
  reconciliationBasis: string;
}

export interface ImportCommitInput {
  batchId: string;
  confirmed: boolean;
}

export interface ImportCommitResult {
  batchId: string;
  inserted: number;
  matched: number;
  conflicted: number;
  rejected: number;
  ignored: number;
  remaining: number;
  noop: number;
  entriesChanged: boolean;
  message: string;
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

export interface FilterState {
  assetNumber: string;
  manufacturer: string;
  model: string;
  description: string;
  location: string;
  calibrationRequirement: CalibrationRequirement | "all";
  calibrationHealth: CalibrationHealth | "all";
  dueWindow: DueWindow;
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
    | "serialNumber"
    | "qty"
    | "manufacturer"
    | "model"
    | "description"
    | "projectName"
    | "location"
    | "links"
    | "calibrationRequirement"
    | "outToCalibration"
    | "calibrationDueAt"
    | "calibrationHealth";
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
  { key: "serialNumber", label: "Serial #", defaultVisible: true, sortable: true },
  { key: "qty", label: "Qty", defaultVisible: true, sortable: true, align: "center" },
  { key: "manufacturer", label: "Manufacturer", defaultVisible: true, sortable: true },
  { key: "model", label: "Model", defaultVisible: true, sortable: true },
  { key: "description", label: "Description", defaultVisible: true, sortable: true },
  { key: "projectName", label: "Project", defaultVisible: false, sortable: true },
  { key: "location", label: "Location", defaultVisible: true, sortable: true },
  { key: "calibrationRequirement", label: "Calibration", defaultVisible: true, sortable: true },
  { key: "outToCalibration", label: "Out to cal", defaultVisible: true, sortable: true, align: "center" },
  { key: "calibrationDueAt", label: "Calibration due", defaultVisible: true, sortable: true },
  { key: "calibrationHealth", label: "Calibration health", defaultVisible: true, sortable: true },
  { key: "links", label: "Links", defaultVisible: true, sortable: true },
] as const satisfies readonly ColumnConfig[];
