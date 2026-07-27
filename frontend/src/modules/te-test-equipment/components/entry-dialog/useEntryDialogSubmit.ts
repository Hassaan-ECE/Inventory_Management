import { useCallback, useState } from "react";
import type { FormEvent, MutableRefObject } from "react";

import type {
  InventoryEntry,
  InventoryEntryEditContext,
  InventoryEntryInput,
} from "@/modules/te-test-equipment/types";

import { buildEditContext } from "./editContext";
import { buildEntryInput, type EntryFormState } from "./form";

interface UseEntryDialogSubmitOptions {
  entry?: InventoryEntry | null;
  form: EntryFormState;
  initialForm: EntryFormState;
  isMountedRef: MutableRefObject<boolean>;
  mode: "add" | "edit";
  onSave: (input: InventoryEntryInput, editContext?: InventoryEntryEditContext) => Promise<void> | void;
  readOnly: boolean;
  setError: (message: string | null) => void;
}

export function useEntryDialogSubmit({
  entry,
  form,
  initialForm,
  isMountedRef,
  mode,
  onSave,
  readOnly,
  setError,
}: UseEntryDialogSubmitOptions) {
  const [isSaving, setIsSaving] = useState(false);

  const saveEntry = useCallback(async (): Promise<boolean> => {
    if (readOnly) {
      return false;
    }

    const result = buildEntryInput(form);
    if ("error" in result) {
      setError(result.error);
      return false;
    }

    try {
      setIsSaving(true);
      setError(null);
      await onSave(result.value, buildEditContext(mode, entry, initialForm, result.value));
      return true;
    } catch (submissionError) {
      if (!isMountedRef.current) {
        return false;
      }
      setIsSaving(false);
      setError(submissionError instanceof Error ? submissionError.message : "Could not save this entry.");
      return false;
    }
  }, [entry, form, initialForm, isMountedRef, mode, onSave, readOnly, setError]);

  const handleSubmit = useCallback(
    async (event: FormEvent<HTMLFormElement>): Promise<void> => {
      event.preventDefault();
      await saveEntry();
    },
    [saveEntry],
  );

  return { handleSubmit, isSaving, saveEntry };
}
