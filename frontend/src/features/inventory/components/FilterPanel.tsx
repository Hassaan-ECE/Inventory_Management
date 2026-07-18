import { Input } from "@/shared/components/ui/input";
import { DropdownSelect } from "@/shared/components/ui/DropdownMenu";
import type { FilterState } from "@/features/inventory/types";
import { cn } from "@/shared/lib/utils";

interface FilterPanelProps {
  compact?: boolean;
  filters: FilterState;
  onChange: (field: keyof FilterState, value: string) => void;
  onClear: () => void;
}

const REQUIREMENT_OPTIONS = [
  { value: "all", label: "All requirements" },
  { value: "required", label: "Required" },
  { value: "reference_only", label: "Reference only" },
  { value: "not_required", label: "Not required" },
  { value: "unknown", label: "Unknown" },
] as const;

const HEALTH_OPTIONS = [
  { value: "all", label: "All health" },
  { value: "overdue", label: "Overdue" },
  { value: "due_soon", label: "Due soon" },
  { value: "missing_due", label: "Missing due" },
  { value: "out_to_cal", label: "Out to cal" },
  { value: "current", label: "Current" },
  { value: "not_applicable", label: "Not applicable" },
  { value: "unknown", label: "Unknown" },
] as const;

const DUE_WINDOW_OPTIONS = [
  { value: "all", label: "All due dates" },
  { value: "overdue", label: "Overdue" },
  { value: "next30", label: "Next 30 days" },
  { value: "next60", label: "Next 60 days" },
  { value: "next90", label: "Next 90 days" },
  { value: "missing", label: "Missing due date" },
] as const;

export function FilterPanel({ compact = false, filters, onChange, onClear }: FilterPanelProps) {
  return (
    <section className={cn(!compact && "rounded-xl border border-border/70 bg-card/80 p-2 sm:p-3")}>
      <div className="mb-2 flex items-center justify-between gap-3">
        <h2 className="text-[11px] font-semibold uppercase tracking-[0.08em] text-foreground/50">Column filters</h2>
        <button className="text-xs text-muted-foreground transition-colors hover:text-foreground" type="button" onClick={onClear}>
          Clear Column Filters
        </button>
      </div>

      <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-4 xl:grid-cols-8">
        <Input
          aria-label="Filter asset number"
          inputClassName="h-8 text-xs"
          placeholder="Asset"
          value={filters.assetNumber}
          onChange={(event) => onChange("assetNumber", event.currentTarget.value)}
        />
        <Input
          aria-label="Filter manufacturer"
          inputClassName="h-8 text-xs"
          placeholder="Manufacturer"
          value={filters.manufacturer}
          onChange={(event) => onChange("manufacturer", event.currentTarget.value)}
        />
        <Input
          aria-label="Filter model"
          inputClassName="h-8 text-xs"
          placeholder="Model"
          value={filters.model}
          onChange={(event) => onChange("model", event.currentTarget.value)}
        />
        <Input
          aria-label="Filter description"
          inputClassName="h-8 text-xs"
          placeholder="Description"
          value={filters.description}
          onChange={(event) => onChange("description", event.currentTarget.value)}
        />
        <Input
          aria-label="Filter location"
          inputClassName="h-8 text-xs"
          placeholder="Location"
          value={filters.location}
          onChange={(event) => onChange("location", event.currentTarget.value)}
        />
        <DropdownSelect
          aria-label="Calibration requirement"
          options={REQUIREMENT_OPTIONS}
          value={filters.calibrationRequirement}
          onChange={(value) => onChange("calibrationRequirement", value)}
        />
        <DropdownSelect
          aria-label="Calibration health"
          options={HEALTH_OPTIONS}
          value={filters.calibrationHealth}
          onChange={(value) => onChange("calibrationHealth", value)}
        />
        <DropdownSelect
          aria-label="Due window"
          options={DUE_WINDOW_OPTIONS}
          value={filters.dueWindow}
          onChange={(value) => onChange("dueWindow", value)}
        />
      </div>
    </section>
  );
}
