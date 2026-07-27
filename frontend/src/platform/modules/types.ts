import type { ComponentType } from "react";

import type { ThemeMode } from "@/platform/ui/theme";

export type ModuleId =
  | "te-test-equipment"
  | "te-lab-components"
  | "me-storage"
  | "te-storage";

export const TE_TEST_EQUIPMENT_CALIBRATION_VIEW_ID = "te-test-equipment-calibration" as const;

export type InventoryViewId = ModuleId | typeof TE_TEST_EQUIPMENT_CALIBRATION_VIEW_ID;

export interface InventoryModuleDefinition {
  id: ModuleId;
  label: string;
  implemented: boolean;
  sharedFolderName: string;
}

export interface InventoryViewDefinition {
  id: InventoryViewId;
  label: string;
  moduleId: ModuleId;
}

/** Shell chrome passed into implemented modules so they can render one unified top bar. */
export interface DesktopModuleViewProps {
  active: boolean;
  activeViewId: InventoryViewId;
  onThemeToggle: () => void;
  onViewChange: (id: InventoryViewId) => void;
  theme: ThemeMode;
}

export type InventoryModuleHost =
  | { kind: "placeholder"; definition: InventoryModuleDefinition }
  | {
      kind: "desktop";
      definition: InventoryModuleDefinition;
      MainView: ComponentType<DesktopModuleViewProps>;
    };

export function placeholderHost(
  definition: InventoryModuleDefinition,
): InventoryModuleHost {
  return { kind: "placeholder", definition };
}
