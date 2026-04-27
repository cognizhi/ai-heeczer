# Plan 14 — Security and privacy

- **Status:** Active
- **Owner:** Security Engineer
- **PRD:** §22, §12.17, §12.18, §12.19
- **ADR:** ADR-0009 (release signing)

## Goal

Enforce the privacy-by-default and security-by-default posture from PRD §22, plus rate limiting, idempotency, retention/deletion, and supply-chain integrity.

## Checklist

### Privacy defaults (PRD §22)

- [x] Schema validation rejects privacy-sensitive extension keys by default. (`heeczer-core::EventValidator` and `core/schema/event.v1.json` now reject prompt/output/attachment/secret/token/API-key-like keys under `meta.extensions`, session Apr-2026)
- [ ] Logs and audit entries never include payload bodies.
- [x] `docs/architecture/privacy.md` — data classification, log hygiene, retention/deletion flow, workspace isolation. (session Apr-2026)
- [x] `docs/architecture/security.md` — API key auth, TLS, CORS, RBAC, rate limiting, idempotency, supply chain, SLSA, threat model summary. (session Apr-2026)

### Auth and transport

- [x] API-key auth with hashed storage. (`services/heeczer-ingest/src/auth.rs`, session Apr-2026)
- [ ] API-key rotation procedure: admin issues new key, both keys valid for an overlap window (default 24h, configurable), audit log entries on issue/revoke; documented in `docs/architecture/security.md`.
- [ ] mTLS option documented and tested.
- [ ] TLS 1.2+ enforced; HSTS on dashboard.
- [ ] Dashboard CORS policy: deny by default; explicit allowlist per workspace; documented in `docs/architecture/security.md`.

### RBAC

- [x] Dashboard role taxonomy: `viewer`, `analyst`, `admin`, `owner`. (`dashboard/src/lib/session.ts` normalizes the planned role vocabulary, session Apr-2026)
- [ ] Admin-only endpoints gated; tested.

### Rate limiting and quotas (PRD §12.18)

- [x] Per-API-key token bucket. (`services/heeczer-ingest/src/auth.rs` + in-memory limiter, session Apr-2026)
- [x] Per-workspace daily quota. (`services/heeczer-ingest/src/handlers.rs`, session Apr-2026)
- [x] 429 response shape documented. (`docs/architecture/security.md`, session Apr-2026)

### Idempotency (PRD §12.19)

- [x] `Idempotency-Key` cache. (`services/heeczer-ingest/src/handlers.rs`, session Apr-2026)
- [x] Replay byte-equality test. (integration coverage in ingest tests, session Apr-2026)

### Retention and deletion (PRD §12.17)

- [ ] Per-workspace retention windows.
- [x] Hard-delete admin flow + audit trail. (`heec admin delete-event` + `heeczer-storage::admin::hard_delete_event`, session Apr-2026)
- [x] Tombstone semantics tested. (`core/heeczer-storage/tests/hard_delete.rs`, session Apr-2026)

### Supply chain

- [ ] CodeQL on Rust/JS/Python/Go/Java.
- [ ] cargo-audit, npm audit, pip-audit, govulncheck, OWASP dep-check.
- [ ] Trivy on container images.
- [ ] betterleaks on every PR.
- [x] cosign keyless signing of release artifacts. (`sign-artifacts` job in `release.yml`, session Apr-2026)
- [x] CycloneDX SBOM on every GitHub Release. (`generate-sbom` job in `release.yml`, session Apr-2026)
- [ ] SLSA Build Level 3 provenance attestation.

### Disclosure

- [x] `SECURITY.md` published. (repo root, links to GitHub private vulnerability reporting, session Apr-2026)
- [x] GitHub private vulnerability reporting enabled. (documented in `SECURITY.md`, session Apr-2026)

## Acceptance

- All security CI jobs green.
- Threat model reviewed and stored in `docs/architecture/threat-model.md`.
