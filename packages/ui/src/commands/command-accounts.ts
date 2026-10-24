import { AccountError, type Accounts } from "@randomtask/core";
import type { Invoke } from "./command-task-repository";

/** Creates and unlocks the account through the Rust account commands, by
 * Tauri on the desktop or the web app's worker in the browser. */
export class CommandAccounts implements Accounts {
  constructor(private readonly invoke: Invoke) {}

  hasAccount(): Promise<boolean> {
    return this.invoke<boolean>("has_account");
  }

  async createAccount(email: string, password: string): Promise<void> {
    await this.call("create_account", { email, password });
  }

  async unlock(password: string): Promise<void> {
    await this.call("unlock", { password });
  }

  /** A refused command rejects with its error message as a plain string. */
  private async call(command: string, args: Record<string, unknown>): Promise<void> {
    try {
      await this.invoke<null>(command, args);
    } catch (message) {
      throw new AccountError(String(message));
    }
  }
}
