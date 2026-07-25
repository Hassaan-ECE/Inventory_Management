import { useEffect, useMemo, useRef, useState } from "react";

import { columnLabel, gridCoordinateLabel, partDisplayName } from "@/modules/te-lab-components/catalog/catalogUtils";
import type { Part, StockPlacement, StorageContainer } from "@/modules/te-lab-components/types";
import { Button } from "@/shared/components/ui/button";
import { cn } from "@/shared/lib/utils";

const PAGE_ROWS = 30;
const PAGE_COLUMNS = 30;

interface GridBinPickerProps {
  areaName: string;
  container: StorageContainer;
  disabled?: boolean;
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

  return (
    <div className="space-y-2">
      <div className="flex flex-wrap items-center justify-between gap-2 text-xs text-muted-foreground">
        <span>
          Rows {rowOffset + 1}–{Math.min(rowOffset + PAGE_ROWS, rowCount)} of {rowCount}; columns{" "}
          {columnLabel(columnOffset)}–{columnLabel(Math.min(columnOffset + PAGE_COLUMNS, columnCount) - 1)} of{" "}
          {columnCount}
        </span>
        <div className="flex gap-1">
          <Button
            disabled={rowOffset === 0}
            onClick={() => setRowOffset(Math.max(0, rowOffset - PAGE_ROWS))}
            size="xs"
            variant="outline"
          >
            Earlier rows
          </Button>
          <Button
            disabled={rowOffset + PAGE_ROWS >= rowCount}
            onClick={() => setRowOffset(Math.min(rowCount - 1, rowOffset + PAGE_ROWS))}
            size="xs"
            variant="outline"
          >
            Later rows
          </Button>
          <Button
            disabled={columnOffset === 0}
            onClick={() => setColumnOffset(Math.max(0, columnOffset - PAGE_COLUMNS))}
            size="xs"
            variant="outline"
          >
            Earlier columns
          </Button>
          <Button
            disabled={columnOffset + PAGE_COLUMNS >= columnCount}
            onClick={() => setColumnOffset(Math.min(columnCount - 1, columnOffset + PAGE_COLUMNS))}
            size="xs"
            variant="outline"
          >
            Later columns
          </Button>
        </div>
      </div>
      <div className="max-h-[54vh] overflow-auto rounded-xl border border-border bg-muted/20 p-2">
        <div
          className="grid w-max gap-1"
          ref={gridRef}
          style={{ gridTemplateColumns: `2.5rem repeat(${visibleColumns.length}, 3.25rem)` }}
        >
          <div aria-hidden="true" className="sticky left-0 top-0 z-20 bg-background" />
          {visibleColumns.map((columnIndex) => (
            <div
              className="sticky top-0 z-10 flex h-8 items-center justify-center rounded-md bg-background text-xs font-semibold shadow-sm"
              key={columnIndex}
            >
              {columnLabel(columnIndex)}
            </div>
          ))}
          {visibleRows.flatMap((rowIndex) => [
            <div
              className="sticky left-0 z-10 flex h-11 items-center justify-center rounded-md bg-background text-xs font-semibold shadow-sm"
              key={`row-${rowIndex}`}
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
                    "relative flex h-11 w-[3.25rem] items-center justify-center rounded-md border text-[11px] font-medium outline-none focus-visible:ring-2 focus-visible:ring-ring",
                    contents.length > 0
                      ? "border-amber-500/50 bg-amber-500/10 text-amber-900 dark:text-amber-100"
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
                  {contents.length > 0 ? (
                    <span className="absolute right-0.5 top-0.5 rounded-full bg-amber-600 px-1 text-[9px] leading-3 text-white">
                      {contents.length}
                    </span>
                  ) : null}
                </button>
              );
            }),
          ])}
        </div>
      </div>
      <p className="text-xs text-muted-foreground">
        Occupied bins show a count badge. They remain selectable so several part types can intentionally share one bin.
      </p>
    </div>
  );
}

function pageOffset(index: number | null, pageSize: number): number {
  return index === null ? 0 : Math.floor(index / pageSize) * pageSize;
}
