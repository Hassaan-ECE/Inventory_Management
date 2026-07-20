import { useState } from "react";
import { PlusIcon } from "lucide-react";

import { ExportMenu } from "@/modules/te-test-equipment/components/header/ExportMenu";
import { ScopeToggle } from "@/modules/te-test-equipment/components/header/ScopeToggle";
import { UpdateActionButton } from "@/modules/te-test-equipment/components/header/UpdateActionButton";
import type { InventoryScope, InventorySharedStatus, UpdateState } from "@/modules/te-test-equipment/types";
import { Button } from "@/shared/components/ui/button";
import { cn } from "@/shared/lib/utils";

interface InventoryHeaderProps {
  archiveCount: number;
  canModifyEntries: boolean;
  inventoryCount: number;
  onAddEntry: () => void;
  onExportExcel: () => void;
  onExportHtml: () => void;
  onScopeChange: (scope: InventoryScope) => void;
  onUpdateAction: () => void;
  scope: InventoryScope;
  sharedStatus?: InventorySharedStatus;
  updateState: UpdateState;
}

export function InventoryHeader({
  archiveCount,
  canModifyEntries,
  inventoryCount,
  onAddEntry,
  onExportExcel,
  onExportHtml,
  onScopeChange,
  onUpdateAction,
  scope,
  sharedStatus,
  updateState,
}: InventoryHeaderProps) {
  const [exportOpen, setExportOpen] = useState(false);
  const isLocalOnly = !sharedStatus?.enabled;
  const modeTitle = isLocalOnly
    ? sharedStatus?.message?.trim() ||
      "Shared sync is off for this session. Changes stay on this computer; sync is not a backup."
    : sharedStatus?.message?.trim() || "Shared sync enabled";

  return (
    <header
      className={cn(
        "relative shrink-0 border-b border-border px-3 py-3 sm:px-5",
        exportOpen ? "z-50" : "z-30",
      )}
    >
      <div className="flex flex-wrap items-center gap-3">
        <div className="flex shrink-0 flex-wrap items-center gap-x-2.5 gap-y-1">
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
