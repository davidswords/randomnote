const MINIMUM_PASSWORD_CHARACTERS = 12;

/** The account on this device, whose password unlocks its tasks. */
export interface Accounts {
  hasAccount(): Promise<boolean>;
  /** Creates the account and unlocks it. */
  createAccount(email: string, password: string): Promise<void>;
  unlock(password: string): Promise<void>;
}

/** A refusal whose message is written for the user, such as "Wrong password". */
export class AccountError extends Error {
  override name = "AccountError";
}

export type NewAccount = {
  email: string;
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

/**
 * The first problem with an email address, or undefined if there is none.
 * The same rule as the Rust vault's: one `@`, a name before it, and a domain
 * with a dot inside it, with no spaces. Surrounding spaces are ignored.
 */
export function checkEmail(email: string): string | undefined {
  const trimmed = email.trim();
  if (trimmed === "") return "Enter your email address.";
  const [name, ...domains] = trimmed.split("@");
  const domain = domains.join("@");
  const valid =
    name !== "" &&
    domains.length === 1 &&
    domain.includes(".") &&
    !domain.startsWith(".") &&
    !domain.endsWith(".") &&
    !/\s/.test(trimmed);
  return valid ? undefined : "Enter a valid email address.";
}

/** The first problem with a sign-up form, or undefined if there is none. */
export function checkNewAccount({
  email,
  password,
  confirmation,
}: NewAccount): string | undefined {
  const emailProblem = checkEmail(email);
  if (emailProblem) return emailProblem;
  if (password !== confirmation) return "The passwords don't match.";
  return checkPassword(password);
}
