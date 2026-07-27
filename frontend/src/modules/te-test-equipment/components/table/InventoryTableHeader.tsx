import type { MouseEvent as ReactMouseEvent, Ref } from "react";

import { cn } from "@/shared/lib/utils";
import type { ColumnConfig, SortState } from "@/modules/te-test-equipment/types";

import { getColumnStyle } from "./columnStyles";

interface InventoryTableColumnGroupProps {
  columns: readonly ColumnConfig[];
}

interface InventoryTableHeaderProps {
  columns: readonly ColumnConfig[];
  headerRef?: Ref<HTMLTableSectionElement | null>;
  onHeaderContextMenu?: (event: ReactMouseEvent) => void;
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

/** Horizontal edge fade so the sort cue softens into the cell sides. */
const SORT_EDGE_MASK =
  "[mask-image:linear-gradient(to_right,transparent_0%,black_32%,black_68%,transparent_100%)] [-webkit-mask-image:linear-gradient(to_right,transparent_0%,black_32%,black_68%,transparent_100%)]";

export function InventoryTableHeader({
  columns,
  headerRef,
  onHeaderContextMenu,
  onSortChange,
  sortState,
}: InventoryTableHeaderProps) {
  return (
    <thead ref={headerRef} className="sticky top-0 z-20 bg-card">
      <tr>
        {columns.map((column) => {
          const isActiveSort = sortState?.column === column.key;
          const sortDirection = isActiveSort ? sortState.direction : null;
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
                // p-0 so the control fills the cell; padding lives on the button/label.
                "relative overflow-hidden border-b border-border bg-card p-0 text-center text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground",
                isActiveSort && "text-foreground",
              )}
              scope="col"
              title={onHeaderContextMenu ? "Right-click to show or hide columns" : undefined}
              onContextMenu={onHeaderContextMenu}
            >
              {sortDirection === "asc" ? (
                <span
                  aria-hidden
                  className={cn(
                    "pointer-events-none absolute inset-x-0 top-0 z-[1] h-2.5 bg-gradient-to-b from-foreground/22 via-foreground/10 to-transparent dark:from-foreground/28 dark:via-foreground/12",
                    SORT_EDGE_MASK,
                  )}
                />
              ) : null}
              {sortDirection === "desc" ? (
                <span
                  aria-hidden
                  className={cn(
                    "pointer-events-none absolute inset-x-0 bottom-0 z-[1] h-2.5 bg-gradient-to-t from-foreground/22 via-foreground/10 to-transparent dark:from-foreground/28 dark:via-foreground/12",
                    SORT_EDGE_MASK,
                  )}
                />
              ) : null}

              {column.sortable ? (
                <button
                  aria-label={sortLabel}
                  aria-pressed={isActiveSort ? true : false}
                  className="relative z-[1] flex min-h-[2.75rem] w-full min-w-0 cursor-pointer items-center justify-center px-1.5 py-2.5 transition-colors hover:bg-accent/35 hover:text-foreground sm:min-h-[3rem] sm:px-2 sm:py-3"
                  type="button"
                  onClick={() => onSortChange(column.key)}
                  onContextMenu={onHeaderContextMenu}
                >
                  <span className="max-w-full truncate leading-none">{column.label}</span>
                </button>
              ) : (
                <span className="relative z-[1] flex min-h-[2.75rem] w-full items-center justify-center px-1.5 py-2.5 leading-none sm:min-h-[3rem] sm:px-2 sm:py-3">
                  <span className="max-w-full truncate">{column.label}</span>
                </span>
              )}
            </th>
          );
        })}
      </tr>
    </thead>
  );
}
