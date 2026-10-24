# randomtask

A minimal, client-side encrypted todo app. It does todos and nothing else: no calendar, notes or timers. Tasks are organised with hierarchical tags.

randomtask is at an early stage. It keeps a simple task list on your device, encrypted once you create a free account, and the rest of the todo features are being built test-first. The decisions behind it are in [docs/decisions.md](docs/decisions.md).

## Install

### macOS (Apple Silicon) with Homebrew

```
brew tap davidswords/tap
brew trust --cask davidswords/tap/randomtask
brew install randomtask
```

randomtask was called randomnote up to version 0.2.0. If you installed it then, `brew upgrade` moves you to randomtask. Versions before 0.3.0 stored tasks unencrypted, so randomtask deletes them rather than carrying them over.

randomtask isn't signed with an Apple Developer ID yet, so macOS blocks it on first launch. To open it, go to System Settings → Privacy & Security and click **Open Anyway**.

### Download

Each [release](https://github.com/davidswords/randomtask/releases) has a `.dmg` for Apple Silicon Macs.

Linux, Windows and Intel Mac builds aren't published yet. You can build randomtask yourself on any of them.

## Run it from source

After cloning the repository, you can run the app, the server, or both.

### The app

You need:

- Node.js, at the version in [`.nvmrc`](.nvmrc).
- Rust and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

```
npm ci
npm run dev
```

### The server

The server for api.randomtask.ch is in [`crates/server`](crates/server). It isn't hosted yet, so released apps keep accounts on the device. You can run it yourself and point the app at it. The easiest way is [Docker](https://docs.docker.com/get-docker/):

```
docker compose up --build
```

It listens on `http://127.0.0.1:8080` and keeps its database in a Docker volume. To check it's running, ask it for an account's key parameters:

```
curl -X POST http://127.0.0.1:8080/v2/login-params \
  -H 'content-type: application/json' \
  -d '{"email": "you@example.com", "code_challenge": "test"}'
```

Stop it with `docker compose down`. Add `-v` to delete its database too.

Without Docker, run it with Rust:

```
cd crates/server
cargo run
```

### The app with the server

With the server running, start the app pointed at it:

```
RANDOMTASK_SERVER=http://127.0.0.1:8080 npm run dev
```

Creating a free account then registers it with the server first, and fails with a message if the server can't be reached. Unlocking an account made without a server registers it too. Tasks themselves stay on the device until sync is built.

### The web app in your browser

The same app runs in the browser, with its Rust compiled to WebAssembly. Besides Node.js and Rust, it needs the WebAssembly target and `wasm-bindgen`, at the version in the lockfile:

```
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
```

With the server running (`docker compose up --build`, or `cargo run` in `crates/server`), start the web app:

```
npm run dev:web
```

Then open <http://localhost:1430>. It keeps its data in the browser's private storage, separate from the desktop app's, and registers accounts with the server on `127.0.0.1:8080`. To start afresh, clear the site's data in your browser.

### The checks CI runs

```
npm run typecheck
npm test
```

Each Rust crate under [`crates`](crates) and [`apps/desktop/src-tauri`](apps/desktop/src-tauri) has its own tests: run `cargo test` in its directory. The web app's end-to-end tests run in Chromium against the real server: build the server (`cargo build` in `crates/server`), then `npm run build:e2e -w @randomtask/web` and `npm run test:e2e -w @randomtask/web`. [CLAUDE.md](CLAUDE.md) lists every command.

## Licence

[GPL-3.0-only](LICENSE)
