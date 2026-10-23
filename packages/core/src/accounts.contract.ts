import { describe, expect, it } from "vitest";
import { AccountError, type Accounts } from "./account";

const EMAIL = "david@example.com";
const PASSWORD = "correct horse battery staple";

/**
 * The behaviour every Accounts implementation must have. The in-memory one
 * runs it as a unit test; the Tauri adapter runs it against a fake of the
 * Rust commands, whose real behaviour the Rust tests pin.
 */
export function accountsContract(makeAccounts: () => Accounts): void {
  describe("Accounts contract", () => {
    it("has no account at first", async () => {
      const accounts = makeAccounts();

      expect(await accounts.hasAccount()).toBe(false);
    });

    it("has an account once one is created", async () => {
      const accounts = makeAccounts();

      await accounts.createAccount(EMAIL, PASSWORD);

      expect(await accounts.hasAccount()).toBe(true);
    });

    it("refuses a password shorter than 12 characters", async () => {
      const accounts = makeAccounts();

      await expect(accounts.createAccount(EMAIL, "too short")).rejects.toThrow(
        new AccountError("The password must be at least 12 characters."),
      );
      expect(await accounts.hasAccount()).toBe(false);
    });

    it("refuses an invalid email", async () => {
      const accounts = makeAccounts();

      await expect(accounts.createAccount("david", PASSWORD)).rejects.toThrow(
        new AccountError("Enter a valid email address."),
      );
      expect(await accounts.hasAccount()).toBe(false);
    });

    it("refuses a second account", async () => {
      const accounts = makeAccounts();
      await accounts.createAccount(EMAIL, PASSWORD);

      await expect(accounts.createAccount("someone@example.com", PASSWORD)).rejects.toThrow(
        new AccountError("An account already exists on this device."),
      );
    });

    it("unlocks with the right password", async () => {
      const accounts = makeAccounts();
      await accounts.createAccount(EMAIL, PASSWORD);

      await expect(accounts.unlock(PASSWORD)).resolves.toBeUndefined();
    });

    it("refuses the wrong password", async () => {
      const accounts = makeAccounts();
      await accounts.createAccount(EMAIL, PASSWORD);

      await expect(accounts.unlock("incorrect horse battery")).rejects.toThrow(
        new AccountError("Wrong password"),
      );
    });

    it("refuses to unlock without an account", async () => {
      const accounts = makeAccounts();

      await expect(accounts.unlock(PASSWORD)).rejects.toThrow(
        new AccountError("There is no account on this device."),
      );
    });
  });
}
