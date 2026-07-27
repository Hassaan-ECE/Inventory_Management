import { memo, useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from "react";

import { InventoryTableBody } from "@/modules/te-test-equipment/components/table/InventoryTableBody";
import { InventoryTableColumnGroup, InventoryTableHeader } from "@/modules/te-test-equipment/components/table/InventoryTableHeader";
import { ROW_HEIGHT, clampScrollTop, getVisibleRange } from "@/modules/te-test-equipment/components/table/virtualization";
import { getLocalDateString, getVisibleDataColumnCount } from "@/modules/te-test-equipment/lib";
import type {
  ColumnConfig,
  ColumnKey,
  InventoryEntry,
  SortState,
  TeTestEquipmentWorkspace,
} from "@/modules/te-test-equipment/types";
import { DropdownPanel } from "@/shared/components/ui/DropdownMenu";
import { ScrollRegion } from "@/shared/components/ui/ScrollRegion";
import { placeFloatingMenu, type FloatingMenuPlacement } from "@/shared/lib/floatingMenu";

interface InventoryTableProps {
  activeEntryId?: string | null;
  allColumns: readonly ColumnConfig[];
  canModifyEntries: boolean;
  colorRows: boolean;
  columnVisibility: Record<ColumnKey, boolean>;
  columns: readonly ColumnConfig[];
  onOpenContextMenu: (entryId: string, clientX: number, clientY: number) => void;
  onOpenEntry: (entryId: string) => void;
  onOpenExternalLink: (url: string) => void;
  onSortChange: (columnKey: ColumnConfig["key"]) => void;
  onToggleColumn: (columnKey: ColumnKey) => void;
  onToggleVerified: (entryId: string) => void;
  entries: InventoryEntry[];
  sortState: SortState | null;
  localDate?: string;
  workspace?: TeTestEquipmentWorkspace;
}

interface ColumnMenuState extends FloatingMenuPlacement {
  anchorX: number;
  anchorY: number;
}

export const InventoryTable = memo(function InventoryTable({
  activeEntryId = null,
  allColumns,
  canModifyEntries,
  colorRows,
  columnVisibility,
  columns,
  onOpenContextMenu,
  onOpenEntry,
  onOpenExternalLink,
  onSortChange,
  onToggleColumn,
  onToggleVerified,
  entries,
  sortState,
  localDate = getLocalDateString(),
  workspace = "equipment",
}: InventoryTableProps) {
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const headerRef = useRef<HTMLTableSectionElement | null>(null);
  const menuRef = useRef<HTMLDivElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(640);
  const [headerHeight, setHeaderHeight] = useState(0);
  const [columnMenu, setColumnMenu] = useState<ColumnMenuState | null>(null);
  const visibleRange = useMemo(
    () => getVisibleRange(entries.length, scrollTop, viewportHeight),
    [entries.length, scrollTop, viewportHeight],
  );
  const visibleEntries = entries.slice(visibleRange.start, visibleRange.end);
  const topSpacerHeight = visibleRange.start * ROW_HEIGHT;
  const bottomSpacerHeight = Math.max(0, (entries.length - visibleRange.end) * ROW_HEIGHT);
  const visibleDataColumns = getVisibleDataColumnCount(columnVisibility, allColumns);

  const measureHeaderHeight = useCallback(() => {
    const header = headerRef.current;
    if (!header) {
      return;
    }
    const nextHeight = Math.round(header.getBoundingClientRect().height);
    setHeaderHeight((current) => (current === nextHeight ? current : nextHeight));
  }, []);

  useEffect(() => {
    const node = scrollRef.current;
    if (!node) {
      return undefined;
    }

    setViewportHeight(node.clientHeight || 640);
    measureHeaderHeight();
    if (typeof ResizeObserver === "undefined") {
      return undefined;
    }

    const observer = new ResizeObserver(() => {
      setViewportHeight(node.clientHeight || 640);
      measureHeaderHeight();
    });
    observer.observe(node);
    if (headerRef.current) {
      observer.observe(headerRef.current);
    }
    return () => observer.disconnect();
  }, [columns, measureHeaderHeight]);

  useEffect(() => {
    const node = scrollRef.current;
    setScrollTop((currentScrollTop) => {
      const nextScrollTop = clampScrollTop(currentScrollTop, entries.length, viewportHeight);
      if (node && node.scrollTop !== nextScrollTop) {
        node.scrollTop = nextScrollTop;
      }

      return currentScrollTop === nextScrollTop ? currentScrollTop : nextScrollTop;
    });
  }, [entries.length, viewportHeight]);

  useEffect(() => {
    if (!columnMenu) {
      return undefined;
    }
    function handlePointerDown(event: MouseEvent): void {
      if (!menuRef.current?.contains(event.target as Node)) {
        setColumnMenu(null);
      }
    }
    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        setColumnMenu(null);
      }
    }
    document.addEventListener("mousedown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [columnMenu]);

  useLayoutEffect(() => {
    if (!columnMenu || !menuRef.current) {
      return undefined;
    }

    function refinePlacement(): void {
      const node = menuRef.current;
      if (!node || !columnMenu) {
        return;
      }
      const rect = node.getBoundingClientRect();
      const next = placeFloatingMenu(columnMenu.anchorX, columnMenu.anchorY, {
        width: rect.width,
        height: rect.height,
      });
      if (
        next.x !== columnMenu.x
        || next.y !== columnMenu.y
        || next.maxHeight !== columnMenu.maxHeight
      ) {
        setColumnMenu({
          anchorX: columnMenu.anchorX,
          anchorY: columnMenu.anchorY,
          ...next,
        });
      }
    }

    refinePlacement();
    window.addEventListener("resize", refinePlacement);
    return () => window.removeEventListener("resize", refinePlacement);
  }, [columnMenu]);

  return (
    <section className="relative flex h-full min-h-0 flex-1 overflow-hidden rounded-xl border border-border/70 bg-card/80 shadow-sm">
      <ScrollRegion
        aria-label={workspace === "calibration" ? "Calibration equipment table" : "Inventory table"}
        className="min-h-0 h-full flex-1"
        scrollClassName="overflow-x-hidden"
        scrollRef={scrollRef}
        topCueClassName="z-10"
        topCueStyle={headerHeight > 0 ? { top: headerHeight } : undefined}
        onScroll={(event) => {
          const nextViewportHeight = event.currentTarget.clientHeight || viewportHeight;
          setScrollTop(clampScrollTop(event.currentTarget.scrollTop, entries.length, nextViewportHeight));
        }}
      >
        <table className="w-full table-fixed border-separate border-spacing-0">
          <InventoryTableColumnGroup columns={columns} />
          <InventoryTableHeader
            columns={columns}
            headerRef={headerRef}
            sortState={sortState}
            onHeaderContextMenu={(event) => {
              event.preventDefault();
              event.stopPropagation();
              const placement = placeFloatingMenu(event.clientX, event.clientY);
              setColumnMenu({
                anchorX: event.clientX,
                anchorY: event.clientY,
                ...placement,
              });
            }}
            onSortChange={onSortChange}
          />
          <InventoryTableBody
            activeEntryId={activeEntryId}
            bottomSpacerHeight={bottomSpacerHeight}
            canModifyEntries={canModifyEntries}
            colorRows={colorRows}
            columns={columns}
            topSpacerHeight={topSpacerHeight}
            visibleEntries={visibleEntries}
            localDate={localDate}
            workspace={workspace}
            onOpenContextMenu={onOpenContextMenu}
            onOpenEntry={onOpenEntry}
            onOpenExternalLink={onOpenExternalLink}
            onToggleVerified={onToggleVerified}
          />
        </table>
      </ScrollRegion>

      {columnMenu ? (
        <div className="fixed z-[60]" ref={menuRef} style={{ left: columnMenu.x, top: columnMenu.y }}>
          <DropdownPanel
            align="left"
            className="relative right-auto mt-0 w-72"
            maxHeightPx={columnMenu.maxHeight}
            title="Columns"
          >
            {allColumns.map((column) => {
              const isLastVisibleDataColumn =
                column.key !== "verified" && columnVisibility[column.key] && visibleDataColumns === 1;
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
                    checked={columnVisibility[column.key]}
                    className="size-4 accent-[var(--primary)]"
                    disabled={isLastVisibleDataColumn}
                    type="checkbox"
                    onChange={() => onToggleColumn(column.key)}
                  />
                </label>
              );
            })}
          </DropdownPanel>
        </div>
      ) : null}
    </section>
  );
});
