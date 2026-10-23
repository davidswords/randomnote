# Decisions

Product and technical decisions for randomtask. Add to this file when a decision is made or changed, and move questions out of "Open questions" once they are answered.

## Product

- **The app is called randomtask.** It was first published as randomnote (up to 0.2.0); from 0.2.1 the repository, app, bundle ID, Homebrew cask and package names are all randomtask.
- **A todo app and nothing else.** No calendar, notes, timers or other features that belong in a separate app. The model is Apple's approach of standalone apps that each do one thing.
- **Minimal.** The design takes inspiration from Apple Reminders. The privacy and encryption model takes inspiration from Standard Notes and Notesnook.
- **Open source first.** Anyone can download, build and run it on their own hardware from GitHub. The licence is GPL-3.0-only.
- **Client-side encrypted.** Data is encrypted on the device.
- **Highly performant.**
- **A dark theme that's comfortable to look at,** in the spirit of terminal and editor themes such as Zed's: a soft charcoal background rather than black, muted text and a few calm accent colours. No blue-and-white look. The colours live in one set of variables.

## Pricing

- **Free, with no limits:** the app, local use and a free account with encryption.
- **One paid plan, Sync:** sync across devices and the web app, plus email backups once they exist. Billed monthly or yearly, yearly by default, with a free trial of sync. No lifetime plans, because the server's costs never stop.
- **Self-hosting the open-source server is free.**
- **Payments go through a merchant of record** (such as Paddle or Lemon Squeezy), which handles VAT and sales tax worldwide. Cancelling stops sync but leaves the data on every device.
- **The window shows the app's name and version** (for example "randomtask 0.2.1") at the bottom, on every screen.

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
- **Unarchiving a tag also unarchives its archived parents,** so an active tag never sits under an archived one.
- **Archived tags are hidden on active tasks.** A task that also has an active tag still shows there, without the archived tag.
- **Selecting a tag shows its tasks and every task in its child tags.**
- **No built-in Inbox, Projects or Areas sections.** Everything is a tag.
- **A default `#inbox` tag.** It is created by default and pinned at the top, but it is an ordinary tag, not a special section.
- **Every task has at least one tag.** A new task gets the selected tag, like a file created in a folder, or `#inbox` when no tag is selected. A task can never be left with no tags.
- **`#inbox` is not removed automatically.** A task can be in `#inbox` and other tags at the same time.
- **Removing a task's last tag puts it back in `#inbox`.**
- **Smart lists are saved filters.** They match *all* or *any* of a set of filters on task properties, such as tags, due date, importance and flags, as in Apple Reminders. Like tags, they have a colour and an icon.
- **The sidebar has two parts,** as in Apple Reminders:
  - **Pinned:** tags and smart lists shown as tiles (icon, colour, count of open tasks) in an order the user arranges.
  - **Everything else:** tags listed alphabetically at every level of the hierarchy.

## Priority and expiry

- **Importance** is set on tags and on tasks, from 1 to 10 (10 is the most important). 7 isn't allowed, so there's no safe middle choice. The default is 1.
- **Urgency comes from the due date and time.** It isn't set by hand.
- **A task can have an estimated duration.**
- **The priority score** combines tag importance, task importance, urgency and estimated duration. An important, urgent task in a low-importance tag can rank above minor tasks in an important tag.
- **Smart lists sort by any single field** (such as due date or importance) **or by the priority score.**
- **An expiry window** can be set on a tag or a task: a whole number of hours, days, weeks, months or years. Expiry is separate from the due date.
- **A task's window counts from when the task was created,** or for a recurring task, from when each occurrence was created. When it passes:
  - a one-off task is archived;
  - an occurrence of a recurring task is set aside, and the next occurrence appears at its usual time.
- **When a tag's window passes,** the tag and all its child tags expire together and are archived, with their tasks. A task that also has a tag elsewhere in the hierarchy stays active there, without the expired tag.

  This is a deliberate difference from Apple Reminders, which has no expiry.

## Platforms and distribution

- **Desktop:** macOS, Linux and Windows. **Mobile** comes later.
- **The first release target is macOS on Apple Silicon, installed with Homebrew.** The app is ad-hoc signed and not notarized for now, so Gatekeeper warns on first launch.
- **`randomtask` in a terminal opens the app,** as clicking it does. The Homebrew cask installs the command.
- **A quick-add command comes after "stay signed in":** `randomtask "a task"` adds a task from the terminal, unlocking with the key in the keychain.

## Technology

- **TypeScript and Tauri 2,** with npm workspaces:
  - `packages/core` holds the domain logic, as pure TypeScript with no UI or platform code.
  - `apps/desktop` holds the Tauri shell and frontend.
- **Vitest** for tests.
- **IDs are UUID v7,** generated on the device. They sort by creation time. Occurrences of recurring tasks are the exception (see the design note).
- **Storage and sync encrypt each record separately,** as Standard Notes and Notesnook do. Sync sends only changed records, conflicts are resolved per record, and the server only sees ciphertext.
- **Storage sits behind interfaces in `core`,** with an in-memory version for unit tests and real adapters in the apps.
- **Accounts and encryption behave like Standard Notes** (protocol 004): Argon2id keys from the password, per-record XChaCha20-Poly1305 encryption, an optional local passcode. See [design/encryption-and-accounts.md](design/encryption-and-accounts.md).
- **A local backup folder, on by default:** an encrypted copy of the account in `~/randomtask/<email>/` (or a location the user chooses), laid out as in S3, as a mirror plus periodic snapshots. A fresh install can restore from it with the password alone. It's a backup, not a sync method.
- **Email backups:** encrypted, weekly, monthly or quarterly at the user's choice (no daily option), probably a paid feature. The app can **import a backup** to check it restores.
- **Sync goes through the randomtask server,** which alone holds the S3 credentials and writes each user's encrypted records to that user's own bucket. Bringing your own bucket comes later, by self-hosting the open-source server.
- **Each user has their own storage bucket;** users' data never shares one.
- **The server is our own, written in Rust,** following Standard Notes' registration and sign-in protocol rather than adapting its server, which is TypeScript and keeps every user's records in one database.
- **Creating a free account needs the server,** as in Standard Notes. If it can't be reached, sign-up fails with a message and the tasks stay unencrypted.
- **Accounts made on the device in 0.3 and 0.4 are registered with the server when next unlocked,** with the same email and keys, so nothing changes for the user.
- **The app's server client is built and tested before hosting is chosen,** against the real server code, but stays out of releases until the server is live.
- **The server sets a minimum app version.** An app older than that shows "Please update randomtask" and stops syncing until it's updated. The minimum is raised whenever a release changes the task format, so an older app can never sync away fields it doesn't know about.
- **The domain is randomtask.ch.** Where the server is hosted is decided before sync, the first stage that needs it live.
- **Encryption at rest:** once an email and password are added, records are always stored encrypted on the device and decrypted only in memory. Before that, tasks are stored unencrypted, as in Standard Notes, and the app says so. The key is never stored unprotected; staying signed in through the operating system's keychain is opt-in.
- **An optional recovery key** can be created in the app at any time; it is never forced. Without one, a forgotten password means the data is lost, as in Standard Notes.
- **An account is an email and a password,** never a username, so we can always reach the user. The email is stored trimmed and in lower case, because the keys are derived from it.
- **Three tiers, as in Standard Notes:**
  - **No account:** the app works the moment it opens, empty and ready to use, with no sign-up.
  - **A free account** (email and password) is registered with the randomtask server and encrypts the device's data. The same account works on every device and in the browser.
  - **A subscription** turns on sync.
- **The account menu sits at the bottom of the sidebar,** as in Standard Notes, with *Create free account*, *Sign in*, *Import*, *Help* and the version. A dismissible notice says when data isn't backed up, with a *Sign up to sync* button.
- **One implementation for desktop and web:** the same frontend and the same Rust core, run by Tauri on the desktop and compiled to WebAssembly in the browser. The web version is the site people reach.
- **Passwords** are typed twice at sign-up and are at least 12 characters.
- **Unencrypted tasks from versions before 0.3.0 are deleted on launch,** not migrated.
- **Sync can be switched off, uploads can be size-limited, and storage is flexible,** with S3-compatible storage as the likely default.
- **The Rust is split by where it runs:** `crates/client` holds what runs on people's devices (the desktop, web, iOS and Android apps), and `crates/server` holds what runs at api.randomtask.ch.
- **Local storage is SQLite, accessed from Rust** with rusqlite and a bundled SQLite, in the `crates/client/store` crate. It was chosen for performance: queries, and later decryption, run natively, and only results cross into the web view.

## Open questions

- What happens to a tag's tasks when the tag is deleted?
- Which task properties smart lists can filter on, and whether a filter can exclude a tag.
- The priority formula: how tag importance, task importance, urgency and estimated duration are weighed.
- When a tag's expiry window starts counting (for example, from when the tag was created).
- Where can archived and expired tasks be seen, and can they be restored?
- Encryption, accounts, sync and backups: see the open questions in [design/encryption-and-accounts.md](design/encryption-and-accounts.md).
- Sync conflicts: how conflicting changes to the same record from two devices are merged.
- The server: where it's hosted, and whether emails are verified.
- Browser storage for the web version: SQLite compiled to WebAssembly, or IndexedDB.
- Pricing details: the price (around $3 a month or $30 a year was suggested), the length of the free trial, and which merchant of record.
- Whether the repository stays public.
