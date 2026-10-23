import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { AppDriver } from "./app";
import {
  EMAIL,
  PASSWORD,
  addTask,
  enterPassword,
  openSignUp,
  shownMessage,
  shownTitles,
  signUp,
  submitSignUp,
  waitForTaskList,
} from "./user";

let driver: AppDriver;

beforeEach(async () => {
  driver = await AppDriver.start();
});

afterEach(async () => {
  await driver.stop();
});

const contains = (file: Buffer, text: string) => file.includes(Buffer.from(text, "utf8"));

describe("accounts", () => {
  it("opens straight into an empty task list, saying the tasks aren't encrypted", async () => {
    const app = await driver.launch();

    await waitForTaskList(app);

    expect(await shownTitles(app)).toEqual([]);
    expect(await app.$('[role="status"]').getText()).toContain(
      "Your tasks aren't encrypted. Create a free account to secure them.",
    );
  });

  it("keeps the tasks when an account is created, and asks for the password next time", async () => {
    const firstRun = await driver.launch();
    await waitForTaskList(firstRun);
    await addTask(firstRun, "Buy milk");

    await signUp(firstRun);

    expect(await shownTitles(firstRun)).toEqual(["Buy milk"]);
    expect(await firstRun.$('[role="status"]').isExisting()).toBe(false);
    await driver.quit(firstRun);
    const secondRun = await driver.launch();
    await enterPassword(secondRun, PASSWORD);
    await waitForTaskList(secondRun);
    expect(await shownTitles(secondRun)).toEqual(["Buy milk"]);
  });

  it("refuses a password shorter than 12 characters at sign-up", async () => {
    const app = await driver.launch();
    await openSignUp(app);

    await submitSignUp(app, EMAIL, "short");

    expect(await shownMessage(app, '[role="dialog"]')).toBe(
      "The password must be at least 12 characters.",
    );
  });

  it("refuses an invalid email at sign-up", async () => {
    const app = await driver.launch();
    await openSignUp(app);

    await submitSignUp(app, "david", PASSWORD);

    expect(await shownMessage(app, '[role="dialog"]')).toBe("Enter a valid email address.");
  });

  it("stays locked with the wrong password, and opens with the right one", async () => {
    const firstRun = await driver.launch();
    await signUp(firstRun);
    await addTask(firstRun, "Buy milk");
    await driver.quit(firstRun);
    const secondRun = await driver.launch();

    await enterPassword(secondRun, "incorrect horse battery");

    expect(await shownMessage(secondRun)).toBe("Wrong password");
    expect(await secondRun.$('[aria-label="Tasks"]').isExisting()).toBe(false);

    await enterPassword(secondRun, PASSWORD);
    await waitForTaskList(secondRun);

    expect(await shownTitles(secondRun)).toEqual(["Buy milk"]);
  });

  it("leaves no task title or password on disk in plain text once secured", async () => {
    const app = await driver.launch();
    await waitForTaskList(app);
    await addTask(app, "Buy oat milk for the office");
    await signUp(app);
    await addTask(app, "Book the dentist for Tuesday");
    await driver.quit(app);

    const files = driver.dataFiles();

    expect(files.length).toBeGreaterThan(0);
    for (const secret of ["Buy oat milk for the office", "Book the dentist for Tuesday", PASSWORD]) {
      expect(files.filter((file) => contains(file, secret))).toEqual([]);
    }
  });
});
