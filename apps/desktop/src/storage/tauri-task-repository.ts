import type { Task, TaskRepository } from "@randomtask/core";

/** The shape of Tauri's invoke, so tests can pass a fake. */
export type Invoke = <T>(
  command: string,
  args?: Record<string, unknown>,
) => Promise<T>;

/** Stores tasks through the Rust task commands. */
export class TauriTaskRepository implements TaskRepository {
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
