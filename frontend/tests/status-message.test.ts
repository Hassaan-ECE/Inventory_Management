import { describe, expect, it } from "vitest";

import {
  buildDefaultStatusMessage,
  isRoutineSharedStatusMessage,
} from "@/features/inventory/components/shell/helpers";
import type { InventorySharedStatus } from "@/features/inventory/types";

function shared(message: string, overrides: Partial<InventorySharedStatus> = {}): InventorySharedStatus {
  return {
    available: true,
    canModify: true,
    enabled: true,
    message,
    mutationMode: "shared",
    ...overrides,
  };
}

describe("status strip shared messages", () => {
  it("treats healthy idle sync text as routine", () => {
    expect(isRoutineSharedStatusMessage("Shared operation sync ready.")).toBe(true);
    expect(isRoutineSharedStatusMessage("Shared operation sync ready. Snapshot refreshed.")).toBe(true);
    expect(isRoutineSharedStatusMessage("FeOxDB local store ready. Shared sync starting.")).toBe(true);
  });

  it("keeps actionable shared messages", () => {
    expect(isRoutineSharedStatusMessage("Shared workspace unavailable. Saving changes locally.")).toBe(false);
    expect(
      isRoutineSharedStatusMessage("Shared operation sync ready. Pending local changes: 3."),
    ).toBe(false);
    expect(isRoutineSharedStatusMessage("Local change published to shared sync.")).toBe(false);
    expect(
      isRoutineSharedStatusMessage("Shared operation sync ready. Ignored 1 corrupt remote file(s)."),
    ).toBe(false);
  });

  it("hides routine ready text from the footer when Shared pill already covers mode", () => {
    expect(buildDefaultStatusMessage(10, 2, "desktop", shared("Shared operation sync ready."))).toBe("");
    expect(
      buildDefaultStatusMessage(10, 2, "desktop", shared("Shared operation sync ready. Snapshot refreshed.")),
    ).toBe("");
  });

  it("still surfaces pending and unavailable messages", () => {
    expect(
      buildDefaultStatusMessage(
        10,
        2,
        "desktop",
        shared("Shared operation sync ready. Pending local changes: 2."),
      ),
    ).toContain("Pending local changes");
    expect(
      buildDefaultStatusMessage(
        10,
        2,
        "desktop",
        shared("Shared workspace unavailable. Saving changes locally.", {
          available: false,
          mutationMode: "local",
        }),
      ),
    ).toContain("unavailable");
  });
});
