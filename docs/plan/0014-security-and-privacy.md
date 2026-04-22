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
- [ ] Data classification documented in `docs/architecture/privacy.md`.

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
- [ ] cosign keyless signing of release artifacts.
- [ ] CycloneDX SBOM on every GitHub Release.
- [ ] SLSA Build Level 3 provenance attestation.

### Disclosure
- [ ] `SECURITY.md` published.
- [ ] GitHub private vulnerability reporting enabled.

## Acceptance
- All security CI jobs green.
- Threat model reviewed and stored in `docs/architecture/threat-model.md`.
