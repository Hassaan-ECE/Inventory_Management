import { afterEach, describe, expect, it, vi } from "vitest";
import { render, screen } from "@testing-library/react";

async function renderFooter(devBuild: boolean) {
  vi.stubEnv("DEV", devBuild);
  vi.resetModules();
  const { StatusFooter } = await import("@/platform/ui/StatusFooter");
  render(<StatusFooter resultsLabel="3 items" message="Saved." />);
}

describe("StatusFooter build tag", () => {
  afterEach(() => {
    vi.unstubAllEnvs();
  });

  it("credits the author in release builds", async () => {
    await renderFooter(false);

    expect(screen.getByText("Built by Syed Hassaan Shah")).toBeInTheDocument();
    expect(screen.queryByText("DEV build")).not.toBeInTheDocument();
    expect(screen.getByText("3 items")).toBeInTheDocument();
    expect(screen.getByText("Saved.")).toBeInTheDocument();
  });

  it("shows a DEV tag instead of the credit in developer builds", async () => {
    await renderFooter(true);

    expect(screen.getByText("DEV build")).toBeInTheDocument();
    expect(screen.queryByText("Built by Syed Hassaan Shah")).not.toBeInTheDocument();
  });
});

describe("StatusFooter sync warning", () => {
  it("shows the warning label with its explanation on hover", async () => {
    const { StatusFooter } = await import("@/platform/ui/StatusFooter");
    render(<StatusFooter message="Synced." warning={{ kind: "skippedFiles", count: 2 }} />);

    const chip = screen.getByRole("status");
    expect(chip).toHaveTextContent("2 shared files skipped");
    expect(chip.getAttribute("title")).toContain("could not be read");
  });

  it("shows no chip without a warning", async () => {
    const { StatusFooter } = await import("@/platform/ui/StatusFooter");
    render(<StatusFooter message="Synced." />);

    expect(screen.queryByRole("status")).not.toBeInTheDocument();
  });
});

describe("parseSharedStatus", () => {
  it("keeps known warnings and drops unknown or malformed ones", async () => {
    const { parseSharedStatus } = await import("@/platform/sync/sharedStatus");
    const base = { available: true, canModify: true, enabled: true, message: "ok" };

    expect(parseSharedStatus({ ...base, warning: { kind: "databaseNearlyFull", percentUsed: 84 } }).warning)
      .toEqual({ kind: "databaseNearlyFull", percentUsed: 84 });
    expect(parseSharedStatus({ ...base, warning: { kind: "updateAvailable", version: "0.3.0" } }).warning)
      .toEqual({ kind: "updateAvailable", version: "0.3.0" });
    expect(parseSharedStatus({ ...base, warning: { kind: "somethingNew" } }).warning).toBeUndefined();
    expect(parseSharedStatus({ ...base, warning: { kind: "skippedFiles" } }).warning).toBeUndefined();
  });
});
