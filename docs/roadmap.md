# Roadmap

The order the core of randomtask is built in. Each stage is built test-first and verified before the next one starts. Open questions for each stage are in [decisions.md](decisions.md).

1. **Task entity.** A task with a title. *Done.*
2. **Local database.** Save and load tasks on the device, with a minimal task list and an end-to-end test. *Done in 0.2.0.*
3. **Encryption.** Encrypt the database on the device, so only the user can read it. *Done:* an email and password (Standard Notes 004 keys) encrypt every task, and the app asks for the password at each launch.
4. **No account needed.** The app opens empty and ready to use. Tasks stay unencrypted until the user secures them with an email and password from the account menu. *Done:* until the server exists, the account is created on the device.
5. **The randomtask server.** In Rust: register and sign in as in Standard Notes, so accounts live on the server. *In progress:* registration and signing in are done.
6. **Subscription and sync.** Payment, then sync through the server into the user's own bucket, and restoring on another device.
7. **The web app.** The same frontend with the Rust core compiled to WebAssembly, as the site people reach.
8. **Two-way sync, two-factor authentication and recovery,** as in Standard Notes.
9. **Backups.** A local backup folder (on by default) with restore, then email backups.

The design for stages 3 to 9 is in [design/encryption-and-accounts.md](design/encryption-and-accounts.md). Following Standard Notes, the encryption keys come from the account password, so encryption and accounts may be built together.

Tags, smart lists and the UI build on these stages. A full list of features is still to be mapped out.
