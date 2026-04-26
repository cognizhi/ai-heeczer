# @cognizhi/heeczer-sdk

JavaScript / TypeScript client for the [ai-heeczer](https://github.com/cognizhi/ai-heeczer) ingestion service.

> ⚠️ Pre-1.0 surface. The HTTP envelope contract (envelope_version `1`) is
> stable; the typed wrapper API may evolve until we ship `1.0.0`.

## Install

> **Pre-release.** `@cognizhi/heeczer-sdk` is not on the npm registry yet (see
> plan 0012). Until then, install from source via the runnable example
> below.

```bash
pnpm add @cognizhi/heeczer-sdk
```

Requires Node.js ≥ 20. The package ships ESM and CJS exports; the default
transport uses the global `fetch`.

## Usage

```ts
import { HeeczerClient } from "@cognizhi/heeczer-sdk";

const client = new HeeczerClient({
    baseUrl: "https://ingest.example.com",
    apiKey: process.env.HEECZER_API_KEY,
    mode: "image",
    timeoutMs: 10_000,
    retry: { attempts: 2, backoffMs: 100 },
});

const { score } = await client.ingestEvent({
    workspaceId: "ws_default",
    event: canonicalEvent, // see core/schema/event.v1.json
});

console.log(score.final_estimated_minutes, score.confidence_band);
```

## Error handling

Every client method throws `HeeczerApiError` on a non-2xx response. The error
exposes the closed `kind` enum from the ingestion service's envelope:

```ts
import { HeeczerApiError } from "@cognizhi/heeczer-sdk";

try {
    await client.ingestEvent({ workspaceId: "ws", event: badEvent });
} catch (err) {
    if (err instanceof HeeczerApiError && err.kind === "schema") {
        // …
    }
}
```

## Configuration

| Option           | Type                   | Default                           | Description                                                                |
| ---------------- | ---------------------- | --------------------------------- | -------------------------------------------------------------------------- |
| `baseUrl`        | `string`               | required                          | Base URL of the ingestion service. Trailing slash is stripped.             |
| `mode`           | `"image" \| "native"`  | `"image"`                         | `image` is HTTP mode. `native` fails fast until the napi-rs binding ships. |
| `apiKey`         | `string \| undefined`  | `undefined`                       | Sent as `x-heeczer-api-key`.                                               |
| `fetch`          | `typeof fetch`         | `globalThis.fetch`                | Inject a custom `fetch` (e.g. `undici.fetch`, mocks in tests).             |
| `timeoutMs`      | `number`               | `10000`                           | Request timeout.                                                           |
| `retry`          | `false \| RetryPolicy` | `{ attempts: 2, backoffMs: 100 }` | Retries transient transport/status failures.                               |
| `validateEvents` | `boolean`              | `true`                            | Validate v1 events locally before `POST /v1/events`.                       |

## Methods

| Method                                                            | HTTP                           | Returns                                                                                                           |
| ----------------------------------------------------------------- | ------------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| `healthz()`                                                       | `GET /healthz`                 | `Promise<boolean>`                                                                                                |
| `version()`                                                       | `GET /v1/version`              | `Promise<VersionResponse>`                                                                                        |
| `ingestEvent({ workspaceId, event })`                             | `POST /v1/events`              | `Promise<IngestEventResponse>`                                                                                    |
| `testScorePipeline({ event, profile?, tierSet?, tierOverride? })` | `POST /v1/test/score-pipeline` | `Promise<{ ok: true; envelope_version: "1"; score: ScoreResult }>` (gated by the test-orchestration feature flag) |

## Error kinds

`HeeczerApiError.kind` is a closed string union mirroring the ingestion
service envelope:

| Kind                       | When                                                       |
| -------------------------- | ---------------------------------------------------------- |
| `schema`                   | Event failed canonical schema validation.                  |
| `bad_request`              | Malformed JSON or missing top-level fields.                |
| `scoring`                  | Engine rejected a normalized event (e.g. unknown tier id). |
| `storage`                  | Persistence layer error.                                   |
| `not_found`                | Read endpoint did not find the resource.                   |
| `unauthorized`             | Missing, invalid, or revoked API key.                      |
| `forbidden`                | Auth or RBAC denied the request.                           |
| `conflict`                 | Duplicate idempotency key or conflicting event payload.    |
| `payload_too_large`        | Payload exceeded service limits.                           |
| `rate_limit_exceeded`      | Per-key or workspace quota was exceeded.                   |
| `feature_disabled`         | Endpoint exists but the feature flag is off.               |
| `unsupported_spec_version` | Event `spec_version` is not accepted by the service.       |
| `unavailable`              | Readiness or dependency check failed.                      |
| `unknown`                  | Non-JSON 5xx body; the raw text is in `message`.           |

## Runnable example

See [`examples/node/quickstart.mjs`](../../examples/node/quickstart.mjs)
and the cross-language index in [`examples/README.md`](../../examples/README.md).

## Common patterns

**Validate locally before sending** (avoids a network round-trip on bad
events). `ingestEvent()` validates v1 events by default. You can also call
`validateEvent(event)` directly when building an event before handing it to
the client.

```ts
import { validateEvent } from "@cognizhi/heeczer-sdk";

validateEvent(event);
```

**Surface schema field errors from the service:**

```ts
try {
    await client.ingestEvent({ workspaceId: "ws", event });
} catch (err) {
    if (err instanceof HeeczerApiError && err.kind === "schema") {
        // err.message contains the field-level detail from the server envelope.
        console.error("schema rejection:", err.message);
    }
}
```

**Batching note.** The ingestion service exposes `POST /v1/events:batch`;
the SDK batch helper follows the public method expansion tracked in
[plan 0005](../../docs/plan/0005-sdk-jsts.md). Until then, call the single
event API sequentially or in parallel with `Promise.allSettled()`.

## Contract

The SDK speaks `envelope_version: "1"` to the ingestion service, which
mirrors the C ABI envelope contract documented in
[ADR-0011](../../docs/adr/0011-c-abi-envelope.md). Additive fields land
without breaking the typed surface (the `ScoreResult` interface keeps an
open index signature).

## License

MIT.
