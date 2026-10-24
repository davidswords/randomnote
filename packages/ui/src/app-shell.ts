import type { Accounts, IdGenerator, TaskRepository } from "@randomtask/core";
import { showSignUp } from "./account-view";
import { showTaskList } from "./task-list-view";

const NOT_ENCRYPTED = "Your tasks aren't encrypted. Create a free account to secure them.";
const ENCRYPTED = "Your tasks are encrypted with your account.";
const CREATE_ACCOUNT = "Create free account";

export type AppServices = {
  accounts: Accounts;
  repository: TaskRepository;
  generateId?: IdGenerator;
};

/**
 * Shows the task list with the account menu. Without an account, it also says
 * the tasks aren't encrypted and offers to create a free account.
 */
export async function showApp(
  root: HTMLElement,
  { accounts, repository, generateId }: AppServices,
): Promise<void> {
  const tasks = document.createElement("main");
  const accountBar = document.createElement("nav");
  root.replaceChildren(tasks, accountBar);
  let secured = await accounts.hasAccount();
  let notice: HTMLElement | undefined;

  const openSignUp = () => {
    const dialog = document.createElement("div");
    dialog.setAttribute("role", "dialog");
    dialog.setAttribute("aria-label", CREATE_ACCOUNT);
    root.append(dialog);
    showSignUp(dialog, accounts, () => {
      secured = true;
      dialog.remove();
      notice?.remove();
      menu.close();
    });
    dialog.append(textButton("Cancel", () => dialog.remove()));
  };

  const menu = accountMenu(accountBar, () => secured, openSignUp);
  if (!secured) {
    notice = notEncryptedNotice(openSignUp);
    root.prepend(notice);
  }
  await showTaskList(tasks, repository, generateId);
}

function notEncryptedNotice(onCreateAccount: () => void): HTMLElement {
  const notice = document.createElement("div");
  notice.setAttribute("role", "status");
  const text = document.createElement("p");
  text.textContent = NOT_ENCRYPTED;
  const dismiss = textButton("×", () => notice.remove());
  dismiss.setAttribute("aria-label", "Dismiss");
  notice.append(text, textButton(CREATE_ACCOUNT, onCreateAccount), dismiss);
  return notice;
}

/** The Account button, which opens and closes a small menu above it. */
function accountMenu(bar: HTMLElement, isSecured: () => boolean, onCreateAccount: () => void) {
  let menu: HTMLElement | undefined;
  const close = () => {
    menu?.remove();
    menu = undefined;
  };
  const open = () => {
    menu = document.createElement("div");
    menu.setAttribute("role", "menu");
    if (isSecured()) {
      const status = document.createElement("p");
      status.textContent = ENCRYPTED;
      menu.append(status);
    } else {
      const item = textButton(CREATE_ACCOUNT, () => {
        close();
        onCreateAccount();
      });
      item.setAttribute("role", "menuitem");
      menu.append(item);
    }
    bar.append(menu);
  };
  bar.append(textButton("Account", () => (menu ? close() : open())));
  return { close };
}

function textButton(text: string, onClick: () => void): HTMLButtonElement {
  const button = document.createElement("button");
  button.type = "button";
  button.textContent = text;
  button.addEventListener("click", onClick);
  return button;
}
