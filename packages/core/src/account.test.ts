import { describe, expect, it } from "vitest";
import { checkEmail, checkNewAccount, checkPassword } from "./account";

const EMAIL = "david@example.com";
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

describe("checkEmail", () => {
  it.each(["david@example.com", "  David@Example.COM ", "d.s+tasks@mail.example.co.uk"])(
    "accepts %j",
    (email) => {
      expect(checkEmail(email)).toBeUndefined();
    },
  );

  it.each(["", "   "])("asks for an email when it is %j", (email) => {
    expect(checkEmail(email)).toBe("Enter your email address.");
  });

  it.each([
    "david",
    "david@",
    "@example.com",
    "david@example",
    "david@.example.com",
    "david@example.",
    "da vid@example.com",
    "david@exam ple.com",
    "david@@example.com",
    "da@vid@example.com",
  ])("refuses %j, as the Rust side does", (email) => {
    expect(checkEmail(email)).toBe("Enter a valid email address.");
  });
});

describe("checkNewAccount", () => {
  it("accepts an email and matching passwords", () => {
    expect(
      checkNewAccount({ email: EMAIL, password: PASSWORD, confirmation: PASSWORD }),
    ).toBeUndefined();
  });

  it("explains a problem with the email", () => {
    expect(
      checkNewAccount({ email: "david", password: PASSWORD, confirmation: PASSWORD }),
    ).toBe("Enter a valid email address.");
  });

  it("refuses a password that is too short", () => {
    expect(
      checkNewAccount({ email: EMAIL, password: "too short", confirmation: "too short" }),
    ).toBe("The password must be at least 12 characters.");
  });

  it("refuses passwords that don't match", () => {
    expect(
      checkNewAccount({
        email: EMAIL,
        password: PASSWORD,
        confirmation: "correct horse battery stapler",
      }),
    ).toBe("The passwords don't match.");
  });
});
