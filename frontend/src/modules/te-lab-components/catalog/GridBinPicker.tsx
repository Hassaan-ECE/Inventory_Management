import { useEffect, useMemo, useRef, useState } from "react";
import { PlusIcon } from "lucide-react";

import { columnLabel, gridCoordinateLabel, partDisplayName } from "@/modules/te-lab-components/catalog/catalogUtils";
import type { Part, StockPlacement, StorageContainer } from "@/modules/te-lab-components/types";
import { cn } from "@/shared/lib/utils";

const PAGE_ROWS = 30;
const PAGE_COLUMNS = 30;
const MAX_GRID_SIZE = 1000;

interface GridBinPickerProps {
  areaName: string;
  container: StorageContainer;
  disabled?: boolean;
  onExpand?: (axis: "row" | "column") => void;
  onSelect?: (rowIndex: number, columnIndex: number) => void;
  partsById: Map<string, Part>;
  placements: StockPlacement[];
  selectedColumnIndex: number | null;
  selectedRowIndex: number | null;
}

export function GridBinPicker({
  areaName,
  container,
  disabled = false,
  onExpand,
  onSelect,
  partsById,
  placements,
  selectedColumnIndex,
  selectedRowIndex,
}: GridBinPickerProps) {
  const rowCount = container.rowCount ?? 0;
  const columnCount = container.columnCount ?? 0;
  const [rowOffset, setRowOffset] = useState(() => pageOffset(selectedRowIndex, PAGE_ROWS));
  const [columnOffset, setColumnOffset] = useState(() => pageOffset(selectedColumnIndex, PAGE_COLUMNS));
  const pendingFocus = useRef<{ rowIndex: number; columnIndex: number } | null>(null);
  const gridRef = useRef<HTMLDivElement | null>(null);

  useEffect(() => {
    const target = pendingFocus.current;
    if (!target) {
      return;
    }
    pendingFocus.current = null;
    requestAnimationFrame(() => {
      gridRef.current
        ?.querySelector<HTMLButtonElement>(
          `[data-grid-row="${target.rowIndex}"][data-grid-column="${target.columnIndex}"]`,
        )
        ?.focus();
    });
  }, [columnOffset, rowOffset]);

  const occupied = useMemo(() => {
    const byCoordinate = new Map<string, StockPlacement[]>();
    for (const placement of placements) {
      if (
        placement.archived ||
        placement.containerUuid !== container.containerUuid ||
        placement.rowIndex === null ||
        placement.columnIndex === null
      ) {
        continue;
      }
      const key = `${placement.rowIndex}:${placement.columnIndex}`;
      const current = byCoordinate.get(key) ?? [];
      current.push(placement);
      byCoordinate.set(key, current);
    }
    return byCoordinate;
  }, [container.containerUuid, placements]);

  const visibleRows = Array.from(
    { length: Math.min(PAGE_ROWS, Math.max(0, rowCount - rowOffset)) },
    (_, index) => rowOffset + index,
  );
  const visibleColumns = Array.from(
    { length: Math.min(PAGE_COLUMNS, Math.max(0, columnCount - columnOffset)) },
    (_, index) => columnOffset + index,
  );

  function focusCoordinate(rowIndex: number, columnIndex: number): void {
    if (rowIndex < 0 || columnIndex < 0 || rowIndex >= rowCount || columnIndex >= columnCount) {
      return;
    }
    const nextRowOffset = Math.floor(rowIndex / PAGE_ROWS) * PAGE_ROWS;
    const nextColumnOffset = Math.floor(columnIndex / PAGE_COLUMNS) * PAGE_COLUMNS;
    if (nextRowOffset !== rowOffset || nextColumnOffset !== columnOffset) {
      pendingFocus.current = { rowIndex, columnIndex };
      setRowOffset(nextRowOffset);
      setColumnOffset(nextColumnOffset);
      return;
    }
    gridRef.current
      ?.querySelector<HTMLButtonElement>(
        `[data-grid-row="${rowIndex}"][data-grid-column="${columnIndex}"]`,
      )
      ?.focus();
  }

  if (!container.gridEnabled || rowCount === 0 || columnCount === 0) {
    return <p className="text-sm text-muted-foreground">This container does not use a grid.</p>;
  }

  const lastVisibleColumn = visibleColumns[visibleColumns.length - 1];
  const lastVisibleRow = visibleRows[visibleRows.length - 1];
  const showAddColumn = Boolean(onExpand) && !disabled && lastVisibleColumn === columnCount - 1 && columnCount < MAX_GRID_SIZE;
  const showAddRow = Boolean(onExpand) && !disabled && lastVisibleRow === rowCount - 1 && rowCount < MAX_GRID_SIZE;
  const columnTracks = visibleColumns.length + (showAddColumn ? 1 : 0);

  return (
    <div className="space-y-2">
      <div className="max-h-[40vh] overflow-auto rounded-lg border border-border bg-muted/20 p-2">
        <div
          className="grid w-max gap-1"
          ref={gridRef}
          style={{ gridTemplateColumns: `2.75rem repeat(${columnTracks}, 3.25rem)` }}
        >
          <div
            aria-hidden="true"
            className="sticky left-0 top-0 z-20 flex h-8 items-center justify-center rounded-sm bg-zinc-300 text-[9px] font-semibold uppercase tracking-wide text-zinc-700 dark:bg-zinc-600 dark:text-zinc-100"
          >
            #
          </div>
          {visibleColumns.map((columnIndex) => (
            <div
              className="sticky top-0 z-10 flex h-8 items-center justify-center rounded-sm bg-sky-300 text-[11px] font-bold tracking-wide text-sky-950 dark:bg-sky-600 dark:text-white"
              key={columnIndex}
              title={`Column ${columnLabel(columnIndex)}`}
            >
              {columnLabel(columnIndex)}
            </div>
          ))}
          {showAddColumn ? (
            <button
              aria-label="Add column"
              className="sticky top-0 z-10 flex h-8 items-center justify-center rounded-sm border border-dashed border-sky-700 bg-sky-300 text-sky-950 hover:bg-sky-200 dark:border-sky-300 dark:bg-sky-600 dark:text-white dark:hover:bg-sky-500"
              type="button"
              onClick={() => onExpand?.("column")}
            >
              <PlusIcon className="size-3.5" />
            </button>
          ) : null}
          {visibleRows.flatMap((rowIndex) => [
            <div
              className="sticky left-0 z-10 flex h-11 items-center justify-center rounded-sm bg-amber-300 text-[11px] font-bold tabular-nums text-amber-950 dark:bg-amber-600 dark:text-white"
              key={`row-${rowIndex}`}
              title={`Row ${container.rowStart + rowIndex}`}
            >
              {container.rowStart + rowIndex}
            </div>,
            ...visibleColumns.map((columnIndex) => {
              const coordinate = gridCoordinateLabel(container, rowIndex, columnIndex);
              const contents = occupied.get(`${rowIndex}:${columnIndex}`) ?? [];
              const selected = rowIndex === selectedRowIndex && columnIndex === selectedColumnIndex;
              const contentSummary = contents.length
                ? contents
                    .map((placement) => {
                      const part = partsById.get(placement.partUuid);
                      return `${part ? partDisplayName(part) : "Unknown part"}: ${placement.quantity} ${placement.unitOfMeasure}`;
                    })
                    .join(", ")
                : "empty";
              const accessibleLabel = `${areaName} / ${container.name} / ${coordinate}; ${contentSummary}`;
              return (
                <button
                  aria-label={accessibleLabel}
                  aria-pressed={selected}
                  className={cn(
                    "relative flex h-11 w-[3.25rem] items-center justify-center rounded-sm border text-[11px] font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring",
                    contents.length > 0
                      ? "border-amber-600 bg-amber-200 text-amber-950 dark:border-amber-400 dark:bg-amber-700 dark:text-amber-50"
                      : "border-border bg-background hover:bg-accent",
                    selected && "border-primary bg-primary text-primary-foreground hover:bg-primary",
                    disabled && "cursor-not-allowed opacity-55",
                  )}
                  data-grid-column={columnIndex}
                  data-grid-row={rowIndex}
                  disabled={disabled}
                  key={`${rowIndex}:${columnIndex}`}
                  onClick={() => onSelect?.(rowIndex, columnIndex)}
                  onKeyDown={(event) => {
                    const movement = {
                      ArrowDown: [1, 0],
                      ArrowLeft: [0, -1],
                      ArrowRight: [0, 1],
                      ArrowUp: [-1, 0],
                    }[event.key];
                    if (!movement) {
                      return;
                    }
                    event.preventDefault();
                    focusCoordinate(rowIndex + movement[0], columnIndex + movement[1]);
                  }}
                  title={accessibleLabel}
                  type="button"
                >
                  <span>{coordinate}</span>
                </button>
              );
            }),
            showAddColumn ? (
              <div aria-hidden="true" className="h-11" key={`pad-col-${rowIndex}`} />
            ) : null,
          ])}
          {showAddRow ? (
            <>
              <button
                aria-label="Add row"
                className="sticky left-0 z-10 flex h-8 items-center justify-center rounded-sm border border-dashed border-amber-700 bg-amber-300 text-amber-950 hover:bg-amber-200 dark:border-amber-300 dark:bg-amber-600 dark:text-white dark:hover:bg-amber-500"
                type="button"
                onClick={() => onExpand?.("row")}
              >
                <PlusIcon className="size-3.5" />
              </button>
              {visibleColumns.map((columnIndex) => (
                <div aria-hidden="true" className="h-8" key={`pad-row-${columnIndex}`} />
              ))}
            </>
          ) : null}
        </div>
      </div>
    </div>
  );
}

function pageOffset(index: number | null, pageSize: number): number {
  return index === null ? 0 : Math.floor(index / pageSize) * pageSize;
}
