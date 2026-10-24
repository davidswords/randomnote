// Starts the web app. Only the end-to-end tests reach this file.
import {
  CommandAccounts,
  CommandTaskRepository,
  showAbout,
  showAccountScreen,
  showApp,
} from "@randomtask/ui";
import "@randomtask/ui/style.css";
import { workerInvoke } from "./worker-invoke";

const worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
const invoke = workerInvoke(worker);
const services = {
  accounts: new CommandAccounts(invoke),
  repository: new CommandTaskRepository(invoke),
};

const app = document.querySelector<HTMLElement>("#app");
const footer = document.querySelector<HTMLElement>("#about");

if (footer) {
  void showAbout(footer, {
    getName: async () => "randomtask",
    getVersion: async () => __APP_VERSION__,
  });
}

if (app) {
  // With an account, the password unlocks the app first; without one, it opens straight away.
  void services.accounts.hasAccount().then((hasAccount) =>
    hasAccount
      ? showAccountScreen(app, services.accounts, () => showApp(app, services))
      : showApp(app, services),
  );
}
