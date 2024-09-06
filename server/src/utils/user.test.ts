import { expect, test, describe } from "bun:test";
import { BadUserSpecFormat, parseUserspec } from "./user";

describe("parseUserspec", () => {
  test("uuid", () => {
    const spec = "550e8400-e29b-41d4-a716-446655440000"; // Example UUIDv4
    const result = parseUserspec(spec);
    expect(result).toEqual({ userId: spec });
  });

  test("@admin1234@example.com", () => {
    const spec = "@admin1234@example.com";
    const result = parseUserspec(spec);
    expect(result).toEqual({ username: "admin1234", hostname: "example.com" });
  });

  test("@admin1234", () => {
    const spec = "@admin1234";
    const result = parseUserspec(spec);
    expect(result).toEqual({ username: "admin1234", hostname: null });
  });

  test("admin@example.com (error)", () => {
    const spec = "admin@example.com";
    expect(() => parseUserspec(spec)).toThrow(BadUserSpecFormat);
  });
});
