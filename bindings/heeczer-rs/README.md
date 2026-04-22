# heeczer

Idiomatic Rust SDK for [ai-heeczer](https://github.com/cognizhi/ai-heeczer).

> ⚠️ Pre-1.0 surface. The scoring math is stable (see ADR-0003); the SDK
> wrapper API may evolve until we ship `1.0.0`.

## Modes

| Feature flag | Description |
| --- | --- |
| `native` (default) | In-process scoring via `heeczer-core`. Zero network hop. |
| `http` | Async HTTP client targeting the ingestion service. |

## Usage (native mode)

```toml
[dependencies]
heeczer = "0.1"
```

```rust
use heeczer::{Client, IngestInput};

let client = Client::native();
let result = client.score_event(IngestInput {
    workspace_id: "ws_default".into(),
    event: my_event,
    profile: None,   // use default_v1
    tier_set: None,  // use default_v1
    tier_override: None,
})?;

println!("{} min   band={:?}", result.final_estimated_minutes, result.confidence_band);
```

## Re-exported core types

`heeczer` re-exports `Event`, `ScoreResult`, `ScoringProfile`, `TierSet`,
`ConfidenceBand` from `heeczer-core` so consumers need only one dependency.

## License

MIT.
