import { beforeEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { InventoryShell } from "@/features/inventory/components/InventoryShell";
import { THEME_STORAGE_KEY } from "@/features/inventory/components/shell/helpers";
import {
  CONNECTED_SHARED_STATUS,
  TEST_DB_PATH,
} from "./inventory-shell/helpers";

describe("InventoryShell views and exports", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove("dark");
    delete window.inventoryDesktop;
  });

  it("switches to archive view and updates the summary", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await user.click(screen.getAllByRole("button", { name: /Archive/i })[0]);

    expect(screen.getByText("Showing all 4 archived entries")).toBeInTheDocument();
    expect(
      screen.getByPlaceholderText("Search archived entries by asset, serial, maker, model, description, location, or notes"),
    ).toBeInTheDocument();
    expect(screen.getByText("Cabinet table saw")).toBeInTheDocument();
  });

  it("shows and clears the filter panel", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await user.click(screen.getByRole("button", { name: "View settings" }));
    await user.click(screen.getByRole("menuitem", { name: /Show filters/i }));
    const manufacturerFilter = screen.getByLabelText("Filter manufacturer");
    await user.type(manufacturerFilter, "Mitutoyo");

    expect(screen.getByText("Showing 1 filtered entries")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Clear Column Filters" }));
    expect(screen.getByText("Showing all 10 entries")).toBeInTheDocument();
  });

  it("shows the inventory empty-state CTA for unmatched searches", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await user.type(screen.getByLabelText("Inventory search"), "no-match-value");

    expect(screen.getByText('No results for "no-match-value"')).toBeInTheDocument();
    expect(
      screen.getByText("Try a broader search, clear the column filters, or add a new entry."),
    ).toBeInTheDocument();
    expect(screen.getAllByRole("button", { name: "Add Entry" }).length).toBeGreaterThan(0);
  });

  it("updates theme preference and shows mock verified feedback", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await user.click(screen.getAllByRole("button", { name: /Dark/i })[0]);
    expect(document.documentElement.classList.contains("dark")).toBe(true);
    expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");

    await user.click(screen.getByRole("button", { name: /Clear verification for Stainless socket-head cap screws/i }));
    expect(screen.getByText("Verified state updated locally.")).toBeInTheDocument();
  });

  it("shows the HTML export placeholder message", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await user.click(screen.getByRole("button", { name: /Export/i }));
    await user.click(screen.getByRole("menuitem", { name: "HTML" }));

    expect(screen.getByText("HTML export is not implemented yet.")).toBeInTheDocument();
  });

  it("runs desktop Excel export when available", async () => {
    const user = userEvent.setup();
    const exportExcel = vi.fn().mockResolvedValue({
      canceled: false,
      outputPath: "D:/exports/TE_Test_Equipment_Inventory_Export.xlsx",
    });

    window.inventoryDesktop = {
      isDesktop: true,
      loadInventory: vi.fn().mockResolvedValue({
        dbPath: TEST_DB_PATH,
        entries: [],
        shared: CONNECTED_SHARED_STATUS,
      }),
      syncInventory: vi.fn().mockResolvedValue({
        dbPath: TEST_DB_PATH,
        entries: [],
        shared: CONNECTED_SHARED_STATUS,
      }),
      toggleVerifiedEntry: vi.fn().mockResolvedValue(null),
      createEntry: vi.fn().mockResolvedValue(null),
      updateEntry: vi.fn().mockResolvedValue(null),
      setArchivedEntry: vi.fn().mockResolvedValue(null),
      deleteEntry: vi.fn().mockResolvedValue({ entryId: "0" }),
      openExternal: vi.fn().mockResolvedValue(true),
      openPath: vi.fn().mockResolvedValue(true),
      pickPicturePath: vi.fn().mockResolvedValue(null),
      pickImportFile: vi.fn().mockResolvedValue(null),
      previewImport: vi.fn(),
      commitImport: vi.fn(),
      exportExcel,
    };

    render(<InventoryShell />);

    await user.click(screen.getByRole("button", { name: /Export/i }));
    await user.click(screen.getByRole("menuitem", { name: "Excel" }));

    expect(exportExcel).toHaveBeenCalledTimes(1);
    expect(await screen.findByText("Excel export completed.")).toBeInTheDocument();
  });
});
