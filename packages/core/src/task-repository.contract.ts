import { describe, expect, it } from "vitest";
import type { Task } from "./task";
import type { TaskRepository } from "./task-repository";

const aTask = (id: string, title: string): Task => ({ id, title });

/**
 * The behaviour every TaskRepository must have. Each implementation runs
 * this suite: the in-memory one as a unit test, real storage adapters as
 * integration tests.
 */
export function taskRepositoryContract(
  makeRepository: () => TaskRepository | Promise<TaskRepository>,
): void {
  describe("TaskRepository contract", () => {
    it("finds a saved task by its ID", async () => {
      const repository = await makeRepository();
      const task = aTask("task-1", "Buy milk");

      await repository.save(task);

      expect(await repository.findById("task-1")).toEqual(task);
    });

    it("finds the task with the given ID among several", async () => {
      const repository = await makeRepository();
      await repository.save(aTask("task-1", "Buy milk"));
      await repository.save(aTask("task-2", "Call the dentist"));

      expect(await repository.findById("task-2")).toEqual(
        aTask("task-2", "Call the dentist"),
      );
    });

    it("finds nothing for an ID that was never saved", async () => {
      const repository = await makeRepository();

      expect(await repository.findById("missing")).toBeUndefined();
    });

    it("lists nothing when no task was saved", async () => {
      const repository = await makeRepository();

      expect(await repository.list()).toEqual([]);
    });

    it("lists every saved task", async () => {
      const repository = await makeRepository();
      const milk = aTask("task-1", "Buy milk");
      const dentist = aTask("task-2", "Call the dentist");

      await repository.save(milk);
      await repository.save(dentist);

      const tasks = await repository.list();
      expect(tasks).toHaveLength(2);
      expect(tasks).toEqual(expect.arrayContaining([milk, dentist]));
    });

    it("updates a task in place when it is saved again with the same ID", async () => {
      const repository = await makeRepository();
      await repository.save(aTask("task-1", "Buy milk"));

      await repository.save(aTask("task-1", "Buy oat milk"));

      expect(await repository.findById("task-1")).toEqual(
        aTask("task-1", "Buy oat milk"),
      );
      expect(await repository.list()).toEqual([aTask("task-1", "Buy oat milk")]);
    });

    it("is not changed by editing a task after saving it", async () => {
      const repository = await makeRepository();
      const task = aTask("task-1", "Buy milk");
      await repository.save(task);

      task.title = "Changed";

      expect(await repository.findById("task-1")).toEqual(
        aTask("task-1", "Buy milk"),
      );
    });

    it("is not changed by editing a task it returned", async () => {
      const repository = await makeRepository();
      await repository.save(aTask("task-1", "Buy milk"));

      const found = await repository.findById("task-1");
      found!.title = "Changed";
      const [listed] = await repository.list();
      listed!.title = "Changed";

      expect(await repository.findById("task-1")).toEqual(
        aTask("task-1", "Buy milk"),
      );
    });
  });
}
