import { useState } from "react";

import { GridBinPicker } from "@/modules/te-lab-components/catalog/GridBinPicker";
import {
  createCatalogLookups,
  gridCoordinateLabel,
  parseGridCoordinate,
} from "@/modules/te-lab-components/catalog/catalogUtils";
import {
  COMPONENT_TYPE_OPTIONS,
  engineeringUnitsFor,
  resolveSimplePickerTarget,
  type SimpleStockProjection,
} from "@/modules/te-lab-components/catalog/simpleComponent";
import type { CatalogSyncResult, StorageContainer } from "@/modules/te-lab-components/types";
import { Button } from "@/shared/components/ui/button";
import { Input } from "@/shared/components/ui/input";

const SELECT_CLASS =
  "h-9 w-full rounded-lg border border-input bg-background px-3 text-sm outline-none focus:border-ring focus:ring-[3px] focus:ring-ring/18 dark:bg-input/30";

export interface SimplePartFieldsProps {
  catalog: CatalogSyncResult;
  componentType: string;
  disabled: boolean;
  location: string;
  partPlacements: CatalogSyncResult["stockPlacements"];
  onComponentTypeChange: (value: string) => void;
  onExpandGrid?: (container: StorageContainer, axis: "row" | "column") => void;
  onLocationChange: (value: string) => void;
  onQuantityChange: (value: number) => void;
  onUnitChange: (value: string) => void;
  onValueChange: (value: string) => void;
  projection: SimpleStockProjection;
  quantity: number;
  unit: string;
  value: string;
}

export function SimplePartFields({
  catalog,
  componentType,
  disabled,
  location,
  partPlacements,
  onComponentTypeChange,
  onExpandGrid,
  onLocationChange,
  onQuantityChange,
  onUnitChange,
  onValueChange,
  projection,
  quantity,
  unit,
  value,
}: SimplePartFieldsProps) {
  const [editingLocation, setEditingLocation] = useState(false);
  const [previewGrowth, setPreviewGrowth] = useState({ columns: 0, rows: 0 });
  const units = engineeringUnitsFor(componentType);
  const typeOptions =
    componentType && !COMPONENT_TYPE_OPTIONS.includes(componentType)
      ? [componentType, ...COMPONENT_TYPE_OPTIONS]
      : [...COMPONENT_TYPE_OPTIONS];
  const lookups = createCatalogLookups(catalog);
  const picker = resolveSimplePickerTarget(catalog, partPlacements);
  const isPreview = picker.container.containerUuid === "simple-preview-shelf";
  const pickerContainer = {
    ...picker.container,
    rowCount: (picker.container.rowCount ?? 0) + (isPreview ? previewGrowth.rows : 0),
    columnCount: (picker.container.columnCount ?? 0) + (isPreview ? previewGrowth.columns : 0),
  };
  const selected = parseGridCoordinate(location, pickerContainer.rowStart ?? 1);
  const inBounds = selected
    && selected.rowIndex >= 0
    && selected.columnIndex >= 0
    && selected.rowIndex < (pickerContainer.rowCount ?? 0)
    && selected.columnIndex < (pickerContainer.columnCount ?? 0);

  return (
    <section className="space-y-3 rounded-xl border border-border bg-card/50 p-4">
      {projection.kind === "review" ? (
        <div
          className="rounded-xl border border-amber-500/30 bg-amber-500/10 px-3 py-2 text-sm text-amber-900 dark:text-amber-100"
          role="status"
        >
          This component requires advanced location review before simple stock edits. Quantity and Location are
          locked; use More Details to manage placements.
        </div>
      ) : null}

      <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-3">
        <label className="space-y-1.5 text-sm">
          <span className="font-medium">Component Type</span>
          <select
            className={SELECT_CLASS}
            value={componentType}
            onChange={(event) => onComponentTypeChange(event.target.value)}
          >
            {!componentType ? <option value="">Select type…</option> : null}
            {typeOptions.map((option) => (
              <option key={option} value={option}>
                {option}
              </option>
            ))}
          </select>
        </label>

        <label className="space-y-1.5 text-sm">
          <span className="font-medium">Value / Part Label</span>
          <Input
            placeholder="1 kΩ, 100 nF, 2N3904…"
            value={value}
            onChange={(event) => onValueChange(event.target.value)}
          />
        </label>

        {units.length > 0 ? (
          <label className="space-y-1.5 text-sm">
            <span className="font-medium">Unit</span>
            <select
              className={SELECT_CLASS}
              value={unit}
              onChange={(event) => onUnitChange(event.target.value)}
            >
              {!unit || !units.includes(unit) ? <option value={unit || ""}>{unit || "Select unit…"}</option> : null}
              {units.map((option) => (
                <option key={option} value={option}>
                  {option}
                </option>
              ))}
            </select>
          </label>
        ) : null}

        <label className="space-y-1.5 text-sm">
          <span className="font-medium">Quantity</span>
          <Input
            disabled={disabled}
            min={0}
            step={1}
            type="number"
            value={Number.isFinite(quantity) ? quantity : 0}
            onChange={(event) => {
              const next = Number(event.target.value);
              onQuantityChange(Number.isFinite(next) ? Math.max(0, Math.trunc(next)) : 0);
            }}
          />
        </label>

        <div className="space-y-1.5 text-sm md:col-span-2 xl:col-span-1">
          <span className="font-medium">Location</span>
          <div className="flex gap-2">
            <Input
              aria-label="Location"
              disabled={disabled}
              placeholder="B3"
              value={location}
              onChange={(event) => onLocationChange(event.target.value)}
              onFocus={() => {
                if (!disabled) {
                  setEditingLocation(true);
                }
              }}
            />
            <Button
              disabled={disabled}
              type="button"
              variant="outline"
              onClick={() => setEditingLocation((open) => !open)}
            >
              {editingLocation ? "Done" : "Choose bin"}
            </Button>
          </div>
        </div>
      </div>

      {editingLocation && !disabled ? (
        <div className="space-y-2">
          <GridBinPicker
            areaName={picker.areaName}
            container={pickerContainer}
            partsById={lookups.partsById}
            placements={catalog.stockPlacements}
            selectedColumnIndex={inBounds ? selected.columnIndex : null}
            selectedRowIndex={inBounds ? selected.rowIndex : null}
            onExpand={(axis) => {
              if (isPreview) {
                setPreviewGrowth((current) => ({
                  columns: current.columns + (axis === "column" ? 1 : 0),
                  rows: current.rows + (axis === "row" ? 1 : 0),
                }));
                return;
              }
              onExpandGrid?.(picker.container, axis);
            }}
            onSelect={(rowIndex, columnIndex) => {
              onLocationChange(gridCoordinateLabel(pickerContainer, rowIndex, columnIndex));
              setEditingLocation(false);
            }}
          />
        </div>
      ) : null}
    </section>
  );
}
