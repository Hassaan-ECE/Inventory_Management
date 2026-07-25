import { Button } from "@/shared/components/ui/button";
import { Empty, EmptyDescription, EmptyHeader, EmptyTitle } from "@/shared/components/ui/empty";
import type { InventoryScope, TeTestEquipmentWorkspace } from "@/modules/te-test-equipment/types";

interface EmptyResultsProps {
  onAddEntry: () => void;
  query: string;
  scope: InventoryScope;
  workspace?: TeTestEquipmentWorkspace;
}

export function EmptyResults({ onAddEntry, query, scope, workspace = "equipment" }: EmptyResultsProps) {
  if (scope === "archive") {
    return (
      <section className="flex h-full min-h-0 flex-1 rounded-xl border border-border/70 bg-card/80 shadow-sm">
        <Empty>
          <EmptyHeader>
            <EmptyTitle>
              {workspace === "calibration"
                ? query.trim() ? "No archived calibration matches" : "No archived calibration equipment yet"
                : query.trim() ? "No archived matches" : "No archived entries yet"}
            </EmptyTitle>
            <EmptyDescription>
              {workspace === "calibration"
                ? query.trim()
                  ? "The current calibration search and filters did not match archived equipment."
                  : "Archived calibration equipment will appear here when tracked equipment is archived."
                : query.trim()
                  ? "The current archive filters did not match any archived entries."
                  : "Archived entries will appear here when entries are restored or moved out of inventory."}
            </EmptyDescription>
          </EmptyHeader>
        </Empty>
      </section>
    );
  }

  return (
    <section className="flex h-full min-h-0 flex-1 rounded-xl border border-border/70 bg-card/80 shadow-sm">
      <Empty>
        <EmptyHeader>
          <EmptyTitle>
            {workspace === "calibration" ? "No calibration equipment matches this view" : "Can&apos;t find what you&apos;re looking for?"}
          </EmptyTitle>
          <EmptyDescription>
            {workspace === "calibration"
              ? "Try a broader search, clear the calibration filters, or add existing equipment to calibration."
              : "Try a broader search, clear the column filters, or add a new entry."}
          </EmptyDescription>
        </EmptyHeader>
        <Button size="sm" onClick={onAddEntry}>
          {workspace === "calibration" ? "Add Equipment" : "Add Entry"}
        </Button>
      </Empty>
    </section>
  );
}
