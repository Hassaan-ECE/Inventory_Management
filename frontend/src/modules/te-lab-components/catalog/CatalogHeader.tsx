import { BoxesIcon, DownloadIcon, MapPinnedIcon, MoonIcon, PlusIcon, SunIcon, WifiIcon } from "lucide-react";

import type { CatalogCounts, CatalogScope, InventorySharedStatus } from "@/modules/te-lab-components/types";
import type { ModuleId } from "@/platform/modules/types";
import type { ThemeMode } from "@/platform/ui/theme";
import { Button } from "@/shared/components/ui/button";
import { InventorySystemSwitcher } from "@/shell/InventorySystemSwitcher";

interface CatalogHeaderProps {
  activeModuleId: ModuleId;
  canModify: boolean;
  counts: CatalogCounts;
  onAddPart: () => void;
  onExport: () => void;
  onLocations: () => void;
  onModuleChange: (id: ModuleId) => void;
  onScopeChange: (scope: CatalogScope) => void;
  onSharedSetup: () => void;
  onThemeToggle: () => void;
  scope: CatalogScope;
  shared: InventorySharedStatus;
  theme: ThemeMode;
}

export function CatalogHeader({
  activeModuleId,
  canModify,
  counts,
  onAddPart,
  onExport,
  onLocations,
  onModuleChange,
  onScopeChange,
  onSharedSetup,
  onThemeToggle,
  scope,
  shared,
  theme,
}: CatalogHeaderProps) {
  const sharedReady = shared.enabled && shared.available;
  const modeTitle = shared.message || (sharedReady ? "Shared catalog synchronization is ready." : "Changes stay local.");

  return (
    <header className="relative z-30 shrink-0 border-b border-border px-3 py-3 sm:px-5">
      <div className="flex flex-wrap items-center gap-3">
        <div className="flex min-w-0 flex-wrap items-center gap-2.5">
          <InventorySystemSwitcher value={activeModuleId} onChange={onModuleChange} />
          <span
            className={
              sharedReady
                ? "rounded-full border border-emerald-500/25 bg-emerald-500/10 px-2.5 py-0.5 text-[11px] font-semibold tracking-wide text-emerald-700 dark:text-emerald-300"
                : "rounded-full border border-border bg-muted/60 px-2.5 py-0.5 text-[11px] font-semibold tracking-wide text-muted-foreground"
            }
            title={modeTitle}
          >
            {sharedReady ? "Shared" : "Local"}
          </span>
          <span className="hidden items-center gap-1 text-xs text-muted-foreground lg:flex">
            <BoxesIcon className="size-3.5" />
            {counts.totalParts.toLocaleString()} catalog parts
          </span>
        </div>

        <div className="ml-auto flex flex-wrap items-center justify-end gap-2">
          <div className="inline-flex rounded-lg border border-border bg-muted/30 p-0.5" aria-label="Catalog scope">
            <button
              aria-pressed={scope === "inventory"}
              className={
                scope === "inventory"
                  ? "rounded-md bg-background px-2.5 py-1.5 text-xs font-medium shadow-sm"
                  : "rounded-md px-2.5 py-1.5 text-xs text-muted-foreground hover:text-foreground"
              }
              onClick={() => onScopeChange("inventory")}
              type="button"
            >
              Inventory {counts.activeParts}
            </button>
            <button
              aria-pressed={scope === "archive"}
              className={
                scope === "archive"
                  ? "rounded-md bg-background px-2.5 py-1.5 text-xs font-medium shadow-sm"
                  : "rounded-md px-2.5 py-1.5 text-xs text-muted-foreground hover:text-foreground"
              }
              onClick={() => onScopeChange("archive")}
              type="button"
            >
              Archive {counts.archivedParts}
            </button>
          </div>
          {!sharedReady && shared.enabled ? (
            <Button onClick={onSharedSetup} size="sm" variant="outline">
              <WifiIcon className="size-3.5" />
              Shared Setup
            </Button>
          ) : null}
          <Button onClick={onLocations} size="sm" variant="outline">
            <MapPinnedIcon className="size-3.5" />
            Locations
          </Button>
          <Button onClick={onExport} size="sm" variant="outline">
            <DownloadIcon className="size-3.5" />
            Export
          </Button>
          <Button aria-label="Toggle theme" onClick={onThemeToggle} size="sm" variant="outline">
            {theme === "light" ? <MoonIcon className="size-3.5" /> : <SunIcon className="size-3.5" />}
            <span className="hidden sm:inline">{theme === "light" ? "Dark" : "Light"}</span>
          </Button>
          <Button disabled={!canModify} onClick={onAddPart} size="sm">
            <PlusIcon className="size-3.5" />
            Add Part
          </Button>
        </div>
      </div>
    </header>
  );
}
