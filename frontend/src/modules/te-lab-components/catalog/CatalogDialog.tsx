import type { ReactNode } from "react";

import { cn } from "@/shared/lib/utils";

interface CatalogDialogProps {
  children: ReactNode;
  description?: string;
  footer?: ReactNode;
  onClose: () => void;
  title: string;
  wide?: boolean;
}

export function CatalogDialog({
  children,
  description,
  footer,
  onClose,
  title,
  wide = false,
}: CatalogDialogProps) {
  return (
    <div
      className="fixed inset-0 z-[100] flex items-center justify-center bg-black/45 p-3 backdrop-blur-[1px]"
      onMouseDown={(event) => {
        if (event.currentTarget === event.target) {
          onClose();
        }
      }}
    >
      <section
        aria-describedby={description ? "catalog-dialog-description" : undefined}
        aria-labelledby="catalog-dialog-title"
        aria-modal="true"
        className={cn(
          "flex max-h-[94vh] w-full flex-col overflow-hidden rounded-2xl border border-border bg-background shadow-2xl",
          wide ? "max-w-6xl" : "max-w-3xl",
        )}
        role="dialog"
      >
        <div className="shrink-0 border-b border-border px-5 py-4">
          <div className="flex items-start gap-4">
            <div className="min-w-0 flex-1">
              <h2 className="text-lg font-semibold" id="catalog-dialog-title">
                {title}
              </h2>
              {description ? (
                <p className="mt-1 text-sm text-muted-foreground" id="catalog-dialog-description">
                  {description}
                </p>
              ) : null}
            </div>
            <button
              aria-label="Close dialog"
              className="rounded-lg border border-border px-2.5 py-1 text-sm text-muted-foreground hover:bg-accent hover:text-foreground"
              onClick={onClose}
              type="button"
            >
              Close
            </button>
          </div>
        </div>
        <div className="min-h-0 flex-1 overflow-auto px-5 py-4">{children}</div>
        {footer ? <div className="shrink-0 border-t border-border px-5 py-3">{footer}</div> : null}
      </section>
    </div>
  );
}
