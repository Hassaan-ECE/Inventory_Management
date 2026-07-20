import { CheckIcon, MenuIcon } from "lucide-react";
import { useId } from "react";

import { getModuleHost, listModuleDefinitions } from "@/platform/modules/registry";
import type { ModuleId } from "@/platform/modules/types";
import { DropdownItem, DropdownPanel } from "@/shared/components/ui/DropdownMenu";
import { useDropdownMenu } from "@/shared/hooks/useDropdownMenu";
import { cn } from "@/shared/lib/utils";

interface InventorySystemSwitcherProps {
  onChange: (id: ModuleId) => void;
  onOpenChange?: (open: boolean) => void;
  value: ModuleId;
}

export function InventorySystemSwitcher({ onChange, onOpenChange, value }: InventorySystemSwitcherProps) {
  const listboxId = useId();
  const { open, menuRef, toggle, close } = useDropdownMenu({ onOpenChange });
  const active = getModuleHost(value).definition;

  return (
    <div className="relative shrink-0" ref={menuRef}>
      <button
        aria-controls={open ? listboxId : undefined}
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label="Switch inventory system"
        className={cn(
          "group inline-flex items-center gap-2.5 rounded-lg text-left outline-none transition-colors",
          "hover:bg-accent/40 focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-1 focus-visible:ring-offset-background",
          "px-1.5 py-0.5 -mx-1.5",
        )}
        type="button"
        onClick={toggle}
      >
        <span
          aria-hidden
          className={cn(
            "inline-flex size-8 shrink-0 items-center justify-center rounded-md border border-border/70 bg-background text-muted-foreground",
            "transition-colors group-hover:border-border group-hover:text-foreground",
            open ? "border-border bg-accent/50 text-foreground" : null,
          )}
        >
          <MenuIcon className="size-4" strokeWidth={2.25} />
        </span>
        <h1 className="whitespace-nowrap text-2xl font-semibold tracking-tight text-foreground">{active.label}</h1>
      </button>

      {open ? (
        <DropdownPanel
          align="left"
          className="w-[min(16rem,calc(100vw-2rem))]"
          maxHeightClassName="max-h-[min(20rem,calc(100vh-6rem))]"
          role="listbox"
        >
          <div id={listboxId}>
            {listModuleDefinitions().map((system) => {
              const isActive = system.id === value;
              return (
                <DropdownItem
                  active={isActive}
                  aria-selected={isActive}
                  itemRole="option"
                  key={system.id}
                  onClick={() => {
                    onChange(system.id);
                    close();
                  }}
                >
                  <span className="min-w-0 flex-1 whitespace-nowrap font-medium">{system.label}</span>
                  {isActive ? <CheckIcon aria-hidden className="size-4 shrink-0 text-foreground" /> : null}
                </DropdownItem>
              );
            })}
          </div>
        </DropdownPanel>
      ) : null}
    </div>
  );
}
