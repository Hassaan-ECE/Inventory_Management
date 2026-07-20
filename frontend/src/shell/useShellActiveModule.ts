import { useState } from "react";

import { getModuleHost } from "@/platform/modules/registry";
import { readStoredModuleId, writeStoredModuleId } from "@/platform/modules/persistence";
import type { ModuleId } from "@/platform/modules/types";

export function useShellActiveModule() {
  const [activeModuleId, setActiveModuleId] = useState<ModuleId>(() => readStoredModuleId());

  function selectModule(id: ModuleId): void {
    setActiveModuleId(id);
    writeStoredModuleId(id);
  }

  return {
    activeHost: getModuleHost(activeModuleId),
    activeModuleId,
    selectModule,
  };
}
