import * as tauriApp from "@tauri-apps/api/app";
import { invoke } from "@tauri-apps/api/core";
import { showAbout, showAccountScreen, showApp } from "@randomtask/ui";
import "@randomtask/ui/style.css";
import { TauriAccounts } from "./storage/tauri-accounts";
import { TauriTaskRepository } from "./storage/tauri-task-repository";

const app = document.querySelector<HTMLElement>("#app");
const footer = document.querySelector<HTMLElement>("#about");
const services = {
  accounts: new TauriAccounts(invoke),
  repository: new TauriTaskRepository(invoke),
};

if (footer) {
  void showAbout(footer, tauriApp);
}

if (app) {
  // With an account, the password unlocks the app first; without one, it opens straight away.
  void services.accounts.hasAccount().then((hasAccount) =>
    hasAccount
      ? showAccountScreen(app, services.accounts, () => showApp(app, services))
      : showApp(app, services),
  );
}
