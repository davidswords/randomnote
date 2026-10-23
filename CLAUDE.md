# randomtask

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
- **Coverage is enforced at 100%.** TypeScript: lines, branches, functions and statements (`npm run test:coverage`). Rust: lines (`cargo llvm-cov --fail-under-lines 100`). CI fails below that. The only exclusions are app entry points that only end-to-end tests reach (`apps/desktop/src/main.ts`, `src-tauri/src/app.rs`, `src-tauri/src/main.rs`) and test-only code; add to them only with a written reason.
- **Error paths are behaviour.** A storage, decoding or decryption failure needs a test showing what the user or caller gets.
- **Mutation check:** when a test passes as soon as it's written, break the code on purpose and confirm the test fails.

## Layout

- `packages/core`: domain logic and its tests.
- `apps/desktop`: the Tauri 2 shell (`src-tauri`) and the Vite + TypeScript frontend (`src`).
- `crates/client`: the Rust that runs on people's devices, for the desktop, web, iOS and Android apps. Nothing in it is specific to Tauri, so it builds and tests anywhere.
  - `crates/client/store`: SQLite storage (rusqlite, bundled SQLite).
  - `crates/client/crypto`: encryption following the Standard Notes 004 protocol (Argon2id key derivation, XChaCha20-Poly1305 record encryption, and the items-key hierarchy). Test vectors come from independent reference implementations, never from this crate.
  - `crates/client/api`: the client for the server at api.randomtask.ch. It registers and signs in as the Standard Notes client does, sending only the server password derived from the user's password.
  - `crates/client/vault`: the account and its encrypted tasks. It derives keys from the password with `crypto`, stores only ciphertext with `store`, and holds keys only in memory while unlocked.
- `crates/server`: the randomtask server at api.randomtask.ch (axum, SQLite). It registers accounts, signs them in, refreshes their sessions and signs them out, at the paths and in the formats the Standard Notes client uses. It never sees passwords or keys, only the server password, which it stores as a bcrypt hash.

## Commands

Run these from the repository root. Node's version is in `.nvmrc`.

- `npm ci`: install dependencies.
- `npm test`: run all tests.
- `npm run test:unit` / `npm run test:integration`: run one kind on its own.
- `npm run test:coverage`: run all TypeScript tests and fail below 100% coverage.
- `npm run typecheck`: typecheck all workspaces.
- `cargo test`, `cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` in each crate under `crates/client`, in `crates/server` or in `apps/desktop/src-tauri`: test and lint the Rust code. On Linux, `src-tauri` needs Tauri's system packages (`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev`).
- `npm run test:e2e -w @randomtask/desktop`: run the end-to-end tests against a debug build (`npm run tauri -w @randomtask/desktop -- build --debug --no-bundle`). On Linux they need `webkit2gtk-driver`, `tauri-driver` (`cargo install tauri-driver --locked`) and a display; in a headless shell, prefix with `xvfb-run -a`.
- `docker compose up --build`: run the server in Docker on `127.0.0.1:8080`, with its database in a volume. `cargo run` in `crates/server` runs it without Docker. `RANDOMTASK_ADDRESS`, `RANDOMTASK_DATABASE` (the SQLite file) and `RANDOMTASK_PSEUDO_KEY` override the defaults; without the last, the database keeps its own secret for made-up key parameters.
- `npm run dev`: run the desktop app. This needs Rust and the Tauri prerequisites for your OS. With `RANDOMTASK_SERVER=http://127.0.0.1:8080`, it registers accounts with that server; without it, as in releases, accounts stay on the device.

## CI and releases

- `.github/workflows/ci.yml` runs typecheck and tests, plus formatting, clippy and tests for each crate under `crates/client`, `crates/server` and `apps/desktop/src-tauri`, on pull requests and on pushes to `main`. Its `e2e` job builds the app and runs the end-to-end tests on Linux. Its `server-image` job starts the server with Docker Compose and checks it answers. Its `cross-platform` job runs the tests of `crypto`, `store` and `vault` on Windows and macOS, and compiles them for iOS and Android, so platform-specific breakage is caught on every pull request.
- `.github/workflows/release.yml` builds the Apple Silicon `.dmg`. On a `v*` tag it publishes a GitHub release. The tag must match the version in `apps/desktop/package.json`.

## Releasing

1. Bump the version in `apps/desktop/package.json` (`npm version X.Y.Z -w @randomtask/desktop --no-git-tag-version`, with npm 11 so the lockfile format matches) and in `apps/desktop/src-tauri/Cargo.toml` (then `cargo update --workspace` in `src-tauri`). Merge it through a pull request.
2. Tag the merge commit on `main` as `vX.Y.Z` and push the tag. The release workflow builds and publishes the `.dmg`.
3. The `Update casks` workflow in [davidswords/homebrew-tap](https://github.com/davidswords/homebrew-tap) updates the cask daily at 06:17 UTC. Run it by hand to publish sooner.
