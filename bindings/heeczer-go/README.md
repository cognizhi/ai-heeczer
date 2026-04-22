# heeczer-go

Go client for the [ai-heeczer](https://github.com/cognizhi/ai-heeczer) ingestion service.

> ⚠️ Pre-1.0 surface. The HTTP envelope contract (envelope_version `1`) is
> stable; the typed wrapper API may evolve until we ship `1.0.0`.

## Install

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
    heeczer.WithAPIKey(os.Getenv("HEECZER_API_KEY")))
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
envelope (`schema`, `bad_request`, `scoring`, `storage`, `not_found`,
`forbidden`, `feature_disabled`), plus an `unknown` fallback for non-JSON
5xx bodies.

## License

MIT.
