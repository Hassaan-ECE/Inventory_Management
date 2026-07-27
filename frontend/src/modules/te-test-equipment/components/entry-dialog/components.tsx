import { ArchiveRestoreIcon, ExternalLinkIcon, FolderOpenIcon, ImageIcon, ImageOffIcon, Trash2Icon } from "lucide-react";

import { Badge } from "@/shared/components/ui/badge";
import { Button } from "@/shared/components/ui/button";
import { cn } from "@/shared/lib/utils";

import { handlePreviewKeyDown, type PicturePreviewState } from "./picturePreview";

interface PicturePreviewCardProps {
  canBrowse: boolean;
  canOpen: boolean;
  compact?: boolean;
  picturePath: string;
  previewSrc: string | null;
  previewState: PicturePreviewState;
  onBrowse: () => void;
  onOpen: () => void;
  onPreviewError: (previewSrc: string) => void;
  onPreviewLoad: (previewSrc: string) => void;
}

export function PicturePreviewCard({
  canBrowse,
  canOpen,
  compact = false,
  picturePath,
  previewSrc,
  previewState,
  onBrowse,
  onOpen,
  onPreviewError,
  onPreviewLoad,
}: PicturePreviewCardProps) {
  const trimmedPath = picturePath.trim();
  const hasPicture = Boolean(trimmedPath);

  return (
    <section className="rounded-2xl border border-border/70 bg-background/70 px-4 py-4">
      <div className="flex items-start justify-between gap-3">
        <div>
          <p className="text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">Picture Preview</p>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          {hasPicture ? (
            <Badge variant={previewState === "loaded" ? "success" : previewState === "missing" ? "warning" : "outline"}>
              {previewState === "loaded" ? "Ready" : previewState === "missing" ? "Missing" : "Selected"}
            </Badge>
          ) : null}
          <Button
            disabled={!canBrowse}
            size="sm"
            title={canBrowse ? "Browse for an entry picture" : "Desktop file picker unavailable"}
            variant="outline"
            onClick={onBrowse}
          >
            <FolderOpenIcon className="size-3.5" />
            Browse
          </Button>
        </div>
      </div>

      <div
        aria-disabled={!canOpen}
        aria-label={hasPicture ? "Picture preview" : "Picture preview unavailable"}
        className={cn(
          "group relative mt-3 flex overflow-hidden rounded-2xl border border-border/70 bg-card/70",
          compact ? "min-h-[14rem]" : "min-h-[17rem]",
          canOpen ? "cursor-zoom-in hover:border-primary/35" : "cursor-default",
        )}
        role={canOpen ? "button" : undefined}
        tabIndex={canOpen ? 0 : undefined}
        title={canOpen ? "Double-click to open in the default image viewer" : undefined}
        onDoubleClick={() => {
          if (canOpen) {
            onOpen();
          }
        }}
        onKeyDown={(event) => handlePreviewKeyDown(event, canOpen, onOpen)}
      >
        {previewSrc && previewState !== "missing" ? (
          <>
            <img
              alt="Entry picture preview"
              className={cn(
                "h-full w-full object-contain bg-background/40 transition-opacity",
                previewState === "loaded" ? "opacity-100" : "opacity-0",
              )}
              src={previewSrc}
              onError={() => onPreviewError(previewSrc)}
              onLoad={() => onPreviewLoad(previewSrc)}
            />
            {previewState !== "loaded" ? (
              <PreviewPlaceholder icon={ImageIcon} label="Loading preview..." />
            ) : canOpen ? (
              <div className="pointer-events-none absolute right-3 top-3 rounded-full bg-card/90 p-2 text-foreground shadow-sm">
                <ExternalLinkIcon className="size-4" />
              </div>
            ) : null}
          </>
        ) : (
          <PreviewPlaceholder icon={hasPicture ? ImageOffIcon : ImageIcon} label={hasPicture ? "Picture not found" : "No picture selected"} />
        )}
      </div>
    </section>
  );
}

interface PreviewPlaceholderProps {
  icon: typeof ImageIcon;
  label: string;
}

function PreviewPlaceholder({ icon: Icon, label }: PreviewPlaceholderProps) {
  return (
    <div className="absolute inset-0 flex flex-col items-center justify-center gap-2 px-4 text-center text-sm text-muted-foreground">
      <Icon className="size-7" />
      <p>{label}</p>
    </div>
  );
}

interface ContextRowProps {
  label: string;
  value: string;
}

export function ContextRow({ label, value }: ContextRowProps) {
  return (
    <div className="rounded-2xl border border-border/70 bg-card/70 px-3 py-3">
      <p className="text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">{label}</p>
      <p className="mt-1 text-sm text-foreground">{value}</p>
    </div>
  );
}

/** Soft amber archive control — less intense than full warning solid. */
export const ARCHIVE_BUTTON_CLASS =
  "border-warning/30 bg-warning/12 text-warning-foreground hover:bg-warning/18";

interface DialogActionsProps {
  archived?: boolean;
  error: string | null;
  formId: string;
  isSaving: boolean;
  /** Edit mode: archive / restore the entry (sets archived flag; user still saves if needed). */
  onArchiveToggle?: () => void;
  /** Edit mode + archived only: hard delete. */
  onDelete?: () => void;
  readOnly: boolean;
  onClose: () => void;
}

export function DialogActions({
  archived = false,
  error,
  formId,
  isSaving,
  onArchiveToggle,
  onDelete,
  readOnly,
  onClose,
}: DialogActionsProps) {
  return (
    <div className="flex flex-wrap items-center justify-between gap-3">
      <div className="flex flex-wrap gap-2">
        {onArchiveToggle ? (
          archived ? (
            <>
              <Button
                className="border-success/30 bg-success/12 text-success-foreground hover:bg-success/18"
                disabled={readOnly || isSaving}
                type="button"
                variant="outline"
                onClick={onArchiveToggle}
              >
                <ArchiveRestoreIcon className="size-3.5" />
                Restore to Inventory
              </Button>
              {onDelete ? (
                <Button
                  disabled={readOnly || isSaving}
                  type="button"
                  variant="destructive"
                  onClick={onDelete}
                >
                  <Trash2Icon className="size-3.5" />
                  Delete Entry
                </Button>
              ) : null}
            </>
          ) : (
            <Button
              className={ARCHIVE_BUTTON_CLASS}
              disabled={readOnly || isSaving}
              type="button"
              variant="outline"
              onClick={onArchiveToggle}
            >
              <ArchiveRestoreIcon className="size-3.5" />
              Archive Entry
            </Button>
          )
        ) : (
          <span className="text-xs text-muted-foreground">
            {error ? null : "Changes apply when you save."}
          </span>
        )}
      </div>
      <div className="flex flex-wrap items-center justify-end gap-2">
        {error ? <p className="mr-auto text-sm text-destructive-foreground sm:mr-2">{error}</p> : null}
        <Button disabled={isSaving} type="button" variant="outline" onClick={onClose}>
          Cancel
        </Button>
        <Button disabled={readOnly || isSaving} form={formId} type="submit">
          {isSaving ? "Saving..." : "Save Entry"}
        </Button>
      </div>
    </div>
  );
}
