# Plan 14 — Security and privacy

- **Status:** Active
- **Owner:** Security Engineer
- **PRD:** §22, §12.17, §12.18, §12.19
- **ADR:** ADR-0009 (release signing)

## Goal

Enforce the privacy-by-default and security-by-default posture from PRD §22, plus rate limiting, idempotency, retention/deletion, and supply-chain integrity.

## Checklist

### Privacy defaults (PRD §22)

- [ ] Schema validation rejects prompt/output content fields by default.
- [ ] Logs and audit entries never include payload bodies.
- [x] `docs/architecture/privacy.md` — data classification, log hygiene, retention/deletion flow, workspace isolation. (session Apr-2026)
- [x] `docs/architecture/security.md` — API key auth, TLS, CORS, RBAC, rate limiting, idempotency, supply chain, SLSA, threat model summary. (session Apr-2026)

### Auth and transport

- [ ] API-key auth with hashed storage and rotation.
- [ ] API-key rotation procedure: admin issues new key, both keys valid for an overlap window (default 24h, configurable), audit log entries on issue/revoke; documented in `docs/architecture/security.md`.
- [ ] mTLS option documented and tested.
- [ ] TLS 1.2+ enforced; HSTS on dashboard.
- [ ] Dashboard CORS policy: deny by default; explicit allowlist per workspace; documented in `docs/architecture/security.md`.

### RBAC

- [ ] Roles: `viewer`, `analyst`, `admin`, `owner`.
- [ ] Admin-only endpoints gated; tested.

### Rate limiting and quotas (PRD §12.18)

- [ ] Per-API-key token bucket.
- [ ] Per-workspace daily quota.
- [ ] 429 response shape documented.

### Idempotency (PRD §12.19)

- [ ] `Idempotency-Key` cache.
- [ ] Replay byte-equality test.

### Retention and deletion (PRD §12.17)

- [ ] Per-workspace retention windows.
- [ ] Hard-delete API + audit trail.
- [ ] Tombstone semantics tested.

### Supply chain

- [ ] CodeQL on Rust/JS/Python/Go/Java.
- [ ] cargo-audit, npm audit, pip-audit, govulncheck, OWASP dep-check.
- [ ] Trivy on container images.
- [ ] gitleaks on every PR.
- [x] cosign keyless signing of release artifacts. (`sign-artifacts` job in `release.yml`, session Apr-2026)
- [x] CycloneDX SBOM on every GitHub Release. (`generate-sbom` job in `release.yml`, session Apr-2026)
- [ ] SLSA Build Level 3 provenance attestation.

### Disclosure

- [x] `SECURITY.md` published. (repo root, links to GitHub private vulnerability reporting, session Apr-2026)
- [x] GitHub private vulnerability reporting enabled. (documented in `SECURITY.md`, session Apr-2026)

## Acceptance

- All security CI jobs green.
- Threat model reviewed and stored in `docs/architecture/threat-model.md`.
