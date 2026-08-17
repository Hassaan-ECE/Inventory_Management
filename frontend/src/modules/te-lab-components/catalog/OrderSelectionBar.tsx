import { Button } from "@/shared/components/ui/button";

interface OrderSelectionBarProps {
  onCancel: () => void;
  onContinue: () => void;
  selectedCount: number;
}

export function OrderSelectionBar({
  onCancel,
  onContinue,
  selectedCount,
}: OrderSelectionBarProps) {
  return (
    <div className="flex shrink-0 flex-wrap items-center gap-2 rounded-xl border border-border bg-card/90 px-3 py-2 shadow-sm">
      <span className="text-sm font-medium tabular-nums text-foreground">
        {selectedCount} selected
      </span>
      <div className="ml-auto flex items-center gap-2">
        <Button
          disabled={selectedCount === 0}
          size="sm"
          type="button"
          onClick={onContinue}
        >
          Continue
        </Button>
        <Button
          size="sm"
          type="button"
          variant="outline"
          onClick={onCancel}
        >
          Cancel
        </Button>
      </div>
    </div>
  );
}
