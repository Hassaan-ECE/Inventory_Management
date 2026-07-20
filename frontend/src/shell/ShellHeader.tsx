import { useState } from "react";
import { MoonIcon, SunIcon } from "lucide-react";

import type { ModuleId } from "@/platform/modules/types";
import type { ThemeMode } from "@/platform/ui/theme";
import { Button } from "@/shared/components/ui/button";
import { cn } from "@/shared/lib/utils";

import { InventorySystemSwitcher } from "./InventorySystemSwitcher";

interface ShellHeaderProps {
  activeModuleId: ModuleId;
  onModuleChange: (id: ModuleId) => void;
  onThemeToggle: () => void;
  theme: ThemeMode;
}

export function ShellHeader({ activeModuleId, onModuleChange, onThemeToggle, theme }: ShellHeaderProps) {
  const [systemMenuOpen, setSystemMenuOpen] = useState(false);

  return (
    <header
      className={cn(
        "relative shrink-0 border-b border-border px-3 py-3 sm:px-5",
        systemMenuOpen ? "z-50" : "z-40",
      )}
    >
      <div className="flex flex-wrap items-center gap-3">
        <InventorySystemSwitcher
          value={activeModuleId}
          onChange={onModuleChange}
          onOpenChange={setSystemMenuOpen}
        />
        <Button className="ml-auto" size="sm" variant="outline" onClick={onThemeToggle}>
          {theme === "light" ? <MoonIcon className="size-3.5" /> : <SunIcon className="size-3.5" />}
          {theme === "light" ? "Dark Theme" : "Light Theme"}
        </Button>
      </div>
    </header>
  );
}
