import { ArrowDownIcon, ArrowUpIcon } from "lucide-react";
import type { Ref } from "react";

import { cn } from "@/shared/lib/utils";
import type { ColumnConfig, SortState } from "@/features/inventory/types";

import { getColumnStyle } from "./columnStyles";

interface InventoryTableColumnGroupProps {
  columns: readonly ColumnConfig[];
}

interface InventoryTableHeaderProps {
  columns: readonly ColumnConfig[];
  headerRef?: Ref<HTMLTableSectionElement | null>;
  onSortChange: (columnKey: ColumnConfig["key"]) => void;
  sortState: SortState | null;
}

export function InventoryTableColumnGroup({ columns }: InventoryTableColumnGroupProps) {
  return (
    <colgroup>
      {columns.map((column) => (
        <col key={column.key} style={getColumnStyle(column.key)} />
      ))}
    </colgroup>
  );
}

export function InventoryTableHeader({ columns, headerRef, onSortChange, sortState }: InventoryTableHeaderProps) {
  return (
    <thead ref={headerRef} className="sticky top-0 z-20 bg-card">
      <tr>
        {columns.map((column) => {
          const isActiveSort = sortState?.column === column.key;
          const sortLabel = !column.sortable
            ? undefined
            : isActiveSort
              ? sortState.direction === "asc"
                ? `Sort by ${column.label}, currently ascending. Activate for descending`
                : `Sort by ${column.label}, currently descending. Activate to clear sort`
              : `Sort by ${column.label}`;

          return (
            <th
              key={column.key}
              className={cn(
                "border-b border-border bg-card px-2.5 py-2.5 text-left text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground sm:px-4 sm:py-3",
                column.align === "center" ? "text-center" : "text-left",
              )}
              scope="col"
            >
              {column.sortable ? (
                <button
                  aria-label={sortLabel}
                  aria-pressed={isActiveSort ? true : false}
                  className={cn(
                    "inline-flex min-w-0 max-w-full items-center gap-1 transition-colors hover:text-foreground",
                    column.align === "center" ? "justify-center" : "",
                    isActiveSort ? "text-foreground" : "",
                  )}
                  type="button"
                  onClick={() => onSortChange(column.key)}
                >
                  <span>{column.label}</span>
                  {isActiveSort ? (
                    sortState.direction === "asc" ? (
                      <ArrowUpIcon aria-hidden className="size-3.5 shrink-0" />
                    ) : (
                      <ArrowDownIcon aria-hidden className="size-3.5 shrink-0" />
                    )
                  ) : null}
                </button>
              ) : (
                column.label
              )}
            </th>
          );
        })}
      </tr>
    </thead>
  );
}
