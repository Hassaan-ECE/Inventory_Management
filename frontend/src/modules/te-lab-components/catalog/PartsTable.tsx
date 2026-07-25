import { ExternalLinkIcon, FileTextIcon, PencilIcon } from "lucide-react";

import {
  createCatalogLookups,
  formatQuantity,
  formatTotals,
  partPlacements,
  placementPath,
} from "@/modules/te-lab-components/catalog/catalogUtils";
import {
  CATALOG_COLUMNS,
  type CatalogColumnKey,
} from "@/modules/te-lab-components/catalog/catalogColumns";
import type { CatalogSyncResult, Part, StockStatus } from "@/modules/te-lab-components/types";
import { Badge } from "@/shared/components/ui/badge";
import { Button } from "@/shared/components/ui/button";
import { cn } from "@/shared/lib/utils";

interface PartsTableProps {
  catalog: CatalogSyncResult;
  onOpenExternal: (url: string) => void;
  onOpenPart: (part: Part) => void;
  parts: Part[];
  visibleColumns: Record<CatalogColumnKey, boolean>;
}

export function PartsTable({ catalog, onOpenExternal, onOpenPart, parts, visibleColumns }: PartsTableProps) {
  const lookups = createCatalogLookups(catalog);
  const columns = CATALOG_COLUMNS.filter((column) => visibleColumns[column.key]);

  return (
    <div className="h-full overflow-auto rounded-xl border border-border bg-card/80 shadow-sm">
      <table className="min-w-full border-separate border-spacing-0 text-left text-xs">
        <thead className="sticky top-0 z-10 bg-muted/95 backdrop-blur">
          <tr>
            {columns.map((column) => (
              <th className="whitespace-nowrap border-b border-border px-3 py-2.5 font-semibold" key={column.key} scope="col">
                {column.label}
              </th>
            ))}
            <th className="w-16 border-b border-border px-3 py-2.5" scope="col">
              Actions
            </th>
          </tr>
        </thead>
        <tbody>
          {parts.map((part) => {
            const summary = lookups.summariesByPartId.get(part.entryUuid);
            const placements = partPlacements(catalog, part.entryUuid);
            return (
              <tr
                className="cursor-pointer bg-background/55 hover:bg-accent/45 focus-within:bg-accent/45"
                key={part.entryUuid}
                onDoubleClick={() => onOpenPart(part)}
              >
                {columns.map((column) => (
                  <td className="max-w-[24rem] border-b border-border/65 px-3 py-2 align-top" key={column.key}>
                    <CatalogCell
                      catalog={catalog}
                      column={column.key}
                      onOpenExternal={onOpenExternal}
                      part={part}
                      placements={placements}
                      status={summary?.stockStatus ?? (part.archived ? "archived" : "no_stock")}
                      totals={summary ? formatTotals(summary) : "0"}
                    />
                  </td>
                ))}
                <td className="border-b border-border/65 px-3 py-2 align-top">
                  <Button aria-label={`Edit ${part.manufacturerPartNumber || part.displayValue || part.id}`} onClick={() => onOpenPart(part)} size="xs" variant="ghost">
                    <PencilIcon className="size-3.5" />
                    Edit
                  </Button>
                </td>
              </tr>
            );
          })}
        </tbody>
      </table>
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
      return (
        <div>
          <div className="font-medium">{part.category || "Uncategorized"}</div>
          {part.subcategory ? <div className="text-muted-foreground">{part.subcategory}</div> : null}
        </div>
      );
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
      if (placements.length === 0) {
        return <span className="text-muted-foreground">No placements</span>;
      }
      const first = placements[0];
      return (
        <div className="space-y-1" title={placements.map((placement) => placementPath(placement, lookups)).join("\n")}>
          <span className="inline-flex max-w-[22rem] items-center rounded-full border border-border bg-muted/50 px-2 py-0.5">
            <span className="truncate">{placementPath(first, lookups)}</span>
            <span className="ml-1 whitespace-nowrap text-muted-foreground">
              ({formatQuantity(first.quantity)} {first.unitOfMeasure})
            </span>
          </span>
          {placements.length > 1 ? <div className="text-[11px] text-muted-foreground">+{placements.length - 1} more</div> : null}
        </div>
      );
    }
    case "manufacturer":
      return part.manufacturer || "—";
    case "documents":
      return (
        <div className="flex gap-1">
          {part.datasheetUrl ? (
            <Button
              aria-label={`Open datasheet for ${part.manufacturerPartNumber || part.id}`}
              onClick={(event) => {
                event.stopPropagation();
                onOpenExternal(part.datasheetUrl);
              }}
              size="xs"
              variant="ghost"
            >
              <FileTextIcon className="size-3.5" />
              Datasheet
            </Button>
          ) : null}
          {part.productUrl ? (
            <Button
              aria-label={`Open product page for ${part.manufacturerPartNumber || part.id}`}
              onClick={(event) => {
                event.stopPropagation();
                onOpenExternal(part.productUrl);
              }}
              size="xs"
              variant="ghost"
            >
              <ExternalLinkIcon className="size-3.5" />
              Product
            </Button>
          ) : null}
          {!part.productUrl && !part.datasheetUrl ? <span className="text-muted-foreground">—</span> : null}
        </div>
      );
    case "internalPartNumber":
      return part.internalPartNumber || "—";
    case "subcategory":
      return part.subcategory || "—";
    case "supplier":
      return part.supplier || "—";
    case "supplierSku":
      return part.supplierSku || "—";
    case "reorderPoint":
      return part.reorderPoint === null ? "—" : `${formatQuantity(part.reorderPoint)} ${part.defaultUnitOfMeasure}`;
    case "targetQuantity":
      return part.targetQuantity === null ? "—" : `${formatQuantity(part.targetQuantity)} ${part.defaultUnitOfMeasure}`;
    case "partStatus":
      return part.partStatus.replaceAll("_", " ");
    case "updatedAt":
      return part.updatedAt ? new Date(part.updatedAt).toLocaleDateString() : "—";
    case "archived":
      return part.archived ? "Yes" : "No";
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
