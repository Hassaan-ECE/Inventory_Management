import type {
  CalibrationRequirement,
  CalibrationRosterResolutionAction,
  CalibrationRosterRowOutcome,
  InventoryEntryInput,
} from "@/modules/te-test-equipment/types";
import {
  candidateModel,
  classificationVariant,
  formatToken,
  identityLabel,
  resolutionActionComplete,
  resolutionActionOptions,
  resolutionComplete,
  type ResolutionDraft,
} from "@/modules/te-test-equipment/components/calibration/rosterResolutions";
import { Badge } from "@/shared/components/ui/badge";
import { DropdownSelect } from "@/shared/components/ui/DropdownMenu";
import { Input } from "@/shared/components/ui/input";
import { Textarea } from "@/shared/components/ui/textarea";

const REQUIREMENT_OPTIONS = [
  { value: "required", label: "Required" },
  { value: "reference_only", label: "Reference only" },
  { value: "not_required", label: "Not required" },
  { value: "unknown", label: "Unknown" },
] as const;

/** One roster row that needs a decision: what changes, and how to resolve it. */
export function ReviewRowCard({
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
