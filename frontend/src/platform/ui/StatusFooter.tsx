import type { ReactNode } from "react";

import { APP_FOOTER_TAG, IS_DEV_BUILD } from "@/app/branding";
import { cn } from "@/shared/lib/utils";

interface StatusFooterProps {
  resultsLabel?: string;
  message?: string;
  /** Module-specific count pills, rendered after the message. */
  children?: ReactNode;
}

/** Footer shared by every inventory: results, status message, counts, and the build tag. */
export function StatusFooter({ resultsLabel, message, children }: StatusFooterProps) {
  return (
    <footer className="relative shrink-0 border-t border-border bg-card/80 px-3 text-xs text-muted-foreground sm:px-5">
      <div className="flex items-center gap-3 overflow-hidden py-0 pr-40">
        <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2">
          {resultsLabel ? <span className="shrink-0">{resultsLabel}</span> : null}
          {message ? <span className="min-w-0 truncate">{message}</span> : null}
          {children}
        </div>
      </div>

      <span
        className={cn(
          "pointer-events-none absolute inset-y-0 right-3 flex items-center font-medium sm:right-5",
          IS_DEV_BUILD && "font-semibold text-warning-foreground",
        )}
        title={IS_DEV_BUILD ? "Developer build: uses .tmp/dev-data and .tmp/dev-share, never live data" : undefined}
      >
        {APP_FOOTER_TAG}
      </span>
    </footer>
  );
}

type CountPillTone = "danger" | "warning" | "muted" | "info" | "success";

export function CountPill({ label, value, tone }: { label: string; value: number | string; tone: CountPillTone }) {
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
