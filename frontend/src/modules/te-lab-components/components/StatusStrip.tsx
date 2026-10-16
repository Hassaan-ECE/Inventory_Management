import type { CatalogCounts } from "@/modules/te-lab-components/types";
import { CountPill, StatusFooter } from "@/platform/ui/StatusFooter";

interface StatusStripProps {
  counts?: Pick<CatalogCounts, "lowStock" | "noStock">;
  message?: string;
  resultsLabel?: string;
}

export function StatusStrip({ message, counts, resultsLabel }: StatusStripProps) {
  return (
    <StatusFooter message={message} resultsLabel={resultsLabel}>
      {counts ? (
        <div aria-label="Catalog status counts" className="flex flex-wrap">
          <CountPill label="Low stock" value={counts.lowStock} tone="warning" />
          <CountPill label="No stock" value={counts.noStock} tone="danger" />
        </div>
      ) : null}
    </StatusFooter>
  );
}
