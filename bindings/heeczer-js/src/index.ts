/**
 * Thin TypeScript client for the ai-heeczer ingestion service (plan 0005).
 *
 * The client is intentionally small: it speaks the ingestion service's
 * envelope-version-1 JSON contract (mirrors the C ABI envelope from
 * ADR-0011) and surfaces typed errors. It does NOT embed the Rust scoring
 * core; for in-process scoring without a network hop, use the future
 * `@cognizhi/heeczer-sdk-core` (FFI binding, plan 0005 follow-up).
 */

// ─── Canonical event types (generated from core/schema/event.v1.json) ──────
// Mirrors the Rust `event.rs` structs in heeczer-core (plan 0001 / ADR-0002).

/** Outcome of a task (closed enum). */
export type Outcome = "success" | "partial_success" | "failure" | "timeout";

/** Risk classification of a task context (closed enum). */
export type RiskClass = "low" | "medium" | "high";

/** Optional identity block: identifies the user/team/tier making the request. */
export interface EventIdentity {
  user_id?: string | null;
  team_id?: string | null;
  business_unit_id?: string | null;
  /** Resolved against the active TierSet (PRD §14.2.1). */
  tier_id?: string | null;
}

/** Task descriptor block. */
export interface EventTask {
  name: string;
  /** Optional; missing/null normalises to `"uncategorized"` per PRD §14.2.1. */
  category?: string | null;
  sub_category?: string | null;
  outcome: Outcome;
}

/** Required telemetry metrics block. */
export interface EventMetrics {
  /** Wall-clock task duration in milliseconds (required). */
  duration_ms: number;
  tokens_prompt?: number | null;
  tokens_completion?: number | null;
  tool_call_count?: number | null;
  workflow_steps?: number | null;
  retries?: number | null;
  artifact_count?: number | null;
  output_size_proxy?: number | null;
}

/** Optional execution context block. */
export interface EventContext {
  human_in_loop?: boolean | null;
  review_required?: boolean | null;
  temperature?: number | null;
  risk_class?: RiskClass | null;
  tags?: string[] | null;
}

/** SDK metadata block. `extensions` is the sole permitted bucket for
 *  unknown fields (PRD §13 / ADR-0002). */
export interface EventMeta {
  /** SDK language identifier (`"node"`, `"python"`, `"go"`, `"java"`, `"rust"`, …). */
  sdk_language: string;
  sdk_version: string;
  /** Override scoring profile id. Omit to use the workspace default. */
  scoring_profile?: string | null;
  /** Sole permitted location for custom/unknown fields (PRD §13). */
  extensions?: Record<string, unknown> | null;
}

/**
 * Canonical ai-heeczer telemetry event (v1).
 *
 * Mirrors `heeczer_core::Event` (Rust) and the JSON Schema in
 * `core/schema/event.v1.json`. Construct this type and pass it to
 * {@link HeeczerClient.ingestEvent} as the `event` field.
 *
 * @example
 * ```ts
 * const event: Event = {
 *   spec_version: "1.0",
 *   event_id: crypto.randomUUID(),
 *   timestamp: new Date().toISOString(),
 *   framework_source: "langgraph",
 *   workspace_id: "ws_default",
 *   task: { name: "summarise_pr", category: "summarization", outcome: "success" },
 *   metrics: { duration_ms: 3200 },
 *   meta: { sdk_language: "node", sdk_version: "0.1.0" },
 * };
 * ```
 */
export interface Event {
  /** Must be the literal string `"1.0"` for v1 events. */
  spec_version: "1.0";
  /** RFC 4122 UUID; primary idempotency key (PRD §12.19). */
  event_id: string;
  correlation_id?: string | null;
  /** RFC 3339 / ISO 8601 timestamp in UTC. */
  timestamp: string;
  /** Originating framework slug (`"langgraph"`, `"google_adk"`, …). */
  framework_source: string;
  workspace_id: string;
  project_id?: string | null;
  identity?: EventIdentity | null;
  task: EventTask;
  metrics: EventMetrics;
  context?: EventContext | null;
  meta: EventMeta;
}

// ─── Client types ────────────────────────────────────────────────────────────

/** Confidence band, matches the Rust `ConfidenceBand` enum. */
export type ConfidenceBand = "Low" | "Medium" | "High";

/** Subset of the ScoreResult shape the SDK exposes as a typed surface.
 *  The wire format carries additional fields; we keep the type open via
 *  index signature so SDK consumers do not need updates when the engine
 *  adds non-breaking fields (per ADR-0003). */
export interface ScoreResult {
  scoring_version: string;
  spec_version: string;
  scoring_profile: string;
  category: string;
  final_estimated_minutes: string;
  estimated_hours: string;
  estimated_days: string;
  financial_equivalent_cost: string;
  confidence_score: string;
  confidence_band: ConfidenceBand;
  human_summary: string;
  [key: string]: unknown;
}

export interface IngestEventResponse {
  ok: true;
  envelope_version: "1";
  event_id: string;
  score: ScoreResult;
}

export interface VersionResponse {
  ok: true;
  envelope_version: "1";
  scoring_version: string;
  spec_version: string;
  service: string;
}

/** Closed enum of error kinds the ingestion service emits, mirrored from
 *  `services/heeczer-ingest/src/error.rs` (envelope_version 1). */
export type ApiErrorKind =
  | "schema"
  | "bad_request"
  | "scoring"
  | "storage"
  | "not_found"
  | "forbidden"
  | "feature_disabled";

/** Typed error thrown by every client method on a non-2xx response. */
export class HeeczerApiError extends Error {
  readonly kind: ApiErrorKind | "unknown";
  readonly status: number;
  constructor(status: number, kind: ApiErrorKind | "unknown", message: string) {
    super(`heeczer ${status} ${kind}: ${message}`);
    this.name = "HeeczerApiError";
    this.status = status;
    this.kind = kind;
  }
}

export interface HeeczerClientOptions {
  /** Base URL of the ingestion service, e.g. `https://ingest.example.com`. */
  baseUrl: string;
  /** Optional API key sent as `x-heeczer-api-key`. */
  apiKey?: string;
  /** Optional fetch implementation; defaults to global `fetch`. Useful for
   *  tests and for environments without a global fetch. */
  fetch?: typeof fetch;
}

interface ErrorEnvelope {
  ok: false;
  envelope_version: string;
  error: { kind: ApiErrorKind; message: string };
}

/** Minimal client for the ingestion service. */
export class HeeczerClient {
  readonly #baseUrl: string;
  readonly #apiKey: string | undefined;
  readonly #fetch: typeof fetch;

  constructor(opts: HeeczerClientOptions) {
    if (!opts.baseUrl) {
      throw new Error("HeeczerClient: baseUrl is required");
    }
    this.#baseUrl = opts.baseUrl.replace(/\/$/, "");
    this.#apiKey = opts.apiKey;
    this.#fetch = opts.fetch ?? globalThis.fetch.bind(globalThis);
  }

  /** Liveness probe; resolves with `true` if the service responds 2xx. */
  async healthz(): Promise<boolean> {
    const resp = await this.#fetch(`${this.#baseUrl}/healthz`, {
      method: "GET",
    });
    return resp.ok;
  }

  /** Returns the engine + spec versions advertised by the service. */
  async version(): Promise<VersionResponse> {
    return this.#getJson<VersionResponse>("/v1/version");
  }

  /** Validate, score, and persist a single canonical event. */
  async ingestEvent(input: {
    workspaceId: string;
    event: unknown;
  }): Promise<IngestEventResponse> {
    return this.#postJson<IngestEventResponse>("/v1/events", {
      workspace_id: input.workspaceId,
      event: input.event,
    });
  }

  /** Run the scoring pipeline back-to-back without persisting. Requires the
   *  test-orchestration feature flag and the `x-heeczer-tester` header on
   *  the server side; the client always sends the header so deployments
   *  with the feature off will return a structured `feature_disabled`
   *  error and deployments without the role will return `forbidden`. */
  async testScorePipeline(input: {
    event: unknown;
    profile?: unknown;
    tierSet?: unknown;
    tierOverride?: string;
  }): Promise<{ ok: true; envelope_version: "1"; score: ScoreResult }> {
    const body: Record<string, unknown> = { event: input.event };
    if (input.profile !== undefined) body["profile"] = input.profile;
    if (input.tierSet !== undefined) body["tier_set"] = input.tierSet;
    if (input.tierOverride !== undefined) {
      body["tier_override"] = input.tierOverride;
    }
    return this.#postJson("/v1/test/score-pipeline", body, {
      "x-heeczer-tester": "1",
    });
  }

  async #getJson<T>(path: string): Promise<T> {
    const resp = await this.#fetch(`${this.#baseUrl}${path}`, {
      method: "GET",
      headers: this.#headers(),
    });
    return this.#handle<T>(resp);
  }

  async #postJson<T>(
    path: string,
    body: unknown,
    extraHeaders: Record<string, string> = {},
  ): Promise<T> {
    const resp = await this.#fetch(`${this.#baseUrl}${path}`, {
      method: "POST",
      headers: {
        ...this.#headers(),
        "content-type": "application/json",
        ...extraHeaders,
      },
      body: JSON.stringify(body),
    });
    return this.#handle<T>(resp);
  }

  #headers(): Record<string, string> {
    return this.#apiKey ? { "x-heeczer-api-key": this.#apiKey } : {};
  }

  async #handle<T>(resp: Response): Promise<T> {
    const text = await resp.text();
    if (resp.ok) {
      return JSON.parse(text) as T;
    }
    let kind: ApiErrorKind | "unknown" = "unknown";
    let message = text || resp.statusText;
    try {
      const env = JSON.parse(text) as ErrorEnvelope;
      if (
        env &&
        env.ok === false &&
        env.error &&
        typeof env.error.kind === "string"
      ) {
        kind = env.error.kind;
        message = env.error.message;
      }
    } catch {
      // Non-JSON error body; fall through with the raw text.
    }
    throw new HeeczerApiError(resp.status, kind, message);
  }
}
