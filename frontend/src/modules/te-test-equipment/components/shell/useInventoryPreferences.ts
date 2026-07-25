import { useEffect, useState } from "react";

import type { ColumnKey, TeTestEquipmentWorkspace } from "@/modules/te-test-equipment/types";

import {
  CALIBRATION_COLUMN_VISIBILITY_STORAGE_KEY,
  COLOR_ROWS_STORAGE_KEY,
  COLUMN_VISIBILITY_STORAGE_KEY,
  readColorRows,
  readColumnVisibility,
} from "./helpers";

export function useInventoryPreferences(workspace: TeTestEquipmentWorkspace = "equipment") {
  const [colorRows, setColorRows] = useState<boolean>(() => readColorRows());
  const [equipmentColumnVisibility, setEquipmentColumnVisibility] = useState<Record<ColumnKey, boolean>>(
    () => readColumnVisibility("equipment"),
  );
  const [calibrationColumnVisibility, setCalibrationColumnVisibility] = useState<Record<ColumnKey, boolean>>(
    () => readColumnVisibility("calibration"),
  );

  useEffect(() => {
    localStorage.setItem(COLOR_ROWS_STORAGE_KEY, JSON.stringify(colorRows));
  }, [colorRows]);

  useEffect(() => {
    localStorage.setItem(COLUMN_VISIBILITY_STORAGE_KEY, JSON.stringify(equipmentColumnVisibility));
  }, [equipmentColumnVisibility]);

  useEffect(() => {
    localStorage.setItem(CALIBRATION_COLUMN_VISIBILITY_STORAGE_KEY, JSON.stringify(calibrationColumnVisibility));
  }, [calibrationColumnVisibility]);

  const columnVisibility = workspace === "calibration"
    ? calibrationColumnVisibility
    : equipmentColumnVisibility;
  const setColumnVisibility = workspace === "calibration"
    ? setCalibrationColumnVisibility
    : setEquipmentColumnVisibility;

  return {
    colorRows,
    columnVisibility,
    setColorRows,
    setColumnVisibility,
  };
}
