import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { InventoryShell } from "@/shell/InventoryShell";
import { createDesktopBridge } from "./inventory-shell/helpers";

describe("TE Test Equipment calibration workspace", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove("dark");
    delete window.inventoryDesktop;
  });

  it("shows active required equipment by default with calibration-specific columns and counts", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    expect(screen.getByRole("columnheader", { name: /Calibration due/i })).toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: /Out to cal/i })).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Calibration" }));

    expect(screen.getByText("Showing all 6 calibration equipment")).toBeInTheDocument();
    expect(screen.getByText("Digital caliper, 6 inch")).toBeInTheDocument();
    expect(screen.queryByText("Deburring wheel assortment")).not.toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: /Last calibrated/i })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: /Vendor/i })).toBeInTheDocument();
    expect(screen.getByRole("columnheader", { name: /Out to cal/i })).toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: /Qty/i })).not.toBeInTheDocument();
    expect(screen.getByText("Tracked: 6")).toBeInTheDocument();
    expect(screen.getByText("Results: 6")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Add Equipment" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Initialize from Calibration Workbook" })).not.toBeInTheDocument();
  });

  it("preserves independent searches when switching workspaces", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await user.type(screen.getByLabelText("Inventory search"), "multimeter");
    expect(screen.getByText('1 result for "multimeter"')).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Calibration" }));
    expect(screen.getByLabelText("Calibration search")).toHaveValue("");
    await user.type(screen.getByLabelText("Calibration search"), "caliper");
    expect(screen.getByText('1 calibration results for "caliper"')).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Equipment" }));
    expect(screen.getByLabelText("Inventory search")).toHaveValue("multimeter");
    expect(screen.getByText("Industrial multimeter")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Calibration" }));
    expect(screen.getByLabelText("Calibration search")).toHaveValue("caliper");
  });

  it("preserves independent filters, sorts, and columns while allowing reference-only opt-in", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await user.click(screen.getByRole("button", { name: "Sort by Model" }));
    await user.click(screen.getByRole("button", { name: "View settings" }));
    await user.click(screen.getByRole("checkbox", { name: "Links" }));
    await user.click(screen.getByRole("menuitem", { name: "Show filters" }));
    await user.type(screen.getByLabelText("Filter manufacturer"), "Fluke");
    expect(await screen.findByText("Industrial multimeter")).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Calibration" }));
    expect(screen.queryByLabelText("Filter manufacturer")).not.toBeInTheDocument();
    expect(screen.getByRole("button", {
      name: "Sort by Calibration due, currently ascending. Activate for descending",
    })).toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "View settings" }));
    await user.click(screen.getByRole("checkbox", { name: "Vendor" }));
    await user.click(screen.getByRole("menuitem", { name: "Show filters" }));
    await user.click(screen.getByRole("button", { name: "Calibration requirement" }));
    await user.click(screen.getByRole("option", { name: "Reference only" }));
    expect(await screen.findByText("Deburring wheel assortment")).toBeInTheDocument();
    expect(screen.queryByText("Digital caliper, 6 inch")).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Sort by Asset #" }));

    await user.click(screen.getByRole("button", { name: "Equipment" }));
    expect(screen.getByLabelText("Filter manufacturer")).toHaveValue("Fluke");
    expect(screen.queryByRole("columnheader", { name: /Links/i })).not.toBeInTheDocument();
    expect(screen.getByRole("button", {
      name: "Sort by Model, currently ascending. Activate for descending",
    })).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Calibration" }));
    expect(screen.getByRole("button", { name: "Calibration requirement" })).toHaveTextContent("Reference only");
    expect(screen.queryByRole("columnheader", { name: /Vendor/i })).not.toBeInTheDocument();
    expect(screen.getByRole("button", {
      name: "Sort by Asset #, currently ascending. Activate for descending",
    })).toBeInTheDocument();
  });

  it("shows roster initialization only in the desktop calibration workspace", async () => {
    const user = userEvent.setup();
    window.inventoryDesktop = createDesktopBridge({
      pickCalibrationRosterFile: vi.fn().mockResolvedValue(null),
      previewCalibrationRoster: vi.fn(),
      commitCalibrationRoster: vi.fn(),
    });

    render(<InventoryShell />);

    expect(await screen.findByText("Showing all 0 entries")).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Initialize from Calibration Workbook" })).not.toBeInTheDocument();
    await user.click(screen.getByRole("button", { name: "Calibration" }));
    expect(screen.getByRole("button", { name: "Initialize from Calibration Workbook" })).toBeInTheDocument();
  });

  it("edits the same equipment record from Calibration and reads the change from Equipment", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await user.click(screen.getByRole("button", { name: "Calibration" }));
    await user.dblClick(screen.getByText("Digital caliper, 6 inch"));

    const editorSections = screen.getByLabelText("Entry editor section");
    expect(within(editorSections).getByRole("button", { name: "Calibration" })).toHaveAttribute("aria-pressed", "true");
    await user.type(screen.getByLabelText("Calibration vendor"), "MetroLab");
    await user.click(screen.getByRole("button", { name: "Save Entry" }));

    await user.click(screen.getByRole("button", { name: "Equipment" }));
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
    await user.click(screen.getByRole("button", { name: "Calibration" }));
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

    await user.click(screen.getByRole("button", { name: "Calibration" }));
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
});
