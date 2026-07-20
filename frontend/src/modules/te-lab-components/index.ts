import type { InventoryModuleDefinition, InventoryModuleHost } from "@/platform/modules/types";

import { TeLabComponentsView } from "./TeLabComponentsView";
import { TE_LAB_COMPONENTS_MODULE_ID } from "./moduleId";

export const teLabComponentsDefinition: InventoryModuleDefinition = {
  id: TE_LAB_COMPONENTS_MODULE_ID,
  label: "TE Lab Components",
  implemented: true,
  sharedFolderName: "TE_Lab_Components",
};

export const teLabComponentsHost: InventoryModuleHost = {
  kind: "desktop",
  definition: teLabComponentsDefinition,
  MainView: TeLabComponentsView,
};
