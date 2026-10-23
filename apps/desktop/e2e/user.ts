import type { RunningApp } from "./app";

export const EMAIL = "david@example.com";
export const PASSWORD = "correct horse battery staple";

/** Waits until the task list is on screen. */
export async function waitForTaskList(app: RunningApp): Promise<void> {
  await app.$('input[aria-label="New task"]').waitForExist();
}

/** Opens sign-up from the account menu, as a user would. */
export async function openSignUp(app: RunningApp): Promise<void> {
  await waitForTaskList(app);
  await app.$("button=Account").click();
  await app.$('[role="menu"]').$("button=Create free account").click();
  await app.$('[role="dialog"]').waitForExist();
}

/** Fills in the sign-up dialog, typing the password twice, and submits it. */
export async function submitSignUp(
  app: RunningApp,
  email: string,
  password: string,
): Promise<void> {
  const dialog = app.$('[role="dialog"]');
  await dialog.$('input[aria-label="Email"]').setValue(email);
  await dialog.$('input[aria-label="Password"]').setValue(password);
  await dialog.$('input[aria-label="Confirm password"]').setValue(password);
  await dialog.$("button=Create account").click();
}

/** Creates a free account from the account menu and waits for it to finish. */
export async function signUp(app: RunningApp): Promise<void> {
  await openSignUp(app);
  await submitSignUp(app, EMAIL, PASSWORD);
  await app.$('[role="dialog"]').waitForExist({ reverse: true });
}

/** Types the password into the unlock screen and submits it. */
export async function enterPassword(app: RunningApp, password: string): Promise<void> {
  const field = app.$('input[aria-label="Password"]');
  await field.waitForExist();
  await field.setValue(password);
  await app.$("button=Unlock").click();
}

export async function addTask(app: RunningApp, title: string): Promise<void> {
  const before = (await shownTitles(app)).length;
  await app.$('input[aria-label="New task"]').setValue(title);
  await app.keys("Enter");
  await app.waitUntil(async () => (await shownTitles(app)).length === before + 1);
}

export async function shownTitles(app: RunningApp): Promise<string[]> {
  return app.$$('[aria-label="Tasks"] li').map((item) => item.getText());
}

/** The message shown in `scope`, such as the sign-up dialog, once there is one. */
export async function shownMessage(app: RunningApp, scope = "body"): Promise<string> {
  const message = app.$(scope).$('[role="alert"]');
  await app.waitUntil(async () => (await message.getText()) !== "");
  return message.getText();
}
