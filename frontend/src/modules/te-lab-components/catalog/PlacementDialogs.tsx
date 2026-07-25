import { useMemo, useState } from "react";

import {
  createCatalogLookups,
  indistinguishablePlacementKey,
  partDisplayName,
  placementPath,
} from "@/modules/te-lab-components/catalog/catalogUtils";
import { CatalogDialog } from "@/modules/te-lab-components/catalog/CatalogDialog";
import { GridBinPicker } from "@/modules/te-lab-components/catalog/GridBinPicker";
import type {
  CatalogSyncResult,
  Part,
  StockCountInput,
  StockMoveInput,
  StockPlacement,
  StockPlacementInput,
  StorageArea,
  StorageContainer,
} from "@/modules/te-lab-components/types";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Textarea } from "@/shared/components/ui/textarea";

const SELECT_CLASS =
  "h-9 w-full rounded-lg border border-input bg-background px-3 text-sm outline-none focus:border-ring focus:ring-[3px] focus:ring-ring/18 dark:bg-input/30";

interface PlacementDialogProps {
  catalog: CatalogSyncResult;
  onClose: () => void;
  onSave: (input: StockPlacementInput, mergeCandidate: StockPlacement | null) => Promise<void>;
  part: Part;
  placement: StockPlacement | null;
}

export function PlacementDialog({ catalog, onClose, onSave, part, placement }: PlacementDialogProps) {
  const currentContainer = catalog.storageContainers.find(
    (container) => container.containerUuid === placement?.containerUuid,
  );
  const initialAreaUuid = currentContainer?.areaUuid ?? catalog.storageAreas.find((area) => !area.archived)?.areaUuid ?? "";
  const initialContainerUuid = placement?.containerUuid ?? firstContainerUuid(catalog, initialAreaUuid);
  const [areaUuid, setAreaUuid] = useState(initialAreaUuid);
  const [input, setInput] = useState<StockPlacementInput>(() => ({
    ...placementInput(part, placement),
    containerUuid: initialContainerUuid,
  }));
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const lookups = useMemo(() => createCatalogLookups(catalog), [catalog]);
  const areas = catalog.storageAreas.filter((area) => !area.archived || area.areaUuid === areaUuid);
  const containers = catalog.storageContainers.filter(
    (container) =>
      container.areaUuid === areaUuid &&
      (!container.archived || container.containerUuid === placement?.containerUuid),
  );
  const container = containers.find((candidate) => candidate.containerUuid === input.containerUuid) ?? null;
  const mergeCandidate = useMemo(() => {
    if (placement) {
      return null;
    }
    const key = indistinguishablePlacementKey(input);
    return catalog.stockPlacements.find(
      (candidate) =>
        !candidate.archived &&
        candidate.partUuid === part.entryUuid &&
        indistinguishablePlacementKey(candidate) === key,
    ) ?? null;
  }, [catalog.stockPlacements, input, part.entryUuid, placement]);

  const occupiedContents = useMemo(() => {
    if (!container || input.rowIndex === null || input.columnIndex === null) {
      return [];
    }
    return catalog.stockPlacements.filter(
      (candidate) =>
        !candidate.archived &&
        candidate.placementUuid !== placement?.placementUuid &&
        candidate.containerUuid === container.containerUuid &&
        candidate.rowIndex === input.rowIndex &&
        candidate.columnIndex === input.columnIndex,
    );
  }, [catalog.stockPlacements, container, input.columnIndex, input.rowIndex, placement?.placementUuid]);

  async function submit(): Promise<void> {
    setError(null);
    if (!container) {
      setError("Select a storage container.");
      return;
    }
    if (!Number.isFinite(input.quantity) || input.quantity < 0) {
      setError("Quantity must be zero or greater.");
      return;
    }
    if (input.unitOfMeasure === "pcs" && !Number.isInteger(input.quantity)) {
      setError("Piece quantities must be whole numbers.");
      return;
    }
    if (container.gridEnabled && (input.rowIndex === null || input.columnIndex === null)) {
      setError("Select a grid bin.");
      return;
    }
    setBusy(true);
    try {
      await onSave(input, mergeCandidate);
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Could not save the stock placement.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <CatalogDialog
      description={`Store ${partDisplayName(part)} in one structured location with its own quantity.`}
      footer={
        <div className="flex justify-end gap-2">
          <Button onClick={onClose} variant="outline">Cancel</Button>
          <Button disabled={busy} onClick={() => void submit()}>
            {busy
              ? "Saving…"
              : placement
                ? "Save Placement"
                : mergeCandidate
                  ? "Add to Existing Placement"
                  : "Add Placement"}
          </Button>
        </div>
      }
      onClose={onClose}
      title={placement ? "Adjust Stock Placement" : "Add Stock Placement"}
      wide
    >
      <div className="space-y-4">
        {error ? <Alert>{error}</Alert> : null}
        <LocationFields
          areaUuid={areaUuid}
          areas={areas}
          catalog={catalog}
          container={container}
          containers={containers}
          input={input}
          onAreaChange={(nextAreaUuid) => {
            setAreaUuid(nextAreaUuid);
            setInput((current) => ({
              ...current,
              containerUuid: firstContainerUuid(catalog, nextAreaUuid),
              columnIndex: null,
              rowIndex: null,
              freeformPosition: "",
            }));
          }}
          onInputChange={setInput}
        />
        {occupiedContents.length > 0 ? (
          <div className="rounded-xl border border-amber-500/40 bg-amber-500/10 p-3 text-sm">
            <div className="font-semibold">Occupied bin — sharing is allowed</div>
            <ul className="mt-1 list-disc pl-5 text-muted-foreground">
              {occupiedContents.map((candidate) => (
                <li key={candidate.placementUuid}>
                  {partDisplayName(lookups.partsById.get(candidate.partUuid) ?? fallbackPart(candidate.partUuid))}: {candidate.quantity} {candidate.unitOfMeasure}
                </li>
              ))}
            </ul>
          </div>
        ) : null}
        {mergeCandidate ? (
          <div className="rounded-xl border border-primary/35 bg-primary/8 p-3 text-sm">
            <div className="font-semibold">Matching placement already exists</div>
            <p className="mt-1 text-muted-foreground">
              Saving adds {input.quantity} {input.unitOfMeasure} to the existing {mergeCandidate.quantity}{" "}
              {mergeCandidate.unitOfMeasure} placement instead of creating a duplicate.
            </p>
          </div>
        ) : null}
        <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
          <Field label="Quantity"><Input min="0" step="any" type="number" value={Number.isFinite(input.quantity) ? input.quantity : ""} onChange={(event) => setInput((current) => ({ ...current, quantity: Number(event.target.value) }))} /></Field>
          <Field label="Unit of measure"><Input list="placement-units" value={input.unitOfMeasure} onChange={(event) => setInput((current) => ({ ...current, unitOfMeasure: event.target.value }))} /><UnitDatalist /></Field>
          <Field label="Packaging"><Input value={input.packaging} onChange={(event) => setInput((current) => ({ ...current, packaging: event.target.value }))} /></Field>
          <Field label="Condition">
            <select className={SELECT_CLASS} value={input.condition} onChange={(event) => setInput((current) => ({ ...current, condition: event.target.value }))}>
              {['unknown', 'new', 'used', 'salvaged', 'damaged', 'quarantined'].map((condition) => <option key={condition} value={condition}>{condition}</option>)}
            </select>
          </Field>
          <Field label="Lot code"><Input value={input.lotCode} onChange={(event) => setInput((current) => ({ ...current, lotCode: event.target.value }))} /></Field>
          <Field label="Date code"><Input value={input.dateCode} onChange={(event) => setInput((current) => ({ ...current, dateCode: event.target.value }))} /></Field>
          <label className="flex min-h-9 items-center gap-2 self-end rounded-lg border border-border px-3 text-sm">
            <input checked={input.archived} onChange={(event) => setInput((current) => ({ ...current, archived: event.target.checked }))} type="checkbox" />
            Archived placement
          </label>
          <Field className="md:col-span-2 xl:col-span-4" label="Placement notes"><Textarea value={input.notes} onChange={(event) => setInput((current) => ({ ...current, notes: event.target.value }))} /></Field>
        </div>
      </div>
    </CatalogDialog>
  );
}

interface MoveStockDialogProps {
  catalog: CatalogSyncResult;
  onClose: () => void;
  onMove: (input: StockMoveInput) => Promise<void>;
  source: StockPlacement;
}

export function MoveStockDialog({ catalog, onClose, onMove, source }: MoveStockDialogProps) {
  const lookups = useMemo(() => createCatalogLookups(catalog), [catalog]);
  const part = lookups.partsById.get(source.partUuid) ?? fallbackPart(source.partUuid);
  const existingDestinations = catalog.stockPlacements.filter(
    (placement) =>
      placement.partUuid === source.partUuid &&
      placement.placementUuid !== source.placementUuid &&
      !placement.archived,
  );
  const firstArea = catalog.storageAreas.find((area) => !area.archived);
  const initialAreaUuid = firstArea?.areaUuid ?? "";
  const [mode, setMode] = useState<"existing" | "new">(existingDestinations.length ? "existing" : "new");
  const [destinationPlacementUuid, setDestinationPlacementUuid] = useState(existingDestinations[0]?.placementUuid ?? "");
  const [areaUuid, setAreaUuid] = useState(initialAreaUuid);
  const [destination, setDestination] = useState<StockPlacementInput>(() => ({
    ...placementInput(part, null),
    containerUuid: firstContainerUuid(catalog, initialAreaUuid),
    unitOfMeasure: source.unitOfMeasure,
    packaging: source.packaging,
    condition: source.condition,
  }));
  const [quantity, setQuantity] = useState(source.quantity);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const areas = catalog.storageAreas.filter((area) => !area.archived);
  const containers = catalog.storageContainers.filter((container) => container.areaUuid === areaUuid && !container.archived);
  const container = containers.find((candidate) => candidate.containerUuid === destination.containerUuid) ?? null;

  async function submit(): Promise<void> {
    setError(null);
    if (!Number.isFinite(quantity) || quantity <= 0 || quantity > source.quantity) {
      setError(`Move a quantity greater than zero and no more than ${source.quantity} ${source.unitOfMeasure}.`);
      return;
    }
    if (mode === "existing" && !destinationPlacementUuid) {
      setError("Select an existing destination placement.");
      return;
    }
    if (mode === "new" && (!container || (container.gridEnabled && (destination.rowIndex === null || destination.columnIndex === null)))) {
      setError("Select a valid new destination location.");
      return;
    }
    setBusy(true);
    try {
      await onMove({
        sourcePlacementUuid: source.placementUuid,
        destinationPlacementUuid: mode === "existing" ? destinationPlacementUuid : null,
        destination: mode === "new" ? { ...destination, quantity: 0, unitOfMeasure: source.unitOfMeasure } : null,
        quantity,
      });
    } catch (moveError) {
      setError(moveError instanceof Error ? moveError.message : "Could not move stock.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <CatalogDialog
      description={`Move ${partDisplayName(part)} from ${placementPath(source, lookups)} without changing the total.`}
      footer={<div className="flex justify-end gap-2"><Button onClick={onClose} variant="outline">Cancel</Button><Button disabled={busy} onClick={() => void submit()}>{busy ? "Moving…" : "Move Stock"}</Button></div>}
      onClose={onClose}
      title="Move Stock"
      wide
    >
      <div className="space-y-4">
        {error ? <Alert>{error}</Alert> : null}
        <Field label={`Quantity to move (${source.unitOfMeasure})`}><Input max={source.quantity} min="0" step="any" type="number" value={quantity} onChange={(event) => setQuantity(Number(event.target.value))} /></Field>
        <div className="flex flex-wrap gap-2">
          <Button onClick={() => setMode("existing")} variant={mode === "existing" ? "default" : "outline"}>Existing placement</Button>
          <Button onClick={() => setMode("new")} variant={mode === "new" ? "default" : "outline"}>New location</Button>
        </div>
        {mode === "existing" ? (
          <Field label="Destination placement">
            <select className={SELECT_CLASS} value={destinationPlacementUuid} onChange={(event) => setDestinationPlacementUuid(event.target.value)}>
              <option value="">Select a destination</option>
              {existingDestinations.map((placement) => <option key={placement.placementUuid} value={placement.placementUuid}>{placementPath(placement, lookups)} — {placement.quantity} {placement.unitOfMeasure}</option>)}
            </select>
          </Field>
        ) : (
          <LocationFields
            areaUuid={areaUuid}
            areas={areas}
            catalog={catalog}
            container={container}
            containers={containers}
            input={destination}
            onAreaChange={(nextAreaUuid) => {
              setAreaUuid(nextAreaUuid);
              setDestination((current) => ({
                ...current,
                containerUuid: firstContainerUuid(catalog, nextAreaUuid),
                rowIndex: null,
                columnIndex: null,
                freeformPosition: "",
              }));
            }}
            onInputChange={setDestination}
          />
        )}
      </div>
    </CatalogDialog>
  );
}

interface CountStockDialogProps {
  catalog: CatalogSyncResult;
  onClose: () => void;
  onCount: (input: StockCountInput) => Promise<void>;
  placement: StockPlacement;
}

export function CountStockDialog({ catalog, onClose, onCount, placement }: CountStockDialogProps) {
  const lookups = useMemo(() => createCatalogLookups(catalog), [catalog]);
  const [quantity, setQuantity] = useState(placement.quantity);
  const [countedBy, setCountedBy] = useState(placement.lastCountedBy);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function submit(): Promise<void> {
    setError(null);
    if (!Number.isFinite(quantity) || quantity < 0) {
      setError("Counted quantity must be zero or greater.");
      return;
    }
    if (placement.unitOfMeasure === "pcs" && !Number.isInteger(quantity)) {
      setError("Piece quantities must be whole numbers.");
      return;
    }
    if (!countedBy.trim()) {
      setError("Enter who performed the physical count.");
      return;
    }
    setBusy(true);
    try {
      await onCount({ quantity, countedBy });
    } catch (countError) {
      setError(countError instanceof Error ? countError.message : "Could not record the physical count.");
    } finally {
      setBusy(false);
    }
  }

  return (
    <CatalogDialog
      description={placementPath(placement, lookups)}
      footer={<div className="flex justify-end gap-2"><Button onClick={onClose} variant="outline">Cancel</Button><Button disabled={busy} onClick={() => void submit()}>{busy ? "Recording…" : "Record Count"}</Button></div>}
      onClose={onClose}
      title="Physical Stock Count"
    >
      <div className="space-y-4">
        {error ? <Alert>{error}</Alert> : null}
        <div className="grid gap-3 sm:grid-cols-2">
          <Field label={`Counted quantity (${placement.unitOfMeasure})`}><Input min="0" step="any" type="number" value={quantity} onChange={(event) => setQuantity(Number(event.target.value))} /></Field>
          <Field label="Counted by"><Input autoFocus value={countedBy} onChange={(event) => setCountedBy(event.target.value)} /></Field>
        </div>
        <p className="text-sm text-muted-foreground">Saving records the current timestamp and marks this placement as counted.</p>
      </div>
    </CatalogDialog>
  );
}

interface LocationFieldsProps {
  areaUuid: string;
  areas: StorageArea[];
  catalog: CatalogSyncResult;
  container: StorageContainer | null;
  containers: StorageContainer[];
  input: StockPlacementInput;
  onAreaChange: (areaUuid: string) => void;
  onInputChange: React.Dispatch<React.SetStateAction<StockPlacementInput>>;
}

function LocationFields({ areaUuid, areas, catalog, container, containers, input, onAreaChange, onInputChange }: LocationFieldsProps) {
  const partsById = useMemo(() => new Map(catalog.parts.map((part) => [part.entryUuid, part])), [catalog.parts]);
  const area = areas.find((candidate) => candidate.areaUuid === areaUuid);
  return (
    <div className="space-y-3 rounded-xl border border-border bg-muted/15 p-3">
      <div className="grid gap-3 sm:grid-cols-2">
        <Field label="Storage area">
          <select className={SELECT_CLASS} value={areaUuid} onChange={(event) => onAreaChange(event.target.value)}>
            <option value="">Select an area</option>
            {areas.map((candidate) => <option key={candidate.areaUuid} value={candidate.areaUuid}>{candidate.name}{candidate.archived ? " (archived)" : ""}</option>)}
          </select>
        </Field>
        <Field label="Storage container">
          <select className={SELECT_CLASS} value={input.containerUuid} onChange={(event) => onInputChange((current) => ({ ...current, containerUuid: event.target.value, rowIndex: null, columnIndex: null, freeformPosition: "" }))}>
            <option value="">Select a container</option>
            {containers.map((candidate) => <option key={candidate.containerUuid} value={candidate.containerUuid}>{candidate.name}{candidate.archived ? " (archived)" : ""}</option>)}
          </select>
        </Field>
      </div>
      {container?.gridEnabled ? (
        <GridBinPicker
          areaName={area?.name ?? "Unknown area"}
          container={container}
          key={container.containerUuid}
          onSelect={(rowIndex, columnIndex) => onInputChange((current) => ({ ...current, rowIndex, columnIndex, freeformPosition: "" }))}
          partsById={partsById}
          placements={catalog.stockPlacements}
          selectedColumnIndex={input.columnIndex}
          selectedRowIndex={input.rowIndex}
        />
      ) : container ? (
        <Field label="Position within container"><Input placeholder="Shelf, drawer, label, or Unassigned" value={input.freeformPosition} onChange={(event) => onInputChange((current) => ({ ...current, freeformPosition: event.target.value, rowIndex: null, columnIndex: null }))} /></Field>
      ) : null}
    </div>
  );
}

function firstContainerUuid(catalog: CatalogSyncResult, areaUuid: string): string {
  return (
    catalog.storageContainers.find((container) => container.areaUuid === areaUuid && !container.archived)
      ?.containerUuid ??
    catalog.storageContainers.find((container) => container.areaUuid === areaUuid)?.containerUuid ??
    ""
  );
}

function placementInput(part: Part, placement: StockPlacement | null): StockPlacementInput {
  return placement
    ? {
        partUuid: placement.partUuid,
        containerUuid: placement.containerUuid,
        columnIndex: placement.columnIndex,
        rowIndex: placement.rowIndex,
        freeformPosition: placement.freeformPosition,
        quantity: placement.quantity,
        unitOfMeasure: placement.unitOfMeasure,
        packaging: placement.packaging,
        lotCode: placement.lotCode,
        dateCode: placement.dateCode,
        condition: placement.condition,
        countState: placement.countState,
        lastCountedAt: placement.lastCountedAt,
        lastCountedBy: placement.lastCountedBy,
        notes: placement.notes,
        archived: placement.archived,
      }
    : {
        partUuid: part.entryUuid,
        containerUuid: "",
        columnIndex: null,
        rowIndex: null,
        freeformPosition: "",
        quantity: 0,
        unitOfMeasure: part.defaultUnitOfMeasure,
        packaging: "",
        lotCode: "",
        dateCode: "",
        condition: "unknown",
        countState: "uncounted",
        lastCountedAt: null,
        lastCountedBy: "",
        notes: "",
        archived: false,
      };
}

function fallbackPart(entryUuid: string): Part {
  return {
    id: entryUuid,
    entryUuid,
    internalPartNumber: "",
    category: "",
    subcategory: "",
    manufacturer: "",
    manufacturerPartNumber: "Unknown part",
    displayValue: "",
    mountingType: "",
    packageType: "",
    description: "",
    attributes: {},
    supplier: "",
    supplierSku: "",
    supplierPackaging: "",
    productUrl: "",
    datasheetUrl: "",
    defaultUnitOfMeasure: "unknown",
    reorderPoint: null,
    targetQuantity: null,
    partStatus: "unknown",
    picturePath: "",
    notes: "",
    archived: false,
    legacy: {
      serialNumber: "",
      projectName: "",
      assignedTo: "",
      lifecycleStatus: "",
      workingStatus: "",
      condition: "",
      verifiedInSurvey: false,
      manualEntry: false,
    },
    createdAt: "",
    updatedAt: "",
  };
}

function UnitDatalist() {
  return <datalist id="placement-units">{['unknown', 'pcs', 'm', 'ft', 'g', 'kg', 'reel', 'roll', 'tube', 'tray', 'bag', 'other'].map((unit) => <option key={unit} value={unit} />)}</datalist>;
}

function Field({ children, className = "", label }: { children: React.ReactNode; className?: string; label: string }) {
  return <label className={`space-y-1.5 text-sm ${className}`}><span className="font-medium">{label}</span>{children}</label>;
}

function Alert({ children }: { children: React.ReactNode }) {
  return <div className="rounded-xl border border-destructive/30 bg-destructive/8 px-3 py-2 text-sm text-destructive-foreground" role="alert">{children}</div>;
}
