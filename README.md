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

The server for api.randomtask.ch is in [`crates/server`](crates/server). The app doesn't talk to it yet; for now you can run it and try its API. The easiest way is [Docker](https://docs.docker.com/get-docker/):

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

### The checks CI runs

```
npm run typecheck
npm test
```

Each Rust crate under [`crates`](crates) and [`apps/desktop/src-tauri`](apps/desktop/src-tauri) has its own tests: run `cargo test` in its directory. [CLAUDE.md](CLAUDE.md) lists every command.

## Licence

[GPL-3.0-only](LICENSE)
