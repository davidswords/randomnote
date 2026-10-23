import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { AppDriver } from "./app";

const { version } = JSON.parse(
  readFileSync(resolve(import.meta.dirname, "../package.json"), "utf8"),
) as { version: string };

let driver: AppDriver;

beforeEach(async () => {
  driver = await AppDriver.start();
});

afterEach(async () => {
  await driver.stop();
});

describe("about", () => {
  it("shows the app's name and released version at the bottom of the window", async () => {
    const app = await driver.launch();
    const footer = app.$("footer");

    await app.waitUntil(async () => (await footer.getText()) !== "");

    expect(await footer.getText()).toBe(`randomtask ${version}`);
  });
});
