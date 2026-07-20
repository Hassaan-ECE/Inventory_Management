import { meStorageHost } from "@/modules/me-storage";
import { teLabComponentsHost } from "@/modules/te-lab-components";
import { teStorageHost } from "@/modules/te-storage";
import { teTestEquipmentHost } from "@/modules/te-test-equipment";

import type { InventoryModuleDefinition, InventoryModuleHost, ModuleId } from "./types";

export const INVENTORY_MODULE_HOSTS: readonly InventoryModuleHost[] = [
  teTestEquipmentHost,
  teLabComponentsHost,
  meStorageHost,
  teStorageHost,
] as const;

export function getModuleHost(id: ModuleId): InventoryModuleHost {
  const found = INVENTORY_MODULE_HOSTS.find((host) => host.definition.id === id);
  if (!found) {
    throw new Error(`Unknown module id: ${id}`);
  }
  return found;
}

export function listModuleDefinitions(): readonly InventoryModuleDefinition[] {
  return INVENTORY_MODULE_HOSTS.map((host) => host.definition);
}
