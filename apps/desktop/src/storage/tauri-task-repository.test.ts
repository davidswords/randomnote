import { taskRepositoryContract } from "@randomnote/core/testing";
import { describe, expect, it } from "vitest";
import { fakeTaskCommands } from "./fake-task-commands";
import { TauriTaskRepository } from "./tauri-task-repository";

taskRepositoryContract(
  () => new TauriTaskRepository(fakeTaskCommands().invoke),
);

describe("TauriTaskRepository messages", () => {
  it("saves with save_task, passing the task", async () => {
    const commands = fakeTaskCommands();
    const repository = new TauriTaskRepository(commands.invoke);

    await repository.save({ id: "task-1", title: "Buy milk" });

    expect(commands.calls).toEqual([
      { command: "save_task", args: { task: { id: "task-1", title: "Buy milk" } } },
    ]);
  });

  it("finds with find_task, passing the ID, and lists with list_tasks", async () => {
    const commands = fakeTaskCommands();
    const repository = new TauriTaskRepository(commands.invoke);

    await repository.findById("task-1");
    await repository.list();

    expect(commands.calls).toEqual([
      { command: "find_task", args: { id: "task-1" } },
      { command: "list_tasks", args: undefined },
    ]);
  });
});

describe("fake task commands", () => {
  it("rejects unknown commands, as Tauri does", async () => {
    const { invoke } = fakeTaskCommands();

    await expect(invoke("delete_everything")).rejects.toThrow(
      "Unknown command: delete_everything",
    );
  });
});
