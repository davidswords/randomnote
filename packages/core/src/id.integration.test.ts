import { describe, expect, it } from "vitest";
import { newId } from "./id";

const UUID_V7 =
  /^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/;

describe("newId with the real clock and Web Crypto", () => {
  it("is a UUID v7", () => {
    expect(newId()).toMatch(UUID_V7);
  });

  it("does not repeat", () => {
    const ids = new Set(Array.from({ length: 10_000 }, () => newId()));

    expect(ids.size).toBe(10_000);
  });
});
