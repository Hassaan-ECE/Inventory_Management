import { SettingsIcon } from "lucide-react";

import { DropdownItem, DropdownPanel } from "@/shared/components/ui/DropdownMenu";
import { useDropdownMenu } from "@/shared/hooks/useDropdownMenu";
import { Button } from "@/shared/components/ui/button";
import { Toggle } from "@/shared/components/ui/toggle";
import { getVisibleDataColumnCount } from "@/features/inventory/lib";
import type { ColumnConfig, ColumnKey } from "@/features/inventory/types";

interface ViewSettingsMenuProps {
  colorRows: boolean;
  columns: readonly ColumnConfig[];
  filtersOpen: boolean;
  onColorRowsChange: (nextValue: boolean) => void;
  onFiltersToggle: () => void;
  onOpenChange?: (open: boolean) => void;
  onToggleColumn: (columnKey: ColumnKey) => void;
  visibility: Record<ColumnKey, boolean>;
}

export function ViewSettingsMenu({
  colorRows,
  columns,
  filtersOpen,
  onColorRowsChange,
  onFiltersToggle,
  onOpenChange,
  onToggleColumn,
  visibility,
}: ViewSettingsMenuProps) {
  const { open, menuRef, toggle, close } = useDropdownMenu({ onOpenChange });
  const visibleDataColumns = getVisibleDataColumnCount(visibility);

  return (
    <div className="relative" ref={menuRef}>
      <Button
        aria-expanded={open}
        aria-haspopup="menu"
        aria-label="View settings"
        className="size-8"
        size="icon"
        variant="outline"
        onClick={toggle}
      >
        <SettingsIcon className="size-3.5" />
      </Button>

      {open ? (
        <DropdownPanel
          align="right"
          className="w-72"
          maxHeightClassName="max-h-[min(24rem,calc(100vh-10rem))]"
          title="View settings"
        >
          <div className="flex items-center justify-between gap-3 rounded-xl px-3 py-2 hover:bg-accent/40">
            <span className="text-sm text-foreground">Color rows</span>
            <Toggle aria-label="Color rows" pressed={colorRows} onPressedChange={onColorRowsChange} />
          </div>

          <DropdownItem
            active={filtersOpen}
            onClick={() => {
              onFiltersToggle();
              close();
            }}
          >
            <span className="flex-1 text-left">{filtersOpen ? "Hide filters" : "Show filters"}</span>
          </DropdownItem>

          <div className="my-1 h-px bg-border/70" />

          <div className="px-2 py-1">
            <p className="text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">Columns</p>
          </div>

          {columns.map((column) => {
            const isLastVisibleDataColumn =
              column.key !== "verified" && visibility[column.key] && visibleDataColumns === 1;

            return (
              <label
                key={column.key}
                className={
                  isLastVisibleDataColumn
                    ? "flex cursor-not-allowed items-center justify-between rounded-xl px-3 py-2 text-sm text-muted-foreground opacity-60"
                    : "flex cursor-pointer items-center justify-between rounded-xl px-3 py-2 text-sm text-foreground hover:bg-accent/60"
                }
              >
                <span>{column.label}</span>
                <input
                  aria-label={column.label}
                  checked={visibility[column.key]}
                  className="size-4 accent-[var(--primary)]"
                  disabled={isLastVisibleDataColumn}
                  type="checkbox"
                  onChange={() => onToggleColumn(column.key)}
                />
              </label>
            );
          })}
        </DropdownPanel>
      ) : null}
    </div>
  );
}
