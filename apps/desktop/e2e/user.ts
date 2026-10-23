import type { RunningApp } from "./app";

export const EMAIL = "david@example.com";
export const PASSWORD = "correct horse battery staple";

/** Fills in sign-up the way a user would, typing the password twice. */
export async function signUp(app: RunningApp, password = PASSWORD): Promise<void> {
  await app.$('input[aria-label="Email"]').setValue(EMAIL);
  await app.$('input[aria-label="Password"]').setValue(password);
  await app.$('input[aria-label="Confirm password"]').setValue(password);
  await app.$("button=Create account").click();
  await app.$('[aria-label="Tasks"]').waitForExist();
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

export async function shownMessage(app: RunningApp): Promise<string> {
  return app.$('[role="alert"]').getText();
}
