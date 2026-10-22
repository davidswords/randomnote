import {
  addTask,
  type IdGenerator,
  InvalidTaskTitleError,
  type Task,
  type TaskRepository,
} from "@randomnote/core";

const EMPTY_TITLE_MESSAGE = "A task needs a title.";

/** Shows the saved tasks in `root`, with a field for adding new ones. */
export async function showTaskList(
  root: HTMLElement,
  repository: TaskRepository,
  generateId?: IdGenerator,
): Promise<void> {
  const form = document.createElement("form");
  const titleField = document.createElement("input");
  titleField.setAttribute("aria-label", "New task");
  titleField.placeholder = "New task";
  const message = document.createElement("p");
  message.setAttribute("role", "alert");
  const list = document.createElement("ul");
  list.setAttribute("aria-label", "Tasks");
  form.append(titleField, message);
  root.replaceChildren(form, list);

  const refresh = async () => renderTasks(list, await repository.list());

  form.addEventListener("submit", async (event) => {
    event.preventDefault();
    try {
      await addTask(titleField.value, repository, generateId);
      titleField.value = "";
      message.textContent = "";
      await refresh();
    } catch (error) {
      if (!(error instanceof InvalidTaskTitleError)) throw error;
      message.textContent = EMPTY_TITLE_MESSAGE;
    }
  });

  await refresh();
}

function renderTasks(list: HTMLUListElement, tasks: Task[]): void {
  list.replaceChildren(...oldestFirst(tasks).map(taskItem));
}

function oldestFirst(tasks: Task[]): Task[] {
  return [...tasks].sort((a, b) => (a.id < b.id ? -1 : a.id > b.id ? 1 : 0));
}

function taskItem(task: Task): HTMLLIElement {
  const item = document.createElement("li");
  item.textContent = task.title;
  return item;
}
