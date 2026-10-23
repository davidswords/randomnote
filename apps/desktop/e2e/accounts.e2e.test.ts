import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { AppDriver } from "./app";
import {
  EMAIL,
  PASSWORD,
  addTask,
  enterPassword,
  shownMessage,
  shownTitles,
  signUp,
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
  it("asks a new user to create an account, then shows their empty task list", async () => {
    const app = await driver.launch();

    await signUp(app);

    expect(await shownTitles(app)).toEqual([]);
  });

  it("refuses a password shorter than 12 characters at sign-up", async () => {
    const app = await driver.launch();
    await app.$('input[aria-label="Email"]').setValue(EMAIL);
    await app.$('input[aria-label="Password"]').setValue("short");
    await app.$('input[aria-label="Confirm password"]').setValue("short");

    await app.$("button=Create account").click();

    expect(await shownMessage(app)).toBe("The password must be at least 12 characters.");
  });

  it("refuses an invalid email at sign-up", async () => {
    const app = await driver.launch();
    await app.$('input[aria-label="Email"]').setValue("david");
    await app.$('input[aria-label="Password"]').setValue(PASSWORD);
    await app.$('input[aria-label="Confirm password"]').setValue(PASSWORD);

    await app.$("button=Create account").click();

    expect(await shownMessage(app)).toBe("Enter a valid email address.");
  });

  it("stays locked with the wrong password, and opens with the right one", async () => {
    const firstRun = await driver.launch();
    await signUp(firstRun);
    await addTask(firstRun, "Buy milk");
    await driver.quit(firstRun);
    const secondRun = await driver.launch();

    await enterPassword(secondRun, "incorrect horse battery");
    await secondRun.waitUntil(async () => (await shownMessage(secondRun)) !== "");

    expect(await shownMessage(secondRun)).toBe("Wrong password");
    expect(await secondRun.$('[aria-label="Tasks"]').isExisting()).toBe(false);

    await enterPassword(secondRun, PASSWORD);
    await secondRun.$('[aria-label="Tasks"]').waitForExist();

    expect(await shownTitles(secondRun)).toEqual(["Buy milk"]);
  });

  it("writes no task title or password to disk in plain text", async () => {
    const app = await driver.launch();
    await signUp(app);
    await addTask(app, "Buy oat milk for the office");
    await driver.quit(app);

    const files = driver.dataFiles();

    expect(files.length).toBeGreaterThan(0);
    expect(files.filter((file) => contains(file, "Buy oat milk for the office"))).toEqual([]);
    expect(files.filter((file) => contains(file, PASSWORD))).toEqual([]);
  });
});
