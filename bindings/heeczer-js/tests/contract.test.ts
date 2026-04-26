/**
 * Contract tests for plan 0001 / ADR-0002 — TypeScript/JavaScript binding.
 *
 * Verifies:
 * 1. Every valid fixture round-trips through the typed Event interface
 *    without data loss (parse → cast → stringify → re-parse == original).
 * 2. Extension fields under meta.extensions survive a round-trip.
 *
 * Note: Unknown top-level field enforcement is provided by the server-side
 * JSON Schema validator (Event interface is structural in TypeScript).
 * The TypeScript type system rejects extra properties in object literals
 * at compile time — see the @ts-expect-error block below.
 */

import { describe, expect, it } from "vitest";
import {
  HeeczerValidationError,
  validateEvent,
  type Event,
} from "../src/index.js";

// Load all valid fixture files as raw strings via Vite's glob import.
// Keys are relative paths; values are raw JSON strings.
// import.meta.glob is a Vite-specific runtime feature; cast via unknown to
// avoid requiring vite/client type definitions in devDependencies.
type GlobFn = (
  pattern: string,
  opts: { query: string; import: string; eager: boolean },
) => Record<string, string>;
const fixtureRaw = (import.meta as unknown as { glob: GlobFn }).glob(
  "../../../core/schema/fixtures/events/valid/*.json",
  {
    query: "?raw",
    import: "default",
    eager: true,
  },
);

function loadValidFixtures(): Array<{ name: string; body: string }> {
  return Object.entries(fixtureRaw)
    .map(([path, body]) => ({
      name: path.split("/").at(-1) ?? path,
      body,
    }))
    .sort((a, b) => a.name.localeCompare(b.name));
}

describe("Event contract: valid fixture round-trips", () => {
  const fixtures = loadValidFixtures();

  it("loads at least one valid fixture", () => {
    expect(fixtures.length).toBeGreaterThan(0);
  });

  for (const { name, body } of fixtures) {
    it(`round-trips ${name} losslessly`, () => {
      // Parse the fixture as an Event (TypeScript cast — no runtime stripping).
      const event = JSON.parse(body) as Event;
      validateEvent(event);

      // Re-serialize and re-parse for semantic comparison.
      const roundTripped = JSON.parse(JSON.stringify(event));
      const original = JSON.parse(body);

      // Must be semantically equal (key order may differ).
      expect(roundTripped).toEqual(original);
    });
  }
});

describe("Event contract: extensions round-trip", () => {
  it("meta.extensions survives JSON round-trip", () => {
    const event: Event = {
      spec_version: "1.0",
      event_id: "00000000-0000-4000-8000-aabbccddeeff",
      timestamp: "2026-04-22T10:00:00Z",
      framework_source: "test",
      workspace_id: "ws_ext",
      task: { name: "ext_test", outcome: "success" },
      metrics: { duration_ms: 100 },
      meta: {
        sdk_language: "node",
        sdk_version: "0.1.0",
        extensions: { custom_key: 42, nested: { x: true } },
      },
    };

    const roundTripped = JSON.parse(JSON.stringify(event)) as Event;

    expect(roundTripped.meta.extensions).toBeDefined();
    expect(
      (roundTripped.meta.extensions as Record<string, unknown>)["custom_key"],
    ).toBe(42);
    expect(
      (
        (roundTripped.meta.extensions as Record<string, unknown>)[
          "nested"
        ] as Record<string, unknown>
      )["x"],
    ).toBe(true);
  });

  it("absent optional fields remain absent after round-trip", () => {
    const event: Event = {
      spec_version: "1.0",
      event_id: "00000000-0000-4000-8000-000000000001",
      timestamp: "2026-04-22T10:00:00Z",
      framework_source: "test",
      workspace_id: "ws_min",
      task: { name: "min_task", outcome: "success" },
      metrics: { duration_ms: 50 },
      meta: { sdk_language: "node", sdk_version: "0.1.0" },
    };

    const roundTripped = JSON.parse(JSON.stringify(event)) as Event;
    // Optional fields not set must not appear.
    expect(roundTripped.correlation_id).toBeUndefined();
    expect(roundTripped.identity).toBeUndefined();
    expect(roundTripped.context).toBeUndefined();
    expect(roundTripped.meta.extensions).toBeUndefined();
  });
});

describe("Event contract: TypeScript unknown-field rejection", () => {
  it("TypeScript type system rejects unknown properties at compile time", () => {
    // The @ts-expect-error below is validated by `pnpm typecheck` (tsc --noEmit).
    // If TypeScript catches the excess property, the directive suppresses the
    // error cleanly. If TypeScript stops catching it, tsc will error on the
    // directive itself — making the compile-time guarantee self-enforcing.
    const _bad: Event = {
      spec_version: "1.0",
      event_id: "00000000-0000-4000-8000-000000000002",
      timestamp: "2026-04-22T10:00:00Z",
      framework_source: "test",
      workspace_id: "ws_ts",
      task: { name: "t", outcome: "success" },
      metrics: { duration_ms: 1 },
      meta: { sdk_language: "node", sdk_version: "0.1.0" },
      // @ts-expect-error – forbidden_extra_field is not in Event (excess property check)
      forbidden_extra_field: "value",
    };
    void _bad; // prevent unused variable warning
    expect(true).toBe(true);
  });

  it("runtime validator rejects unknown top-level fields before transport", () => {
    expect(() =>
      validateEvent({
        spec_version: "1.0",
        event_id: "00000000-0000-4000-8000-000000000002",
        timestamp: "2026-04-22T10:00:00Z",
        framework_source: "test",
        workspace_id: "ws_ts",
        task: { name: "t", outcome: "success" },
        metrics: { duration_ms: 1 },
        meta: { sdk_language: "node", sdk_version: "0.5.1" },
        forbidden_extra_field: "value",
      }),
    ).toThrow(HeeczerValidationError);
  });
});
