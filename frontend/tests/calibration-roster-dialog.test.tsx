import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { CalibrationRosterDialog } from "@/modules/te-test-equipment/components/calibration/CalibrationRosterDialog";
import type {
  CalibrationRosterCommitResult,
  CalibrationRosterPreviewReport,
  CalibrationRosterRowOutcome,
  InventoryEntryInput,
} from "@/modules/te-test-equipment/types";

describe("CalibrationRosterDialog", () => {
  beforeEach(() => {
    delete window.inventoryDesktop;
  });

  it("defaults four reviewed creates and two junk rows, then commits explicit confirmations", async () => {
    const user = userEvent.setup();
    const report = createAndIgnoreReport();
    const committed: CalibrationRosterCommitResult = {
      batchId: report.batchId,
      updated: 0,
      created: 4,
      reset: 3,
      ignored: 2,
      noop: 0,
      finalRequired: 4,
      entriesChanged: true,
      message: "Calibration roster applied.",
    };
    const commitCalibrationRoster = vi.fn().mockResolvedValue(committed);
    const onCommitted = vi.fn().mockResolvedValue(undefined);
    window.inventoryDesktop = desktopBridge(report, commitCalibrationRoster);

    render(<CalibrationRosterDialog onClose={vi.fn()} onCommitted={onCommitted} />);
    await user.click(screen.getByRole("button", { name: "Choose Calibration Workbook" }));

    const summary = await screen.findByLabelText("Calibration roster preview summary");
    expect(summary).toHaveTextContent("Create candidates");
    expect(summary).toHaveTextContent("Ignored junk");
    expect(screen.getAllByRole("checkbox", { name: /Confirm resolution for/i })).toHaveLength(6);
    expect(screen.getAllByRole("button", { name: /Resolution for/i }).slice(0, 4)).toEqual(
      expect.arrayContaining([expect.objectContaining({ textContent: "Create reviewed equipment" })]),
    );

    const descriptions = screen.getAllByLabelText("Create description");
    await user.clear(descriptions[0]!);
    await user.type(descriptions[0]!, "Reviewed create one");
    for (const checkbox of screen.getAllByRole("checkbox", { name: /Confirm resolution for/i })) {
      await user.click(checkbox);
    }
    await user.click(screen.getByRole("checkbox", { name: /replaces the active required calibration roster/i }));
    await user.click(screen.getByRole("checkbox", { name: /re-verified at the actual commit time/i }));

    const commitButton = screen.getByRole("button", { name: "Commit Calibration Roster" });
    expect(commitButton).toBeEnabled();
    await user.click(commitButton);

    expect(commitCalibrationRoster).toHaveBeenCalledWith(
      "te-test-equipment",
      expect.objectContaining({
        batchId: report.batchId,
        confirmed: true,
        replaceActiveRequiredRoster: true,
        verificationAttribution: expect.stringContaining("Calibration roster cutover"),
        resolutions: expect.arrayContaining([
          expect.objectContaining({
            action: "create",
            confirmed: true,
            createInput: expect.objectContaining({ description: "Reviewed create one" }),
          }),
          expect.objectContaining({ action: "ignore", confirmed: true }),
        ]),
      }),
    );
    const commitInput = commitCalibrationRoster.mock.calls[0]?.[1];
    expect(commitInput.resolutions.filter((resolution: { action: string }) => resolution.action === "create")).toHaveLength(4);
    expect(commitInput.resolutions.filter((resolution: { action: string }) => resolution.action === "ignore")).toHaveLength(2);
    expect(onCommitted).toHaveBeenCalledWith(committed);
    expect(await screen.findByLabelText("Calibration roster commit result")).toHaveTextContent("Final active required roster: 4");
  });

  it("blocks an unresolved conflict until a candidate target is selected and confirmed", async () => {
    const user = userEvent.setup();
    const report = conflictReport();
    const commitCalibrationRoster = vi.fn().mockResolvedValue({
      batchId: report.batchId,
      updated: 1,
      created: 0,
      reset: 0,
      ignored: 0,
      noop: 0,
      finalRequired: 1,
      entriesChanged: true,
      message: "Conflict resolved.",
    } satisfies CalibrationRosterCommitResult);
    window.inventoryDesktop = desktopBridge(report, commitCalibrationRoster);

    render(<CalibrationRosterDialog onClose={vi.fn()} onCommitted={vi.fn()} />);
    await user.click(screen.getByRole("button", { name: "Choose Calibration Workbook" }));

    const commitButton = await screen.findByRole("button", { name: "Commit Calibration Roster" });
    expect(commitButton).toBeDisabled();
    await user.click(screen.getByRole("button", { name: "Resolution for October row 12" }));
    await user.click(screen.getByRole("option", { name: "Use existing equipment" }));
    await user.click(screen.getByRole("button", { name: "Target equipment for October row 12" }));
    await user.click(screen.getByRole("option", { name: /A-2 \/ S-2 — Second candidate/i }));
    await user.click(screen.getByRole("checkbox", { name: "Confirm resolution for October row 12" }));
    await user.click(screen.getByRole("checkbox", { name: /replaces the active required calibration roster/i }));
    await user.click(screen.getByRole("checkbox", { name: /re-verified at the actual commit time/i }));
    await user.click(commitButton);

    expect(commitCalibrationRoster).toHaveBeenCalledWith(
      "te-test-equipment",
      expect.objectContaining({
        resolutions: [expect.objectContaining({
          action: "use_existing",
          sourceRow: 12,
          targetEntryUuid: "entry-2",
        })],
      }),
    );
  });

  it("requires an explicit resolution for create candidates with review hints", async () => {
    const user = userEvent.setup();
    const report = hintedCreateReport();
    const committed: CalibrationRosterCommitResult = {
      batchId: report.batchId,
      updated: 1,
      created: 0,
      reset: 0,
      ignored: 0,
      noop: 0,
      finalRequired: 1,
      entriesChanged: true,
      message: "Hinted create resolved to existing equipment.",
    };
    const commitCalibrationRoster = vi.fn().mockResolvedValue(committed);
    window.inventoryDesktop = desktopBridge(report, commitCalibrationRoster);

    render(<CalibrationRosterDialog onClose={vi.fn()} onCommitted={vi.fn()} />);
    await user.click(screen.getByRole("button", { name: "Choose Calibration Workbook" }));

    const resolution = await screen.findByRole("button", { name: "Resolution for October row 15" });
    const confirmation = screen.getByRole("checkbox", { name: "Confirm resolution for October row 15" });
    const commitButton = screen.getByRole("button", { name: "Commit Calibration Roster" });
    expect(resolution).toHaveTextContent("Choose resolution");
    expect(confirmation).toBeDisabled();
    expect(commitButton).toBeDisabled();

    await user.click(resolution);
    await user.click(screen.getByRole("option", { name: "Use existing equipment" }));
    expect(screen.getByRole("button", { name: "Target equipment for October row 15" })).toHaveTextContent(
      "HINT-1 / HINT-S — Existing hinted equipment",
    );
    await user.click(confirmation);
    await user.click(screen.getByRole("checkbox", { name: /replaces the active required calibration roster/i }));
    await user.click(screen.getByRole("checkbox", { name: /re-verified at the actual commit time/i }));
    expect(commitButton).toBeEnabled();
    await user.click(commitButton);

    expect(commitCalibrationRoster).toHaveBeenCalledWith(
      "te-test-equipment",
      expect.objectContaining({
        resolutions: [expect.objectContaining({
          action: "use_existing",
          sourceRow: 15,
          targetEntryUuid: "entry-hint",
        })],
      }),
    );
  });

  it("summarizes every reconciliation classification and app-only reset group", async () => {
    const user = userEvent.setup();
    const report = allClassificationsReport();
    window.inventoryDesktop = desktopBridge(report, vi.fn());

    render(<CalibrationRosterDialog onClose={vi.fn()} onCommitted={vi.fn()} />);
    await user.click(screen.getByRole("button", { name: "Choose Calibration Workbook" }));

    const summary = await screen.findByLabelText("Calibration roster preview summary");
    for (const label of [
      "Matched updates",
      "Create candidates",
      "Conflicts",
      "Duplicate source rows",
      "Ignored junk",
      "Current required absent",
    ]) {
      expect(within(summary).getByText(label).parentElement).toHaveTextContent("1");
    }
    expect(screen.getByRole("heading", { name: "Required row review" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Semantic exceptions already resolved" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Current required equipment absent from workbook" })).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Commit Calibration Roster" })).toBeDisabled();
  });
});

function desktopBridge(
  report: CalibrationRosterPreviewReport,
  commitCalibrationRoster: ReturnType<typeof vi.fn>,
): NonNullable<Window["inventoryDesktop"]> {
  return {
    isDesktop: true,
    activateInventorySync: vi.fn().mockResolvedValue("roster-session"),
    loadInventory: vi.fn(),
    syncInventory: vi.fn(),
    deactivateInventorySync: vi.fn().mockResolvedValue(true),
    toggleVerifiedEntry: vi.fn(),
    createEntry: vi.fn(),
    updateEntry: vi.fn(),
    setArchivedEntry: vi.fn(),
    deleteEntry: vi.fn(),
    pickImportFile: vi.fn().mockResolvedValue(null),
    previewImport: vi.fn(),
    commitImport: vi.fn(),
    pickCalibrationRosterFile: vi.fn().mockResolvedValue("C:/calibration.xlsx"),
    previewCalibrationRoster: vi.fn().mockResolvedValue(report),
    commitCalibrationRoster: commitCalibrationRoster as NonNullable<
      NonNullable<Window["inventoryDesktop"]>["commitCalibrationRoster"]
    >,
  };
}

function createAndIgnoreReport(): CalibrationRosterPreviewReport {
  const createRows = Array.from({ length: 4 }, (_, index) => createRow(index + 2, index + 1));
  const junkRows = [junkRow(6), junkRow(7)];
  return {
    batchId: "calibration-batch-dialog",
    sourceFingerprint: "sha256-dialog",
    sourceFilename: "TE Lab Equip Calibration Data for New List.xlsx",
    mappingVersion: "te-calibration-roster-v1",
    contributingSheets: ["May", "October"],
    totalSourceRows: 6,
    counts: {
      matchedUpdates: 0,
      createCandidates: 4,
      conflicts: 0,
      duplicateSourceRows: 0,
      ignoredJunk: 2,
      currentRequiredAbsent: 3,
    },
    rowOutcomes: [...createRows, ...junkRows],
    currentRequiredAbsent: [
      {
        entry: candidate("absent-1", "APP-1", "APP-S", "App-only required"),
        calibrationDueAt: "2026-08-01",
        calibrationVendor: "Vendor",
        calibrationNotes: "Preserved",
      },
      {
        entry: candidate("absent-2", "APP-2", "APP-S2", "Second app-only required"),
        calibrationDueAt: null,
        calibrationVendor: null,
        calibrationNotes: null,
      },
      {
        entry: candidate("absent-3", "APP-3", "APP-S3", "Third app-only required"),
        calibrationDueAt: "2027-01-01",
        calibrationVendor: null,
        calibrationNotes: null,
      },
    ],
    prospectiveRequiredCount: 4,
    reconciliationBasis: "basis-dialog",
    blocking: true,
  };
}

function conflictReport(): CalibrationRosterPreviewReport {
  const row: CalibrationRosterRowOutcome = {
    sourceSheet: "October",
    sourceRow: 12,
    classification: "conflict_review_required",
    issues: ["Asset and serial resolve to different entries."],
    ignoredIdentityPlaceholders: [],
    assetNumber: "A-1",
    serialNumber: "S-2",
    manufacturer: "Maker",
    model: "Model",
    description: "Conflicted equipment",
    candidateEntryUuid: null,
    candidateEntries: [
      candidate("entry-1", "A-1", "S-1", "First candidate"),
      candidate("entry-2", "A-2", "S-2", "Second candidate"),
    ],
    proposedRequirement: "required",
    proposedOutToCalibration: false,
    proposedInput: entryInput("A-1", "S-2", "Conflicted equipment"),
    changes: [],
    semanticFlags: ["normal_dated"],
    requiresReview: true,
  };
  return {
    batchId: "calibration-batch-conflict",
    sourceFingerprint: "sha256-conflict",
    sourceFilename: "calibration.xlsx",
    mappingVersion: "te-calibration-roster-v1",
    contributingSheets: ["October"],
    totalSourceRows: 1,
    counts: {
      matchedUpdates: 0,
      createCandidates: 0,
      conflicts: 1,
      duplicateSourceRows: 0,
      ignoredJunk: 0,
      currentRequiredAbsent: 0,
    },
    rowOutcomes: [row],
    currentRequiredAbsent: [],
    prospectiveRequiredCount: 1,
    reconciliationBasis: "basis-conflict",
    blocking: true,
  };
}

function hintedCreateReport(): CalibrationRosterPreviewReport {
  const hintedRow: CalibrationRosterRowOutcome = {
    ...createRow(15, 1),
    sourceSheet: "October",
    sourceRow: 15,
    issues: ["Manufacturer, model, or description produced review hints only."],
    candidateEntries: [candidate("entry-hint", "HINT-1", "HINT-S", "Existing hinted equipment")],
  };
  return {
    batchId: "calibration-batch-hinted-create",
    sourceFingerprint: "sha256-hinted-create",
    sourceFilename: "calibration.xlsx",
    mappingVersion: "te-calibration-roster-v1",
    contributingSheets: ["October"],
    totalSourceRows: 1,
    counts: {
      matchedUpdates: 0,
      createCandidates: 1,
      conflicts: 0,
      duplicateSourceRows: 0,
      ignoredJunk: 0,
      currentRequiredAbsent: 0,
    },
    rowOutcomes: [hintedRow],
    currentRequiredAbsent: [],
    prospectiveRequiredCount: 1,
    reconciliationBasis: "basis-hinted-create",
    blocking: true,
  };
}

function allClassificationsReport(): CalibrationRosterPreviewReport {
  const matchedRow: CalibrationRosterRowOutcome = {
    ...createRow(2, 1),
    classification: "matched_update",
    candidateEntryUuid: "matched-1",
    candidateEntries: [candidate("matched-1", "MATCH-1", "MATCH-S", "Matched equipment")],
    description: "Matched reference equipment",
    proposedRequirement: "reference_only",
    proposedInput: null,
    semanticFlags: ["reference_only"],
    requiresReview: false,
  };
  const createCandidate = createRow(3, 2);
  const conflict = conflictReport().rowOutcomes[0]!;
  const duplicate: CalibrationRosterRowOutcome = {
    ...createRow(13, 3),
    classification: "duplicate_source_row",
    issues: ["Another source row resolves to the same equipment."],
    candidateEntryUuid: "matched-1",
    candidateEntries: [candidate("matched-1", "MATCH-1", "MATCH-S", "Matched equipment")],
  };
  const junk = junkRow(14);

  return {
    batchId: "calibration-batch-all-classifications",
    sourceFingerprint: "sha256-all-classifications",
    sourceFilename: "calibration.xlsx",
    mappingVersion: "te-calibration-roster-v1",
    contributingSheets: ["May", "October"],
    totalSourceRows: 5,
    counts: {
      matchedUpdates: 1,
      createCandidates: 1,
      conflicts: 1,
      duplicateSourceRows: 1,
      ignoredJunk: 1,
      currentRequiredAbsent: 1,
    },
    rowOutcomes: [matchedRow, createCandidate, conflict, duplicate, junk],
    currentRequiredAbsent: [{
      entry: candidate("absent-all", "APP-ALL", "APP-S", "App-only required equipment"),
      calibrationDueAt: "2026-08-01",
      calibrationVendor: "Vendor",
      calibrationNotes: "Preserved",
    }],
    prospectiveRequiredCount: 2,
    reconciliationBasis: "basis-all-classifications",
    blocking: true,
  };
}

function createRow(sourceRow: number, index: number): CalibrationRosterRowOutcome {
  return {
    sourceSheet: index % 2 ? "May" : "October",
    sourceRow,
    classification: "create_candidate",
    issues: ["No unique identifier match was found."],
    ignoredIdentityPlaceholders: [],
    assetNumber: `NEW-${index}`,
    serialNumber: `SER-${index}`,
    manufacturer: "Synthetic Maker",
    model: `M${index}`,
    description: `Create candidate ${index}`,
    candidateEntryUuid: null,
    candidateEntries: [],
    proposedRequirement: "required",
    proposedOutToCalibration: false,
    proposedInput: entryInput(`NEW-${index}`, `SER-${index}`, `Create candidate ${index}`),
    changes: [],
    semanticFlags: ["normal_dated"],
    requiresReview: true,
  };
}

function junkRow(sourceRow: number): CalibrationRosterRowOutcome {
  return {
    sourceSheet: "May",
    sourceRow,
    classification: "ignored_junk",
    issues: ["Row has no usable equipment identity or description context."],
    ignoredIdentityPlaceholders: [],
    assetNumber: null,
    serialNumber: null,
    manufacturer: null,
    model: null,
    description: null,
    candidateEntryUuid: null,
    candidateEntries: [],
    proposedRequirement: "unknown",
    proposedOutToCalibration: false,
    proposedInput: null,
    changes: [],
    semanticFlags: [],
    requiresReview: true,
  };
}

function candidate(entryUuid: string, assetNumber: string, serialNumber: string, description: string) {
  return {
    entryUuid,
    id: entryUuid,
    assetNumber,
    serialNumber,
    manufacturer: "Maker",
    model: "Model",
    description,
  };
}

function entryInput(assetNumber: string, serialNumber: string, description: string): InventoryEntryInput {
  return {
    assetNumber,
    serialNumber,
    qty: null,
    manufacturer: "Synthetic Maker",
    model: "Synthetic Model",
    description,
    projectName: "",
    location: "Engineering",
    assignedTo: "Owner",
    links: "",
    notes: "",
    lifecycleStatus: "active",
    workingStatus: "unknown",
    condition: "Working",
    calibrationRequirement: "required",
    outToCalibration: false,
    lastCalibratedAt: "2026-05-31",
    calibrationDueAt: "2027-05-31",
    calibrationVendor: "Vendor",
    calibrationNotes: "Workbook note",
    archived: false,
    picturePath: "",
  };
}
