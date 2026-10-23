import { InMemoryAccounts } from "@randomtask/core";
import type { Invoke } from "./tauri-task-repository";

type Call = { command: string; args?: Record<string, unknown> };

/**
 * Stands in for the Rust account commands behind Tauri's invoke: same
 * command names and arguments, and refusals as plain message strings, as
 * Tauri rejects with a command's Err value.
 */
export function fakeAccountCommands(): { invoke: Invoke; calls: Call[] } {
  const accounts = new InMemoryAccounts();
  const calls: Call[] = [];

  const run = async (command: string, args?: Record<string, unknown>) => {
    switch (command) {
      case "has_account":
        return accounts.hasAccount();
      case "create_account":
        return accounts.createAccount(args?.email as string, args?.password as string);
      case "unlock":
        return accounts.unlock(args?.password as string);
      default:
        throw new Error(`Unknown command: ${command}`);
    }
  };

  const invoke = (async (command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args });
    try {
      return (await run(command, args)) ?? null;
    } catch (error) {
      throw (error as Error).message;
    }
  }) as Invoke;

  return { invoke, calls };
}
