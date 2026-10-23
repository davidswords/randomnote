import { AccountError, type Accounts, checkEmail, checkPassword } from "./account";

/** Keeps one account in memory, for tests and as a reference. */
export class InMemoryAccounts implements Accounts {
  private password: string | undefined;

  async hasAccount(): Promise<boolean> {
    return this.password !== undefined;
  }

  async createAccount(email: string, password: string): Promise<void> {
    const problem = checkEmail(email) ?? checkPassword(password);
    if (problem) throw new AccountError(problem);
    if (await this.hasAccount()) {
      throw new AccountError("An account already exists on this device.");
    }
    this.password = password;
  }

  async unlock(password: string): Promise<void> {
    if (!(await this.hasAccount())) {
      throw new AccountError("There is no account on this device.");
    }
    if (password !== this.password) throw new AccountError("Wrong password");
  }
}
