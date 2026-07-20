import { useEffect, useState } from "react";

import type { ColumnKey } from "@/modules/te-test-equipment/types";

import {
  COLOR_ROWS_STORAGE_KEY,
  COLUMN_VISIBILITY_STORAGE_KEY,
  readColorRows,
  readColumnVisibility,
} from "./helpers";

export function useInventoryPreferences() {
  const [colorRows, setColorRows] = useState<boolean>(() => readColorRows());
  const [columnVisibility, setColumnVisibility] = useState<Record<ColumnKey, boolean>>(() => readColumnVisibility());

  useEffect(() => {
    localStorage.setItem(COLOR_ROWS_STORAGE_KEY, JSON.stringify(colorRows));
  }, [colorRows]);

  useEffect(() => {
    localStorage.setItem(COLUMN_VISIBILITY_STORAGE_KEY, JSON.stringify(columnVisibility));
  }, [columnVisibility]);

  return {
    colorRows,
    columnVisibility,
    setColorRows,
    setColumnVisibility,
  };
}
