import { useState } from "react";
import { MapPinnedIcon, MoonIcon, PlusIcon, SunIcon, WifiIcon } from "lucide-react";

import { ExportMenu } from "@/modules/te-lab-components/components/header/ExportMenu";
import { UpdateActionButton } from "@/modules/te-lab-components/components/header/UpdateActionButton";
import type { CatalogCounts, CatalogScope, InventorySharedStatus, UpdateState } from "@/modules/te-lab-components/types";
import type { InventoryViewId } from "@/platform/modules/types";
import type { ThemeMode } from "@/platform/ui/theme";
import { Button } from "@/shared/components/ui/button";
import { cn } from "@/shared/lib/utils";
import { InventorySystemSwitcher } from "@/shell/InventorySystemSwitcher";

interface CatalogHeaderProps {
  activeViewId: InventoryViewId;
  canModify: boolean;
  counts: CatalogCounts;
  onAddPart: () => void;
  onExportExcel: () => void;
  onExportHtml: () => void;
  onManageLocations?: () => void;
  onOpenSharedCutover?: () => void;
  onScopeChange: (scope: CatalogScope) => void;
  onThemeToggle: () => void;
  onUpdateAction: () => void;
  onViewChange: (id: InventoryViewId) => void;
  scope: CatalogScope;
  shared: InventorySharedStatus;
  theme: ThemeMode;
  updateState: UpdateState;
}

export function CatalogHeader({
  activeViewId,
  canModify,
  counts,
  onAddPart,
  onExportExcel,
  onExportHtml,
  onManageLocations,
  onOpenSharedCutover,
  onScopeChange,
  onThemeToggle,
  onUpdateAction,
  onViewChange,
  scope,
  shared,
  theme,
  updateState,
}: CatalogHeaderProps) {
  const [exportOpen, setExportOpen] = useState(false);
  const [systemMenuOpen, setSystemMenuOpen] = useState(false);
  const sharedReady = shared.enabled && shared.available;
  const isLocalOnly = !shared.enabled;
  const themeLabel = theme === "light" ? "Dark Theme" : "Light Theme";
  const modeTitle =
    shared.message?.trim() ||
    (sharedReady
      ? "Shared catalog synchronization is ready."
      : isLocalOnly
        ? "Shared sync is off for this session. Changes stay on this computer; sync is not a backup."
        : "Changes stay local.");

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
              sharedReady
                ? "shrink-0 rounded-full border border-emerald-500/25 bg-emerald-500/10 px-2.5 py-0.5 text-[11px] font-semibold tracking-wide text-emerald-700 dark:text-emerald-300"
                : "shrink-0 rounded-full border border-border bg-muted/60 px-2.5 py-0.5 text-[11px] font-semibold tracking-wide text-muted-foreground"
            }
            title={modeTitle}
          >
            {sharedReady ? "Shared" : "Local"}
          </span>
        </div>

        <div className="ml-auto flex min-w-0 flex-wrap items-center justify-end gap-2">
          <UpdateActionButton state={updateState} onClick={onUpdateAction} />
          <div className="inline-flex rounded-2xl border border-border/70 bg-card/80 p-1" aria-label="Catalog scope">
            <button
              aria-pressed={scope === "inventory"}
              className={cn(
                "rounded-xl px-3 py-1.5 text-sm font-medium transition-colors",
                scope === "inventory"
                  ? "bg-success/15 text-success-foreground"
                  : "text-success-foreground/80 hover:bg-success/10 hover:text-success-foreground",
              )}
              type="button"
              onClick={() => onScopeChange("inventory")}
            >
              Inventory ({counts.activeParts})
            </button>
            <button
              aria-pressed={scope === "archive"}
              className={cn(
                "rounded-xl px-3 py-1.5 text-sm font-medium transition-colors",
                scope === "archive"
                  ? "bg-warning/15 text-warning-foreground"
                  : "text-warning-foreground/80 hover:bg-warning/10 hover:text-warning-foreground",
              )}
              type="button"
              onClick={() => onScopeChange("archive")}
            >
              Archive ({counts.archivedParts})
            </button>
          </div>
          {onOpenSharedCutover ? (
            <Button
              aria-label="Set Up Lab Shared Catalog"
              className="size-8"
              size="icon"
              title="Set Up Lab Shared Catalog"
              variant="outline"
              onClick={onOpenSharedCutover}
            >
              <WifiIcon className="size-3.5" />
            </Button>
          ) : null}
          {onManageLocations ? (
            <Button
              aria-label="Manage Lab Storage Locations"
              className="size-8"
              size="icon"
              title="Manage Lab Storage Locations"
              variant="outline"
              onClick={onManageLocations}
            >
              <MapPinnedIcon className="size-3.5" />
            </Button>
          ) : null}
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
          <ExportMenu
            onExportExcel={onExportExcel}
            onExportHtml={onExportHtml}
            onOpenChange={setExportOpen}
          />
          <Button
            aria-label="Add Part"
            className="size-8"
            disabled={!canModify}
            size="icon"
            title="Add Part"
            onClick={onAddPart}
          >
            <PlusIcon className="size-3.5" />
          </Button>
        </div>
      </div>
    </header>
  );
}
