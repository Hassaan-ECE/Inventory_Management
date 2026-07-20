import { listModuleDefinitions } from "./registry";
import type { ModuleId } from "./types";

export const DEFAULT_MODULE_ID: ModuleId = "te-test-equipment";

const STORAGE_KEY = "inventory.activeSystem";
const LEGACY_MODULE_IDS: Record<string, ModuleId> = {
  "te-parts": "te-lab-components",
  "me-inventory": "me-storage",
};

export function isModuleId(value: string): value is ModuleId {
  return listModuleDefinitions().some((definition) => definition.id === value);
}

export function readStoredModuleId(): ModuleId {
  if (typeof window === "undefined") {
    return DEFAULT_MODULE_ID;
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return DEFAULT_MODULE_ID;
    }
    if (isModuleId(raw)) {
      return raw;
    }
    const migrated = LEGACY_MODULE_IDS[raw];
    if (migrated) {
      window.localStorage.setItem(STORAGE_KEY, migrated);
      return migrated;
    }
  } catch {
    return DEFAULT_MODULE_ID;
  }

  return DEFAULT_MODULE_ID;
}

export function writeStoredModuleId(id: ModuleId): void {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(STORAGE_KEY, id);
  } catch {
    return;
  }
}
