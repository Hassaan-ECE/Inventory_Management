import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";

import { InventoryTableBody } from "@/features/inventory/components/table/InventoryTableBody";
import { InventoryTableColumnGroup, InventoryTableHeader } from "@/features/inventory/components/table/InventoryTableHeader";
import { ROW_HEIGHT, clampScrollTop, getVisibleRange } from "@/features/inventory/components/table/virtualization";
import type { ColumnConfig, InventoryEntry, SortState } from "@/features/inventory/types";
import { getLocalDateString } from "@/features/inventory/lib";
import { ScrollRegion } from "@/shared/components/ui/ScrollRegion";

interface InventoryTableProps {
  activeEntryId?: string | null;
  canModifyEntries: boolean;
  colorRows: boolean;
  columns: readonly ColumnConfig[];
  onOpenContextMenu: (entryId: string, clientX: number, clientY: number) => void;
  onOpenEntry: (entryId: string) => void;
  onOpenExternalLink: (url: string) => void;
  onSortChange: (columnKey: ColumnConfig["key"]) => void;
  onToggleVerified: (entryId: string) => void;
  entries: InventoryEntry[];
  sortState: SortState | null;
  localDate?: string;
}

export const InventoryTable = memo(function InventoryTable({
  activeEntryId = null,
  canModifyEntries,
  colorRows,
  columns,
  onOpenContextMenu,
  onOpenEntry,
  onOpenExternalLink,
  onSortChange,
  onToggleVerified,
  entries,
  sortState,
  localDate = getLocalDateString(),
}: InventoryTableProps) {
  const scrollRef = useRef<HTMLDivElement | null>(null);
  const headerRef = useRef<HTMLTableSectionElement | null>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [viewportHeight, setViewportHeight] = useState(640);
  const [headerHeight, setHeaderHeight] = useState(0);
  const visibleRange = useMemo(
    () => getVisibleRange(entries.length, scrollTop, viewportHeight),
    [entries.length, scrollTop, viewportHeight],
  );
  const visibleEntries = entries.slice(visibleRange.start, visibleRange.end);
  const topSpacerHeight = visibleRange.start * ROW_HEIGHT;
  const bottomSpacerHeight = Math.max(0, (entries.length - visibleRange.end) * ROW_HEIGHT);

  const measureHeaderHeight = useCallback(() => {
    const header = headerRef.current;
    if (!header) {
      return;
    }
    // Round so the fade sits flush under the sticky header (no 1px hairline gap).
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

  return (
    <section className="flex h-full min-h-0 flex-1 overflow-hidden rounded-xl border border-border/70 bg-card/80 shadow-sm">
      <ScrollRegion
        aria-label="Inventory table"
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
            onOpenContextMenu={onOpenContextMenu}
            onOpenEntry={onOpenEntry}
            onOpenExternalLink={onOpenExternalLink}
            onToggleVerified={onToggleVerified}
          />
        </table>
      </ScrollRegion>
    </section>
  );
});
