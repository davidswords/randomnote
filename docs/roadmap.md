# Roadmap

The order the core of randomnote is built in. Each stage is built test-first and verified before the next one starts. Open questions for each stage are in [decisions.md](decisions.md).

1. **Task entity.** A task with a title. *In progress.*
2. **Local database.** Save and load tasks on the device.
3. **Encryption.** Encrypt the database on the device, so only the user can read it.
4. **Sync up.** Send the encrypted data to a sync location.
5. **Sync down.** Pull the encrypted data onto another device and merge it.
6. **Accounts.** Username and password, two-factor authentication, and recovery keys.

Tags, smart lists and the UI build on these stages. A full list of features is still to be mapped out.
