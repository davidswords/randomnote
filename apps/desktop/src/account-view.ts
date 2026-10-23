import { AccountError, type Accounts, checkNewAccount } from "@randomtask/core";

const WARNING =
  "Your password encrypts your tasks. If you forget it, they can't be recovered.";
const UNEXPECTED_FAILURE = "Something went wrong. Please try again.";

/**
 * Shows sign-up when the device has no account, otherwise asks for the
 * password. Calls `onUnlocked` once the account is created or unlocked.
 */
export async function showAccountScreen(
  root: HTMLElement,
  accounts: Accounts,
  onUnlocked: () => unknown,
): Promise<void> {
  if (await accounts.hasAccount()) {
    showUnlock(root, accounts, onUnlocked);
  } else {
    showSignUp(root, accounts, onUnlocked);
  }
}

function showSignUp(root: HTMLElement, accounts: Accounts, onUnlocked: () => unknown) {
  const email = input("Email", "email");
  const password = input("Password", "password");
  const confirmation = input("Confirm password", "password");
  const warning = document.createElement("p");
  warning.textContent = WARNING;
  const { form, message } = accountForm(root, "Create account", [
    email,
    password,
    confirmation,
    warning,
  ]);

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    const values = {
      email: email.value,
      password: password.value,
      confirmation: confirmation.value,
    };
    const problem = checkNewAccount(values);
    if (problem) {
      message.textContent = problem;
      return;
    }
    await attempt(message, async () => {
      await accounts.createAccount(values.email, values.password);
      onUnlocked();
    });
  });
}

function showUnlock(root: HTMLElement, accounts: Accounts, onUnlocked: () => unknown) {
  const password = input("Password", "password");
  const { form, message } = accountForm(root, "Unlock", [password]);

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    await attempt(message, async () => {
      try {
        await accounts.unlock(password.value);
      } finally {
        password.value = "";
      }
      onUnlocked();
    });
  });
}

/** Runs `action`, showing why it failed if it does. */
async function attempt(message: HTMLElement, action: () => Promise<void>) {
  try {
    await action();
  } catch (error) {
    message.textContent =
      error instanceof AccountError ? error.message : UNEXPECTED_FAILURE;
  }
}

function accountForm(root: HTMLElement, action: string, children: HTMLElement[]) {
  const form = document.createElement("form");
  // The app checks the fields itself, so it can explain each problem.
  form.noValidate = true;
  const message = document.createElement("p");
  message.setAttribute("role", "alert");
  const button = document.createElement("button");
  button.type = "submit";
  button.textContent = action;
  form.append(...children, message, button);
  root.replaceChildren(form);
  return { form, message };
}

function input(label: string, type: "email" | "password"): HTMLInputElement {
  const field = document.createElement("input");
  field.type = type;
  field.setAttribute("aria-label", label);
  field.placeholder = label;
  return field;
}
