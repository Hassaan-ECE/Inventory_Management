import {
  COMPONENT_TYPE_OPTIONS,
  engineeringUnitsFor,
  type SimpleStockProjection,
} from "@/modules/te-lab-components/catalog/simpleComponent";
import { Input } from "@/shared/components/ui/input";

const SELECT_CLASS =
  "h-9 w-full rounded-lg border border-input bg-background px-3 text-sm outline-none focus:border-ring focus:ring-[3px] focus:ring-ring/18 dark:bg-input/30";

export interface SimplePartFieldsProps {
  componentType: string;
  disabled: boolean;
  location: string;
  onComponentTypeChange: (value: string) => void;
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
  componentType,
  disabled,
  location,
  onComponentTypeChange,
  onLocationChange,
  onQuantityChange,
  onUnitChange,
  onValueChange,
  projection,
  quantity,
  unit,
  value,
}: SimplePartFieldsProps) {
  const units = engineeringUnitsFor(componentType);
  const typeOptions =
    componentType && !COMPONENT_TYPE_OPTIONS.includes(componentType)
      ? [componentType, ...COMPONENT_TYPE_OPTIONS]
      : [...COMPONENT_TYPE_OPTIONS];

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

        <label className="space-y-1.5 text-sm">
          <span className="font-medium">Location</span>
          <Input
            disabled={disabled}
            placeholder="B3"
            value={location}
            onChange={(event) => onLocationChange(event.target.value)}
          />
        </label>
      </div>
    </section>
  );
}
