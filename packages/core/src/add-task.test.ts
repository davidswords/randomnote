import { describe, expect, it } from "vitest";
import { addTask } from "./add-task";
import { InMemoryTaskRepository } from "./in-memory-task-repository";
import { InvalidTaskTitleError } from "./task";

const fixedId = () => "task-1";

describe("addTask", () => {
  it("saves a new task with the given title", async () => {
    const repository = new InMemoryTaskRepository();

    await addTask("Buy milk", repository, fixedId);

    expect(await repository.list()).toEqual([{ id: "task-1", title: "Buy milk" }]);
  });

  it("returns the task it saved", async () => {
    const repository = new InMemoryTaskRepository();

    const task = await addTask("Buy milk", repository, fixedId);

    expect(task).toEqual({ id: "task-1", title: "Buy milk" });
  });

  it("saves nothing when the title is empty", async () => {
    const repository = new InMemoryTaskRepository();

    await expect(addTask(" ", repository, fixedId)).rejects.toThrow(
      InvalidTaskTitleError,
    );
    expect(await repository.list()).toEqual([]);
  });
});
