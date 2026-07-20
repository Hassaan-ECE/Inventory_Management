import { placeholderHost, type InventoryModuleDefinition } from "@/platform/modules/types";

export const teStorageDefinition: InventoryModuleDefinition = {
  id: "te-storage",
  label: "TE Storage Room",
  implemented: false,
  sharedFolderName: "TE_Storage_Room",
};

export const teStorageHost = placeholderHost(teStorageDefinition);
