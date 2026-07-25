import { useEffect, useId, useMemo, useState } from "react";

import { TE_TEST_EQUIPMENT_MODULE_ID } from "@/modules/te-test-equipment/moduleId";
import type {
  CalibrationRequirement,
  CalibrationRosterCommitResult,
  CalibrationRosterPreviewReport,
  CalibrationRosterResolution,
  CalibrationRosterResolutionAction,
  CalibrationRosterRowOutcome,
  InventoryEntryInput,
} from "@/modules/te-test-equipment/types";
import { Badge } from "@/shared/components/ui/badge";
import { Button } from "@/shared/components/ui/button";
import { DropdownSelect } from "@/shared/components/ui/DropdownMenu";
import { Input } from "@/shared/components/ui/input";
import { ScrollRegion } from "@/shared/components/ui/ScrollRegion";
import { Textarea } from "@/shared/components/ui/textarea";

interface CalibrationRosterDialogProps {
  onClose: () => void;
  onCommitted: (result: CalibrationRosterCommitResult) => Promise<void> | void;
}

interface ResolutionDraft {
  action: CalibrationRosterResolutionAction | "";
  confirmed: boolean;
  createInput?: InventoryEntryInput;
  targetEntryUuid?: string;
}

const REQUIREMENT_OPTIONS = [
  { value: "required", label: "Required" },
  { value: "reference_only", label: "Reference only" },
  { value: "not_required", label: "Not required" },
  { value: "unknown", label: "Unknown" },
] as const;

const DEFAULT_ATTRIBUTION = "Calibration roster cutover — TE Lab Equip Calibration Data for New List.xlsx";

export function CalibrationRosterDialog({ onClose, onCommitted }: CalibrationRosterDialogProps) {
  const titleId = useId();
  const bridge = window.inventoryDesktop;
  const desktopAvailable = Boolean(
    bridge?.isDesktop
      && bridge.pickCalibrationRosterFile
      && bridge.previewCalibrationRoster
      && bridge.commitCalibrationRoster,
  );
  const [report, setReport] = useState<CalibrationRosterPreviewReport | null>(null);
  const [drafts, setDrafts] = useState<Record<string, ResolutionDraft>>({});
  const [previewing, setPreviewing] = useState(false);
  const [committing, setCommitting] = useState(false);
  const [replaceConfirmed, setReplaceConfirmed] = useState(false);
  const [reverificationConfirmed, setReverificationConfirmed] = useState(false);
  const [verificationAttribution, setVerificationAttribution] = useState(DEFAULT_ATTRIBUTION);
  const [result, setResult] = useState<CalibrationRosterCommitResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const busy = previewing || committing;
  const reviewRows = useMemo(
    () => report?.rowOutcomes.filter((row) => row.requiresReview) ?? [],
    [report],
  );
  const semanticRows = useMemo(
    () => report?.rowOutcomes.filter((row) => (
      row.semanticFlags.some((flag) => flag !== "normal_dated") && !row.requiresReview
    )) ?? [],
    [report],
  );
  const allResolutionsComplete = reviewRows.every((row) => resolutionComplete(row, drafts[rowKey(row)]));
  const canCommit = Boolean(
    report
      && !busy
      && !result
      && allResolutionsComplete
      && replaceConfirmed
      && reverificationConfirmed
      && verificationAttribution.trim(),
  );

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key === "Escape" && !busy) {
        onClose();
      }
    }
    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [busy, onClose]);

  async function chooseWorkbook(): Promise<void> {
    const pickFile = bridge?.pickCalibrationRosterFile;
    const previewRoster = bridge?.previewCalibrationRoster;
    if (!pickFile || !previewRoster) {
      return;
    }
    setError(null);
    setResult(null);
    setReport(null);
    setDrafts({});
    setReplaceConfirmed(false);
    setReverificationConfirmed(false);
    setPreviewing(true);
    try {
      const path = await pickFile(TE_TEST_EQUIPMENT_MODULE_ID);
      if (!path) {
        return;
      }
      const preview = await previewRoster(TE_TEST_EQUIPMENT_MODULE_ID, path);
      setReport(preview);
      setDrafts(defaultResolutionDrafts(preview));
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setPreviewing(false);
    }
  }

  async function commitRoster(): Promise<void> {
    const commit = bridge?.commitCalibrationRoster;
    if (!commit || !report || !canCommit) {
      return;
    }
    setError(null);
    setCommitting(true);
    try {
      const committed = await commit(TE_TEST_EQUIPMENT_MODULE_ID, {
        batchId: report.batchId,
        confirmed: true,
        replaceActiveRequiredRoster: true,
        verificationAttribution: verificationAttribution.trim(),
        resolutions: buildResolutions(reviewRows, drafts),
      });
      setResult(committed);
      await onCommitted(committed);
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setCommitting(false);
    }
  }

  return (
    <div
      aria-labelledby={titleId}
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-3 backdrop-blur-[2px] sm:p-4"
      role="dialog"
      onClick={(event) => {
        if (event.target === event.currentTarget && !busy) {
          onClose();
        }
      }}
    >
      <section className="flex max-h-[94vh] w-full max-w-7xl flex-col overflow-hidden rounded-2xl border border-border/70 bg-card text-card-foreground shadow-2xl">
        <header className="flex shrink-0 items-start justify-between gap-4 border-b border-border/70 px-5 py-4">
          <div>
            <p className="text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">
              Calibration roster cutover
            </p>
            <h2 className="mt-1 text-xl font-semibold tracking-tight text-foreground" id={titleId}>
              Initialize from Calibration Workbook
            </h2>
            <p className="mt-1 text-sm text-muted-foreground">
              Preview and resolve the workbook first. No equipment changes are written until the final commit.
            </p>
          </div>
          <Button aria-label="Close calibration roster dialog" disabled={busy} size="sm" variant="ghost" onClick={onClose}>
            Close
          </Button>
        </header>

        <ScrollRegion className="min-h-0 flex-1" contentClassName="space-y-5 p-5">
          {!desktopAvailable ? (
            <p className="rounded-xl border border-border/70 bg-background/70 p-4 text-sm text-muted-foreground">
              Calibration roster initialization is available only in the desktop app.
            </p>
          ) : (
            <div className="flex flex-wrap items-center gap-3">
              <Button disabled={busy} onClick={() => void chooseWorkbook()}>
                {previewing ? "Reading workbook..." : report ? "Choose Different Workbook" : "Choose Calibration Workbook"}
              </Button>
              <span className="text-sm text-muted-foreground">Only .xlsx calibration roster workbooks are accepted.</span>
            </div>
          )}

          {error ? (
            <p className="rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive-foreground" role="alert">
              {error}
            </p>
          ) : null}

          {report ? (
            <>
              <RosterSummary report={report} />

              {reviewRows.length > 0 ? (
                <section aria-labelledby="calibration-roster-review-heading">
                  <div className="flex flex-wrap items-end justify-between gap-2">
                    <div>
                      <h3 className="text-base font-semibold text-foreground" id="calibration-roster-review-heading">
                        Required row review
                      </h3>
                      <p className="mt-1 text-sm text-muted-foreground">
                        Confirm every create, ignored label, conflict, duplicate, and destructive change.
                      </p>
                    </div>
                    <Badge variant={allResolutionsComplete ? "success" : "warning"}>
                      {reviewRows.filter((row) => resolutionComplete(row, drafts[rowKey(row)])).length} / {reviewRows.length} confirmed
                    </Badge>
                  </div>
                  <div className="mt-3 space-y-3">
                    {reviewRows.map((row) => (
                      <ReviewRowCard
                        draft={drafts[rowKey(row)] ?? defaultResolutionDraft(row)}
                        key={rowKey(row)}
                        row={row}
                        onChange={(nextDraft) => {
                          setDrafts((current) => ({ ...current, [rowKey(row)]: nextDraft }));
                        }}
                      />
                    ))}
                  </div>
                </section>
              ) : null}

              {semanticRows.length > 0 ? (
                <section aria-labelledby="calibration-roster-semantic-heading">
                  <h3 className="text-base font-semibold text-foreground" id="calibration-roster-semantic-heading">
                    Semantic exceptions already resolved
                  </h3>
                  <p className="mt-1 text-sm text-muted-foreground">
                    These rows are not blocking, but their explicit workbook meaning will be applied.
                  </p>
                  <div className="mt-3 grid gap-2 lg:grid-cols-2">
                    {semanticRows.map((row) => <CompactRow key={rowKey(row)} row={row} />)}
                  </div>
                </section>
              ) : null}

              {report.currentRequiredAbsent.length > 0 ? (
                <section aria-labelledby="calibration-roster-absent-heading">
                  <h3 className="text-base font-semibold text-foreground" id="calibration-roster-absent-heading">
                    Current required equipment absent from workbook
                  </h3>
                  <p className="mt-1 text-sm text-muted-foreground">
                    These active records will move to Unknown and leave the default Calibration roster. Dates, vendor,
                    certificate, notes, verification, lifecycle, and archive state remain intact.
                  </p>
                  <div className="mt-3 overflow-x-auto rounded-xl border border-border/70">
                    <table className="w-full min-w-[48rem] text-left text-sm">
                      <thead className="bg-muted/50 text-xs text-muted-foreground">
                        <tr>
                          <th className="p-2">Equipment</th>
                          <th className="p-2">Asset / serial</th>
                          <th className="p-2">Due</th>
                          <th className="p-2">Vendor</th>
                        </tr>
                      </thead>
                      <tbody>
                        {report.currentRequiredAbsent.map((absent) => (
                          <tr className="border-t border-border/70" key={absent.entry.entryUuid}>
                            <td className="p-2 font-medium">{absent.entry.description || candidateModel(absent.entry)}</td>
                            <td className="p-2">{identityLabel(absent.entry.assetNumber, absent.entry.serialNumber)}</td>
                            <td className="p-2">{absent.calibrationDueAt ?? "Missing"}</td>
                            <td className="p-2">{absent.calibrationVendor ?? "—"}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </section>
              ) : null}

              {result ? <CommitResult result={result} /> : null}
            </>
          ) : null}
        </ScrollRegion>

        {report ? (
          <footer className="shrink-0 border-t border-border/70 bg-card px-5 py-4">
            {!result ? (
              <div className="grid gap-3 lg:grid-cols-[minmax(0,1fr)_minmax(18rem,28rem)]">
                <div className="space-y-2">
                  <label className="flex items-start gap-2 text-sm text-foreground">
                    <input
                      checked={replaceConfirmed}
                      className="mt-0.5 size-4 accent-[var(--primary)]"
                      disabled={busy}
                      type="checkbox"
                      onChange={(event) => setReplaceConfirmed(event.currentTarget.checked)}
                    />
                    I confirm this workbook replaces the active required calibration roster. App-only required rows will move to Unknown.
                  </label>
                  <label className="flex items-start gap-2 text-sm text-foreground">
                    <input
                      checked={reverificationConfirmed}
                      className="mt-0.5 size-4 accent-[var(--primary)]"
                      disabled={busy}
                      type="checkbox"
                      onChange={(event) => setReverificationConfirmed(event.currentTarget.checked)}
                    />
                    I confirm every approved workbook row will be re-verified at the actual commit time.
                  </label>
                </div>
                <label className="block">
                  <span className="mb-1 block text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">
                    Verification attribution
                  </span>
                  <Input
                    aria-label="Calibration roster verification attribution"
                    disabled={busy}
                    value={verificationAttribution}
                    onChange={(event) => setVerificationAttribution(event.currentTarget.value)}
                  />
                </label>
              </div>
            ) : null}
            <div className="mt-4 flex flex-wrap justify-end gap-2">
              <Button disabled={busy} variant="ghost" onClick={onClose}>
                {result ? "Done" : "Cancel"}
              </Button>
              {!result ? (
                <Button disabled={!canCommit} onClick={() => void commitRoster()}>
                  {committing ? "Applying roster..." : "Commit Calibration Roster"}
                </Button>
              ) : null}
            </div>
          </footer>
        ) : null}
      </section>
    </div>
  );
}

function RosterSummary({ report }: { report: CalibrationRosterPreviewReport }) {
  const totals = [
    ["Matched updates", report.counts.matchedUpdates],
    ["Create candidates", report.counts.createCandidates],
    ["Conflicts", report.counts.conflicts],
    ["Duplicate source rows", report.counts.duplicateSourceRows],
    ["Ignored junk", report.counts.ignoredJunk],
    ["Current required absent", report.counts.currentRequiredAbsent],
  ] as const;
  return (
    <section className="space-y-3" aria-label="Calibration roster preview summary">
      <dl className="grid gap-3 rounded-xl border border-border/70 bg-background/60 p-4 text-sm sm:grid-cols-2 lg:grid-cols-4">
        <Metadata label="Source" value={report.sourceFilename} />
        <Metadata label="Sheets" value={report.contributingSheets.join(", ")} />
        <Metadata label="Workbook rows" value={String(report.totalSourceRows)} />
        <Metadata label="Prospective required" value={String(report.prospectiveRequiredCount)} />
      </dl>
      <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
        {totals.map(([label, value]) => (
          <div className="rounded-xl border border-border/70 bg-background/60 px-3 py-3" key={label}>
            <p className="text-2xl font-semibold text-foreground">{value}</p>
            <p className="mt-0.5 text-xs text-muted-foreground">{label}</p>
          </div>
        ))}
      </div>
    </section>
  );
}

function ReviewRowCard({
  draft,
  onChange,
  row,
}: {
  draft: ResolutionDraft;
  onChange: (draft: ResolutionDraft) => void;
  row: CalibrationRosterRowOutcome;
}) {
  const actionOptions = resolutionActionOptions(row);
  const complete = resolutionComplete(row, draft);
  return (
    <article className="rounded-xl border border-border/70 bg-background/55 p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="min-w-0">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant={classificationVariant(row.classification)}>{formatToken(row.classification)}</Badge>
            {row.semanticFlags.map((flag) => <Badge key={flag} size="sm">{formatToken(flag)}</Badge>)}
            <span className="text-xs text-muted-foreground">{row.sourceSheet} · row {row.sourceRow}</span>
          </div>
          <p className="mt-2 font-medium text-foreground">{row.description || candidateModel(row) || "Unlabeled source row"}</p>
          <p className="mt-0.5 text-sm text-muted-foreground">
            {identityLabel(row.assetNumber, row.serialNumber)}
            {row.manufacturer || row.model ? ` · ${[row.manufacturer, row.model].filter(Boolean).join(" ")}` : ""}
          </p>
        </div>
        <Badge variant={complete ? "success" : "warning"}>{complete ? "Confirmed" : "Review required"}</Badge>
      </div>

      {row.issues.length || row.ignoredIdentityPlaceholders.length ? (
        <ul className="mt-3 space-y-1 rounded-lg border border-warning/25 bg-warning/5 px-3 py-2 text-sm text-foreground">
          {[...row.issues, ...row.ignoredIdentityPlaceholders].map((issue, index) => (
            <li key={`${issue}:${index}`}>• {issue}</li>
          ))}
        </ul>
      ) : null}

      {row.changes.length > 0 ? (
        <div className="mt-3 overflow-x-auto rounded-lg border border-border/70">
          <table className="w-full min-w-[36rem] text-left text-xs">
            <thead className="bg-muted/45 text-muted-foreground">
              <tr><th className="p-2">Field</th><th className="p-2">Before</th><th className="p-2">After</th></tr>
            </thead>
            <tbody>
              {row.changes.map((change) => (
                <tr className="border-t border-border/60" key={change.field}>
                  <td className="p-2 font-medium">
                    {formatToken(change.field)} {change.destructive ? <Badge size="sm" variant="warning">Clears/changes scope</Badge> : null}
                  </td>
                  <td className="p-2">{change.before ?? "Empty"}</td>
                  <td className="p-2">{change.after ?? "Empty"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : null}

      <div className="mt-4 grid gap-3 lg:grid-cols-2">
        <label className="block">
          <span className="mb-1 block text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">Resolution</span>
          <DropdownSelect
            aria-label={`Resolution for ${row.sourceSheet} row ${row.sourceRow}`}
            options={actionOptions}
            placeholder="Choose resolution"
            value={draft.action}
            onChange={(action) => {
              const nextAction = action as CalibrationRosterResolutionAction;
              onChange({
                ...draft,
                action: nextAction,
                confirmed: false,
                createInput: nextAction === "create" ? draft.createInput ?? row.proposedInput ?? undefined : draft.createInput,
                targetEntryUuid: nextAction === "use_existing"
                  ? draft.targetEntryUuid ?? row.candidateEntryUuid ?? row.candidateEntries[0]?.entryUuid
                  : draft.targetEntryUuid,
              });
            }}
          />
        </label>
        {draft.action === "use_existing" ? (
          <label className="block">
            <span className="mb-1 block text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">Target equipment</span>
            <DropdownSelect
              aria-label={`Target equipment for ${row.sourceSheet} row ${row.sourceRow}`}
              options={row.candidateEntries.map((candidate) => ({
                value: candidate.entryUuid,
                label: `${identityLabel(candidate.assetNumber, candidate.serialNumber)} — ${candidate.description || candidateModel(candidate)}`,
              }))}
              placeholder="Choose equipment"
              value={draft.targetEntryUuid ?? ""}
              onChange={(targetEntryUuid) => onChange({ ...draft, confirmed: false, targetEntryUuid })}
            />
          </label>
        ) : null}
      </div>

      {draft.action === "create" && draft.createInput ? (
        <CreateInputEditor
          input={draft.createInput}
          onChange={(createInput) => onChange({ ...draft, confirmed: false, createInput })}
        />
      ) : null}

      <label className="mt-4 flex items-start gap-2 text-sm font-medium text-foreground">
        <input
          aria-label={`Confirm resolution for ${row.sourceSheet} row ${row.sourceRow}`}
          checked={draft.confirmed}
          className="mt-0.5 size-4 accent-[var(--primary)]"
          disabled={!resolutionActionComplete(row, draft)}
          type="checkbox"
          onChange={(event) => onChange({ ...draft, confirmed: event.currentTarget.checked })}
        />
        I reviewed this source row and confirm the selected resolution.
      </label>
    </article>
  );
}

function CreateInputEditor({
  input,
  onChange,
}: {
  input: InventoryEntryInput;
  onChange: (input: InventoryEntryInput) => void;
}) {
  const setString = (field: keyof InventoryEntryInput, value: string): void => {
    onChange({ ...input, [field]: value });
  };
  return (
    <fieldset className="mt-4 rounded-xl border border-border/70 bg-card/70 p-4">
      <legend className="px-1 text-sm font-semibold text-foreground">Reviewed new equipment input</legend>
      <div className="mt-2 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
        <InputField label="Asset number" value={input.assetNumber} onChange={(value) => setString("assetNumber", value)} />
        <InputField label="Serial number" value={input.serialNumber} onChange={(value) => setString("serialNumber", value)} />
        <label className="block">
          <FieldLabel>Quantity</FieldLabel>
          <Input
            aria-label="Create quantity"
            min="0"
            step="any"
            type="number"
            value={input.qty ?? ""}
            onChange={(event) => onChange({
              ...input,
              qty: event.currentTarget.value === "" ? null : Number(event.currentTarget.value),
            })}
          />
        </label>
        <InputField label="Manufacturer" value={input.manufacturer} onChange={(value) => setString("manufacturer", value)} />
        <InputField label="Model" value={input.model} onChange={(value) => setString("model", value)} />
        <InputField label="Description" value={input.description} onChange={(value) => setString("description", value)} />
        <InputField label="Location" value={input.location} onChange={(value) => setString("location", value)} />
        <InputField label="Assigned to" value={input.assignedTo} onChange={(value) => setString("assignedTo", value)} />
        <InputField label="Condition" value={input.condition} onChange={(value) => setString("condition", value)} />
        <label className="block">
          <FieldLabel>Calibration requirement</FieldLabel>
          <DropdownSelect
            aria-label="Create calibration requirement"
            options={REQUIREMENT_OPTIONS}
            value={input.calibrationRequirement}
            onChange={(value) => onChange({ ...input, calibrationRequirement: value as CalibrationRequirement })}
          />
        </label>
        <InputField
          label="Last calibrated"
          type="date"
          value={input.lastCalibratedAt ?? ""}
          onChange={(value) => onChange({ ...input, lastCalibratedAt: value || undefined })}
        />
        <InputField
          label="Calibration due"
          type="date"
          value={input.calibrationDueAt ?? ""}
          onChange={(value) => onChange({ ...input, calibrationDueAt: value || undefined })}
        />
        <label className="block">
          <FieldLabel>Interval months</FieldLabel>
          <Input
            aria-label="Create calibration interval months"
            min="1"
            type="number"
            value={input.calibrationIntervalMonths ?? ""}
            onChange={(event) => onChange({
              ...input,
              calibrationIntervalMonths: event.currentTarget.value === ""
                ? undefined
                : Number(event.currentTarget.value),
            })}
          />
        </label>
        <InputField
          label="Calibration vendor"
          value={input.calibrationVendor ?? ""}
          onChange={(value) => onChange({ ...input, calibrationVendor: value || undefined })}
        />
        <InputField
          label="Certificate"
          value={input.certificateRef ?? ""}
          onChange={(value) => onChange({ ...input, certificateRef: value || undefined })}
        />
        <label className="flex items-center gap-2 self-end pb-2 text-sm text-foreground">
          <input
            checked={input.outToCalibration}
            className="size-4 accent-[var(--primary)]"
            type="checkbox"
            onChange={(event) => onChange({ ...input, outToCalibration: event.currentTarget.checked })}
          />
          Out to calibration
        </label>
      </div>
      <label className="mt-3 block">
        <FieldLabel>Calibration notes</FieldLabel>
        <Textarea
          aria-label="Create calibration notes"
          value={input.calibrationNotes ?? ""}
          onChange={(event) => onChange({ ...input, calibrationNotes: event.currentTarget.value || undefined })}
        />
      </label>
    </fieldset>
  );
}

function InputField({
  label,
  onChange,
  type = "text",
  value,
}: {
  label: string;
  onChange: (value: string) => void;
  type?: string;
  value: string;
}) {
  return (
    <label className="block">
      <FieldLabel>{label}</FieldLabel>
      <Input aria-label={`Create ${label.toLowerCase()}`} type={type} value={value} onChange={(event) => onChange(event.currentTarget.value)} />
    </label>
  );
}

function FieldLabel({ children }: { children: string }) {
  return <span className="mb-1 block text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">{children}</span>;
}

function CompactRow({ row }: { row: CalibrationRosterRowOutcome }) {
  return (
    <article className="rounded-xl border border-border/70 bg-background/55 p-3 text-sm">
      <div className="flex flex-wrap items-center gap-2">
        {row.semanticFlags.map((flag) => <Badge key={flag} size="sm">{formatToken(flag)}</Badge>)}
        <span className="text-xs text-muted-foreground">{row.sourceSheet} · row {row.sourceRow}</span>
      </div>
      <p className="mt-2 font-medium text-foreground">{row.description || candidateModel(row)}</p>
      <p className="mt-0.5 text-muted-foreground">{identityLabel(row.assetNumber, row.serialNumber)}</p>
    </article>
  );
}

function CommitResult({ result }: { result: CalibrationRosterCommitResult }) {
  return (
    <section className="rounded-xl border border-success/30 bg-success/10 p-4" aria-label="Calibration roster commit result">
      <p className="font-medium text-success-foreground">{result.message}</p>
      <p className="mt-1 text-sm text-muted-foreground">
        Updated {result.updated}, created {result.created}, reset {result.reset}, ignored {result.ignored}, no-op {result.noop}.
        Final active required roster: {result.finalRequired}.
      </p>
    </section>
  );
}

function Metadata({ label, value }: { label: string; value: string }) {
  return <div><dt className="text-xs font-medium text-muted-foreground">{label}</dt><dd className="mt-0.5 break-all text-foreground">{value}</dd></div>;
}

function defaultResolutionDrafts(report: CalibrationRosterPreviewReport): Record<string, ResolutionDraft> {
  return Object.fromEntries(
    report.rowOutcomes
      .filter((row) => row.requiresReview)
      .map((row) => [rowKey(row), defaultResolutionDraft(row)]),
  );
}

function defaultResolutionDraft(row: CalibrationRosterRowOutcome): ResolutionDraft {
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

function resolutionActionOptions(row: CalibrationRosterRowOutcome) {
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

function resolutionComplete(row: CalibrationRosterRowOutcome, draft: ResolutionDraft | undefined): boolean {
  return Boolean(draft?.confirmed && resolutionActionComplete(row, draft));
}

function resolutionActionComplete(row: CalibrationRosterRowOutcome, draft: ResolutionDraft): boolean {
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

function buildResolutions(
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

function rowKey(row: Pick<CalibrationRosterRowOutcome, "sourceRow" | "sourceSheet">): string {
  return `${row.sourceSheet}\u0000${row.sourceRow}`;
}

function identityLabel(assetNumber: string | null, serialNumber: string | null): string {
  return [assetNumber, serialNumber].filter(Boolean).join(" / ") || "No usable asset or serial";
}

function candidateModel(candidate: { manufacturer?: string | null; model?: string | null }): string {
  return [candidate.manufacturer, candidate.model].filter(Boolean).join(" ") || "Equipment";
}

function classificationVariant(
  classification: CalibrationRosterRowOutcome["classification"],
): "error" | "outline" | "secondary" | "success" | "warning" {
  if (classification === "matched_update") return "secondary";
  if (classification === "create_candidate") return "success";
  if (classification === "ignored_junk") return "warning";
  return "error";
}

function formatToken(value: string): string {
  return value
    .replace(/([a-z])([A-Z])/g, "$1 $2")
    .replaceAll("_", " ")
    .replace(/^./, (character) => character.toUpperCase());
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : "Calibration roster operation failed.";
}
