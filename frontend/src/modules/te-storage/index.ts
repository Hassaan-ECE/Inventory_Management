import type { InventoryModuleDefinition, InventoryModuleHost } from "@/platform/modules/types";
import { TeStorageView } from "./TeStorageView";

export const teStorageDefinition: InventoryModuleDefinition = {
  id: "te-storage",
  label: "TE Storage",
  implemented: true,
  sharedFolderName: "TE_Storage_Room",
};

export const teStorageHost: InventoryModuleHost = { kind: "desktop", definition: teStorageDefinition, MainView: TeStorageView };
