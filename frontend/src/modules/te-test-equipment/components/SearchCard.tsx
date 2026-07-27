import { ListFilterIcon, XIcon } from "lucide-react";

import { FilterPanel } from "@/modules/te-test-equipment/components/FilterPanel";
import { hasActiveFilters } from "@/modules/te-test-equipment/lib";
import type {
  FilterState,
  InventoryScope,
  TeTestEquipmentWorkspace,
} from "@/modules/te-test-equipment/types";
import { Button } from "@/shared/components/ui/button";
import { ColorRowsButton } from "@/shared/components/ui/ColorRowsButton";
import { Input } from "@/shared/components/ui/input";
import { cn } from "@/shared/lib/utils";

interface SearchCardProps {
  colorRows: boolean;
  filters: FilterState;
  filtersOpen: boolean;
  onColorRowsChange: (nextValue: boolean) => void;
  onFilterChange: (field: keyof FilterState, value: string) => void;
  onFiltersClear: () => void;
  onFiltersToggle: () => void;
  onQueryChange: (value: string) => void;
  query: string;
  scope: InventoryScope;
  workspace: TeTestEquipmentWorkspace;
}

export function SearchCard({
  colorRows,
  filters,
  filtersOpen,
  onColorRowsChange,
  onFilterChange,
  onFiltersClear,
  onFiltersToggle,
  onQueryChange,
  query,
  scope,
  workspace,
}: SearchCardProps) {
  const filtersActive = hasActiveFilters(filters, workspace) || Boolean(query.trim());

  return (
    <section
      className={cn(
        "relative shrink-0 rounded-xl border border-border/70 bg-card/80 p-2 shadow-sm sm:p-2.5",
        // Stay below product header menus (z-40 / z-80) so the system switcher stays usable.
        filtersOpen ? "z-30" : "z-0",
      )}
    >
      <div className="flex flex-wrap items-center gap-2">
        <div className="min-w-0 flex-1 basis-56">
          <Input
            aria-label={workspace === "calibration" ? "Calibration search" : "Inventory search"}
            inputClassName="h-9 px-3 text-sm"
            placeholder={
              workspace === "calibration"
                ? scope === "archive"
                  ? "Search archived calibration equipment by asset, serial, maker, model, dates, vendor, certificate, notes, or location"
                  : "Search calibration equipment by asset, serial, maker, model, dates, vendor, certificate, notes, or location"
                : scope === "archive"
                  ? "Search archived entries by asset, serial, maker, model, description, location, or notes"
                  : "Search entries by asset, serial, maker, model, description, location, status, or notes"
            }
            value={query}
            onChange={(event) => onQueryChange(event.currentTarget.value)}
          />
        </div>

        <div className="flex shrink-0 items-center gap-1">
          <ColorRowsButton pressed={colorRows} onPressedChange={onColorRowsChange} />
          <Button
            aria-expanded={filtersOpen}
            aria-label={
              filtersOpen
                ? "Hide filters"
                : filtersActive
                  ? "Show filters (active)"
                  : "Show filters"
            }
            className={cn("size-9", (filtersOpen || filtersActive) && "border-foreground/25 bg-muted/60")}
            size="icon"
            title={filtersOpen ? "Hide filters" : "Show filters"}
            variant="outline"
            onClick={onFiltersToggle}
          >
            <ListFilterIcon className="size-3.5" />
          </Button>
          {filtersActive ? (
            <Button
              aria-label="Clear filters"
              className="size-7 px-0"
              size="icon"
              title="Clear filters"
              variant="ghost"
              onClick={onFiltersClear}
            >
              <XIcon className="size-3.5" />
            </Button>
          ) : null}
        </div>
      </div>

      {filtersOpen ? (
        <div className="mt-2 border-t border-border/60 pt-2">
          <FilterPanel
            compact
            filters={filters}
            hideHeader
            workspace={workspace}
            onChange={onFilterChange}
            onClear={onFiltersClear}
          />
        </div>
      ) : null}
    </section>
  );
}
