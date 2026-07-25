import type {
  ColumnConfig,
  ColumnKey,
  TeTestEquipmentWorkspace,
} from "@/modules/te-test-equipment/types";
import {
  CALIBRATION_COLUMNS,
  EQUIPMENT_COLUMNS,
} from "@/modules/te-test-equipment/types";

const ALL_COLUMN_KEYS = Array.from(
  new Set([...EQUIPMENT_COLUMNS, ...CALIBRATION_COLUMNS].map((column) => column.key)),
);

export function getColumnsForWorkspace(workspace: TeTestEquipmentWorkspace): readonly ColumnConfig[] {
  return workspace === "calibration" ? CALIBRATION_COLUMNS : EQUIPMENT_COLUMNS;
}

export function buildDefaultColumnVisibility(
  columns: readonly ColumnConfig[] = EQUIPMENT_COLUMNS,
): Record<ColumnKey, boolean> {
  const visibility = ALL_COLUMN_KEYS.reduce<Record<ColumnKey, boolean>>((current, columnKey) => {
    current[columnKey] = false;
    return current;
  }, {} as Record<ColumnKey, boolean>);

  return columns.reduce<Record<ColumnKey, boolean>>((current, column) => {
    current[column.key] = column.defaultVisible;
    return current;
  }, visibility);
}

export function mergeColumnVisibility(
  storedValue: Partial<Record<ColumnKey, boolean>> | null | undefined,
  columns: readonly ColumnConfig[] = EQUIPMENT_COLUMNS,
): Record<ColumnKey, boolean> {
  const visibility = buildDefaultColumnVisibility(columns);
  for (const column of columns) {
    const storedVisibility = storedValue?.[column.key];
    if (typeof storedVisibility === "boolean") {
      visibility[column.key] = storedVisibility;
    }
  }

  if (getVisibleDataColumnCount(visibility, columns) === 0) {
    visibility[firstDefaultDataColumnKey(columns)] = true;
  }
  return visibility;
}

export function getVisibleColumns(
  columnVisibility: Record<ColumnKey, boolean>,
  columns: readonly ColumnConfig[] = EQUIPMENT_COLUMNS,
): ColumnConfig[] {
  return columns.filter((column) => columnVisibility[column.key]);
}

export function getVisibleDataColumnCount(
  columnVisibility: Record<ColumnKey, boolean>,
  columns: readonly ColumnConfig[] = EQUIPMENT_COLUMNS,
): number {
  let visibleColumns = 0;
  for (const column of columns) {
    if (column.key !== "verified" && columnVisibility[column.key]) {
      visibleColumns += 1;
    }
  }
  return visibleColumns;
}

function firstDefaultDataColumnKey(columns: readonly ColumnConfig[]): ColumnKey {
  return (
    columns.find((column) => column.key !== "verified" && column.defaultVisible)?.key ??
    columns.find((column) => column.key !== "verified")?.key ??
    "description"
  );
}

export function formatLinkLabel(link: string): string {
  const text = link.trim();
  if (!text) {
    return "";
  }

  try {
    const parsed = new URL(text);
    const compact = `${parsed.host}${parsed.pathname.replace(/\/$/, "")}`;
    if (compact.length <= 54) {
      return compact;
    }
    return `${compact.slice(0, 51)}...`;
  } catch {
    if (text.length <= 54) {
      return text;
    }
    return `${text.slice(0, 51)}...`;
  }
}
