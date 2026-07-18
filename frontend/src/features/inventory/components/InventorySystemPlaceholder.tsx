import type { InventorySystemDefinition } from "@/features/inventory/lib/inventorySystems";

interface InventorySystemPlaceholderProps {
  system: InventorySystemDefinition;
}

/** Shown when a system is selected but not yet ported into this unified shell. */
export function InventorySystemPlaceholder({ system }: InventorySystemPlaceholderProps) {
  return (
    <section className="flex h-full min-h-0 flex-1 items-center justify-center rounded-xl border border-border/70 bg-card/80 p-8 shadow-sm">
      <div className="max-w-md text-center">
        <p className="text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">
          Inventory system
        </p>
        <h2 className="mt-2 text-xl font-semibold tracking-tight text-foreground">{system.label}</h2>
        <p className="mt-3 text-sm leading-relaxed text-muted-foreground">
          Switching works. This module is not connected yet — {system.label} will plug in here later without changing
          how you switch systems.
        </p>
        <p className="mt-4 text-xs text-muted-foreground">
          Use the menu above to return to <span className="font-medium text-foreground">TE Test Equipment</span>.
        </p>
      </div>
    </section>
  );
}
