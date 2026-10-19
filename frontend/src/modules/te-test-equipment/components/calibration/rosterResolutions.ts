import type {
  CalibrationRosterPreviewReport,
  CalibrationRosterResolution,
  CalibrationRosterResolutionAction,
  CalibrationRosterRowOutcome,
  InventoryEntryInput,
} from "@/modules/te-test-equipment/types";

/** What the person has chosen so far for one roster row that needs review. */
export interface ResolutionDraft {
  action: CalibrationRosterResolutionAction | "";
  confirmed: boolean;
  createInput?: InventoryEntryInput;
  targetEntryUuid?: string;
}

export function defaultResolutionDrafts(report: CalibrationRosterPreviewReport): Record<string, ResolutionDraft> {
  return Object.fromEntries(
    report.rowOutcomes
      .filter((row) => row.requiresReview)
      .map((row) => [rowKey(row), defaultResolutionDraft(row)]),
  );
}

export function defaultResolutionDraft(row: CalibrationRosterRowOutcome): ResolutionDraft {
  if (row.classification === "ignored_junk") {
    return { action: "ignore", confirmed: false };
  }
  if (row.classification === "create_candidate") {
    if (row.candidateEntries.length > 0) {
      return {
        action: "",
        confirmed: false,
        createInput: row.proposedInput ?? undefined,
        targetEntryUuid: row.candidateEntries[0]?.entryUuid,
      };
    }
    return { action: "create", confirmed: false, createInput: row.proposedInput ?? undefined };
  }
  if (row.classification === "matched_update") {
    return {
      action: "use_existing",
      confirmed: false,
      targetEntryUuid: row.candidateEntryUuid ?? row.candidateEntries[0]?.entryUuid,
    };
  }
  return {
    action: "",
    confirmed: false,
    createInput: row.proposedInput ?? undefined,
    targetEntryUuid: row.candidateEntryUuid ?? row.candidateEntries[0]?.entryUuid,
  };
}

export function resolutionActionOptions(row: CalibrationRosterRowOutcome) {
  if (row.classification === "ignored_junk") {
    return [{ value: "ignore", label: "Ignore label / junk row" }];
  }
  const options: Array<{ value: string; label: string }> = [];
  if (row.candidateEntries.length > 0) {
    options.push({ value: "use_existing", label: "Use existing equipment" });
  }
  if (row.proposedInput) {
    options.push({ value: "create", label: "Create reviewed equipment" });
  }
  options.push({ value: "ignore", label: "Ignore source row" });
  return options;
}

export function resolutionComplete(row: CalibrationRosterRowOutcome, draft: ResolutionDraft | undefined): boolean {
  return Boolean(draft?.confirmed && resolutionActionComplete(row, draft));
}

export function resolutionActionComplete(row: CalibrationRosterRowOutcome, draft: ResolutionDraft): boolean {
  if (draft.action === "ignore") {
    return true;
  }
  if (draft.action === "use_existing") {
    return Boolean(
      draft.targetEntryUuid
        && row.candidateEntries.some((candidate) => candidate.entryUuid === draft.targetEntryUuid),
    );
  }
  if (draft.action === "create") {
    const input = draft.createInput;
    return Boolean(input && [input.assetNumber, input.serialNumber, input.manufacturer, input.model, input.description]
      .some((value) => value.trim()));
  }
  return false;
}

export function buildResolutions(
  rows: CalibrationRosterRowOutcome[],
  drafts: Record<string, ResolutionDraft>,
): CalibrationRosterResolution[] {
  return rows.map((row) => {
    const draft = drafts[rowKey(row)];
    if (!draft || !draft.action || !draft.confirmed) {
      throw new Error(`Calibration roster row ${row.sourceRow} in ${row.sourceSheet} is unresolved.`);
    }
    return {
      sourceSheet: row.sourceSheet,
      sourceRow: row.sourceRow,
      action: draft.action,
      targetEntryUuid: draft.action === "use_existing" ? draft.targetEntryUuid : undefined,
      createInput: draft.action === "create" ? draft.createInput : undefined,
      confirmed: true,
    };
  });
}

export function rowKey(row: Pick<CalibrationRosterRowOutcome, "sourceRow" | "sourceSheet">): string {
  return `${row.sourceSheet}\u0000${row.sourceRow}`;
}

export function identityLabel(assetNumber: string | null, serialNumber: string | null): string {
  return [assetNumber, serialNumber].filter(Boolean).join(" / ") || "No usable asset or serial";
}

export function candidateModel(candidate: { manufacturer?: string | null; model?: string | null }): string {
  return [candidate.manufacturer, candidate.model].filter(Boolean).join(" ") || "Equipment";
}

export function classificationVariant(
  classification: CalibrationRosterRowOutcome["classification"],
): "error" | "outline" | "secondary" | "success" | "warning" {
  if (classification === "matched_update") return "secondary";
  if (classification === "create_candidate") return "success";
  if (classification === "ignored_junk") return "warning";
  return "error";
}

export function formatToken(value: string): string {
  return value
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replaceAll("_", " ")
    .replace(/^./, (character) => character.toUpperCase());
}
