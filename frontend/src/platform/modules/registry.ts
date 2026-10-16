import { teLabComponentsHost } from "@/modules/te-lab-components";
import { teStorageHost } from "@/modules/te-storage";
import { teTestEquipmentHost } from "@/modules/te-test-equipment";

import {
  TE_TEST_EQUIPMENT_CALIBRATION_VIEW_ID,
  type InventoryModuleDefinition,
  type InventoryModuleHost,
  type InventoryViewDefinition,
  type InventoryViewId,
} from "./types";

export const INVENTORY_MODULE_HOSTS: readonly InventoryModuleHost[] = [
  teTestEquipmentHost,
  teLabComponentsHost,
  teStorageHost,
] as const;

function moduleView(definition: InventoryModuleDefinition): InventoryViewDefinition {
  return {
    id: definition.id,
    label: definition.label,
    moduleId: definition.id,
  };
}

const INVENTORY_VIEW_DEFINITIONS: readonly InventoryViewDefinition[] = [
  moduleView(teTestEquipmentHost.definition),
  {
    id: TE_TEST_EQUIPMENT_CALIBRATION_VIEW_ID,
    label: "TE Test Equipment Calibration",
    moduleId: "te-test-equipment",
  },
  moduleView(teLabComponentsHost.definition),
  moduleView(teStorageHost.definition),
] as const;

export function getInventoryViewDefinition(id: InventoryViewId): InventoryViewDefinition {
  const found = INVENTORY_VIEW_DEFINITIONS.find((definition) => definition.id === id);
  if (!found) {
    throw new Error(`Unknown inventory view id: ${id}`);
  }
  return found;
}

export function listInventoryViewDefinitions(): readonly InventoryViewDefinition[] {
  return INVENTORY_VIEW_DEFINITIONS;
}
