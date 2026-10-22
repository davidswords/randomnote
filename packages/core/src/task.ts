import { newId } from "./id";

export type Task = {
  id: string;
  title: string;
};

export type IdGenerator = () => string;

export class InvalidTaskTitleError extends Error {
  constructor() {
    super("A task title cannot be empty.");
    this.name = "InvalidTaskTitleError";
  }
}

export function createTask(
  title: string,
  generateId: IdGenerator = newId,
): Task {
  const trimmed = title.trim();
  if (trimmed === "") {
    throw new InvalidTaskTitleError();
  }
  return { id: generateId(), title: trimmed };
}
