import { placeholderHost, type InventoryModuleDefinition } from "@/platform/modules/types";

export const teLabComponentsDefinition: InventoryModuleDefinition = {
  id: "te-lab-components",
  label: "TE Lab Components",
  implemented: false,
  sharedFolderName: "TE_Lab_Components",
};

export const teLabComponentsHost = placeholderHost(teLabComponentsDefinition);
