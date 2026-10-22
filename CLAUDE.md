# randomnote

A minimal, client-side encrypted, open source todo app built with TypeScript and Tauri 2. Product decisions and open questions are in [docs/decisions.md](docs/decisions.md). Read it before changing behaviour, and update it when a decision is made.

## How we work

- **Test-driven:** write a failing test, make it pass, then refactor. No behaviour without a test.
- **Small atomic steps:** each step is verified working, with tests, before the next one starts.
- **Clean code:** small, well-named functions. Keep domain logic in `packages/core` as pure TypeScript, free of UI and platform code.
- **Ask, don't guess:** when a product decision is open in `docs/decisions.md`, ask instead of assuming.

## Layout

- `packages/core`: domain logic and its tests.
- `apps/desktop`: the Tauri 2 shell (`src-tauri`) and the Vite + TypeScript frontend (`src`).

## Commands

Run these from the repository root. Node's version is in `.nvmrc`.

- `npm ci`: install dependencies.
- `npm test`: run all tests.
- `npm run typecheck`: typecheck all workspaces.
- `npm run dev`: run the desktop app. This needs Rust and the Tauri prerequisites for your OS.

## CI and releases

- `.github/workflows/ci.yml` runs typecheck and tests on pull requests and on pushes to `main`.
- `.github/workflows/release.yml` builds the Apple Silicon `.dmg`. On a `v*` tag it publishes a GitHub release. The tag must match the version in `apps/desktop/package.json`.
