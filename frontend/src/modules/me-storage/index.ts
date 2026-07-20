import { placeholderHost, type InventoryModuleDefinition } from "@/platform/modules/types";

export const meStorageDefinition: InventoryModuleDefinition = {
  id: "me-storage",
  label: "ME Storage",
  implemented: false,
  sharedFolderName: "ME_Storage",
};

export const meStorageHost = placeholderHost(meStorageDefinition);
