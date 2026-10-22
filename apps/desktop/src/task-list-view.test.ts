// @vitest-environment happy-dom
import { InMemoryTaskRepository } from "@randomnote/core";
import { beforeEach, describe, expect, it } from "vitest";
import { showTaskList } from "./task-list-view";

let root: HTMLElement;
let repository: InMemoryTaskRepository;
let nextId: number;
const sequentialIds = () => `task-${++nextId}`;

beforeEach(() => {
  document.body.innerHTML = "";
  root = document.createElement("main");
  document.body.append(root);
  repository = new InMemoryTaskRepository();
  nextId = 0;
});

const open = () => showTaskList(root, repository, sequentialIds);
const titleField = () => root.querySelector<HTMLInputElement>('input[aria-label="New task"]')!;
const shownTitles = () =>
  [...root.querySelectorAll('[aria-label="Tasks"] li')].map((item) => item.textContent);
const message = () => root.querySelector('[role="alert"]')!.textContent;

async function add(title: string) {
  titleField().value = title;
  titleField().form!.requestSubmit();
  await new Promise((resolve) => setTimeout(resolve, 0));
}

describe("task list view", () => {
  it("shows the saved tasks, oldest first", async () => {
    await repository.save({ id: "task-2", title: "Call the dentist" });
    await repository.save({ id: "task-1", title: "Buy milk" });

    await open();

    expect(shownTitles()).toEqual(["Buy milk", "Call the dentist"]);
  });

  it("shows titles as plain text, not HTML", async () => {
    await repository.save({ id: "task-1", title: "<b>Buy</b> milk" });

    await open();

    expect(shownTitles()).toEqual(["<b>Buy</b> milk"]);
  });

  it("adds a task when a title is entered", async () => {
    await open();

    await add("Buy milk");

    expect(shownTitles()).toEqual(["Buy milk"]);
    expect(await repository.list()).toEqual([{ id: "task-1", title: "Buy milk" }]);
  });

  it("clears the title field after adding a task", async () => {
    await open();

    await add("Buy milk");

    expect(titleField().value).toBe("");
  });

  it("asks for a title and saves nothing when the title is empty", async () => {
    await open();

    await add("   ");

    expect(message()).toBe("A task needs a title.");
    expect(await repository.list()).toEqual([]);
  });

  it("clears the message once a task is added", async () => {
    await open();
    await add("");

    await add("Buy milk");

    expect(message()).toBe("");
  });
});
