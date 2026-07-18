import { expect, test } from "bun:test";

import { startFrontendDevServer } from "./vite-dev";

test(
  "Bun-native frontend launcher serves the current app and tears down cleanly",
  async () => {
    const reservation = Bun.serve({
      hostname: "127.0.0.1",
      port: 0,
      fetch: () => new Response("reserved"),
    });
    const port = reservation.port;
    await reservation.stop(true);

    const url = `http://127.0.0.1:${port}/`;
    const server = await startFrontendDevServer(["--port", String(port)]);

    try {
      await server.warmupRequest("/src/app/main.tsx");
      await server.waitForRequestsIdle();
      const response = await fetch(url);
      expect(response.ok).toBe(true);
      const html = await response.text();
      expect(html).toContain("<title>Inventory Management</title>");
      expect(html).toContain('<div id="root"></div>');
    } finally {
      await server.close();
    }

    await waitForPortToClose(url);
  },
  30_000,
);

async function waitForPortToClose(url: string): Promise<void> {
  const deadline = Date.now() + 5_000;
  while (Date.now() < deadline) {
    try {
      await fetch(url);
    } catch {
      return;
    }
    await Bun.sleep(50);
  }
  throw new Error(`Bun frontend launcher left ${url} listening after termination.`);
}
