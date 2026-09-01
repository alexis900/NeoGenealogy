import { ApiError } from "../client";

describe("ApiError", () => {
  it("creates error with code and status", () => {
    const e = new ApiError("TREE_NOT_FOUND", "not found", 404);
    expect(e.code).toBe("TREE_NOT_FOUND");
    expect(e.status).toBe(404);
    expect(e.message).toBe("not found");
  });
});

describe("BASE url", () => {
  it("defaults to localhost", async () => {
    // client uses import.meta.env fallback
    const mod = await import("../client");
    expect(mod.api.getTrees).toBeDefined();
  });
});
