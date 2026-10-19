import type { Dispatch, SetStateAction } from "react";
import { ArchiveRestoreIcon, BoxesIcon, CalculatorIcon, MapPinIcon, MoveRightIcon, PlusIcon, Trash2Icon } from "lucide-react";

import { formatTotals, placementPath, type createCatalogLookups } from "@/modules/te-lab-components/catalog/catalogUtils";
import {
  CATEGORY_TEMPLATES,
  categoryTemplate,
  humanizeAttributeKey,
  normalizeAttributeKey,
} from "@/modules/te-lab-components/catalog/categoryTemplates";
import { Field, FormSection, SELECT_CLASS } from "@/modules/te-lab-components/catalog/formControls";
import { numberOrNull, type AttributeRow } from "@/modules/te-lab-components/catalog/partForm";
import { StockStatusBadge } from "@/modules/te-lab-components/catalog/PartsTable";
import type { Part, PartInput, PartStockSummary, StockPlacement } from "@/modules/te-lab-components/types";
import { Badge } from "@/shared/components/ui/badge";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Textarea } from "@/shared/components/ui/textarea";

interface PlacementActions {
  onAddPlacement: (part: Part) => void;
  onCountPlacement: (placement: StockPlacement) => void;
  onDeletePlacement: (placement: StockPlacement) => void;
  onEditPlacement: (placement: StockPlacement) => void;
  onMovePlacement: (placement: StockPlacement) => void;
}

interface PartDetailsSectionsProps extends PlacementActions {
  attributes: AttributeRow[];
  form: PartInput;
  lookups: ReturnType<typeof createCatalogLookups>;
  onAttributesChange: Dispatch<SetStateAction<AttributeRow[]>>;
  onFieldChange: <K extends keyof PartInput>(key: K, value: PartInput[K]) => void;
  part: Part | null;
  placements: StockPlacement[];
  readOnly: boolean;
  summary: PartStockSummary | undefined;
}

/** The part dialog's "More Details": every catalog field and the part's stock placements. */
export function PartDetailsSections({
  attributes,
  form,
  lookups,
  onAttributesChange,
  onFieldChange: update,
  part,
  placements,
  readOnly,
  summary,
  ...placementActions
}: PartDetailsSectionsProps) {
  return (
    <>
      <FormSection description="Identifiers used by the lab, manufacturer, and catalog search." title="Identity">
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          <Field label="Internal part number">
            <Input disabled={readOnly} value={form.internalPartNumber} onChange={(event) => update("internalPartNumber", event.target.value)} />
          </Field>
          <Field label="Category">
            <Input
              disabled={readOnly}
              list="lab-component-categories"
              value={form.category}
              onChange={(event) => update("category", event.target.value)}
            />
            <datalist id="lab-component-categories">
              {CATEGORY_TEMPLATES.map((category) => <option key={category.category} value={category.category} />)}
            </datalist>
          </Field>
          <Field label="Manufacturer">
            <Input disabled={readOnly} value={form.manufacturer} onChange={(event) => update("manufacturer", event.target.value)} />
          </Field>
          <Field label="Manufacturer part number">
            <Input disabled={readOnly} value={form.manufacturerPartNumber} onChange={(event) => update("manufacturerPartNumber", event.target.value)} />
          </Field>
        </div>
      </FormSection>

      <FormSection description="Physical form and lifecycle of this component type." title="Form">
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          <Field label="Mounting type">
            <select className={SELECT_CLASS} disabled={readOnly} value={form.mountingType} onChange={(event) => update("mountingType", event.target.value)}>
              {['unknown', 'through_hole', 'surface_mount', 'panel_mount', 'chassis', 'wire', 'module', 'other'].map((option) => (
                <option key={option} value={option}>{option.replaceAll('_', ' ')}</option>
              ))}
            </select>
          </Field>
          <Field label="Package type">
            <Input disabled={readOnly} placeholder="Axial, TO-92, DIP-8…" value={form.packageType} onChange={(event) => update("packageType", event.target.value)} />
          </Field>
          <Field label="Part status">
            <select className={SELECT_CLASS} disabled={readOnly} value={form.partStatus} onChange={(event) => update("partStatus", event.target.value)}>
              {['active', 'nrnd', 'obsolete', 'discontinued', 'unknown'].map((option) => <option key={option} value={option}>{option.toUpperCase()}</option>)}
            </select>
          </Field>
          <div className="flex min-h-9 items-center self-end text-sm text-muted-foreground">
            Status:{" "}
            <span className={form.archived ? "ml-1 font-medium text-warning-foreground" : "ml-1 font-medium text-foreground"}>
              {form.archived ? "Archived" : "Inventory"}
            </span>
          </div>
          <Field className="md:col-span-2 xl:col-span-4" label="Description">
            <Textarea disabled={readOnly} value={form.description} onChange={(event) => update("description", event.target.value)} />
          </Field>
        </div>
      </FormSection>

      <FormSection description="Template suggestions help consistency, but custom attributes remain allowed." title="Specifications">
        <AttributeEditor attributes={attributes} category={form.category} onChange={onAttributesChange} readOnly={readOnly} />
      </FormSection>

      <FormSection description="Supplier identity and engineering documentation stay separate." title="Supplier and Documentation">
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
          <Field label="Supplier"><Input disabled={readOnly} value={form.supplier} onChange={(event) => update("supplier", event.target.value)} /></Field>
          <Field label="Supplier SKU"><Input disabled={readOnly} value={form.supplierSku} onChange={(event) => update("supplierSku", event.target.value)} /></Field>
          <Field label="Supplier packaging"><Input disabled={readOnly} placeholder="Cut tape, reel, tube…" value={form.supplierPackaging} onChange={(event) => update("supplierPackaging", event.target.value)} /></Field>
          <Field className="md:col-span-2" label="Product URL"><Input disabled={readOnly} placeholder="https://…" type="url" value={form.productUrl} onChange={(event) => update("productUrl", event.target.value)} /></Field>
          <Field className="md:col-span-2" label="Datasheet URL"><Input disabled={readOnly} placeholder="https://…" type="url" value={form.datasheetUrl} onChange={(event) => update("datasheetUrl", event.target.value)} /></Field>
          <Field className="md:col-span-2" label="Picture path"><Input disabled={readOnly} value={form.picturePath ?? ""} onChange={(event) => update("picturePath", event.target.value)} /></Field>
        </div>
      </FormSection>

      <FormSection description="Low-stock status uses only the configured default unit." title="Replenishment">
        <div className="grid gap-3 md:grid-cols-3">
          <Field label="Default unit of measure">
            <Input disabled={readOnly} list="lab-stock-units" value={form.defaultUnitOfMeasure} onChange={(event) => update("defaultUnitOfMeasure", event.target.value)} />
            <datalist id="lab-stock-units">
              {['unknown', 'pcs', 'm', 'ft', 'g', 'kg', 'reel', 'roll', 'tube', 'tray', 'bag', 'other'].map((option) => <option key={option} value={option} />)}
            </datalist>
          </Field>
          <Field label="Reorder point"><Input disabled={readOnly} min="0" step="any" type="number" value={form.reorderPoint ?? ""} onChange={(event) => update("reorderPoint", numberOrNull(event.target.value))} /></Field>
          <Field label="Target quantity"><Input disabled={readOnly} min="0" step="any" type="number" value={form.targetQuantity ?? ""} onChange={(event) => update("targetQuantity", numberOrNull(event.target.value))} /></Field>
        </div>
      </FormSection>

      <FormSection description="Each placement owns its physical location and quantity." title="Stock Locations">
        <PartPlacements lookups={lookups} part={part} placements={placements} readOnly={readOnly} summary={summary} {...placementActions} />
      </FormSection>

      <FormSection description="Free-form information that does not belong in a structured field." title="Notes">
        <Textarea disabled={readOnly} value={form.notes} onChange={(event) => update("notes", event.target.value)} />
      </FormSection>
    </>
  );
}

function AttributeEditor({
  attributes,
  category,
  onChange,
  readOnly,
}: {
  attributes: AttributeRow[];
  category: string;
  onChange: Dispatch<SetStateAction<AttributeRow[]>>;
  readOnly: boolean;
}) {
  const template = categoryTemplate(category);
  const hasKey = (key: string) => attributes.some((row) => normalizeAttributeKey(row.key) === key);
  const editRow = (rowId: string, change: Partial<AttributeRow>) =>
    onChange((current) => current.map((row) => (row.rowId === rowId ? { ...row, ...change } : row)));

  function addSuggestedAttribute(key: string, unit = ""): void {
    if (hasKey(key)) {
      return;
    }
    onChange((current) => [...current, { rowId: `${key}-${Date.now()}`, key, value: "", unit }]);
  }

  return (
    <>
      {template?.attributes.length ? (
        <div className="mb-3 flex flex-wrap gap-1.5">
          {template.attributes.map((attribute) => (
            <Button
              disabled={readOnly || hasKey(attribute.key)}
              key={attribute.key}
              onClick={() => addSuggestedAttribute(attribute.key, attribute.unit)}
              size="xs"
              variant="outline"
            >
              <PlusIcon className="size-3" />
              {attribute.label}
            </Button>
          ))}
        </div>
      ) : null}
      <div className="space-y-2">
        {attributes.map((attribute, index) => (
          <div className="grid gap-2 md:grid-cols-[1fr_1.4fr_0.7fr_auto]" key={attribute.rowId}>
            <Input
              aria-label={`Attribute ${index + 1} key`}
              disabled={readOnly}
              placeholder="Attribute key"
              value={attribute.key}
              onChange={(event) => editRow(attribute.rowId, { key: event.target.value })}
            />
            <Input
              aria-label={`Attribute ${index + 1} value`}
              disabled={readOnly}
              placeholder={humanizeAttributeKey(attribute.key || "value")}
              value={attribute.value}
              onChange={(event) => editRow(attribute.rowId, { value: event.target.value })}
            />
            <Input
              aria-label={`Attribute ${index + 1} unit`}
              disabled={readOnly}
              placeholder="Unit"
              value={attribute.unit}
              onChange={(event) => editRow(attribute.rowId, { unit: event.target.value })}
            />
            <Button
              aria-label={`Remove attribute ${index + 1}`}
              disabled={readOnly}
              onClick={() => onChange((current) => current.filter((row) => row.rowId !== attribute.rowId))}
              size="icon"
              variant="ghost"
            >
              <Trash2Icon className="size-4" />
            </Button>
          </div>
        ))}
        <Button disabled={readOnly} onClick={() => onChange((current) => [...current, { rowId: `custom-${Date.now()}`, key: "", value: "", unit: "" }])} size="sm" variant="outline">
          <PlusIcon className="size-3.5" />
          Add custom attribute
        </Button>
      </div>
    </>
  );
}

function PartPlacements({
  lookups,
  onAddPlacement,
  onCountPlacement,
  onDeletePlacement,
  onEditPlacement,
  onMovePlacement,
  part,
  placements,
  readOnly,
  summary,
}: PlacementActions & {
  lookups: ReturnType<typeof createCatalogLookups>;
  part: Part | null;
  placements: StockPlacement[];
  readOnly: boolean;
  summary: PartStockSummary | undefined;
}) {
  if (!part) {
    return (
      <div className="rounded-xl border border-dashed border-border p-5 text-center text-sm text-muted-foreground">
        Save the component first, then add advanced stock placements from More Details.
      </div>
    );
  }
  return (
    <div className="space-y-3">
      <div className="flex flex-wrap items-center gap-2">
        <StockStatusBadge status={summary?.stockStatus ?? (part.archived ? "archived" : "no_stock")} />
        <span className="font-semibold">Total: {formatTotals(summary)}</span>
        <Button className="ml-auto" disabled={readOnly || part.archived} onClick={() => onAddPlacement(part)} size="sm">
          <MapPinIcon className="size-3.5" />
          Add Placement
        </Button>
      </div>
      {placements.length ? (
        <div className="grid gap-2 lg:grid-cols-2">
          {placements.map((placement) => (
            <div className="rounded-xl border border-border bg-muted/20 p-3" key={placement.placementUuid}>
              <div className="flex items-start gap-2">
                <div className="min-w-0 flex-1">
                  <div className="font-medium">{placementPath(placement, lookups)}</div>
                  <div className="mt-0.5 text-sm text-muted-foreground">
                    {placement.quantity.toLocaleString()} {placement.unitOfMeasure}
                    {placement.packaging ? ` · ${placement.packaging}` : ""}
                  </div>
                </div>
                {placement.archived ? <Badge variant="outline">Archived</Badge> : null}
              </div>
              <div className="mt-3 flex flex-wrap gap-1">
                <Button disabled={readOnly} onClick={() => onEditPlacement(placement)} size="xs" variant="outline">
                  <BoxesIcon className="size-3" /> Adjust
                </Button>
                <Button disabled={readOnly || placement.archived} onClick={() => onMovePlacement(placement)} size="xs" variant="outline">
                  <MoveRightIcon className="size-3" /> Move
                </Button>
                <Button disabled={readOnly || placement.archived} onClick={() => onCountPlacement(placement)} size="xs" variant="outline">
                  <CalculatorIcon className="size-3" /> Count
                </Button>
                <Button disabled={readOnly} onClick={() => onEditPlacement(placement)} size="xs" variant="outline">
                  <ArchiveRestoreIcon className="size-3" /> {placement.archived ? "Restore" : "Archive"}
                </Button>
                <Button disabled={readOnly} onClick={() => onDeletePlacement(placement)} size="xs" variant="destructive-outline">
                  <Trash2Icon className="size-3" /> Delete
                </Button>
              </div>
            </div>
          ))}
        </div>
      ) : (
        <div className="rounded-xl border border-dashed border-border p-5 text-center text-sm text-muted-foreground">
          This catalog part has no stock placements yet.
        </div>
      )}
    </div>
  );
}
