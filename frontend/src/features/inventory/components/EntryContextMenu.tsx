import { useEffect, useRef } from "react";

import { DropdownItem } from "@/shared/components/ui/DropdownMenu";
import type { InventoryEntry, InventoryScope } from "@/features/inventory/types";

export type EntryContextAction = "open" | "open-link" | "search-online" | "archive-toggle" | "delete";

interface EntryContextMenuProps {
  canModifyEntries: boolean;
  onAction: (action: EntryContextAction) => void;
  onClose: () => void;
  position: {
    x: number;
    y: number;
  };
  entry: InventoryEntry;
  scope: InventoryScope;
}

export function EntryContextMenu({ canModifyEntries, onAction, onClose, position, entry, scope }: EntryContextMenuProps) {
  const menuRef = useRef<HTMLDivElement | null>(null);
  const archiveLabel = scope === "archive" || entry.archived ? "Restore Entry" : "Archive Entry";
  const hasSavedLink = entry.links.trim().length > 0;

  useEffect(() => {
    function handlePointerDown(event: MouseEvent): void {
      if (!menuRef.current?.contains(event.target as Node)) {
        onClose();
      }
    }

    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key === "Escape") {
        onClose();
      }
    }

    document.addEventListener("mousedown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [onClose]);

  return (
    <div
      className="fixed inset-0 z-30"
      onContextMenu={(event) => {
        event.preventDefault();
        onClose();
      }}
    >
      <div
        ref={menuRef}
        className="absolute w-56 overflow-hidden rounded-2xl border border-border/70 bg-card p-2 shadow-xl"
        role="menu"
        style={{ left: position.x, top: position.y }}
      >
        <div className="space-y-1">
          <DropdownItem onClick={() => onAction("open")}>Open Full Entry</DropdownItem>
          {hasSavedLink ? <DropdownItem onClick={() => onAction("open-link")}>Open Saved Link</DropdownItem> : null}
          <DropdownItem onClick={() => onAction("search-online")}>Search Online</DropdownItem>
          <div className="my-1 h-px bg-border/70" />
          <DropdownItem disabled={!canModifyEntries} onClick={() => onAction("archive-toggle")}>
            {archiveLabel}
          </DropdownItem>
          <DropdownItem destructive disabled={!canModifyEntries} onClick={() => onAction("delete")}>
            Delete Entry
          </DropdownItem>
        </div>
      </div>
    </div>
  );
}
