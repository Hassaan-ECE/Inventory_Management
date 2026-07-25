import { useMemo, useState } from "react";
import { ArchiveRestoreIcon, BoxesIcon, CalculatorIcon, MapPinIcon, MoveRightIcon, PlusIcon, Trash2Icon } from "lucide-react";

import {
  createCatalogLookups,
  formatTotals,
  placementPath,
} from "@/modules/te-lab-components/catalog/catalogUtils";
import {
  CATEGORY_TEMPLATES,
  categoryTemplate,
  humanizeAttributeKey,
  normalizeAttributeKey,
} from "@/modules/te-lab-components/catalog/categoryTemplates";
import { CatalogDialog } from "@/modules/te-lab-components/catalog/CatalogDialog";
import { StockStatusBadge } from "@/modules/te-lab-components/catalog/PartsTable";
import type {
  CatalogSyncResult,
  ComponentAttributes,
  Part,
  PartInput,
  StockPlacement,
} from "@/modules/te-lab-components/types";
import { Badge } from "@/shared/components/ui/badge";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Textarea } from "@/shared/components/ui/textarea";

const SELECT_CLASS =
  "h-9 w-full rounded-lg border border-input bg-background px-3 text-sm outline-none focus:border-ring focus:ring-[3px] focus:ring-ring/18 dark:bg-input/30";

interface AttributeRow {
  rowId: string;
  key: string;
  value: string;
  unit: string;
}

interface PartDialogProps {
  catalog: CatalogSyncResult;
  onAddPlacement: (part: Part) => void;
  onClose: () => void;
  onCountPlacement: (placement: StockPlacement) => void;
  onDeletePlacement: (placement: StockPlacement) => void;
  onDeletePart: (part: Part) => void;
  onEditPlacement: (placement: StockPlacement) => void;
  onMovePlacement: (placement: StockPlacement) => void;
  onSave: (input: PartInput) => Promise<void>;
  part: Part | null;
  readOnly: boolean;
}

export function PartDialog({
  catalog,
  onAddPlacement,
  onClose,
  onCountPlacement,
  onDeletePlacement,
  onDeletePart,
  onEditPlacement,
  onMovePlacement,
  onSave,
  part,
  readOnly,
}: PartDialogProps) {
  const [form, setForm] = useState<PartInput>(() => inputFromPart(part));
  const [attributes, setAttributes] = useState<AttributeRow[]>(() => attributeRows(part?.attributes ?? {}));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const template = categoryTemplate(form.category);
  const subcategories = template?.subcategories ?? [];
  const lookups = useMemo(() => createCatalogLookups(catalog), [catalog]);
  const placements = useMemo(
    () => catalog.stockPlacements.filter((placement) => placement.partUuid === part?.entryUuid),
    [catalog.stockPlacements, part?.entryUuid],
  );
  const summary = part ? lookups.summariesByPartId.get(part.entryUuid) : undefined;

  function update<K extends keyof PartInput>(key: K, value: PartInput[K]): void {
    setForm((current) => ({ ...current, [key]: value }));
  }

  function addSuggestedAttribute(key: string, unit = ""): void {
    if (attributes.some((attribute) => normalizeAttributeKey(attribute.key) === key)) {
      return;
    }
    setAttributes((current) => [
      ...current,
      { rowId: `${key}-${Date.now()}`, key, value: "", unit },
    ]);
  }

  async function submit(): Promise<void> {
    setError(null);
    if (
      !form.internalPartNumber.trim() &&
      !form.manufacturerPartNumber.trim() &&
      !form.displayValue.trim() &&
      !form.description.trim()
    ) {
      setError("Provide an internal part number, manufacturer part number, value/label, or description.");
      return;
    }
    if (
      form.reorderPoint !== null &&
      form.targetQuantity !== null &&
      form.targetQuantity < form.reorderPoint
    ) {
      setError("Target quantity must be greater than or equal to the reorder point.");
      return;
    }
    const normalizedAttributes: ComponentAttributes = {};
    for (const attribute of attributes) {
      const key = normalizeAttributeKey(attribute.key);
      if (!key || !attribute.value.trim()) {
        continue;
      }
      if (normalizedAttributes[key]) {
        setError(`Attribute key “${key}” is duplicated.`);
        return;
      }
      normalizedAttributes[key] = {
        value: attribute.value.trim(),
        unit: attribute.unit.trim(),
      };
    }
    setBusy(true);
    try {
      await onSave({ ...form, attributes: normalizedAttributes });
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Could not save the part.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <CatalogDialog
      description="Catalog identity is separate from stock placement and quantity."
      footer={
        <div className="flex items-center justify-between gap-3">
          <div>
            {part ? (
              <Button disabled={readOnly} onClick={() => onDeletePart(part)} variant="destructive-outline">
                <Trash2Icon className="size-3.5" /> Delete Part
              </Button>
            ) : (
              <span className="text-xs text-muted-foreground">
                {readOnly ? "Editing is unavailable for the current catalog state." : "Totals are derived from active placements."}
              </span>
            )}
          </div>
          <div className="flex gap-2">
            <Button onClick={onClose} variant="outline">
              Cancel
            </Button>
            <Button disabled={busy || readOnly} onClick={() => void submit()}>
              {busy ? "Saving…" : part ? "Save Part" : "Create Part"}
            </Button>
          </div>
        </div>
      }
      onClose={onClose}
      title={part ? "Edit Catalog Part" : "Add Catalog Part"}
      wide
    >
      <div className="space-y-6">
        {error ? (
          <div className="rounded-xl border border-destructive/30 bg-destructive/8 px-3 py-2 text-sm text-destructive-foreground" role="alert">
            {error}
          </div>
        ) : null}

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
            <Field label="Subcategory">
              <Input
                disabled={readOnly}
                list="lab-component-subcategories"
                value={form.subcategory}
                onChange={(event) => update("subcategory", event.target.value)}
              />
              <datalist id="lab-component-subcategories">
                {subcategories.map((subcategory) => <option key={subcategory} value={subcategory} />)}
              </datalist>
            </Field>
            <Field label="Manufacturer">
              <Input disabled={readOnly} value={form.manufacturer} onChange={(event) => update("manufacturer", event.target.value)} />
            </Field>
            <Field label="Manufacturer part number">
              <Input disabled={readOnly} value={form.manufacturerPartNumber} onChange={(event) => update("manufacturerPartNumber", event.target.value)} />
            </Field>
            <Field label="Value / label">
              <Input disabled={readOnly} placeholder="1 kΩ, 100 nF, 2N3904…" value={form.displayValue} onChange={(event) => update("displayValue", event.target.value)} />
            </Field>
          </div>
        </FormSection>

        <FormSection description="Physical form and lifecycle of this component type." title="Form">
          <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
            <Field label="Mounting type">
              <select className={SELECT_CLASS} disabled={readOnly} value={form.mountingType} onChange={(event) => update("mountingType", event.target.value)}>
                {['unknown', 'through_hole', 'surface_mount', 'panel_mount', 'chassis', 'wire', 'module', 'other'].map((value) => (
                  <option key={value} value={value}>{value.replaceAll('_', ' ')}</option>
                ))}
              </select>
            </Field>
            <Field label="Package type">
              <Input disabled={readOnly} placeholder="Axial, TO-92, DIP-8…" value={form.packageType} onChange={(event) => update("packageType", event.target.value)} />
            </Field>
            <Field label="Part status">
              <select className={SELECT_CLASS} disabled={readOnly} value={form.partStatus} onChange={(event) => update("partStatus", event.target.value)}>
                {['active', 'nrnd', 'obsolete', 'discontinued', 'unknown'].map((value) => <option key={value} value={value}>{value.toUpperCase()}</option>)}
              </select>
            </Field>
            <label className="flex min-h-9 items-center gap-2 self-end rounded-lg border border-border px-3 text-sm">
              <input checked={form.archived} disabled={readOnly} onChange={(event) => update("archived", event.target.checked)} type="checkbox" />
              Archived part
            </label>
            <Field className="md:col-span-2 xl:col-span-4" label="Description">
              <Textarea disabled={readOnly} value={form.description} onChange={(event) => update("description", event.target.value)} />
            </Field>
          </div>
        </FormSection>

        <FormSection description="Template suggestions help consistency, but custom attributes remain allowed." title="Specifications">
          {template?.attributes.length ? (
            <div className="mb-3 flex flex-wrap gap-1.5">
              {template.attributes.map((attribute) => (
                <Button
                  disabled={readOnly || attributes.some((row) => normalizeAttributeKey(row.key) === attribute.key)}
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
                  onChange={(event) => setAttributes((current) => current.map((row) => row.rowId === attribute.rowId ? { ...row, key: event.target.value } : row))}
                />
                <Input
                  aria-label={`Attribute ${index + 1} value`}
                  disabled={readOnly}
                  placeholder={humanizeAttributeKey(attribute.key || "value")}
                  value={attribute.value}
                  onChange={(event) => setAttributes((current) => current.map((row) => row.rowId === attribute.rowId ? { ...row, value: event.target.value } : row))}
                />
                <Input
                  aria-label={`Attribute ${index + 1} unit`}
                  disabled={readOnly}
                  placeholder="Unit"
                  value={attribute.unit}
                  onChange={(event) => setAttributes((current) => current.map((row) => row.rowId === attribute.rowId ? { ...row, unit: event.target.value } : row))}
                />
                <Button
                  aria-label={`Remove attribute ${index + 1}`}
                  disabled={readOnly}
                  onClick={() => setAttributes((current) => current.filter((row) => row.rowId !== attribute.rowId))}
                  size="icon"
                  variant="ghost"
                >
                  <Trash2Icon className="size-4" />
                </Button>
              </div>
            ))}
            <Button disabled={readOnly} onClick={() => setAttributes((current) => [...current, { rowId: `custom-${Date.now()}`, key: "", value: "", unit: "" }])} size="sm" variant="outline">
              <PlusIcon className="size-3.5" />
              Add custom attribute
            </Button>
          </div>
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
                {['unknown', 'pcs', 'm', 'ft', 'g', 'kg', 'reel', 'roll', 'tube', 'tray', 'bag', 'other'].map((unit) => <option key={unit} value={unit} />)}
              </datalist>
            </Field>
            <Field label="Reorder point"><Input disabled={readOnly} min="0" step="any" type="number" value={form.reorderPoint ?? ""} onChange={(event) => update("reorderPoint", numberOrNull(event.target.value))} /></Field>
            <Field label="Target quantity"><Input disabled={readOnly} min="0" step="any" type="number" value={form.targetQuantity ?? ""} onChange={(event) => update("targetQuantity", numberOrNull(event.target.value))} /></Field>
          </div>
        </FormSection>

        <FormSection description="Each placement owns its physical location and quantity." title="Stock Locations">
          {part ? (
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
          ) : (
            <div className="rounded-xl border border-dashed border-border p-5 text-center text-sm text-muted-foreground">
              Save the part first, then add one or more stock placements.
            </div>
          )}
        </FormSection>

        <FormSection description="Free-form information that does not belong in a structured field." title="Notes">
          <Textarea disabled={readOnly} value={form.notes} onChange={(event) => update("notes", event.target.value)} />
        </FormSection>
      </div>
    </CatalogDialog>
  );
}

function inputFromPart(part: Part | null): PartInput {
  return part
    ? {
        internalPartNumber: part.internalPartNumber,
        category: part.category,
        subcategory: part.subcategory,
        manufacturer: part.manufacturer,
        manufacturerPartNumber: part.manufacturerPartNumber,
        displayValue: part.displayValue,
        mountingType: part.mountingType,
        packageType: part.packageType,
        description: part.description,
        attributes: part.attributes,
        supplier: part.supplier,
        supplierSku: part.supplierSku,
        supplierPackaging: part.supplierPackaging,
        productUrl: part.productUrl,
        datasheetUrl: part.datasheetUrl,
        defaultUnitOfMeasure: part.defaultUnitOfMeasure,
        reorderPoint: part.reorderPoint,
        targetQuantity: part.targetQuantity,
        partStatus: part.partStatus,
        picturePath: part.picturePath,
        notes: part.notes,
        archived: part.archived,
      }
    : {
        internalPartNumber: "",
        category: "Passive",
        subcategory: "",
        manufacturer: "",
        manufacturerPartNumber: "",
        displayValue: "",
        mountingType: "through_hole",
        packageType: "",
        description: "",
        attributes: {},
        supplier: "",
        supplierSku: "",
        supplierPackaging: "",
        productUrl: "",
        datasheetUrl: "",
        defaultUnitOfMeasure: "pcs",
        reorderPoint: null,
        targetQuantity: null,
        partStatus: "active",
        picturePath: "",
        notes: "",
        archived: false,
      };
}

function attributeRows(attributes: ComponentAttributes): AttributeRow[] {
  return Object.entries(attributes).map(([key, value], index) => ({
    rowId: `${key}-${index}`,
    key,
    value: value.value,
    unit: value.unit,
  }));
}

function numberOrNull(value: string): number | null {
  if (!value.trim()) {
    return null;
  }
  const parsed = Number(value);
  return Number.isFinite(parsed) ? parsed : null;
}

function FormSection({ children, description, title }: { children: React.ReactNode; description: string; title: string }) {
  return (
    <section className="rounded-xl border border-border bg-card/50 p-4">
      <div className="mb-3">
        <h3 className="font-semibold">{title}</h3>
        <p className="text-xs text-muted-foreground">{description}</p>
      </div>
      {children}
    </section>
  );
}

function Field({ children, className = "", label }: { children: React.ReactNode; className?: string; label: string }) {
  return (
    <label className={`space-y-1.5 text-sm ${className}`}>
      <span className="font-medium">{label}</span>
      {children}
    </label>
  );
}
