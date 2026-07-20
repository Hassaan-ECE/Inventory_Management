import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { InventoryShell } from "@/features/inventory/components/InventoryShell";
import type { InventoryEntry } from "@/features/inventory/types";
import type {
  InventorySharedChangedPayload,
  InventorySyncResult,
} from "@/integrations/tauri/desktop-bridge";
import {
  CONNECTED_SHARED_STATUS,
  DISABLED_SHARED_STATUS,
  TEST_DB_PATH,
  buildDesktopSyncResult,
  buildTestEntry,
  createDeferred,
  createDesktopBridge,
  delay,
  flushAsyncWork,
} from "./inventory-shell/helpers";

describe("InventoryShell shared sync", () => {
  beforeEach(() => {
    vi.restoreAllMocks();
    localStorage.clear();
    document.documentElement.classList.remove("dark");
    delete window.inventoryDesktop;
    vi.spyOn(document, "hasFocus").mockReturnValue(true);
  });

  it("activates, subscribes, loads locally, then deactivates without syncing when disabled", async () => {
    const callOrder: string[] = [];
    const unsubscribe = vi.fn(() => callOrder.push("unsubscribe"));
    const activateInventorySync = vi.fn(async () => {
      callOrder.push("activate");
      return "session-disabled";
    });
    const deactivateInventorySync = vi.fn(async () => {
      callOrder.push("deactivate");
      return true;
    });
    const loadInventory = vi.fn(async () => {
      callOrder.push("load");
      return buildDesktopSyncResult(DISABLED_SHARED_STATUS);
    });
    const onSharedInventoryChanged = vi.fn(() => {
      callOrder.push("subscribe");
      return unsubscribe;
    });
    const syncInventory = vi.fn().mockResolvedValue({
      dbPath: TEST_DB_PATH,
      entries: [],
      entriesChanged: false,
      shared: DISABLED_SHARED_STATUS,
    });

    window.inventoryDesktop = createDesktopBridge({
      activateInventorySync,
      deactivateInventorySync,
      loadInventory,
      onSharedInventoryChanged,
      syncInventory,
    });

    render(<InventoryShell />);

    expect(await screen.findByText("Showing all 0 entries")).toBeInTheDocument();
    await waitFor(() => expect(deactivateInventorySync).toHaveBeenCalledWith("session-disabled"));
    expect(callOrder).toEqual(["activate", "subscribe", "load", "unsubscribe", "deactivate"]);
    expect(syncInventory).not.toHaveBeenCalled();
  });

  it("launches selected TE in activate, subscribe, load, initial sync order", async () => {
    const callOrder: string[] = [];
    const activateInventorySync = vi.fn(async () => {
      callOrder.push("activate");
      return "session-active";
    });
    const onSharedInventoryChanged = vi.fn(() => {
      callOrder.push("subscribe");
      return () => undefined;
    });
    const loadInventory = vi.fn(async () => {
      callOrder.push("load");
      return buildDesktopSyncResult(CONNECTED_SHARED_STATUS);
    });
    const syncInventory = vi.fn(async (sessionId: string) => {
      callOrder.push(`sync:${sessionId}`);
      return {
        dbPath: TEST_DB_PATH,
        entries: [],
        entriesChanged: false,
        shared: CONNECTED_SHARED_STATUS,
      };
    });

    window.inventoryDesktop = createDesktopBridge({
      activateInventorySync,
      loadInventory,
      onSharedInventoryChanged,
      syncInventory,
    });

    render(<InventoryShell />);

    expect(await screen.findByText("Showing all 0 entries")).toBeInTheDocument();
    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(1));
    expect(callOrder).toEqual(["activate", "subscribe", "load", "sync:session-active"]);
  });

  it("subscribes to scoped shared changes and syncs without reloading local rows", async () => {
    let sharedChangeCallback: ((payload: InventorySharedChangedPayload) => void) | null = null;
    const unsubscribeSharedChanges = vi.fn();
    const deactivateInventorySync = vi.fn().mockResolvedValue(true);
    const loadInventory = vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS));
    const syncInventory = vi.fn().mockResolvedValue({
      dbPath: TEST_DB_PATH,
      entries: [],
      entriesChanged: false,
      shared: CONNECTED_SHARED_STATUS,
    });
    const onSharedInventoryChanged = vi.fn((callback: (payload: InventorySharedChangedPayload) => void) => {
      sharedChangeCallback = callback;
      return unsubscribeSharedChanges;
    });

    window.inventoryDesktop = createDesktopBridge({
      deactivateInventorySync,
      loadInventory,
      onSharedInventoryChanged,
      syncInventory,
    });

    const { unmount } = render(<InventoryShell />);

    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(1));
    expect(onSharedInventoryChanged).toHaveBeenCalledTimes(1);
    syncInventory.mockClear();

    act(() => {
      sharedChangeCallback?.({ systemId: "te-test-equipment" });
    });

    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(1));
    expect(syncInventory).toHaveBeenCalledWith("session-1");
    expect(loadInventory).toHaveBeenCalledTimes(1);

    unmount();
    expect(unsubscribeSharedChanges).toHaveBeenCalledTimes(1);
    expect(deactivateInventorySync).toHaveBeenCalledWith("session-1");
  });

  it("does not activate or load TE when a placeholder is selected at launch", async () => {
    localStorage.setItem("inventory.activeSystem", "me-storage");
    const activateInventorySync = vi.fn().mockResolvedValue("unused-session");
    const deactivateInventorySync = vi.fn().mockResolvedValue(true);
    const loadInventory = vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS));
    const onSharedInventoryChanged = vi.fn(() => () => undefined);
    const syncInventory = vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS));
    window.inventoryDesktop = createDesktopBridge({
      activateInventorySync,
      deactivateInventorySync,
      loadInventory,
      onSharedInventoryChanged,
      syncInventory,
    });

    render(<InventoryShell />);
    await flushAsyncWork();

    expect(screen.getByText(/ME Storage — module not connected yet/i)).toBeInTheDocument();
    expect(activateInventorySync).not.toHaveBeenCalled();
    expect(loadInventory).not.toHaveBeenCalled();
    expect(onSharedInventoryChanged).not.toHaveBeenCalled();
    expect(syncInventory).not.toHaveBeenCalled();
    expect(deactivateInventorySync).not.toHaveBeenCalled();
  });

  it("treats a nullable stale sync result as a no-op without entering error state", async () => {
    const entry = buildTestEntry({ description: "Stale null survives" });
    const syncInventory = vi.fn().mockResolvedValue(null);
    window.inventoryDesktop = createDesktopBridge({
      loadInventory: vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS, [entry])),
      syncInventory,
    });

    render(<InventoryShell />);

    expect(await screen.findByText("Stale null survives")).toBeInTheDocument();
    expect(syncInventory).toHaveBeenCalledWith("session-1");
    expect(screen.queryByText("Shared workspace unavailable. Saving changes locally.")).not.toBeInTheDocument();
  });

  it("syncs immediately when window focus is restored", async () => {
    const syncInventory = vi.fn().mockResolvedValue({
      dbPath: TEST_DB_PATH,
      entries: [],
      entriesChanged: false,
      shared: CONNECTED_SHARED_STATUS,
    });
    window.inventoryDesktop = createDesktopBridge({
      loadInventory: vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS)),
      syncInventory,
    });

    render(<InventoryShell />);
    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(1));
    syncInventory.mockClear();

    act(() => {
      window.dispatchEvent(new Event("blur"));
      window.dispatchEvent(new Event("focus"));
    });

    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(1));
    expect(syncInventory).toHaveBeenCalledWith("session-1");
  });

  it("deactivates a token that resolves after unmount without loading or syncing", async () => {
    const activation = createDeferred<string>();
    const activateInventorySync = vi.fn(() => activation.promise);
    const deactivateInventorySync = vi.fn().mockResolvedValue(true);
    const loadInventory = vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS));
    const syncInventory = vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS));
    window.inventoryDesktop = createDesktopBridge({
      activateInventorySync,
      deactivateInventorySync,
      loadInventory,
      syncInventory,
    });

    const { unmount } = render(<InventoryShell />);
    await waitFor(() => expect(activateInventorySync).toHaveBeenCalledTimes(1));
    unmount();

    await act(async () => {
      activation.resolve("late-session");
      await activation.promise;
      await Promise.resolve();
    });

    expect(deactivateInventorySync).toHaveBeenCalledWith("late-session");
    expect(loadInventory).not.toHaveBeenCalled();
    expect(syncInventory).not.toHaveBeenCalled();
  });

  it("keeps cached TE rows visible across placeholder switch and ignores the stale subscription", async () => {
    const user = userEvent.setup();
    const cachedEntry = buildTestEntry({ description: "Cached TE row" });
    const secondLoad = createDeferred<InventorySyncResult>();
    const activateInventorySync = vi.fn()
      .mockResolvedValueOnce("session-1")
      .mockResolvedValueOnce("session-2");
    const deactivateInventorySync = vi.fn().mockResolvedValue(true);
    const loadInventory = vi.fn()
      .mockResolvedValueOnce(buildDesktopSyncResult(CONNECTED_SHARED_STATUS, [cachedEntry]))
      .mockReturnValueOnce(secondLoad.promise);
    const sharedCallbacks: Array<(payload: InventorySharedChangedPayload) => void> = [];
    const unsubscribeCallbacks = [vi.fn(), vi.fn()];
    const onSharedInventoryChanged = vi.fn((callback: (payload: InventorySharedChangedPayload) => void) => {
      const index = sharedCallbacks.length;
      sharedCallbacks.push(callback);
      return unsubscribeCallbacks[index] ?? (() => undefined);
    });
    const syncInventory = vi.fn().mockResolvedValue({
      dbPath: TEST_DB_PATH,
      entries: [],
      entriesChanged: false,
      shared: CONNECTED_SHARED_STATUS,
    });
    window.inventoryDesktop = createDesktopBridge({
      activateInventorySync,
      deactivateInventorySync,
      loadInventory,
      onSharedInventoryChanged,
      syncInventory,
    });

    render(<InventoryShell />);
    expect(await screen.findByText("Cached TE row")).toBeInTheDocument();
    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(1));

    await switchInventory(user, "ME Storage");
    expect(screen.getByText(/ME Storage — module not connected yet/i)).toBeInTheDocument();
    await waitFor(() => expect(deactivateInventorySync).toHaveBeenCalledWith("session-1"));
    expect(unsubscribeCallbacks[0]).toHaveBeenCalledTimes(1);

    await switchInventory(user, "TE Test Equipment");
    expect(screen.getByText("Cached TE row")).toBeInTheDocument();
    await waitFor(() => expect(loadInventory).toHaveBeenCalledTimes(2));

    await act(async () => {
      secondLoad.resolve(buildDesktopSyncResult(CONNECTED_SHARED_STATUS, [cachedEntry]));
      await secondLoad.promise;
      await Promise.resolve();
    });
    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(2));
    syncInventory.mockClear();

    act(() => {
      sharedCallbacks[0]?.({ systemId: "te-test-equipment" });
      sharedCallbacks[1]?.({ systemId: "te-test-equipment" });
    });

    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(1));
    expect(syncInventory).toHaveBeenCalledWith("session-2");
    expect(activateInventorySync).toHaveBeenCalledTimes(2);
    expect(onSharedInventoryChanged).toHaveBeenCalledTimes(2);
  });

  it("ignores an in-flight sync result and does not rearm after TE is deselected", async () => {
    const user = userEvent.setup();
    const inFlightSync = createDeferred<InventorySyncResult | null>();
    let sharedChangeCallback: ((payload: InventorySharedChangedPayload) => void) | null = null;
    const deactivateInventorySync = vi.fn().mockResolvedValue(true);
    const syncInventory = vi.fn()
      .mockResolvedValueOnce({
        dbPath: TEST_DB_PATH,
        entries: [],
        entriesChanged: false,
        shared: CONNECTED_SHARED_STATUS,
      })
      .mockReturnValueOnce(inFlightSync.promise);
    window.inventoryDesktop = createDesktopBridge({
      deactivateInventorySync,
      loadInventory: vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS)),
      onSharedInventoryChanged: vi.fn((callback: (payload: InventorySharedChangedPayload) => void) => {
        sharedChangeCallback = callback;
        return () => undefined;
      }),
      syncInventory,
    });

    render(<InventoryShell />);
    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(1));
    act(() => {
      sharedChangeCallback?.({ systemId: "te-test-equipment" });
    });
    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(2));

    await switchInventory(user, "ME Storage");
    await waitFor(() => expect(deactivateInventorySync).toHaveBeenCalledWith("session-1"));

    await act(async () => {
      inFlightSync.resolve({
        dbPath: TEST_DB_PATH,
        entries: [buildTestEntry({ description: "Late inactive result" })],
        entriesChanged: true,
        shared: CONNECTED_SHARED_STATUS,
      });
      await inFlightSync.promise;
      sharedChangeCallback?.({ systemId: "te-test-equipment" });
      await Promise.resolve();
    });

    expect(syncInventory).toHaveBeenCalledTimes(2);
    expect(screen.getByText(/ME Storage — module not connected yet/i)).toBeInTheDocument();
  });

  it("cancels the 75 ms mutation sync when TE is deselected", async () => {
    vi.useFakeTimers();
    try {
      const entry = buildTestEntry({ description: "Deselect mutation row" });
      const syncInventory = vi.fn().mockResolvedValue({
        dbPath: TEST_DB_PATH,
        entries: [],
        entriesChanged: false,
        shared: CONNECTED_SHARED_STATUS,
      });
      const deactivateInventorySync = vi.fn().mockResolvedValue(true);
      const toggleVerifiedEntry = vi.fn().mockResolvedValue({
        entry: { ...entry, verifiedAt: "2026-07-20T12:00:00Z" },
        message: "Verified state updated.",
        mutationMode: "local",
        shared: CONNECTED_SHARED_STATUS,
      });
      window.inventoryDesktop = createDesktopBridge({
        deactivateInventorySync,
        loadInventory: vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS, [entry])),
        syncInventory,
        toggleVerifiedEntry,
      });

      render(<InventoryShell />);
      await flushAsyncWork();
      await flushAsyncWork();
      expect(screen.getByText("Deselect mutation row")).toBeInTheDocument();
      syncInventory.mockClear();

      fireEvent.click(screen.getByRole("button", { name: /Verify Deselect mutation row/i }));
      await flushAsyncWork();
      fireEvent.click(screen.getByRole("button", { name: "Switch inventory system" }));
      fireEvent.click(screen.getByRole("option", { name: "ME Storage" }));
      await flushAsyncWork();
      await vi.advanceTimersByTimeAsync(75);

      expect(syncInventory).not.toHaveBeenCalled();
      expect(deactivateInventorySync).toHaveBeenCalledWith("session-1");
    } finally {
      vi.useRealTimers();
    }
  });

  it("keeps startup loading until the initial shared sync finishes", async () => {
    const staleLocalEntry = buildTestEntry({
      id: "901",
      description: "Stale local startup entry",
      manufacturer: "Local Only Maker",
    });
    const syncedEntry = buildTestEntry({
      id: "902",
      description: "Shared startup entry",
      manufacturer: "Shared Maker",
    });
    const initialSync = createDeferred<InventorySyncResult>();
    const loadInventory = vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS, [staleLocalEntry]));
    const syncInventory = vi.fn().mockReturnValue(initialSync.promise);

    window.inventoryDesktop = createDesktopBridge({ loadInventory, syncInventory });

    render(<InventoryShell />);

    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(1));
    expect(screen.getByText("Loading inventory entries...")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Inventory (0)" })).toBeInTheDocument();
    expect(screen.queryByText("Local Only Maker")).not.toBeInTheDocument();

    await act(async () => {
      initialSync.resolve({
        dbPath: TEST_DB_PATH,
        entries: [syncedEntry],
        entriesChanged: true,
        shared: CONNECTED_SHARED_STATUS,
      });
      await initialSync.promise;
      await Promise.resolve();
    });

    expect(await screen.findByText("Shared Maker")).toBeInTheDocument();
    expect(screen.queryByText("Local Only Maker")).not.toBeInTheDocument();
  });

  it("pushes desktop mutations to shared sync almost immediately", async () => {
    const user = userEvent.setup();
    const entry = buildTestEntry({ description: "Delayed sync entry" });
    const syncInventory = vi.fn().mockResolvedValue({
      dbPath: TEST_DB_PATH,
      entries: [],
      entriesChanged: false,
      shared: CONNECTED_SHARED_STATUS,
    });
    const toggleVerifiedEntry = vi
      .fn()
      .mockResolvedValueOnce({
        entry: { ...entry, verifiedAt: "2026-07-13T12:00:00Z" },
        message: "Verified state updated.",
        mutationMode: "local",
        shared: CONNECTED_SHARED_STATUS,
      })
      .mockResolvedValueOnce({
        entry,
        message: "Verified state updated.",
        mutationMode: "local",
        shared: CONNECTED_SHARED_STATUS,
      });

    window.inventoryDesktop = createDesktopBridge({
      loadInventory: vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS, [entry])),
      syncInventory,
      toggleVerifiedEntry,
    });

    render(<InventoryShell />);

    expect(await screen.findByText("Delayed sync entry")).toBeInTheDocument();
    await flushAsyncWork();
    syncInventory.mockClear();

    const toggleButton = screen.getByRole("button", { name: /Verify Delayed sync entry/i });
    await user.click(toggleButton);
    await user.click(toggleButton);

    expect(toggleVerifiedEntry).toHaveBeenCalledTimes(2);

    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(1));
    await delay(100);
    expect(syncInventory).toHaveBeenCalledTimes(1);
  });

  it("runs one follow-up sync when a shared change arrives during an in-flight sync", async () => {
    const firstSync = createDeferred<Awaited<ReturnType<NonNullable<Window["inventoryDesktop"]>["syncInventory"]>>>();
    let sharedChangeCallback: ((payload: { systemId: "te-test-equipment" }) => void) | null = null;
    const entry = buildTestEntry({ description: "In-flight sync entry" });
    const syncInventory = vi
      .fn()
      .mockReturnValueOnce(firstSync.promise)
      .mockResolvedValue({
        dbPath: TEST_DB_PATH,
        entries: [],
        entriesChanged: false,
        shared: CONNECTED_SHARED_STATUS,
      });

    window.inventoryDesktop = createDesktopBridge({
      onSharedInventoryChanged: vi.fn((callback: (payload: { systemId: "te-test-equipment" }) => void) => {
        sharedChangeCallback = callback;
        return () => undefined;
      }),
      loadInventory: vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS, [entry])),
      syncInventory,
    });

    render(<InventoryShell />);

    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(sharedChangeCallback).not.toBeNull());
    expect(screen.getByText("Loading inventory entries...")).toBeInTheDocument();
    expect(screen.queryByText("In-flight sync entry")).not.toBeInTheDocument();

    act(() => {
      sharedChangeCallback?.({ systemId: "te-test-equipment" });
      sharedChangeCallback?.({ systemId: "te-test-equipment" });
    });
    expect(syncInventory).toHaveBeenCalledTimes(1);

    await act(async () => {
      firstSync.resolve({
        dbPath: TEST_DB_PATH,
        entries: [],
        entriesChanged: false,
        shared: CONNECTED_SHARED_STATUS,
      });
      await firstSync.promise;
      await Promise.resolve();
    });

    expect(await screen.findByText("In-flight sync entry")).toBeInTheDocument();
    await waitFor(() => expect(syncInventory).toHaveBeenCalledTimes(2));
  });

  it("cleans up delayed mutation sync timers on unmount", async () => {
    const user = userEvent.setup();
    const entry = buildTestEntry({ description: "Unmount sync entry" });
    const syncInventory = vi.fn().mockResolvedValue({
      dbPath: TEST_DB_PATH,
      entries: [],
      entriesChanged: false,
      shared: CONNECTED_SHARED_STATUS,
    });

    window.inventoryDesktop = createDesktopBridge({
      loadInventory: vi.fn().mockResolvedValue(buildDesktopSyncResult(CONNECTED_SHARED_STATUS, [entry])),
      syncInventory,
      toggleVerifiedEntry: vi.fn().mockResolvedValue({
        entry: { ...entry, verifiedAt: "2026-07-13T12:00:00Z" },
        message: "Verified state updated.",
        mutationMode: "local",
        shared: CONNECTED_SHARED_STATUS,
      }),
    });

    const { unmount } = render(<InventoryShell />);

    expect(await screen.findByText("Unmount sync entry")).toBeInTheDocument();
    await flushAsyncWork();
    syncInventory.mockClear();

    await user.click(screen.getByRole("button", { name: /Verify Unmount sync entry/i }));
    unmount();

    await delay(50);

    expect(syncInventory).not.toHaveBeenCalled();
  });

  it("does not start initial sync after a desktop load resolves post-unmount", async () => {
    const deferredLoad = createDeferred<InventorySyncResult>();
    const loadInventory = vi.fn(() => deferredLoad.promise);
    const syncInventory = vi.fn().mockResolvedValue({
      dbPath: TEST_DB_PATH,
      entries: [],
      entriesChanged: false,
      shared: CONNECTED_SHARED_STATUS,
    });
    window.inventoryDesktop = createDesktopBridge({ loadInventory, syncInventory });

    const { unmount } = render(<InventoryShell />);
    await waitFor(() => expect(loadInventory).toHaveBeenCalled());

    unmount();
    await act(async () => {
      deferredLoad.resolve(buildDesktopSyncResult(CONNECTED_SHARED_STATUS));
      await deferredLoad.promise;
    });

    expect(syncInventory).not.toHaveBeenCalled();
  });

  it("keeps current rows when desktop sync reports no entry changes", async () => {
    const desktopEntries: InventoryEntry[] = [
      buildTestEntry({
        id: "301",
        assetNumber: "ME-301",
        qty: 1,
        manufacturer: "Stable Maker",
        model: "SM-1",
        description: "Stable entry",
        projectName: "Sync",
        location: "Bench",
        links: "",
        notes: "",
        lifecycleStatus: "active",
        workingStatus: "working",
        verifiedAt: "2026-07-13T12:00:00Z",
        archived: false,
        updatedAt: "2026-04-23 10:00:00",
      }),
    ];

    window.inventoryDesktop = createDesktopBridge({
      loadInventory: vi.fn().mockResolvedValue({
        dbPath: TEST_DB_PATH,
        entries: desktopEntries,
        shared: CONNECTED_SHARED_STATUS,
      }),
      syncInventory: vi.fn().mockResolvedValue({
        dbPath: TEST_DB_PATH,
        entries: [{ ...desktopEntries[0], manufacturer: "Replacement Maker" }],
        entriesChanged: false,
        shared: CONNECTED_SHARED_STATUS,
      }),
      toggleVerifiedEntry: vi.fn().mockResolvedValue(desktopEntries[0]),
      createEntry: vi.fn().mockResolvedValue(desktopEntries[0]),
      updateEntry: vi.fn().mockResolvedValue(desktopEntries[0]),
      setArchivedEntry: vi.fn().mockResolvedValue(desktopEntries[0]),
      deleteEntry: vi.fn().mockResolvedValue({ entryId: desktopEntries[0].id }),
      openExternal: vi.fn().mockResolvedValue(true),
      openPath: vi.fn().mockResolvedValue(true),
      pickPicturePath: vi.fn().mockResolvedValue(null),
      exportExcel: vi.fn().mockResolvedValue({ canceled: false, outputPath: "D:/exports/TE_Test_Equipment_Inventory_Export.xlsx" }),
    });

    render(<InventoryShell />);

    expect(await screen.findByText("Stable Maker")).toBeInTheDocument();
    await waitFor(() => expect(window.inventoryDesktop?.syncInventory).toHaveBeenCalled());
    expect(screen.getByText("Stable Maker")).toBeInTheDocument();
    expect(screen.queryByText("Replacement Maker")).not.toBeInTheDocument();
  });
});

async function switchInventory(
  user: ReturnType<typeof userEvent.setup>,
  label: "ME Storage" | "TE Test Equipment",
): Promise<void> {
  await user.click(screen.getByRole("button", { name: "Switch inventory system" }));
  await user.click(screen.getByRole("option", { name: label }));
}
