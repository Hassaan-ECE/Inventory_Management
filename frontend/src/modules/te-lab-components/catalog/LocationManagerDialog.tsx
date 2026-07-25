import { useMemo, useState } from "react";
import { ArchiveRestoreIcon, BoxIcon, MapPinIcon, PencilIcon, PlusIcon, Trash2Icon } from "lucide-react";

import {
  createCatalogLookups,
  gridCoordinateLabel,
  partDisplayName,
  placementPosition,
} from "@/modules/te-lab-components/catalog/catalogUtils";
import { CatalogDialog } from "@/modules/te-lab-components/catalog/CatalogDialog";
import { GridBinPicker } from "@/modules/te-lab-components/catalog/GridBinPicker";
import type {
  CatalogSyncResult,
  StorageArea,
  StorageAreaInput,
  StorageContainer,
  StorageContainerInput,
} from "@/modules/te-lab-components/types";
import { Badge } from "@/shared/components/ui/badge";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";
import { Textarea } from "@/shared/components/ui/textarea";
import { cn } from "@/shared/lib/utils";

const SELECT_CLASS =
  "h-9 w-full rounded-lg border border-input bg-background px-3 text-sm outline-none focus:border-ring focus:ring-[3px] focus:ring-ring/18 dark:bg-input/30";

interface LocationManagerDialogProps {
  catalog: CatalogSyncResult;
  onClose: () => void;
  onCreateArea: (input: StorageAreaInput) => Promise<void>;
  onCreateContainer: (input: StorageContainerInput) => Promise<void>;
  onDeleteArea: (area: StorageArea) => Promise<void>;
  onDeleteContainer: (container: StorageContainer) => Promise<void>;
  onUpdateArea: (area: StorageArea, input: StorageAreaInput) => Promise<void>;
  onUpdateContainer: (container: StorageContainer, input: StorageContainerInput) => Promise<void>;
  readOnly: boolean;
}

export function LocationManagerDialog({
  catalog,
  onClose,
  onCreateArea,
  onCreateContainer,
  onDeleteArea,
  onDeleteContainer,
  onUpdateArea,
  onUpdateContainer,
  readOnly,
}: LocationManagerDialogProps) {
  const initialAreaUuid = catalog.storageAreas[0]?.areaUuid ?? "";
  const [query, setQuery] = useState("");
  const [selectedAreaUuid, setSelectedAreaUuid] = useState(initialAreaUuid);
  const [selectedContainerUuid, setSelectedContainerUuid] = useState(
    () => catalog.storageContainers.find((container) => container.areaUuid === initialAreaUuid)?.containerUuid ?? "",
  );
  const [selectedCell, setSelectedCell] = useState<{ rowIndex: number; columnIndex: number } | null>(null);
  const [areaEditor, setAreaEditor] = useState<StorageArea | "new" | null>(null);
  const [containerEditor, setContainerEditor] = useState<StorageContainer | "new" | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);
  const lookups = useMemo(() => createCatalogLookups(catalog), [catalog]);
  const normalizedQuery = query.trim().toLocaleLowerCase();

  const visibleAreas = useMemo(() => {
    if (!normalizedQuery) {
      return catalog.storageAreas;
    }
    return catalog.storageAreas.filter((area) => {
      if ([area.name, area.areaType, area.owner, area.description].join(" ").toLocaleLowerCase().includes(normalizedQuery)) {
        return true;
      }
      return catalog.storageContainers
        .filter((container) => container.areaUuid === area.areaUuid)
        .some((container) => containerMatches(container, catalog, lookups, normalizedQuery));
    });
  }, [catalog, lookups, normalizedQuery]);

  const selectedArea = catalog.storageAreas.find((area) => area.areaUuid === selectedAreaUuid) ?? visibleAreas[0] ?? null;
  const visibleContainers = catalog.storageContainers.filter(
    (container) =>
      container.areaUuid === selectedArea?.areaUuid &&
      (!normalizedQuery || containerMatches(container, catalog, lookups, normalizedQuery)),
  );
  const selectedContainer =
    visibleContainers.find((container) => container.containerUuid === selectedContainerUuid) ??
    visibleContainers[0] ??
    null;
  const selectedCellForContainer = selectedContainer?.containerUuid === selectedContainerUuid ? selectedCell : null;
  const selectedCellContents = selectedContainer && selectedCellForContainer
    ? catalog.stockPlacements.filter(
        (placement) =>
          !placement.archived &&
          placement.containerUuid === selectedContainer.containerUuid &&
          placement.rowIndex === selectedCellForContainer.rowIndex &&
          placement.columnIndex === selectedCellForContainer.columnIndex,
      )
    : [];

  async function toggleAreaArchive(area: StorageArea): Promise<void> {
    setActionError(null);
    try {
      await onUpdateArea(area, areaInput(area, !area.archived));
    } catch (error) {
      setActionError(error instanceof Error ? error.message : "Could not update the storage area.");
    }
  }

  async function toggleContainerArchive(container: StorageContainer): Promise<void> {
    setActionError(null);
    try {
      await onUpdateContainer(container, containerInput(container, !container.archived));
    } catch (error) {
      setActionError(error instanceof Error ? error.message : "Could not update the storage container.");
    }
  }

  return (
    <>
      <CatalogDialog
        description="Manage area or desk → container → Excel-style bin coordinates."
        onClose={onClose}
        title="Lab Storage Locations"
        wide
      >
        <div className="space-y-3">
          {actionError ? <Alert>{actionError}</Alert> : null}
          <div className="flex flex-wrap items-center gap-2">
            <Input
              aria-label="Search storage locations"
              className="max-w-xl"
              placeholder="Search areas, containers, coordinates, MPNs, or descriptions…"
              value={query}
              onChange={(event) => setQuery(event.target.value)}
            />
            <Button className="ml-auto" disabled={readOnly} onClick={() => setAreaEditor("new")} size="sm">
              <PlusIcon className="size-3.5" /> Add Area
            </Button>
          </div>

          <div className="grid min-h-[62vh] gap-3 lg:grid-cols-[15rem_18rem_minmax(0,1fr)]">
            <section className="overflow-auto rounded-xl border border-border bg-muted/15 p-2">
              <h3 className="px-2 pb-2 text-xs font-semibold uppercase tracking-wide text-muted-foreground">Areas and desks</h3>
              <div className="space-y-1">
                {visibleAreas.map((area) => (
                  <button
                    className={cn(
                      "w-full rounded-lg border px-3 py-2 text-left text-sm",
                      selectedArea?.areaUuid === area.areaUuid
                        ? "border-primary bg-primary/8"
                        : "border-transparent hover:border-border hover:bg-accent/50",
                    )}
                    key={area.areaUuid}
                    onClick={() => {
                      setSelectedAreaUuid(area.areaUuid);
                      setSelectedContainerUuid(
                        catalog.storageContainers.find((container) => container.areaUuid === area.areaUuid)?.containerUuid ?? "",
                      );
                      setSelectedCell(null);
                    }}
                    type="button"
                  >
                    <span className="flex items-center gap-2 font-medium">
                      <MapPinIcon className="size-3.5" /> {area.name}
                    </span>
                    <span className="mt-0.5 block text-xs text-muted-foreground">
                      {area.areaType}{area.owner ? ` · ${area.owner}` : ""}{area.archived ? " · Archived" : ""}
                    </span>
                  </button>
                ))}
                {visibleAreas.length === 0 ? <p className="px-2 py-4 text-sm text-muted-foreground">No locations match this search.</p> : null}
              </div>
            </section>

            <section className="overflow-auto rounded-xl border border-border bg-muted/15 p-2">
              <div className="flex items-center justify-between gap-2 px-2 pb-2">
                <h3 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Containers</h3>
                <Button disabled={readOnly || !selectedArea || selectedArea.archived} onClick={() => setContainerEditor("new")} size="xs" variant="outline">
                  <PlusIcon className="size-3" /> Add
                </Button>
              </div>
              <div className="space-y-1">
                {visibleContainers.map((container) => (
                  <button
                    className={cn(
                      "w-full rounded-lg border px-3 py-2 text-left text-sm",
                      selectedContainer?.containerUuid === container.containerUuid
                        ? "border-primary bg-primary/8"
                        : "border-transparent hover:border-border hover:bg-accent/50",
                    )}
                    key={container.containerUuid}
                    onClick={() => {
                      setSelectedContainerUuid(container.containerUuid);
                      setSelectedCell(null);
                    }}
                    type="button"
                  >
                    <span className="flex items-center gap-2 font-medium"><BoxIcon className="size-3.5" /> {container.name}</span>
                    <span className="mt-0.5 block text-xs text-muted-foreground">
                      {container.gridEnabled ? `${container.columnCount} columns × ${container.rowCount} rows` : "Freeform positions"}
                      {container.archived ? " · Archived" : ""}
                    </span>
                  </button>
                ))}
                {selectedArea && visibleContainers.length === 0 ? <p className="px-2 py-4 text-sm text-muted-foreground">No containers in this area.</p> : null}
              </div>
            </section>

            <section className="min-w-0 overflow-auto rounded-xl border border-border bg-card/50 p-3">
              {selectedArea && selectedContainer ? (
                <div className="space-y-3">
                  <div className="flex flex-wrap items-start gap-2">
                    <div className="min-w-0 flex-1">
                      <div className="flex items-center gap-2">
                        <h3 className="font-semibold">{selectedArea.name} / {selectedContainer.name}</h3>
                        {selectedContainer.archived ? <Badge variant="outline">Archived</Badge> : null}
                      </div>
                      <p className="text-xs text-muted-foreground">{selectedContainer.description || selectedContainer.containerType}</p>
                    </div>
                    <Button disabled={readOnly} onClick={() => setAreaEditor(selectedArea)} size="xs" variant="outline"><PencilIcon className="size-3" /> Area</Button>
                    <Button disabled={readOnly} onClick={() => setContainerEditor(selectedContainer)} size="xs" variant="outline"><PencilIcon className="size-3" /> Container</Button>
                    <Button disabled={readOnly} onClick={() => void toggleContainerArchive(selectedContainer)} size="xs" variant="outline"><ArchiveRestoreIcon className="size-3" /> {selectedContainer.archived ? "Restore" : "Archive"}</Button>
                  </div>
                  {selectedContainer.gridEnabled ? (
                    <GridBinPicker
                      areaName={selectedArea.name}
                      container={selectedContainer}
                      key={selectedContainer.containerUuid}
                      onSelect={(rowIndex, columnIndex) => {
                        setSelectedContainerUuid(selectedContainer.containerUuid);
                        setSelectedCell({ rowIndex, columnIndex });
                      }}
                      partsById={lookups.partsById}
                      placements={catalog.stockPlacements}
                      selectedColumnIndex={selectedCellForContainer?.columnIndex ?? null}
                      selectedRowIndex={selectedCellForContainer?.rowIndex ?? null}
                    />
                  ) : (
                    <NonGridContents catalog={catalog} container={selectedContainer} />
                  )}
                  {selectedCellForContainer ? (
                    <div className="rounded-xl border border-border bg-background p-3">
                      <h4 className="font-semibold">Bin {gridCoordinateLabel(selectedContainer, selectedCellForContainer.rowIndex, selectedCellForContainer.columnIndex)} contents</h4>
                      {selectedCellContents.length ? (
                        <ul className="mt-2 space-y-1 text-sm">
                          {selectedCellContents.map((placement) => (
                            <li className="flex items-center justify-between gap-3" key={placement.placementUuid}>
                              <span>{partDisplayName(lookups.partsById.get(placement.partUuid) ?? fallbackPart(placement.partUuid))}</span>
                              <span className="font-medium tabular-nums">{placement.quantity} {placement.unitOfMeasure}</span>
                            </li>
                          ))}
                        </ul>
                      ) : <p className="mt-1 text-sm text-muted-foreground">This bin is empty.</p>}
                    </div>
                  ) : null}
                  <div className="flex flex-wrap gap-2 border-t border-border pt-3">
                    <Button disabled={readOnly} onClick={() => void toggleAreaArchive(selectedArea)} size="xs" variant="outline"><ArchiveRestoreIcon className="size-3" /> {selectedArea.archived ? "Restore Area" : "Archive Area"}</Button>
                    <Button disabled={readOnly} onClick={() => { if (window.confirm(`Delete ${selectedContainer.name}?`)) void onDeleteContainer(selectedContainer).catch((error) => setActionError(error instanceof Error ? error.message : "Could not delete the container.")); }} size="xs" variant="destructive-outline"><Trash2Icon className="size-3" /> Delete Container</Button>
                    <Button disabled={readOnly} onClick={() => { if (window.confirm(`Delete ${selectedArea.name}?`)) void onDeleteArea(selectedArea).catch((error) => setActionError(error instanceof Error ? error.message : "Could not delete the area.")); }} size="xs" variant="destructive-outline"><Trash2Icon className="size-3" /> Delete Area</Button>
                  </div>
                </div>
              ) : selectedArea ? (
                <div className="flex h-full min-h-72 items-center justify-center text-sm text-muted-foreground">Add or select a container.</div>
              ) : (
                <div className="flex h-full min-h-72 items-center justify-center text-sm text-muted-foreground">Add or select a storage area.</div>
              )}
            </section>
          </div>
        </div>
      </CatalogDialog>

      {areaEditor ? (
        <AreaEditorDialog
          area={areaEditor === "new" ? null : areaEditor}
          onClose={() => setAreaEditor(null)}
          onSave={async (input) => {
            if (areaEditor === "new") {
              await onCreateArea(input);
            } else {
              await onUpdateArea(areaEditor, input);
            }
            setAreaEditor(null);
          }}
        />
      ) : null}

      {containerEditor && selectedArea ? (
        <ContainerEditorDialog
          area={selectedArea}
          catalog={catalog}
          container={containerEditor === "new" ? null : containerEditor}
          onClose={() => setContainerEditor(null)}
          onSave={async (input) => {
            if (containerEditor === "new") {
              await onCreateContainer(input);
            } else {
              await onUpdateContainer(containerEditor, input);
            }
            setContainerEditor(null);
          }}
        />
      ) : null}
    </>
  );
}

function AreaEditorDialog({ area, onClose, onSave }: { area: StorageArea | null; onClose: () => void; onSave: (input: StorageAreaInput) => Promise<void> }) {
  const [input, setInput] = useState<StorageAreaInput>(() => area ? areaInput(area, area.archived) : { name: "", areaType: "lab", owner: "", description: "", archived: false });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  async function submit(): Promise<void> {
    if (!input.name.trim()) {
      setError("Storage area name is required.");
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await onSave(input);
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Could not save the storage area.");
    } finally {
      setBusy(false);
    }
  }
  return (
    <CatalogDialog footer={<div className="flex justify-end gap-2"><Button onClick={onClose} variant="outline">Cancel</Button><Button disabled={busy} onClick={() => void submit()}>{busy ? "Saving…" : "Save Area"}</Button></div>} onClose={onClose} title={area ? "Edit Storage Area" : "Add Storage Area"}>
      <div className="space-y-3">
        {error ? <Alert>{error}</Alert> : null}
        <Field label="Area name"><Input autoFocus value={input.name} onChange={(event) => setInput((current) => ({ ...current, name: event.target.value }))} /></Field>
        <div className="grid gap-3 sm:grid-cols-2">
          <Field label="Area type"><select className={SELECT_CLASS} value={input.areaType} onChange={(event) => setInput((current) => ({ ...current, areaType: event.target.value }))}>{['lab', 'room', 'desk', 'mobile', 'other'].map((value) => <option key={value} value={value}>{value}</option>)}</select></Field>
          <Field label="Owner"><Input value={input.owner} onChange={(event) => setInput((current) => ({ ...current, owner: event.target.value }))} /></Field>
        </div>
        <Field label="Description"><Textarea value={input.description} onChange={(event) => setInput((current) => ({ ...current, description: event.target.value }))} /></Field>
        <label className="flex items-center gap-2 text-sm"><input checked={input.archived} onChange={(event) => setInput((current) => ({ ...current, archived: event.target.checked }))} type="checkbox" /> Archived area</label>
      </div>
    </CatalogDialog>
  );
}

function ContainerEditorDialog({ area, catalog, container, onClose, onSave }: { area: StorageArea; catalog: CatalogSyncResult; container: StorageContainer | null; onClose: () => void; onSave: (input: StorageContainerInput) => Promise<void> }) {
  const [input, setInput] = useState<StorageContainerInput>(() => container ? containerInput(container, container.archived) : { areaUuid: area.areaUuid, name: "", containerType: "organizer", gridEnabled: true, rowCount: 8, columnCount: 8, rowStart: 1, description: "", archived: false });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const outOfBounds = container ? catalog.stockPlacements.filter((placement) => !placement.archived && placement.containerUuid === container.containerUuid && input.gridEnabled && (placement.rowIndex === null || placement.columnIndex === null || placement.rowIndex >= (input.rowCount ?? 0) || placement.columnIndex >= (input.columnCount ?? 0))) : [];
  const previewContainer: StorageContainer = {
    containerUuid: container?.containerUuid ?? "preview-container",
    areaUuid: area.areaUuid,
    name: input.name || "Grid preview",
    containerType: input.containerType,
    gridEnabled: input.gridEnabled,
    rowCount: input.gridEnabled ? input.rowCount : null,
    columnCount: input.gridEnabled ? input.columnCount : null,
    rowStart: input.rowStart ?? 1,
    origin: "top_left",
    description: input.description,
    archived: input.archived,
    createdAt: container?.createdAt ?? "",
    updatedAt: container?.updatedAt ?? "",
  };
  async function submit(): Promise<void> {
    setError(null);
    if (!input.name.trim()) {
      setError("Container name is required.");
      return;
    }
    if (input.gridEnabled && ((!input.rowCount || input.rowCount < 1) || (!input.columnCount || input.columnCount < 1))) {
      setError("Grid rows and columns must both be at least 1.");
      return;
    }
    if (outOfBounds.length) {
      setError(`This resize would invalidate ${outOfBounds.length} active placement(s). Move or archive them first.`);
      return;
    }
    setBusy(true);
    try {
      await onSave({ ...input, rowCount: input.gridEnabled ? input.rowCount : null, columnCount: input.gridEnabled ? input.columnCount : null });
    } catch (saveError) {
      setError(saveError instanceof Error ? saveError.message : "Could not save the storage container.");
    } finally {
      setBusy(false);
    }
  }
  return (
    <CatalogDialog footer={<div className="flex justify-end gap-2"><Button onClick={onClose} variant="outline">Cancel</Button><Button disabled={busy || outOfBounds.length > 0} onClick={() => void submit()}>{busy ? "Saving…" : "Save Container"}</Button></div>} onClose={onClose} title={container ? "Edit Storage Container" : `Add Container to ${area.name}`} wide>
      <div className="space-y-4">
        {error ? <Alert>{error}</Alert> : null}
        <div className="grid gap-3 md:grid-cols-2">
          <Field label="Container name"><Input autoFocus value={input.name} onChange={(event) => setInput((current) => ({ ...current, name: event.target.value }))} /></Field>
          <Field label="Container type"><select className={SELECT_CLASS} value={input.containerType} onChange={(event) => setInput((current) => ({ ...current, containerType: event.target.value }))}>{['cabinet', 'organizer', 'drawer_bank', 'shelf', 'box', 'other'].map((value) => <option key={value} value={value}>{value.replaceAll('_', ' ')}</option>)}</select></Field>
          <label className="flex min-h-9 items-center gap-2 rounded-lg border border-border px-3 text-sm"><input checked={input.gridEnabled} onChange={(event) => setInput((current) => ({ ...current, gridEnabled: event.target.checked, rowCount: event.target.checked ? current.rowCount ?? 8 : null, columnCount: event.target.checked ? current.columnCount ?? 8 : null }))} type="checkbox" /> Excel-style grid</label>
          <label className="flex min-h-9 items-center gap-2 rounded-lg border border-border px-3 text-sm"><input checked={input.archived} onChange={(event) => setInput((current) => ({ ...current, archived: event.target.checked }))} type="checkbox" /> Archived container</label>
          {input.gridEnabled ? <><Field label="Rows"><Input min="1" max="1000" type="number" value={input.rowCount ?? ""} onChange={(event) => setInput((current) => ({ ...current, rowCount: Number(event.target.value) }))} /></Field><Field label="Columns"><Input min="1" max="1000" type="number" value={input.columnCount ?? ""} onChange={(event) => setInput((current) => ({ ...current, columnCount: Number(event.target.value) }))} /></Field><Field label="First row number"><Input min="1" type="number" value={input.rowStart ?? 1} onChange={(event) => setInput((current) => ({ ...current, rowStart: Number(event.target.value) }))} /></Field></> : null}
          <Field className="md:col-span-2" label="Description"><Textarea value={input.description} onChange={(event) => setInput((current) => ({ ...current, description: event.target.value }))} /></Field>
        </div>
        {input.gridEnabled && (input.rowCount ?? 0) > 0 && (input.columnCount ?? 0) > 0 ? (
          <div>
            <h3 className="mb-2 font-semibold">Live grid preview</h3>
            <GridBinPicker areaName={area.name} container={previewContainer} disabled partsById={new Map()} placements={[]} selectedColumnIndex={null} selectedRowIndex={null} />
          </div>
        ) : null}
      </div>
    </CatalogDialog>
  );
}

function NonGridContents({ catalog, container }: { catalog: CatalogSyncResult; container: StorageContainer }) {
  const lookups = createCatalogLookups(catalog);
  const placements = catalog.stockPlacements.filter((placement) => !placement.archived && placement.containerUuid === container.containerUuid);
  return placements.length ? <div className="space-y-2">{placements.map((placement) => <div className="flex items-center justify-between gap-3 rounded-lg border border-border bg-background px-3 py-2 text-sm" key={placement.placementUuid}><span>{placementPosition(placement, container)} · {partDisplayName(lookups.partsById.get(placement.partUuid) ?? fallbackPart(placement.partUuid))}</span><span className="font-medium">{placement.quantity} {placement.unitOfMeasure}</span></div>)}</div> : <p className="rounded-xl border border-dashed border-border p-5 text-center text-sm text-muted-foreground">No active stock in this container.</p>;
}

function containerMatches(container: StorageContainer, catalog: CatalogSyncResult, lookups: ReturnType<typeof createCatalogLookups>, query: string): boolean {
  if ([container.name, container.containerType, container.description].join(" ").toLocaleLowerCase().includes(query)) return true;
  return catalog.stockPlacements.filter((placement) => placement.containerUuid === container.containerUuid).some((placement) => {
    const part = lookups.partsById.get(placement.partUuid);
    return [placementPosition(placement, container), placement.freeformPosition, part?.manufacturerPartNumber, part?.internalPartNumber, part?.description].join(" ").toLocaleLowerCase().includes(query);
  });
}

function areaInput(area: StorageArea, archived: boolean): StorageAreaInput {
  return { name: area.name, areaType: area.areaType, owner: area.owner, description: area.description, archived };
}

function containerInput(container: StorageContainer, archived: boolean): StorageContainerInput {
  return { areaUuid: container.areaUuid, name: container.name, containerType: container.containerType, gridEnabled: container.gridEnabled, rowCount: container.rowCount, columnCount: container.columnCount, rowStart: container.rowStart, description: container.description, archived };
}

function fallbackPart(entryUuid: string) {
  return { id: entryUuid, entryUuid, manufacturerPartNumber: "Unknown part" } as ReturnType<typeof createCatalogLookups>["partsById"] extends Map<string, infer PartType> ? PartType : never;
}

function Field({ children, className = "", label }: { children: React.ReactNode; className?: string; label: string }) {
  return <label className={`space-y-1.5 text-sm ${className}`}><span className="font-medium">{label}</span>{children}</label>;
}

function Alert({ children }: { children: React.ReactNode }) {
  return <div className="rounded-xl border border-destructive/30 bg-destructive/8 px-3 py-2 text-sm text-destructive-foreground" role="alert">{children}</div>;
}
