# heeczer

Idiomatic Rust SDK for [ai-heeczer](https://github.com/cognizhi/ai-heeczer).

> ⚠️ Pre-1.0 surface. The scoring math is stable (see ADR-0003); the SDK
> wrapper API may evolve until we ship `1.0.0`.

## Modes

| Feature flag       | Description                                              |
| ------------------ | -------------------------------------------------------- |
| `native` (default) | In-process scoring via `heeczer-core`. Zero network hop. |
| `http`             | Async HTTP client targeting the ingestion service.       |

## Usage (native mode)

```toml
[dependencies]
heeczer = "0.5.1"
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

## Runnable example

```bash
cargo run -p heeczer --example quickstart
```

See [`examples/quickstart.rs`](examples/quickstart.rs). The cross-language
index lives in [`examples/README.md`](../../examples/README.md).

## Loading a custom scoring profile or tier set

The defaults match the embedded `core/schema/profiles/default.v1.json` and
`core/schema/tiers/default.v1.json`. Pass overrides through `IngestInput`:

```rust
use heeczer::{Client, IngestInput, ScoringProfile, TierSet};

let profile: ScoringProfile = serde_json::from_str(&std::fs::read_to_string(
    "my-profile.json",
)?)?;
let tier_set: TierSet = serde_json::from_str(&std::fs::read_to_string(
    "my-tiers.json",
)?)?;

let result = Client::native().score_event(IngestInput {
    workspace_id: "ws_finance".into(),
    event,
    profile: Some(profile),
    tier_set: Some(tier_set),
    tier_override: Some("tier_senior_eng".into()),
})?;
```

## Errors

`heeczer::Error` wraps `heeczer_core::Error`. The variants are:

| Variant                        | Cause                                                                            |
| ------------------------------ | -------------------------------------------------------------------------------- |
| `Schema { path, message }`     | Event failed schema validation.                                                  |
| `MissingRequired(field)`       | Required non-derivable field absent (PRD §14.2.1).                               |
| `UnknownEnum { value, field }` | Closed enum received an unexpected value (e.g. bad outcome).                     |
| `UnknownTier(id)`              | `identity.tier_id` (or `tier_override`) does not exist in the supplied tier set. |
| `Overflow`                     | Decimal arithmetic overflowed the supported range.                               |
| `Json(_)`                      | JSON (de)serialization failure.                                                  |

See [`heeczer_core::Error`](https://docs.rs/heeczer-core) for the canonical definitions.

## Contract

The HTTP-mode client (feature `http`, plan 0008 follow-up) will speak
`envelope_version: "1"` to the ingestion service per
[ADR-0011](../../docs/adr/0011-c-abi-envelope.md) and surface the closed
`kind` enum from the wire envelope. Native mode operates entirely on the
local `heeczer_core::Error` enum above.

Contract tests score every shared valid fixture in native mode, and the
`http` feature has WireMock-backed coverage for envelope success and error
responses.

## License

MIT.
