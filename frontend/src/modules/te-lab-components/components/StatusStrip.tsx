import type { CatalogCounts } from "@/modules/te-lab-components/types";
import type { SyncWarning } from "@/platform/sync/sharedStatus";
import { CountPill, StatusFooter } from "@/platform/ui/StatusFooter";

interface StatusStripProps {
  counts?: Pick<CatalogCounts, "lowStock" | "noStock">;
  message?: string;
  resultsLabel?: string;
  warning?: SyncWarning | null;
}

export function StatusStrip({ message, counts, resultsLabel, warning }: StatusStripProps) {
  return (
    <StatusFooter message={message} resultsLabel={resultsLabel} warning={warning}>
      {counts ? (
        <div aria-label="Catalog status counts" className="flex flex-wrap">
          <CountPill label="Low stock" value={counts.lowStock} tone="warning" />
          <CountPill label="No stock" value={counts.noStock} tone="danger" />
        </div>
      ) : null}
    </StatusFooter>
  );
}
