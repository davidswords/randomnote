// Runs the Rust app, compiled to WebAssembly, in a dedicated worker: SQLite's
// storage in the browser (OPFS) only works there. Only the end-to-end tests
// reach this file.
import init, { WebApp } from "randomtask-web-wasm";
import { type Command, serveCommands } from "./serve-commands";

// Without a server, as until api.randomtask.ch is live, accounts stay in the browser.
const server: string | undefined = import.meta.env.VITE_RANDOMTASK_SERVER || undefined;

const opening: Promise<Command> = init().then(() => WebApp.open(server));
serveCommands(self as unknown as Parameters<typeof serveCommands>[0], opening);
