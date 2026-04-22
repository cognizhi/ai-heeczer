# @heeczer/sdk

JavaScript / TypeScript client for the [ai-heeczer](https://github.com/cognizhi/ai-heeczer) ingestion service.

> ⚠️ Pre-1.0 surface. The HTTP envelope contract (envelope_version `1`) is
> stable; the typed wrapper API may evolve until we ship `1.0.0`.

## Install

```bash
pnpm add @heeczer/sdk
```

Requires Node.js ≥ 20 (the SDK uses the global `fetch`).

## Usage

```ts
import { HeeczerClient } from "@heeczer/sdk";

const client = new HeeczerClient({
  baseUrl: "https://ingest.example.com",
  apiKey: process.env.HEECZER_API_KEY,
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
import { HeeczerApiError } from "@heeczer/sdk";

try {
  await client.ingestEvent({ workspaceId: "ws", event: badEvent });
} catch (err) {
  if (err instanceof HeeczerApiError && err.kind === "schema") {
    // …
  }
}
```

## Contract

The SDK speaks `envelope_version: "1"` to the ingestion service, which
mirrors the C ABI envelope contract documented in
[ADR-0011](../../docs/adr/0011-c-abi-envelope.md). Additive fields land
without breaking the typed surface (the `ScoreResult` interface keeps an
open index signature).

## License

MIT.
