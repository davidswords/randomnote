// @vitest-environment happy-dom
import { AccountError, InMemoryAccounts } from "@randomtask/core";
import { beforeEach, describe, expect, it } from "vitest";
import { showAccountScreen } from "./account-view";

const EMAIL = "david@example.com";
const PASSWORD = "correct horse battery staple";

let root: HTMLElement;
let accounts: InMemoryAccounts;
let unlockedCount: number;

beforeEach(() => {
  document.body.innerHTML = "";
  root = document.createElement("main");
  document.body.append(root);
  accounts = new InMemoryAccounts();
  unlockedCount = 0;
});

const open = () =>
  showAccountScreen(root, accounts, () => {
    unlockedCount += 1;
  });
const field = (label: string) =>
  root.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`);
const button = () => root.querySelector("button");
const message = () => root.querySelector('[role="alert"]')!.textContent;

async function submit(values: Record<string, string>) {
  for (const [label, value] of Object.entries(values)) field(label)!.value = value;
  button()!.form!.requestSubmit();
  await new Promise((resolve) => setTimeout(resolve, 0));
}

const signUp = (email: string, password: string, confirmation: string) =>
  submit({ Email: email, Password: password, "Confirm password": confirmation });

describe("account screen without an account", () => {
  it("asks for an email and the password twice, hiding the passwords", async () => {
    await open();

    expect(field("Email")!.type).toBe("email");
    expect(field("Password")!.type).toBe("password");
    expect(field("Confirm password")!.type).toBe("password");
    expect(button()!.textContent).toBe("Create account");
  });

  it("warns that a forgotten password can't be recovered", async () => {
    await open();

    expect(root.textContent).toContain(
      "Your password encrypts your tasks. If you forget it, they can't be recovered.",
    );
  });

  it("creates the account and opens the app", async () => {
    await open();

    await signUp(EMAIL, PASSWORD, PASSWORD);

    expect(await accounts.hasAccount()).toBe(true);
    expect(unlockedCount).toBe(1);
  });

  it("leaves its own checks to the app, so it can explain an invalid email", async () => {
    await open();

    expect(field("Email")!.form!.noValidate).toBe(true);
  });

  it("explains a problem with the form and creates nothing", async () => {
    await open();

    await signUp(EMAIL, PASSWORD, "something else entirely");

    expect(message()).toBe("The passwords don't match.");
    expect(await accounts.hasAccount()).toBe(false);
    expect(unlockedCount).toBe(0);
  });

  it("shows why the account couldn't be created", async () => {
    accounts.createAccount = async () => {
      throw new AccountError("An account already exists on this device.");
    };
    await open();

    await signUp(EMAIL, PASSWORD, PASSWORD);

    expect(message()).toBe("An account already exists on this device.");
    expect(unlockedCount).toBe(0);
  });

  it("says something went wrong when creating fails unexpectedly", async () => {
    accounts.createAccount = async () => {
      throw new Error("disk full");
    };
    await open();

    await signUp(EMAIL, PASSWORD, PASSWORD);

    expect(message()).toBe("Something went wrong. Please try again.");
  });
});

describe("account screen with an account", () => {
  beforeEach(async () => {
    await accounts.createAccount(EMAIL, PASSWORD);
  });

  it("asks only for the password, hiding it", async () => {
    await open();

    expect(field("Email")).toBeNull();
    expect(field("Confirm password")).toBeNull();
    expect(field("Password")!.type).toBe("password");
    expect(button()!.textContent).toBe("Unlock");
  });

  it("opens the app with the right password", async () => {
    await open();

    await submit({ Password: PASSWORD });

    expect(unlockedCount).toBe(1);
  });

  it("says the password is wrong, clears it and stays locked", async () => {
    await open();

    await submit({ Password: "incorrect horse battery" });

    expect(message()).toBe("Wrong password");
    expect(field("Password")!.value).toBe("");
    expect(unlockedCount).toBe(0);
  });

  it("says something went wrong when unlocking fails unexpectedly", async () => {
    accounts.unlock = async () => {
      throw new Error("disk full");
    };
    await open();

    await submit({ Password: PASSWORD });

    expect(message()).toBe("Something went wrong. Please try again.");
  });
});
