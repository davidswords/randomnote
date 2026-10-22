import { createTask } from "@randomnote/core";

const app = document.querySelector<HTMLElement>("#app");

if (app) {
  app.textContent = createTask("Buy milk").title;
}
