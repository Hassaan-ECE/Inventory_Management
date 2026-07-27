import { useState } from "react";
import { MoonIcon, SunIcon } from "lucide-react";

import type { InventoryViewId } from "@/platform/modules/types";
import type { ThemeMode } from "@/platform/ui/theme";
import { Button } from "@/shared/components/ui/button";
import { cn } from "@/shared/lib/utils";

import { InventorySystemSwitcher } from "./InventorySystemSwitcher";

interface ShellHeaderProps {
  activeViewId: InventoryViewId;
  onThemeToggle: () => void;
  onViewChange: (id: InventoryViewId) => void;
  theme: ThemeMode;
}

export function ShellHeader({ activeViewId, onThemeToggle, onViewChange, theme }: ShellHeaderProps) {
  const [systemMenuOpen, setSystemMenuOpen] = useState(false);

  return (
    <header
      className={cn(
        "relative shrink-0 border-b border-border px-3 py-3 sm:px-5",
        systemMenuOpen ? "z-50" : "z-40",
      )}
    >
      {/* Single top bar for placeholder modules (TE renders its own unified bar). */}
      <div className="flex flex-wrap items-center gap-3">
        <InventorySystemSwitcher
          value={activeViewId}
          onChange={onViewChange}
          onOpenChange={setSystemMenuOpen}
        />
        <Button className="ml-auto shrink-0" size="sm" variant="outline" onClick={onThemeToggle}>
          {theme === "light" ? <MoonIcon className="size-3.5" /> : <SunIcon className="size-3.5" />}
          {theme === "light" ? "Dark Theme" : "Light Theme"}
        </Button>
      </div>
    </header>
  );
}
