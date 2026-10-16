import { describe, expect, it } from "vitest";

import { INVENTORY_MODULE_HOSTS, listInventoryViewDefinitions } from "@/platform/modules/registry";

describe("platform module registry", () => {
  it("lists the three inventories in switcher order", () => {
    expect(INVENTORY_MODULE_HOSTS.map((host) => host.definition)).toEqual([
      { id: "te-test-equipment", label: "TE Test Equipment" },
      { id: "te-lab-components", label: "TE Lab Components" },
      { id: "te-storage", label: "EVCS Storage" },
    ]);
  });

  it("adds the Calibration view under TE Test Equipment", () => {
    expect(listInventoryViewDefinitions().map((view) => [view.id, view.moduleId])).toEqual([
      ["te-test-equipment", "te-test-equipment"],
      ["te-test-equipment-calibration", "te-test-equipment"],
      ["te-lab-components", "te-lab-components"],
      ["te-storage", "te-storage"],
    ]);
  });
});
