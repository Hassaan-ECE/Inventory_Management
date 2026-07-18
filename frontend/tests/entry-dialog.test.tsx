import { beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";

import { EntryDialog } from "@/features/inventory/components/EntryDialog";
import type { InventoryEntry, InventoryEntryInput } from "@/features/inventory/types";

const BASE_ENTRY: InventoryEntry = {
  archived: false,
  assetNumber: "TE-401",
  assignedTo: "",
  calibrationRequirement: "unknown",
  lastCalibratedAt: "2026-01-31",
  calibrationDueAt: "2027-01-31",
  calibrationIntervalMonths: 12,
  certificateRef: "CERT-401",
  calibrationVendor: "Acme Calibration",
  calibrationNotes: "Return through metrology intake",
  condition: "",
  createdAt: "2026-04-23T09:00:00Z",
  description: "Precision fixture plate",
  id: "401",
  links: "",
  lifecycleStatus: "active",
  location: "Shelf B2",
  manufacturer: "Acme",
  model: "FP-401",
  notes: "",
  picturePath: "C:\\Pictures\\fixture-plate.jpg",
  outToCalibration: false,
  projectName: "Fixture Lab",
  qty: 1,
  entryUuid: "uuid-401",
  manualEntry: true,
  serialNumber: "SER-401",
  updatedAt: "2026-04-23 09:00:00",
  verifiedAt: "2026-04-23T09:00:00Z",
  verifiedBy: "Avery",
  workingStatus: "working",
};

describe("EntryDialog", () => {
  beforeEach(() => {
    document.documentElement.classList.remove("dark");
    delete window.inventoryDesktop;
    vi.restoreAllMocks();
    mockMatchMedia(false);
  });

  it("hides the picture path from the preview but saves it with the entry input", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn().mockResolvedValue(undefined) as unknown as (_: InventoryEntryInput) => Promise<void>;

    render(
      <EntryDialog
        mode="edit"
        entry={BASE_ENTRY}
        onClose={vi.fn()}
        onSave={onSave}
      />,
    );

    expect(screen.queryByLabelText("Picture Path")).not.toBeInTheDocument();
    expect(screen.queryByText("C:\\Pictures\\fixture-plate.jpg")).not.toBeInTheDocument();
    expect(screen.queryByText("Selected image")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Save Entry" }));

    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        picturePath: "C:\\Pictures\\fixture-plate.jpg",
      }),
      expect.objectContaining({
        baseVersion: BASE_ENTRY.updatedAt,
        changedFields: [],
      }),
    );
  });

  it("sends the entry base version and only the fields changed in the open dialog", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn().mockResolvedValue(undefined);

    render(
      <EntryDialog
        mode="edit"
        entry={BASE_ENTRY}
        onClose={vi.fn()}
        onSave={onSave}
      />,
    );

    await user.clear(screen.getByLabelText("Location"));
    await user.type(screen.getByLabelText("Location"), "Shelf C3");
    await user.type(screen.getByLabelText("Notes"), "Needs calibration");
    await user.click(screen.getByRole("button", { name: "Save Entry" }));

    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        location: "Shelf C3",
        notes: "Needs calibration",
      }),
      {
        baseVersion: BASE_ENTRY.updatedAt,
        changedFields: ["location", "notes"],
      },
    );
  });

  it("fills the picture path from the desktop picker without showing the selected path", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn().mockResolvedValue(undefined) as unknown as (_: InventoryEntryInput) => Promise<void>;
    const pickPicturePath = vi.fn().mockResolvedValue("C:\\Pictures\\selected-image.jpg");
    const loadPicturePreview = vi.fn().mockResolvedValue("data:image/jpeg;base64,cHJldmlldw==");
    window.inventoryDesktop = createDesktopBridge({
      loadPicturePreview,
      pickPicturePath,
    });

    render(
      <EntryDialog
        mode="add"
        onClose={vi.fn()}
        onSave={onSave}
      />,
    );

    await user.type(screen.getByLabelText("Asset Number"), "ME-900");
    await user.click(screen.getByRole("button", { name: "Browse" }));

    expect(pickPicturePath).toHaveBeenCalledTimes(1);
    expect(await screen.findByAltText("Entry picture preview")).toHaveAttribute(
      "src",
      "data:image/jpeg;base64,cHJldmlldw==",
    );
    expect(loadPicturePreview).toHaveBeenCalledWith("C:\\Pictures\\selected-image.jpg");
    expect(screen.queryByLabelText("Picture Path")).not.toBeInTheDocument();
    expect(screen.queryByText("C:\\Pictures\\selected-image.jpg")).not.toBeInTheDocument();
    expect(screen.queryByText("Selected image")).not.toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Save Entry" }));

    expect(onSave).toHaveBeenCalledWith(
      expect.objectContaining({
        assetNumber: "ME-900",
        picturePath: "C:\\Pictures\\selected-image.jpg",
      }),
      undefined,
    );
  });

  it("opens the picture in the desktop viewer from the large-screen preview", async () => {
    const openPath = vi.fn().mockResolvedValue(true);
    window.inventoryDesktop = createDesktopBridge({
      openPath,
    });
    mockMatchMedia(true);

    render(
      <EntryDialog
        mode="edit"
        entry={BASE_ENTRY}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />,
    );

    fireEvent.load(await screen.findByAltText("Entry picture preview"));
    fireEvent.doubleClick(screen.getByRole("button", { name: "Picture preview" }));

    expect(openPath).toHaveBeenCalledWith("C:\\Pictures\\fixture-plate.jpg");
  });

  it("shows a missing-picture fallback when the preview fails to load", async () => {
    render(
      <EntryDialog
        mode="edit"
        entry={BASE_ENTRY}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />,
    );

    fireEvent.error(await screen.findByAltText("Entry picture preview"));

    expect(screen.getAllByText("Picture not found").length).toBeGreaterThan(0);
  });

  it("uses the taller large-screen dialog sizing so the 1080p editor has more headroom", () => {
    mockMatchMedia(true);

    render(
      <EntryDialog
        mode="edit"
        entry={BASE_ENTRY}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />,
    );

    const dialogPanel = screen.getByRole("dialog").firstElementChild;
    expect(dialogPanel).toHaveClass("max-h-[92vh]", "lg:max-h-[94vh]");
  });

  it("uses styled dropdown selects for lifecycle and working status", async () => {
    const user = userEvent.setup();
    render(
      <EntryDialog
        mode="edit"
        entry={BASE_ENTRY}
        onClose={vi.fn()}
        onSave={vi.fn()}
      />,
    );

    const lifecycleSelect = screen.getByRole("button", { name: "Lifecycle" });
    const workingStatusSelect = screen.getByRole("button", { name: "Working Status" });

    expect(lifecycleSelect).toHaveAttribute("aria-haspopup", "listbox");
    expect(workingStatusSelect).toHaveAttribute("aria-haspopup", "listbox");
    expect(lifecycleSelect).toHaveTextContent(/Active/i);
    expect(workingStatusSelect).toHaveTextContent(/Working/i);

    await user.click(lifecycleSelect);
    expect(screen.getByRole("option", { name: "Active" })).toBeInTheDocument();
  });

  it("shows and preserves every calibration and verification field while editing", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    render(<EntryDialog mode="edit" entry={{ ...BASE_ENTRY, calibrationRequirement: "reference_only", outToCalibration: true }} onClose={vi.fn()} onSave={onSave} />);

    expect(screen.getByRole("button", { name: "Calibration requirement" })).toHaveTextContent(/Reference only/i);
    expect(screen.getByLabelText("Out to calibration")).toBeChecked();
    expect(screen.getByLabelText("Last calibrated")).toHaveValue("2026-01-31");
    expect(screen.getByLabelText("Calibration due")).toHaveValue("2027-01-31");
    expect(screen.getByLabelText("Calibration interval (months)")).toHaveValue(12);
    expect(screen.getByLabelText("Certificate reference")).toHaveValue("CERT-401");
    expect(screen.getByLabelText("Calibration vendor")).toHaveValue("Acme Calibration");
    expect(screen.getByLabelText("Calibration notes")).toHaveValue("Return through metrology intake");
    expect(screen.getByLabelText("Verified by")).toHaveValue("Avery");
    expect(screen.getByText(/Verified 2026-04-23T09:00:00Z/)).toBeInTheDocument();

    await user.click(screen.getByRole("button", { name: "Save Entry" }));
    expect(onSave).toHaveBeenCalledWith(expect.objectContaining({
      calibrationRequirement: "reference_only",
      outToCalibration: true,
      lastCalibratedAt: "2026-01-31",
      calibrationDueAt: "2027-01-31",
      calibrationIntervalMonths: 12,
      certificateRef: "CERT-401",
      calibrationVendor: "Acme Calibration",
      calibrationNotes: "Return through metrology intake",
      verifiedAt: "2026-04-23T09:00:00Z",
      verifiedBy: "Avery",
    }), expect.objectContaining({ changedFields: [] }));
  });

  it("blocks invalid calibration dates and intervals with field-specific messages", async () => {
    const user = userEvent.setup();
    const onSave = vi.fn();
    render(<EntryDialog mode="edit" entry={BASE_ENTRY} onClose={vi.fn()} onSave={onSave} />);

    await user.clear(screen.getByLabelText("Calibration due"));
    await user.type(screen.getByLabelText("Calibration due"), "2025-12-31");
    await user.click(screen.getByRole("button", { name: "Save Entry" }));
    expect(screen.getByText(/Calibration due date cannot be before last calibrated date/i)).toBeInTheDocument();
    expect(onSave).not.toHaveBeenCalled();

    await user.clear(screen.getByLabelText("Calibration due"));
    await user.type(screen.getByLabelText("Calibration due"), "2027-01-31");
    await user.clear(screen.getByLabelText("Calibration interval (months)"));
    await user.type(screen.getByLabelText("Calibration interval (months)"), "1201");
    await user.click(screen.getByRole("button", { name: "Save Entry" }));
    expect(screen.getByText(/Calibration interval must be between 1 and 1200 months/i)).toBeInTheDocument();
  });

  it("suggests a due date only after an explicit button click", async () => {
    const user = userEvent.setup();
    render(<EntryDialog mode="add" onClose={vi.fn()} onSave={vi.fn()} />);
    await user.type(screen.getByLabelText("Asset Number"), "TE-900");
    await user.click(screen.getByRole("button", { name: "Calibration requirement" }));
    await user.click(screen.getByRole("option", { name: "Required" }));
    await user.type(screen.getByLabelText("Last calibrated"), "2026-01-31");
    await user.type(screen.getByLabelText("Calibration interval (months)"), "1");

    expect(screen.getByLabelText("Calibration due")).toHaveValue("");
    await user.click(screen.getByRole("button", { name: "Suggest calibration due date" }));
    expect(screen.getByLabelText("Calibration due")).toHaveValue("2026-02-28");
  });

  it("labels only the calibration interval input instead of wrapping its action button", () => {
    render(<EntryDialog mode="add" onClose={vi.fn()} onSave={vi.fn()} />);

    const input = screen.getByLabelText("Calibration interval (months)");
    const label = screen.getByText("Calibration interval (months)");
    const suggestButton = screen.getByRole("button", { name: "Suggest calibration due date" });

    expect(label).toHaveAttribute("for", input.id);
    expect(label).not.toContainElement(suggestButton);
  });
});

function mockMatchMedia(matches: boolean) {
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      addEventListener: vi.fn(),
      addListener: vi.fn(),
      dispatchEvent: vi.fn(),
      matches,
      media: query,
      onchange: null,
      removeEventListener: vi.fn(),
      removeListener: vi.fn(),
    })),
    writable: true,
  });
}

function createDesktopBridge(
  overrides: Partial<NonNullable<Window["inventoryDesktop"]>>,
): NonNullable<Window["inventoryDesktop"]> {
  return {
    isDesktop: true,
    loadInventory: vi.fn().mockResolvedValue({ dbPath: "", entries: [] }),
    syncInventory: vi.fn().mockResolvedValue({
      dbPath: "",
      entries: [],
      shared: {
        available: true,
        canModify: true,
        enabled: true,
        message: "",
      },
    }),
    toggleVerifiedEntry: vi.fn().mockResolvedValue(BASE_ENTRY),
    createEntry: vi.fn().mockResolvedValue(BASE_ENTRY),
    updateEntry: vi.fn().mockResolvedValue(BASE_ENTRY),
    setArchivedEntry: vi.fn().mockResolvedValue(BASE_ENTRY),
    deleteEntry: vi.fn().mockResolvedValue({ entryId: BASE_ENTRY.id }),
    openExternal: vi.fn().mockResolvedValue(true),
    openPath: vi.fn().mockResolvedValue(true),
    loadPicturePreview: vi.fn().mockResolvedValue("data:image/jpeg;base64,cHJldmlldw=="),
    pickPicturePath: vi.fn().mockResolvedValue(null),
    ...overrides,
  } as NonNullable<Window["inventoryDesktop"]>;
}
