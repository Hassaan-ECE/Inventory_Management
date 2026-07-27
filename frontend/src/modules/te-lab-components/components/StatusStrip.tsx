import { APP_CREDIT } from "@/app/branding";
import type { CatalogCounts } from "@/modules/te-lab-components/types";
import { cn } from "@/shared/lib/utils";

interface StatusStripProps {
  counts?: Pick<CatalogCounts, "lowStock" | "noStock">;
  message?: string;
  resultsLabel?: string;
}

export function StatusStrip({ message, counts, resultsLabel }: StatusStripProps) {
  return (
    <footer className="relative shrink-0 border-t border-border bg-card/80 px-3 text-xs text-muted-foreground sm:px-5">
      <div className="flex items-center gap-3 overflow-hidden py-0 pr-40">
        <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2">
          {resultsLabel ? <span className="shrink-0 text-muted-foreground">{resultsLabel}</span> : null}
          {message ? <span className="min-w-0 truncate text-muted-foreground">{message}</span> : null}
          {counts ? (
            <div aria-label="Catalog status counts" className="flex flex-wrap">
              <CountPill label="Low stock" value={counts.lowStock} tone="warning" />
              <CountPill label="No stock" value={counts.noStock} tone="danger" />
            </div>
          ) : null}
        </div>
      </div>

      <span className="pointer-events-none absolute inset-y-0 right-3 flex items-center font-medium text-muted-foreground sm:right-5">
        {APP_CREDIT}
      </span>
    </footer>
  );
}

function CountPill({
  label,
  value,
  tone,
}: {
  label: string;
  value: number | string;
  tone: "danger" | "warning" | "muted" | "info" | "success";
}) {
  return (
    <span
      className={cn(
        "rounded-none border px-2 py-0.5 font-medium not-first:border-l-0",
        tone === "success" && "border-success/25 bg-success/10 text-success-foreground",
        tone === "danger" && "border-destructive/25 bg-destructive/10 text-destructive-foreground",
        tone === "warning" && "border-warning/25 bg-warning/10 text-warning-foreground",
        tone === "info" && "border-sky-500/25 bg-sky-500/10 text-sky-700 dark:text-sky-300",
        tone === "muted" && "border-border bg-muted/50 text-muted-foreground",
      )}
    >
      {label}: {value}
    </span>
  );
}
