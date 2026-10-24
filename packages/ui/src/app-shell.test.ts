// @vitest-environment happy-dom
import { InMemoryAccounts, InMemoryTaskRepository } from "@randomtask/core";
import { beforeEach, describe, expect, it } from "vitest";
import { showApp } from "./app-shell";

const EMAIL = "david@example.com";
const PASSWORD = "correct horse battery staple";
const NOTICE = "Your tasks aren't encrypted. Create a free account to secure them.";

let root: HTMLElement;
let accounts: InMemoryAccounts;
let repository: InMemoryTaskRepository;

beforeEach(() => {
  document.body.innerHTML = "";
  root = document.createElement("div");
  document.body.append(root);
  accounts = new InMemoryAccounts();
  repository = new InMemoryTaskRepository();
});

const open = () => showApp(root, { accounts, repository });
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
const byLabel = (label: string) => root.querySelector<HTMLElement>(`[aria-label="${label}"]`);
const button = (name: string) =>
  [...root.querySelectorAll("button")].find((each) => each.textContent === name);
const notice = () => root.querySelector('[role="status"]');
const menu = () => root.querySelector('[role="menu"]');
const menuItem = (name: string) =>
  [...(menu()?.querySelectorAll("button") ?? [])].find((each) => each.textContent === name);
const dialog = () => root.querySelector('[role="dialog"]');
const shownTitles = () =>
  [...root.querySelectorAll('[aria-label="Tasks"] li')].map((item) => item.textContent);

async function click(element: HTMLElement | undefined | null) {
  element!.click();
  await settle();
}

async function signUp(email: string, password: string) {
  const field = (label: string) =>
    dialog()!.querySelector<HTMLInputElement>(`input[aria-label="${label}"]`)!;
  field("Email").value = email;
  field("Password").value = password;
  field("Confirm password").value = password;
  field("Email").form!.requestSubmit();
  await settle();
}

describe("app without an account", () => {
  it("opens straight into the task list", async () => {
    await repository.save({ id: "task-1", title: "Buy milk" });

    await open();

    expect(shownTitles()).toEqual(["Buy milk"]);
    expect(byLabel("New task")).not.toBeNull();
  });

  it("says the tasks aren't encrypted", async () => {
    await open();

    expect(notice()!.textContent).toContain(NOTICE);
  });

  it("hides the notice when it is dismissed", async () => {
    await open();

    await click(byLabel("Dismiss"));

    expect(notice()).toBeNull();
  });

  it("offers to create a free account from the account menu", async () => {
    await open();

    await click(button("Account"));

    expect(menuItem("Create free account")).not.toBeUndefined();
  });

  it("closes the account menu when the account button is pressed again", async () => {
    await open();
    await click(button("Account"));

    await click(button("Account"));

    expect(menu()).toBeNull();
  });

  it("opens sign-up from the notice", async () => {
    await open();

    await click(notice()!.querySelector("button"));

    expect(dialog()!.getAttribute("aria-label")).toBe("Create free account");
  });

  it("creates the account, keeps the tasks, and stops warning", async () => {
    await repository.save({ id: "task-1", title: "Buy milk" });
    await open();
    await click(button("Account"));
    await click(menuItem("Create free account"));

    await signUp(EMAIL, PASSWORD);

    expect(await accounts.hasAccount()).toBe(true);
    expect(dialog()).toBeNull();
    expect(notice()).toBeNull();
    expect(shownTitles()).toEqual(["Buy milk"]);
  });

  it("says in the account menu that tasks are encrypted once the account exists", async () => {
    await open();
    await click(button("Account"));
    await click(menuItem("Create free account"));
    await signUp(EMAIL, PASSWORD);

    await click(button("Account"));

    expect(menuItem("Create free account")).toBeUndefined();
    expect(menu()!.textContent).toContain("Your tasks are encrypted with your account.");
  });

  it("keeps the dialog open and explains a problem with sign-up", async () => {
    await open();
    await click(button("Account"));
    await click(menuItem("Create free account"));

    await signUp("david", PASSWORD);

    expect(dialog()!.querySelector('[role="alert"]')!.textContent).toBe(
      "Enter a valid email address.",
    );
    expect(await accounts.hasAccount()).toBe(false);
  });

  it("closes sign-up without creating an account when it is cancelled", async () => {
    await open();
    await click(button("Account"));
    await click(menuItem("Create free account"));

    await click(button("Cancel"));

    expect(dialog()).toBeNull();
    expect(await accounts.hasAccount()).toBe(false);
  });
});

describe("app with an account", () => {
  beforeEach(async () => {
    await accounts.createAccount(EMAIL, PASSWORD);
  });

  it("doesn't warn that tasks aren't encrypted", async () => {
    await open();

    expect(notice()).toBeNull();
  });

  it("says in the account menu that tasks are encrypted", async () => {
    await open();

    await click(button("Account"));

    expect(menuItem("Create free account")).toBeUndefined();
    expect(menu()!.textContent).toContain(
      "Your tasks are encrypted with your account.",
    );
  });
});
