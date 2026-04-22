import { describe, expect, it, vi } from "vitest";
import { HeeczerApiError, HeeczerClient } from "../src/index.js";

function jsonResponse(body: unknown, init: ResponseInit = {}): Response {
  return new Response(JSON.stringify(body), {
    status: init.status ?? 200,
    headers: { "content-type": "application/json", ...(init.headers ?? {}) },
  });
}

describe("HeeczerClient", () => {
  it("requires baseUrl", () => {
    expect(() => new HeeczerClient({ baseUrl: "" })).toThrow(/baseUrl/);
  });

  it("strips trailing slash from baseUrl", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL) => {
      expect(String(input)).toBe("https://api.example.com/healthz");
      return new Response(null, { status: 200 });
    });
    const client = new HeeczerClient({
      baseUrl: "https://api.example.com/",
      fetch: fetchMock as typeof fetch,
    });
    expect(await client.healthz()).toBe(true);
  });

  it("version returns the typed envelope", async () => {
    const fetchMock = vi.fn(async () =>
      jsonResponse({
        ok: true,
        envelope_version: "1",
        scoring_version: "1.0.0",
        spec_version: "1.0",
        service: "0.1.0",
      }),
    );
    const client = new HeeczerClient({
      baseUrl: "https://api.example.com",
      fetch: fetchMock as typeof fetch,
    });
    const v = await client.version();
    expect(v.scoring_version).toBe("1.0.0");
    expect(v.spec_version).toBe("1.0");
  });

  it("ingestEvent posts the canonical body shape", async () => {
    const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
      expect(String(input)).toBe("https://api.example.com/v1/events");
      expect(init?.method).toBe("POST");
      const body = JSON.parse(String(init?.body)) as Record<string, unknown>;
      expect(body["workspace_id"]).toBe("ws_test");
      expect(body["event"]).toEqual({ event_id: "evt-1" });
      return jsonResponse({
        ok: true,
        envelope_version: "1",
        event_id: "evt-1",
        score: {
          scoring_version: "1.0.0",
          spec_version: "1.0",
          scoring_profile: "default",
          category: "uncategorized",
          final_estimated_minutes: "1",
          estimated_hours: "0.02",
          estimated_days: "0.0025",
          financial_equivalent_cost: "1",
          confidence_score: "0.5",
          confidence_band: "Medium",
          human_summary: "ok",
        },
      });
    });
    const client = new HeeczerClient({
      baseUrl: "https://api.example.com",
      fetch: fetchMock as typeof fetch,
    });
    const r = await client.ingestEvent({
      workspaceId: "ws_test",
      event: { event_id: "evt-1" },
    });
    expect(r.event_id).toBe("evt-1");
    expect(r.score.confidence_band).toBe("Medium");
  });

  it("maps error envelopes to typed HeeczerApiError", async () => {
    const fetchMock = vi.fn(async () =>
      jsonResponse(
        {
          ok: false,
          envelope_version: "1",
          error: { kind: "schema", message: "missing field event_id" },
        },
        { status: 400 },
      ),
    );
    const client = new HeeczerClient({
      baseUrl: "https://api.example.com",
      fetch: fetchMock as typeof fetch,
    });
    await expect(
      client.ingestEvent({ workspaceId: "ws", event: {} }),
    ).rejects.toMatchObject({
      name: "HeeczerApiError",
      status: 400,
      kind: "schema",
    });
  });

  it("falls back to unknown kind for non-JSON error bodies", async () => {
    const fetchMock = vi.fn(
      async () => new Response("upstream timed out", { status: 504 }),
    );
    const client = new HeeczerClient({
      baseUrl: "https://api.example.com",
      fetch: fetchMock as typeof fetch,
    });
    try {
      await client.version();
      throw new Error("expected throw");
    } catch (err) {
      expect(err).toBeInstanceOf(HeeczerApiError);
      expect((err as HeeczerApiError).kind).toBe("unknown");
      expect((err as HeeczerApiError).status).toBe(504);
    }
  });

  it("testScorePipeline always sends the tester header", async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) => {
      const headers = init?.headers as Record<string, string>;
      expect(headers["x-heeczer-tester"]).toBe("1");
      return jsonResponse({
        ok: true,
        envelope_version: "1",
        score: {
          scoring_version: "1.0.0",
          spec_version: "1.0",
          scoring_profile: "default",
          category: "uncategorized",
          final_estimated_minutes: "1",
          estimated_hours: "0.02",
          estimated_days: "0.0025",
          financial_equivalent_cost: "1",
          confidence_score: "0.5",
          confidence_band: "Medium",
          human_summary: "ok",
        },
      });
    });
    const client = new HeeczerClient({
      baseUrl: "https://api.example.com",
      fetch: fetchMock as typeof fetch,
    });
    const r = await client.testScorePipeline({ event: {} });
    expect(r.score.confidence_band).toBe("Medium");
  });

  it("forwards api key when provided", async () => {
    const fetchMock = vi.fn(async (_input: RequestInfo | URL, init?: RequestInit) => {
      const headers = init?.headers as Record<string, string>;
      expect(headers["x-heeczer-api-key"]).toBe("k_secret");
      return jsonResponse({
        ok: true,
        envelope_version: "1",
        scoring_version: "1.0.0",
        spec_version: "1.0",
        service: "0.1.0",
      });
    });
    const client = new HeeczerClient({
      baseUrl: "https://api.example.com",
      apiKey: "k_secret",
      fetch: fetchMock as typeof fetch,
    });
    await client.version();
  });
});
