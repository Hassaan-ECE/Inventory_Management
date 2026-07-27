import { cn } from "@/shared/lib/utils";

interface ColorRowsButtonProps {
  pressed: boolean;
  onPressedChange: (nextValue: boolean) => void;
}

/**
 * Toggle for status-colored table rows.
 * Gradient uses the same success / warning / destructive tokens as row tones
 * (bg-success/10, bg-warning/10, bg-destructive/10), a bit denser so the chip reads.
 */
export function ColorRowsButton({ pressed, onPressedChange }: ColorRowsButtonProps) {
  return (
    <button
      aria-label="Color rows"
      aria-pressed={pressed}
      className={cn(
        "relative size-9 shrink-0 overflow-hidden rounded-lg border transition-[opacity,filter,box-shadow]",
        pressed
          ? "border-foreground/25 shadow-sm"
          : "border-border/80 opacity-55 grayscale hover:opacity-80",
      )}
      title={pressed ? "Color rows on" : "Color rows off"}
      type="button"
      onClick={() => onPressedChange(!pressed)}
    >
      <span
        aria-hidden
        className="absolute inset-0 bg-[linear-gradient(to_bottom,color-mix(in_srgb,var(--success)_42%,var(--card))_0%,color-mix(in_srgb,var(--warning)_42%,var(--card))_50%,color-mix(in_srgb,var(--destructive)_42%,var(--card))_100%)]"
      />
    </button>
  );
}
