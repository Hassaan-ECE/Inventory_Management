import { useState } from "react";
import { MoonIcon, PlusIcon, SunIcon } from "lucide-react";

import { ExportMenu } from "@/modules/te-test-equipment/components/header/ExportMenu";
import { ScopeToggle } from "@/modules/te-test-equipment/components/header/ScopeToggle";
import { UpdateActionButton } from "@/modules/te-test-equipment/components/header/UpdateActionButton";
import type { InventoryScope, InventorySharedStatus, UpdateState } from "@/modules/te-test-equipment/types";
import type { ModuleId } from "@/platform/modules/types";
import type { ThemeMode } from "@/platform/ui/theme";
import { Button } from "@/shared/components/ui/button";
import { cn } from "@/shared/lib/utils";
import { InventorySystemSwitcher } from "@/shell/InventorySystemSwitcher";

interface InventoryHeaderProps {
  activeModuleId: ModuleId;
  archiveCount: number;
  canModifyEntries: boolean;
  inventoryCount: number;
  onAddEntry: () => void;
  onExportExcel: () => void;
  onExportHtml: () => void;
  onModuleChange: (id: ModuleId) => void;
  onScopeChange: (scope: InventoryScope) => void;
  onThemeToggle: () => void;
  onUpdateAction: () => void;
  scope: InventoryScope;
  sharedStatus?: InventorySharedStatus;
  theme: ThemeMode;
  updateState: UpdateState;
}

export function InventoryHeader({
  activeModuleId,
  archiveCount,
  canModifyEntries,
  inventoryCount,
  onAddEntry,
  onExportExcel,
  onExportHtml,
  onModuleChange,
  onScopeChange,
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
        <div className="flex min-w-0 flex-wrap items-center gap-x-2.5 gap-y-1">
          <InventorySystemSwitcher
            value={activeModuleId}
            onChange={onModuleChange}
            onOpenChange={setSystemMenuOpen}
          />
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
        </div>

        <div className="ml-auto flex flex-wrap items-center justify-end gap-2">
          <ScopeToggle
            archiveCount={archiveCount}
            inventoryCount={inventoryCount}
            scope={scope}
            onScopeChange={onScopeChange}
          />
          <Button size="sm" variant="outline" onClick={onThemeToggle}>
            {theme === "light" ? <MoonIcon className="size-3.5" /> : <SunIcon className="size-3.5" />}
            {theme === "light" ? "Dark Theme" : "Light Theme"}
          </Button>
          <ExportMenu onExportExcel={onExportExcel} onExportHtml={onExportHtml} onOpenChange={setExportOpen} />
          <Button disabled={!canModifyEntries} size="sm" onClick={onAddEntry}>
            <PlusIcon className="size-3.5" />
            Add Entry
          </Button>
        </div>
      </div>
    </header>
  );
}
