import type { ComponentType } from "react";

export type ModuleId =
  | "te-test-equipment"
  | "te-lab-components"
  | "me-storage"
  | "te-storage";

export interface InventoryModuleDefinition {
  id: ModuleId;
  label: string;
  implemented: boolean;
  sharedFolderName: string;
}

export type InventoryModuleHost =
  | { kind: "placeholder"; definition: InventoryModuleDefinition }
  | {
      kind: "desktop";
      definition: InventoryModuleDefinition;
      MainView: ComponentType<{ active: boolean }>;
    };

export function placeholderHost(
  definition: InventoryModuleDefinition,
): InventoryModuleHost {
  return { kind: "placeholder", definition };
}
