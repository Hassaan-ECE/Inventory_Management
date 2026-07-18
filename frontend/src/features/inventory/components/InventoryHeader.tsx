import { useState } from "react";
import { MoonIcon, PlusIcon, SunIcon } from "lucide-react";

import { ExportMenu } from "@/features/inventory/components/header/ExportMenu";
import { InventorySystemSwitcher } from "@/features/inventory/components/header/InventorySystemSwitcher";
import { ScopeToggle } from "@/features/inventory/components/header/ScopeToggle";
import { UpdateActionButton } from "@/features/inventory/components/header/UpdateActionButton";
import type { InventorySystemId } from "@/features/inventory/lib/inventorySystems";
import { Button } from "@/shared/components/ui/button";
import { cn } from "@/shared/lib/utils";
import type { InventoryScope, InventorySharedStatus, ThemeMode, UpdateState } from "@/features/inventory/types";

interface InventoryHeaderProps {
  activeSystemId: InventorySystemId;
  archiveCount: number;
  canModifyEntries: boolean;
  /** When false, hide inventory-scoped chrome (scope, export, add) for placeholder modules. */
  inventoryChrome: boolean;
  inventoryCount: number;
  onAddEntry: () => void;
  onExportExcel: () => void;
  onExportHtml: () => void;
  onScopeChange: (scope: InventoryScope) => void;
  onSystemChange: (id: InventorySystemId) => void;
  onThemeToggle: () => void;
  onUpdateAction: () => void;
  scope: InventoryScope;
  sharedStatus?: InventorySharedStatus;
  theme: ThemeMode;
  updateState: UpdateState;
}

export function InventoryHeader({
  activeSystemId,
  archiveCount,
  canModifyEntries,
  inventoryChrome,
  inventoryCount,
  onAddEntry,
  onExportExcel,
  onExportHtml,
  onScopeChange,
  onSystemChange,
  onThemeToggle,
  onUpdateAction,
  scope,
  sharedStatus,
  theme,
  updateState,
}: InventoryHeaderProps) {
  const [exportOpen, setExportOpen] = useState(false);
  const [systemMenuOpen, setSystemMenuOpen] = useState(false);
  const isLocalOnly = !sharedStatus?.enabled;
  const modeTitle = isLocalOnly
    ? sharedStatus?.message?.trim() ||
      "Shared sync is off for this session. Changes stay on this computer; sync is not a backup."
    : sharedStatus?.message?.trim() || "Shared sync enabled";

  return (
    <header
      className={cn(
        "relative shrink-0 border-b border-border px-3 py-3 sm:px-5",
        // Keep header above the search card so Export / system switcher are never covered.
        exportOpen || systemMenuOpen ? "z-50" : "z-30",
      )}
    >
      <div className="flex flex-wrap items-center gap-3">
        <div className="flex shrink-0 flex-wrap items-center gap-x-2.5 gap-y-1">
          <InventorySystemSwitcher
            value={activeSystemId}
            onChange={onSystemChange}
            onOpenChange={setSystemMenuOpen}
          />
          {inventoryChrome ? (
            <>
              <span
                className={
                  isLocalOnly
                    ? "shrink-0 rounded-full border border-border bg-muted/60 px-2.5 py-0.5 text-[11px] font-semibold tracking-wide text-muted-foreground"
                    : "shrink-0 rounded-full border border-emerald-500/25 bg-emerald-500/10 px-2.5 py-0.5 text-[11px] font-semibold tracking-wide text-emerald-700 dark:text-emerald-300"
                }
                title={modeTitle}
              >
                {isLocalOnly ? "Local" : "Shared"}
              </span>
              <UpdateActionButton state={updateState} onClick={onUpdateAction} />
            </>
          ) : null}
        </div>

        <div className="ml-auto flex flex-wrap items-center justify-end gap-2">
          {inventoryChrome ? (
            <ScopeToggle
              archiveCount={archiveCount}
              inventoryCount={inventoryCount}
              scope={scope}
              onScopeChange={onScopeChange}
            />
          ) : null}

          <Button size="sm" variant="outline" onClick={onThemeToggle}>
            {theme === "light" ? <MoonIcon className="size-3.5" /> : <SunIcon className="size-3.5" />}
            {theme === "light" ? "Dark Theme" : "Light Theme"}
          </Button>
          {inventoryChrome ? (
            <>
              <ExportMenu onExportExcel={onExportExcel} onExportHtml={onExportHtml} onOpenChange={setExportOpen} />
              <Button disabled={!canModifyEntries} size="sm" onClick={onAddEntry}>
                <PlusIcon className="size-3.5" />
                Add Entry
              </Button>
            </>
          ) : null}
        </div>
      </div>
    </header>
  );
}
