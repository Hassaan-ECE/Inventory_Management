import { useState } from "react";

import { Input } from "@/shared/components/ui/input";
import type { ColumnConfig, ColumnKey, FilterState, InventoryScope } from "@/features/inventory/types";
import { FilterPanel } from "@/features/inventory/components/FilterPanel";
import { ViewSettingsMenu } from "@/features/inventory/components/ViewSettingsMenu";
import { cn } from "@/shared/lib/utils";

interface SearchCardProps {
  colorRows: boolean;
  columns: readonly ColumnConfig[];
  columnVisibility: Record<ColumnKey, boolean>;
  filters: FilterState;
  filtersOpen: boolean;
  onColorRowsChange: (nextValue: boolean) => void;
  onFilterChange: (field: keyof FilterState, value: string) => void;
  onFiltersClear: () => void;
  onFiltersToggle: () => void;
  onQueryChange: (value: string) => void;
  onToggleColumn: (columnKey: ColumnKey) => void;
  query: string;
  scope: InventoryScope;
}

export function SearchCard({
  colorRows,
  columns,
  columnVisibility,
  filters,
  filtersOpen,
  onColorRowsChange,
  onFilterChange,
  onFiltersClear,
  onFiltersToggle,
  onQueryChange,
  onToggleColumn,
  query,
  scope,
}: SearchCardProps) {
  // Elevate above the table while the settings menu or column filters are open so
  // absolute dropdown panels (and filter selects) are not painted under the grid.
  // Stay at z-0 when both are closed so Export / other header menus stay on top.
  const [settingsOpen, setSettingsOpen] = useState(false);
  const elevateAboveTable = settingsOpen || filtersOpen;

  return (
    <section
      className={cn(
        "relative shrink-0 rounded-xl border border-border/70 bg-card/80 p-2 shadow-sm sm:p-2.5",
        elevateAboveTable ? "z-50" : "z-0",
      )}
    >
      <div className="flex flex-col gap-2 sm:flex-row sm:items-center">
        <div className="min-w-0 flex-1">
          <Input
            aria-label="Inventory search"
            inputClassName="h-9 px-3 text-sm"
            placeholder={
              scope === "archive"
                ? "Search archived entries by asset, serial, maker, model, description, location, or notes"
                : "Search entries by asset, serial, maker, model, description, location, status, or notes"
            }
            value={query}
            onChange={(event) => onQueryChange(event.currentTarget.value)}
          />
        </div>

        <div className="flex shrink-0 items-center justify-end">
          <ViewSettingsMenu
            colorRows={colorRows}
            columns={columns}
            filtersOpen={filtersOpen}
            visibility={columnVisibility}
            onColorRowsChange={onColorRowsChange}
            onFiltersToggle={onFiltersToggle}
            onOpenChange={setSettingsOpen}
            onToggleColumn={onToggleColumn}
          />
        </div>
      </div>

      {filtersOpen ? (
        <div className="mt-2 border-t border-border/60 pt-2">
          <FilterPanel compact filters={filters} onChange={onFilterChange} onClear={onFiltersClear} />
        </div>
      ) : null}
    </section>
  );
}
