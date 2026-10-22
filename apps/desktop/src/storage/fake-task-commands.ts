import type { Task } from "@randomnote/core";
import type { Invoke } from "./tauri-task-repository";

type Call = { command: string; args?: Record<string, unknown> };

/**
 * Stands in for the Rust task commands behind Tauri's invoke: same command
 * names and arguments, data copied through JSON as IPC does, and null for a
 * missing task, as Rust's None is serialized.
 */
export function fakeTaskCommands(): { invoke: Invoke; calls: Call[] } {
  const tasks = new Map<string, Task>();
  const calls: Call[] = [];
  const throughIpc = <T>(value: T): T => JSON.parse(JSON.stringify(value));

  const invoke = (async (command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args });
    switch (command) {
      case "save_task": {
        const task = throughIpc(args?.task as Task);
        tasks.set(task.id, task);
        return null;
      }
      case "find_task":
        return throughIpc(tasks.get(args?.id as string) ?? null);
      case "list_tasks":
        return throughIpc([...tasks.values()]);
      default:
        throw new Error(`Unknown command: ${command}`);
    }
  }) as Invoke;

  return { invoke, calls };
}
