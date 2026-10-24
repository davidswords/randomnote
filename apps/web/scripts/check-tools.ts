// Checks the tools the WebAssembly build needs, before it runs, and says how
// to install whatever's missing. Its decisions are tested in tools.test.ts.
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { lockedVersion, missingTools } from "./tools.ts";

function output(command: string, args: string[]): string | undefined {
  try {
    return execFileSync(command, args, { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
  } catch {
    return undefined;
  }
}

const lock = readFileSync(new URL("../../../crates/client/web/Cargo.lock", import.meta.url), "utf8");
const problems = missingTools({
  installedTargets: output("rustup", ["target", "list", "--installed"]),
  wasmBindgen: output("wasm-bindgen", ["--version"]),
  wantedWasmBindgen: lockedVersion(lock, "wasm-bindgen"),
});

if (problems.length > 0) {
  console.error(`\nThe web app needs a few tools first.\n\n${problems.join("\n\n")}\n`);
  process.exit(1);
}
