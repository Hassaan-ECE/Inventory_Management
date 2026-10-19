import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { ArchiveRestoreIcon, Trash2Icon } from "lucide-react";

import { createCatalogLookups } from "@/modules/te-lab-components/catalog/catalogUtils";
import { CatalogDialog } from "@/modules/te-lab-components/catalog/CatalogDialog";
import { Alert } from "@/modules/te-lab-components/catalog/formControls";
import { PartDetailsSections } from "@/modules/te-lab-components/catalog/PartDetailsSections";
import {
  attributeRows,
  attributeRowsEqual,
  attributesFromRows,
  inputFromPart,
  partFormEquals,
  type AttributeRow,
} from "@/modules/te-lab-components/catalog/partForm";
import { SimplePartFields } from "@/modules/te-lab-components/catalog/SimplePartFields";
import {
  applySimpleIdentity,
  engineeringUnitsFor,
  normalizeShelfLocation,
  projectSimpleStock,
  readComponentValue,
  shelfLocationError,
} from "@/modules/te-lab-components/catalog/simpleComponent";
import type {
  CatalogSyncResult,
  ComponentAttributes,
  Part,
  PartInput,
  SimpleComponentInput,
  StockPlacement,
  StorageContainer,
} from "@/modules/te-lab-components/types";
import { Button } from "@/shared/components/ui/button";
import { UnsavedChangesDialog } from "@/shared/components/ui/UnsavedChangesDialog";

interface PartDialogProps {
  catalog: CatalogSyncResult;
  onAddPlacement: (part: Part) => void;
  onClose: () => void;
  onCountPlacement: (placement: StockPlacement) => void;
  onDeletePlacement: (placement: StockPlacement) => void;
  onDeletePart: (part: Part) => Promise<void> | void;
  onEditPlacement: (placement: StockPlacement) => void;
  onExpandGrid?: (container: StorageContainer, axis: "row" | "column") => Promise<void> | void;
  onMovePlacement: (placement: StockPlacement) => void;
  onSaveAdvanced: (input: PartInput) => Promise<void>;
  onSaveSimple: (input: SimpleComponentInput) => Promise<void>;
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
  onExpandGrid,
  onMovePlacement,
  onSaveAdvanced,
  onSaveSimple,
  part,
  readOnly,
}: PartDialogProps) {
  const lookups = useMemo(() => createCatalogLookups(catalog), [catalog]);
  const placements = useMemo(
    () => catalog.stockPlacements.filter((placement) => placement.partUuid === part?.entryUuid && !placement.archived),
    [catalog.stockPlacements, part?.entryUuid],
  );
  const projection = useMemo(
    () => projectSimpleStock(placements, lookups.containersById),
    [lookups.containersById, placements],
  );
  const initialComponentValue = useMemo(
    () => (part ? readComponentValue(part) : { value: "", unit: "" }),
    [part],
  );

  const [initialForm] = useState<PartInput>(() => inputFromPart(part));
  const [initialAttributes] = useState<AttributeRow[]>(() => attributeRows(part?.attributes ?? {}));
  const [initialComponentType] = useState(() => part?.subcategory ?? "");
  const [initialValue] = useState(() => initialComponentValue.value);
  const [initialUnit] = useState(() => initialComponentValue.unit);
  const [initialQuantity] = useState(() =>
    projection.kind === "simple" ? projection.quantity : 0,
  );
  const [initialLocation] = useState(() =>
    projection.kind === "simple" ? projection.location : "",
  );

  const [form, setForm] = useState<PartInput>(initialForm);
  const [attributes, setAttributes] = useState<AttributeRow[]>(initialAttributes);
  const [componentType, setComponentType] = useState(initialComponentType);
  const [value, setValue] = useState(initialValue);
  const [unit, setUnit] = useState(initialUnit);
  const [quantity, setQuantity] = useState(initialQuantity);
  const [location, setLocation] = useState(initialLocation);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [showUnsavedPrompt, setShowUnsavedPrompt] = useState(false);
  const previousProjectionKindRef = useRef(projection.kind);

  // When placements collapse from review → simple, seed stock fields from the live projection.
  useEffect(() => {
    const previousKind = previousProjectionKindRef.current;
    previousProjectionKindRef.current = projection.kind;
    if (previousKind === "review" && projection.kind === "simple") {
      setQuantity(projection.quantity);
      setLocation(projection.location);
    }
  }, [projection]);

  const isArchived = form.archived;
  const summary = part ? lookups.summariesByPartId.get(part.entryUuid) : undefined;
  const reviewRequired = projection.kind === "review";
  const stockDisabled = readOnly || reviewRequired;

  const isDirty = useMemo(
    () =>
      !partFormEquals(form, initialForm) ||
      !attributeRowsEqual(attributes, initialAttributes) ||
      componentType !== initialComponentType ||
      value !== initialValue ||
      unit !== initialUnit ||
      quantity !== initialQuantity ||
      location !== initialLocation,
    [
      attributes,
      componentType,
      form,
      initialAttributes,
      initialComponentType,
      initialForm,
      initialLocation,
      initialQuantity,
      initialUnit,
      initialValue,
      location,
      quantity,
      unit,
      value,
    ],
  );

  const requestClose = useCallback((): void => {
    if (busy) {
      return;
    }
    if (confirmDelete) {
      setConfirmDelete(false);
      return;
    }
    if (isDirty && !readOnly) {
      setShowUnsavedPrompt(true);
      return;
    }
    onClose();
  }, [busy, confirmDelete, isDirty, onClose, readOnly]);

  function update<K extends keyof PartInput>(key: K, value: PartInput[K]): void {
    setForm((current) => ({ ...current, [key]: value }));
  }

  function handleComponentTypeChange(nextType: string): void {
    setComponentType(nextType);
    const nextUnits = engineeringUnitsFor(nextType);
    if (nextUnits.length === 0) {
      setUnit("");
    } else if (!nextUnits.includes(unit)) {
      setUnit(nextUnits[0] ?? "");
    }
  }

  function buildNormalizedAttributes(): ComponentAttributes | null {
    const result = attributesFromRows(attributes);
    if ("error" in result) {
      setError(result.error);
      return null;
    }
    return result.attributes;
  }

  async function submit(): Promise<boolean> {
    setError(null);

    if (reviewRequired) {
      if (
        !form.internalPartNumber.trim() &&
        !form.manufacturerPartNumber.trim() &&
        !form.displayValue.trim() &&
        !form.description.trim() &&
        !value.trim()
      ) {
        setError("Provide an internal part number, manufacturer part number, value/label, or description.");
        return false;
      }
      if (
        form.reorderPoint !== null &&
        form.targetQuantity !== null &&
        form.targetQuantity < form.reorderPoint
      ) {
        setError("Target quantity must be greater than or equal to the reorder point.");
        return false;
      }
      const normalizedAttributes = buildNormalizedAttributes();
      if (!normalizedAttributes) {
        return false;
      }
      setBusy(true);
      try {
        const advancedInput = applySimpleIdentity(
          { ...form, attributes: normalizedAttributes },
          componentType,
          value.trim(),
          unit,
        );
        await onSaveAdvanced(advancedInput);
        return true;
      } catch (saveError) {
        setError(saveError instanceof Error ? saveError.message : "Could not save the part.");
        return false;
      } finally {
        setBusy(false);
      }
    }

    if (!componentType.trim()) {
      setError("Select a component type.");
      return false;
    }
    if (!value.trim() && !form.manufacturerPartNumber.trim() && !form.internalPartNumber.trim() && !form.description.trim()) {
      setError("Provide a value/part label or another identifier.");
      return false;
    }
    if (
      form.reorderPoint !== null &&
      form.targetQuantity !== null &&
      form.targetQuantity < form.reorderPoint
    ) {
      setError("Target quantity must be greater than or equal to the reorder point.");
      return false;
    }

    if (!Number.isInteger(quantity) || quantity < 0) {
      setError("Quantity must be a whole number of pieces.");
      return false;
    }

    const normalizedLocation = normalizeShelfLocation(location);
    const locationError = shelfLocationError(normalizedLocation, quantity);
    if (locationError) {
      setError(locationError);
      return false;
    }

    const normalizedAttributes = buildNormalizedAttributes();
    if (!normalizedAttributes) {
      return false;
    }

    const baseInput: PartInput = {
      ...form,
      attributes: normalizedAttributes,
      defaultUnitOfMeasure: "pcs",
      mountingType: form.mountingType || "through_hole",
    };
    const partInput = applySimpleIdentity(baseInput, componentType, value.trim(), unit);

    setBusy(true);
    try {
      await onSaveSimple({
        part: partInput,
        quantity,
        location: normalizedLocation,
      });
      return true;
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Could not save the component.");
      return false;
    } finally {
      setBusy(false);
    }
  }

  async function saveAndClose(): Promise<void> {
    const saved = await submit();
    if (saved) {
      setShowUnsavedPrompt(false);
      onClose();
    }
  }

  async function toggleArchive(): Promise<void> {
    const next = { ...form, archived: !form.archived };
    setForm(next);
    setError(null);
    setBusy(true);
    try {
      const normalizedAttributes = buildNormalizedAttributes() ?? form.attributes;
      await onSaveAdvanced({ ...next, attributes: normalizedAttributes });
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Could not update archive state.");
      setForm(form);
    } finally {
      setBusy(false);
    }
  }

  function requestDelete(): void {
    if (!part || !isArchived) {
      return;
    }
    setConfirmDelete(true);
  }

  async function confirmDeletePart(): Promise<void> {
    if (!part) {
      return;
    }
    setError(null);
    setBusy(true);
    try {
      await onDeletePart(part);
      setConfirmDelete(false);
    } catch (deleteError) {
      setConfirmDelete(false);
      setError(deleteError instanceof Error ? deleteError.message : "Could not delete the part.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <CatalogDialog
      description="Everyday fields stay short; open More Details for full catalog and placement controls."
      footer={
        <div className="flex items-center justify-between gap-3">
          <div className="flex flex-wrap gap-2">
            {part ? (
              isArchived ? (
                <>
                  <Button
                    className="border-success/30 bg-success/12 text-success-foreground hover:bg-success/18"
                    disabled={busy || readOnly}
                    type="button"
                    variant="outline"
                    onClick={() => void toggleArchive()}
                  >
                    <ArchiveRestoreIcon className="size-3.5" /> Restore to Inventory
                  </Button>
                  <Button disabled={busy || readOnly} type="button" variant="destructive" onClick={requestDelete}>
                    <Trash2Icon className="size-3.5" /> Delete Part
                  </Button>
                </>
              ) : (
                <Button
                  className="border-warning/30 bg-warning/12 text-warning-foreground hover:bg-warning/18"
                  disabled={busy || readOnly}
                  type="button"
                  variant="outline"
                  onClick={() => void toggleArchive()}
                >
                  <ArchiveRestoreIcon className="size-3.5" /> Archive Part
                </Button>
              )
            ) : (
              <span className="text-xs text-muted-foreground">
                {readOnly ? "Editing is unavailable for the current catalog state." : "Totals are derived from active placements."}
              </span>
            )}
          </div>
          <div className="flex gap-2">
            <Button disabled={busy} onClick={requestClose} variant="outline">
              Cancel
            </Button>
            <Button disabled={busy || readOnly} onClick={() => void submit()}>
              {busy ? "Saving…" : part ? "Save Component" : "Add Component"}
            </Button>
          </div>
        </div>
      }
      onClose={requestClose}
      title={part ? "Edit Component" : "Add Component"}
      wide
    >
      <UnsavedChangesDialog
        isSaving={busy}
        open={showUnsavedPrompt}
        readOnly={readOnly}
        onCancel={() => setShowUnsavedPrompt(false)}
        onDiscard={() => {
          setShowUnsavedPrompt(false);
          onClose();
        }}
        onSave={() => {
          void saveAndClose();
        }}
      />

      {confirmDelete && part ? (
        <ConfirmDeletePart
          busy={busy}
          onCancel={() => setConfirmDelete(false)}
          onConfirm={() => void confirmDeletePart()}
          part={part}
        />
      ) : null}

      <div className="space-y-6">
        {error ? <Alert>{error}</Alert> : null}

        <SimplePartFields
          catalog={catalog}
          componentType={componentType}
          disabled={stockDisabled}
          location={location}
          onComponentTypeChange={handleComponentTypeChange}
          onExpandGrid={onExpandGrid}
          onLocationChange={setLocation}
          onQuantityChange={setQuantity}
          onUnitChange={setUnit}
          onValueChange={setValue}
          partPlacements={placements}
          projection={projection}
          quantity={quantity}
          unit={unit}
          value={value}
        />

        <details className="rounded-xl border border-border">
          <summary className="cursor-pointer px-4 py-3 font-semibold">More Details</summary>
          <div className="space-y-4 border-t border-border p-4">
            <PartDetailsSections
              attributes={attributes}
              form={form}
              lookups={lookups}
              onAddPlacement={onAddPlacement}
              onAttributesChange={setAttributes}
              onCountPlacement={onCountPlacement}
              onDeletePlacement={onDeletePlacement}
              onEditPlacement={onEditPlacement}
              onFieldChange={update}
              onMovePlacement={onMovePlacement}
              part={part}
              placements={placements}
              readOnly={readOnly}
              summary={summary}
            />
          </div>
        </details>
      </div>
    </CatalogDialog>
  );
}

function ConfirmDeletePart({
  busy,
  onCancel,
  onConfirm,
  part,
}: {
  busy: boolean;
  onCancel: () => void;
  onConfirm: () => void;
  part: Part;
}) {
  return (
    <div
      className="fixed inset-0 z-[110] flex items-center justify-center bg-black/40 p-4"
      role="alertdialog"
      aria-labelledby="delete-part-title"
      aria-describedby="delete-part-desc"
      onClick={(event) => {
        if (event.target === event.currentTarget) {
          onCancel();
        }
      }}
    >
      <div className="w-full max-w-md rounded-2xl border border-border bg-card p-5 shadow-2xl">
        <h3 id="delete-part-title" className="text-lg font-semibold text-foreground">
          Delete part permanently?
        </h3>
        <p id="delete-part-desc" className="mt-2 text-sm text-muted-foreground">
          Delete{" "}
          <strong className="text-foreground">
            {part.manufacturerPartNumber || part.displayValue || part.internalPartNumber || part.id}
          </strong>
          ? This cannot be undone. Any remaining stock placements for this part are removed with it.
        </p>
        <div className="mt-5 flex justify-end gap-2">
          <Button type="button" variant="outline" onClick={onCancel}>
            Cancel
          </Button>
          <Button disabled={busy} type="button" variant="destructive" onClick={onConfirm}>
            <Trash2Icon className="size-3.5" /> {busy ? "Deleting…" : "Delete Part"}
          </Button>
        </div>
      </div>
    </div>
  );
}
