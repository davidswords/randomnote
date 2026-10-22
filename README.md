# randomnote

A minimal, client-side encrypted todo app. It does todos and nothing else: no calendar, notes or timers. Tasks are organised with hierarchical tags.

randomnote is at an early stage. Version 0.1 is only a desktop shell, and the todo features are being built test-first. The decisions behind it are in [docs/decisions.md](docs/decisions.md).

## Install

### macOS (Apple Silicon) with Homebrew

```
brew tap davidswords/tap
brew trust --cask davidswords/tap/randomnote
brew install randomnote
```

randomnote isn't signed with an Apple Developer ID yet, so macOS blocks it on first launch. To open it, go to System Settings → Privacy & Security and click **Open Anyway**.

### Download

Each [release](https://github.com/davidswords/randomnote/releases) has a `.dmg` for Apple Silicon Macs.

Linux, Windows and Intel Mac builds aren't published yet. You can build randomnote yourself on any of them.

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
