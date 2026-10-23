import { AccountError, type Accounts } from "@randomtask/core";
import type { Invoke } from "./tauri-task-repository";

/** Creates and unlocks the account through the Rust account commands. */
export class TauriAccounts implements Accounts {
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

  /** Tauri rejects with the command's error message as a plain string. */
  private async call(command: string, args: Record<string, unknown>): Promise<void> {
    try {
      await this.invoke<null>(command, args);
    } catch (message) {
      throw new AccountError(String(message));
    }
  }
}
