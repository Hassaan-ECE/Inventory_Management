import { useEffect, useRef, useState } from "react";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Textarea } from "@/shared/components/ui/textarea";
import { UnsavedChangesDialog } from "@/shared/components/ui/UnsavedChangesDialog";
import { entryInput, STORAGE_COLUMNS, type StorageEntry, type StorageInput } from "./types";

interface Props {
  entry: StorageEntry | null;
  onClose: () => void;
  onSave: (input: StorageInput, original: StorageEntry | null) => Promise<void>;
  onDelete: (entry: StorageEntry) => Promise<void>;
}
export function StorageEntryDialog({ entry, onClose, onSave, onDelete }: Props) {
  const [form, setForm] = useState(() => ({ ...entryInput(entry), qty: entry?.qty?.toString() ?? "" }));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState("");
  const [confirmClose, setConfirmClose] = useState(false);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const panel = useRef<HTMLDivElement>(null);
  const [initial] = useState(() => JSON.stringify(form));
  const dirty = JSON.stringify(form) !== initial;
  const close = () => { if (!busy) { if (dirty) setConfirmClose(true); else onClose(); } };

  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    panel.current?.querySelector<HTMLInputElement>("input")?.focus();
    return () => previous?.focus();
  }, []);

  async function save() {
    const qty = form.qty.trim() === "" ? null : Number(form.qty);
    if (qty !== null && (!Number.isFinite(qty) || qty < 0 || qty > 1_000_000)) {
      setError("Qty must be a number between 0 and 1000000."); return;
    }
    const input = Object.fromEntries(Object.entries({ ...form, qty }).map(([key, value]) => [key, typeof value === "string" ? value.trim() : value])) as StorageInput;
    if (![input.pn, input.pr, input.po, input.manufacturer, input.model, input.description].some(Boolean)) {
      setError("Enter a PN #, PR #, PO #, manufacturer, model, or description before saving."); return;
    }
    setBusy(true); setError("");
    try { await onSave(input, entry); onClose(); }
    catch (cause) { setError(String(cause)); setConfirmClose(false); }
    finally { setBusy(false); }
  }
  async function remove() {
    if (!entry) return;
    setBusy(true); setError("");
    try { await onDelete(entry); onClose(); }
    catch (cause) { setError(String(cause)); setConfirmDelete(false); }
    finally { setBusy(false); }
  }

  return (
    <div className="fixed inset-0 z-[100] flex items-center justify-center bg-black/45 p-4" onMouseDown={event => { if (event.target === event.currentTarget) close(); }}>
      <div ref={panel} role="dialog" aria-modal="true" aria-labelledby="storage-editor-title"
        className="flex max-h-[90dvh] w-full max-w-2xl flex-col overflow-auto rounded-2xl border border-border bg-card p-5 text-card-foreground shadow-2xl"
        onKeyDown={event => {
          if (event.key === "Escape") { event.stopPropagation(); if (confirmClose) setConfirmClose(false); else if (confirmDelete) setConfirmDelete(false); else close(); }
          if (event.key === "Tab") {
            const elements = Array.from(panel.current?.querySelectorAll<HTMLElement>('button:not(:disabled), input:not(:disabled), textarea:not(:disabled)') ?? []);
            const first = elements[0]; const last = elements.at(-1);
            if (event.shiftKey && document.activeElement === first) { event.preventDefault(); last?.focus(); }
            else if (!event.shiftKey && document.activeElement === last) { event.preventDefault(); first?.focus(); }
          }
        }}>
        <h2 id="storage-editor-title" className="text-xl font-semibold">{entry ? "Edit storage item" : "Add storage item"}</h2>
        <p className="mt-1 text-sm text-muted-foreground">PR # and Description are saved even when their table columns are hidden.</p>
        <form className="mt-5" onSubmit={event => { event.preventDefault(); void save(); }}>
          <fieldset disabled={busy} className="grid grid-cols-1 gap-4 sm:grid-cols-2">
            {STORAGE_COLUMNS.map(({ key, label }) => (
              <label key={key} className={`flex flex-col gap-1.5 text-sm font-medium ${key === "notes" || key === "description" ? "sm:col-span-2" : ""}`}>
                {label}
                {key === "notes" || key === "description" ? (
                  <Textarea value={form[key]} maxLength={key === "notes" ? 8000 : 4000} rows={key === "notes" ? 3 : 2}
                    onChange={event => setForm(current => ({ ...current, [key]: event.target.value }))} />
                ) : (
                  <Input value={form[key]} type={key === "qty" ? "number" : "text"} min={key === "qty" ? 0 : undefined}
                    max={key === "qty" ? 1_000_000 : undefined} step={key === "qty" ? "any" : undefined} maxLength={512}
                    onChange={event => setForm(current => ({ ...current, [key]: event.target.value }))} />
                )}
              </label>
            ))}
          </fieldset>
          {error ? <p role="alert" className="mt-4 text-sm text-destructive-foreground">{error}</p> : null}
          {confirmDelete ? (
            <div role="alertdialog" aria-label="Confirm item deletion" className="mt-4 rounded-xl border border-destructive/30 p-3">
              <p className="text-sm">Delete this item from the inventory? This deletion will sync to the team.</p>
              <div className="mt-3 flex gap-2">
                <Button type="button" variant="destructive" disabled={busy} onClick={() => { void remove(); }}>Confirm delete</Button>
                <Button type="button" variant="outline" disabled={busy} onClick={() => setConfirmDelete(false)}>Keep item</Button>
              </div>
            </div>
          ) : null}
          <div className="mt-5 flex items-center justify-end gap-2">
            {entry ? <Button className="mr-auto" type="button" variant="ghost" disabled={busy} onClick={() => setConfirmDelete(true)}>Delete item</Button> : null}
            <Button type="button" variant="outline" disabled={busy} onClick={close}>Cancel</Button>
            <Button type="submit" disabled={busy || (Boolean(entry) && !dirty)}>{busy ? "Saving…" : "Save item"}</Button>
          </div>
        </form>
        <UnsavedChangesDialog open={confirmClose} isSaving={busy} onCancel={() => setConfirmClose(false)} onDiscard={onClose} onSave={() => { void save(); }} />
      </div>
    </div>
  );
}
