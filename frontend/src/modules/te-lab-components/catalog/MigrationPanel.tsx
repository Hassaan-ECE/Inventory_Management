import { useState } from "react";
import { AlertTriangleIcon, CheckCircle2Icon, DatabaseBackupIcon, RefreshCwIcon } from "lucide-react";

import type {
  CatalogMigrationCommitResult,
  CatalogMigrationPreview,
  CatalogMigrationStatus,
} from "@/modules/te-lab-components/types";
import { Button } from "@/shared/components/ui/button";

interface MigrationPanelProps {
  migration: CatalogMigrationStatus;
  onCommit: (preview: CatalogMigrationPreview) => Promise<CatalogMigrationCommitResult>;
  onPreview: () => Promise<CatalogMigrationPreview>;
}

export function MigrationPanel({ migration, onCommit, onPreview }: MigrationPanelProps) {
  const [preview, setPreview] = useState<CatalogMigrationPreview | null>(null);
  const [confirmed, setConfirmed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [complete, setComplete] = useState<CatalogMigrationCommitResult | null>(null);

  async function runPreview(): Promise<void> {
    setBusy(true);
    setError(null);
    setComplete(null);
    try {
      setPreview(await onPreview());
      setConfirmed(false);
    } catch (previewError) {
      setError(previewError instanceof Error ? previewError.message : "Could not preview the catalog migration.");
    } finally {
      setBusy(false);
    }
  }

  async function commit(): Promise<void> {
    if (!preview || !confirmed) {
      return;
    }
    setBusy(true);
    setError(null);
    try {
      setComplete(await onCommit(preview));
    } catch (commitError) {
      setError(commitError instanceof Error ? commitError.message : "Could not commit the catalog migration.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="flex h-full min-h-0 items-start justify-center overflow-auto p-4 sm:p-8">
      <section className="w-full max-w-5xl rounded-2xl border border-amber-500/35 bg-card p-5 shadow-sm">
        <div className="flex items-start gap-3">
          <DatabaseBackupIcon className="mt-0.5 size-6 text-amber-600" />
          <div className="min-w-0 flex-1">
            <h2 className="text-lg font-semibold">Legacy Lab data requires reviewed migration</h2>
            <p className="mt-1 text-sm text-muted-foreground">{migration.message}</p>
          </div>
          <Button disabled={busy} onClick={() => void runPreview()} variant="outline">
            <RefreshCwIcon className={`size-3.5 ${busy ? "animate-spin" : ""}`} />
            Run Dry-Run
          </Button>
        </div>

        <div className="mt-4 rounded-xl border border-border bg-muted/20 p-3 text-sm">
          <strong>{migration.legacyEntryCount.toLocaleString()}</strong> legacy rows remain unchanged until a fingerprinted migration is explicitly confirmed.
        </div>

        {error ? <div className="mt-4 rounded-xl border border-destructive/30 bg-destructive/8 p-3 text-sm text-destructive-foreground" role="alert">{error}</div> : null}
        {complete ? (
          <div className="mt-4 rounded-xl border border-emerald-500/35 bg-emerald-500/10 p-4 text-sm">
            <div className="flex items-center gap-2 font-semibold"><CheckCircle2Icon className="size-4" /> Migration complete</div>
            <p className="mt-1">{complete.message}</p>
            <p className="mt-1 text-muted-foreground">{complete.partsCreated} parts, {complete.placementsCreated} placements, {complete.areasCreated} areas, and {complete.containersCreated} containers.</p>
          </div>
        ) : null}

        {preview ? (
          <div className="mt-5 space-y-4">
            <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-4">
              <Metric label="Legacy rows" value={preview.legacyRows} />
              <Metric label="Proposed parts" value={preview.proposedParts} />
              <Metric label="Proposed placements" value={preview.proposedPlacements} />
              <Metric label="Archived parts" value={preview.archivedParts} />
            </div>

            <div className="grid gap-4 lg:grid-cols-2">
              <ReportList title="Warnings" values={preview.warnings} warning />
              <ReportList title="Invalid rows" values={preview.invalidRows} warning={preview.invalidRows.length > 0} />
              <DuplicateGroups title="Duplicate internal part numbers" groups={preview.duplicateInternalPartNumbers} blocking />
              <DuplicateGroups title="Likely manufacturer / MPN duplicates" groups={preview.likelyMpnDuplicates} />
            </div>

            <div className="rounded-xl border border-border bg-muted/20 p-3 text-xs text-muted-foreground">
              Fingerprint: <code className="break-all">{preview.sourceFingerprint}</code>
            </div>

            {preview.blocking ? (
              <div className="flex items-start gap-2 rounded-xl border border-destructive/35 bg-destructive/8 p-3 text-sm text-destructive-foreground">
                <AlertTriangleIcon className="mt-0.5 size-4 shrink-0" />
                Migration is blocked. Resolve duplicate internal part numbers, invalid rows, or partial catalog data, then run the dry-run again.
              </div>
            ) : (
              <div className="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-amber-500/30 bg-amber-500/8 p-3">
                <label className="flex items-start gap-2 text-sm">
                  <input checked={confirmed} className="mt-0.5" onChange={(event) => setConfirmed(event.target.checked)} type="checkbox" />
                  I reviewed this exact dry-run fingerprint and confirm the lossless catalog migration.
                </label>
                <Button disabled={!confirmed || busy} onClick={() => void commit()}>{busy ? "Migrating…" : "Commit Migration"}</Button>
              </div>
            )}
          </div>
        ) : (
          <div className="mt-5 rounded-xl border border-dashed border-border p-8 text-center text-sm text-muted-foreground">
            Run the dry-run to review mappings, warnings, duplicate groups, and the source fingerprint. No data is modified during preview.
          </div>
        )}
      </section>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return <div className="rounded-xl border border-border bg-background p-3"><div className="text-2xl font-semibold tabular-nums">{value.toLocaleString()}</div><div className="text-xs text-muted-foreground">{label}</div></div>;
}

function ReportList({ title, values, warning = false }: { title: string; values: string[]; warning?: boolean }) {
  return <section className={`rounded-xl border p-3 ${warning && values.length ? "border-amber-500/35 bg-amber-500/8" : "border-border bg-background"}`}><h3 className="font-semibold">{title} ({values.length})</h3>{values.length ? <ul className="mt-2 list-disc space-y-1 pl-5 text-sm text-muted-foreground">{values.map((value) => <li key={value}>{value}</li>)}</ul> : <p className="mt-1 text-sm text-muted-foreground">None</p>}</section>;
}

function DuplicateGroups({ title, groups, blocking = false }: { title: string; groups: CatalogMigrationPreview["likelyMpnDuplicates"]; blocking?: boolean }) {
  return <section className={`rounded-xl border p-3 ${blocking && groups.length ? "border-destructive/35 bg-destructive/8" : "border-border bg-background"}`}><h3 className="font-semibold">{title} ({groups.length})</h3>{groups.length ? <div className="mt-2 space-y-2 text-sm">{groups.map((group) => <div className="rounded-lg border border-border/70 p-2" key={`${group.key}-${group.entryUuids.join("-")}`}><div className="font-medium">{group.key}</div><div className="text-xs text-muted-foreground">Entries: {group.entryIds.join(", ")}</div></div>)}</div> : <p className="mt-1 text-sm text-muted-foreground">None</p>}</section>;
}
