# Decisions

Product and technical decisions for randomnote. Add to this file when a decision is made or changed, and move questions out of "Open questions" once they are answered.

## Product

- **A todo app and nothing else.** No calendar, notes, timers or other features that belong in a separate app. The model is Apple's approach of standalone apps that each do one thing.
- **Minimal.** The design takes inspiration from Apple Reminders. The privacy and encryption model takes inspiration from Standard Notes and Notesnook.
- **Open source first.** Anyone can download, build and run it on their own hardware from GitHub. The licence is GPL-3.0-only.
- **Client-side encrypted.** Data is encrypted on the device.
- **Highly performant.**

## Tasks

- **A task has a title.** Surrounding whitespace is trimmed; whitespace inside the title is kept as typed.
- **A title cannot be empty** or whitespace-only. Creating such a task fails with `InvalidTaskTitleError`.
- **Titles have no length limit** for now, and accept any Unicode text.
- **Tasks are listed oldest first,** with new tasks added at the bottom, as in Apple Reminders.
- **An empty title shows "A task needs a title."** and saves nothing.
- **Renaming changes the task in place.** It stays the same task, with the same ID and record.
- **Recurring tasks are a series plus occurrences,** and behave exactly like Apple Reminders. See [design/recurring-tasks.md](design/recurring-tasks.md).

## Organising tasks

- **Hierarchical tags.** Tags form a tree, and a task can have several tags.
- **A list is a level in the tag hierarchy.** There is no separate list or project concept.
- **The tag hierarchy can change.** Tags can be renamed and moved, and tasks keep their tags through those changes.
- **Tags can be archived,** so finished projects stop cluttering the sidebar. Apple Reminders can't do this, and old lists pile up.
- **Archiving a tag archives its child tags too.**
- **No built-in Inbox, Projects or Areas sections.** Everything is a tag.
- **A default `#inbox` tag.** It is created by default and pinned at the top, but it is an ordinary tag, not a special section.
- **Every task has at least one tag.** A new task is tagged `#inbox` automatically, and a task can never be left with no tags.
- **`#inbox` is not removed automatically.** A task can be in `#inbox` and other tags at the same time.
- **Removing a task's last tag puts it back in `#inbox`.**
- **Any tag can be pinned** to the top.
- **Smart lists** are saved queries over several tags, and they can be pinned too.

## Platforms and distribution

- **Desktop:** macOS, Linux and Windows. **Mobile** comes later.
- **The first release target is macOS on Apple Silicon, installed with Homebrew.** The app is ad-hoc signed and not notarized for now, so Gatekeeper warns on first launch.

## Technology

- **TypeScript and Tauri 2,** with npm workspaces:
  - `packages/core` holds the domain logic, as pure TypeScript with no UI or platform code.
  - `apps/desktop` holds the Tauri shell and frontend.
- **Vitest** for tests.
- **IDs are UUID v7,** generated on the device. They sort by creation time. Occurrences of recurring tasks are the exception (see the design note).
- **Storage and sync encrypt each record separately,** as Standard Notes and Notesnook do. Sync sends only changed records, conflicts are resolved per record, and the server only sees ciphertext.
- **Storage sits behind interfaces in `core`,** with an in-memory version for unit tests and real adapters in the apps.
- **Accounts and encryption behave like Standard Notes** (protocol 004): Argon2id keys from the password, per-record XChaCha20-Poly1305 encryption, an optional local passcode. See [design/encryption-and-accounts.md](design/encryption-and-accounts.md).
- **Email backups:** encrypted, weekly, monthly or quarterly at the user's choice, probably a paid feature. The app can **import a backup** to check it restores.
- **Sync has two modes behind one client interface:** bring your own storage (no randomnote server) and a hosted service with a randomnote server.
- **Each user has their own storage bucket;** users' data never shares one.
- **The first accounts proof of concept has no server:** a local username and password that encrypt the data on the device.
- **Sync can be switched off, uploads can be size-limited, and storage is flexible,** with S3-compatible storage as the likely default.
- **Local storage is SQLite, accessed from Rust** with rusqlite and a bundled SQLite, in the `crates/store` crate. It was chosen for performance: queries, and later decryption, run natively, and only results cross into the web view.

## Open questions

- Does viewing a parent tag also show tasks from its child tags?
- Is a task created while viewing another tag (for example `Work`) tagged `#inbox`, or with that tag?
- How does a task show if it has both archived and active tags?
- Can a child tag be unarchived while its parent stays archived?
- What happens to a tag's tasks when the tag is deleted?
- Smart lists: which operators do queries support (all of these tags, any of them, excluding a tag), and can they filter on anything besides tags?
- Encryption, accounts, sync and backups: see the open questions in [design/encryption-and-accounts.md](design/encryption-and-accounts.md).
- Sync conflicts: how conflicting changes to the same record from two devices are merged.
- Web version: possibly later, with a browser storage adapter (SQLite compiled to WebAssembly, or IndexedDB) behind the same `TaskRepository`. Encryption code would then need to run in the browser too.
- Business model and licence: whether randomnote stays open source with paid sync, and whether the repository stays public.
