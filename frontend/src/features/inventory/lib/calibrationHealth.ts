import type { CalibrationHealth, CalibrationRequirement, InventoryEntry } from "@/features/inventory/types";

const DATE_ONLY_PATTERN = /^(\d{4})-(\d{2})-(\d{2})$/;
const MILLISECONDS_PER_DAY = 86_400_000;

export function deriveCalibrationHealth(
  entry: InventoryEntry,
  localDate: string,
  dueSoonDays = 30,
): CalibrationHealth | null {
  if (entry.archived) {
    return null;
  }
  if (entry.calibrationRequirement === "reference_only" || entry.calibrationRequirement === "not_required") {
    return "not_applicable";
  }
  if (entry.calibrationRequirement === "unknown") {
    return "unknown";
  }
  if (entry.outToCalibration) {
    return "out_to_cal";
  }

  const dueDay = parseDateOnlyToDay(entry.calibrationDueAt);
  if (dueDay === null) {
    return "missing_due";
  }
  const today = parseDateOnlyToDay(localDate);
  if (today === null) {
    throw new Error("localDate must be a valid date in YYYY-MM-DD format.");
  }
  if (dueDay < today) {
    return "overdue";
  }
  return dueDay <= today + Math.max(0, Math.trunc(dueSoonDays)) ? "due_soon" : "current";
}

export function isValidDateOnly(value: string): boolean {
  return parseDateOnlyToDay(value) !== null;
}

export function getLocalDateString(now = new Date()): string {
  const year = now.getFullYear().toString().padStart(4, "0");
  const month = (now.getMonth() + 1).toString().padStart(2, "0");
  const day = now.getDate().toString().padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function calibrationRequirementLabel(value: CalibrationRequirement): string {
  return value === "reference_only" ? "Reference only" : value === "not_required" ? "Not required" : value === "required" ? "Required" : "Unknown";
}

export function calibrationHealthLabel(value: CalibrationHealth): string {
  return ({ missing_due: "Missing due", overdue: "Overdue", due_soon: "Due soon", current: "Current", not_applicable: "Not applicable", unknown: "Unknown", out_to_cal: "Out to cal" } satisfies Record<CalibrationHealth, string>)[value];
}

function parseDateOnlyToDay(value: string | undefined): number | null {
  if (!value || !DATE_ONLY_PATTERN.test(value)) {
    return null;
  }
  const timestamp = Date.parse(`${value}T00:00:00.000Z`);
  if (!Number.isFinite(timestamp) || new Date(timestamp).toISOString().slice(0, 10) !== value) {
    return null;
  }
  return Math.trunc(timestamp / MILLISECONDS_PER_DAY);
}
