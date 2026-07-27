import { useState } from "react";

import { getInventoryViewDefinition, getModuleHost } from "@/platform/modules/registry";
import {
  readStoredInventoryViewId,
  writeStoredInventoryViewId,
} from "@/platform/modules/persistence";
import type { InventoryViewId } from "@/platform/modules/types";

export function useShellActiveModule() {
  const [activeViewId, setActiveViewId] = useState<InventoryViewId>(() => readStoredInventoryViewId());
  const activeView = getInventoryViewDefinition(activeViewId);
  const activeModuleId = activeView.moduleId;

  function selectView(id: InventoryViewId): void {
    setActiveViewId(id);
    writeStoredInventoryViewId(id);
  }

  return {
    activeHost: getModuleHost(activeModuleId),
    activeModuleId,
    activeView,
    activeViewId,
    selectView,
  };
}
