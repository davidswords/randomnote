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
});
