/** Product modules available in the unified shell (switcher prototype). */

export type InventorySystemId =
  | "te-test-equipment"
  | "te-lab-components"
  | "me-storage"
  | "te-storage";

export interface InventorySystemDefinition {
  id: InventorySystemId;
  /** Label in the switcher menu and header title */
  label: string;
  /**
   * Whether this module is wired to real inventory data in this app.
   * Other modules are placeholders until they are ported.
   */
  implemented: boolean;
}

export const INVENTORY_SYSTEMS: readonly InventorySystemDefinition[] = [
  {
    id: "te-test-equipment",
    label: "TE Test Equipment",
    implemented: true,
  },
  {
    id: "te-lab-components",
    label: "TE Lab Components",
    implemented: false,
  },
  {
    id: "me-storage",
    label: "ME Storage",
    implemented: false,
  },
  {
    id: "te-storage",
    label: "TE Storage Room",
    implemented: false,
  },
] as const;

export const DEFAULT_INVENTORY_SYSTEM_ID: InventorySystemId = "te-test-equipment";

const STORAGE_KEY = "inventory.activeSystem";

/** Older prototype ids → current ids (localStorage migration). */
const LEGACY_SYSTEM_IDS: Record<string, InventorySystemId> = {
  "te-parts": "te-lab-components",
  "me-inventory": "me-storage",
};

export function getInventorySystem(id: InventorySystemId): InventorySystemDefinition {
  const found = INVENTORY_SYSTEMS.find((system) => system.id === id);
  return found ?? INVENTORY_SYSTEMS[0];
}

export function isInventorySystemId(value: string): value is InventorySystemId {
  return INVENTORY_SYSTEMS.some((system) => system.id === value);
}

export function readStoredInventorySystemId(): InventorySystemId {
  if (typeof window === "undefined") {
    return DEFAULT_INVENTORY_SYSTEM_ID;
  }

  try {
    const raw = window.localStorage.getItem(STORAGE_KEY);
    if (!raw) {
      return DEFAULT_INVENTORY_SYSTEM_ID;
    }
    if (isInventorySystemId(raw)) {
      return raw;
    }
    const migrated = LEGACY_SYSTEM_IDS[raw];
    if (migrated) {
      window.localStorage.setItem(STORAGE_KEY, migrated);
      return migrated;
    }
  } catch {
    // Ignore storage failures (private mode, etc.).
  }

  return DEFAULT_INVENTORY_SYSTEM_ID;
}

export function writeStoredInventorySystemId(id: InventorySystemId): void {
  if (typeof window === "undefined") {
    return;
  }

  try {
    window.localStorage.setItem(STORAGE_KEY, id);
  } catch {
    // Ignore storage failures.
  }
}
