import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { InventoryShell } from "@/shell/InventoryShell";
import { createDesktopBridge } from "./inventory-shell/helpers";

describe("TE Test Equipment calibration table", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove("dark");
    delete window.inventoryDesktop;
  });

  it("selects the calibration table from the inventory title menu", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    expect(screen.getByRole("columnheader", { name: /Calibration due/i })).toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: /Out to cal/i })).not.toBeInTheDocument();

    await switchTeTable(user, "TE Test Equipment Calibration");

    expect(screen.getByRole("heading", { name: "TE Test Equipment Calibration" })).toBeInTheDocument();
    expect(localStorage.getItem("inventory.activeSystem")).toBe("te-test-equipment-calibration");
    expect(screen.queryByRole("button", { name: "Equipment" })).not.toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Calibration" })).not.toBeInTheDocument();
    expect(screen.getByText("Showing all 6 calibration equipment")).toBeInTheDocument();
    expect(screen.getByText("Digital caliper, 6 inch")).toBeInTheDocument();
    expect(screen.queryByText("Deburring wheel assortment")).not.toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: /^Last$/i })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: /^Due$/i })).toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: /^Health$/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: /Out to cal/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: /Certificate/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: /Vendor/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: /Serial #/i })).not.toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: /Qty/i })).not.toBeInTheDocument();
    expect(screen.queryByText("Tracked: 6")).not.toBeInTheDocument();
    expect(screen.queryByText("Results: 6")).not.toBeInTheDocument();
    expect(screen.getByText(/Verified:/)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add Equipment" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Initialize from Calibration Workbook" })).not.toBeInTheDocument();
  });

  it("preserves independent searches when switching tables", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await user.type(screen.getByLabelText("Inventory search"), "multimeter");
    expect(screen.getByText('1 result for "multimeter"')).toBeInTheDocument();

    await switchTeTable(user, "TE Test Equipment Calibration");
    expect(screen.getByLabelText("Calibration search")).toHaveValue("");
    await user.type(screen.getByLabelText("Calibration search"), "caliper");
    expect(screen.getByText('1 calibration results for "caliper"')).toBeInTheDocument();

    await switchTeTable(user, "TE Test Equipment");
    expect(screen.getByLabelText("Inventory search")).toHaveValue("multimeter");
    expect(screen.getByText("Industrial multimeter")).toBeInTheDocument();

    await switchTeTable(user, "TE Test Equipment Calibration");
    expect(screen.getByLabelText("Calibration search")).toHaveValue("caliper");
  });

  it("preserves independent filters, sorts, and columns while allowing reference-only opt-in", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await user.click(screen.getByRole("button", { name: "Sort by Model" }));
    fireEvent.contextMenu(screen.getAllByRole("columnheader")[0]);
    await user.click(await screen.findByRole("checkbox", { name: "Links" }));
    await user.click(screen.getByRole("button", { name: "Show filters" }));
    await user.type(screen.getByLabelText("Filter manufacturer"), "Fluke");
    expect(await screen.findByText("Industrial multimeter")).toBeInTheDocument();

    await switchTeTable(user, "TE Test Equipment Calibration");
    expect(screen.queryByLabelText("Filter manufacturer")).not.toBeInTheDocument();
    expect(screen.getByRole("button", {
      name: "Sort by Due, currently ascending. Activate for descending",
    })).toBeInTheDocument();
    fireEvent.contextMenu(screen.getAllByRole("columnheader")[0]);
    // Opt in to an optional column; defaults stay 1080p-friendly (no Vendor/Certificate/Serial).
    await user.click(await screen.findByRole("checkbox", { name: "Vendor" }));
    await user.click(screen.getByRole("button", { name: "Show filters" }));
    await user.click(screen.getByRole("button", { name: "Calibration requirement" }));
    await user.click(screen.getByRole("option", { name: "Reference only" }));
    expect(await screen.findByText("Deburring wheel assortment")).toBeInTheDocument();
    expect(screen.queryByText("Digital caliper, 6 inch")).not.toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: /Vendor/i })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Sort by Asset #" }));

    await switchTeTable(user, "TE Test Equipment");
    expect(screen.getByLabelText("Filter manufacturer")).toHaveValue("Fluke");
    expect(screen.queryByRole("columnheader", { name: /Links/i })).not.toBeInTheDocument();
    expect(screen.getByRole("button", {
      name: "Sort by Model, currently ascending. Activate for descending",
    })).toBeInTheDocument();

    await switchTeTable(user, "TE Test Equipment Calibration");
    expect(screen.getByRole("button", { name: "Calibration requirement" })).toHaveTextContent("Reference only");
    expect(screen.getByRole("columnheader", { name: /Vendor/i })).toBeInTheDocument();
    expect(screen.getByRole("button", {
      name: "Sort by Asset #, currently ascending. Activate for descending",
    })).toBeInTheDocument();
  });

  it("does not show roster initialization in the calibration workspace", async () => {
    const user = userEvent.setup();
    window.inventoryDesktop = createDesktopBridge({
      pickCalibrationRosterFile: vi.fn().mockResolvedValue(null),
      previewCalibrationRoster: vi.fn(),
      commitCalibrationRoster: vi.fn(),
    });

    render(<InventoryShell />);

    expect(await screen.findByText("Showing all 0 entries")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Initialize from Calibration Workbook" })).not.toBeInTheDocument();
    await switchTeTable(user, "TE Test Equipment Calibration");
    expect(screen.queryByRole("button", { name: "Initialize from Calibration Workbook" })).not.toBeInTheDocument();
  });

  it("edits the same equipment record from Calibration and reads the change from Equipment", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await switchTeTable(user, "TE Test Equipment Calibration");
    await user.dblClick(screen.getByText("Digital caliper, 6 inch"));

    const editorSections = screen.getByLabelText("Entry editor section");
    expect(within(editorSections).getByRole("button", { name: "Calibration" })).toHaveAttribute("aria-pressed", "true");
    await user.type(screen.getByLabelText("Calibration vendor"), "MetroLab");
    await user.click(screen.getByRole("button", { name: "Save Entry" }));

    await switchTeTable(user, "TE Test Equipment");
    await user.dblClick(screen.getByText("Digital caliper, 6 inch"));
    expect(within(screen.getByLabelText("Entry editor section")).getByRole("button", { name: "Equipment" })).toHaveAttribute("aria-pressed", "true");
    await user.click(within(screen.getByLabelText("Entry editor section")).getByRole("button", { name: "Calibration" }));
    expect(screen.getByLabelText("Calibration vendor")).toHaveValue("MetroLab");
  });

  it("adds and removes calibration membership with explicit semantics", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    fireEvent.contextMenu(screen.getByText("Quick disconnect coupling set"));
    await user.click(screen.getByRole("menuitem", { name: "Add to Calibration" }));
    await switchTeTable(user, "TE Test Equipment Calibration");
    expect(screen.getByText("Quick disconnect coupling set")).toBeInTheDocument();

    fireEvent.contextMenu(screen.getByText("Quick disconnect coupling set"));
    await user.click(screen.getByRole("menuitem", { name: "Remove from Calibration" }));
    expect(screen.getByRole("button", { name: "Requirement after removal" })).toHaveTextContent(/Unknown/i);
    await user.click(screen.getByRole("button", { name: "Remove from Calibration" }));
    expect(screen.queryByText("Quick disconnect coupling set")).not.toBeInTheDocument();
  });

  it("adds existing equipment through the picker and preselects required for new equipment", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await switchTeTable(user, "TE Test Equipment Calibration");
    await user.click(screen.getByRole("button", { name: "Add Equipment" }));
    await user.type(screen.getByLabelText("Search equipment to add"), "Deburring");
    await user.click(screen.getByRole("checkbox", { name: "Select Deburring wheel assortment" }));
    await user.click(screen.getByRole("button", { name: "Add Selected (1)" }));
    expect(screen.getByText("Deburring wheel assortment")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Add Equipment" }));
    await user.click(screen.getByRole("button", { name: "Create New Equipment" }));
    await user.click(within(screen.getByLabelText("Entry editor section")).getByRole("button", { name: "Calibration" }));
    expect(screen.getByRole("button", { name: "Calibration requirement" })).toHaveTextContent(/Required/i);
    expect(within(screen.getByRole("dialog")).getByText("Pending")).toBeInTheDocument();
  });

  it("restores the calibration table from the stored selection", () => {
    localStorage.setItem("inventory.activeSystem", "te-test-equipment-calibration");

    render(<InventoryShell />);

    expect(screen.getByRole("heading", { name: "TE Test Equipment Calibration" })).toBeInTheDocument();
    expect(screen.getByLabelText("Calibration search")).toBeInTheDocument();
    expect(screen.getByText("Showing all 6 calibration equipment")).toBeInTheDocument();
  });
});

async function switchTeTable(
  user: ReturnType<typeof userEvent.setup>,
  label: "TE Test Equipment" | "TE Test Equipment Calibration",
): Promise<void> {
  await user.click(screen.getByRole("button", { name: "Switch inventory system" }));
  await user.click(screen.getByRole("option", { name: label }));
}
