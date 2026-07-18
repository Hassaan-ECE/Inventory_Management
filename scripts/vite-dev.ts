import { fileURLToPath } from "node:url";

import { createServer, type ViteDevServer } from "vite";

const configFile = fileURLToPath(new URL("../frontend/vite.config.ts", import.meta.url));

export async function startFrontendDevServer(args: string[] = Bun.argv.slice(2)): Promise<ViteDevServer> {
  const port = readPort(args) ?? 5173;
  const server = await createServer({
    configFile,
    server: {
      host: "127.0.0.1",
      port,
      strictPort: true,
    },
  });
  await server.listen();
  return server;
}

if (import.meta.main) {
  let server: ViteDevServer | undefined;
  let closing = false;

  const close = async (exitCode: number): Promise<void> => {
    if (closing) return;
    closing = true;
    try {
      await server?.close();
    } finally {
      process.exit(exitCode);
    }
  };

  process.once("SIGINT", () => void close(0));
  process.once("SIGTERM", () => void close(0));

  try {
    server = await startFrontendDevServer();
    console.log("TE frontend ready.");
    server.printUrls();
  } catch (error) {
    console.error("Could not start the TE frontend development server.", error);
    await close(1);
  }
}

function readPort(args: string[]): number | undefined {
  const portIndex = args.indexOf("--port");
  const raw = portIndex >= 0 ? args[portIndex + 1] : args.find((arg) => arg.startsWith("--port="))?.slice(7);
  if (raw === undefined) return undefined;

  const parsed = Number(raw);
  if (!Number.isInteger(parsed) || parsed < 1 || parsed > 65_535) {
    throw new Error(`--port must be an integer from 1 to 65535; received '${raw}'.`);
  }
  return parsed;
}
