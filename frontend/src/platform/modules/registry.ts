import { meStorageHost } from "@/modules/me-storage";
import { teLabComponentsHost } from "@/modules/te-lab-components";
import { teStorageHost } from "@/modules/te-storage";
import { teTestEquipmentHost } from "@/modules/te-test-equipment";

import {
  TE_TEST_EQUIPMENT_CALIBRATION_VIEW_ID,
  type InventoryModuleDefinition,
  type InventoryModuleHost,
  type InventoryViewDefinition,
  type InventoryViewId,
  type ModuleId,
} from "./types";

export const INVENTORY_MODULE_HOSTS: readonly InventoryModuleHost[] = [
  teTestEquipmentHost,
  teLabComponentsHost,
  meStorageHost,
  teStorageHost,
] as const;

function moduleView(definition: InventoryModuleDefinition): InventoryViewDefinition {
  return {
    id: definition.id,
    label: definition.label,
    moduleId: definition.id,
  };
}

export const INVENTORY_VIEW_DEFINITIONS: readonly InventoryViewDefinition[] = [
  moduleView(teTestEquipmentHost.definition),
  {
    id: TE_TEST_EQUIPMENT_CALIBRATION_VIEW_ID,
    label: "TE Test Equipment Calibration",
    moduleId: "te-test-equipment",
  },
  moduleView(teLabComponentsHost.definition),
  moduleView(meStorageHost.definition),
  moduleView(teStorageHost.definition),
] as const;

export function getModuleHost(id: ModuleId): InventoryModuleHost {
  const found = INVENTORY_MODULE_HOSTS.find((host) => host.definition.id === id);
  if (!found) {
    throw new Error(`Unknown module id: ${id}`);
  }
  return found;
}

export function getInventoryViewDefinition(id: InventoryViewId): InventoryViewDefinition {
  const found = INVENTORY_VIEW_DEFINITIONS.find((definition) => definition.id === id);
  if (!found) {
    throw new Error(`Unknown inventory view id: ${id}`);
  }
  return found;
}

export function listModuleDefinitions(): readonly InventoryModuleDefinition[] {
  return INVENTORY_MODULE_HOSTS.map((host) => host.definition);
}

export function listInventoryViewDefinitions(): readonly InventoryViewDefinition[] {
  return INVENTORY_VIEW_DEFINITIONS;
}
