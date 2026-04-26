# heeczer-go

Go client for the [ai-heeczer](https://github.com/cognizhi/ai-heeczer) ingestion service.

> ⚠️ Pre-1.0 surface. The HTTP envelope contract (envelope_version `1`) is
> stable; the typed wrapper API may evolve until we ship `1.0.0`.

## Install

> **Pre-release.** No release tag has been pushed to the canonical module
> path yet (see plan 0012). For local development, the
> [`examples/go/`](../../examples/go/) module pulls the SDK in via a
> `replace` directive.

```bash
go get github.com/cognizhi/ai-heeczer/bindings/heeczer-go
```

Requires Go ≥ 1.22. The client uses only the standard library (`net/http`,
`encoding/json`).

## Usage

```go
import (
    "context"
    heeczer "github.com/cognizhi/ai-heeczer/bindings/heeczer-go"
)

client, err := heeczer.New("https://ingest.example.com",
    heeczer.WithAPIKey(os.Getenv("HEECZER_API_KEY")),
    heeczer.WithMode(heeczer.ModeImage),
    heeczer.WithRetry(2, 100*time.Millisecond))
if err != nil { /* … */ }

resp, err := client.IngestEvent(ctx, "ws_default", canonicalEvent)
if err != nil {
    if heeczer.IsKind(err, heeczer.ErrSchema) {
        // handle schema rejection
    }
    return err
}
fmt.Println(resp.Score.FinalEstimatedMinutes, resp.Score.ConfidenceBand)
```

## Error handling

Every method returns `*heeczer.APIError` on a non-2xx response. The error
carries the closed `Kind` enum mirrored from the ingestion service
envelope:

| Kind                 | When                                                       |
| -------------------- | ---------------------------------------------------------- |
| `ErrSchema`          | Event failed canonical schema validation.                  |
| `ErrBadRequest`      | Malformed JSON or missing top-level fields.                |
| `ErrScoring`         | Engine rejected a normalized event (e.g. unknown tier id). |
| `ErrStorage`         | Persistence layer error.                                   |
| `ErrNotFound`        | Read endpoint did not find the resource.                   |
| `ErrUnauthorized`    | Missing, invalid, or revoked API key.                      |
| `ErrForbidden`       | Auth or RBAC denied the request.                           |
| `ErrConflict`        | Duplicate idempotency key or conflicting event payload.    |
| `ErrPayloadTooLarge` | Payload exceeded service limits.                           |
| `ErrRateLimited`     | Per-key or workspace quota was exceeded.                   |
| `ErrFeatureDisabled` | Endpoint exists but the feature flag is off.               |
| `ErrUnsupportedSpec` | Event `spec_version` is not accepted.                      |
| `ErrUnavailable`     | Readiness or dependency check failed.                      |
| `ErrUnknown`         | Non-JSON 5xx body; the raw text is in `Message`.           |

Use `heeczer.IsKind(err, heeczer.ErrSchema)` for typed branching.

## Functional options

| Option                                              | Description                                                                        |
| --------------------------------------------------- | ---------------------------------------------------------------------------------- |
| `WithAPIKey(string)`                                | Sets the `x-heeczer-api-key` header.                                               |
| `WithHTTPClient(heeczer.Doer)`                      | Inject a custom `Doer` (e.g. `*http.Client` with a transport, or a fake in tests). |
| `WithMode(heeczer.ModeImage \| heeczer.ModeNative)` | Selects image/native mode. Native fails fast until the cgo binding ships.          |
| `WithTimeout(time.Duration)`                        | Updates the default `*http.Client` timeout.                                        |
| `WithRetry(attempts, backoff, statuses...)`         | Retries transient transport/status failures.                                       |

## Methods

| Method                                 | HTTP                           | Returns                                                                       |
| -------------------------------------- | ------------------------------ | ----------------------------------------------------------------------------- |
| `Healthz(ctx)`                         | `GET /healthz`                 | `bool, error`                                                                 |
| `Version(ctx)`                         | `GET /v1/version`              | `*VersionResponse, error`                                                     |
| `IngestEvent(ctx, workspaceID, event)` | `POST /v1/events`              | `*IngestEventResponse, error`                                                 |
| `TestScorePipeline(ctx, req)`          | `POST /v1/test/score-pipeline` | `*TestPipelineResponse, error` (gated by the test-orchestration feature flag) |

## Contract

The SDK speaks `envelope_version: "1"` to the ingestion service per
[ADR-0011](../../docs/adr/0011-c-abi-envelope.md). Additive fields land
without breaking the typed surface.

## Runnable example

See [`examples/go/quickstart.go`](../../examples/go/quickstart.go) and the
cross-language index in [`examples/README.md`](../../examples/README.md).

## Common patterns

**Validate locally before sending** (avoids a network round-trip on bad
events). The schema is JSON Schema Draft 2020-12;
[`santhosh-tekuri/jsonschema/v6`](https://pkg.go.dev/github.com/santhosh-tekuri/jsonschema/v6)
works well:

```go
import (
    "os"
    jschema "github.com/santhosh-tekuri/jsonschema/v6"
)

f, _ := os.Open("core/schema/event.v1.json")
sch, _ := jschema.UnmarshalJSON(f)
if err := sch.Validate(eventMap); err != nil { /* … */ }
```

**Surface schema field errors from the service:**

```go
_, err := client.IngestEvent(ctx, "ws", event)
if heeczer.IsKind(err, heeczer.ErrSchema) {
    var apiErr *heeczer.APIError
    errors.As(err, &apiErr)
    fmt.Println("schema rejection:", apiErr.Message)
}
```

**Batching note.** The ingestion service exposes `POST /v1/events:batch`;
the SDK batch helper follows the public method expansion tracked in
[plan 0007](../../docs/plan/0007-sdk-go.md). Until then, send events
concurrently with goroutines + `errgroup`.

## License

MIT.
