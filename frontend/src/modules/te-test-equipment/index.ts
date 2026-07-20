import type { InventoryModuleDefinition, InventoryModuleHost } from "@/platform/modules/types";

import { TeTestEquipmentView } from "./TeTestEquipmentView";

export const teTestEquipmentDefinition: InventoryModuleDefinition = {
  id: "te-test-equipment",
  label: "TE Test Equipment",
  implemented: true,
  sharedFolderName: "TE_Test_Equipment",
};

export const teTestEquipmentHost: InventoryModuleHost = {
  kind: "desktop",
  definition: teTestEquipmentDefinition,
  MainView: TeTestEquipmentView,
};
