import type { InventoryModuleDefinition, InventoryModuleHost } from "@/platform/modules/types";
import { TeStorageView } from "./TeStorageView";

const teStorageDefinition: InventoryModuleDefinition = {
  id: "te-storage",
  label: "EVCS Storage",
};

export const teStorageHost: InventoryModuleHost = { definition: teStorageDefinition, MainView: TeStorageView };
