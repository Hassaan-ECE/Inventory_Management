import type { InventoryModuleDefinition, InventoryModuleHost } from "@/platform/modules/types";

import { TeTestEquipmentView } from "./TeTestEquipmentView";
import { TE_TEST_EQUIPMENT_MODULE_ID } from "./moduleId";

export const teTestEquipmentDefinition: InventoryModuleDefinition = {
  id: TE_TEST_EQUIPMENT_MODULE_ID,
  label: "TE Test Equipment",
  implemented: true,
  sharedFolderName: "TE_Test_Equipment",
};

export const teTestEquipmentHost: InventoryModuleHost = {
  kind: "desktop",
  definition: teTestEquipmentDefinition,
  MainView: TeTestEquipmentView,
};
