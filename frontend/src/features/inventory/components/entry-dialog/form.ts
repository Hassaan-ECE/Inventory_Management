import type { Dispatch, SetStateAction } from "react";

import type {
  CalibrationRequirement,
  InventoryEntry,
  InventoryEntryInput,
  LifecycleStatus,
  WorkingStatus,
} from "@/features/inventory/types";
import { isValidDateOnly } from "@/features/inventory/lib";

export interface EntryFormState {
  archived: boolean;
  assetNumber: string;
  assignedTo: string;
  condition: string;
  calibrationRequirement: CalibrationRequirement;
  outToCalibration: boolean;
  lastCalibratedAt: string;
  calibrationDueAt: string;
  calibrationIntervalMonths: string;
  certificateRef: string;
  calibrationVendor: string;
  calibrationNotes: string;
  verifiedAt: string;
  verifiedBy: string;
  description: string;
  lifecycleStatus: LifecycleStatus;
  links: string;
  location: string;
  manufacturer: string;
  model: string;
  notes: string;
  picturePath: string;
  projectName: string;
  qty: string;
  serialNumber: string;
  workingStatus: WorkingStatus;
}

export function buildFormState(entry: InventoryEntry | null | undefined, defaultArchived: boolean): EntryFormState {
  return {
    archived: entry?.archived ?? defaultArchived,
    assetNumber: entry?.assetNumber ?? "",
    assignedTo: entry?.assignedTo ?? "",
    condition: entry?.condition ?? "",
    calibrationRequirement: entry?.calibrationRequirement ?? "unknown",
    outToCalibration: entry?.outToCalibration ?? false,
    lastCalibratedAt: entry?.lastCalibratedAt ?? "",
    calibrationDueAt: entry?.calibrationDueAt ?? "",
    calibrationIntervalMonths: entry?.calibrationIntervalMonths?.toString() ?? "",
    certificateRef: entry?.certificateRef ?? "",
    calibrationVendor: entry?.calibrationVendor ?? "",
    calibrationNotes: entry?.calibrationNotes ?? "",
    verifiedAt: entry?.verifiedAt ?? "",
    verifiedBy: entry?.verifiedBy ?? "",
    description: entry?.description ?? "",
    lifecycleStatus: entry?.lifecycleStatus ?? "active",
    links: entry?.links ?? "",
    location: entry?.location ?? "",
    manufacturer: entry?.manufacturer ?? "",
    model: entry?.model ?? "",
    notes: entry?.notes ?? "",
    picturePath: entry?.picturePath ?? "",
    projectName: entry?.projectName ?? "",
    qty: entry?.qty == null ? "" : String(entry.qty),
    serialNumber: entry?.serialNumber ?? "",
    workingStatus: entry?.workingStatus ?? "unknown",
  };
}

export function buildEntryInput(form: EntryFormState): { value: InventoryEntryInput } | { error: string } {
  const qtyText = form.qty.trim();
  let qty: number | null = null;

  if (qtyText) {
    qty = Number(qtyText);
    if (!Number.isFinite(qty)) {
      return { error: "Enter quantity as a number, for example 4 or 4.5." };
    }
  }

  if (!hasIdentity(form)) {
    return {
      error: "Provide at least an asset number, serial number, manufacturer, model, or description before saving.",
    };
  }

  for (const [label, value] of [["Last calibrated date", form.lastCalibratedAt], ["Calibration due date", form.calibrationDueAt]] as const) {
    if (value.trim() && !isValidDateOnly(value.trim())) return { error: `${label} must be a valid date in YYYY-MM-DD format.` };
  }
  if (form.lastCalibratedAt.trim() && form.calibrationDueAt.trim() && form.calibrationDueAt.trim() < form.lastCalibratedAt.trim()) {
    return { error: "Calibration due date cannot be before last calibrated date." };
  }
  const intervalText = form.calibrationIntervalMonths.trim();
  const calibrationIntervalMonths = intervalText ? Number(intervalText) : undefined;
  if (calibrationIntervalMonths !== undefined && (!Number.isInteger(calibrationIntervalMonths) || calibrationIntervalMonths < 1 || calibrationIntervalMonths > 1200)) {
    return { error: "Calibration interval must be between 1 and 1200 months." };
  }
  if (form.verifiedBy.trim() && !form.verifiedAt.trim()) {
    return { error: "Verified by requires a verification timestamp." };
  }

  return {
    value: {
      archived: form.archived,
      assetNumber: form.assetNumber.trim(),
      assignedTo: form.assignedTo.trim(),
      condition: form.condition.trim(),
      calibrationRequirement: form.calibrationRequirement,
      outToCalibration: form.outToCalibration,
      lastCalibratedAt: form.lastCalibratedAt.trim() || undefined,
      calibrationDueAt: form.calibrationDueAt.trim() || undefined,
      calibrationIntervalMonths,
      certificateRef: form.certificateRef.trim() || undefined,
      calibrationVendor: form.calibrationVendor.trim() || undefined,
      calibrationNotes: form.calibrationNotes.trim() || undefined,
      verifiedAt: form.verifiedAt.trim() || undefined,
      verifiedBy: form.verifiedBy.trim() || undefined,
      description: form.description.trim(),
      lifecycleStatus: form.lifecycleStatus,
      links: form.links.trim(),
      location: form.location.trim(),
      manufacturer: form.manufacturer.trim(),
      model: form.model.trim(),
      notes: form.notes.trim(),
      picturePath: form.picturePath.trim(),
      projectName: form.projectName.trim(),
      qty,
      serialNumber: form.serialNumber.trim(),
      workingStatus: form.workingStatus,
    },
  };
}

export function changedEntryInputFields(before: InventoryEntryInput, after: InventoryEntryInput): string[] {
  const fields: string[] = [];

  if (before.assetNumber !== after.assetNumber) {
    fields.push("assetNumber");
  }
  if (before.serialNumber !== after.serialNumber) {
    fields.push("serialNumber");
  }
  if (before.qty !== after.qty) {
    fields.push("qty");
  }
  if (before.manufacturer !== after.manufacturer) {
    fields.push("manufacturer");
  }
  if (before.model !== after.model) {
    fields.push("model");
  }
  if (before.description !== after.description) {
    fields.push("description");
  }
  if (before.projectName !== after.projectName) {
    fields.push("projectName");
  }
  if (before.location !== after.location) {
    fields.push("location");
  }
  if (before.assignedTo !== after.assignedTo) {
    fields.push("assignedTo");
  }
  if (before.links !== after.links) {
    fields.push("links");
  }
  if (before.notes !== after.notes) {
    fields.push("notes");
  }
  if (before.lifecycleStatus !== after.lifecycleStatus) {
    fields.push("lifecycleStatus");
  }
  if (before.workingStatus !== after.workingStatus) {
    fields.push("workingStatus");
  }
  if (before.condition !== after.condition) {
    fields.push("condition");
  }
  for (const field of ["calibrationRequirement", "outToCalibration", "lastCalibratedAt", "calibrationDueAt", "calibrationIntervalMonths", "certificateRef", "calibrationVendor", "calibrationNotes", "verifiedAt", "verifiedBy"] as const) {
    if (before[field] !== after[field]) fields.push(field);
  }
  if (before.archived !== after.archived) {
    fields.push("archived");
  }
  if ((before.picturePath ?? "") !== (after.picturePath ?? "")) {
    fields.push("picturePath");
  }

  return fields;
}

function hasIdentity(form: EntryFormState): boolean {
  return Boolean(
    form.assetNumber.trim() ||
      form.serialNumber.trim() ||
      form.manufacturer.trim() ||
      form.model.trim() ||
      form.description.trim(),
  );
}

export function updateForm<Key extends keyof EntryFormState>(
  setForm: Dispatch<SetStateAction<EntryFormState>>,
  key: Key,
  value: EntryFormState[Key],
): void {
  setForm((current) => ({ ...current, [key]: value }));
}

export function formatOptionLabel(option: string): string {
  if (option === "reference_only") return "Reference only";
  if (option === "not_required") return "Not required";
  return option.replaceAll("_", " ").replace(/\b\w/g, (character) => character.toUpperCase());
}

export function suggestCalibrationDueDate(lastCalibratedAt: string, intervalMonths: number): string | null {
  if (!isValidDateOnly(lastCalibratedAt) || !Number.isInteger(intervalMonths) || intervalMonths < 1 || intervalMonths > 1200) return null;
  const [year, month, day] = lastCalibratedAt.split("-").map(Number) as [number, number, number];
  const targetMonthIndex = month - 1 + intervalMonths;
  const targetYear = year + Math.floor(targetMonthIndex / 12);
  const targetMonth = ((targetMonthIndex % 12) + 12) % 12;
  const lastDay = new Date(Date.UTC(targetYear, targetMonth + 1, 0)).getUTCDate();
  return `${targetYear.toString().padStart(4, "0")}-${(targetMonth + 1).toString().padStart(2, "0")}-${Math.min(day, lastDay).toString().padStart(2, "0")}`;
}
