import { CheckIcon, CircleIcon } from "lucide-react";

import { Badge } from "@/shared/components/ui/badge";
import { toSafeExternalUrl } from "@/shared/lib/externalUrl";
import { calibrationHealthLabel, calibrationRequirementLabel, deriveCalibrationHealth, formatLinkLabel } from "@/modules/te-test-equipment/lib";
import { cn } from "@/shared/lib/utils";
import type {
  ColumnConfig,
  InventoryEntry,
  TeTestEquipmentWorkspace,
} from "@/modules/te-test-equipment/types";

interface InventoryTableBodyProps {
  activeEntryId: string | null;
  bottomSpacerHeight: number;
  canModifyEntries: boolean;
  colorRows: boolean;
  columns: readonly ColumnConfig[];
  onOpenContextMenu: (entryId: string, clientX: number, clientY: number) => void;
  onOpenEntry: (entryId: string) => void;
  onOpenExternalLink: (url: string) => void;
  onToggleVerified: (entryId: string) => void;
  topSpacerHeight: number;
  visibleEntries: InventoryEntry[];
  localDate: string;
  workspace: TeTestEquipmentWorkspace;
}

interface InventoryTableRowProps {
  activeEntryId: string | null;
  canModifyEntries: boolean;
  colorRows: boolean;
  columns: readonly ColumnConfig[];
  entry: InventoryEntry;
  localDate: string;
  onOpenContextMenu: (entryId: string, clientX: number, clientY: number) => void;
  onOpenEntry: (entryId: string) => void;
  onOpenExternalLink: (url: string) => void;
  onToggleVerified: (entryId: string) => void;
  workspace: TeTestEquipmentWorkspace;
}

export function InventoryTableBody({
  activeEntryId,
  bottomSpacerHeight,
  canModifyEntries,
  colorRows,
  columns,
  onOpenContextMenu,
  onOpenEntry,
  onOpenExternalLink,
  onToggleVerified,
  topSpacerHeight,
  visibleEntries,
  localDate,
  workspace,
}: InventoryTableBodyProps) {
  return (
    <tbody>
      {topSpacerHeight > 0 ? <SpacerRow colSpan={columns.length} height={topSpacerHeight} /> : null}
      {visibleEntries.map((entry) => (
        <InventoryTableRow
          key={entry.id}
          activeEntryId={activeEntryId}
          canModifyEntries={canModifyEntries}
          colorRows={colorRows}
          columns={columns}
          entry={entry}
          localDate={localDate}
          onOpenContextMenu={onOpenContextMenu}
          onOpenEntry={onOpenEntry}
          onOpenExternalLink={onOpenExternalLink}
          onToggleVerified={onToggleVerified}
          workspace={workspace}
        />
      ))}
      {bottomSpacerHeight > 0 ? <SpacerRow colSpan={columns.length} height={bottomSpacerHeight} /> : null}
    </tbody>
  );
}

function InventoryTableRow({
  activeEntryId,
  canModifyEntries,
  colorRows,
  columns,
  entry,
  localDate,
  onOpenContextMenu,
  onOpenEntry,
  onOpenExternalLink,
  onToggleVerified,
  workspace,
}: InventoryTableRowProps) {
  return (
    <tr
      className={cn(
        rowToneClass(entry, colorRows, workspace, localDate),
        activeEntryId === entry.id ? "ring-1 ring-inset ring-primary/25" : "",
        "cursor-default transition-colors hover:bg-accent/35",
      )}
      onContextMenu={(event) => {
        event.preventDefault();
        onOpenContextMenu(entry.id, event.clientX, event.clientY);
      }}
      onDoubleClick={(event) => {
        if (event.target instanceof Element && event.target.closest("button,a,input")) {
          return;
        }
        onOpenEntry(entry.id);
      }}
    >
      {columns.map((column) => (
        <td
          key={`${entry.id}-${column.key}`}
          className={cn(
            "border-b border-border/60 px-2.5 py-2.5 text-sm text-foreground/92 sm:px-3 sm:py-3",
            column.align === "center" || column.key === "verified"
              ? "text-center align-middle"
              : "text-left align-middle",
          )}
        >
          {column.key === "verified" || column.align === "center" ? (
            <div className="flex w-full items-center justify-center">
              {renderCell(entry, column, onToggleVerified, canModifyEntries, onOpenExternalLink, localDate)}
            </div>
          ) : (
            renderCell(entry, column, onToggleVerified, canModifyEntries, onOpenExternalLink, localDate)
          )}
        </td>
      ))}
    </tr>
  );
}

function SpacerRow({ colSpan, height }: { colSpan: number; height: number }) {
  return (
    <tr aria-hidden="true">
      <td colSpan={colSpan} style={{ height, padding: 0 }} />
    </tr>
  );
}

function renderCell(
  entry: InventoryEntry,
  column: ColumnConfig,
  onToggleVerified: (entryId: string) => void,
  canModifyEntries: boolean,
  onOpenExternalLink: (url: string) => void,
  localDate: string,
) {
  switch (column.key) {
    case "verified": {
      const verified = Boolean(entry.verifiedAt);
      const title = verified ? "Verified" : "Pending verification";
      return (
        <button
          aria-label={
            verified
              ? `Clear verification for ${entry.description}`
              : `Verify ${entry.description}`
          }
          className={cn(
            "inline-flex size-5 shrink-0 aspect-square items-center justify-center rounded border transition-colors",
            verified
              ? "border-emerald-500/35 bg-emerald-500/12 text-emerald-700 hover:bg-emerald-500/18 dark:text-emerald-300"
              : "border-border bg-muted/40 text-muted-foreground hover:bg-muted/70",
          )}
          disabled={!canModifyEntries}
          title={title}
          type="button"
          onClick={() => onToggleVerified(entry.id)}
        >
          {verified ? (
            <CheckIcon aria-hidden className="size-3" />
          ) : (
            <CircleIcon aria-hidden className="size-2.5 opacity-70" />
          )}
        </button>
      );
    }
    case "assetNumber":
      return renderText(entry.assetNumber);
    case "serialNumber":
      return renderText(entry.serialNumber ?? "");
    case "qty":
      return renderText(entry.qty == null ? "" : String(entry.qty));
    case "manufacturer":
      return renderText(entry.manufacturer);
    case "model":
      return renderText(entry.model);
    case "description":
      return renderText(entry.description);
    case "projectName":
      return renderText(entry.projectName);
    case "location":
      return renderText(entry.location);
    case "assignedTo":
      return renderText(entry.assignedTo);
    case "calibrationRequirement":
      return <Badge size="sm" variant="outline">{calibrationRequirementLabel(entry.calibrationRequirement)}</Badge>;
    case "outToCalibration":
      return entry.outToCalibration ? <Badge size="sm" variant="warning">Out to cal</Badge> : renderText("No");
    case "lastCalibratedAt":
      return renderCompactDate(entry.lastCalibratedAt);
    case "calibrationDueAt":
      return renderCompactDate(entry.calibrationDueAt);
    case "calibrationIntervalMonths":
      return renderText(entry.calibrationIntervalMonths == null ? "" : String(entry.calibrationIntervalMonths));
    case "certificateRef":
      return renderText(entry.certificateRef ?? "");
    case "calibrationVendor":
      return renderText(entry.calibrationVendor ?? "");
    case "calibrationNotes":
      return renderText(entry.calibrationNotes ?? "");
    case "calibrationHealth": {
      // Include archived so Archive → Calibration still shows health badges.
      const health = deriveCalibrationHealth(entry, localDate, 30, true);
      if (!health) return renderText("");
      const variant = health === "overdue" || health === "missing_due" ? "error" : health === "due_soon" || health === "out_to_cal" ? "warning" : health === "current" ? "success" : "outline";
      return <Badge size="sm" variant={variant}>{calibrationHealthLabel(health)}</Badge>;
    }
    case "links": {
      const label = formatLinkLabel(entry.links);
      if (!label) {
        return renderText("");
      }
      const safeUrl = toSafeExternalUrl(entry.links);
      if (!safeUrl) {
        return renderText(entry.links.trim());
      }
      return (
        <a
          className="inline-block max-w-full truncate font-mono text-xs text-foreground underline decoration-border underline-offset-4 transition-colors hover:text-primary"
          href={safeUrl}
          rel="noreferrer"
          title={safeUrl}
          onClick={(event) => {
            event.preventDefault();
            onOpenExternalLink(safeUrl);
          }}
        >
          {label}
        </a>
      );
    }
  }
}

function renderText(value: string | null | undefined) {
  const text = value ?? "";
  if (!text.trim()) {
    return <span className="text-muted-foreground">-</span>;
  }
  return (
    <span className="block min-w-0 truncate" title={text}>
      {text}
    </span>
  );
}

/** Compact table display for date-only values: 2026-05-31 → 05/26. Full value stays on title. */
function renderCompactDate(value: string | null | undefined) {
  const full = value?.trim() ?? "";
  if (!full) {
    return <span className="text-muted-foreground">-</span>;
  }
  const match = full.match(/^(\d{4})-(\d{2})(?:-(\d{2}))?/);
  const compact = match ? `${match[2]}/${match[1].slice(2)}` : full;
  return (
    <span className="block whitespace-nowrap tabular-nums" title={full}>
      {compact}
    </span>
  );
}

function rowToneClass(
  entry: InventoryEntry,
  colorRows: boolean,
  workspace: TeTestEquipmentWorkspace,
  localDate: string,
): string {
  if (!colorRows) {
    return "bg-transparent";
  }

  if (workspace === "calibration") {
    // includeArchived: archive tab should still color by last known cal health.
    switch (deriveCalibrationHealth(entry, localDate, 30, true)) {
      case "overdue":
      case "missing_due":
        return "bg-destructive/10";
      case "due_soon":
      case "out_to_cal":
        return "bg-warning/10";
      case "current":
        return "bg-success/10";
      case "not_applicable":
      case "unknown":
      case null:
        return "bg-muted/30";
    }
  }

  switch (entry.lifecycleStatus) {
    case "active":
      return "bg-success/10";
    case "repair":
      return "bg-warning/10";
    case "scrapped":
    case "missing":
      return "bg-destructive/10";
    case "rental":
      return "bg-accent/60";
  }
}
