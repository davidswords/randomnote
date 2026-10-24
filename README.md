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

You can run the desktop app, the web app and the server from a copy of this repository.

### 1. Install the tools, once

- **Node.js**, at the version in [`.nvmrc`](.nvmrc). With Homebrew: `brew install node`.
- **Rust, through rustup.** Homebrew's `rust` formula can't add the WebAssembly target the web app needs, so use rustup:

  ```
  brew install rustup
  rustup-init
  ```

  On other systems, follow [rustup.rs](https://rustup.rs). Then open a new terminal.
- **Cargo's tools on your `PATH`.** Programs installed with `cargo install` go in `~/.cargo/bin`. If `ls ~/.cargo/bin` lists them but your shell can't find them, add the folder to your `PATH`:

  ```
  echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc
  ```

- **For the desktop app:** the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.
- **For the web app:** Rust's WebAssembly target and `wasm-bindgen`, at the version in the lockfile:

  ```
  rustup target add wasm32-unknown-unknown
  cargo install wasm-bindgen-cli --version 0.2.129 --locked
  ```

  `npm run dev:web` checks these first, and says how to install whatever's missing.

### 2. Get the code

```
git clone https://github.com/davidswords/randomtask.git
cd randomtask
npm ci
```

After every `git pull`, run `npm ci` again, so new packages in the repository are installed.

### 3. Run it

| To run | Command | Then |
|---|---|---|
| The server | `cd crates/server && cargo run` | It listens on `http://127.0.0.1:8080`. |
| The web app | `npm run dev:web` | Open <http://localhost:1430>. |
| The desktop app | `npm run dev` | Its window opens. |
| The desktop app with the server | `RANDOMTASK_SERVER=http://127.0.0.1:8080 npm run dev` | Its window opens. |

Run the server in its own terminal, and the app in another. The first run of each compiles the Rust, which takes a few minutes; later runs start in seconds.

- **The web app** registers accounts with the server on `127.0.0.1:8080`, so creating a free account needs the server running. It keeps its tasks in the browser's private storage, separate from the desktop app's; to start afresh, clear the site's data for `localhost:1430` in your browser.
- **The desktop app** keeps accounts on the device unless `RANDOMTASK_SERVER` is set, as releases do until api.randomtask.ch is live. With it set, creating a free account registers it with the server first, and unlocking an account made without a server registers it too. Tasks stay on each device until sync is built.

### The server with Docker

Instead of `cargo run`, you can run the server with [Docker](https://docs.docker.com/get-docker/), or [OrbStack](https://orbstack.dev) on a Mac:

```
docker compose up --build
```

It listens on `http://127.0.0.1:8080`, lets the web app's dev server call it, and keeps its database in a Docker volume. Stop it with `docker compose down`; add `-v` to delete its database too. To check the server is running, however you started it:

```
curl -X POST http://127.0.0.1:8080/v2/login-params \
  -H 'content-type: application/json' \
  -d '{"email": "you@example.com", "code_challenge": "test"}'
```

### If something goes wrong

- **`Missing script: "dev:web"`:** your copy is older than the web app. Run `git pull`, then `npm ci`.
- **`The following dependencies are imported but could not be resolved: @randomtask/ui`:** run `npm ci`, then start again.
- **`npm run dev:web` stops with "The web app needs a few tools first":** run the commands it lists.
- **Creating an account says it can't reach the server:** start the server, as above.

### The checks CI runs

```
npm run typecheck
npm test
```

Each Rust crate under [`crates`](crates) and [`apps/desktop/src-tauri`](apps/desktop/src-tauri) has its own tests: run `cargo test` in its directory. The web app's end-to-end tests run in Chromium against the real server: build the server (`cargo build` in `crates/server`), then `npm run build:e2e -w @randomtask/web` and `npm run test:e2e -w @randomtask/web`. [CLAUDE.md](CLAUDE.md) lists every command.

## Hosting

randomtask.ch and api.randomtask.ch run on one server with Docker Compose. [deploy/README.md](deploy/README.md) explains how to set it up and update it.

## Licence

[GPL-3.0-only](LICENSE)
