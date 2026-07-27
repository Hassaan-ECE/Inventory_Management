import { ChevronDownIcon } from "lucide-react";
import { useId, type ButtonHTMLAttributes, type CSSProperties, type ReactNode } from "react";

import { ScrollRegion } from "@/shared/components/ui/ScrollRegion";
import { useDropdownMenu } from "@/shared/hooks/useDropdownMenu";
import { cn } from "@/shared/lib/utils";

export type DropdownAlign = "left" | "right";

interface DropdownPanelProps {
  align?: DropdownAlign;
  children: ReactNode;
  className?: string;
  /**
   * Cap panel height. Prefer viewport-relative limits so short lists never scroll
   * when the window still has room.
   */
  maxHeightClassName?: string;
  /** Pixel max height (e.g. from floating placement). Overrides class when set. */
  maxHeightPx?: number;
  role?: string;
  style?: CSSProperties;
  title?: string;
}

/**
 * Shared panel chrome for menus / selects.
 *
 * Uses ScrollRegion (hidden native scrollbar + fade cues). ScrollRegion only
 * enables overflow when content exceeds the max height — short menus stay solid.
 */
export function DropdownPanel({
  align = "right",
  children,
  className,
  maxHeightClassName = "max-h-[min(48rem,calc(100dvh-2rem))]",
  maxHeightPx,
  role = "menu",
  style,
  title,
}: DropdownPanelProps) {
  return (
    <div
      className={cn(
        // Above sticky table headers so menus are not covered by the grid.
        "absolute z-50 mt-2 flex min-w-[11rem] flex-col overflow-hidden rounded-2xl border border-border/70 bg-card p-2 text-card-foreground shadow-lg",
        maxHeightPx == null ? maxHeightClassName : null,
        align === "right" ? "right-0" : "left-0",
        className,
      )}
      role={role}
      style={{
        ...style,
        ...(maxHeightPx != null ? { maxHeight: maxHeightPx } : null),
      }}
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

export type DropdownOptionTone = "success" | "warning" | "danger" | "muted" | "info";

export type DropdownOption = {
  disabled?: boolean;
  label: string;
  /** Soft status color for the option and the closed trigger when selected. */
  tone?: DropdownOptionTone;
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

const TONE_TRIGGER: Record<DropdownOptionTone, string> = {
  success: "border-success/30 bg-success/10 text-success-foreground hover:bg-success/15",
  warning: "border-warning/30 bg-warning/10 text-warning-foreground hover:bg-warning/15",
  danger: "border-destructive/30 bg-destructive/10 text-destructive-foreground hover:bg-destructive/15",
  muted: "border-border bg-muted/50 text-muted-foreground hover:bg-muted/70",
  info: "border-sky-500/30 bg-sky-500/10 text-sky-700 hover:bg-sky-500/15 dark:text-sky-300",
};

const TONE_OPTION: Record<DropdownOptionTone, string> = {
  success: "text-success-foreground hover:bg-success/12",
  warning: "text-warning-foreground hover:bg-warning/12",
  danger: "text-destructive-foreground hover:bg-destructive/12",
  muted: "text-muted-foreground hover:bg-muted/70",
  info: "text-sky-700 hover:bg-sky-500/12 dark:text-sky-300",
};

const TONE_OPTION_ACTIVE: Record<DropdownOptionTone, string> = {
  success: "bg-success/15 font-medium text-success-foreground",
  warning: "bg-warning/15 font-medium text-warning-foreground",
  danger: "bg-destructive/15 font-medium text-destructive-foreground",
  muted: "bg-muted/80 font-medium text-muted-foreground",
  info: "bg-sky-500/15 font-medium text-sky-700 dark:text-sky-300",
};

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
  const selectedTone = selected?.tone;

  return (
    <div className={cn("relative w-full", className)} ref={menuRef}>
      <button
        aria-controls={open ? listboxId : undefined}
        aria-expanded={open}
        aria-haspopup="listbox"
        aria-label={ariaLabel}
        className={cn(
          "flex h-8 w-full items-center justify-between gap-2 rounded-md border px-2.5 text-left text-xs outline-none transition-shadow",
          "focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/18",
          selectedTone
            ? TONE_TRIGGER[selectedTone]
            : "border-input bg-background text-foreground hover:bg-accent/30",
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
        <ChevronDownIcon
          className={cn(
            "size-3.5 shrink-0 transition-transform",
            selectedTone ? "opacity-70" : "text-muted-foreground",
            open ? "rotate-180" : null,
          )}
        />
      </button>
      {open ? (
        <DropdownPanel
          align={align}
          className="w-full min-w-full"
          maxHeightClassName="max-h-[min(40rem,calc(100dvh-5rem))]"
          role="listbox"
        >
          <div id={listboxId}>
            {options.map((option) => {
              const isActive = option.value === value;
              const tone = option.tone;
              return (
                <DropdownItem
                  active={isActive && !tone}
                  aria-selected={isActive}
                  className={cn(
                    tone ? (isActive ? TONE_OPTION_ACTIVE[tone] : TONE_OPTION[tone]) : null,
                  )}
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
              );
            })}
          </div>
        </DropdownPanel>
      ) : null}
    </div>
  );
}
