import type { InventoryDesktopBridge } from "@/integrations/tauri/desktop-bridge";

type LabCatalogMethod =
  | "createLabPart"
  | "updateLabPart"
  | "createLabSimpleComponent"
  | "updateLabSimpleComponent"
  | "deleteLabPart"
  | "createLabStockPlacement"
  | "updateLabStockPlacement"
  | "deleteLabStockPlacement"
  | "moveLabStock"
  | "countLabStock";

type LabLocationMethod =
  | "createLabStorageArea"
  | "updateLabStorageArea"
  | "deleteLabStorageArea"
  | "createLabStorageContainer"
  | "updateLabStorageContainer"
  | "deleteLabStorageContainer";

type BridgeWith<M extends keyof InventoryDesktopBridge> = InventoryDesktopBridge & Required<Pick<InventoryDesktopBridge, M>>;

const CATALOG_METHODS: LabCatalogMethod[] = [
  "createLabPart",
  "updateLabPart",
  "createLabSimpleComponent",
  "updateLabSimpleComponent",
  "deleteLabPart",
  "createLabStockPlacement",
  "updateLabStockPlacement",
  "deleteLabStockPlacement",
  "moveLabStock",
  "countLabStock",
];

const LOCATION_METHODS: LabLocationMethod[] = [
  "createLabStorageArea",
  "updateLabStorageArea",
  "deleteLabStorageArea",
  "createLabStorageContainer",
  "updateLabStorageContainer",
  "deleteLabStorageContainer",
];

function hasMethods<M extends keyof InventoryDesktopBridge>(
  bridge: InventoryDesktopBridge | undefined,
  methods: M[],
): bridge is BridgeWith<M> {
  return Boolean(bridge?.isDesktop && methods.every((method) => typeof bridge[method] === "function"));
}

/** Whether this desktop build can edit storage areas and containers. */
export function labLocationsAvailable(): boolean {
  return hasMethods(window.inventoryDesktop, LOCATION_METHODS);
}

/** The desktop bridge with every Lab catalog command, or an error saying it is missing. */
export function requireLabCatalogBridge(): BridgeWith<LabCatalogMethod> {
  const bridge = window.inventoryDesktop;
  if (!hasMethods(bridge, CATALOG_METHODS)) {
    throw new Error("This desktop build does not expose the Lab catalog-v2 commands.");
  }
  return bridge;
}

/** The desktop bridge with every storage-location command, or an error saying it is missing. */
export function requireLabLocationBridge(): BridgeWith<LabLocationMethod> {
  const bridge = window.inventoryDesktop;
  if (!hasMethods(bridge, LOCATION_METHODS)) {
    throw new Error("This desktop build does not expose the Lab storage-location commands.");
  }
  return bridge;
}
