import type { ReactNode } from "react";

/** Native `<select>` styled to match the shared `Input`. */
export const SELECT_CLASS =
  "h-9 w-full rounded-lg border border-input bg-background px-3 text-sm outline-none focus:border-ring focus:ring-[3px] focus:ring-ring/18 dark:bg-input/30";

export function Field({ children, className = "", label }: { children: ReactNode; className?: string; label: string }) {
  return (
    <label className={`space-y-1.5 text-sm ${className}`}>
      <span className="font-medium">{label}</span>
      {children}
    </label>
  );
}

export function FormSection({ children, description, title }: { children: ReactNode; description: string; title: string }) {
  return (
    <section className="rounded-xl border border-border bg-card/50 p-4">
      <div className="mb-3">
        <h3 className="font-semibold">{title}</h3>
        <p className="text-xs text-muted-foreground">{description}</p>
      </div>
      {children}
    </section>
  );
}

export function Alert({ children }: { children: ReactNode }) {
  return (
    <div className="rounded-xl border border-destructive/30 bg-destructive/8 px-3 py-2 text-sm text-destructive-foreground" role="alert">
      {children}
    </div>
  );
}
