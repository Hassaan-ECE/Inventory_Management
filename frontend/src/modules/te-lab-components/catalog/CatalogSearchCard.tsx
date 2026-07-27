import { ListFilterIcon, XIcon } from "lucide-react";

import type { CatalogColumnKey } from "@/modules/te-lab-components/catalog/catalogColumns";
import type { CatalogFilters } from "@/modules/te-lab-components/catalog/catalogFilters";
import type { StockStatus, StorageArea } from "@/modules/te-lab-components/types";
import { DropdownSelect } from "@/shared/components/ui/DropdownMenu";
import { Button } from "@/shared/components/ui/button";
import { ColorRowsButton } from "@/shared/components/ui/ColorRowsButton";
import { Input } from "@/shared/components/ui/input";
import { cn } from "@/shared/lib/utils";

const STOCK_STATUS_OPTIONS: Array<{ value: "" | StockStatus; label: string }> = [
  { value: "", label: "All stock states" },
  { value: "in_stock", label: "In stock" },
  { value: "low_stock", label: "Low stock" },
  { value: "no_stock", label: "No stock" },
  { value: "unit_review", label: "Unit review" },
  { value: "mixed_units", label: "Mixed units" },
  { value: "archived", label: "Archived" },
];

interface CatalogSearchCardProps {
  activeFilterCount: number;
  areaOptions: StorageArea[];
  categoryOptions: string[];
  colorRows: boolean;
  containerOptions: Array<{ label: string; value: string }>;
  filters: CatalogFilters;
  filtersOpen: boolean;
  hasActiveFilters: boolean;
  manufacturerOptions: string[];
  mountingTypeOptions: string[];
  onClearFilters: () => void;
  onColorRowsChange: (nextValue: boolean) => void;
  onFilterChange: <K extends keyof CatalogFilters>(field: K, value: CatalogFilters[K]) => void;
  onFiltersToggle: () => void;
  onQueryChange: (value: string) => void;
  packageTypeOptions: string[];
  partStatusOptions: string[];
  query: string;
  subcategoryOptions: string[];
}

export function CatalogSearchCard({
  activeFilterCount,
  areaOptions,
  categoryOptions,
  colorRows,
  containerOptions,
  filters,
  filtersOpen,
  hasActiveFilters,
  manufacturerOptions,
  mountingTypeOptions,
  onClearFilters,
  onColorRowsChange,
  onFilterChange,
  onFiltersToggle,
  onQueryChange,
  packageTypeOptions,
  partStatusOptions,
  query,
  subcategoryOptions,
}: CatalogSearchCardProps) {
  return (
    <section
      className={cn(
        "relative shrink-0 rounded-xl border border-border/70 bg-card/80 p-2 shadow-sm sm:p-2.5",
        // Stay below the product header (z-40/z-80) so the system switcher stays clickable.
        filtersOpen ? "z-30" : "z-0",
      )}
    >
      <div className="flex flex-wrap items-center gap-2">
        <div className="min-w-0 flex-1 basis-56">
          <Input
            aria-label="Search Lab components"
            inputClassName="h-9 px-3 text-sm"
            placeholder="Search MPN, value, attributes, supplier, or location…"
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
                : activeFilterCount > 0
                  ? `Show filters (${activeFilterCount} active)`
                  : "Show filters"
            }
            className={cn(
              "size-9",
              (filtersOpen || activeFilterCount > 0) && "border-foreground/25 bg-muted/60",
            )}
            size="icon"
            title={filtersOpen ? "Hide filters" : "Show filters"}
            variant="outline"
            onClick={onFiltersToggle}
          >
            <ListFilterIcon className="size-3.5" />
          </Button>
          {hasActiveFilters ? (
            <Button
              aria-label="Clear catalog filters"
              className="size-7 px-0"
              size="icon"
              title="Clear filters"
              variant="ghost"
              onClick={onClearFilters}
            >
              <XIcon className="size-3.5" />
            </Button>
          ) : null}
        </div>
      </div>

      {filtersOpen ? (
        <div className="mt-2 border-t border-border/60 pt-2">
          <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
            <DropdownSelect
              aria-label="Filter category"
              options={[
                { value: "", label: "All categories" },
                ...categoryOptions.map((value) => ({ value, label: value })),
              ]}
              value={filters.category}
              onChange={(value) => onFilterChange("category", value)}
            />
            <DropdownSelect
              aria-label="Filter subcategory"
              options={[
                { value: "", label: "All subcategories" },
                ...subcategoryOptions.map((value) => ({ value, label: value })),
              ]}
              value={filters.subcategory}
              onChange={(value) => onFilterChange("subcategory", value)}
            />
            <DropdownSelect
              aria-label="Filter stock status"
              options={STOCK_STATUS_OPTIONS}
              value={filters.stockStatus}
              onChange={(value) => onFilterChange("stockStatus", value as CatalogFilters["stockStatus"])}
            />
            <DropdownSelect
              aria-label="Filter manufacturer"
              options={[
                { value: "", label: "All manufacturers" },
                ...manufacturerOptions.map((value) => ({ value, label: value })),
              ]}
              value={filters.manufacturer}
              onChange={(value) => onFilterChange("manufacturer", value)}
            />
            <Input
              aria-label="Filter manufacturer part number"
              inputClassName="h-8 text-xs"
              placeholder="MPN contains…"
              value={filters.manufacturerPartNumber}
              onChange={(event) => onFilterChange("manufacturerPartNumber", event.currentTarget.value)}
            />
            <Input
              aria-label="Filter value or label"
              inputClassName="h-8 text-xs"
              placeholder="Value contains…"
              value={filters.displayValue}
              onChange={(event) => onFilterChange("displayValue", event.currentTarget.value)}
            />
            <DropdownSelect
              aria-label="Filter mounting type"
              options={[
                { value: "", label: "All mounting types" },
                ...mountingTypeOptions.map((value) => ({
                  value,
                  label: value.replaceAll("_", " "),
                })),
              ]}
              value={filters.mountingType}
              onChange={(value) => onFilterChange("mountingType", value)}
            />
            <DropdownSelect
              aria-label="Filter package type"
              options={[
                { value: "", label: "All packages" },
                ...packageTypeOptions.map((value) => ({ value, label: value })),
              ]}
              value={filters.packageType}
              onChange={(value) => onFilterChange("packageType", value)}
            />
            <DropdownSelect
              aria-label="Filter storage area"
              options={[
                { value: "", label: "All areas" },
                ...areaOptions.map((area) => ({ value: area.areaUuid, label: area.name })),
              ]}
              value={filters.areaUuid}
              onChange={(value) => {
                onFilterChange("areaUuid", value);
                if (value !== filters.areaUuid) {
                  onFilterChange("containerUuid", "");
                }
              }}
            />
            <DropdownSelect
              aria-label="Filter storage container"
              options={[{ value: "", label: "All containers" }, ...containerOptions]}
              value={filters.containerUuid}
              onChange={(value) => onFilterChange("containerUuid", value)}
            />
            <Input
              aria-label="Filter bin coordinate"
              inputClassName="h-8 text-xs"
              placeholder="C7, AA1…"
              value={filters.coordinate}
              onChange={(event) => onFilterChange("coordinate", event.currentTarget.value)}
            />
            <DropdownSelect
              aria-label="Filter part status"
              options={[
                { value: "", label: "All part states" },
                ...partStatusOptions.map((value) => ({
                  value,
                  label: value.replaceAll("_", " "),
                })),
              ]}
              value={filters.partStatus}
              onChange={(value) => onFilterChange("partStatus", value)}
            />
          </div>
        </div>
      ) : null}
    </section>
  );
}

// Re-export for callers that still type against column keys via this module path.
export type { CatalogColumnKey };
