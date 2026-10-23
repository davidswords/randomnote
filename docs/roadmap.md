# Roadmap

The order the core of randomtask is built in. Each stage is built test-first and verified before the next one starts. Open questions for each stage are in [decisions.md](decisions.md).

1. **Task entity.** A task with a title. *Done.*
2. **Local database.** Save and load tasks on the device, with a minimal task list and an end-to-end test. *Done in 0.2.0.*
3. **Encryption.** Encrypt the database on the device, so only the user can read it. *Done:* a local username and password (Standard Notes 004 keys) encrypt every task, and the app asks for the password at each launch.
4. **Sync up.** Send the encrypted data to a sync location.
5. **Sync down.** Pull the encrypted data onto another device and merge it.
6. **Accounts.** Email and password, two-factor authentication and recovery, as in Standard Notes.
7. **Backups.** Email backups, and possibly cloud and local backups.

The design for stages 3 to 7 is in [design/encryption-and-accounts.md](design/encryption-and-accounts.md). Following Standard Notes, the encryption keys come from the account password, so encryption and accounts may be built together.

Tags, smart lists and the UI build on these stages. A full list of features is still to be mapped out.
