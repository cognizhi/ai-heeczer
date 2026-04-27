# Security architecture

> Status: foundation slice — reflects PRD §22, ADR-0009, and plan 0014.
> Last reviewed: 2026-04-24
> Owner: Security Engineer

## Implemented In The Ingestion Slice

- API-key middleware for protected ingestion routes.
- SHA-256 lookup against `heec_api_keys.hashed_key`.
- Revocation check via `heec_api_keys.revoked_at`.
- Workspace scoping against the authenticated key.
- Auth-failure audit rows.
- Per-key and per-workspace rate/quota enforcement for ingestion routes.

## Target Platform Controls

The sections below describe the intended platform security model. Some controls, including key-management endpoints, dashboard RBAC, CORS enforcement, TLS termination, and native mTLS termination, are tracked in plan 0014 and are not fully implemented by the ingestion binary yet.

## API key authentication

API keys are issued per workspace. The platform stores only the SHA-256 hash
of each key; the raw key is shown once at issuance and never stored.

### Rotation procedure

1. Admin calls `POST /workspaces/{id}/api-keys` to issue a new key.
2. Both the old and new keys are valid for the configured overlap window
   (default 24 hours; configurable per workspace).
3. Admin revokes the old key via `DELETE /workspaces/{id}/api-keys/{key-id}`.
4. The audit log records an entry for each of: `api_key_issued` and
   `api_key_revoked`, containing `workspace_id`, `key_id` (not the key
   itself), `actor_id`, and `timestamp`.

No raw key material appears in any log, audit entry, or error response.

## Transport security

- TLS 1.2 or higher is required on all endpoints. TLS 1.0 and 1.1 are
  disabled.
- The dashboard sets `Strict-Transport-Security: max-age=63072000;
includeSubDomains; preload` (HSTS) on all responses.
- Mutual TLS (mTLS) remains the recommended production ingress pattern, but
  the Rust service does not terminate mTLS directly in the current slice. Put
  mTLS at the edge proxy or service mesh and keep API-key auth enabled behind it.
  Native `HEECZER_TLS_CLIENT_CA_PATH` support is a follow-up under plan 0014.

## CORS policy

The dashboard API enforces a deny-by-default CORS policy. Allowed origins are
configured as an explicit allowlist per workspace. Requests from origins not
on the allowlist receive `403 Forbidden`; no CORS headers are emitted. The
allowlist is stored in workspace settings and is editable only by `admin` or
`owner` roles.

## Role-based access control (RBAC)

| Role      | Scope                                                               |
| --------- | ------------------------------------------------------------------- |
| `viewer`  | Read-only access to scores, aggregates, and dashboard pages         |
| `analyst` | All viewer permissions plus export and calibration run triggers     |
| `admin`   | All analyst permissions plus workspace settings, API keys, and RBAC |
| `owner`   | All admin permissions plus workspace deletion and billing           |

Admin-only endpoints (workspace settings, key management, RBAC mutations) are
gated at the router layer. A middleware check rejects any request from a
non-admin identity before the handler is reached. Role assignments are stored
in the audit log when changed.

## Rate limiting

Rate limiting uses a token-bucket algorithm implemented via
[`tower-governor`](https://crates.io/crates/tower-governor).

| Scope             | Default limit                       |
| ----------------- | ----------------------------------- |
| Per API key       | 1,000 requests/minute (burst: 200)  |
| Per workspace/day | 5,000,000 events/day (configurable) |

When a limit is exceeded the server responds with:

```json
{
    "ok": false,
    "envelope_version": "1",
    "error": {
        "kind": "rate_limit_exceeded",
        "message": "request rate exceeded"
    }
}
```

HTTP status code `429 Too Many Requests` with a `Retry-After: 42` header
(seconds until the bucket refills). Quota responses also include
`X-Heeczer-Quota-Limit`, `X-Heeczer-Quota-Remaining`, and
`X-Heeczer-Quota-Reset-After`.

## Idempotency

### Ingest deduplication

`event_id` is the primary key of `heec_events`. A duplicate ingest request
with an already-seen `event_id` and the same normalized payload is accepted
(HTTP 200) and returns the original stored result. No second write occurs.

A duplicate `event_id` with a different normalized payload returns
`409 conflict` and writes an `ingest_conflict` audit entry. Callers must either
reuse the exact original event body or mint a new `event_id`.

### Batch `Idempotency-Key` header

Batch ingest endpoints accept an opaque `Idempotency-Key` request header up to 128 characters.
The key is cached for 24 hours. A replayed request with the same key within
that window returns the original response byte-for-byte. After 24 hours the
key expires and a new request will be processed normally.

Idempotency is asserted by a test that sends the same batch twice and
verifies the stored row count and response body are identical.

## Supply-chain integrity

| Tool             | Scope                         | Trigger                      |
| ---------------- | ----------------------------- | ---------------------------- |
| CodeQL           | Rust, JS/TS, Python, Go, Java | Push and PR to `main`        |
| `cargo-audit`    | Rust dependencies             | CI on every push             |
| `cargo-deny`     | Rust licenses + advisories    | CI on every push             |
| `pnpm audit`     | JS/TS dependencies            | CI on every push             |
| `pip-audit`      | Python dependencies           | CI on every push             |
| `govulncheck`    | Go dependencies               | CI on every push             |
| Trivy            | Container images              | On release image build       |
| betterleaks      | Secrets in source             | On every PR                  |
| cosign (keyless) | Release artifacts             | On every GitHub Release      |
| CycloneDX SBOM   | All languages                 | Generated per GitHub Release |

### SLSA provenance

ai-heeczer targets **SLSA Build Level 3** via GitHub Actions provenance
attestations. Release workflows use `slsa-github-generator` to produce
signed provenance for all release artifacts. Provenance files are attached
to GitHub Releases alongside the SBOM.

See [ADR-0009](../adr/0009-release-control-plane.md) for the full release
signing and provenance design.

## Threat model summary

The following OWASP Top 10 risks are in scope and have explicit mitigations:

| OWASP risk                        | Mitigation                                                      |
| --------------------------------- | --------------------------------------------------------------- |
| A01 — Broken access control       | RBAC middleware; workspace-scoped queries; unit-tested          |
| A02 — Cryptographic failures      | TLS 1.2+; SHA-256 key hashing; cosign keyless signing           |
| A03 — Injection                   | sqlx parameterized queries; JSON schema validation on ingest    |
| A05 — Security misconfiguration   | CORS deny-by-default; HSTS; edge/service-mesh mTLS guidance     |
| A06 — Vulnerable components       | cargo-audit, cargo-deny, pip-audit, govulncheck, pnpm audit     |
| A07 — Auth failures               | API key required; rotation with audit log; no key in logs       |
| A08 — Software integrity failures | cosign keyless + SLSA provenance; SBOM on every release         |
| A09 — Logging failures            | Structured logs; payload bodies never logged; audit trail       |
| A10 — SSRF                        | Outbound HTTP disabled in core; no user-supplied URL evaluation |

A fuller threat model is tracked in `docs/architecture/threat-model.md`
(plan 0014 deliverable).

## Security disclosure

Vulnerabilities should be reported via the process in
[`SECURITY.md`](../../SECURITY.md). GitHub private vulnerability reporting
is enabled on this repository.

Do not open a public issue for a security vulnerability.

## References

- PRD §22 — Security and privacy requirements
- [ADR-0009 — Release control plane](../adr/0009-release-control-plane.md)
- [Plan 0014 — Security and privacy](../plan/0014-security-and-privacy.md)
- [Privacy architecture](privacy.md)
- [`SECURITY.md`](../../SECURITY.md)
