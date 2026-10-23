import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { afterEach, beforeEach, describe, expect, it } from "vitest";
import { AppDriver } from "./app";

let driver: AppDriver;

beforeEach(async () => {
  driver = await AppDriver.start();
});

afterEach(async () => {
  await driver.stop();
});

function writeFile(path: string, contents: string): void {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, contents);
}

describe("upgrading from an unencrypted version", () => {
  it("deletes the plain-text tasks left by randomnote 0.2.0 and randomtask 0.2.1", async () => {
    const randomnoteTasks = driver.dataPath("engineering.swords.randomnote", "tasks.db");
    const randomtaskTasks = driver.dataPath("engineering.swords.randomtask", "tasks.db");
    writeFile(randomnoteTasks, "Buy milk");
    writeFile(randomtaskTasks, "Buy milk");

    const app = await driver.launch();
    await app.$('input[aria-label="New task"]').waitForExist();

    expect(existsSync(randomnoteTasks)).toBe(false);
    expect(existsSync(randomtaskTasks)).toBe(false);
  });
});
