// Starts the real randomtask server and serves the built web app for the
// end-to-end tests. The app must be built pointing at the same server:
//   npm run build:e2e -w @randomtask/web
import { type ChildProcess, spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

export const APP_URL = "http://localhost:1430/";
export const SERVER_URL = "http://127.0.0.1:8181";

const root = fileURLToPath(new URL("../../..", import.meta.url));
const serverBinary =
  process.env.RANDOMTASK_SERVER_BINARY ??
  join(root, "crates/server/target/debug/randomtask-server");

async function waitFor(url: string, init?: RequestInit): Promise<void> {
  for (let attempt = 0; attempt < 120; attempt++) {
    try {
      await fetch(url, init);
      return;
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
  }
  throw new Error(`${url} didn't start`);
}

export default async function setup(): Promise<() => void> {
  const directory = mkdtempSync(join(tmpdir(), "randomtask-web-e2e-"));
  const children: ChildProcess[] = [
    spawn(serverBinary, [], {
      env: {
        ...process.env,
        RANDOMTASK_ADDRESS: "127.0.0.1:8181",
        RANDOMTASK_DATABASE: join(directory, "server.db"),
        RANDOMTASK_ALLOWED_ORIGINS: "http://localhost:1430",
      },
      stdio: "inherit",
      detached: true,
    }),
    spawn("npx", ["vite", "preview", "--host", "localhost"], {
      cwd: fileURLToPath(new URL("..", import.meta.url)),
      stdio: "inherit",
      // Its own process group, so stopping it also stops the vite that npx starts.
      detached: true,
    }),
  ];
  await waitFor(APP_URL);
  await waitFor(`${SERVER_URL}/v2/login-params`, { method: "POST" });

  return () => {
    children.forEach((child) => process.kill(-child.pid!));
    rmSync(directory, { recursive: true, force: true });
  };
}
