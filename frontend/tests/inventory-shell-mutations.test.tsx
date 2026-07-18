import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { InventoryShell } from "@/features/inventory/components/InventoryShell";
import type { InventoryEntry } from "@/features/inventory/types";
import {
  CONNECTED_SHARED_STATUS,
  LOCAL_SHARED_STATUS,
  TEST_DB_PATH,
  buildTestEntry,
  createDesktopBridge,
} from "./inventory-shell/helpers";

describe("InventoryShell mutations", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove("dark");
    delete window.inventoryDesktop;
  });

  it("keeps desktop editing enabled when mutations are local-only", async () => {
    const user = userEvent.setup();
    let desktopEntries: InventoryEntry[] = [
      buildTestEntry({
        id: "201",
        assetNumber: "ME-201",
        qty: 1,
        manufacturer: "Offline Maker",
        model: "OM-1",
        description: "Offline editable entry",
        projectName: "Local Work",
        location: "Bench 1",
        links: "",
        notes: "",
        lifecycleStatus: "active",
        workingStatus: "working",
        archived: false,
        updatedAt: "2026-04-23 10:00:00",
      }),
    ];
    const createdEntry: InventoryEntry = {
      ...desktopEntries[0],
      id: "202",
      assetNumber: "",
      description: "Local-only saved entry",
      manufacturer: "Local Maker",
    };
    const createEntry = vi.fn().mockImplementation(async () => {
      desktopEntries = [createdEntry, ...desktopEntries];
      return {
        entry: createdEntry,
        message: "Entry added locally.",
        mutationMode: "local",
        shared: LOCAL_SHARED_STATUS,
      };
    });

    window.inventoryDesktop = {
      isDesktop: true,
      loadInventory: vi.fn().mockImplementation(async () => ({
        dbPath: TEST_DB_PATH,
        entries: desktopEntries,
        shared: LOCAL_SHARED_STATUS,
      })),
      syncInventory: vi.fn().mockImplementation(async () => ({
        dbPath: TEST_DB_PATH,
        entries: desktopEntries,
        shared: LOCAL_SHARED_STATUS,
      })),
      toggleVerifiedEntry: vi.fn().mockImplementation(async (entryId: string, nextVerified: boolean) => {
        const currentEntry = desktopEntries.find((entry) => entry.id === entryId)!;
        const updatedEntry = {
          ...currentEntry,
          verifiedAt: nextVerified ? "2026-07-13T12:00:00Z" : undefined,
          verifiedBy: nextVerified ? currentEntry.verifiedBy : undefined,
        };
        desktopEntries = desktopEntries.map((entry) => (entry.id === entryId ? updatedEntry : entry));
        return {
          entry: updatedEntry,
          message: "Verified state updated locally.",
          mutationMode: "local",
          shared: LOCAL_SHARED_STATUS,
        };
      }),
      createEntry,
      updateEntry: vi.fn().mockResolvedValue(desktopEntries[0]),
      setArchivedEntry: vi.fn().mockResolvedValue(desktopEntries[0]),
      deleteEntry: vi.fn().mockResolvedValue({ entryId: desktopEntries[0].id }),
      openExternal: vi.fn().mockResolvedValue(true),
      openPath: vi.fn().mockResolvedValue(true),
      pickPicturePath: vi.fn().mockResolvedValue(null),
      pickImportFile: vi.fn().mockResolvedValue(null),
      previewImport: vi.fn(),
      commitImport: vi.fn(),
      exportExcel: vi.fn().mockResolvedValue({ canceled: false, outputPath: "D:/exports/TE_Test_Equipment_Inventory_Export.xlsx" }),
    };

    render(<InventoryShell />);

    expect(await screen.findByText(/Shared workspace unavailable\. Saving changes locally\./)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add Entry" })).toBeEnabled();

    await user.click(screen.getByRole("button", { name: "Add Entry" }));
    const manufacturerInput = screen.getByLabelText("Manufacturer / Brand");
    await user.type(manufacturerInput, "Local Maker");
    await user.type(screen.getByLabelText("Description"), "Local-only saved entry");

    expect(manufacturerInput).toHaveValue("Local Maker");

    await user.click(screen.getByRole("button", { name: "Save Entry" }));

    expect(createEntry).toHaveBeenCalledTimes(1);
    expect(await screen.findByText("Entry added locally.")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: /Verify Local-only saved entry/i }));
    expect(await screen.findByText("Verified state updated locally.")).toBeInTheDocument();
  });

  it("uses the mutation result message when shared status changes during a save", async () => {
    const user = userEvent.setup();
    const existingEntry: InventoryEntry = buildTestEntry({
      id: "601",
      assetNumber: "ME-601",
      qty: 1,
      manufacturer: "Connected Maker",
      model: "CM-1",
      description: "Connected entry",
      projectName: "Shared",
      location: "Bench 1",
      links: "",
      notes: "",
      lifecycleStatus: "active",
      workingStatus: "working",
      archived: false,
      updatedAt: "2026-04-23 10:00:00",
    });
    const createdEntry: InventoryEntry = {
      ...existingEntry,
      id: "602",
      description: "Saved while shared vanished",
      manufacturer: "Local Maker",
    };

    window.inventoryDesktop = {
      isDesktop: true,
      loadInventory: vi.fn().mockResolvedValue({
        dbPath: TEST_DB_PATH,
        entries: [existingEntry],
        shared: CONNECTED_SHARED_STATUS,
      }),
      syncInventory: vi.fn().mockResolvedValue({
        dbPath: TEST_DB_PATH,
        entries: [existingEntry],
        entriesChanged: false,
        shared: CONNECTED_SHARED_STATUS,
      }),
      toggleVerifiedEntry: vi.fn(),
      createEntry: vi.fn().mockResolvedValue({
        entry: createdEntry,
        message: "Entry added locally.",
        mutationMode: "local",
        shared: LOCAL_SHARED_STATUS,
      }),
      updateEntry: vi.fn(),
      setArchivedEntry: vi.fn(),
      deleteEntry: vi.fn(),
      openExternal: vi.fn().mockResolvedValue(true),
      openPath: vi.fn().mockResolvedValue(true),
      pickPicturePath: vi.fn().mockResolvedValue(null),
      pickImportFile: vi.fn().mockResolvedValue(null),
      previewImport: vi.fn(),
      commitImport: vi.fn(),
      exportExcel: vi.fn().mockResolvedValue({ canceled: false, outputPath: "D:/exports/TE_Test_Equipment_Inventory_Export.xlsx" }),
    };

    render(<InventoryShell />);

    expect(await screen.findByText("Connected Maker")).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Add Entry" }));
    await user.type(screen.getByLabelText("Manufacturer / Brand"), "Local Maker");
    await user.type(screen.getByLabelText("Description"), "Saved while shared vanished");
    await user.click(screen.getByRole("button", { name: "Save Entry" }));

    expect(await screen.findByText("Entry added locally.")).toBeInTheDocument();
    expect(screen.queryByText("Entry added to the TE Test Equipment Inventory database.")).not.toBeInTheDocument();
  });

  it("passes the next verification boolean based only on verifiedAt", async () => {
    const user = userEvent.setup();
    let entry = buildTestEntry({ id: "verify-1", description: "Timestamp meter", verifiedAt: undefined });
    const toggleVerifiedEntry = vi.fn(async (_entryId: string, nextVerified: boolean) => {
      entry = { ...entry, verifiedAt: nextVerified ? "2026-07-13T12:00:00Z" : undefined, verifiedBy: nextVerified ? "Avery" : undefined };
      return { entry, message: "Verification updated.", mutationMode: "local" as const };
    });
    window.inventoryDesktop = createDesktopBridge({
      loadInventory: vi.fn().mockResolvedValue({ dbPath: TEST_DB_PATH, entries: [entry], shared: LOCAL_SHARED_STATUS }),
      toggleVerifiedEntry,
    });
    render(<InventoryShell />);

    await user.click(await screen.findByRole("button", { name: /Verify Timestamp meter/i }));
    expect(toggleVerifiedEntry).toHaveBeenLastCalledWith("verify-1", true);
    await user.click(screen.getByRole("button", { name: /Clear verification for Timestamp meter/i }));
    expect(toggleVerifiedEntry).toHaveBeenLastCalledWith("verify-1", false);
  });

  it("sets a local RFC3339 verification timestamp and clears timestamp plus verifier", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await user.click(screen.getByRole("button", { name: /Verify Long handle ratchet/i }));
    expect(screen.getByRole("button", { name: /Clear verification for Long handle ratchet, verified \d{4}-\d{2}-\d{2}T/i })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: /Clear verification for Long handle ratchet/i }));
    expect(screen.getByRole("button", { name: /Verify Long handle ratchet/i })).toBeInTheDocument();
  });
});
