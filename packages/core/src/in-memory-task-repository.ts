import type { Task } from "./task";
import type { TaskRepository } from "./task-repository";

/**
 * Keeps tasks in memory. Like real storage, it stores and returns copies,
 * so a task can only change through save.
 */
export class InMemoryTaskRepository implements TaskRepository {
  private readonly tasks = new Map<string, Task>();

  async save(task: Task): Promise<void> {
    this.tasks.set(task.id, copy(task));
  }

  async findById(id: string): Promise<Task | undefined> {
    const task = this.tasks.get(id);
    return task && copy(task);
  }

  async list(): Promise<Task[]> {
    return [...this.tasks.values()].map(copy);
  }
}

const copy = (task: Task): Task => ({ ...task });
