import { useState } from "react";
import { FileSpreadsheetIcon, MoonIcon, PlusIcon, SunIcon } from "lucide-react";

import { ExportMenu } from "@/modules/te-test-equipment/components/header/ExportMenu";
import { ScopeToggle } from "@/modules/te-test-equipment/components/header/ScopeToggle";
import { UpdateActionButton } from "@/modules/te-test-equipment/components/header/UpdateActionButton";
import type {
  InventoryScope,
  InventorySharedStatus,
  TeTestEquipmentWorkspace,
  UpdateState,
} from "@/modules/te-test-equipment/types";
import type { InventoryViewId } from "@/platform/modules/types";
import type { ThemeMode } from "@/platform/ui/theme";
import { Button } from "@/shared/components/ui/button";
import { cn } from "@/shared/lib/utils";
import { InventorySystemSwitcher } from "@/shell/InventorySystemSwitcher";

interface InventoryHeaderProps {
  activeViewId: InventoryViewId;
  archiveCount: number;
  calibrationRosterAvailable: boolean;
  canModifyEntries: boolean;
  inventoryCount: number;
  onAddEntry: () => void;
  onExportExcel: () => void;
  onExportHtml: () => void;
  onInitializeCalibrationRoster: () => void;
  onScopeChange: (scope: InventoryScope) => void;
  onThemeToggle: () => void;
  onUpdateAction: () => void;
  onViewChange: (id: InventoryViewId) => void;
  scope: InventoryScope;
  sharedStatus?: InventorySharedStatus;
  theme: ThemeMode;
  updateState: UpdateState;
  workspace: TeTestEquipmentWorkspace;
}

export function InventoryHeader({
  activeViewId,
  archiveCount,
  calibrationRosterAvailable,
  canModifyEntries,
  inventoryCount,
  onAddEntry,
  onExportExcel,
  onExportHtml,
  onInitializeCalibrationRoster,
  onScopeChange,
  onThemeToggle,
  onUpdateAction,
  onViewChange,
  scope,
  sharedStatus,
  theme,
  updateState,
  workspace,
}: InventoryHeaderProps) {
  const [exportOpen, setExportOpen] = useState(false);
  const [systemMenuOpen, setSystemMenuOpen] = useState(false);
  const isLocalOnly = !sharedStatus?.enabled;
  const addEntryLabel = workspace === "calibration" ? "Add Equipment" : "Add Entry";
  const themeLabel = theme === "light" ? "Dark Theme" : "Light Theme";
  const modeTitle = isLocalOnly
    ? sharedStatus?.message?.trim() ||
      "Shared sync is off for this session. Changes stay on this computer; sync is not a backup."
    : sharedStatus?.message?.trim() || "Shared sync enabled";

  return (
    <header
      className={cn(
        "relative shrink-0 border-b border-border px-3 py-3 sm:px-5",
        // Keep switcher / export menus above search, filters, and table chrome.
        exportOpen || systemMenuOpen ? "z-[80]" : "z-40",
      )}
    >
      <div className="flex flex-wrap items-center gap-2 sm:gap-3">
        <div className="flex min-w-0 flex-wrap items-center gap-x-2.5 gap-y-1">
          <InventorySystemSwitcher
            value={activeViewId}
            onChange={onViewChange}
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
        </div>

        <div className="ml-auto flex min-w-0 flex-wrap items-center justify-end gap-2">
          <UpdateActionButton state={updateState} onClick={onUpdateAction} />
          <ScopeToggle
            archiveCount={archiveCount}
            inventoryCount={inventoryCount}
            scope={scope}
            onScopeChange={onScopeChange}
          />
          <Button
            aria-label={themeLabel}
            className="size-8"
            size="icon"
            title={themeLabel}
            variant="outline"
            onClick={onThemeToggle}
          >
            {theme === "light" ? <MoonIcon className="size-3.5" /> : <SunIcon className="size-3.5" />}
          </Button>
          <ExportMenu onExportExcel={onExportExcel} onExportHtml={onExportHtml} onOpenChange={setExportOpen} />
          {workspace === "calibration" && calibrationRosterAvailable ? (
            <Button
              aria-label="Initialize from Calibration Workbook"
              className="size-8"
              disabled={!canModifyEntries}
              size="icon"
              title="Initialize from Calibration Workbook"
              variant="outline"
              onClick={onInitializeCalibrationRoster}
            >
              <FileSpreadsheetIcon className="size-3.5" />
            </Button>
          ) : null}
          <Button
            aria-label={addEntryLabel}
            className="size-8"
            disabled={!canModifyEntries}
            size="icon"
            title={addEntryLabel}
            onClick={onAddEntry}
          >
            <PlusIcon className="size-3.5" />
          </Button>
        </div>
      </div>
    </header>
  );
}
