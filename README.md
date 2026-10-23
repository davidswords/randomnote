# randomtask

A minimal, client-side encrypted todo app. It does todos and nothing else: no calendar, notes or timers. Tasks are organised with hierarchical tags.

randomtask is at an early stage. It keeps a simple task list, encrypted on your device under a local account password, and the rest of the todo features are being built test-first. The decisions behind it are in [docs/decisions.md](docs/decisions.md).

## Install

### macOS (Apple Silicon) with Homebrew

```
brew tap davidswords/tap
brew trust --cask davidswords/tap/randomtask
brew install randomtask
```

randomtask was called randomnote up to version 0.2.0. If you installed it then, `brew upgrade` moves you to randomtask. Tasks from 0.2.0 were stored unencrypted, so randomtask deletes them rather than carrying them over.

randomtask isn't signed with an Apple Developer ID yet, so macOS blocks it on first launch. To open it, go to System Settings → Privacy & Security and click **Open Anyway**.

### Download

Each [release](https://github.com/davidswords/randomtask/releases) has a `.dmg` for Apple Silicon Macs.

Linux, Windows and Intel Mac builds aren't published yet. You can build randomtask yourself on any of them.

## Build from source

You need:

- Node.js, at the version in [`.nvmrc`](.nvmrc).
- Rust and the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

```
npm ci
npm run dev
```

To run the checks CI runs:

```
npm run typecheck
npm test
```

## Licence

[GPL-3.0-only](LICENSE)
