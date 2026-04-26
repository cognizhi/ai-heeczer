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

/** SDK execution mode. `image` speaks to the ingestion service over HTTP.
 * `native` is reserved for the future napi-rs binding and fails fast today. */
export type HeeczerMode = "image" | "native";

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
  | "unauthorized"
  | "forbidden"
  | "conflict"
  | "payload_too_large"
  | "rate_limit_exceeded"
  | "feature_disabled"
  | "unsupported_spec_version"
  | "unavailable";

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

/** Raised before transport when an event fails the local v1 contract check. */
export class HeeczerValidationError extends Error {
  readonly kind = "schema" as const;

  constructor(message: string) {
    super(`heeczer schema: ${message}`);
    this.name = "HeeczerValidationError";
  }
}

export interface RetryPolicy {
  /** Total attempts, including the first request. Set to 1 to disable retries. */
  attempts?: number;
  /** Initial backoff in milliseconds. Retries use exponential backoff. */
  backoffMs?: number;
  /** HTTP statuses that are safe to retry. */
  statusCodes?: number[];
}

export interface HeeczerClientOptions {
  /** Base URL of the ingestion service, e.g. `https://ingest.example.com`. */
  baseUrl: string;
  /** Execution mode. `image` is available today; `native` requires napi-rs. */
  mode?: HeeczerMode;
  /** Optional API key sent as `x-heeczer-api-key`. */
  apiKey?: string;
  /** Optional fetch implementation; defaults to global `fetch`. Useful for
   *  tests and for environments without a global fetch. */
  fetch?: typeof fetch;
  /** Request timeout in milliseconds. Defaults to 10 seconds. */
  timeoutMs?: number;
  /** Retry policy for transient transport failures and retryable status codes. */
  retry?: false | RetryPolicy;
  /** Validate canonical events locally before transport. Defaults to true. */
  validateEvents?: boolean;
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
  readonly #timeoutMs: number;
  readonly #retry: Required<RetryPolicy> | null;
  readonly #validateEvents: boolean;

  constructor(opts: HeeczerClientOptions) {
    if (!opts.baseUrl) {
      throw new Error("HeeczerClient: baseUrl is required");
    }
    if ((opts.mode ?? "image") === "native") {
      throw new Error(
        "HeeczerClient: native mode requires the deferred napi-rs binding; use mode: 'image'",
      );
    }
    this.#baseUrl = opts.baseUrl.replace(/\/$/, "");
    this.#apiKey = opts.apiKey;
    this.#fetch = opts.fetch ?? globalThis.fetch.bind(globalThis);
    this.#timeoutMs = opts.timeoutMs ?? 10_000;
    this.#retry = normaliseRetry(opts.retry);
    this.#validateEvents = opts.validateEvents ?? true;
  }

  /** Liveness probe; resolves with `true` if the service responds 2xx. */
  async healthz(): Promise<boolean> {
    const resp = await this.#request(`${this.#baseUrl}/healthz`, {
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
    if (this.#validateEvents) validateEvent(input.event);
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
    const resp = await this.#request(`${this.#baseUrl}${path}`, {
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
    const resp = await this.#request(`${this.#baseUrl}${path}`, {
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

  async #request(url: string, init: RequestInit): Promise<Response> {
    const attempts = this.#retry?.attempts ?? 1;
    let lastError: unknown;
    for (let attempt = 0; attempt < attempts; attempt += 1) {
      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), this.#timeoutMs);
      try {
        const resp = await this.#fetch(url, {
          ...init,
          signal: controller.signal,
        });
        if (!this.#shouldRetry(resp.status, attempt, attempts)) return resp;
        lastError = new HeeczerApiError(
          resp.status,
          "unknown",
          `retryable response status ${resp.status}`,
        );
      } catch (err) {
        lastError = controller.signal.aborted
          ? new HeeczerApiError(
              0,
              "unknown",
              `request timed out after ${this.#timeoutMs}ms`,
            )
          : err;
        if (attempt === attempts - 1) throw lastError;
      } finally {
        clearTimeout(timeout);
      }
      await sleep((this.#retry?.backoffMs ?? 0) * 2 ** attempt);
    }
    throw lastError instanceof Error ? lastError : new Error(String(lastError));
  }

  #shouldRetry(status: number, attempt: number, attempts: number): boolean {
    if (!this.#retry || attempt >= attempts - 1) return false;
    return this.#retry.statusCodes.includes(status);
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

function normaliseRetry(
  retry: false | RetryPolicy | undefined,
): Required<RetryPolicy> | null {
  if (retry === false) return null;
  return {
    attempts: Math.max(1, retry?.attempts ?? 2),
    backoffMs: Math.max(0, retry?.backoffMs ?? 100),
    statusCodes: retry?.statusCodes ?? [408, 429, 500, 502, 503, 504],
  };
}

function sleep(ms: number): Promise<void> {
  return ms <= 0
    ? Promise.resolve()
    : new Promise((resolve) => setTimeout(resolve, ms));
}

const TOP_LEVEL_KEYS = new Set([
  "spec_version",
  "event_id",
  "correlation_id",
  "timestamp",
  "framework_source",
  "workspace_id",
  "project_id",
  "identity",
  "task",
  "metrics",
  "context",
  "meta",
]);
const IDENTITY_KEYS = new Set([
  "user_id",
  "team_id",
  "business_unit_id",
  "tier_id",
]);
const TASK_KEYS = new Set(["name", "category", "sub_category", "outcome"]);
const METRICS_KEYS = new Set([
  "duration_ms",
  "tokens_prompt",
  "tokens_completion",
  "tool_call_count",
  "workflow_steps",
  "retries",
  "artifact_count",
  "output_size_proxy",
]);
const CONTEXT_KEYS = new Set([
  "human_in_loop",
  "review_required",
  "temperature",
  "risk_class",
  "tags",
]);
const META_KEYS = new Set([
  "sdk_language",
  "sdk_version",
  "scoring_profile",
  "extensions",
]);

/** Runtime v1 event validation for the Node SDK. It mirrors the canonical
 * schema closely enough to reject bad inputs before transport while keeping the
 * dependency footprint small for the pre-1.0 HTTP SDK. */
export function validateEvent(event: unknown): asserts event is Event {
  const root = objectAt(event, "event");
  forbidUnknown(root, TOP_LEVEL_KEYS, "event");
  literal(root["spec_version"], "1.0", "event.spec_version");
  string(root["event_id"], "event.event_id");
  optionalString(root["correlation_id"], "event.correlation_id");
  string(root["timestamp"], "event.timestamp");
  pattern(
    root["framework_source"],
    /^[a-z0-9][a-z0-9_.-]*$/,
    "event.framework_source",
  );
  pattern(root["workspace_id"], /^[a-zA-Z0-9_.-]+$/, "event.workspace_id");
  optionalPattern(root["project_id"], /^[a-zA-Z0-9_.-]+$/, "event.project_id");

  if (root["identity"] !== undefined && root["identity"] !== null) {
    const identity = objectAt(root["identity"], "event.identity");
    forbidUnknown(identity, IDENTITY_KEYS, "event.identity");
    for (const key of IDENTITY_KEYS)
      optionalString(identity[key], `event.identity.${key}`);
  }

  const task = objectAt(root["task"], "event.task");
  forbidUnknown(task, TASK_KEYS, "event.task");
  string(task["name"], "event.task.name");
  optionalPattern(
    task["category"],
    /^[a-z0-9][a-z0-9_]*$/,
    "event.task.category",
  );
  optionalPattern(
    task["sub_category"],
    /^[a-z0-9][a-z0-9_]*$/,
    "event.task.sub_category",
  );
  oneOf(
    task["outcome"],
    ["success", "partial_success", "failure", "timeout"],
    "event.task.outcome",
  );

  const metrics = objectAt(root["metrics"], "event.metrics");
  forbidUnknown(metrics, METRICS_KEYS, "event.metrics");
  integerInRange(
    metrics["duration_ms"],
    0,
    86_400_000,
    "event.metrics.duration_ms",
  );
  for (const key of ["tokens_prompt", "tokens_completion"] as const) {
    optionalIntegerInRange(metrics[key], 0, 10_000_000, `event.metrics.${key}`);
  }
  for (const key of [
    "tool_call_count",
    "workflow_steps",
    "artifact_count",
  ] as const) {
    optionalIntegerInRange(metrics[key], 0, 10_000, `event.metrics.${key}`);
  }
  optionalIntegerInRange(metrics["retries"], 0, 1_000, "event.metrics.retries");
  optionalNumberInRange(
    metrics["output_size_proxy"],
    0,
    1_000_000,
    "event.metrics.output_size_proxy",
  );

  if (root["context"] !== undefined && root["context"] !== null) {
    const context = objectAt(root["context"], "event.context");
    forbidUnknown(context, CONTEXT_KEYS, "event.context");
    optionalBoolean(context["human_in_loop"], "event.context.human_in_loop");
    optionalBoolean(
      context["review_required"],
      "event.context.review_required",
    );
    optionalNumberInRange(
      context["temperature"],
      0,
      2,
      "event.context.temperature",
    );
    if (context["risk_class"] !== undefined && context["risk_class"] !== null) {
      oneOf(
        context["risk_class"],
        ["low", "medium", "high"],
        "event.context.risk_class",
      );
    }
    if (context["tags"] !== undefined && context["tags"] !== null) {
      if (!Array.isArray(context["tags"]) || context["tags"].length > 32) {
        throw new HeeczerValidationError(
          "event.context.tags must be an array of at most 32 strings",
        );
      }
      for (const [idx, tag] of context["tags"].entries())
        string(tag, `event.context.tags[${idx}]`);
    }
  }

  const meta = objectAt(root["meta"], "event.meta");
  forbidUnknown(meta, META_KEYS, "event.meta");
  oneOf(
    meta["sdk_language"],
    ["rust", "node", "python", "go", "java", "cli", "test"],
    "event.meta.sdk_language",
  );
  string(meta["sdk_version"], "event.meta.sdk_version");
  optionalString(meta["scoring_profile"], "event.meta.scoring_profile");
  if (meta["extensions"] !== undefined && meta["extensions"] !== null) {
    objectAt(meta["extensions"], "event.meta.extensions");
  }
}

function objectAt(value: unknown, path: string): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new HeeczerValidationError(`${path} must be an object`);
  }
  return value as Record<string, unknown>;
}

function forbidUnknown(
  value: Record<string, unknown>,
  allowed: Set<string>,
  path: string,
): void {
  for (const key of Object.keys(value)) {
    if (!allowed.has(key))
      throw new HeeczerValidationError(`${path}.${key} is not allowed`);
  }
}

function literal(value: unknown, expected: string, path: string): void {
  if (value !== expected)
    throw new HeeczerValidationError(
      `${path} must be ${JSON.stringify(expected)}`,
    );
}

function string(value: unknown, path: string): void {
  if (typeof value !== "string" || value.length === 0) {
    throw new HeeczerValidationError(`${path} must be a non-empty string`);
  }
}

function optionalString(value: unknown, path: string): void {
  if (value === undefined || value === null) return;
  string(value, path);
}

function pattern(value: unknown, re: RegExp, path: string): void {
  string(value, path);
  const text = value as string;
  if (!re.test(text))
    throw new HeeczerValidationError(`${path} has invalid format`);
}

function optionalPattern(value: unknown, re: RegExp, path: string): void {
  if (value === undefined || value === null) return;
  pattern(value, re, path);
}

function oneOf(value: unknown, allowed: readonly string[], path: string): void {
  if (typeof value !== "string" || !allowed.includes(value)) {
    throw new HeeczerValidationError(
      `${path} must be one of ${allowed.join(", ")}`,
    );
  }
}

function integerInRange(
  value: unknown,
  min: number,
  max: number,
  path: string,
): void {
  if (
    !Number.isInteger(value) ||
    (value as number) < min ||
    (value as number) > max
  ) {
    throw new HeeczerValidationError(
      `${path} must be an integer between ${min} and ${max}`,
    );
  }
}

function optionalIntegerInRange(
  value: unknown,
  min: number,
  max: number,
  path: string,
): void {
  if (value === undefined || value === null) return;
  integerInRange(value, min, max, path);
}

function optionalNumberInRange(
  value: unknown,
  min: number,
  max: number,
  path: string,
): void {
  if (value === undefined || value === null) return;
  if (
    typeof value !== "number" ||
    Number.isNaN(value) ||
    value < min ||
    value > max
  ) {
    throw new HeeczerValidationError(
      `${path} must be a number between ${min} and ${max}`,
    );
  }
}

function optionalBoolean(value: unknown, path: string): void {
  if (value === undefined || value === null) return;
  if (typeof value !== "boolean")
    throw new HeeczerValidationError(`${path} must be boolean or null`);
}
