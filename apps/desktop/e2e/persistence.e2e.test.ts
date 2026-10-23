import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { AppDriver } from "./app";
import { PASSWORD, addTask, enterPassword, shownTitles, signUp } from "./user";

let driver: AppDriver;

beforeEach(async () => {
  driver = await AppDriver.start();
});

afterEach(async () => {
  await driver.stop();
});

describe("persistence", () => {
  it("keeps a task after the app is closed and opened again", async () => {
    const firstRun = await driver.launch();
    await signUp(firstRun);
    await addTask(firstRun, "Buy milk");
    await driver.quit(firstRun);

    const secondRun = await driver.launch();
    await enterPassword(secondRun, PASSWORD);
    await secondRun.$('[aria-label="Tasks"]').waitForExist();

    expect(await shownTitles(secondRun)).toEqual(["Buy milk"]);
  });
});
