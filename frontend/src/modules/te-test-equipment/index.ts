import type { InventoryModuleDefinition, InventoryModuleHost } from "@/platform/modules/types";

import { TeTestEquipmentView } from "./TeTestEquipmentView";
import { TE_TEST_EQUIPMENT_MODULE_ID } from "./moduleId";

const teTestEquipmentDefinition: InventoryModuleDefinition = {
  id: TE_TEST_EQUIPMENT_MODULE_ID,
  label: "TE Test Equipment",
};

export const teTestEquipmentHost: InventoryModuleHost = {
  definition: teTestEquipmentDefinition,
  MainView: TeTestEquipmentView,
};
