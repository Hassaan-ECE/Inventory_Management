import type { InventoryModuleDefinition, InventoryModuleHost } from "@/platform/modules/types";

import { TeLabComponentsView } from "./TeLabComponentsView";
import { TE_LAB_COMPONENTS_MODULE_ID } from "./moduleId";

const teLabComponentsDefinition: InventoryModuleDefinition = {
  id: TE_LAB_COMPONENTS_MODULE_ID,
  label: "TE Lab Components",
};

export const teLabComponentsHost: InventoryModuleHost = {
  definition: teLabComponentsDefinition,
  MainView: TeLabComponentsView,
};
