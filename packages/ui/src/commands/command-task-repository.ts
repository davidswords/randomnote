import type { Task, TaskRepository } from "@randomtask/core";

/** Runs one of the Rust app's commands by name: Tauri's invoke on the
 * desktop, a message to the worker in the browser, or a fake in tests. */
export type Invoke = <T>(
  command: string,
  args?: Record<string, unknown>,
) => Promise<T>;

/** Stores tasks through the Rust task commands. */
export class CommandTaskRepository implements TaskRepository {
  constructor(private readonly invoke: Invoke) {}

  async save(task: Task): Promise<void> {
    await this.invoke<null>("save_task", { task });
  }

  async findById(id: string): Promise<Task | undefined> {
    return (await this.invoke<Task | null>("find_task", { id })) ?? undefined;
  }

  list(): Promise<Task[]> {
    return this.invoke<Task[]>("list_tasks");
  }
}
