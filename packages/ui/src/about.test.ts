// @vitest-environment happy-dom
import { beforeEach, describe, expect, it } from "vitest";
import { showAbout } from "./about";

let footer: HTMLElement;

beforeEach(() => {
  footer = document.createElement("footer");
});

describe("about footer", () => {
  it("shows the app's name and version", async () => {
    await showAbout(footer, {
      getName: async () => "randomtask",
      getVersion: async () => "0.2.1",
    });

    expect(footer.textContent).toBe("randomtask 0.2.1");
  });

  it("stays empty when the name or version can't be read", async () => {
    await showAbout(footer, {
      getName: async () => "randomtask",
      getVersion: async () => {
        throw new Error("not allowed");
      },
    });

    expect(footer.textContent).toBe("");
  });
});
