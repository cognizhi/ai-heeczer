// Quickstart: submit an event to the ingestion service via the JS SDK.
//
// Prereq: ingestion service running locally (cargo run -p heeczer-ingest).
//
// Run:
//   pnpm --dir bindings/heeczer-js install   # one-time
//   pnpm --dir bindings/heeczer-js build
//   node examples/node/quickstart.mjs
//
// Or import the published package once it's released:
//   import { HeeczerClient } from "@cognizhi/heeczer-sdk";

import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { HeeczerClient, HeeczerApiError } from "../../bindings/heeczer-js/dist/index.js";

const here = dirname(fileURLToPath(import.meta.url));
const eventPath = join(here, "..", "event.json");
const event = JSON.parse(await readFile(eventPath, "utf8"));

const baseUrl = process.env.HEECZER_BASE_URL ?? "http://127.0.0.1:8080";
const client = new HeeczerClient({
  baseUrl,
  apiKey: process.env.HEECZER_API_KEY,
});

console.log("» service version:", await client.version());

try {
  const { score, event_id } = await client.ingestEvent({
    workspaceId: "ws_default",
    event,
  });
  console.log(`» event ${event_id} ingested`);
  console.log(`» summary: ${score.human_summary}`);
  console.log(`» minutes=${score.final_estimated_minutes} band=${score.confidence_band}`);
} catch (err) {
  if (err instanceof HeeczerApiError) {
    console.error(`SDK error: kind=${err.kind} status=${err.status} message=${err.message}`);
    process.exit(1);
  }
  throw err;
}
