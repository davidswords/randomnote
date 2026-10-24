/** What's installed, as the commands report it, or undefined if a command is missing. */
export type Installed = {
  /** `rustup target list --installed`. */
  installedTargets: string | undefined;
  /** `wasm-bindgen --version`. */
  wasmBindgen: string | undefined;
  /** The wasm-bindgen version in the WebAssembly crate's lockfile. */
  wantedWasmBindgen: string;
};

const TARGET = "wasm32-unknown-unknown";

/** What's missing to build the web app, each with how to fix it. */
export function missingTools(installed: Installed): string[] {
  return [targetProblem(installed), wasmBindgenProblem(installed)].filter(
    (problem): problem is string => problem !== undefined,
  );
}

function targetProblem({ installedTargets }: Installed): string | undefined {
  if (installedTargets === undefined) {
    return [
      "rustup isn't installed, so the WebAssembly target can't be added. Rust from",
      "Homebrew's `rust` formula can't add targets; install rustup instead:",
      "  brew uninstall rust && brew install rustup && rustup-init",
      `then:\n  rustup target add ${TARGET}`,
    ].join("\n");
  }
  if (!installedTargets.split("\n").includes(TARGET)) {
    return `Rust's WebAssembly target isn't installed. Run:\n  rustup target add ${TARGET}`;
  }
  return undefined;
}

function wasmBindgenProblem({ wasmBindgen, wantedWasmBindgen }: Installed): string | undefined {
  const install = `cargo install wasm-bindgen-cli --version ${wantedWasmBindgen} --locked`;
  if (wasmBindgen === undefined) {
    return [
      `wasm-bindgen isn't installed, or isn't on your PATH. Run:\n  ${install}`,
      "If it's installed already, cargo puts it in ~/.cargo/bin; add that to your PATH:",
      `  echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc`,
    ].join("\n");
  }
  const version = wasmBindgen.trim().split(" ").at(-1);
  if (version !== wantedWasmBindgen) {
    return `wasm-bindgen is ${version}, but this project needs ${wantedWasmBindgen}. Run:\n  ${install} --force`;
  }
  return undefined;
}

/** The version of `name` in a Cargo.lock. */
export function lockedVersion(lock: string, name: string): string {
  const match = lock.match(new RegExp(`name = "${name}"\\nversion = "([^"]+)"`));
  if (!match) {
    throw new Error(`${name} isn't in the lockfile`);
  }
  return match[1];
}
