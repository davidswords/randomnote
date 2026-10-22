import { type ChildProcess, spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { createConnection } from "node:net";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { remote } from "webdriverio";

const DRIVER_PORT = 4444;
const APPLICATION = resolve(
  import.meta.dirname,
  "../src-tauri/target/debug/randomnote",
);

export type RunningApp = Awaited<ReturnType<typeof remote>>;

/**
 * Drives the built app through tauri-driver. Every launch shares one fresh
 * data directory, so tests start empty and never touch real data.
 */
export class AppDriver {
  private constructor(
    private readonly driver: ChildProcess,
    private readonly dataDirectory: string,
  ) {}

  static async start(): Promise<AppDriver> {
    const dataDirectory = mkdtempSync(join(tmpdir(), "randomnote-e2e-"));
    const driver = spawn("tauri-driver", ["--port", String(DRIVER_PORT)], {
      env: { ...process.env, XDG_DATA_HOME: dataDirectory },
      stdio: "inherit",
    });
    await waitForPort(DRIVER_PORT);
    return new AppDriver(driver, dataDirectory);
  }

  launch(): Promise<RunningApp> {
    return remote({
      hostname: "127.0.0.1",
      port: DRIVER_PORT,
      logLevel: "warn",
      capabilities: { "tauri:options": { application: APPLICATION } } as never,
    });
  }

  stop(): void {
    this.driver.kill();
    rmSync(this.dataDirectory, { recursive: true, force: true });
  }
}

async function waitForPort(port: number, timeoutMs = 15_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (await isListening(port)) return;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error(`tauri-driver did not start on port ${port}`);
}

function isListening(port: number): Promise<boolean> {
  return new Promise((resolve) => {
    const socket = createConnection({ host: "127.0.0.1", port });
    socket.once("connect", () => resolve(socket.end() && true));
    socket.once("error", () => resolve(false));
  });
}
