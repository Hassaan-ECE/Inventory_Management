import { beforeEach, describe, expect, it, vi } from "vitest";
import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { InventoryShell } from "@/shell/InventoryShell";
import { TeStorageView } from "@/modules/te-storage/TeStorageView";
import { entryInput, type StorageBridge, type StorageEntry, type StorageSharedStatus, type StorageSnapshot } from "@/modules/te-storage/types";
import { parseStorageSnapshot } from "@/modules/te-storage/storageBridge";

const shared: StorageSharedStatus = { available: true, enabled: true, canModify: true, hasLocalOnlyChanges: false, message: "Shared sync ready.", mutationMode: "shared" };
const props = { active: true, activeViewId: "te-storage" as const, theme: "light" as const, onThemeToggle: vi.fn(), onViewChange: vi.fn() };
function testEntry(overrides: Partial<StorageEntry> = {}): StorageEntry {
  return { ...entryInput(), id: "1", entryUuid: "storage-1", pn: "00123", pr: "00045", po: "00987", qty: 2, location: "A1", createdAt: "2026-09-28T12:00:00Z", updatedAt: "2026-09-28T12:00:00Z", archived: false, ...overrides };
}
function testBridge(initial: StorageEntry[] = []): StorageBridge {
  let rows = initial;
  return {
    load: vi.fn(async () => ({ entries: rows, shared })),
    activate: vi.fn(async () => "room-session"), deactivate: vi.fn(async () => true),
    subscribe: vi.fn(async () => () => undefined), sync: vi.fn(async () => ({ entries: rows, shared })),
    save: vi.fn(async (input, original) => {
      const entry = { ...testEntry(), ...original, ...input };
      rows = [...rows.filter(row => row.entryUuid !== entry.entryUuid), entry];
      return { entry, shared, message: "Saved." };
    }),
    remove: vi.fn(async entry => { rows = rows.filter(row => row.entryUuid !== entry.entryUuid); return { shared, message: "Deleted." }; }),
  };
}

describe("TE Storage", () => {
  beforeEach(() => { localStorage.clear(); delete window.inventoryDesktop; });

  it("opens an empty inventory with optional columns hidden and remembers visibility", async () => {
    const user = userEvent.setup();
    const view = render(<InventoryShell />);
    await user.click(screen.getByRole("button", { name: "Switch inventory system" }));
    await user.click(screen.getByRole("option", { name: "TE Storage" }));
    expect(screen.getByRole("table", { name: "TE Storage inventory" })).toBeInTheDocument();
    expect(screen.getAllByRole("columnheader").map(cell => cell.textContent)).toEqual([
      "PN #", "PO #", "Manufacturer", "Model", "Qty", "Location", "Notes",
    ]);
    expect(screen.getByText("No storage items yet")).toBeInTheDocument();
    fireEvent.contextMenu(screen.getByRole("columnheader", { name: "PN #" }), { clientX: 100, clientY: 150 });
    await user.click(screen.getByRole("checkbox", { name: "PR #" }));
    await user.click(screen.getByRole("checkbox", { name: "Description" }));
    await user.keyboard("{Escape}");
    expect(screen.queryByRole("menu")).not.toBeInTheDocument();
    expect(screen.getAllByRole("columnheader").map(cell => cell.textContent)).toEqual([
      "PN #", "PR #", "PO #", "Manufacturer", "Model", "Description", "Qty", "Location", "Notes",
    ]);
    view.unmount();
    render(<InventoryShell />);
    expect(screen.getByRole("columnheader", { name: "PR #" })).toBeInTheDocument();
  });

  it("adds, edits and deletes while preserving hidden identifiers", async () => {
    const user = userEvent.setup(); const bridge = testBridge();
    render(<TeStorageView {...props} bridge={bridge} />);
    await waitFor(() => expect(screen.getByRole("button", { name: "Add Entry" })).toBeEnabled());
    await user.click(screen.getByRole("button", { name: "Add Entry" }));
    await user.type(screen.getByRole("textbox", { name: "PN #" }), "00123");
    await user.type(screen.getByRole("textbox", { name: "PR #" }), "00045");
    await user.type(screen.getByRole("textbox", { name: "PO #" }), "00987");
    await user.type(screen.getByRole("spinbutton", { name: "Qty" }), "2");
    await user.click(screen.getByRole("button", { name: "Save item" }));
    expect(await screen.findByRole("button", { name: "Edit item 00123" })).toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: "PR #" })).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Edit item 00123" }));
    expect(screen.getByRole("textbox", { name: "PR #" })).toHaveValue("00045");
    await user.type(screen.getByRole("textbox", { name: "Location" }), "B4");
    await user.click(screen.getByRole("button", { name: "Save item" }));
    expect(await screen.findByText("B4")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Edit item 00123" }));
    await user.click(screen.getByRole("button", { name: "Delete item" }));
    expect(bridge.remove).not.toHaveBeenCalled();
    await user.click(screen.getByRole("button", { name: "Confirm delete" }));
    expect(await screen.findByText("No storage items yet")).toBeInTheDocument();
  });

  it("retains form data after a failed save and prompts before discarding it", async () => {
    const user = userEvent.setup(); const bridge = testBridge();
    bridge.save = vi.fn().mockRejectedValue(new Error("Disk is full"));
    render(<TeStorageView {...props} bridge={bridge} />);
    await waitFor(() => expect(screen.getByRole("button", { name: "Add Entry" })).toBeEnabled());
    await user.click(screen.getByRole("button", { name: "Add Entry" }));
    await user.type(screen.getByRole("textbox", { name: "PN #" }), "Keep me");
    await user.click(screen.getByRole("button", { name: "Save item" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Disk is full");
    expect(screen.getByRole("textbox", { name: "PN #" })).toHaveValue("Keep me");
    await user.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.getByRole("alertdialog")).toHaveTextContent("Unsaved changes");
    await user.click(screen.getByRole("button", { name: "Discard" }));
    expect(screen.queryByRole("dialog")).not.toBeInTheDocument();
  });

  it("does not enable saving or seed records after a load failure", async () => {
    const bridge = testBridge(); bridge.load = vi.fn().mockRejectedValue(new Error("Database unavailable"));
    render(<TeStorageView {...props} bridge={bridge} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("Database unavailable");
    expect(screen.getByRole("button", { name: "Add Entry" })).toBeDisabled();
    expect(screen.getAllByRole("row")).toHaveLength(2);
    expect(bridge.activate).not.toHaveBeenCalled();
  });

  it("searches hidden PR values and sorts quantities numerically", async () => {
    const user = userEvent.setup();
    const bridge = testBridge([testEntry({ pn: "Ten", qty: 10 }), testEntry({ id: "2", entryUuid: "storage-2", pn: "Two", pr: "SECRET-PR", qty: 2 })]);
    render(<TeStorageView {...props} bridge={bridge} />);
    await screen.findByText("Ten");
    await user.click(screen.getByRole("button", { name: "Qty" }));
    const table = screen.getByRole("table");
    expect(within(table).getAllByRole("row")[1]).toHaveTextContent("Two");
    await user.type(screen.getByRole("textbox", { name: "Search TE Storage" }), "SECRET-PR");
    expect(screen.getByText("Two")).toBeInTheDocument();
    expect(screen.queryByText("Ten")).not.toBeInTheDocument();
  });

  it("discards a late sync response after a local mutation and deactivates on switch", async () => {
    const user = userEvent.setup(); const bridge = testBridge([testEntry()]);
    let finishSync!: (value: StorageSnapshot) => void;
    bridge.sync = vi.fn().mockImplementationOnce(() => new Promise(resolve => { finishSync = resolve; })).mockResolvedValue(null);
    const view = render(<TeStorageView {...props} bridge={bridge} />);
    await waitFor(() => expect(bridge.sync).toHaveBeenCalled());
    await user.click(screen.getByRole("button", { name: "Edit item 00123" }));
    await user.clear(screen.getByRole("textbox", { name: "Location" }));
    await user.type(screen.getByRole("textbox", { name: "Location" }), "New location");
    await user.click(screen.getByRole("button", { name: "Save item" }));
    await screen.findByText("New location");
    await act(async () => finishSync({ entries: [testEntry()], shared }));
    expect(screen.getByText("New location")).toBeInTheDocument();
    view.rerender(<TeStorageView {...props} active={false} bridge={bridge} />);
    await waitFor(() => expect(bridge.deactivate).toHaveBeenCalledWith("room-session"));
    expect(screen.queryByRole("table")).not.toBeInTheDocument();
  });

  it("rejects malformed desktop rows instead of silently accepting another module's data", () => {
    expect(() => parseStorageSnapshot({ entries: [{ assetNumber: "wrong module" }], shared })).toThrow();
    expect(() => parseStorageSnapshot({ entries: [testEntry({ qty: -1 })], shared })).toThrow();
    expect(parseStorageSnapshot({ entries: [testEntry()], shared }).entries[0].pr).toBe("00045");
  });
});
