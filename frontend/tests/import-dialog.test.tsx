import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { ImportDialog } from "@/features/inventory/components/ImportDialog";
import type { ImportDryRunReport } from "@/features/inventory/types";
import { createDesktopBridge } from "./inventory-shell/helpers";

describe("ImportDialog", () => {
  beforeEach(() => {
    delete window.inventoryDesktop;
  });

  it("states that file import requires the desktop app in mock mode", () => {
    render(<ImportDialog onClose={vi.fn()} onEntriesChanged={vi.fn()} />);
    expect(screen.getByText(/Import is available in the desktop app/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: "Choose import file" })).not.toBeInTheDocument();
  });

  it("renders all five totals, every row, column treatment, issue, identifier, candidate, and raw value", async () => {
    const user = userEvent.setup();
    const report = allClassificationsReport();
    window.inventoryDesktop = createDesktopBridge({
      pickImportFile: vi.fn().mockResolvedValue("C:/imports/equipment.csv"),
      previewImport: vi.fn().mockResolvedValue(report),
    });
    render(<ImportDialog onClose={vi.fn()} onEntriesChanged={vi.fn()} />);
    await user.click(screen.getByRole("button", { name: "Choose import file" }));

    expect(screen.getByText("equipment.csv")).toBeInTheDocument();
    expect(screen.getByText("Equipment")).toBeInTheDocument();
    expect(screen.getByText("sha256:batch-1")).toBeInTheDocument();
    for (const label of ["Inserted: 1", "Matched: 1", "Conflicted: 1", "Rejected: 1", "Ignored: 1"]) {
      expect(screen.getByText(label)).toBeInTheDocument();
    }
    expect(screen.getByText("Unknown column retained for review")).toBeInTheDocument();
    expect(screen.getByText("Duplicate asset number")).toBeInTheDocument();
    expect(screen.getByText("legacy-3")).toBeInTheDocument();
  });

  it("keeps blocking previews non-committable", async () => {
    const user = userEvent.setup();
    const commitImport = vi.fn();
    window.inventoryDesktop = createDesktopBridge({
      pickImportFile: vi.fn().mockResolvedValue("C:/imports/equipment.csv"),
      previewImport: vi.fn().mockResolvedValue(allClassificationsReport()),
      commitImport,
    });
    render(<ImportDialog onClose={vi.fn()} onEntriesChanged={vi.fn()} />);
    await user.click(screen.getByRole("button", { name: "Choose import file" }));
    const commit = screen.getByRole("button", { name: "Commit import" });
    expect(commit).toBeDisabled();
    await user.click(screen.getByRole("checkbox", { name: /I confirm this import/i }));
    expect(commit).toBeDisabled();
    expect(screen.getByText(/conflicted and .* rejected rows block commit/i)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /Import 1 clean rows/i })).not.toBeInTheDocument();
    expect(commitImport).not.toHaveBeenCalled();
  });

  it("reports matched-only numeric no-op without claiming inserts", async () => {
    const user = userEvent.setup();
    const report = matchedOnlyReport();
    window.inventoryDesktop = createDesktopBridge({
      pickImportFile: vi.fn().mockResolvedValue("C:/imports/equipment.csv"),
      previewImport: vi.fn().mockResolvedValue(report),
      commitImport: vi.fn().mockResolvedValue({
        batchId: report.batchId, inserted: 0, matched: 1, conflicted: 0, rejected: 0, ignored: 0,
        remaining: 0, noop: 1, entriesChanged: false, message: "Batch already reconciled.",
      }),
    });
    render(<ImportDialog onClose={vi.fn()} onEntriesChanged={vi.fn()} />);
    await user.click(screen.getByRole("button", { name: "Choose import file" }));
    await user.click(screen.getByRole("checkbox", { name: /I confirm this import/i }));
    await user.click(screen.getByRole("button", { name: "Commit import" }));
    expect(await screen.findByText(/No inventory changes/i)).toBeInTheDocument();
    expect(screen.getByText(/No-op rows: 1/i)).toBeInTheDocument();
    expect(screen.queryByText(/Inserted: 1/i)).not.toBeInTheDocument();
  });

  it("refreshes entries after a changed commit and surfaces preview or commit errors", async () => {
    const user = userEvent.setup();
    const report = insertOnlyReport();
    const onEntriesChanged = vi.fn();
    const previewImport = vi.fn().mockRejectedValueOnce(new Error("Workbook is unreadable")).mockResolvedValue(report);
    window.inventoryDesktop = createDesktopBridge({
      pickImportFile: vi.fn().mockResolvedValue("C:/imports/equipment.csv"),
      previewImport,
      commitImport: vi.fn().mockResolvedValue({
        batchId: report.batchId, inserted: 1, matched: 0, conflicted: 0, rejected: 0, ignored: 0,
        remaining: 0, noop: 0, entriesChanged: true, message: "Import committed.",
      }),
    });
    render(<ImportDialog onClose={vi.fn()} onEntriesChanged={onEntriesChanged} />);
    await user.click(screen.getByRole("button", { name: "Choose import file" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Workbook is unreadable");
    await user.click(screen.getByRole("button", { name: "Choose import file" }));
    await user.click(screen.getByRole("checkbox", { name: /I confirm this import/i }));
    await user.click(screen.getByRole("button", { name: "Commit import" }));
    expect(await screen.findByText("Import committed.")).toBeInTheDocument();
    expect(onEntriesChanged).toHaveBeenCalledTimes(1);
  });

  it("prevents closing while a preview is in flight and closes after it completes", async () => {
    const user = userEvent.setup();
    const preview = createDeferred<ImportDryRunReport>();
    const onClose = vi.fn();
    window.inventoryDesktop = createDesktopBridge({
      pickImportFile: vi.fn().mockResolvedValue("C:/imports/equipment.csv"),
      previewImport: vi.fn().mockReturnValue(preview.promise),
    });
    render(<ImportDialog onClose={onClose} onEntriesChanged={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: "Choose import file" }));
    const closeButton = screen.getByRole("button", { name: "Close import dialog" });
    expect(closeButton).toBeDisabled();
    fireEvent.keyDown(document, { key: "Escape" });
    fireEvent.click(screen.getByRole("dialog"));
    expect(onClose).not.toHaveBeenCalled();

    preview.resolve(insertOnlyReport());
    await screen.findByText("sha256:batch-1");
    await user.click(closeButton);
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it("prevents closing while a commit is in flight and re-enables both close actions afterward", async () => {
    const user = userEvent.setup();
    const report = insertOnlyReport();
    const committed = createDeferred<{
      batchId: string;
      inserted: number;
      matched: number;
      conflicted: number;
      rejected: number;
      ignored: number;
      remaining: number;
      noop: number;
      entriesChanged: boolean;
      message: string;
    }>();
    const onClose = vi.fn();
    window.inventoryDesktop = createDesktopBridge({
      pickImportFile: vi.fn().mockResolvedValue("C:/imports/equipment.csv"),
      previewImport: vi.fn().mockResolvedValue(report),
      commitImport: vi.fn().mockReturnValue(committed.promise),
    });
    render(<ImportDialog onClose={onClose} onEntriesChanged={vi.fn()} />);

    await user.click(screen.getByRole("button", { name: "Choose import file" }));
    await user.click(screen.getByRole("checkbox", { name: /I confirm this import/i }));
    await user.click(screen.getByRole("button", { name: "Commit import" }));

    const closeButton = screen.getByRole("button", { name: "Close import dialog" });
    const cancelButton = screen.getByRole("button", { name: "Cancel" });
    expect(closeButton).toBeDisabled();
    expect(cancelButton).toBeDisabled();
    fireEvent.keyDown(document, { key: "Escape" });
    fireEvent.click(screen.getByRole("dialog"));
    expect(onClose).not.toHaveBeenCalled();

    committed.resolve({
      batchId: report.batchId,
      inserted: 1,
      matched: 0,
      conflicted: 0,
      rejected: 0,
      ignored: 0,
      remaining: 0,
      noop: 0,
      entriesChanged: false,
      message: "Import committed.",
    });
    await screen.findByText("Import committed.");
    expect(closeButton).toBeEnabled();
    expect(cancelButton).toBeEnabled();
    await user.click(cancelButton);
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});

function createDeferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((resolvePromise) => {
    resolve = resolvePromise;
  });
  return { promise, resolve };
}

function insertOnlyReport(): ImportDryRunReport {
  return reportFor("inserted");
}

function matchedOnlyReport(): ImportDryRunReport {
  return reportFor("matched");
}

function reportFor(classification: "inserted" | "matched"): ImportDryRunReport {
  return {
    batchId: "sha256:batch-1", sourceFingerprint: "sha256:source", sourceFilename: "equipment.csv",
    selectedSheet: "Equipment", mappingVersion: "te-test-equipment-v1", totalRows: 1,
    inserted: classification === "inserted" ? 1 : 0, matched: classification === "matched" ? 1 : 0,
    conflicted: 0, rejected: 0, ignored: 0, blocking: false, reconciliationBasis: "revision-1",
    columns: [{ originalHeader: "Asset Number", normalizedTarget: "assetNumber", treatment: "mapped", nonblankCount: 1, reason: "Mapped" }],
    rowOutcomes: [{ sourceRow: 2, classification, issues: [], originalId: "legacy-1", originalAssetNumber: "TE-1", originalSerialNumber: null, candidateEntryUuid: classification === "matched" ? "candidate-1" : null, rawValues: { "Asset Number": "TE-1" } }],
  };
}

function allClassificationsReport(): ImportDryRunReport {
  const classifications = ["inserted", "matched", "conflicted", "rejected", "ignored"] as const;
  return {
    ...reportFor("inserted"), totalRows: 5, inserted: 1, matched: 1, conflicted: 1, rejected: 1, ignored: 1, blocking: true,
    columns: [{ originalHeader: "Mystery", normalizedTarget: null, treatment: "unknown", nonblankCount: 1, reason: "Unknown column retained for review" }],
    rowOutcomes: classifications.map((classification, index) => ({
      sourceRow: index + 2, classification, issues: classification === "conflicted" ? ["Duplicate asset number"] : [],
      originalId: `legacy-${index + 1}`, originalAssetNumber: `TE-${index + 1}`, originalSerialNumber: null,
      candidateEntryUuid: classification === "conflicted" ? "candidate-3" : null,
      rawValues: { Mystery: index === 0 ? "Unmapped Value" : `value-${index + 1}` },
    })),
  };
}
