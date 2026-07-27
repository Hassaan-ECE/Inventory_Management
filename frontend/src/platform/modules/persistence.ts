import { listInventoryViewDefinitions } from "./registry";
import type { InventoryViewId } from "./types";

export const DEFAULT_INVENTORY_VIEW_ID: InventoryViewId = "te-test-equipment";

const STORAGE_KEY = "inventory.activeSystem";
const LEGACY_VIEW_IDS: Record<string, InventoryViewId> = {
  "te-parts": "te-lab-components",
  "me-inventory": "me-storage",
};

export function isInventoryViewId(value: string): value is InventoryViewId {
  return listInventoryViewDefinitions().some((definition) => definition.id === value);
}

export function readStoredInventoryViewId(): InventoryViewId {
  if (typeof window === "undefined") {
    return DEFAULT_INVENTORY_VIEW_ID;
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return DEFAULT_INVENTORY_VIEW_ID;
    }
    if (isInventoryViewId(raw)) {
      return raw;
    }
    const migrated = LEGACY_VIEW_IDS[raw];
    if (migrated) {
      window.localStorage.setItem(STORAGE_KEY, migrated);
      return migrated;
    }
  } catch {
    return DEFAULT_INVENTORY_VIEW_ID;
  }

  return DEFAULT_INVENTORY_VIEW_ID;
}

export function writeStoredInventoryViewId(id: InventoryViewId): void {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(STORAGE_KEY, id);
  } catch {
    return;
  }
}
