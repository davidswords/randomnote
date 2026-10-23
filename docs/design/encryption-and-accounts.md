# Encryption, accounts, sync and backups

How randomtask protects data, signs people in, syncs and backs up. Decided points are marked **Decided**. Everything else is open and listed at the end.

## Guiding principle (Decided)

**Accounts and encryption behave like Standard Notes,** following its published protocol, version 004 ([specification](https://github.com/standardnotes/app/blob/main/packages/snjs/specification.md)). Where this note and Standard Notes disagree, Standard Notes wins, unless a difference is recorded here on purpose.

The sections marked *(from the specification)* summarise that document. Those marked *to confirm* describe Standard Notes features the specification doesn't cover; they are written from general knowledge and must be checked against Standard Notes before they become tests.

## Keys from the password *(from the specification)*

- The account is an **email and a password**. The password never leaves the device.
- At registration the app makes a random 256-bit **seed**. The **salt** is the first 128 bits of `SHA-256("email:seed")`.
- **Argon2id** turns the password and salt into 512 bits, with 64 MiB of memory, 5 iterations and a parallelism of 1. These settings are deliberately slow, to make guessing passwords expensive.
- The first half is the **master key**; it never leaves the device. The second half is the **server password**, which is all the server ever sees and is what signs the user in.
- The server stores the email, the seed, the server password (to check sign-ins) and the protocol version. It never receives the password, the master key or any unencrypted key.

## Signing in *(from the specification)*

1. The app asks the server for the account's key parameters (seed and version). With two-factor authentication on, this step needs the 2FA code.
2. It derives the keys from the password locally, as above.
3. It signs in with the server password.

A "strict sign-in" option makes the app refuse a server that claims an older protocol version, to block downgrade attacks.

## Encrypting data *(from the specification)*

Three layers of keys:

1. **Master key**, from the password.
2. **Items keys:** random 256-bit keys, each stored encrypted with the master key. Normally there's one per protocol version.
3. **Item keys:** every record gets its own random 256-bit key, stored encrypted with an items key.

- **Cipher:** XChaCha20-Poly1305 with a random 192-bit nonce for every encryption. It's authenticated, so tampering is detected.
- **Format:** `004:nonce:ciphertext:authenticated-data`. The authenticated data is the record's ID and the version (plus the key parameters, for items keys), so an encrypted record can't be moved onto a different record without detection.
- **What the server sees per record:** its ID, which items key encrypted it, and the encrypted key and content. It never sees titles, tags or anything else inside a task.

## Changing or losing the password *(from the specification)*

- **Changing it** derives a new master key and re-encrypts only the items keys. Tasks themselves don't need re-encrypting. A new items key is also made for future data.
- **Losing it:** the specification has no recovery mechanism. Without the password, the data can't be decrypted by anyone, including the server.

## Recovery key (Decided)

- **Optional, never forced.** Signup doesn't require one. Users can create a recovery key in the app's settings at any time.
- **Created on the device and shown once.** The app stores an extra copy of the data keys encrypted with the recovery key; the recovery key itself is never stored. With it, a user who forgets their password can unlock their data and set a new password.
- **Without one, behaviour matches Standard Notes:** a forgotten password means the data can't be recovered. The app says so clearly at signup.
- *Open:* replacing or removing a recovery key, and confirming the user has saved it before it counts.

## Local passcode *(from the specification)*

- An optional **passcode** locks the app on the device. It goes through the same key derivation, with a random ID in place of the email, and the resulting key encrypts the stored master key. Nothing that could verify the passcode is stored; it's correct when decryption succeeds.
- Without a passcode, the master key is kept in the **operating system's keychain** (the macOS Keychain on a Mac).

## Encryption at rest (Decided)

A seized or stolen device, or a drive removed from it, must yield only ciphertext. This is stronger than Standard Notes' default, which keeps the master key in the operating system's keychain unless a local passcode is set.

- **Records are always stored encrypted** in the local database, whether or not a passcode is set. They are decrypted only in memory, in Rust, and only the results cross into the web view.
- **The key is never stored unprotected on disk.** By default the password unlocks the app at every launch, so the key exists only in memory while the app is open.
- **Staying signed in is opt-in:** "stay signed in on this device" stores the key in the operating system's keychain (the macOS Keychain), trading some protection for convenience.
- **The app can be locked** on demand and after a period of inactivity, which clears the key from memory.
- **No plaintext side copies:** search indexes, caches, logs, crash reports and temporary files must not contain task contents.
- **Tested end to end:** after adding tasks, a test scans the files on disk and checks no task title appears in plain text.
- **Limits:** this protects a device that is off or locked. While the app is unlocked, data is in memory. Operating-system full-disk encryption (such as FileVault) also covers swap and hibernation files, and is recommended alongside.

## Two-factor authentication *(to confirm)*

Time-based one-time codes (TOTP) from an authenticator app, with a one-off recovery code for when the authenticator is lost.

## Backups

- **Email backups (Decided):** an email with an encrypted backup file, at a frequency the user chooses: **weekly, monthly or quarterly**. There's no daily option, to keep sending volume reasonable. Probably a paid feature.
- **Import and restore (Decided):** the app can import a backup file and decrypt it with the password, so users can check a backup works before they need it.
- *To confirm:* Standard Notes also offers backups to cloud storage (such as Dropbox, Google Drive and OneDrive) and automatic local backups from the desktop app, all importable to restore.
- Backups contain encrypted data, so they're only as readable as the password allows.

## Sync

- **Two ways to sync, one client (Decided).** The client encrypts everything before it leaves the device, so storage only holds ciphertext. Both options sit behind one sync interface in the client:
  - **Bring your own storage:** the app syncs straight to storage the user controls (for example their own S3 bucket or a synced folder). No randomtask server is involved.
  - **Hosted service:** a randomtask server handles accounts, sessions, two-factor authentication, quotas, billing, email backups and a hosted web app. It never sees passwords, keys or task contents.
- **Each user has their own storage bucket (Decided).** Users' data never shares a bucket. In the hosted service the server creates a bucket per user and gives the app short-lived credentials limited to that bucket, so the app reads and writes its encrypted data directly. Cloud providers cap buckets per account, so very large numbers of users may need several cloud accounts.
- **Sync can be switched on or off (Decided).** With it off, the app works purely locally.
- **Uploads can be size-limited (Decided).** For example a per-account quota, enforced by the server and shown by the app.
- **Where data is stored is flexible (Decided).** randomtask is open source, so people can bring their own storage or self-host. The likely default is S3-compatible object storage (Amazon S3, Cloudflare R2, MinIO and others), behind an interface so other storage can plug in.
- *To confirm:* Standard Notes' server is open source and self-hostable, storing records in a database and files in S3 or on disk.

## First proof of concept (Decided)

No server yet. On first launch the user creates an account with a **username and password** on the device. Keys are derived exactly as above, with the username in place of the email, and tasks are encrypted in the local database. Each launch asks for the password to unlock them. The server password half is unused until a server exists.

- **Sign-up:** the password is typed twice and must be at least **12 characters**. The sign-up screen warns that a forgotten password can't be recovered without a recovery key.
- **Unlocking:** a wrong password shows "Wrong password" and unlocks nothing. It's detected when the stored items key fails to decrypt.
- **Tasks from before 0.3.0 start fresh:** they were stored as plain text (randomnote 0.2.0's data folder, and randomtask 0.2.1's `tasks.db`), so on launch randomtask deletes them rather than migrating them. Deleting can't guarantee the old bytes are gone from an SSD, which is another reason to use full-disk encryption.

## Open questions

- **The hosted server.** Write our own, or adapt Standard Notes' open source server? Standard Notes keeps all users' records in one database, so adapting it to a bucket per user would be a significant change.
- **Using the app without an account.** Standard Notes works offline without an account. Is local data encrypted then (only with a passcode, or always)?
- **Where the cryptography runs.** In Rust, for performance, as decided for storage. A future web version would need the same code in the browser, for example compiled to WebAssembly.
- **Size limits.** What the limit is, whether it differs by plan, and what happens when it's reached.
- **Email backups.** Whether they're definitely a paid feature, and which email service sends them.
- **Accounts versus the roadmap.** In this design the encryption keys come from the account password, so encryption and accounts are built together rather than as separate stages.
