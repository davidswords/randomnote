import { describe, expect, it } from "vitest";
import { createTask, InvalidTaskTitleError } from "./task";

const anyId = () => "task-1";

describe("createTask", () => {
  it("gives the task an ID from the ID generator", () => {
    const task = createTask("Buy milk", () => "task-1");

    expect(task.id).toBe("task-1");
  });

  it("creates a task with the given title", () => {
    const task = createTask("Buy milk", anyId);

    expect(task.title).toBe("Buy milk");
  });

  it("creates a task with a different title", () => {
    const task = createTask("Call the dentist", anyId);

    expect(task.title).toBe("Call the dentist");
  });

  it("trims whitespace around the title", () => {
    const task = createTask("  Buy milk \n", anyId);

    expect(task.title).toBe("Buy milk");
  });

  it("rejects an empty title", () => {
    expect(() => createTask("", anyId)).toThrow(InvalidTaskTitleError);
  });

  it.each([" ", "   ", "\t", "\n", " \t\n "])(
    "rejects a whitespace-only title %j",
    (title) => {
      expect(() => createTask(title, anyId)).toThrow(InvalidTaskTitleError);
    },
  );

  it("keeps whitespace inside the title as typed", () => {
    const task = createTask("Buy  milk\tand\neggs", anyId);

    expect(task.title).toBe("Buy  milk\tand\neggs");
  });

  it("accepts a very long title", () => {
    const title = "a".repeat(100_000);

    expect(createTask(title, anyId).title).toBe(title);
  });

  it("accepts any Unicode text", () => {
    const task = createTask("Café ☕ 日本語 🛒", anyId);

    expect(task.title).toBe("Café ☕ 日本語 🛒");
  });
});
