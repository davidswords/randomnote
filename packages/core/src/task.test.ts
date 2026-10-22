import { describe, expect, it } from "vitest";
import { createTask, InvalidTaskTitleError } from "./task";

describe("createTask", () => {
  it("creates a task with the given title", () => {
    const task = createTask("Buy milk");

    expect(task.title).toBe("Buy milk");
  });

  it("creates a task with a different title", () => {
    const task = createTask("Call the dentist");

    expect(task.title).toBe("Call the dentist");
  });

  it("trims whitespace around the title", () => {
    const task = createTask("  Buy milk \n");

    expect(task.title).toBe("Buy milk");
  });

  it("rejects an empty title", () => {
    expect(() => createTask("")).toThrow(InvalidTaskTitleError);
  });

  it.each([" ", "   ", "\t", "\n", " \t\n "])(
    "rejects a whitespace-only title %j",
    (title) => {
      expect(() => createTask(title)).toThrow(InvalidTaskTitleError);
    },
  );

  it("keeps whitespace inside the title as typed", () => {
    const task = createTask("Buy  milk\tand\neggs");

    expect(task.title).toBe("Buy  milk\tand\neggs");
  });

  it("accepts a very long title", () => {
    const title = "a".repeat(100_000);

    expect(createTask(title).title).toBe(title);
  });

  it("accepts any Unicode text", () => {
    const task = createTask("Café ☕ 日本語 🛒");

    expect(task.title).toBe("Café ☕ 日本語 🛒");
  });
});
