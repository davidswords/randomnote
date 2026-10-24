// The app's screens, shared by the desktop and web apps. Each takes the
// account and its tasks through the interfaces in @randomtask/core.
export { showAbout } from "./about";
export type { AppInfo } from "./about";
export { showAccountScreen, showSignUp } from "./account-view";
export { showApp } from "./app-shell";
export type { AppServices } from "./app-shell";
export { showTaskList } from "./task-list-view";
export { CommandAccounts } from "./commands/command-accounts";
export { CommandTaskRepository } from "./commands/command-task-repository";
export type { Invoke } from "./commands/command-task-repository";
