# Decisions

Product and technical decisions for randomnote. Add to this file when a decision is made or changed, and move questions out of "Open questions" once they are answered.

## Product

- **A todo app and nothing else.** No calendar, notes, timers or other features that belong in a separate app. The model is Apple's approach of standalone apps that each do one thing.
- **Minimal.** The design takes inspiration from Apple Reminders. The privacy and encryption model takes inspiration from Standard Notes and Notesnook.
- **Open source first.** Anyone can download, build and run it on their own hardware from GitHub. The licence is GPL-3.0-only.
- **Client-side encrypted.** Data is encrypted on the device.
- **Highly performant.**

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

## Open questions

- Does viewing a parent tag also show tasks from its child tags?
- Removing a task's last tag: is it refused, or does the task go back to `#inbox`?
- Is a task created while viewing another tag (for example `Work`) tagged `#inbox`, or with that tag?
- How does a task show if it has both archived and active tags?
- Can a child tag be unarchived while its parent stays archived?
- What happens to a tag's tasks when the tag is deleted?
- Smart lists: which operators do queries support (all of these tags, any of them, excluding a tag), and can they filter on anything besides tags?
- Encryption: the algorithm, key derivation, what is encrypted, and what happens if the password is lost.
- Sync between devices: whether to have it, and how.
