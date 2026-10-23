import * as tauriApp from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { showAbout } from "./about";
import { showAccountScreen } from "./account-view";
import { TauriAccounts } from "./storage/tauri-accounts";
import { TauriTaskRepository } from "./storage/tauri-task-repository";
import { showTaskList } from "./task-list-view";
import "./style.css";

const app = document.querySelector<HTMLElement>("#app");
const footer = document.querySelector<HTMLElement>("#about");

if (footer) {
  void showAbout(footer, tauriApp);
}

if (app) {
  void showAccountScreen(app, new TauriAccounts(invoke), () =>
    showTaskList(app, new TauriTaskRepository(invoke)),
  );
}
