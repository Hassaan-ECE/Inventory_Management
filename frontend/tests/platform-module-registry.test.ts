import { describe, expect, it } from "vitest";

import { getModuleHost, listModuleDefinitions } from "@/platform/modules/registry";

describe("platform module registry", () => {
  it("lists TE and Lab as desktop modules before the placeholders", () => {
    const definitions = listModuleDefinitions();

    expect(definitions).toHaveLength(4);
    expect(definitions.map((definition) => definition.id)).toEqual([
      "te-test-equipment",
      "te-lab-components",
      "me-storage",
      "te-storage",
    ]);
    expect(definitions.filter((definition) => definition.implemented)).toEqual([
      expect.objectContaining({
        id: "te-test-equipment",
        sharedFolderName: "TE_Test_Equipment",
      }),
      expect.objectContaining({
        id: "te-lab-components",
        sharedFolderName: "TE_Lab_Components",
      }),
    ]);
  });

  it("returns desktop hosts for TE and Lab only", () => {
    expect(getModuleHost("me-storage").kind).toBe("placeholder");
    expect(getModuleHost("te-storage").kind).toBe("placeholder");
    expect(getModuleHost("te-test-equipment").kind).toBe("desktop");
    expect(getModuleHost("te-lab-components").kind).toBe("desktop");
  });
});
