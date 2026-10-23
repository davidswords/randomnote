import { invoke } from "@tauri-apps/api/core";
import { showAccountScreen } from "./account-view";
import { TauriAccounts } from "./storage/tauri-accounts";
import { TauriTaskRepository } from "./storage/tauri-task-repository";
import { showTaskList } from "./task-list-view";

const app = document.querySelector<HTMLElement>("#app");

if (app) {
  void showAccountScreen(app, new TauriAccounts(invoke), () =>
    showTaskList(app, new TauriTaskRepository(invoke)),
  );
}
