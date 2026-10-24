import { describe, expect, it } from "vitest";
import { lockedVersion, missingTools } from "./tools";

const READY = {
  installedTargets: "aarch64-apple-darwin\nwasm32-unknown-unknown\n",
  wasmBindgen: "wasm-bindgen 0.2.129\n",
  wantedWasmBindgen: "0.2.129",
};

describe("missingTools", () => {
  it("finds nothing missing when everything is installed", () => {
    expect(missingTools(READY)).toEqual([]);
  });

  it("says to install rustup when Rust didn't come from it", () => {
    const problems = missingTools({ ...READY, installedTargets: undefined });

    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("rustup isn't installed");
    expect(problems[0]).toContain("brew install rustup");
  });

  it("says how to add the WebAssembly target", () => {
    const problems = missingTools({ ...READY, installedTargets: "aarch64-apple-darwin\n" });

    expect(problems).toEqual([
      expect.stringContaining("rustup target add wasm32-unknown-unknown"),
    ]);
  });

  it("doesn't take another WebAssembly target for the one the web app needs", () => {
    const problems = missingTools({ ...READY, installedTargets: "wasm32-wasip1\n" });

    expect(problems).toEqual([
      expect.stringContaining("rustup target add wasm32-unknown-unknown"),
    ]);
  });

  it("says how to install wasm-bindgen, and how to put cargo's tools on the PATH", () => {
    const problems = missingTools({ ...READY, wasmBindgen: undefined });

    expect(problems).toHaveLength(1);
    expect(problems[0]).toContain("cargo install wasm-bindgen-cli --version 0.2.129 --locked");
    expect(problems[0]).toContain('export PATH="$HOME/.cargo/bin:$PATH"');
  });

  it("says how to switch to the wanted wasm-bindgen version", () => {
    const problems = missingTools({ ...READY, wasmBindgen: "wasm-bindgen 0.2.120\n" });

    expect(problems).toEqual([
      expect.stringContaining(
        "wasm-bindgen is 0.2.120, but this project needs 0.2.129. Run:\n  cargo install wasm-bindgen-cli --version 0.2.129 --locked --force",
      ),
    ]);
  });

  it("reports every missing tool at once", () => {
    const problems = missingTools({
      ...READY,
      installedTargets: "aarch64-apple-darwin\n",
      wasmBindgen: undefined,
    });

    expect(problems).toHaveLength(2);
  });
});

describe("lockedVersion", () => {
  it("finds a package's version in a Cargo.lock", () => {
    const lock = [
      "[[package]]",
      'name = "wasm-bindgen-macro"',
      'version = "0.2.1"',
      "",
      "[[package]]",
      'name = "wasm-bindgen"',
      'version = "0.2.129"',
      'source = "registry+https://github.com/rust-lang/crates.io-index"',
    ].join("\n");

    expect(lockedVersion(lock, "wasm-bindgen")).toBe("0.2.129");
  });

  it("fails clearly when the package isn't in the lockfile", () => {
    expect(() => lockedVersion("", "wasm-bindgen")).toThrow(
      "wasm-bindgen isn't in the lockfile",
    );
  });
});
