import type { CalibrationRequirement } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRequirement";
import type { InventoryEntry } from "@/integrations/tauri/generated/te-test-equipment/InventoryEntry";

// Bridge payloads are generated from the Rust types; `cargo test` regenerates them.
export type { CalibrationRequirement, InventoryEntry };
export type { CalibrationRosterAbsentEntry } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterAbsentEntry";
export type { CalibrationRosterCandidateContext } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterCandidateContext";
export type { CalibrationRosterClassification } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterClassification";
export type { CalibrationRosterCommitInput } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterCommitInput";
export type { CalibrationRosterCommitResult } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterCommitResult";
export type { CalibrationRosterCounts } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterCounts";
export type { CalibrationRosterFieldChange } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterFieldChange";
export type { CalibrationRosterPreviewReport } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterPreviewReport";
export type { CalibrationRosterResolution } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterResolution";
export type { CalibrationRosterResolutionAction } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterResolutionAction";
export type { CalibrationRosterRowOutcome } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterRowOutcome";
export type { CalibrationRosterSemanticFlag } from "@/integrations/tauri/generated/te-test-equipment/CalibrationRosterSemanticFlag";
export type { ImportProvenance } from "@/integrations/tauri/generated/te-test-equipment/ImportProvenance";
export type { InventoryDeleteMutationResult } from "@/integrations/tauri/generated/te-test-equipment/InventoryDeleteMutationResult";
export type { InventoryEntryEditContext } from "@/integrations/tauri/generated/te-test-equipment/InventoryEntryEditContext";
export type { InventoryEntryInput } from "@/integrations/tauri/generated/te-test-equipment/InventoryEntryInput";
export type { InventoryEntryMutationResult } from "@/integrations/tauri/generated/te-test-equipment/InventoryEntryMutationResult";
export type { InventorySyncResult } from "@/integrations/tauri/generated/te-test-equipment/InventorySyncResult";
export type { ExcelExportResult } from "@/integrations/tauri/generated/platform/ExcelExportResult";
export type { InventorySharedStatus } from "@/integrations/tauri/generated/platform/InventorySharedStatus";

export type LifecycleStatus = InventoryEntry["lifecycleStatus"];
export type WorkingStatus = InventoryEntry["workingStatus"];

export type InventoryScope = "inventory" | "archive";
export type TeTestEquipmentWorkspace = "equipment" | "calibration";
type SortDirection = "asc" | "desc";
export type CalibrationHealth =
  | "missing_due"
  | "overdue"
  | "due_soon"
  | "current"
  | "not_applicable"
  | "unknown"
  | "out_to_cal";
type DueWindow = "all" | "overdue" | "next30" | "next60" | "next90" | "missing";

export const LIFECYCLE_OPTIONS = ["active", "repair", "scrapped", "missing", "rental"] as const satisfies readonly LifecycleStatus[];
export const WORKING_STATUS_OPTIONS = ["unknown", "working", "limited", "not_working"] as const satisfies readonly WorkingStatus[];
export const CALIBRATION_REQUIREMENT_OPTIONS = [
  "required",
  "reference_only",
  "not_required",
  "unknown",
] as const satisfies readonly CalibrationRequirement[];

/** Counts the footer and filter chips show, computed in the frontend from the loaded entries. */
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

export type ColumnKey =
  | "verified"
  | "assetNumber"
  | "serialNumber"
  | "qty"
  | "manufacturer"
  | "model"
  | "description"
  | "projectName"
  | "location"
  | "assignedTo"
  | "links"
  | "calibrationRequirement"
  | "outToCalibration"
  | "lastCalibratedAt"
  | "calibrationDueAt"
  | "calibrationIntervalMonths"
  | "certificateRef"
  | "calibrationVendor"
  | "calibrationNotes"
  | "calibrationHealth";

export interface ColumnConfig {
  key: ColumnKey;
  label: string;
  defaultVisible: boolean;
  sortable: boolean;
  align?: "left" | "center";
}

export interface SortState {
  column: ColumnKey;
  direction: SortDirection;
}

export const EQUIPMENT_COLUMNS = [
  { key: "verified", label: "✓", defaultVisible: true, sortable: true, align: "center" },
  { key: "assetNumber", label: "Asset #", defaultVisible: false, sortable: true },
  { key: "serialNumber", label: "Serial #", defaultVisible: true, sortable: true },
  { key: "qty", label: "Qty", defaultVisible: true, sortable: true, align: "center" },
  { key: "manufacturer", label: "Manufacturer", defaultVisible: true, sortable: true },
  { key: "model", label: "Model", defaultVisible: true, sortable: true },
  { key: "description", label: "Description", defaultVisible: true, sortable: true },
  { key: "projectName", label: "Project", defaultVisible: false, sortable: true },
  { key: "location", label: "Location", defaultVisible: true, sortable: true },
  { key: "calibrationDueAt", label: "Calibration due", defaultVisible: true, sortable: true },
  { key: "calibrationHealth", label: "Calibration health", defaultVisible: true, sortable: true },
  { key: "links", label: "Links", defaultVisible: true, sortable: true },
] as const satisfies readonly ColumnConfig[];

// Defaults are sized for a single vertical scroll on a 1080p desktop window.
// Optional columns (serial, vendor, certificate, notes, …) stay available in View settings.
export const CALIBRATION_COLUMNS = [
  { key: "verified", label: "✓", defaultVisible: true, sortable: true, align: "center" },
  { key: "assetNumber", label: "Asset #", defaultVisible: true, sortable: true },
  { key: "serialNumber", label: "Serial #", defaultVisible: false, sortable: true },
  { key: "manufacturer", label: "Manufacturer", defaultVisible: true, sortable: true },
  { key: "model", label: "Model", defaultVisible: true, sortable: true },
  { key: "description", label: "Description", defaultVisible: true, sortable: true },
  { key: "calibrationHealth", label: "Health", defaultVisible: false, sortable: true },
  { key: "lastCalibratedAt", label: "Last", defaultVisible: true, sortable: true },
  { key: "calibrationDueAt", label: "Due", defaultVisible: true, sortable: true },
  { key: "calibrationIntervalMonths", label: "Interval (months)", defaultVisible: false, sortable: true, align: "center" },
  { key: "outToCalibration", label: "Out to cal", defaultVisible: false, sortable: true, align: "center" },
  { key: "calibrationVendor", label: "Vendor", defaultVisible: false, sortable: true },
  { key: "certificateRef", label: "Certificate", defaultVisible: false, sortable: true },
  { key: "location", label: "Location", defaultVisible: true, sortable: true },
  { key: "assignedTo", label: "Assigned to", defaultVisible: false, sortable: true },
  { key: "calibrationNotes", label: "Calibration notes", defaultVisible: false, sortable: true },
] as const satisfies readonly ColumnConfig[];
