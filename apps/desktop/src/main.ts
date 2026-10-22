import { invoke } from "@tauri-apps/api/core";
import { TauriTaskRepository } from "./storage/tauri-task-repository";
import { showTaskList } from "./task-list-view";

const app = document.querySelector<HTMLElement>("#app");

if (app) {
  void showTaskList(app, new TauriTaskRepository(invoke));
}
