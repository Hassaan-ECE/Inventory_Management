import { APP_CREDIT } from "@/app/branding";

interface ShellStatusStripProps {
  message: string;
  resultsLabel: string;
}

export function ShellStatusStrip({ message, resultsLabel }: ShellStatusStripProps) {
  return (
    <footer className="relative shrink-0 border-t border-border bg-card/80 px-3 text-xs text-muted-foreground sm:px-5">
      <div className="flex items-center gap-3 overflow-hidden py-0 pr-40">
        <div className="flex min-w-0 flex-1 flex-wrap items-center gap-2">
          <span className="shrink-0 text-muted-foreground">{resultsLabel}</span>
          <span className="min-w-0 truncate text-muted-foreground">{message}</span>
        </div>
      </div>
      <span className="pointer-events-none absolute inset-y-0 right-3 flex items-center font-medium text-muted-foreground sm:right-5">
        {APP_CREDIT}
      </span>
    </footer>
  );
}
