import type { ComponentType } from "react";

import type { ThemeMode } from "@/platform/ui/theme";

export type ModuleId = "te-test-equipment" | "te-lab-components" | "te-storage";

export const TE_TEST_EQUIPMENT_CALIBRATION_VIEW_ID = "te-test-equipment-calibration" as const;

export type InventoryViewId = ModuleId | typeof TE_TEST_EQUIPMENT_CALIBRATION_VIEW_ID;

export interface InventoryModuleDefinition {
  id: ModuleId;
  label: string;
}

export interface InventoryViewDefinition {
  id: InventoryViewId;
  label: string;
  moduleId: ModuleId;
}

/** Shell chrome passed into each module so it can render its own top bar with the shared switcher and theme toggle. */
export interface DesktopModuleViewProps {
  active: boolean;
  activeViewId: InventoryViewId;
  onThemeToggle: () => void;
  onViewChange: (id: InventoryViewId) => void;
  theme: ThemeMode;
}

export interface InventoryModuleHost {
  definition: InventoryModuleDefinition;
  MainView: ComponentType<DesktopModuleViewProps>;
}
