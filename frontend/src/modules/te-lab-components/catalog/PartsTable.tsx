import { useEffect, useLayoutEffect, useRef, useState, type MouseEvent as ReactMouseEvent } from "react";

import {
  createCatalogLookups,
  formatTotals,
  partPlacements,
} from "@/modules/te-lab-components/catalog/catalogUtils";
import {
  CATALOG_COLUMNS,
  type CatalogColumnKey,
} from "@/modules/te-lab-components/catalog/catalogColumns";
import type { CatalogSortState } from "@/modules/te-lab-components/catalog/catalogSorting";
import { isOrderSelectable, projectSimpleStock } from "@/modules/te-lab-components/catalog/simpleComponent";
import type { CatalogSyncResult, Part, StockStatus } from "@/modules/te-lab-components/types";
import { Badge } from "@/shared/components/ui/badge";
import { DropdownPanel } from "@/shared/components/ui/DropdownMenu";
import { toSafeExternalUrl } from "@/shared/lib/externalUrl";
import { placeFloatingMenu, type FloatingMenuPlacement } from "@/shared/lib/floatingMenu";
import { cn } from "@/shared/lib/utils";

/** Horizontal edge fade so the sort cue softens into the cell sides (matches TE tables). */
const SORT_EDGE_MASK =
  "[mask-image:linear-gradient(to_right,transparent_0%,black_32%,black_68%,transparent_100%)] [-webkit-mask-image:linear-gradient(to_right,transparent_0%,black_32%,black_68%,transparent_100%)]";

interface PartsTableProps {
  catalog: CatalogSyncResult;
  colorRows: boolean;
  columnVisibility: Record<CatalogColumnKey, boolean>;
  onOpenExternal: (url: string) => void;
  onOpenPart: (part: Part) => void;
  onSortChange: (columnKey: CatalogColumnKey) => void;
  onToggleColumn: (columnKey: CatalogColumnKey) => void;
  onToggleSelection?: (part: Part) => void;
  parts: Part[];
  selectedPartIds?: ReadonlySet<string>;
  selectionMode?: boolean;
  sortState: CatalogSortState | null;
  visibleColumns: Record<CatalogColumnKey, boolean>;
}

interface ColumnMenuState extends FloatingMenuPlacement {
  anchorX: number;
  anchorY: number;
}

export function PartsTable({
  catalog,
  colorRows,
  columnVisibility,
  onOpenExternal,
  onOpenPart,
  onSortChange,
  onToggleColumn,
  onToggleSelection,
  parts,
  selectedPartIds,
  selectionMode = false,
  sortState,
  visibleColumns,
}: PartsTableProps) {
  const lookups = createCatalogLookups(catalog);
  const columns = CATALOG_COLUMNS.filter((column) => visibleColumns[column.key]);
  const [columnMenu, setColumnMenu] = useState<ColumnMenuState | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);
  const selectedIds = selectedPartIds ?? EMPTY_SELECTED_PART_IDS;

  useEffect(() => {
    if (!columnMenu) {
      return undefined;
    }
    function handlePointerDown(event: MouseEvent): void {
      if (!menuRef.current?.contains(event.target as Node)) {
        setColumnMenu(null);
      }
    }
    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        setColumnMenu(null);
      }
    }
    document.addEventListener("mousedown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [columnMenu]);

  useLayoutEffect(() => {
    if (!columnMenu || !menuRef.current) {
      return undefined;
    }

    function refinePlacement(): void {
      const node = menuRef.current;
      if (!node || !columnMenu) {
        return;
      }
      const rect = node.getBoundingClientRect();
      const next = placeFloatingMenu(columnMenu.anchorX, columnMenu.anchorY, {
        width: rect.width,
        height: rect.height,
      });
      if (
        next.x !== columnMenu.x
        || next.y !== columnMenu.y
        || next.maxHeight !== columnMenu.maxHeight
      ) {
        setColumnMenu({
          anchorX: columnMenu.anchorX,
          anchorY: columnMenu.anchorY,
          ...next,
        });
      }
    }

    refinePlacement();
    window.addEventListener("resize", refinePlacement);
    return () => window.removeEventListener("resize", refinePlacement);
  }, [columnMenu]);

  const visibleCount = CATALOG_COLUMNS.filter((column) => columnVisibility[column.key]).length;

  function openColumnMenu(event: ReactMouseEvent): void {
    event.preventDefault();
    event.stopPropagation();
    const placement = placeFloatingMenu(event.clientX, event.clientY);
    setColumnMenu({
      anchorX: event.clientX,
      anchorY: event.clientY,
      ...placement,
    });
  }

  return (
    <div className="relative h-full overflow-auto rounded-xl border border-border bg-card/80 shadow-sm">
      <table className="min-w-full border-separate border-spacing-0 text-left text-xs">
        <thead className="sticky top-0 z-20 bg-card">
          <tr>
            {selectionMode ? (
              <th
                className="relative overflow-hidden whitespace-nowrap border-b border-border bg-card p-0 text-center text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground"
                scope="col"
              >
                <span className="relative z-[1] flex min-h-[2.75rem] w-full items-center justify-center px-1.5 py-2.5 leading-none sm:min-h-[3rem] sm:px-2 sm:py-3">
                  Select
                </span>
              </th>
            ) : null}
            {columns.map((column) => {
              const isActiveSort = sortState?.column === column.key;
              const sortDirection = isActiveSort ? sortState.direction : null;
              const sortLabel = !column.sortable
                ? undefined
                : isActiveSort
                  ? sortState.direction === "asc"
                    ? `Sort by ${column.label}, currently ascending. Activate for descending`
                    : `Sort by ${column.label}, currently descending. Activate to clear sort`
                  : `Sort by ${column.label}`;

              return (
                <th
                  className={cn(
                    // Match TE Equipment/Calibration header chrome (bg-card, full-cell sort hit target).
                    "relative overflow-hidden whitespace-nowrap border-b border-border bg-card p-0 text-center text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground",
                    isActiveSort && "text-foreground",
                  )}
                  key={column.key}
                  scope="col"
                  title="Right-click to show or hide columns"
                  onContextMenu={openColumnMenu}
                >
                  {sortDirection === "asc" ? (
                    <span
                      aria-hidden
                      className={cn(
                        "pointer-events-none absolute inset-x-0 top-0 z-[1] h-2.5 bg-gradient-to-b from-foreground/22 via-foreground/10 to-transparent dark:from-foreground/28 dark:via-foreground/12",
                        SORT_EDGE_MASK,
                      )}
                    />
                  ) : null}
                  {sortDirection === "desc" ? (
                    <span
                      aria-hidden
                      className={cn(
                        "pointer-events-none absolute inset-x-0 bottom-0 z-[1] h-2.5 bg-gradient-to-t from-foreground/22 via-foreground/10 to-transparent dark:from-foreground/28 dark:via-foreground/12",
                        SORT_EDGE_MASK,
                      )}
                    />
                  ) : null}

                  {column.sortable ? (
                    <button
                      aria-label={sortLabel}
                      aria-pressed={isActiveSort ? true : false}
                      className="relative z-[1] flex min-h-[2.75rem] w-full min-w-0 cursor-pointer items-center justify-center px-1.5 py-2.5 transition-colors hover:bg-accent/35 hover:text-foreground sm:min-h-[3rem] sm:px-2 sm:py-3"
                      type="button"
                      onClick={() => onSortChange(column.key)}
                      onContextMenu={openColumnMenu}
                    >
                      <span className="max-w-full truncate leading-none">{column.label}</span>
                    </button>
                  ) : (
                    <span className="relative z-[1] flex min-h-[2.75rem] w-full items-center justify-center px-1.5 py-2.5 leading-none sm:min-h-[3rem] sm:px-2 sm:py-3">
                      <span className="max-w-full truncate">{column.label}</span>
                    </span>
                  )}
                </th>
              );
            })}
          </tr>
        </thead>
        <tbody>
          {parts.map((part) => {
            const summary = lookups.summariesByPartId.get(part.entryUuid);
            const placements = partPlacements(catalog, part.entryUuid);
            const status = summary?.stockStatus ?? (part.archived ? "archived" : "no_stock");
            const selectable = isOrderSelectable(summary);
            const isSelected = selectedIds.has(part.entryUuid);
            const selectionLabel = `Select ${part.subcategory || "component"} ${part.displayValue}`.trim();
            return (
              <tr
                aria-selected={selectionMode ? isSelected : undefined}
                className={cn(
                  "cursor-pointer transition-colors hover:bg-accent/45 focus-within:bg-accent/45",
                  stockRowToneClass(status, colorRows),
                  selectionMode && isSelected && "bg-primary/10 ring-1 ring-inset ring-primary/25",
                )}
                key={part.entryUuid}
                onClick={
                  selectionMode
                    ? () => {
                        if (selectable) {
                          onToggleSelection?.(part);
                        }
                      }
                    : undefined
                }
                onDoubleClick={
                  selectionMode
                    ? undefined
                    : () => onOpenPart(part)
                }
              >
                {selectionMode ? (
                  <td className="border-b border-border/65 px-3 py-2 align-middle">
                    <input
                      aria-label={selectionLabel}
                      checked={isSelected}
                      className="size-4 accent-[var(--primary)]"
                      disabled={!selectable}
                      type="checkbox"
                      onChange={() => {
                        if (selectable) {
                          onToggleSelection?.(part);
                        }
                      }}
                      onClick={(event) => event.stopPropagation()}
                    />
                  </td>
                ) : null}
                {columns.map((column) => (
                  <td className="max-w-[24rem] border-b border-border/65 px-3 py-2 align-top" key={column.key}>
                    <CatalogCell
                      catalog={catalog}
                      column={column.key}
                      onOpenExternal={onOpenExternal}
                      part={part}
                      placements={placements}
                      status={status}
                      totals={summary ? formatTotals(summary) : "0"}
                    />
                  </td>
                ))}
              </tr>
            );
          })}
        </tbody>
      </table>

      {columnMenu ? (
        <div
          className="fixed z-[60]"
          ref={menuRef}
          style={{ left: columnMenu.x, top: columnMenu.y }}
        >
          <DropdownPanel
            align="left"
            className="relative right-auto mt-0 w-72"
            maxHeightPx={columnMenu.maxHeight}
            title="Columns"
          >
            {CATALOG_COLUMNS.map((column) => {
              const isLastVisible = columnVisibility[column.key] && visibleCount === 1;
              return (
                <label
                  key={column.key}
                  className={
                    isLastVisible
                      ? "flex cursor-not-allowed items-center justify-between rounded-xl px-3 py-2 text-sm text-muted-foreground opacity-60"
                      : "flex cursor-pointer items-center justify-between rounded-xl px-3 py-2 text-sm text-foreground hover:bg-accent/60"
                  }
                >
                  <span>{column.label}</span>
                  <input
                    aria-label={column.label}
                    checked={columnVisibility[column.key]}
                    className="size-4 accent-[var(--primary)]"
                    disabled={isLastVisible}
                    type="checkbox"
                    onChange={() => onToggleColumn(column.key)}
                  />
                </label>
              );
            })}
          </DropdownPanel>
        </div>
      ) : null}
    </div>
  );
}

interface CatalogCellProps {
  catalog: CatalogSyncResult;
  column: CatalogColumnKey;
  onOpenExternal: (url: string) => void;
  part: Part;
  placements: ReturnType<typeof partPlacements>;
  status: StockStatus;
  totals: string;
}

function CatalogCell({ catalog, column, onOpenExternal, part, placements, status, totals }: CatalogCellProps) {
  const lookups = createCatalogLookups(catalog);
  switch (column) {
    case "stockStatus":
      return <StockStatusBadge status={status} />;
    case "category":
      return part.category || "Uncategorized";
    case "manufacturerPartNumber":
      return <span className="font-medium">{part.manufacturerPartNumber || "—"}</span>;
    case "displayValue":
      return part.displayValue || "—";
    case "packageType":
      return part.packageType || "—";
    case "mountingType":
      return part.mountingType.replaceAll("_", " ") || "—";
    case "totals":
      return <span className="whitespace-nowrap font-semibold tabular-nums">{totals}</span>;
    case "locations": {
      const projection = projectSimpleStock(placements, lookups.containersById);
      if (projection.kind === "simple") {
        return projection.location ? (
          <span className="font-medium tabular-nums">{projection.location}</span>
        ) : (
          <span className="text-muted-foreground">—</span>
        );
      }
      const reviewLabel = projection.reasons.includes("multiple_placements")
        ? `Review · ${placements.length} locations`
        : "Review units";
      return (
        <span className="text-amber-800 dark:text-amber-200" title={projection.locationLabel}>
          {reviewLabel}
        </span>
      );
    }
    case "manufacturer":
      return part.manufacturer || "—";
    case "links": {
      const links = [
        part.productUrl ? { url: part.productUrl, kind: "Product" as const } : null,
        part.datasheetUrl ? { url: part.datasheetUrl, kind: "Datasheet" as const } : null,
      ].filter((link): link is { url: string; kind: "Product" | "Datasheet" } => Boolean(link));

      if (links.length === 0) {
        return <span className="text-muted-foreground">—</span>;
      }

      return (
        <div className="flex min-w-0 flex-col gap-0.5">
          {links.map((link) => {
            const safeUrl = toSafeExternalUrl(link.url);
            const label = formatCatalogLinkLabel(link.url) || link.kind;
            if (!safeUrl) {
              return (
                <span className="block min-w-0 truncate font-mono text-xs text-muted-foreground" key={`${link.kind}-${link.url}`} title={link.url}>
                  {label}
                </span>
              );
            }
            return (
              <a
                className="inline-block max-w-full truncate font-mono text-xs text-foreground underline decoration-border underline-offset-4 transition-colors hover:text-primary"
                href={safeUrl}
                key={`${link.kind}-${safeUrl}`}
                rel="noreferrer"
                title={`${link.kind}: ${safeUrl}`}
                onClick={(event) => {
                  event.preventDefault();
                  event.stopPropagation();
                  onOpenExternal(safeUrl);
                }}
              >
                {label}
              </a>
            );
          })}
        </div>
      );
    }
    case "internalPartNumber":
      return part.internalPartNumber || "—";
    case "subcategory":
      return part.subcategory || "—";
    case "supplier":
      return part.supplier || "—";
    case "supplierSku":
      return part.supplierSku || "—";
    case "partStatus":
      return part.partStatus.replaceAll("_", " ");
  }
}

function formatCatalogLinkLabel(link: string): string {
  const text = link.trim();
  if (!text) {
    return "";
  }
  try {
    const parsed = new URL(text);
    const compact = `${parsed.host}${parsed.pathname.replace(/\/$/, "")}`;
    if (compact.length <= 40) {
      return compact;
    }
    return `${compact.slice(0, 37)}...`;
  } catch {
    if (text.length <= 40) {
      return text;
    }
    return `${text.slice(0, 37)}...`;
  }
}

export function StockStatusBadge({ status }: { status: StockStatus }) {
  const labels: Record<StockStatus, string> = {
    archived: "Archived",
    no_stock: "No stock",
    unit_review: "Unit review",
    low_stock: "Low stock",
    in_stock: "In stock",
    mixed_units: "Mixed units",
  };
  return (
    <Badge
      className={cn(
        status === "in_stock" && "border-emerald-500/25 bg-emerald-500/10 text-emerald-700 dark:text-emerald-300",
        status === "low_stock" && "border-amber-500/30 bg-amber-500/10 text-amber-800 dark:text-amber-200",
        status === "no_stock" && "border-rose-500/25 bg-rose-500/10 text-rose-700 dark:text-rose-300",
        (status === "unit_review" || status === "mixed_units") &&
          "border-sky-500/25 bg-sky-500/10 text-sky-700 dark:text-sky-300",
      )}
      variant="outline"
    >
      {labels[status]}
    </Badge>
  );
}

function stockRowToneClass(status: StockStatus, colorRows: boolean): string {
  if (!colorRows) {
    return "bg-background/55";
  }
  switch (status) {
    case "in_stock":
      return "bg-success/10";
    case "low_stock":
      return "bg-warning/10";
    case "no_stock":
    case "archived":
      return "bg-destructive/10";
    case "unit_review":
    case "mixed_units":
      return "bg-sky-500/10";
    default:
      return "bg-background/55";
  }
}

const EMPTY_SELECTED_PART_IDS: ReadonlySet<string> = new Set();
