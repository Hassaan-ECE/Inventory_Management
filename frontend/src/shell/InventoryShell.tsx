import { useEffect, useState } from "react";

import { APP_VERSION } from "@/app/branding";
import { INVENTORY_MODULE_HOSTS } from "@/platform/modules/registry";
import { readTheme, THEME_STORAGE_KEY } from "@/platform/ui/theme";

import { InventorySystemPlaceholder } from "./InventorySystemPlaceholder";
import { ShellHeader } from "./ShellHeader";
import { ShellStatusStrip } from "./ShellStatusStrip";
import { useShellActiveModule } from "./useShellActiveModule";

export function InventoryShell() {
  const { activeHost, activeModuleId, selectModule } = useShellActiveModule();
  const [theme, setTheme] = useState(() => readTheme());

  useEffect(() => {
    document.documentElement.classList.toggle("dark", theme === "dark");
    window.localStorage.setItem(THEME_STORAGE_KEY, theme);
  }, [theme]);

  useEffect(() => {
    document.title = `Inventory Management — ${activeHost.definition.label} v${APP_VERSION}`;
  }, [activeHost.definition.label]);

  const onThemeToggle = (): void => {
    setTheme((current) => (current === "light" ? "dark" : "light"));
  };

  const placeholderMessage = `${activeHost.definition.label} — module not connected yet. Switch systems from the title menu.`;

  return (
    <div className="h-screen overflow-hidden bg-background text-foreground">
      <main className="flex h-full min-h-0 flex-col overflow-hidden">
        {/* Placeholder modules: shell-only chrome (switcher + theme). TE owns a unified top bar. */}
        {activeHost.kind === "placeholder" ? (
          <ShellHeader
            activeModuleId={activeModuleId}
            onModuleChange={selectModule}
            onThemeToggle={onThemeToggle}
            theme={theme}
          />
        ) : null}

        {INVENTORY_MODULE_HOSTS.map((host) => {
          if (host.kind !== "desktop") {
            return null;
          }
          const MainView = host.MainView;
          return (
            <MainView
              active={activeModuleId === host.definition.id}
              activeModuleId={activeModuleId}
              key={host.definition.id}
              onModuleChange={selectModule}
              onThemeToggle={onThemeToggle}
              theme={theme}
            />
          );
        })}

        {activeHost.kind === "placeholder" ? (
          <>
            <div className="flex min-h-0 flex-1 overflow-hidden px-2 py-1.5 sm:px-3">
              <InventorySystemPlaceholder system={activeHost.definition} />
            </div>
            <ShellStatusStrip message={placeholderMessage} resultsLabel={activeHost.definition.label} />
          </>
        ) : null}
      </main>
    </div>
  );
}
