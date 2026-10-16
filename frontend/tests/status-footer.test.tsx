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
