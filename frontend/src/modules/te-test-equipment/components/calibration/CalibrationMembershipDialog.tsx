import { useMemo, useState } from "react";

import type {
  CalibrationRequirement,
  InventoryEntry,
} from "@/modules/te-test-equipment/types";
import { Button } from "@/shared/components/ui/button";
import { DropdownSelect } from "@/shared/components/ui/DropdownMenu";
import { Input } from "@/shared/components/ui/input";
import { ScrollRegion } from "@/shared/components/ui/ScrollRegion";

interface CalibrationMembershipDialogProps {
  candidates: InventoryEntry[];
  entry?: InventoryEntry | null;
  mode: "add" | "remove";
  onAdd: (entryIds: string[]) => Promise<boolean> | boolean;
  onClose: () => void;
  onCreateNew: () => void;
  onRemove: (requirement: Exclude<CalibrationRequirement, "required">) => Promise<boolean> | boolean;
  readOnly?: boolean;
}

const REMOVAL_OPTIONS = [
  { value: "unknown", label: "Unknown — calibration need is not confirmed" },
  { value: "reference_only", label: "Reference only — keep visible when requested" },
  { value: "not_required", label: "Not required — calibration is not needed" },
] as const;

export function CalibrationMembershipDialog({
  candidates,
  entry,
  mode,
  onAdd,
  onClose,
  onCreateNew,
  onRemove,
  readOnly = false,
}: CalibrationMembershipDialogProps) {
  const [query, setQuery] = useState("");
  const [selectedEntryIds, setSelectedEntryIds] = useState<string[]>([]);
  const [removalRequirement, setRemovalRequirement] = useState<Exclude<CalibrationRequirement, "required">>("unknown");
  const [isSaving, setIsSaving] = useState(false);
  const normalizedQuery = query.trim().toLowerCase();
  const filteredCandidates = useMemo(
    () => candidates.filter((candidate) => {
      if (!normalizedQuery) {
        return true;
      }
      return [
        candidate.assetNumber,
        candidate.serialNumber,
        candidate.manufacturer,
        candidate.model,
        candidate.description,
        candidate.location,
        candidate.assignedTo,
      ].some((value) => value.toLowerCase().includes(normalizedQuery));
    }),
    [candidates, normalizedQuery],
  );

  async function handleAdd(): Promise<void> {
    setIsSaving(true);
    const updated = await onAdd(selectedEntryIds);
    setIsSaving(false);
    if (updated) {
      onClose();
    }
  }

  async function handleRemove(): Promise<void> {
    setIsSaving(true);
    const updated = await onRemove(removalRequirement);
    setIsSaving(false);
    if (updated) {
      onClose();
    }
  }

  return (
    <div
      aria-modal="true"
      className="fixed inset-0 z-40 flex items-center justify-center bg-black/45 p-4 backdrop-blur-[2px]"
      role="dialog"
      onClick={(event) => {
        if (event.target === event.currentTarget && !isSaving) {
          onClose();
        }
      }}
    >
      <section className="flex max-h-[88vh] w-full max-w-2xl flex-col overflow-hidden rounded-[1.5rem] border border-border/70 bg-card text-card-foreground shadow-2xl">
        <header className="shrink-0 border-b border-border/70 px-5 py-4">
          <p className="text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">Calibration Membership</p>
          <h2 className="mt-1 text-xl font-semibold tracking-tight text-foreground">
            {mode === "add" ? "Add Equipment to Calibration" : "Remove Equipment from Calibration"}
          </h2>
        </header>

        {mode === "add" ? (
          <div className="flex min-h-0 flex-1 flex-col gap-3 px-5 py-4">
            <Input
              aria-label="Search equipment to add"
              placeholder="Search by asset, serial, maker, model, description, location, or assignment"
              value={query}
              onChange={(event) => setQuery(event.currentTarget.value)}
            />
            <ScrollRegion className="min-h-0 flex-1 rounded-xl border border-border/70" contentClassName="divide-y divide-border/60">
              {filteredCandidates.length > 0 ? filteredCandidates.map((candidate) => {
                const checked = selectedEntryIds.includes(candidate.id);
                const identity = [candidate.assetNumber, candidate.serialNumber].filter(Boolean).join(" / ") || "No asset or serial";
                const model = [candidate.manufacturer, candidate.model].filter(Boolean).join(" ");
                return (
                  <label className="flex cursor-pointer items-start gap-3 px-4 py-3 hover:bg-accent/40" key={candidate.id}>
                    <input
                      aria-label={`Select ${candidate.description || identity}`}
                      checked={checked}
                      className="mt-1 size-4 accent-[var(--primary)]"
                      disabled={readOnly || isSaving}
                      type="checkbox"
                      onChange={() => {
                        setSelectedEntryIds((current) => checked
                          ? current.filter((entryId) => entryId !== candidate.id)
                          : [...current, candidate.id]);
                      }}
                    />
                    <span className="min-w-0">
                      <span className="block font-medium text-foreground">{candidate.description || model || identity}</span>
                      <span className="mt-0.5 block text-xs text-muted-foreground">
                        {[identity, model, candidate.location].filter(Boolean).join(" • ")}
                      </span>
                    </span>
                  </label>
                );
              }) : (
                <p className="px-4 py-8 text-center text-sm text-muted-foreground">No eligible equipment matches this search.</p>
              )}
            </ScrollRegion>
          </div>
        ) : (
          <div className="space-y-4 px-5 py-5">
            <div className="rounded-xl border border-border/70 bg-background/60 px-4 py-3">
              <p className="font-medium text-foreground">{entry?.description || entry?.model || entry?.assetNumber || "Selected equipment"}</p>
              <p className="mt-1 text-sm text-muted-foreground">
                {[entry?.assetNumber, entry?.serialNumber, entry?.manufacturer, entry?.model].filter(Boolean).join(" • ")}
              </p>
            </div>
            <label className="block">
              <span className="mb-1.5 block text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">
                Requirement after removal
              </span>
              <DropdownSelect
                aria-label="Requirement after removal"
                options={REMOVAL_OPTIONS}
                value={removalRequirement}
                onChange={(value) => setRemovalRequirement(value as Exclude<CalibrationRequirement, "required">)}
              />
            </label>
            <p className="text-sm text-muted-foreground">
              Existing dates, vendor, certificate, notes, and verification details remain on the equipment record.
            </p>
          </div>
        )}

        <footer className="flex shrink-0 flex-wrap items-center justify-between gap-3 border-t border-border/70 px-5 py-4">
          {mode === "add" ? (
            <Button disabled={readOnly || isSaving} type="button" variant="outline" onClick={onCreateNew}>
              Create New Equipment
            </Button>
          ) : <span />}
          <div className="ml-auto flex items-center gap-2">
            <Button disabled={isSaving} type="button" variant="ghost" onClick={onClose}>Cancel</Button>
            <Button
              disabled={readOnly || isSaving || (mode === "add" && selectedEntryIds.length === 0)}
              type="button"
              onClick={() => {
                void (mode === "add" ? handleAdd() : handleRemove());
              }}
            >
              {isSaving
                ? "Saving..."
                : mode === "add"
                  ? `Add Selected${selectedEntryIds.length > 0 ? ` (${selectedEntryIds.length})` : ""}`
                  : "Remove from Calibration"}
            </Button>
          </div>
        </footer>
      </section>
    </div>
  );
}
