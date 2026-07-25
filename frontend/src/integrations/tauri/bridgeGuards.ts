import { APP_VERSION } from "@/app/branding";
import type { InventorySyncResult } from "@/integrations/tauri/desktop-bridge";
import {
  type CalibrationRosterAbsentEntry,
  type CalibrationRosterCandidateContext,
  type CalibrationRosterClassification,
  type CalibrationRosterCommitResult,
  type CalibrationRosterCounts,
  type CalibrationRosterFieldChange,
  type CalibrationRosterPreviewReport,
  type CalibrationRosterRowOutcome,
  type CalibrationRosterSemanticFlag,
  CALIBRATION_REQUIREMENT_OPTIONS,
  LIFECYCLE_OPTIONS,
  WORKING_STATUS_OPTIONS,
  type CalibrationRequirement,
  type InventoryCounts,
  type InventoryDeleteMutationResult,
  type InventoryEntry,
  type InventoryEntryInput,
  type InventoryEntryMutationResult,
  type InventoryQueryResult,
  type InventorySharedStatus,
  type ImportClassification,
  type ImportColumnReport,
  type ImportColumnTreatment,
  type ImportCommitResult,
  type ImportDryRunReport,
  type ImportProvenance,
  type ImportRowOutcome,
  type ExcelExportResult,
  type LifecycleStatus,
  type UpdateState,
  type UpdateStatus,
  type WorkingStatus,
} from "@/modules/te-test-equipment/types";
import { isValidDateOnly } from "@/modules/te-test-equipment/lib/calibrationHealth";

const IMPORT_CLASSIFICATIONS = new Set<ImportClassification>([
  "inserted", "matched", "conflicted", "rejected", "ignored",
]);
const IMPORT_COLUMN_TREATMENTS = new Set<ImportColumnTreatment>([
  "mapped", "intentionally_ignored", "unknown",
]);
const CALIBRATION_ROSTER_CLASSIFICATIONS = new Set<CalibrationRosterClassification>([
  "matched_update",
  "create_candidate",
  "conflict_review_required",
  "duplicate_source_row",
  "ignored_junk",
]);
const CALIBRATION_ROSTER_SEMANTIC_FLAGS = new Set<CalibrationRosterSemanticFlag>([
  "normal_dated",
  "reference_only",
  "needs_calibration",
  "out_to_calibration",
  "failed_calibration",
  "inactive_or_scrapped",
  "unclear",
]);
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

export function parseInventorySyncResult(value: unknown): InventorySyncResult {
  const record = requireRecord(value, "inventory sync payload");
  return {
    dbPath: optionalString(record.dbPath) ?? "",
    entries: requireArray(record.entries, "inventory entries").map(parseInventoryEntry),
    entriesChanged: optionalBoolean(record.entriesChanged),
    shared: parseSharedStatus(record.shared),
  };
}

export function parseNullableInventorySyncResult(value: unknown): InventorySyncResult | null {
  return value === null || value === undefined ? null : parseInventorySyncResult(value);
}

export function parseInventoryQueryResult(value: unknown): InventoryQueryResult {
  const record = requireRecord(value, "inventory query payload");
  return {
    counts: parseCounts(record.counts),
    dbPath: optionalString(record.dbPath) ?? "",
    entries: requireArray(record.entries, "inventory entries").map(parseInventoryEntry),
    shared: parseSharedStatus(record.shared),
    totalFiltered: optionalFiniteNumber(record.totalFiltered) ?? 0,
  };
}

export function parseEntryMutationResult(value: unknown): InventoryEntryMutationResult {
  const record = requireRecord(value, "inventory mutation payload");
  return {
    entry: parseInventoryEntry(record.entry),
    message: optionalString(record.message) ?? "Inventory entry saved.",
    mutationMode: record.mutationMode === "shared" ? "shared" : "local",
    shared: parseSharedStatus(record.shared),
  };
}

export function parseDeleteMutationResult(value: unknown): InventoryDeleteMutationResult {
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

export function parseExcelExportResult(value: unknown): ExcelExportResult {
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

export function parseImportDryRunReport(value: unknown): ImportDryRunReport {
  const record = requireRecord(value, "import dry-run report");
  const report: ImportDryRunReport = {
    batchId: requireString(record.batchId, "Invalid import dry-run report: batchId must be a string."),
    sourceFingerprint: requireString(record.sourceFingerprint, "Invalid import dry-run report: sourceFingerprint must be a string."),
    sourceFilename: requireString(record.sourceFilename, "Invalid import dry-run report: sourceFilename must be a string."),
    selectedSheet: requireString(record.selectedSheet, "Invalid import dry-run report: selectedSheet must be a string."),
    mappingVersion: requireString(record.mappingVersion, "Invalid import dry-run report: mappingVersion must be a string."),
    totalRows: requireNonnegativeInteger(record.totalRows, "Invalid import dry-run report: totalRows must be a nonnegative integer."),
    inserted: requireNonnegativeInteger(record.inserted, "Invalid import dry-run report: inserted must be a nonnegative integer."),
    matched: requireNonnegativeInteger(record.matched, "Invalid import dry-run report: matched must be a nonnegative integer."),
    conflicted: requireNonnegativeInteger(record.conflicted, "Invalid import dry-run report: conflicted must be a nonnegative integer."),
    rejected: requireNonnegativeInteger(record.rejected, "Invalid import dry-run report: rejected must be a nonnegative integer."),
    ignored: requireNonnegativeInteger(record.ignored, "Invalid import dry-run report: ignored must be a nonnegative integer."),
    columns: requireArray(record.columns, "import dry-run columns").map(parseImportColumnReport),
    rowOutcomes: requireArray(record.rowOutcomes, "import dry-run row outcomes").map(parseImportRowOutcome),
    blocking: requireBoolean(record.blocking, "Invalid import dry-run report: blocking must be a boolean."),
    reconciliationBasis: requireString(record.reconciliationBasis, "Invalid import dry-run report: reconciliationBasis must be a string."),
  };
  const total = report.inserted + report.matched + report.conflicted + report.rejected + report.ignored;
  if (report.totalRows !== total || report.rowOutcomes.length !== report.totalRows) {
    throw new Error("Invalid import dry-run report: totalRows must equal the five totals and row outcomes.");
  }
  const actualCounts: Record<ImportClassification, number> = {
    inserted: 0,
    matched: 0,
    conflicted: 0,
    rejected: 0,
    ignored: 0,
  };
  for (const row of report.rowOutcomes) actualCounts[row.classification] += 1;
  if (
    actualCounts.inserted !== report.inserted ||
    actualCounts.matched !== report.matched ||
    actualCounts.conflicted !== report.conflicted ||
    actualCounts.rejected !== report.rejected ||
    actualCounts.ignored !== report.ignored
  ) {
    throw new Error("Invalid import dry-run report: row classifications do not match the five totals.");
  }
  return report;
}

export function parseImportCommitResult(value: unknown): ImportCommitResult {
  const record = requireRecord(value, "import commit result");
  return {
    batchId: requireString(record.batchId, "Invalid import commit result: batchId must be a string."),
    inserted: requireNonnegativeInteger(record.inserted, "Invalid import commit result: inserted must be a nonnegative integer."),
    matched: requireNonnegativeInteger(record.matched, "Invalid import commit result: matched must be a nonnegative integer."),
    conflicted: requireNonnegativeInteger(record.conflicted, "Invalid import commit result: conflicted must be a nonnegative integer."),
    rejected: requireNonnegativeInteger(record.rejected, "Invalid import commit result: rejected must be a nonnegative integer."),
    ignored: requireNonnegativeInteger(record.ignored, "Invalid import commit result: ignored must be a nonnegative integer."),
    remaining: requireNonnegativeInteger(record.remaining, "Invalid import commit result: remaining must be a nonnegative integer."),
    noop: requireNonnegativeInteger(record.noop, "Invalid import commit result: noop must be a nonnegative integer."),
    entriesChanged: requireBoolean(record.entriesChanged, "Invalid import commit result: entriesChanged must be a boolean."),
    message: requireString(record.message, "Invalid import commit result: message must be a string."),
  };
}

export function parseCalibrationRosterPreviewReport(value: unknown): CalibrationRosterPreviewReport {
  const record = requireRecord(value, "calibration roster preview report");
  const counts = parseCalibrationRosterCounts(record.counts);
  const report: CalibrationRosterPreviewReport = {
    batchId: requireString(record.batchId, "Invalid calibration roster preview: batchId must be a string."),
    sourceFingerprint: requireString(
      record.sourceFingerprint,
      "Invalid calibration roster preview: sourceFingerprint must be a string.",
    ),
    sourceFilename: requireString(
      record.sourceFilename,
      "Invalid calibration roster preview: sourceFilename must be a string.",
    ),
    mappingVersion: requireString(
      record.mappingVersion,
      "Invalid calibration roster preview: mappingVersion must be a string.",
    ),
    contributingSheets: requireArray(record.contributingSheets, "calibration roster contributing sheets").map(
      (sheet) => requireString(sheet, "Invalid calibration roster preview: sheet names must be strings."),
    ),
    totalSourceRows: requireNonnegativeInteger(
      record.totalSourceRows,
      "Invalid calibration roster preview: totalSourceRows must be a nonnegative integer.",
    ),
    counts,
    rowOutcomes: requireArray(record.rowOutcomes, "calibration roster row outcomes").map(
      parseCalibrationRosterRowOutcome,
    ),
    currentRequiredAbsent: requireArray(
      record.currentRequiredAbsent,
      "calibration roster absent entries",
    ).map(parseCalibrationRosterAbsentEntry),
    prospectiveRequiredCount: requireNonnegativeInteger(
      record.prospectiveRequiredCount,
      "Invalid calibration roster preview: prospectiveRequiredCount must be a nonnegative integer.",
    ),
    reconciliationBasis: requireString(
      record.reconciliationBasis,
      "Invalid calibration roster preview: reconciliationBasis must be a string.",
    ),
    blocking: requireBoolean(
      record.blocking,
      "Invalid calibration roster preview: blocking must be a boolean.",
    ),
  };
  const classifiedTotal = counts.matchedUpdates
    + counts.createCandidates
    + counts.conflicts
    + counts.duplicateSourceRows
    + counts.ignoredJunk;
  if (classifiedTotal !== report.totalSourceRows || report.rowOutcomes.length !== report.totalSourceRows) {
    throw new Error("Invalid calibration roster preview: source-row totals are inconsistent.");
  }
  if (
    counts.currentRequiredAbsent !== report.currentRequiredAbsent.length
    || report.contributingSheets.length === 0
  ) {
    throw new Error("Invalid calibration roster preview: absent-entry or sheet totals are inconsistent.");
  }
  return report;
}

export function parseCalibrationRosterCommitResult(value: unknown): CalibrationRosterCommitResult {
  const record = requireRecord(value, "calibration roster commit result");
  return {
    batchId: requireString(record.batchId, "Invalid calibration roster commit result: batchId must be a string."),
    updated: requireNonnegativeInteger(record.updated, "Invalid calibration roster commit result: updated is invalid."),
    created: requireNonnegativeInteger(record.created, "Invalid calibration roster commit result: created is invalid."),
    reset: requireNonnegativeInteger(record.reset, "Invalid calibration roster commit result: reset is invalid."),
    ignored: requireNonnegativeInteger(record.ignored, "Invalid calibration roster commit result: ignored is invalid."),
    noop: requireNonnegativeInteger(record.noop, "Invalid calibration roster commit result: noop is invalid."),
    finalRequired: requireNonnegativeInteger(
      record.finalRequired,
      "Invalid calibration roster commit result: finalRequired is invalid.",
    ),
    entriesChanged: requireBoolean(
      record.entriesChanged,
      "Invalid calibration roster commit result: entriesChanged must be a boolean.",
    ),
    message: requireString(record.message, "Invalid calibration roster commit result: message must be a string."),
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

export function parseNonemptyString(value: unknown, label: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`Invalid ${label}: expected a nonempty string.`);
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

function parseInventoryEntry(value: unknown): InventoryEntry {
  const record = requireRecord(value, "inventory entry");
  const id = optionalString(record.id);
  if (!id) {
    throw new Error("Invalid inventory entry: missing id.");
  }

  const calibrationRequirement = parseCalibrationRequirement(record.calibrationRequirement);
  const outToCalibration = parseDefaultedBoolean(
    record.outToCalibration,
    false,
    "Invalid inventory entry: outToCalibration must be a boolean.",
  );
  const lastCalibratedAt = parseOptionalDateOnly(record.lastCalibratedAt, "lastCalibratedAt");
  const calibrationDueAt = parseOptionalDateOnly(record.calibrationDueAt, "calibrationDueAt");
  if (lastCalibratedAt && calibrationDueAt && calibrationDueAt < lastCalibratedAt) {
    throw new Error("Invalid inventory entry: calibrationDueAt cannot be before lastCalibratedAt.");
  }

  return {
    id,
    archived: record.archived === true,
    assetNumber: optionalString(record.assetNumber) ?? "",
    assignedTo: optionalString(record.assignedTo) ?? "",
    calibrationDueAt,
    calibrationIntervalMonths: parseOptionalPositiveInteger(record.calibrationIntervalMonths, "calibrationIntervalMonths", 1_200),
    calibrationNotes: parseOptionalString(record.calibrationNotes, "calibrationNotes"),
    calibrationRequirement,
    calibrationVendor: parseOptionalString(record.calibrationVendor, "calibrationVendor"),
    certificateRef: parseOptionalString(record.certificateRef, "certificateRef"),
    condition: optionalString(record.condition) ?? "",
    createdAt: optionalString(record.createdAt) ?? "",
    databaseId: optionalFiniteNumber(record.databaseId),
    description: optionalString(record.description) ?? "",
    entryUuid: optionalString(record.entryUuid) ?? "",
    importProvenance: parseImportProvenance(record.importProvenance),
    lastCalibratedAt,
    lifecycleStatus: parseLifecycleStatus(record.lifecycleStatus),
    links: optionalString(record.links) ?? "",
    location: optionalString(record.location) ?? "",
    manufacturer: optionalString(record.manufacturer) ?? "",
    manualEntry: record.manualEntry === true,
    model: optionalString(record.model) ?? "",
    notes: optionalString(record.notes) ?? "",
    outToCalibration,
    picturePath: optionalString(record.picturePath) ?? "",
    projectName: optionalString(record.projectName) ?? "",
    qty: parseNullableFiniteNumber(record.qty),
    serialNumber: optionalString(record.serialNumber) ?? "",
    updatedAt: optionalString(record.updatedAt) ?? "",
    verifiedAt: parseOptionalRfc3339(record.verifiedAt, "verifiedAt"),
    verifiedBy: parseOptionalString(record.verifiedBy, "verifiedBy"),
    workingStatus: parseWorkingStatus(record.workingStatus),
  };
}

function parseImportColumnReport(value: unknown): ImportColumnReport {
  const record = requireRecord(value, "import dry-run column");
  if (!IMPORT_COLUMN_TREATMENTS.has(record.treatment as ImportColumnTreatment)) {
    throw new Error("Invalid import dry-run report: column treatment is invalid.");
  }
  return {
    originalHeader: requireString(record.originalHeader, "Invalid import dry-run report: column header must be a string."),
    normalizedTarget: parseNullableStrictString(record.normalizedTarget, "Invalid import dry-run report: normalizedTarget must be a string or null."),
    treatment: record.treatment as ImportColumnTreatment,
    nonblankCount: requireNonnegativeInteger(record.nonblankCount, "Invalid import dry-run report: nonblankCount must be a nonnegative integer."),
    reason: requireString(record.reason, "Invalid import dry-run report: column reason must be a string."),
  };
}

function parseImportRowOutcome(value: unknown): ImportRowOutcome {
  const record = requireRecord(value, "import dry-run row outcome");
  if (!IMPORT_CLASSIFICATIONS.has(record.classification as ImportClassification)) {
    throw new Error("Invalid import dry-run report: row classification is invalid.");
  }
  const issues = requireArray(record.issues, "import dry-run row issues").map((issue) =>
    requireString(issue, "Invalid import dry-run report: row issues must be strings."),
  );
  const rawRecord = requireRecord(record.rawValues, "import dry-run raw values");
  const rawValues: Record<string, string> = {};
  for (const [key, rawValue] of Object.entries(rawRecord)) {
    rawValues[key] = requireString(rawValue, "Invalid import dry-run report: raw values must be strings.");
  }
  return {
    sourceRow: requirePositiveInteger(record.sourceRow, "Invalid import dry-run report: sourceRow must be a positive integer."),
    classification: record.classification as ImportClassification,
    issues,
    originalId: parseNullableStrictString(record.originalId, "Invalid import dry-run report: originalId must be a string or null."),
    originalAssetNumber: parseNullableStrictString(record.originalAssetNumber, "Invalid import dry-run report: originalAssetNumber must be a string or null."),
    originalSerialNumber: parseNullableStrictString(record.originalSerialNumber, "Invalid import dry-run report: originalSerialNumber must be a string or null."),
    candidateEntryUuid: parseNullableStrictString(record.candidateEntryUuid, "Invalid import dry-run report: candidateEntryUuid must be a string or null."),
    rawValues,
  };
}

function parseCalibrationRosterCounts(value: unknown): CalibrationRosterCounts {
  const record = requireRecord(value, "calibration roster counts");
  return {
    matchedUpdates: requireNonnegativeInteger(record.matchedUpdates, "Invalid calibration roster counts: matchedUpdates is invalid."),
    createCandidates: requireNonnegativeInteger(record.createCandidates, "Invalid calibration roster counts: createCandidates is invalid."),
    conflicts: requireNonnegativeInteger(record.conflicts, "Invalid calibration roster counts: conflicts is invalid."),
    duplicateSourceRows: requireNonnegativeInteger(
      record.duplicateSourceRows,
      "Invalid calibration roster counts: duplicateSourceRows is invalid.",
    ),
    ignoredJunk: requireNonnegativeInteger(record.ignoredJunk, "Invalid calibration roster counts: ignoredJunk is invalid."),
    currentRequiredAbsent: requireNonnegativeInteger(
      record.currentRequiredAbsent,
      "Invalid calibration roster counts: currentRequiredAbsent is invalid.",
    ),
  };
}

function parseCalibrationRosterRowOutcome(value: unknown): CalibrationRosterRowOutcome {
  const record = requireRecord(value, "calibration roster row outcome");
  if (!CALIBRATION_ROSTER_CLASSIFICATIONS.has(record.classification as CalibrationRosterClassification)) {
    throw new Error("Invalid calibration roster row outcome: classification is invalid.");
  }
  const semanticFlags = requireArray(record.semanticFlags, "calibration roster semantic flags").map((flag) => {
    if (!CALIBRATION_ROSTER_SEMANTIC_FLAGS.has(flag as CalibrationRosterSemanticFlag)) {
      throw new Error("Invalid calibration roster row outcome: semantic flag is invalid.");
    }
    return flag as CalibrationRosterSemanticFlag;
  });
  return {
    sourceSheet: requireString(record.sourceSheet, "Invalid calibration roster row outcome: sourceSheet is invalid."),
    sourceRow: requirePositiveInteger(record.sourceRow, "Invalid calibration roster row outcome: sourceRow is invalid."),
    classification: record.classification as CalibrationRosterClassification,
    issues: requireArray(record.issues, "calibration roster row issues").map((issue) =>
      requireString(issue, "Invalid calibration roster row outcome: issues must be strings."),
    ),
    ignoredIdentityPlaceholders: requireArray(
      record.ignoredIdentityPlaceholders,
      "calibration roster ignored identity placeholders",
    ).map((issue) => requireString(issue, "Invalid calibration roster row outcome: placeholder issues must be strings.")),
    assetNumber: parseNullableStrictString(record.assetNumber, "Invalid calibration roster row outcome: assetNumber is invalid."),
    serialNumber: parseNullableStrictString(record.serialNumber, "Invalid calibration roster row outcome: serialNumber is invalid."),
    manufacturer: parseNullableStrictString(record.manufacturer, "Invalid calibration roster row outcome: manufacturer is invalid."),
    model: parseNullableStrictString(record.model, "Invalid calibration roster row outcome: model is invalid."),
    description: parseNullableStrictString(record.description, "Invalid calibration roster row outcome: description is invalid."),
    candidateEntryUuid: parseNullableStrictString(
      record.candidateEntryUuid,
      "Invalid calibration roster row outcome: candidateEntryUuid is invalid.",
    ),
    candidateEntries: requireArray(record.candidateEntries, "calibration roster candidate entries").map(
      parseCalibrationRosterCandidate,
    ),
    proposedRequirement: parseCalibrationRequirement(record.proposedRequirement),
    proposedOutToCalibration: requireBoolean(
      record.proposedOutToCalibration,
      "Invalid calibration roster row outcome: proposedOutToCalibration is invalid.",
    ),
    proposedInput: record.proposedInput === null || record.proposedInput === undefined
      ? null
      : parseCalibrationRosterEntryInput(record.proposedInput),
    changes: requireArray(record.changes, "calibration roster field changes").map(parseCalibrationRosterFieldChange),
    semanticFlags,
    requiresReview: requireBoolean(
      record.requiresReview,
      "Invalid calibration roster row outcome: requiresReview must be a boolean.",
    ),
  };
}

function parseCalibrationRosterCandidate(value: unknown): CalibrationRosterCandidateContext {
  const record = requireRecord(value, "calibration roster candidate");
  return {
    entryUuid: requireString(record.entryUuid, "Invalid calibration roster candidate: entryUuid is invalid."),
    id: requireString(record.id, "Invalid calibration roster candidate: id is invalid."),
    assetNumber: requireString(record.assetNumber, "Invalid calibration roster candidate: assetNumber is invalid."),
    serialNumber: requireString(record.serialNumber, "Invalid calibration roster candidate: serialNumber is invalid."),
    manufacturer: requireString(record.manufacturer, "Invalid calibration roster candidate: manufacturer is invalid."),
    model: requireString(record.model, "Invalid calibration roster candidate: model is invalid."),
    description: requireString(record.description, "Invalid calibration roster candidate: description is invalid."),
  };
}

function parseCalibrationRosterFieldChange(value: unknown): CalibrationRosterFieldChange {
  const record = requireRecord(value, "calibration roster field change");
  return {
    field: requireString(record.field, "Invalid calibration roster field change: field is invalid."),
    before: parseNullableStrictString(record.before, "Invalid calibration roster field change: before is invalid."),
    after: parseNullableStrictString(record.after, "Invalid calibration roster field change: after is invalid."),
    destructive: requireBoolean(
      record.destructive,
      "Invalid calibration roster field change: destructive must be a boolean.",
    ),
  };
}

function parseCalibrationRosterAbsentEntry(value: unknown): CalibrationRosterAbsentEntry {
  const record = requireRecord(value, "calibration roster absent entry");
  return {
    entry: parseCalibrationRosterCandidate(record.entry),
    calibrationDueAt: parseNullableStrictString(
      record.calibrationDueAt,
      "Invalid calibration roster absent entry: calibrationDueAt is invalid.",
    ),
    calibrationVendor: parseNullableStrictString(
      record.calibrationVendor,
      "Invalid calibration roster absent entry: calibrationVendor is invalid.",
    ),
    calibrationNotes: parseNullableStrictString(
      record.calibrationNotes,
      "Invalid calibration roster absent entry: calibrationNotes is invalid.",
    ),
  };
}

function parseCalibrationRosterEntryInput(value: unknown): InventoryEntryInput {
  const record = requireRecord(value, "calibration roster proposed input");
  const lastCalibratedAt = parseOptionalDateOnly(record.lastCalibratedAt, "lastCalibratedAt");
  const calibrationDueAt = parseOptionalDateOnly(record.calibrationDueAt, "calibrationDueAt");
  if (lastCalibratedAt && calibrationDueAt && calibrationDueAt < lastCalibratedAt) {
    throw new Error("Invalid calibration roster proposed input: due date is before last calibration date.");
  }
  return {
    assetNumber: optionalString(record.assetNumber) ?? "",
    serialNumber: optionalString(record.serialNumber) ?? "",
    qty: parseNullableFiniteNumber(record.qty),
    manufacturer: optionalString(record.manufacturer) ?? "",
    model: optionalString(record.model) ?? "",
    description: optionalString(record.description) ?? "",
    projectName: optionalString(record.projectName) ?? "",
    location: optionalString(record.location) ?? "",
    assignedTo: optionalString(record.assignedTo) ?? "",
    links: optionalString(record.links) ?? "",
    notes: optionalString(record.notes) ?? "",
    lifecycleStatus: parseLifecycleStatus(record.lifecycleStatus),
    workingStatus: parseWorkingStatus(record.workingStatus),
    condition: optionalString(record.condition) ?? "",
    calibrationRequirement: parseCalibrationRequirement(record.calibrationRequirement),
    outToCalibration: parseDefaultedBoolean(
      record.outToCalibration,
      false,
      "Invalid calibration roster proposed input: outToCalibration is invalid.",
    ),
    lastCalibratedAt,
    calibrationDueAt,
    calibrationIntervalMonths: parseOptionalPositiveInteger(
      record.calibrationIntervalMonths,
      "calibrationIntervalMonths",
      1_200,
    ),
    certificateRef: parseOptionalString(record.certificateRef, "certificateRef"),
    calibrationVendor: parseOptionalString(record.calibrationVendor, "calibrationVendor"),
    calibrationNotes: parseOptionalString(record.calibrationNotes, "calibrationNotes"),
    verifiedAt: parseOptionalRfc3339(record.verifiedAt, "verifiedAt"),
    verifiedBy: parseOptionalString(record.verifiedBy, "verifiedBy"),
    archived: parseDefaultedBoolean(
      record.archived,
      false,
      "Invalid calibration roster proposed input: archived is invalid.",
    ),
    picturePath: parseOptionalString(record.picturePath, "picturePath"),
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
    dueSoon: optionalFiniteNumber(record.dueSoon) ?? 0,
    inventory: optionalFiniteNumber(record.inventory) ?? 0,
    missingDue: optionalFiniteNumber(record.missingDue) ?? 0,
    outToCal: optionalFiniteNumber(record.outToCal) ?? 0,
    overdue: optionalFiniteNumber(record.overdue) ?? 0,
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

function parseCalibrationRequirement(value: unknown): CalibrationRequirement {
  if (value === undefined) return "unknown";
  if (!CALIBRATION_REQUIREMENT_OPTIONS.includes(value as CalibrationRequirement)) {
    throw new Error("Invalid inventory entry: calibrationRequirement is invalid.");
  }
  return value as CalibrationRequirement;
}

function parseImportProvenance(value: unknown): ImportProvenance | undefined {
  if (value === undefined || value === null) return undefined;
  const record = requireRecord(value, "inventory entry import provenance");
  return {
    batchId: requireString(record.batchId, "Invalid inventory entry: import provenance batchId must be a string."),
    sourceFilename: requireString(record.sourceFilename, "Invalid inventory entry: import provenance sourceFilename must be a string."),
    sourceSheet: parseOptionalString(record.sourceSheet, "importProvenance.sourceSheet"),
    sourceRow: requirePositiveInteger(record.sourceRow, "Invalid inventory entry: import provenance sourceRow must be a positive integer."),
    originalId: parseOptionalString(record.originalId, "importProvenance.originalId"),
    originalAssetNumber: parseOptionalString(record.originalAssetNumber, "importProvenance.originalAssetNumber"),
    originalSerialNumber: parseOptionalString(record.originalSerialNumber, "importProvenance.originalSerialNumber"),
  };
}

function parseOptionalDateOnly(value: unknown, field: string): string | undefined {
  const parsed = parseOptionalString(value, field);
  if (parsed !== undefined && !isValidDateOnly(parsed)) {
    throw new Error(`Invalid inventory entry: ${field} must be a valid YYYY-MM-DD date.`);
  }
  return parsed;
}

function parseOptionalRfc3339(value: unknown, field: string): string | undefined {
  const parsed = parseOptionalString(value, field);
  if (parsed === undefined) return undefined;
  const match = /^(\d{4}-\d{2}-\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})$/.exec(parsed);
  if (
    !match ||
    !isValidDateOnly(match[1]!) ||
    Number(match[2]) > 23 ||
    Number(match[3]) > 59 ||
    Number(match[4]) > 59 ||
    !Number.isFinite(Date.parse(parsed))
  ) {
    throw new Error(`Invalid inventory entry: ${field} must be an RFC 3339 timestamp.`);
  }
  return parsed;
}

function parseOptionalPositiveInteger(value: unknown, field: string, maximum: number): number | undefined {
  if (value === undefined || value === null) return undefined;
  if (!Number.isInteger(value) || (value as number) <= 0 || (value as number) > maximum) {
    throw new Error(`Invalid inventory entry: ${field} must be a positive integer.`);
  }
  return value as number;
}

function parseDefaultedBoolean(value: unknown, fallback: boolean, message: string): boolean {
  if (value === undefined) return fallback;
  return requireBoolean(value, message);
}

function parseOptionalString(value: unknown, field: string): string | undefined {
  if (value === undefined || value === null) return undefined;
  if (typeof value !== "string") {
    throw new Error(`Invalid inventory entry: ${field} must be a string.`);
  }
  return value;
}

function parseNullableStrictString(value: unknown, message: string): string | null {
  if (value === undefined || value === null) return null;
  return requireString(value, message);
}

function requireString(value: unknown, message: string): string {
  if (typeof value !== "string") throw new Error(message);
  return value;
}

function requireBoolean(value: unknown, message: string): boolean {
  if (typeof value !== "boolean") throw new Error(message);
  return value;
}

function requireNonnegativeInteger(value: unknown, message: string): number {
  if (!Number.isInteger(value) || (value as number) < 0) throw new Error(message);
  return value as number;
}

function requirePositiveInteger(value: unknown, message: string): number {
  const parsed = requireNonnegativeInteger(value, message);
  if (parsed === 0) throw new Error(message);
  return parsed;
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
