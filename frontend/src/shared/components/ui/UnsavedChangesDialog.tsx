import { Button } from "@/shared/components/ui/button";

interface UnsavedChangesDialogProps {
  isSaving?: boolean;
  onCancel: () => void;
  onDiscard: () => void;
  onSave: () => void;
  open: boolean;
  readOnly?: boolean;
}

/** Prompt when closing an editor with unsaved edits: Save, Discard, or Cancel. */
export function UnsavedChangesDialog({
  isSaving = false,
  onCancel,
  onDiscard,
  onSave,
  open,
  readOnly = false,
}: UnsavedChangesDialogProps) {
  if (!open) {
    return null;
  }

  return (
    <div
      aria-describedby="unsaved-changes-desc"
      aria-labelledby="unsaved-changes-title"
      aria-modal="true"
      className="fixed inset-0 z-[120] flex items-center justify-center bg-black/45 p-4"
      role="alertdialog"
      onClick={(event) => {
        if (event.target === event.currentTarget && !isSaving) {
          onCancel();
        }
      }}
    >
      <div className="w-full max-w-md rounded-2xl border border-border bg-card p-5 text-card-foreground shadow-2xl">
        <h3 id="unsaved-changes-title" className="text-lg font-semibold text-foreground">
          Unsaved changes
        </h3>
        <p id="unsaved-changes-desc" className="mt-2 text-sm text-muted-foreground">
          You have unsaved changes. Save them, discard them, or cancel to keep editing.
        </p>
        <div className="mt-5 flex flex-wrap items-center justify-end gap-2">
          <Button disabled={isSaving} type="button" variant="outline" onClick={onCancel}>
            Cancel
          </Button>
          <Button disabled={isSaving} type="button" variant="ghost" onClick={onDiscard}>
            Discard
          </Button>
          <Button disabled={isSaving || readOnly} type="button" onClick={onSave}>
            {isSaving ? "Saving…" : "Save"}
          </Button>
        </div>
      </div>
    </div>
  );
}
