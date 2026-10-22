import { describe, expect, it } from "vitest";
import { createTask } from "./task";

const UUID_V7 =
  /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;

describe("createTask with the default ID generator", () => {
  it("gives the task a UUID v7", () => {
    const task = createTask("Buy milk");

    expect(task.id).toMatch(UUID_V7);
  });

  it("gives each task a different ID", () => {
    const first = createTask("Buy milk");
    const second = createTask("Buy milk");

    expect(first.id).not.toBe(second.id);
  });
});
