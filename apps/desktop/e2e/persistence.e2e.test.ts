import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { AppDriver } from "./app";
import { addTask, shownTitles, waitForTaskList } from "./user";

let driver: AppDriver;

beforeEach(async () => {
  driver = await AppDriver.start();
});

afterEach(async () => {
  await driver.stop();
});

describe("persistence", () => {
  it("keeps a task after the app is closed and opened again, with no account", async () => {
    const firstRun = await driver.launch();
    await waitForTaskList(firstRun);
    await addTask(firstRun, "Buy milk");
    await driver.quit(firstRun);

    const secondRun = await driver.launch();
    await waitForTaskList(secondRun);

    expect(await shownTitles(secondRun)).toEqual(["Buy milk"]);
  });
});
