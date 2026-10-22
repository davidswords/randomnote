import { describe, expect, it } from "vitest";
import { newId } from "./id";

const fixedClock = () => 1761134400000; // 2025-10-22T12:00:00Z
const allBits = (length: number) => new Uint8Array(length).fill(0xff);
const noBits = (length: number) => new Uint8Array(length);

describe("newId", () => {
  it("starts with the creation time in milliseconds", () => {
    expect(newId(fixedClock, noBits)).toBe(
      "019a0bca-5200-7000-8000-000000000000",
    );
  });

  it("fills the rest with random bits, keeping the version and variant", () => {
    expect(newId(fixedClock, allBits)).toBe(
      "019a0bca-5200-7fff-bfff-ffffffffffff",
    );
  });

  it("sorts by creation time", () => {
    const earlier = newId(() => 1761134400000, allBits);
    const later = newId(() => 1761134400001, noBits);

    expect(earlier < later).toBe(true);
  });
});
