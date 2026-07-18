import { useEffect, useId, useState } from "react";

import { Badge } from "@/shared/components/ui/badge";
import { Button } from "@/shared/components/ui/button";
import { ScrollRegion } from "@/shared/components/ui/ScrollRegion";
import type {
  ImportClassification,
  ImportCommitResult,
  ImportDryRunReport,
} from "@/features/inventory/types";

interface ImportDialogProps {
  onClose: () => void;
  onEntriesChanged: () => Promise<void> | void;
}

export function ImportDialog({ onClose, onEntriesChanged }: ImportDialogProps) {
  const titleId = useId();
  const [report, setReport] = useState<ImportDryRunReport | null>(null);
  const [confirmed, setConfirmed] = useState(false);
  const [previewing, setPreviewing] = useState(false);
  const [committing, setCommitting] = useState(false);
  const [result, setResult] = useState<ImportCommitResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const bridge = window.inventoryDesktop;
  const desktopAvailable = Boolean(bridge?.isDesktop);
  const busy = previewing || committing;

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key === "Escape" && !busy) onClose();
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [busy, onClose]);

  async function chooseFile(): Promise<void> {
    if (!bridge?.isDesktop) return;

    setReport(null);
    setConfirmed(false);
    setResult(null);
    setError(null);
    setPreviewing(true);
    try {
      const path = await bridge.pickImportFile();
      if (!path) return;
      setReport(await bridge.previewImport(path));
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setPreviewing(false);
    }
  }

  async function commit(): Promise<void> {
    if (!bridge?.isDesktop || !report || !confirmed) {
      return;
    }
    const hasBlockingRows = report.blocking || report.conflicted > 0 || report.rejected > 0;
    if (hasBlockingRows) {
      return;
    }

    setError(null);
    setResult(null);
    setCommitting(true);
    try {
      const committed = await bridge.commitImport({
        batchId: report.batchId,
        confirmed: true,
      });
      if (committed.entriesChanged) await onEntriesChanged();
      setResult(committed);
    } catch (cause) {
      setError(errorMessage(cause));
    } finally {
      setCommitting(false);
    }
  }

  const hasBlockingRows = Boolean(report && (report.blocking || report.conflicted > 0 || report.rejected > 0));
  const fullCommitDisabled = !report || !confirmed || hasBlockingRows || busy;

  return (
    <div
      aria-labelledby={titleId}
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/45 p-4 backdrop-blur-[2px]"
      role="dialog"
      onClick={(event) => {
        if (event.target === event.currentTarget && !busy) onClose();
      }}
    >
      <section className="flex max-h-[92vh] w-full max-w-6xl flex-col overflow-hidden rounded-2xl border border-border/70 bg-card text-card-foreground shadow-2xl">
        <header className="flex shrink-0 items-start justify-between gap-4 border-b border-border/70 p-5">
          <div>
            <p className="text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">Inventory import</p>
            <h2 className="mt-1 text-xl font-semibold tracking-tight text-foreground" id={titleId}>
              Import equipment
            </h2>
          </div>
          <Button aria-label="Close import dialog" disabled={busy} size="sm" variant="ghost" onClick={onClose}>
            Close
          </Button>
        </header>

        <ScrollRegion className="min-h-0 flex-1" contentClassName="p-5">
          {!desktopAvailable ? (
            <p className="rounded-xl border border-border/70 bg-background/70 p-4 text-sm text-muted-foreground">
              Import is available in the desktop app.
            </p>
          ) : (
            <>
              <div className="flex flex-wrap items-center gap-3">
                <Button disabled={busy} onClick={() => void chooseFile()}>
                  {previewing ? "Reading import..." : "Choose import file"}
                </Button>
                <span className="text-sm text-muted-foreground">Preview first. Nothing is written until commit.</span>
              </div>

              {error ? (
                <p
                  className="mt-4 rounded-lg border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive-foreground"
                  role="alert"
                >
                  {error}
                </p>
              ) : null}

              {report ? <ImportReport report={report} /> : null}
              {result ? <CommitResult result={result} /> : null}
            </>
          )}
        </ScrollRegion>

        {desktopAvailable && report ? (
          <footer className="shrink-0 border-t border-border/70 bg-card p-5">
            <label className="flex items-start gap-2 text-sm font-medium text-foreground">
              <input
                checked={confirmed}
                className="mt-0.5 size-4"
                disabled={busy}
                type="checkbox"
                onChange={(event) => setConfirmed(event.currentTarget.checked)}
              />
              I confirm this import preview and want to write the full batch to the local database.
            </label>
            {hasBlockingRows ? (
              <p className="mt-2 text-sm text-muted-foreground">
                {report.conflicted} conflicted and {report.rejected} rejected rows block commit. Correct the source and
                run the dry run again; no rows can be committed from this batch.
              </p>
            ) : null}
            <div className="mt-4 flex flex-wrap justify-end gap-2">
              <Button disabled={busy} variant="ghost" onClick={onClose}>
                Cancel
              </Button>
              <Button disabled={fullCommitDisabled} onClick={() => void commit()}>
                {committing ? "Committing..." : "Commit import"}
              </Button>
            </div>
          </footer>
        ) : null}
      </section>
    </div>
  );
}

function ImportReport({ report }: { report: ImportDryRunReport }) {
  const totals = [
    ["Inserted", report.inserted],
    ["Matched", report.matched],
    ["Conflicted", report.conflicted],
    ["Rejected", report.rejected],
    ["Ignored", report.ignored],
  ] as const;

  return (
    <div className="mt-5 space-y-5">
      <dl className="grid gap-3 rounded-xl border border-border/70 bg-background/60 p-4 text-sm sm:grid-cols-2 lg:grid-cols-3">
        <Metadata label="Source" value={report.sourceFilename} />
        <Metadata label="Sheet" value={report.selectedSheet} />
        <Metadata label="Batch" value={report.batchId} />
        <Metadata label="Mapping" value={report.mappingVersion} />
        <Metadata label="Reconciliation" value={report.reconciliationBasis} />
        <Metadata label="Rows" value={String(report.totalRows)} />
      </dl>

      <div aria-label="Import reconciliation totals" className="flex flex-wrap gap-2">
        {totals.map(([label, value]) => (
          <Badge key={label} variant={value > 0 ? "secondary" : "outline"}>
            {label}: {value}
          </Badge>
        ))}
        {report.blocking ? <Badge variant="error">Blocking</Badge> : <Badge variant="success">Ready</Badge>}
      </div>

      <section aria-labelledby="import-columns-heading">
        <h3 className="text-sm font-semibold text-foreground" id="import-columns-heading">
          Column mapping
        </h3>
        <div className="mt-2 overflow-x-auto rounded-xl border border-border/70">
          <table className="w-full text-left text-sm">
            <thead className="bg-muted/50 text-xs text-muted-foreground">
              <tr>
                <th className="p-2">Source header</th>
                <th className="p-2">Target</th>
                <th className="p-2">Treatment</th>
                <th className="p-2">Nonblank</th>
                <th className="p-2">Reason</th>
              </tr>
            </thead>
            <tbody>
              {report.columns.map((column, index) => (
                <tr className="border-t border-border/70 align-top" key={`${column.originalHeader}:${index}`}>
                  <td className="p-2 font-medium">{column.originalHeader}</td>
                  <td className="p-2">{column.normalizedTarget ?? "Unmapped"}</td>
                  <td className="p-2">
                    <Badge size="sm">{formatToken(column.treatment)}</Badge>
                  </td>
                  <td className="p-2">{column.nonblankCount}</td>
                  <td className="p-2">{column.reason}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>

      <section aria-labelledby="import-rows-heading">
        <h3 className="text-sm font-semibold text-foreground" id="import-rows-heading">
          Row reconciliation (problem rows first)
        </h3>
        <div className="mt-2 max-h-72 overflow-hidden rounded-xl border border-border/70">
          <ScrollRegion className="max-h-72" topCueClassName="z-10" topCueStyle={{ top: 36 }}>
            <table className="w-full text-left text-sm">
              <thead className="sticky top-0 z-20 bg-muted/95 text-xs text-muted-foreground backdrop-blur">
                <tr>
                  <th className="p-2">Source row</th>
                  <th className="p-2">Classification</th>
                  <th className="p-2">Issues</th>
                  <th className="p-2">Original identifiers</th>
                </tr>
              </thead>
              <tbody>
                {[...report.rowOutcomes]
                  .sort((a, b) => classificationRank(a.classification) - classificationRank(b.classification))
                  .map((row) => (
                    <tr className="border-t border-border/70 align-top" key={`${row.sourceRow}:${row.classification}`}>
                      <td className="p-2">{row.sourceRow}</td>
                      <td className="p-2">
                        <Badge size="sm" variant={classificationVariant(row.classification)}>
                          {formatToken(row.classification)}
                        </Badge>
                      </td>
                      <td className="p-2">
                        {row.issues.length ? (
                          <ul>
                            {row.issues.map((issue, index) => (
                              <li key={`${issue}:${index}`}>{issue}</li>
                            ))}
                          </ul>
                        ) : (
                          "None"
                        )}
                      </td>
                      <td className="p-2">
                        <ValueList values={[row.originalId, row.originalAssetNumber, row.originalSerialNumber]} />
                      </td>
                    </tr>
                  ))}
              </tbody>
            </table>
          </ScrollRegion>
        </div>
      </section>
    </div>
  );
}

function CommitResult({ result }: { result: ImportCommitResult }) {
  if (!result.entriesChanged) {
    return (
      <section className="mt-5 rounded-xl border border-border/70 bg-background/60 p-4">
        <h3 className="font-semibold">No inventory changes</h3>
        <p className="mt-1 text-sm text-muted-foreground">No-op rows: {result.noop}</p>
        <p className="mt-1 text-sm">{result.message}</p>
      </section>
    );
  }
  return (
    <section className="mt-5 rounded-xl border border-success/30 bg-success/10 p-4">
      <p className="text-sm font-medium text-success-foreground">{result.message}</p>
      <p className="mt-1 text-sm text-muted-foreground">
        Inserted {result.inserted}. Conflicted {result.conflicted}, rejected {result.rejected}.
      </p>
    </section>
  );
}

function Metadata({ label, value }: { label: string; value: string }) {
  return (
    <div>
      <dt className="text-xs font-medium text-muted-foreground">{label}</dt>
      <dd className="mt-0.5 break-all text-foreground">{value}</dd>
    </div>
  );
}

function ValueList({ values }: { values: Array<string | null> }) {
  const present = values.filter((value): value is string => Boolean(value));
  return present.length ? (
    <ul>
      {present.map((value, index) => (
        <li key={`${value}:${index}`}>{value}</li>
      ))}
    </ul>
  ) : (
    <>None</>
  );
}

function classificationRank(classification: ImportClassification): number {
  if (classification === "conflicted" || classification === "rejected") return 0;
  if (classification === "matched") return 1;
  if (classification === "ignored") return 2;
  return 3;
}

function classificationVariant(classification: ImportClassification): "error" | "outline" | "secondary" | "success" | "warning" {
  if (classification === "inserted") return "success";
  if (classification === "conflicted" || classification === "rejected") return "error";
  if (classification === "ignored") return "warning";
  return classification === "matched" ? "secondary" : "outline";
}

function formatToken(value: string): string {
  return value.replaceAll("_", " ").replace(/^./, (character) => character.toUpperCase());
}

function errorMessage(cause: unknown): string {
  return cause instanceof Error ? cause.message : "Import failed.";
}
