import {
  LIFECYCLE_OPTIONS,
  WORKING_STATUS_OPTIONS,
  CALIBRATION_REQUIREMENT_OPTIONS,
  type InventoryEntry,
} from "@/features/inventory/types";

import type { EntryFormState } from "./form";

type EntryInputFieldKey = Exclude<
  keyof EntryFormState,
  "archived" | "lifecycleStatus" | "notes" | "picturePath" | "workingStatus" |
  "calibrationRequirement" | "outToCalibration" | "lastCalibratedAt" | "calibrationDueAt" |
  "calibrationIntervalMonths" | "certificateRef" | "calibrationVendor" | "calibrationNotes" |
  "verifiedAt" | "verifiedBy"
>;

interface EntryInputFieldConfig {
  autoFocus?: boolean;
  className?: string;
  inputMode?: "decimal";
  key: EntryInputFieldKey;
  label: string;
  placeholder: string;
}

export const ENTRY_MAIN_INPUT_FIELDS: readonly EntryInputFieldConfig[] = [
  { autoFocus: true, key: "assetNumber", label: "Asset Number", placeholder: "Optional asset tag" },
  { key: "serialNumber", label: "Serial / Internal ID", placeholder: "Serial or internal ID" },
  { key: "manufacturer", label: "Manufacturer / Brand", placeholder: "Maker, brand, or supplier" },
  { key: "model", label: "Model / Part No.", placeholder: "Model or part number" },
  { inputMode: "decimal", key: "qty", label: "Quantity", placeholder: "Quantity on hand" },
  { key: "projectName", label: "Project", placeholder: "Project this entry supports" },
  { className: "lg:col-span-2", key: "description", label: "Description", placeholder: "Part or entry description" },
  { key: "location", label: "Location", placeholder: "Shelf, room, bin, or area" },
  { key: "assignedTo", label: "Used By / Assigned To", placeholder: "Person or team using it" },
  { className: "lg:col-span-2", key: "links", label: "Links", placeholder: "Product, vendor, or reference link" },
] as const;

export const ENTRY_CONDITION_FIELD: EntryInputFieldConfig = {
  className: "lg:col-span-2",
  key: "condition",
  label: "Condition",
  placeholder: "Condition or operating note",
};

export const ENTRY_SELECT_FIELDS = [
  { key: "lifecycleStatus", label: "Lifecycle", options: LIFECYCLE_OPTIONS },
  { key: "workingStatus", label: "Working Status", options: WORKING_STATUS_OPTIONS },
  { key: "calibrationRequirement", label: "Calibration requirement", options: CALIBRATION_REQUIREMENT_OPTIONS },
] as const;

export type EntrySelectField = (typeof ENTRY_SELECT_FIELDS)[number];

export const ENTRY_BOOLEAN_FIELDS = [
  { key: "outToCalibration", label: "Out to calibration" },
  { key: "archived", label: "Archived entry" },
] as const;

export function buildEntryContextRows(entry: InventoryEntry): Array<{ label: string; value: string }> {
  return [
    { label: "Entry ID", value: entry.id },
    { label: "Created", value: entry.createdAt || "-" },
    { label: "Updated", value: entry.updatedAt || "-" },
    { label: "Status", value: entry.archived ? "Archived" : "Inventory" },
    { label: "Verified", value: entry.verifiedAt ? `${entry.verifiedAt}${entry.verifiedBy ? ` by ${entry.verifiedBy}` : ""}` : "Pending" },
    { label: "Manual Entry", value: entry.manualEntry ? "Yes" : "No" },
  ];
}
