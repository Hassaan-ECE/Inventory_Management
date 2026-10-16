import type { InventoryCounts, TeTestEquipmentWorkspace } from "@/modules/te-test-equipment/types";
import { CountPill, StatusFooter } from "@/platform/ui/StatusFooter";

interface StatusStripProps {
  counts?: InventoryCounts;
  message?: string;
  resultsLabel?: string;
  workspace?: TeTestEquipmentWorkspace;
}

export function StatusStrip({ message, counts, resultsLabel, workspace = "equipment" }: StatusStripProps) {
  return (
    <StatusFooter message={message} resultsLabel={resultsLabel}>
      {counts ? (
        <div
          aria-label={workspace === "calibration" ? "Calibration status counts" : "Inventory status counts"}
          className="flex flex-wrap"
        >
          <CountPill label="Verified" value={`${counts.verified}/${counts.total}`} tone="success" />
          {workspace === "calibration" ? (
            <>
              <CountPill label="Overdue" value={counts.overdue} tone="danger" />
              <CountPill label="Due soon" value={counts.dueSoon} tone="warning" />
            </>
          ) : null}
        </div>
      ) : null}
    </StatusFooter>
  );
}
