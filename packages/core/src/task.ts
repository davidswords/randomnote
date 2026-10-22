export type Task = {
  title: string;
};

export function createTask(title: string): Task {
  return { title: title.trim() };
}
