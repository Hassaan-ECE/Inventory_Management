import type { TeTestEquipmentWorkspace } from "@/modules/te-test-equipment/types";
import { cn } from "@/shared/lib/utils";

interface WorkspaceToggleProps {
  onWorkspaceChange: (workspace: TeTestEquipmentWorkspace) => void;
  workspace: TeTestEquipmentWorkspace;
}

export function WorkspaceToggle({ onWorkspaceChange, workspace }: WorkspaceToggleProps) {
  return (
    <div aria-label="TE Test Equipment workspace" className="inline-flex rounded-2xl border border-border/70 bg-card/80 p-1">
      <button
        aria-pressed={workspace === "equipment"}
        className={cn(
          "rounded-xl px-3 py-1.5 text-sm font-medium transition-colors",
          workspace === "equipment"
            ? "bg-primary/15 text-primary"
            : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
        )}
        type="button"
        onClick={() => onWorkspaceChange("equipment")}
      >
        Equipment
      </button>
      <button
        aria-pressed={workspace === "calibration"}
        className={cn(
          "rounded-xl px-3 py-1.5 text-sm font-medium transition-colors",
          workspace === "calibration"
            ? "bg-primary/15 text-primary"
            : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
        )}
        type="button"
        onClick={() => onWorkspaceChange("calibration")}
      >
        Calibration
      </button>
    </div>
  );
}
