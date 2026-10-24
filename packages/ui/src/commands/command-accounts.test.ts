import { accountsContract } from "@randomtask/core/testing";
import { describe, expect, it } from "vitest";
import { fakeAccountCommands } from "./fake-account-commands";
import { CommandAccounts } from "./command-accounts";

accountsContract(() => new CommandAccounts(fakeAccountCommands().invoke));

describe("CommandAccounts messages", () => {
  it("checks for an account with has_account", async () => {
    const commands = fakeAccountCommands();

    await new CommandAccounts(commands.invoke).hasAccount();

    expect(commands.calls).toEqual([{ command: "has_account", args: undefined }]);
  });

  it("creates the account with create_account, passing the email and password", async () => {
    const commands = fakeAccountCommands();

    await new CommandAccounts(commands.invoke).createAccount(
      "david@example.com",
      "correct horse battery staple",
    );

    expect(commands.calls).toEqual([
      {
        command: "create_account",
        args: { email: "david@example.com", password: "correct horse battery staple" },
      },
    ]);
  });

  it("unlocks with unlock, passing the password", async () => {
    const commands = fakeAccountCommands();
    const accounts = new CommandAccounts(commands.invoke);
    await accounts.createAccount("david@example.com", "correct horse battery staple");

    await accounts.unlock("correct horse battery staple");

    expect(commands.calls.at(-1)).toEqual({
      command: "unlock",
      args: { password: "correct horse battery staple" },
    });
  });
});

describe("fake account commands", () => {
  it("rejects unknown commands, as Tauri and the worker do", async () => {
    const { invoke } = fakeAccountCommands();

    await expect(invoke("delete_everything")).rejects.toBe(
      "Unknown command: delete_everything",
    );
  });
});
