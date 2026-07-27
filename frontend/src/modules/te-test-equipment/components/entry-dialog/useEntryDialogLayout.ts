import type { InventoryEntry } from "@/modules/te-test-equipment/types";

interface UseEntryDialogLayoutOptions {
  entry?: InventoryEntry | null;
  mode: "add" | "edit";
  picturePath: string;
  readOnly: boolean;
}

/** Layout is always a single column + footer actions (no right sidebar). */
export function useEntryDialogLayout({
  picturePath,
  readOnly,
}: UseEntryDialogLayoutOptions) {
  const hasPicturePath = Boolean(picturePath);

  return {
    showInlinePicturePreview: !readOnly || hasPicturePath,
  };
}
