# randomnote

A minimal, client-side encrypted, open source todo app built with TypeScript and Tauri 2. Product decisions and open questions are in [docs/decisions.md](docs/decisions.md). Read it before changing behaviour, and update it when a decision is made.

## How we work

- **Test-driven:** write a failing test, make it pass, then refactor. No behaviour without a test.
- **Small atomic steps:** each step is verified working, with tests, before the next one starts.
- **Clean code:** small, well-named functions. Keep domain logic in `packages/core` as pure TypeScript, free of UI and platform code.
- **Ask, don't guess:** when a product decision is open in `docs/decisions.md`, ask instead of assuming.

## Testing

- **Unit tests** (`*.test.ts`) cover one behaviour with no real I/O, clock, randomness, crypto, storage or network. Dependencies are passed in as simple fakes. This is where red, green, refactor happens.
- **Integration tests** (`*.integration.test.ts`) run our code against the real dependency at a boundary: Web Crypto, storage, encryption, Tauri, the sync server.
- **Contract tests:** each port (for example a repository interface) has one shared suite. The in-memory version runs it as a unit test, and every real adapter runs the same suite as an integration test.
- **Conventions:** arrange, act, assert; one behaviour per test; test names describe behaviour; no logic in assertions; test through the public API; prefer fakes over mocking libraries.
- **Rust:** tests in a crate's `tests/` directory use only its public API against real dependencies, so they are integration tests. Pure logic gets unit tests in a `#[cfg(test)]` module next to the code.
- **Tauri commands:** integration tests in `src-tauri/tests/` send real IPC messages through `tauri::test` against a real store, pinning the command names, argument keys and JSON. The TypeScript adapter's unit tests pin the same messages against a fake `invoke`. Tauri turns Rust `snake_case` argument names into `camelCase` keys, so prefer single-word argument names.
- **End-to-end tests** (`apps/desktop/e2e/*.e2e.test.ts`) drive the built app through `tauri-driver` and WebDriver, with a fresh temporary data directory, exactly as a user would. `tauri-driver` supports Linux and Windows but not macOS, so they run on Linux. They run with `test:e2e`, not `npm test`.
- **Mutation check:** when a test passes as soon as it's written, break the code on purpose and confirm the test fails.

## Layout

- `packages/core`: domain logic and its tests.
- `apps/desktop`: the Tauri 2 shell (`src-tauri`) and the Vite + TypeScript frontend (`src`).
- `crates/store`: SQLite storage in Rust (rusqlite, bundled SQLite). No Tauri dependency, so it builds and tests anywhere.
- `crates/crypto`: encryption following the Standard Notes 004 protocol (Argon2id key derivation and XChaCha20-Poly1305 record encryption so far). Test vectors come from independent reference implementations, never from this crate.

## Commands

Run these from the repository root. Node's version is in `.nvmrc`.

- `npm ci`: install dependencies.
- `npm test`: run all tests.
- `npm run test:unit` / `npm run test:integration`: run one kind on its own.
- `npm run typecheck`: typecheck all workspaces.
- `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` in `crates/store`, `crates/crypto` or `apps/desktop/src-tauri`: test and lint the Rust code. On Linux, `src-tauri` needs Tauri's system packages (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev`).
- `npm run test:e2e -w @randomnote/desktop`: run the end-to-end tests against a debug build (`npm run tauri -w @randomnote/desktop -- build --debug --no-bundle`). On Linux they need `webkit2gtk-driver`, `tauri-driver` (`cargo install tauri-driver --locked`) and a display; in a headless shell, prefix with `xvfb-run -a`.
- `npm run dev`: run the desktop app. This needs Rust and the Tauri prerequisites for your OS.

## CI and releases

- `.github/workflows/ci.yml` runs typecheck and tests, plus formatting, clippy and tests for `crates/store`, `crates/crypto` and `apps/desktop/src-tauri`, on pull requests and on pushes to `main`. Its `e2e` job builds the app and runs the end-to-end tests on Linux.
- `.github/workflows/release.yml` builds the Apple Silicon `.dmg`. On a `v*` tag it publishes a GitHub release. The tag must match the version in `apps/desktop/package.json`.

## Releasing

1. Bump the version in `apps/desktop/package.json` (`npm version X.Y.Z -w @randomnote/desktop --no-git-tag-version`, with npm 11 so the lockfile format matches) and in `apps/desktop/src-tauri/Cargo.toml` (then `cargo update --workspace` in `src-tauri`). Merge it through a pull request.
2. Tag the merge commit on `main` as `vX.Y.Z` and push the tag. The release workflow builds and publishes the `.dmg`.
3. The `Update casks` workflow in [davidswords/homebrew-tap](https://github.com/davidswords/homebrew-tap) updates the cask daily at 06:17 UTC. Run it by hand to publish sooner.
