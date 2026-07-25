import { useState } from "react";
import { AlertTriangleIcon, CheckCircle2Icon, RefreshCwIcon } from "lucide-react";

import { CatalogDialog } from "@/modules/te-lab-components/catalog/CatalogDialog";
import type {
  CatalogSharedCutoverCommitResult,
  CatalogSharedCutoverPreview,
} from "@/modules/te-lab-components/types";
import { Button } from "@/shared/components/ui/button";

interface SharedCutoverDialogProps {
  onClose: () => void;
  onCommit: (preview: CatalogSharedCutoverPreview) => Promise<CatalogSharedCutoverCommitResult>;
  onPreview: () => Promise<CatalogSharedCutoverPreview>;
}

export function SharedCutoverDialog({ onClose, onCommit, onPreview }: SharedCutoverDialogProps) {
  const [preview, setPreview] = useState<CatalogSharedCutoverPreview | null>(null);
  const [result, setResult] = useState<CatalogSharedCutoverCommitResult | null>(null);
  const [confirmed, setConfirmed] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function runPreview(): Promise<void> {
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      setPreview(await onPreview());
      setConfirmed(false);
    } catch (previewError) {
      setError(previewError instanceof Error ? previewError.message : "Could not preview shared cutover.");
    } finally {
      setBusy(false);
    }
  }

  async function commit(): Promise<void> {
    if (!preview || !confirmed) return;
    setBusy(true);
    setError(null);
    try {
      setResult(await onCommit(preview));
    } catch (commitError) {
      setError(commitError instanceof Error ? commitError.message : "Could not commit shared cutover.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <CatalogDialog
      description="This explicitly initializes the Lab-only catalog-v2 shared stream and blocks old Lab v1 writers."
      footer={<div className="flex justify-end gap-2"><Button onClick={onClose} variant="outline">Close</Button>{preview && !preview.blocking && !result ? <Button disabled={!confirmed || busy} onClick={() => void commit()}>{busy ? "Cutting over…" : "Commit Shared Cutover"}</Button> : null}</div>}
      onClose={onClose}
      title="Lab Shared Catalog Cutover"
    >
      <div className="space-y-4">
        <div className="flex items-center justify-between gap-3 rounded-xl border border-border bg-muted/20 p-3">
          <p className="text-sm text-muted-foreground">Preview is read-only and tied to the current local catalog fingerprint.</p>
          <Button disabled={busy} onClick={() => void runPreview()} variant="outline"><RefreshCwIcon className={`size-3.5 ${busy ? "animate-spin" : ""}`} /> Preview</Button>
        </div>
        {error ? <div className="rounded-xl border border-destructive/30 bg-destructive/8 p-3 text-sm text-destructive-foreground" role="alert">{error}</div> : null}
        {result ? <div className="rounded-xl border border-emerald-500/35 bg-emerald-500/10 p-3 text-sm"><div className="flex items-center gap-2 font-semibold"><CheckCircle2Icon className="size-4" /> Cutover complete</div><p className="mt-1">{result.message}</p><p className="mt-1 break-all text-xs text-muted-foreground">Catalog root: {result.catalogRootPath}{result.legacyBackupPath ? ` · Legacy backup: ${result.legacyBackupPath}` : ""}</p></div> : null}
        {preview ? <>
          <div className="grid gap-2 sm:grid-cols-2"><Metric label="Parts" value={preview.partCount} /><Metric label="Placements" value={preview.placementCount} /><Metric label="Areas" value={preview.areaCount} /><Metric label="Containers" value={preview.containerCount} /></div>
          <div className="rounded-xl border border-border bg-background p-3 text-sm"><div><strong>Shared root:</strong> <span className="break-all">{preview.sharedRootPath}</span></div><div className="mt-1"><strong>Legacy stream:</strong> {preview.legacyStreamState}</div><div className="mt-1"><strong>Catalog v2 initialized:</strong> {preview.catalogV2Initialized ? "Yes" : "No"}</div></div>
          {preview.warnings.length ? <ul className="list-disc space-y-1 rounded-xl border border-amber-500/35 bg-amber-500/8 p-3 pl-8 text-sm">{preview.warnings.map((warning) => <li key={warning}>{warning}</li>)}</ul> : null}
          {preview.blocking ? <div className="flex gap-2 rounded-xl border border-destructive/35 bg-destructive/8 p-3 text-sm text-destructive-foreground"><AlertTriangleIcon className="mt-0.5 size-4 shrink-0" /> Cutover is blocked. Resolve the reported shared-root or catalog condition and preview again.</div> : <label className="flex items-start gap-2 rounded-xl border border-amber-500/30 bg-amber-500/8 p-3 text-sm"><input checked={confirmed} className="mt-0.5" onChange={(event) => setConfirmed(event.target.checked)} type="checkbox" /> I confirmed all standalone/unified Lab writers are stopped and reviewed this exact fingerprint.</label>}
          <div className="rounded-xl border border-border bg-muted/20 p-3 text-xs text-muted-foreground">Fingerprint: <code className="break-all">{preview.localFingerprint}</code></div>
        </> : <div className="rounded-xl border border-dashed border-border p-7 text-center text-sm text-muted-foreground">Run preview before enabling the catalog-v2 shared stream.</div>}
      </div>
    </CatalogDialog>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return <div className="rounded-xl border border-border bg-background p-3"><div className="text-xl font-semibold tabular-nums">{value.toLocaleString()}</div><div className="text-xs text-muted-foreground">{label}</div></div>;
}
