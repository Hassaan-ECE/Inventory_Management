import { useMemo, useState } from "react";

import { CatalogDialog } from "@/modules/te-lab-components/catalog/CatalogDialog";
import {
  createCatalogLookups,
  formatTotals,
} from "@/modules/te-lab-components/catalog/catalogUtils";
import { StockStatusBadge } from "@/modules/te-lab-components/catalog/PartsTable";
import type {
  CatalogSyncResult,
  LabOrderRequestLineInput,
  Part,
} from "@/modules/te-lab-components/types";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Textarea } from "@/shared/components/ui/textarea";

export interface OrderDraftLine {
  note: string;
  partUuid: string;
  requestedQuantity: string;
}

interface OrderExportDialogProps {
  catalog: CatalogSyncResult;
  drafts: Record<string, OrderDraftLine>;
  onClose: () => void;
  onDraftChange: (partUuid: string, patch: Partial<OrderDraftLine>) => void;
  onExport: (lines: LabOrderRequestLineInput[]) => Promise<void>;
  selectedPartIds: ReadonlySet<string>;
}

function partLabel(part: Part): string {
  const type = part.subcategory || "component";
  return `${type} ${part.displayValue}`.trim();
}

function hasMissingPurchasingFields(part: Part): boolean {
  return (
    !part.manufacturer.trim()
    || !part.manufacturerPartNumber.trim()
    || !part.supplier.trim()
    || !part.supplierSku.trim()
    || !part.productUrl.trim()
  );
}

export function OrderExportDialog({
  catalog,
  drafts,
  onClose,
  onDraftChange,
  onExport,
  selectedPartIds,
}: OrderExportDialogProps) {
  const lookups = useMemo(() => createCatalogLookups(catalog), [catalog]);
  const selectedParts = useMemo(
    () =>
      catalog.parts.filter((part) => selectedPartIds.has(part.entryUuid)),
    [catalog.parts, selectedPartIds],
  );
  const [error, setError] = useState<string | null>(null);
  const [exporting, setExporting] = useState(false);

  async function handleExport(): Promise<void> {
    const lines: LabOrderRequestLineInput[] = [];
    for (const part of selectedParts) {
      const draft = drafts[part.entryUuid] ?? {
        partUuid: part.entryUuid,
        requestedQuantity: "",
        note: "",
      };
      const requestedQuantity = Number(draft.requestedQuantity);
      if (!Number.isInteger(requestedQuantity) || requestedQuantity <= 0) {
        setError(`Enter a positive whole requested quantity for ${partLabel(part)}.`);
        return;
      }
      lines.push({
        partUuid: part.entryUuid,
        requestedQuantity,
        note: draft.note.trim(),
      });
    }
    setError(null);
    setExporting(true);
    try {
      await onExport(lines);
    } catch (exportError) {
      const message =
        exportError instanceof Error
          ? exportError.message
          : "Could not create the order workbook.";
      setError(message);
    } finally {
      setExporting(false);
    }
  }

  return (
    <CatalogDialog
      description="Enter requested quantities for the selected components. Order notes are temporary and are not saved on the part."
      footer={(
        <div className="flex flex-wrap items-center justify-end gap-2">
          {error ? (
            <p className="mr-auto max-w-xl text-sm text-destructive" role="alert">
              {error}
            </p>
          ) : null}
          <Button type="button" variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button disabled={exporting || selectedParts.length === 0} type="button" onClick={() => void handleExport()}>
            Create Order Excel
          </Button>
        </div>
      )}
      title="Prepare Order Request"
      wide
      onClose={onClose}
    >
      <div className="space-y-4">
        {selectedParts.map((part) => {
          const summary = lookups.summariesByPartId.get(part.entryUuid);
          const status = summary?.stockStatus ?? (part.archived ? "archived" : "no_stock");
          const draft = drafts[part.entryUuid] ?? {
            partUuid: part.entryUuid,
            requestedQuantity: "",
            note: "",
          };
          const label = partLabel(part);
          const missingPurchasing = hasMissingPurchasingFields(part);
          const identityBits = [
            part.manufacturer.trim() || null,
            part.manufacturerPartNumber.trim() || null,
          ].filter(Boolean);

          return (
            <section
              className="rounded-xl border border-border bg-card/60 p-4"
              key={part.entryUuid}
            >
              <div className="flex flex-wrap items-start justify-between gap-2">
                <div className="min-w-0 space-y-1">
                  <h3 className="font-semibold text-foreground">
                    {part.subcategory || "Component"} · {part.displayValue || "—"}
                  </h3>
                  {identityBits.length > 0 ? (
                    <p className="text-sm text-muted-foreground">{identityBits.join(" · ")}</p>
                  ) : null}
                  <p className="text-sm text-muted-foreground">
                    Current qty: {summary ? formatTotals(summary) : "0"}
                  </p>
                </div>
                <StockStatusBadge status={status} />
              </div>

              {missingPurchasing ? (
                <p className="mt-3 rounded-lg border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-sm text-amber-900 dark:text-amber-100">
                  Missing manufacturer, MPN, supplier, SKU, or product link.
                </p>
              ) : null}

              <div className="mt-3 grid gap-3 sm:grid-cols-2">
                <label className="block space-y-1.5">
                  <span className="text-sm font-medium">Qty Requested for {label}</span>
                  <Input
                    aria-label={`Qty Requested for ${label}`}
                    inputMode="numeric"
                    min={1}
                    step={1}
                    type="number"
                    value={draft.requestedQuantity}
                    onChange={(event) =>
                      onDraftChange(part.entryUuid, { requestedQuantity: event.target.value })
                    }
                  />
                </label>
                <label className="block space-y-1.5 sm:col-span-2">
                  <span className="text-sm font-medium">Order note for {label}</span>
                  <Textarea
                    aria-label={`Order note for ${label}`}
                    rows={2}
                    value={draft.note}
                    onChange={(event) =>
                      onDraftChange(part.entryUuid, { note: event.target.value })
                    }
                  />
                </label>
              </div>
            </section>
          );
        })}
      </div>
    </CatalogDialog>
  );
}
