import { describe, expect, it } from "vitest";

import { getModuleHost, listModuleDefinitions } from "@/platform/modules/registry";

describe("platform module registry", () => {
  it("lists four modules with only TE implemented", () => {
    const definitions = listModuleDefinitions();

    expect(definitions).toHaveLength(4);
    expect(definitions.filter((definition) => definition.implemented)).toEqual([
      expect.objectContaining({
        id: "te-test-equipment",
        sharedFolderName: "TE_Test_Equipment",
      }),
    ]);
  });

  it("returns placeholder hosts for non-TE modules", () => {
    expect(getModuleHost("me-storage").kind).toBe("placeholder");
    expect(getModuleHost("te-test-equipment").kind).toBe("desktop");
  });
});
