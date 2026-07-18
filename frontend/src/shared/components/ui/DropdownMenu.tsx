import { ChevronDownIcon } from "lucide-react";
import { useId, type ButtonHTMLAttributes, type ReactNode } from "react";

import { ScrollRegion } from "@/shared/components/ui/ScrollRegion";
import { useDropdownMenu } from "@/shared/hooks/useDropdownMenu";
import { cn } from "@/shared/lib/utils";

export type DropdownAlign = "left" | "right";

interface DropdownPanelProps {
  align?: DropdownAlign;
  children: ReactNode;
  className?: string;
  /** When set, list body scrolls with modular ScrollRegion. */
  maxHeightClassName?: string;
  role?: string;
  title?: string;
}

/** Shared panel chrome matching the Columns picker. */
export function DropdownPanel({
  align = "right",
  children,
  className,
  maxHeightClassName = "max-h-[min(20rem,calc(100vh-8rem))]",
  role = "menu",
  title,
}: DropdownPanelProps) {
  return (
    <div
      className={cn(
        // Above sticky table headers so menus are not covered by the grid.
        // flex-col + max-height so ScrollRegion can constrain and cue-scroll.
        "absolute z-50 mt-2 flex min-w-[11rem] flex-col overflow-hidden rounded-2xl border border-border/70 bg-card p-2 text-card-foreground shadow-lg",
        maxHeightClassName,
        align === "right" ? "right-0" : "left-0",
        className,
      )}
      role={role}
    >
      {title ? (
        <div className="shrink-0 px-2 py-1">
          <p className="text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">{title}</p>
        </div>
      ) : null}
      <ScrollRegion className="mt-1 min-h-0 flex-1" contentClassName="space-y-1">
        {children}
      </ScrollRegion>
    </div>
  );
}

interface DropdownItemProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  active?: boolean;
  destructive?: boolean;
  itemRole?: string;
}

export function DropdownItem({
  active = false,
  className,
  destructive = false,
  itemRole = "menuitem",
  type = "button",
  ...props
}: DropdownItemProps) {
  return (
    <button
      className={cn(
        "flex w-full items-center gap-2 rounded-xl px-3 py-2 text-left text-sm transition-colors",
        destructive
          ? "text-destructive-foreground hover:bg-destructive/10"
          : "text-foreground hover:bg-accent/60",
        active && !destructive ? "bg-accent/70 font-medium" : null,
        props.disabled ? "cursor-not-allowed opacity-50 hover:bg-transparent" : null,
        className,
      )}
      role={itemRole}
      type={type}
      {...props}
    />
  );
}

export type DropdownOption = {
  disabled?: boolean;
  label: string;
  value: string;
};

interface DropdownSelectProps {
  align?: DropdownAlign;
  "aria-label"?: string;
  className?: string;
  disabled?: boolean;
  id?: string;
  onChange: (value: string) => void;
  options: readonly DropdownOption[];
  placeholder?: string;
  value: string;
}

/** Form-style select using the same dropdown panel as Columns / Export. */
export function DropdownSelect({
  align = "left",
  "aria-label": ariaLabel,
  className,
  disabled = false,
  id,
  onChange,
  options,
  placeholder = "Select…",
  value,
}: DropdownSelectProps) {
  const listboxId = useId();
  const { open, menuRef, toggle, close } = useDropdownMenu();
  const selected = options.find((option) => option.value === value);
  const label = selected?.label ?? placeholder;

  return (
    <div className={cn("relative w-full", className)} ref={menuRef}>
      <button
        aria-controls={open ? listboxId : undefined}
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label={ariaLabel}
        className={cn(
          "flex h-8 w-full items-center justify-between gap-2 rounded-md border border-input bg-background px-2.5 text-left text-xs text-foreground outline-none transition-shadow",
          "hover:bg-accent/30 focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/18",
          disabled ? "cursor-not-allowed opacity-60" : null,
          !selected ? "text-muted-foreground" : null,
        )}
        disabled={disabled}
        id={id}
        type="button"
        onClick={() => {
          if (!disabled) toggle();
        }}
      >
        <span className="min-w-0 truncate">{label}</span>
        <ChevronDownIcon className={cn("size-3.5 shrink-0 text-muted-foreground transition-transform", open ? "rotate-180" : null)} />
      </button>
      {open ? (
        <DropdownPanel
          align={align}
          className="w-full min-w-full"
          // Tall enough for the longest filter select (8 health options) without a scroll cue.
          maxHeightClassName="max-h-[min(24rem,calc(100vh-6rem))]"
          role="listbox"
        >
          <div id={listboxId}>
            {options.map((option) => (
              <DropdownItem
                active={option.value === value}
                aria-selected={option.value === value}
                disabled={option.disabled}
                itemRole="option"
                key={option.value}
                onClick={() => {
                  if (option.disabled) return;
                  onChange(option.value);
                  close();
                }}
              >
                {option.label}
              </DropdownItem>
            ))}
          </div>
        </DropdownPanel>
      ) : null}
    </div>
  );
}
