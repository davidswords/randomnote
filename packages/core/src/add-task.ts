import { newId } from "./id";
import { createTask, type IdGenerator, type Task } from "./task";
import type { TaskRepository } from "./task-repository";

/** Creates a task from a title and saves it. */
export async function addTask(
  title: string,
  repository: TaskRepository,
  generateId: IdGenerator = newId,
): Promise<Task> {
  const task = createTask(title, generateId);
  await repository.save(task);
  return task;
}
