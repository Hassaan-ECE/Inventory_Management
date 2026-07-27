import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { InventoryTable } from "@/modules/te-test-equipment/components/InventoryTable";
import { OVERSCAN_ROWS, ROW_HEIGHT, getVisibleRange } from "@/modules/te-test-equipment/components/table/virtualization";
import { buildDefaultColumnVisibility } from "@/modules/te-test-equipment/lib";
import {
  CALIBRATION_COLUMNS,
  INVENTORY_COLUMNS,
  type ColumnConfig,
  type InventoryEntry,
} from "@/modules/te-test-equipment/types";
import { InventoryShell } from "@/shell/InventoryShell";

function tableExtras(columns: readonly ColumnConfig[] = INVENTORY_COLUMNS) {
  return {
    allColumns: columns,
    columnVisibility: buildDefaultColumnVisibility(columns),
    onToggleColumn: () => undefined,
  };
}

async function openColumnMenu() {
  const header = screen.getAllByRole("columnheader")[0];
  fireEvent.contextMenu(header);
  expect(await screen.findByRole("checkbox", { name: "Qty" })).toBeInTheDocument();
}

describe("InventoryShell table controls", () => {
  beforeEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove("dark");
  });

  it("renders compact labels for long links", () => {
    render(<InventoryShell />);

    expect(screen.getByText("www.cejn.com/en-us/products/thermal-control")).toBeInTheDocument();
  });

  it("renders unsafe link values as inert text instead of anchors", () => {
    const unsafeEntry: InventoryEntry = {
      archived: false,
      assetNumber: "ME-UNSAFE",
      assignedTo: "",
      calibrationRequirement: "unknown",
      condition: "",
      createdAt: "2026-04-25T12:00:00.000Z",
      description: "Unsafe link entry",
      entryUuid: "unsafe-1",
      id: "unsafe-1",
      links: "javascript:alert(1)",
      lifecycleStatus: "active",
      location: "Bench",
      manufacturer: "Acme",
      model: "Unsafe",
      notes: "",
      outToCalibration: false,
      manualEntry: true,
      picturePath: "",
      projectName: "Security",
      qty: 1,
      serialNumber: "",
      updatedAt: "2026-04-25T12:00:00.000Z",
      workingStatus: "working",
    };

    render(
      <InventoryTable
        canModifyEntries
        colorRows={false}
        columns={INVENTORY_COLUMNS}
        entries={[unsafeEntry]}
        sortState={{ column: "manufacturer", direction: "asc" }}
        onOpenContextMenu={() => undefined}
        onOpenEntry={() => undefined}
        onOpenExternalLink={() => undefined}
        onSortChange={() => undefined}
        onToggleVerified={() => undefined}
        {...tableExtras()}
      />,
    );

    expect(screen.getByText("javascript:alert(1)")).toBeInTheDocument();
    expect(screen.queryByRole("link", { name: "javascript:alert(1)" })).not.toBeInTheDocument();
  });

  it("routes safe table links through the supplied native-open handler", async () => {
    const user = userEvent.setup();
    const onOpenExternalLink = vi.fn();
    const entry: InventoryEntry = {
      archived: false,
      assetNumber: "ME-LINK",
      assignedTo: "",
      calibrationRequirement: "unknown",
      condition: "",
      createdAt: "2026-04-25T12:00:00.000Z",
      description: "Safe link entry",
      entryUuid: "link-1",
      id: "link-1",
      links: "https://example.com/item",
      lifecycleStatus: "active",
      location: "Bench",
      manufacturer: "Acme",
      model: "Link",
      notes: "",
      outToCalibration: false,
      manualEntry: true,
      picturePath: "",
      projectName: "Security",
      qty: 1,
      serialNumber: "",
      updatedAt: "2026-04-25T12:00:00.000Z",
      workingStatus: "working",
    };

    render(
      <InventoryTable
        canModifyEntries
        colorRows={false}
        columns={INVENTORY_COLUMNS}
        entries={[entry]}
        sortState={{ column: "manufacturer", direction: "asc" }}
        onOpenContextMenu={() => undefined}
        onOpenEntry={() => undefined}
        onOpenExternalLink={onOpenExternalLink}
        onSortChange={() => undefined}
        onToggleVerified={() => undefined}
        {...tableExtras()}
      />,
    );

    await user.click(screen.getByRole("link", { name: "example.com/item" }));

    expect(onOpenExternalLink).toHaveBeenCalledWith("https://example.com/item");
  });

  it("keeps row actions and verified controls wired through extracted table rows", async () => {
    const user = userEvent.setup();
    const onOpenContextMenu = vi.fn();
    const onOpenEntry = vi.fn();
    const onToggleVerified = vi.fn();
    const entry = buildEntry({
      description: "Row actions entry",
      id: "row-actions-1",
      links: "https://example.com/item",
    });

    render(
      <InventoryTable
        canModifyEntries
        colorRows={false}
        columns={INVENTORY_COLUMNS}
        entries={[entry]}
        sortState={{ column: "manufacturer", direction: "asc" }}
        onOpenContextMenu={onOpenContextMenu}
        onOpenEntry={onOpenEntry}
        onOpenExternalLink={() => undefined}
        onSortChange={() => undefined}
        onToggleVerified={onToggleVerified}
        {...tableExtras()}
      />,
    );

    const row = screen.getByText("Row actions entry").closest("tr");

    await user.dblClick(screen.getByText("Row actions entry"));
    expect(onOpenEntry).toHaveBeenCalledWith("row-actions-1");

    expect(row).not.toBeNull();
    fireEvent.contextMenu(row as HTMLTableRowElement, { clientX: 42, clientY: 84 });
    expect(onOpenContextMenu).toHaveBeenCalledWith("row-actions-1", 42, 84);

    const verifiedButton = screen.getByRole("button", { name: "Verify Row actions entry" });
    await user.click(verifiedButton);
    expect(onToggleVerified).toHaveBeenCalledWith("row-actions-1");

    await user.dblClick(verifiedButton);
    expect(onOpenEntry).toHaveBeenCalledTimes(1);
  });

  it("calculates virtualized ranges with the table row height and overscan window", () => {
    const viewportHeight = ROW_HEIGHT * 5;

    expect(getVisibleRange(100, ROW_HEIGHT * 20, viewportHeight)).toEqual({
      end: 20 + 5 + OVERSCAN_ROWS,
      start: 20 - OVERSCAN_ROWS,
    });
    expect(getVisibleRange(100, ROW_HEIGHT * 500, viewportHeight)).toEqual({
      end: 100,
      start: 95 - OVERSCAN_ROWS,
    });
  });

  it("renders calibration cells and a verifiedAt-driven accessible action", () => {
    const entry = buildEntry({
      calibrationRequirement: "required",
      outToCalibration: true,
      calibrationDueAt: "2026-07-20",
      verifiedAt: "2026-07-13T12:00:00Z",
      verifiedBy: "Avery",
    });
    render(
      <InventoryTable
        canModifyEntries
        colorRows
        columns={CALIBRATION_COLUMNS}
        entries={[entry]}
        localDate="2026-07-13"
        sortState={{ column: "calibrationHealth", direction: "asc" }}
        onOpenContextMenu={() => undefined}
        onOpenEntry={() => undefined}
        onOpenExternalLink={() => undefined}
        onSortChange={() => undefined}
        onToggleVerified={() => undefined}
        workspace="calibration"
        {...tableExtras(CALIBRATION_COLUMNS)}
      />,
    );
    expect(screen.getAllByText("Out to cal").length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText("07/26")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Clear verification for Table row entry" })).toBeInTheDocument();
    expect(screen.getByText("Table row entry").closest("tr")?.className).toContain("bg-warning/10");
  });

  it("colors archived calibration rows by health when color rows is on", () => {
    const entry = buildEntry({
      archived: true,
      calibrationRequirement: "required",
      calibrationDueAt: "2026-07-01",
      outToCalibration: false,
    });
    render(
      <InventoryTable
        canModifyEntries
        colorRows
        columns={CALIBRATION_COLUMNS}
        entries={[entry]}
        localDate="2026-07-13"
        sortState={{ column: "calibrationDueAt", direction: "asc" }}
        onOpenContextMenu={() => undefined}
        onOpenEntry={() => undefined}
        onOpenExternalLink={() => undefined}
        onSortChange={() => undefined}
        onToggleVerified={() => undefined}
        workspace="calibration"
        {...tableExtras(CALIBRATION_COLUMNS)}
      />,
    );
    expect(screen.getByText("Table row entry").closest("tr")?.className).toContain("bg-destructive/10");
    expect(screen.getByText("Overdue")).toBeInTheDocument();
  });

  it("hides a selected column from the table", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await openColumnMenu();
    await user.click(screen.getByRole("checkbox", { name: "Links" }));

    expect(screen.queryByRole("columnheader", { name: /Links/i })).not.toBeInTheDocument();
  });

  it("shows a selected style on the color rows toggle and still toggles row colors", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    const colorRowsToggle = screen.getByRole("button", { name: "Color rows" });
    const firstRow = screen.getByText("Stainless socket-head cap screws, 1/4-20").closest("tr");

    expect(colorRowsToggle).toHaveAttribute("aria-pressed", "true");
    expect(colorRowsToggle.className).not.toContain("grayscale");
    expect(firstRow?.className).toContain("bg-success/10");

    await user.click(colorRowsToggle);

    expect(colorRowsToggle).toHaveAttribute("aria-pressed", "false");
    expect(colorRowsToggle.className).toContain("grayscale");
    expect(firstRow?.className).toContain("bg-transparent");
  });

  it("disables the last visible data column in the menu", async () => {
    const user = userEvent.setup();
    render(<InventoryShell />);

    await openColumnMenu();
    await user.click(screen.getByRole("checkbox", { name: "Links" }));
    await user.click(screen.getByRole("checkbox", { name: "Location" }));
    await user.click(screen.getByRole("checkbox", { name: "Description" }));
    await user.click(screen.getByRole("checkbox", { name: "Model" }));
    await user.click(screen.getByRole("checkbox", { name: "Manufacturer" }));
    await user.click(screen.getByRole("checkbox", { name: "Serial #" }));
    await user.click(screen.getByRole("checkbox", { name: "Calibration due" }));
    await user.click(screen.getByRole("checkbox", { name: "Calibration health" }));

    expect(screen.getByRole("checkbox", { name: "Qty" })).toBeDisabled();
    expect(screen.getByRole("columnheader", { name: /Qty/i })).toBeInTheDocument();
  });
});

function buildEntry(overrides: Partial<InventoryEntry> = {}): InventoryEntry {
  return {
    archived: false,
    assetNumber: "ME-ROW",
    assignedTo: "",
    calibrationRequirement: "unknown",
    condition: "",
    createdAt: "2026-04-25T12:00:00.000Z",
    description: "Table row entry",
    entryUuid: "row-1",
    id: "row-1",
    links: "",
    lifecycleStatus: "active",
    location: "Bench",
    manufacturer: "Acme",
    model: "Row",
    notes: "",
    outToCalibration: false,
    manualEntry: true,
    picturePath: "",
    projectName: "Inventory",
    qty: 1,
    serialNumber: "",
    updatedAt: "2026-04-25T12:00:00.000Z",
    workingStatus: "working",
    ...overrides,
  } as InventoryEntry;
}
