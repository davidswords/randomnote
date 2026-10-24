// A real Chromium, driven through Playwright. Each test opens the app in its
// own browser context, so its browser storage starts empty.
import { type Browser, type Page, chromium } from "playwright-core";
import { APP_URL, SERVER_URL } from "./setup";

export const PASSWORD = "correct horse battery staple";

export async function launch(): Promise<Browser> {
  // CHROMIUM_PATH points at an installed Chromium; otherwise Playwright's own is used.
  return chromium.launch({ executablePath: process.env.CHROMIUM_PATH, headless: true });
}

export async function openApp(browser: Browser): Promise<Page> {
  const page = await (await browser.newContext()).newPage();
  await page.goto(APP_URL);
  return page;
}

export async function waitForTaskList(page: Page): Promise<void> {
  await page.waitForSelector('input[aria-label="New task"]');
}

export async function addTask(page: Page, title: string): Promise<void> {
  await page.fill('input[aria-label="New task"]', title);
  await page.keyboard.press("Enter");
  await page.waitForSelector(`[aria-label="Tasks"] li:text-is("${title}")`);
}

export async function shownTitles(page: Page): Promise<string[]> {
  return page.locator('[aria-label="Tasks"] li').allTextContents();
}

export async function signUp(page: Page, email: string, password = PASSWORD): Promise<void> {
  await page.click('nav button:text-is("Account")');
  await page.click('[role="menu"] button:text-is("Create free account")');
  const dialog = page.locator('[role="dialog"]');
  await dialog.locator('input[aria-label="Email"]').fill(email);
  await dialog.locator('input[aria-label="Password"]').fill(password);
  await dialog.locator('input[aria-label="Confirm password"]').fill(password);
  await dialog.locator('input[aria-label="Email"]').press("Enter");
}

export async function unlock(page: Page, password: string): Promise<void> {
  await page.fill('input[type="password"]', password);
  await page.keyboard.press("Enter");
}

/** Whether the server has an account for `email`: it gives a made-up seed otherwise. */
export async function serverSeed(email: string): Promise<string> {
  const response = await fetch(`${SERVER_URL}/v2/login-params`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ email, code_challenge: "test" }),
  });
  return ((await response.json()) as { pw_nonce: string }).pw_nonce;
}

export async function registerElsewhere(email: string): Promise<void> {
  await fetch(`${SERVER_URL}/v1/users`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      email,
      password: "a".repeat(64),
      pw_nonce: "b".repeat(64),
      version: "004",
    }),
  });
}
