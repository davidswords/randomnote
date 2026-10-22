export type Task = {
  title: string;
};

export class InvalidTaskTitleError extends Error {
  constructor() {
    super("A task title cannot be empty.");
    this.name = "InvalidTaskTitleError";
  }
}

export function createTask(title: string): Task {
  const trimmed = title.trim();
  if (trimmed === "") {
    throw new InvalidTaskTitleError();
  }
  return { title: trimmed };
}
