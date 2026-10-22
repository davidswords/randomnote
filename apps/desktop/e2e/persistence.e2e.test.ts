import { afterAll, beforeAll, describe, expect, it } from "vitest";
import { AppDriver, type RunningApp } from "./app";

let driver: AppDriver;

beforeAll(async () => {
  driver = await AppDriver.start();
});

afterAll(() => {
  driver.stop();
});

async function addTask(app: RunningApp, title: string): Promise<void> {
  const field = app.$('input[aria-label="New task"]');
  await field.setValue(title);
  await app.keys("Enter");
}

async function shownTitles(app: RunningApp): Promise<string[]> {
  return app.$$('[aria-label="Tasks"] li').map((item) => item.getText());
}

describe("persistence", () => {
  it("keeps a task after the app is closed and opened again", async () => {
    const firstRun = await driver.launch();
    await addTask(firstRun, "Buy milk");
    await firstRun.waitUntil(async () => (await shownTitles(firstRun)).length === 1);
    await firstRun.deleteSession();

    const secondRun = await driver.launch();
    await secondRun.$('[aria-label="Tasks"]').waitForExist();

    expect(await shownTitles(secondRun)).toEqual(["Buy milk"]);
    await secondRun.deleteSession();
  });
});
