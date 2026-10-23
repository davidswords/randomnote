import { describe, expect, it } from "vitest";
import { checkNewAccount, checkPassword } from "./account";

const PASSWORD = "correct horse battery staple";

describe("checkPassword", () => {
  it("accepts a password of exactly 12 characters", () => {
    expect(checkPassword("123456789012")).toBeUndefined();
  });

  it("refuses a password of 11 characters", () => {
    expect(checkPassword("12345678901")).toBe(
      "The password must be at least 12 characters.",
    );
  });

  it("counts characters rather than UTF-16 code units, as the Rust side does", () => {
    expect(checkPassword("🔑🔑🔑🔑🔑🔑🔑🔑🔑🔑🔑")).toBe(
      "The password must be at least 12 characters.",
    );
  });
});

describe("checkNewAccount", () => {
  it("accepts a username and matching passwords", () => {
    expect(
      checkNewAccount({ username: "david", password: PASSWORD, confirmation: PASSWORD }),
    ).toBeUndefined();
  });

  it.each(["", "   "])("asks for a username when it is %j", (username) => {
    expect(
      checkNewAccount({ username, password: PASSWORD, confirmation: PASSWORD }),
    ).toBe("Choose a username.");
  });

  it("refuses a password that is too short", () => {
    expect(
      checkNewAccount({ username: "david", password: "too short", confirmation: "too short" }),
    ).toBe("The password must be at least 12 characters.");
  });

  it("refuses passwords that don't match", () => {
    expect(
      checkNewAccount({
        username: "david",
        password: PASSWORD,
        confirmation: "correct horse battery stapler",
      }),
    ).toBe("The passwords don't match.");
  });
});
