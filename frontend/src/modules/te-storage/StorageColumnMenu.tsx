import { useLayoutEffect, useState } from "react";
import { DropdownPanel } from "@/shared/components/ui/DropdownMenu";
import { useDropdownMenu } from "@/shared/hooks/useDropdownMenu";
import { placeFloatingMenu } from "@/shared/lib/floatingMenu";
import { STORAGE_COLUMNS, type StorageColumn } from "./types";

interface Props {
  anchor: { x: number; y: number };
  visibility: Record<StorageColumn, boolean>;
  onToggle: (key: StorageColumn) => void;
  onClose: () => void;
}

export function StorageColumnMenu({ anchor, visibility, onToggle, onClose }: Props) {
  const { menuRef } = useDropdownMenu({ open: true, onOpenChange: open => { if (!open) onClose(); } });
  const [placement, setPlacement] = useState(() => placeFloatingMenu(anchor.x, anchor.y));
  const visibleCount = STORAGE_COLUMNS.filter(({ key }) => visibility[key]).length;

  useLayoutEffect(() => {
    menuRef.current?.querySelector<HTMLInputElement>("input")?.focus({ preventScroll: true });
  }, [menuRef]);

  useLayoutEffect(() => {
    function refinePlacement() {
      const rect = menuRef.current?.getBoundingClientRect();
      if (!rect) return;
      const next = placeFloatingMenu(anchor.x, anchor.y, rect);
      setPlacement(current => current.x === next.x && current.y === next.y && current.maxHeight === next.maxHeight ? current : next);
    }
    refinePlacement();
    window.addEventListener("resize", refinePlacement);
    return () => window.removeEventListener("resize", refinePlacement);
  }, [anchor.x, anchor.y, menuRef]);

  return (
    <div ref={menuRef} className="fixed z-[60]" style={{ left: placement.x, top: placement.y }}>
      <DropdownPanel align="left" className="relative right-auto mt-0 w-72" maxHeightPx={placement.maxHeight} title="Columns">
        {STORAGE_COLUMNS.map(({ key, label }) => {
          const lastVisible = visibility[key] && visibleCount === 1;
          return (
            <label key={key} className={lastVisible
              ? "flex cursor-not-allowed items-center justify-between rounded-xl px-3 py-2 text-sm text-muted-foreground opacity-60"
              : "flex cursor-pointer items-center justify-between rounded-xl px-3 py-2 text-sm text-foreground hover:bg-accent/60"}>
              <span>{label}</span>
              <input aria-label={label} checked={visibility[key]} className="size-4 accent-[var(--primary)]"
                disabled={lastVisible} type="checkbox" onChange={() => onToggle(key)} />
            </label>
          );
        })}
      </DropdownPanel>
    </div>
  );
}
