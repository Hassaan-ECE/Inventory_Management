import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { APP_VERSION } from "@/app/branding";
import type {
  InventorySharedChangedPayload,
  InventorySyncResult,
} from "@/integrations/tauri/desktop-bridge";
import type {
  InventoryEntry as LabInventoryEntry,
  InventorySharedStatus as LabInventorySharedStatus,
} from "@/modules/te-lab-components/types";
import { InventoryShell } from "@/shell/InventoryShell";

import {
  CONNECTED_SHARED_STATUS,
  TEST_DB_PATH,
  buildDesktopSyncResult,
  buildTestEntry,
  createDeferred,
  createDesktopBridge,
  flushAsyncWork,
} from "./inventory-shell/helpers";

const LAB_DB_PATH =
  "C:/Users/Test/AppData/Local/com.inventory.management/te-lab-components.feox";
const LAB_SHARED_STATUS: LabInventorySharedStatus = {
  available: true,
  canModify: true,
  enabled: true,
  message: "",
  mutationMode: "shared",
};

describe("TE Lab Components shell integration", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
    document.documentElement.classList.remove("dark");
    delete window.inventoryDesktop;
    vi.spyOn(document, "hasFocus").mockReturnValue(true);
  });

  it("switches Lab to TE and back with independent sessions and cached Lab rows", async () => {
    localStorage.setItem("inventory.activeSystem", "te-lab-components");
    const user = userEvent.setup();
    const labEntry = buildLabEntry({ description: "Cached Lab row" });
    const teEntry = buildTestEntry({ description: "Cached TE row" });
    const secondLabLoad = createDeferred<InventorySyncResult<"te-lab-components">>();
    const sharedCallbacks: Array<(payload: InventorySharedChangedPayload) => void> = [];
    const unsubscribeCallbacks = [vi.fn(), vi.fn(), vi.fn()];
    let labActivations = 0;
    let teActivations = 0;
    let labLoads = 0;

    const activateInventorySync = vi.fn(
      async (moduleId: "te-test-equipment" | "te-lab-components") => {
        if (moduleId === "te-lab-components") {
          labActivations += 1;
          return `lab-session-${labActivations}`;
        }

        teActivations += 1;
        return `te-session-${teActivations}`;
      },
    );
    const deactivateInventorySync = vi.fn().mockResolvedValue(true);
    const loadInventory = vi.fn(
      (moduleId: "te-test-equipment" | "te-lab-components") => {
        if (moduleId === "te-test-equipment") {
          return Promise.resolve(
            buildDesktopSyncResult(CONNECTED_SHARED_STATUS, [teEntry]),
          );
        }

        labLoads += 1;
        return labLoads === 1
          ? Promise.resolve(buildLabSyncResult([labEntry]))
          : secondLabLoad.promise;
      },
    );
    const syncInventory = vi.fn(
      async (moduleId: "te-test-equipment" | "te-lab-components") =>
        moduleId === "te-test-equipment"
          ? {
              dbPath: TEST_DB_PATH,
              entries: [],
              entriesChanged: false,
              shared: CONNECTED_SHARED_STATUS,
            }
          : buildLabSyncResult([], false),
    );
    const onSharedInventoryChanged = vi.fn(
      (callback: (payload: InventorySharedChangedPayload) => void) => {
        const index = sharedCallbacks.length;
        sharedCallbacks.push(callback);
        return unsubscribeCallbacks[index] ?? (() => undefined);
      },
    );

    window.inventoryDesktop = createDesktopBridge({
      activateInventorySync,
      deactivateInventorySync,
      loadInventory,
      onSharedInventoryChanged,
      syncInventory,
    });

    render(<InventoryShell />);

    expect(await screen.findByText("Cached Lab row")).toBeInTheDocument();
    await waitFor(() =>
      expect(syncInventory).toHaveBeenCalledWith(
        "te-lab-components",
        "lab-session-1",
      ),
    );
    expect(activateInventorySync).toHaveBeenNthCalledWith(1, "te-lab-components");
    expect(document.title).toBe(
      `Inventory Management — TE Lab Components v${APP_VERSION}`,
    );

    await switchInventory(user, "TE Test Equipment");

    expect(await screen.findByText("Cached TE row")).toBeInTheDocument();
    await waitFor(() =>
      expect(deactivateInventorySync).toHaveBeenCalledWith(
        "te-lab-components",
        "lab-session-1",
      ),
    );
    await waitFor(() =>
      expect(syncInventory).toHaveBeenCalledWith(
        "te-test-equipment",
        "te-session-1",
      ),
    );
    expect(unsubscribeCallbacks[0]).toHaveBeenCalledTimes(1);
    expect(document.title).toBe(
      `Inventory Management — TE Test Equipment v${APP_VERSION}`,
    );

    syncInventory.mockClear();
    act(() => {
      sharedCallbacks[1]?.({ systemId: "te-lab-components" });
    });
    await flushAsyncWork();
    expect(syncInventory).not.toHaveBeenCalled();

    act(() => {
      sharedCallbacks[1]?.({ systemId: "te-test-equipment" });
    });
    await waitFor(() =>
      expect(syncInventory).toHaveBeenCalledWith(
        "te-test-equipment",
        "te-session-1",
      ),
    );

    await switchInventory(user, "TE Lab Components");

    expect(screen.getByText("Cached Lab row")).toBeInTheDocument();
    await waitFor(() =>
      expect(deactivateInventorySync).toHaveBeenCalledWith(
        "te-test-equipment",
        "te-session-1",
      ),
    );
    expect(loadInventory.mock.calls.filter(([moduleId]) => moduleId === "te-lab-components")).toHaveLength(2);
    expect(unsubscribeCallbacks[1]).toHaveBeenCalledTimes(1);

    await act(async () => {
      secondLabLoad.resolve(buildLabSyncResult([labEntry]));
      await secondLabLoad.promise;
      await Promise.resolve();
    });

    await waitFor(() =>
      expect(syncInventory).toHaveBeenCalledWith(
        "te-lab-components",
        "lab-session-2",
      ),
    );
    expect(document.title).toBe(
      `Inventory Management — TE Lab Components v${APP_VERSION}`,
    );
  });

  it("shows Lab filters and entry fields without calibration controls", async () => {
    localStorage.setItem("inventory.activeSystem", "te-lab-components");
    const user = userEvent.setup();

    render(<InventoryShell />);

    expect(screen.getByRole("columnheader", { name: /Verified/i })).toBeInTheDocument();
    expect(screen.getByText(/Verified: \d+\/\d+/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Import" })).not.toBeInTheDocument();
    expect(document.title).toBe(
      `Inventory Management — TE Lab Components v${APP_VERSION}`,
    );

    await user.click(screen.getByRole("button", { name: "View settings" }));
    await user.click(screen.getByRole("menuitem", { name: /Show filters/i }));

    expect(screen.getByLabelText("Filter manufacturer")).toBeInTheDocument();
    expect(screen.getByLabelText("Filter location")).toBeInTheDocument();
    expect(screen.queryByLabelText("Calibration requirement")).not.toBeInTheDocument();
    expect(screen.queryByLabelText("Calibration health")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Add Entry" }));

    const dialog = screen.getByRole("dialog");
    expect(within(dialog).getByRole("heading", { name: "Add Entry" })).toBeInTheDocument();
    expect(within(dialog).getByLabelText("Verified in survey")).toBeInTheDocument();
    expect(within(dialog).getByLabelText("Quantity")).toBeInTheDocument();
    expect(within(dialog).queryByText(/Calibration/i)).not.toBeInTheDocument();
  });
});

function buildLabEntry(
  overrides: Partial<LabInventoryEntry> = {},
): LabInventoryEntry {
  return {
    id: "lab-701",
    assetNumber: "LAB-701",
    serialNumber: "",
    qty: 25,
    manufacturer: "Lab Maker",
    model: "LC-701",
    description: "Lab component",
    projectName: "Bench Controls",
    location: "Cabinet A / Bin 1",
    assignedTo: "",
    links: "",
    notes: "",
    lifecycleStatus: "active",
    workingStatus: "working",
    condition: "",
    verifiedInSurvey: true,
    archived: false,
    createdAt: "2026-07-20T10:00:00.000Z",
    updatedAt: "2026-07-20T10:00:00.000Z",
    entryUuid: "lab-entry-701",
    manualEntry: true,
    picturePath: "",
    ...overrides,
  };
}

function buildLabSyncResult(
  entries: LabInventoryEntry[],
  entriesChanged?: boolean,
): InventorySyncResult<"te-lab-components"> {
  return {
    dbPath: LAB_DB_PATH,
    entries,
    entriesChanged,
    shared: LAB_SHARED_STATUS,
  };
}

async function switchInventory(
  user: ReturnType<typeof userEvent.setup>,
  label: "TE Lab Components" | "TE Test Equipment",
): Promise<void> {
  await user.click(screen.getByRole("button", { name: "Switch inventory system" }));
  await user.click(screen.getByRole("option", { name: label }));
}
