import { describe, expect, it } from "vitest";

import { canAdmin, normalizeDashboardRole } from "@/lib/session";

describe("session role helpers", () => {
  it("normalizes planned dashboard roles", () => {
    expect(normalizeDashboardRole("viewer")).toBe("viewer");
    expect(normalizeDashboardRole("analyst")).toBe("analyst");
    expect(normalizeDashboardRole("admin")).toBe("admin");
    expect(normalizeDashboardRole("owner")).toBe("owner");
  });

  it("maps legacy tester to analyst and defaults unknown values to viewer", () => {
    expect(normalizeDashboardRole("tester")).toBe("analyst");
    expect(normalizeDashboardRole("unknown")).toBe("viewer");
    expect(normalizeDashboardRole(undefined)).toBe("viewer");
  });

  it("treats admin and owner as admin-capable roles", () => {
    expect(canAdmin("viewer")).toBe(false);
    expect(canAdmin("analyst")).toBe(false);
    expect(canAdmin("admin")).toBe(true);
    expect(canAdmin("owner")).toBe(true);
  });
});
