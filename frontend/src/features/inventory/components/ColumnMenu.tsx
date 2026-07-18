import { ChevronDownIcon, SlidersHorizontalIcon } from "lucide-react";

import { DropdownPanel } from "@/shared/components/ui/DropdownMenu";
import { useDropdownMenu } from "@/shared/hooks/useDropdownMenu";
import { Button } from "@/shared/components/ui/button";
import { getVisibleDataColumnCount } from "@/features/inventory/lib";
import type { ColumnConfig, ColumnKey } from "@/features/inventory/types";

interface ColumnMenuProps {
  columns: readonly ColumnConfig[];
  onToggleColumn: (columnKey: ColumnKey) => void;
  visibility: Record<ColumnKey, boolean>;
}

export function ColumnMenu({ columns, onToggleColumn, visibility }: ColumnMenuProps) {
  const { open, menuRef, toggle } = useDropdownMenu();
  const visibleDataColumns = getVisibleDataColumnCount(visibility);

  return (
    <div className="relative" ref={menuRef}>
      <Button aria-expanded={open} size="sm" variant="outline" onClick={toggle}>
        <SlidersHorizontalIcon className="size-3.5" />
        Columns
        <ChevronDownIcon className="size-3.5" />
      </Button>

      {open ? (
        <DropdownPanel
          align="right"
          className="w-64"
          maxHeightClassName="max-h-[min(24rem,calc(100vh-8rem))]"
          title="Visible columns"
        >
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
