import type { Browser } from "playwright-core";
import { afterAll, beforeAll, describe, expect, it } from "vitest";
import {
  PASSWORD,
  addTask,
  launch,
  openApp,
  registerElsewhere,
  serverSeed,
  shownTitles,
  signUp,
  unlock,
  waitForTaskList,
} from "./browser";

let browser: Browser;

beforeAll(async () => {
  browser = await launch();
});

afterAll(async () => {
  await browser.close();
});

describe("the web app", () => {
  it("opens straight into the task list and says the tasks aren't encrypted", async () => {
    const page = await openApp(browser);

    await waitForTaskList(page);

    expect(await page.textContent('[role="status"]')).toContain(
      "Your tasks aren't encrypted.",
    );
  });

  it("keeps tasks in the browser across a reload", async () => {
    const page = await openApp(browser);
    await waitForTaskList(page);
    await addTask(page, "Buy milk");

    await page.reload();
    await waitForTaskList(page);

    expect(await shownTitles(page)).toEqual(["Buy milk"]);
  });

  it("registers a new account with the server, then asks for the password next time", async () => {
    const page = await openApp(browser);
    await waitForTaskList(page);
    await addTask(page, "Buy milk");
    const madeUpSeed = await serverSeed("signup@example.com");

    await signUp(page, "signup@example.com");
    await page.waitForSelector('[role="dialog"]', { state: "detached" });

    expect(await serverSeed("signup@example.com")).not.toBe(madeUpSeed);
    await page.reload();
    await unlock(page, PASSWORD);
    await waitForTaskList(page);
    expect(await shownTitles(page)).toEqual(["Buy milk"]);
  });

  it("shows the server's message when the email is already registered", async () => {
    await registerElsewhere("taken@example.com");
    const page = await openApp(browser);
    await waitForTaskList(page);

    await signUp(page, "taken@example.com");

    await expect
      .poll(() => page.locator('[role="dialog"] [role="alert"]').textContent())
      .toBe("This email is already registered.");
  });

  it("refuses the wrong password", async () => {
    const page = await openApp(browser);
    await waitForTaskList(page);
    await signUp(page, "unlock@example.com");
    await page.waitForSelector('[role="dialog"]', { state: "detached" });
    await page.reload();

    await unlock(page, "the wrong password");

    await expect.poll(() => page.locator('[role="alert"]').textContent()).toBe("Wrong password");
  });
});
