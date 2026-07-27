import { useCallback, useEffect, useId, useMemo, useState } from "react";
import type { ReactNode } from "react";

import { Badge } from "@/shared/components/ui/badge";
import { DropdownSelect } from "@/shared/components/ui/DropdownMenu";
import { Input } from "@/shared/components/ui/input";
import { ScrollRegion } from "@/shared/components/ui/ScrollRegion";
import { Textarea } from "@/shared/components/ui/textarea";
import { Button } from "@/shared/components/ui/button";
import { UnsavedChangesDialog } from "@/shared/components/ui/UnsavedChangesDialog";
import { cn } from "@/shared/lib/utils";
import type {
  InventoryEntry,
  InventoryEntryEditContext,
  InventoryEntryInput,
  CalibrationRequirement,
  LifecycleStatus,
  TeTestEquipmentWorkspace,
  WorkingStatus,
} from "@/modules/te-test-equipment/types";

import { ContextRow, DialogActions } from "./entry-dialog/components";
import {
  ENTRY_CALIBRATION_BOOLEAN_FIELDS,
  ENTRY_CALIBRATION_SELECT_FIELDS,
  ENTRY_CONDITION_FIELD,
  ENTRY_EQUIPMENT_SELECT_FIELDS,
  ENTRY_MAIN_INPUT_FIELDS,
  buildEntryContextRows,
  type EntrySelectField,
} from "./entry-dialog/fieldMetadata";
import {
  buildFormState,
  formatOptionLabel,
  statusOptionTone,
  type EntryFormState,
  suggestCalibrationDueDate,
  updateForm,
} from "./entry-dialog/form";
import { PicturePreviewPanel } from "./entry-dialog/PicturePreviewPanel";
import { useEntryDialogLayout } from "./entry-dialog/useEntryDialogLayout";
import { useEntryDialogSubmit } from "./entry-dialog/useEntryDialogSubmit";
import { useEntryPicturePreview } from "./entry-dialog/useEntryPicturePreview";
import { useMountedRef } from "./entry-dialog/useMountedRef";

interface EntryDialogProps {
  defaultArchived?: boolean;
  defaultCalibrationRequirement?: CalibrationRequirement;
  defaultSection?: TeTestEquipmentWorkspace;
  mode: "add" | "edit";
  onClose: () => void;
  /** Edit mode only — opens the existing delete confirmation flow. */
  onDelete?: () => void;
  onSave: (input: InventoryEntryInput, editContext?: InventoryEntryEditContext) => Promise<void> | void;
  readOnly?: boolean;
  entry?: InventoryEntry | null;
}

export function EntryDialog({
  defaultArchived = false,
  defaultCalibrationRequirement = "unknown",
  defaultSection = "equipment",
  mode,
  onClose,
  onDelete,
  onSave,
  readOnly = false,
  entry,
}: EntryDialogProps) {
  const isMountedRef = useMountedRef();
  const [initialForm] = useState<EntryFormState>(() => (
    buildFormState(entry, defaultArchived, defaultCalibrationRequirement)
  ));
  const [form, setForm] = useState<EntryFormState>(initialForm);
  const [section, setSection] = useState<TeTestEquipmentWorkspace>(() =>
    form.calibrationRequirement === "required" ? defaultSection : "equipment",
  );
  const [showAddToCalibrationPrompt, setShowAddToCalibrationPrompt] = useState(false);
  const [showUnsavedPrompt, setShowUnsavedPrompt] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const formId = useId();
  const calibrationIntervalId = useId();
  const picturePath = form.picturePath.trim();
  const onCalibrationRoster = form.calibrationRequirement === "required";
  const { handleSubmit, isSaving, saveEntry } = useEntryDialogSubmit({
    entry,
    form,
    initialForm,
    isMountedRef,
    mode,
    onSave,
    readOnly,
    setError,
  });
  const picturePreview = useEntryPicturePreview({
    isMountedRef,
    onPicturePathChange: (selectedPath) => updateForm(setForm, "picturePath", selectedPath),
    picturePath,
    setError,
  });
  const { showInlinePicturePreview } = useEntryDialogLayout({
    entry,
    mode,
    picturePath,
    readOnly,
  });

  const isDirty = useMemo(() => !entryFormEquals(form, initialForm), [form, initialForm]);

  const requestClose = useCallback((): void => {
    if (isSaving) {
      return;
    }
    if (showAddToCalibrationPrompt) {
      setShowAddToCalibrationPrompt(false);
      return;
    }
    if (isDirty && !readOnly) {
      setShowUnsavedPrompt(true);
      return;
    }
    onClose();
  }, [isDirty, isSaving, onClose, readOnly, showAddToCalibrationPrompt]);

  const handleSaveAndClose = useCallback(async (): Promise<void> => {
    const saved = await saveEntry();
    if (saved) {
      setShowUnsavedPrompt(false);
      onClose();
    }
  }, [onClose, saveEntry]);

  useEffect(() => {
    function handleKeyDown(event: KeyboardEvent): void {
      if (event.key !== "Escape" || isSaving) {
        return;
      }
      if (showUnsavedPrompt) {
        setShowUnsavedPrompt(false);
        return;
      }
      if (showAddToCalibrationPrompt) {
        setShowAddToCalibrationPrompt(false);
        return;
      }
      requestClose();
    }

    document.addEventListener("keydown", handleKeyDown);
    return () => document.removeEventListener("keydown", handleKeyDown);
  }, [isSaving, requestClose, showAddToCalibrationPrompt, showUnsavedPrompt]);

  // Keep section in sync if requirement changes away from required (e.g. form edits).
  // Adjust during render when roster membership flips — avoids setState-in-effect lint.
  if (!onCalibrationRoster && section === "calibration") {
    setSection("equipment");
  }

  function handleSectionSelect(nextSection: TeTestEquipmentWorkspace): void {
    if (nextSection === "equipment") {
      setSection("equipment");
      setShowAddToCalibrationPrompt(false);
      return;
    }
    if (onCalibrationRoster) {
      setSection("calibration");
      return;
    }
    if (readOnly) {
      setError("This equipment is not on the Calibration roster. You cannot edit membership while read-only.");
      return;
    }
    setShowAddToCalibrationPrompt(true);
  }

  function confirmAddToCalibration(): void {
    updateForm(setForm, "calibrationRequirement", "required");
    setSection("calibration");
    setShowAddToCalibrationPrompt(false);
  }

  function handleSelectChange(field: EntrySelectField, value: string): void {
    if (field.key === "lifecycleStatus") {
      updateForm(setForm, field.key, value as LifecycleStatus);
      return;
    }
    if (field.key === "calibrationRequirement") {
      updateForm(setForm, field.key, value as CalibrationRequirement);
      return;
    }

    updateForm(setForm, field.key, value as WorkingStatus);
  }

  return (
    <div
      aria-modal="true"
      className="fixed inset-0 z-40 flex items-center justify-center bg-black/45 p-4 backdrop-blur-[2px]"
      role="dialog"
      onClick={(event) => {
        if (event.target === event.currentTarget && !isSaving && !showUnsavedPrompt) {
          requestClose();
        }
      }}
    >
      <div className="flex max-h-[92vh] w-full max-w-[72rem] overflow-hidden rounded-[1.75rem] border border-border/70 bg-card text-card-foreground shadow-2xl lg:max-h-[94vh]">
        <form
          className="flex min-w-0 flex-1 flex-col overflow-hidden"
          id={formId}
          onSubmit={handleSubmit}
        >
          <div className="shrink-0 border-b border-border/70 px-5 py-4 lg:py-3.5">
            <div className="grid grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)] items-center gap-3">
              <div className="min-w-0">
                <p className="text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">
                  {mode === "edit" ? "Open Full Entry" : "Add Entry"}
                </p>
                <h2 className="text-xl font-semibold tracking-tight text-foreground">
                  {mode === "edit" ? "Edit Entry" : "Add Entry"}
                </h2>
              </div>

              <div
                aria-label="Entry editor section"
                className="inline-flex justify-self-center rounded-2xl border border-border/70 bg-background/60 p-1"
              >
                <button
                  aria-pressed={section === "equipment"}
                  className={cn(
                    "rounded-xl px-3 py-1.5 text-sm font-medium transition-colors",
                    section === "equipment"
                      ? "bg-primary/15 text-primary"
                      : "text-muted-foreground hover:bg-accent/60 hover:text-foreground",
                  )}
                  type="button"
                  onClick={() => handleSectionSelect("equipment")}
                >
                  Equipment
                </button>
                <button
                  aria-pressed={section === "calibration"}
                  aria-disabled={!onCalibrationRoster}
                  className={cn(
                    "rounded-xl px-3 py-1.5 text-sm font-medium transition-colors",
                    section === "calibration" && onCalibrationRoster
                      ? "bg-primary/15 text-primary"
                      : onCalibrationRoster
                        ? "text-muted-foreground hover:bg-accent/60 hover:text-foreground"
                        : "cursor-pointer text-muted-foreground/55 opacity-60",
                  )}
                  title={
                    onCalibrationRoster
                      ? "Calibration fields"
                      : "Not on Calibration roster — click to add"
                  }
                  type="button"
                  onClick={() => handleSectionSelect("calibration")}
                >
                  Calibration
                </button>
              </div>

              <div className="flex flex-wrap items-center justify-end gap-2">
                <Badge variant={form.archived ? "warning" : "secondary"}>
                  {form.archived ? "Archive" : "Inventory"}
                </Badge>
                <Badge variant={form.verifiedAt ? "success" : "outline"}>
                  {form.verifiedAt ? "Verified" : "Pending"}
                </Badge>
              </div>
            </div>
          </div>

          {showAddToCalibrationPrompt ? (
            <div
              className="fixed inset-0 z-50 flex items-center justify-center bg-black/40 p-4"
              role="alertdialog"
              aria-labelledby="add-to-calibration-title"
              aria-describedby="add-to-calibration-desc"
              onClick={(event) => {
                if (event.target === event.currentTarget) {
                  setShowAddToCalibrationPrompt(false);
                }
              }}
            >
              <div className="w-full max-w-md rounded-2xl border border-border/70 bg-card p-5 shadow-2xl">
                <h3 id="add-to-calibration-title" className="text-lg font-semibold text-foreground">
                  Add to Calibration?
                </h3>
                <p id="add-to-calibration-desc" className="mt-2 text-sm text-muted-foreground">
                  This equipment is not on the Calibration roster. Add it so you can edit calibration
                  fields? That sets Calibration requirement to <strong className="text-foreground">Required</strong>.
                  Save the entry to keep the change.
                </p>
                <div className="mt-5 flex justify-end gap-2">
                  <Button type="button" variant="outline" onClick={() => setShowAddToCalibrationPrompt(false)}>
                    Cancel
                  </Button>
                  <Button type="button" onClick={confirmAddToCalibration}>
                    Add to Calibration
                  </Button>
                </div>
              </div>
            </div>
          ) : null}

          <fieldset className="contents" disabled={readOnly || isSaving}>
            <ScrollRegion className="min-h-0 flex-1" contentClassName="px-5 py-4 lg:py-4">
              {section === "equipment" ? (
                <>
                  <div className="grid gap-4 lg:grid-cols-2 lg:gap-5">
                    {ENTRY_MAIN_INPUT_FIELDS.map((field) => (
                      <Field className={field.className} key={field.key} label={field.label}>
                        <Input
                          autoFocus={field.autoFocus}
                          inputMode={field.inputMode}
                          placeholder={field.placeholder}
                          value={form[field.key]}
                          onChange={(event) => updateForm(setForm, field.key, event.currentTarget.value)}
                        />
                      </Field>
                    ))}

                    {ENTRY_EQUIPMENT_SELECT_FIELDS.map((field) => (
                      <Field key={field.key} label={field.label}>
                        <DropdownSelect
                          aria-label={field.label}
                          options={field.options.map((option) => ({
                            value: option,
                            label: formatOptionLabel(option),
                            tone: statusOptionTone(option),
                          }))}
                          value={form[field.key]}
                          onChange={(value) => handleSelectChange(field, value)}
                        />
                      </Field>
                    ))}

                    <Field className={ENTRY_CONDITION_FIELD.className} label={ENTRY_CONDITION_FIELD.label}>
                      <Input
                        placeholder={ENTRY_CONDITION_FIELD.placeholder}
                        value={form[ENTRY_CONDITION_FIELD.key]}
                        onChange={(event) => updateForm(setForm, ENTRY_CONDITION_FIELD.key, event.currentTarget.value)}
                      />
                    </Field>

                    {showInlinePicturePreview ? (
                      <div className="lg:col-span-2">
                        <PicturePreviewPanel picturePath={picturePath} preview={picturePreview} />
                      </div>
                    ) : null}

                    <Field className="lg:col-span-2" label="Notes">
                      <Textarea
                        placeholder="Operational notes, repair history, or provenance"
                        value={form.notes}
                        onChange={(event) => updateForm(setForm, "notes", event.currentTarget.value)}
                      />
                    </Field>
                  </div>
                </>
              ) : (
                <>
                  <div className="mb-4 rounded-2xl border border-border/70 bg-background/60 px-4 py-3">
                    <p className="text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">Equipment Identity</p>
                    <p className="mt-1 font-medium text-foreground">
                      {form.description || [form.manufacturer, form.model].filter(Boolean).join(" ") || form.assetNumber || form.serialNumber || "New equipment"}
                    </p>
                    <p className="mt-1 text-sm text-muted-foreground">
                      {[form.assetNumber, form.serialNumber, form.manufacturer, form.model, form.location].filter(Boolean).join(" • ") || "Complete identity fields in the Equipment section."}
                    </p>
                  </div>

                  <div className="grid gap-4 lg:grid-cols-2 lg:gap-5">
                    {ENTRY_CALIBRATION_SELECT_FIELDS.map((field) => (
                      <Field key={field.key} label={field.label}>
                        <DropdownSelect
                          aria-label={field.label}
                          options={field.options.map((option) => ({
                            value: option,
                            label: formatOptionLabel(option),
                            tone: statusOptionTone(option),
                          }))}
                          value={form[field.key]}
                          onChange={(value) => handleSelectChange(field, value)}
                        />
                      </Field>
                    ))}
                    <div />
                    <Field label="Last calibrated">
                      <Input type="date" value={form.lastCalibratedAt} onChange={(event) => updateForm(setForm, "lastCalibratedAt", event.currentTarget.value)} />
                    </Field>
                    <Field label="Calibration due">
                      <Input type="date" value={form.calibrationDueAt} onChange={(event) => updateForm(setForm, "calibrationDueAt", event.currentTarget.value)} />
                    </Field>
                    <div className="block lg:col-span-2">
                      <label
                        className="mb-1.5 block text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground"
                        htmlFor={calibrationIntervalId}
                      >
                        Calibration interval (months)
                      </label>
                      <div className="flex flex-col gap-2 sm:flex-row">
                        <Input
                          id={calibrationIntervalId}
                          inputMode="numeric"
                          type="number"
                          value={form.calibrationIntervalMonths}
                          onChange={(event) => updateForm(setForm, "calibrationIntervalMonths", event.currentTarget.value)}
                        />
                        <Button
                          type="button"
                          variant="outline"
                          onClick={() => {
                            const suggestion = suggestCalibrationDueDate(
                              form.lastCalibratedAt.trim(),
                              Number(form.calibrationIntervalMonths),
                            );
                            if (suggestion) updateForm(setForm, "calibrationDueAt", suggestion);
                          }}
                        >
                          Suggest calibration due date
                        </Button>
                      </div>
                    </div>
                    <Field label="Certificate reference">
                      <Input value={form.certificateRef} onChange={(event) => updateForm(setForm, "certificateRef", event.currentTarget.value)} />
                    </Field>
                    <Field label="Calibration vendor">
                      <Input value={form.calibrationVendor} onChange={(event) => updateForm(setForm, "calibrationVendor", event.currentTarget.value)} />
                    </Field>
                    <Field label="Verified by">
                      <Input value={form.verifiedBy} onChange={(event) => updateForm(setForm, "verifiedBy", event.currentTarget.value)} />
                    </Field>
                    <div />
                    <Field className="lg:col-span-2" label="Calibration notes">
                      <Textarea value={form.calibrationNotes} onChange={(event) => updateForm(setForm, "calibrationNotes", event.currentTarget.value)} />
                    </Field>
                  </div>

                  <div className="mt-4 flex flex-wrap items-center gap-4 rounded-2xl border border-border/70 bg-background/70 px-4 py-3">
                    {ENTRY_CALIBRATION_BOOLEAN_FIELDS.map((field) => (
                      <label className="flex items-center gap-2 text-sm text-foreground" key={field.key}>
                        <input
                          checked={form[field.key]}
                          className="size-4 accent-[var(--primary)]"
                          type="checkbox"
                          onChange={(event) => updateForm(setForm, field.key, event.currentTarget.checked)}
                        />
                        {field.label}
                      </label>
                    ))}
                  </div>
                </>
              )}

              {mode === "edit" && entry ? (
                <div className="mt-6 border-t border-border/60 pt-4">
                  <p className="text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">
                    Entry Context
                  </p>
                  <div className="mt-3 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
                    {buildEntryContextRows(entry).map((row) => (
                      <ContextRow key={row.label} label={row.label} value={row.value} />
                    ))}
                  </div>
                </div>
              ) : null}
            </ScrollRegion>
          </fieldset>

          <div className="shrink-0 border-t border-border/70 px-5 py-4">
            <DialogActions
              archived={form.archived}
              error={error}
              formId={formId}
              isSaving={isSaving}
              readOnly={readOnly}
              onArchiveToggle={
                mode === "edit" && entry
                  ? () => updateForm(setForm, "archived", !form.archived)
                  : undefined
              }
              onClose={requestClose}
              onDelete={
                mode === "edit" && entry?.archived && onDelete ? onDelete : undefined
              }
            />
          </div>
        </form>
      </div>

      <UnsavedChangesDialog
        isSaving={isSaving}
        open={showUnsavedPrompt}
        readOnly={readOnly}
        onCancel={() => setShowUnsavedPrompt(false)}
        onDiscard={() => {
          setShowUnsavedPrompt(false);
          onClose();
        }}
        onSave={() => {
          void handleSaveAndClose();
        }}
      />
    </div>
  );
}

function entryFormEquals(a: EntryFormState, b: EntryFormState): boolean {
  const keys = Object.keys(a) as Array<keyof EntryFormState>;
  return keys.every((key) => a[key] === b[key]);
}

interface FieldProps {
  children: ReactNode;
  className?: string;
  label: string;
}

function Field({ children, className, label }: FieldProps) {
  return (
    <label className={cn("block", className)}>
      <span className="mb-1.5 block text-[11px] font-semibold uppercase tracking-[0.08em] text-muted-foreground">
        {label}
      </span>
      {children}
    </label>
  );
}
