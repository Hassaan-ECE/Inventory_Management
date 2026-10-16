import { useEffect, useState } from "react";

import { APP_VERSION } from "@/app/branding";
import { INVENTORY_MODULE_HOSTS } from "@/platform/modules/registry";
import { readTheme, THEME_STORAGE_KEY } from "@/platform/ui/theme";

import { useShellActiveModule } from "./useShellActiveModule";

/** Mounts every inventory module; each renders its own header and footer and stays idle while inactive. */
export function InventoryShell() {
  const { activeModuleId, activeView, activeViewId, selectView } = useShellActiveModule();
  const [theme, setTheme] = useState(() => readTheme());

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
    window.localStorage.setItem(THEME_STORAGE_KEY, theme);
  }, [theme]);

  useEffect(() => {
    document.title = `Inventory Management — ${activeView.label} v${APP_VERSION}`;
  }, [activeView.label]);

  const onThemeToggle = (): void => {
    setTheme((current) => (current === "light" ? "dark" : "light"));
  };

  return (
    <div className="h-screen overflow-hidden bg-background text-foreground">
      <main className="flex h-full min-h-0 flex-col overflow-hidden">
        {INVENTORY_MODULE_HOSTS.map(({ definition, MainView }) => (
          <MainView
            active={activeModuleId === definition.id}
            activeViewId={activeViewId}
            key={definition.id}
            onThemeToggle={onThemeToggle}
            onViewChange={selectView}
            theme={theme}
          />
        ))}
      </main>
    </div>
  );
}
