import type { Task } from "./task";

/** Stores tasks. Implementations: in memory for tests, real storage in the apps. */
export interface TaskRepository {
  save(task: Task): Promise<void>;
  findById(id: string): Promise<Task | undefined>;
  list(): Promise<Task[]>;
}
