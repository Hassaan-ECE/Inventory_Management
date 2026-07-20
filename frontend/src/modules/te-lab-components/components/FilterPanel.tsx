import { Input } from "@/shared/components/ui/input";
import type { FilterState } from "@/modules/te-lab-components/types";
import { cn } from "@/shared/lib/utils";

interface FilterPanelProps {
  compact?: boolean;
  filters: FilterState;
  onChange: (field: keyof FilterState, value: string) => void;
  onClear: () => void;
}

export function FilterPanel({ compact = false, filters, onChange, onClear }: FilterPanelProps) {
  return (
    <section className={cn(!compact && "rounded-xl border border-border/70 bg-card/80 p-2 sm:p-3")}>
      <div className="mb-2 flex items-center justify-between gap-3">
        <h2 className="text-[11px] font-semibold uppercase tracking-[0.08em] text-foreground/50">Column filters</h2>
        <button className="text-xs text-muted-foreground transition-colors hover:text-foreground" type="button" onClick={onClear}>
          Clear Column Filters
        </button>
      </div>

      <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-5">
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
      </div>
    </section>
  );
}
