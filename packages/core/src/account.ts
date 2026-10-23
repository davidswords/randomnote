const MINIMUM_PASSWORD_CHARACTERS = 12;

/** The account on this device, whose password unlocks its tasks. */
export interface Accounts {
  hasAccount(): Promise<boolean>;
  /** Creates the account and unlocks it. */
  createAccount(username: string, password: string): Promise<void>;
  unlock(password: string): Promise<void>;
}

/** A refusal whose message is written for the user, such as "Wrong password". */
export class AccountError extends Error {
  override name = "AccountError";
}

export type NewAccount = {
  username: string;
  password: string;
  confirmation: string;
};

/** The first problem with a password, or undefined if there is none. */
export function checkPassword(password: string): string | undefined {
  // Spreading a string splits it into code points, as Rust's chars() does.
  return [...password].length < MINIMUM_PASSWORD_CHARACTERS
    ? `The password must be at least ${MINIMUM_PASSWORD_CHARACTERS} characters.`
    : undefined;
}

/** The first problem with a sign-up form, or undefined if there is none. */
export function checkNewAccount({
  username,
  password,
  confirmation,
}: NewAccount): string | undefined {
  if (username.trim() === "") return "Choose a username.";
  if (password !== confirmation) return "The passwords don't match.";
  return checkPassword(password);
}
