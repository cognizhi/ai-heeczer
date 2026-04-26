# Re-score Runbook

Use `POST /v1/events/{event_id}/rescore` when an already-ingested event needs a new score because the scoring profile, tier set, or tier override changed. Re-scoring is append-only: it never mutates the original event or an existing score row.

## Preconditions

- The event exists in `heec_events` and has not been tombstoned.
- The caller has an API key for the event's workspace.
- The new scoring inputs are known and reproducible: default profile, explicit `profile`, explicit `tier_set`, or `tier_override`.

## Trigger

```bash
curl -sS \
  -X POST "http://localhost:8080/v1/events/${EVENT_ID}/rescore" \
  -H "content-type: application/json" \
  -H "x-heeczer-api-key: ${HEECZER_API_KEY}" \
  -d '{"workspace_id":"'"${WORKSPACE_ID}"'"}'
```

Optional body fields:

```text
{
    "workspace_id": "ws_123",
  "profile": { ... complete ScoringProfile JSON ... },
  "tier_set": { ... complete TierSet JSON ... },
    "tier_override": "tier_senior_eng"
}
```

## Verify

1. Confirm the HTTP response is an envelope with `ok: true` and the expected `event_id`.
2. Query `heec_scores` for the event and verify a row exists for the intended scoring tuple.
3. Query `heec_audit_log` for `action = 'rescore'` and `target_id = event_id` when a new score row was inserted.

If the request uses the exact same scoring tuple as an existing row, the endpoint returns `200 OK` with the score and does not write a duplicate audit entry.

## Downstream Notifications

After a successful re-score, refresh any dashboard panels or exported aggregates that depend on the affected workspace/event. Automated aggregate refresh and webhook notification are follow-up workflow work; until then, notify the workspace owner through the operational channel used for the incident or calibration change.
