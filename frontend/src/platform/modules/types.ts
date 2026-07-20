import type { ComponentType } from "react";

import type { ThemeMode } from "@/platform/ui/theme";

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

/** Shell chrome passed into implemented modules so they can render one unified top bar. */
export interface DesktopModuleViewProps {
  active: boolean;
  activeModuleId: ModuleId;
  onModuleChange: (id: ModuleId) => void;
  onThemeToggle: () => void;
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
