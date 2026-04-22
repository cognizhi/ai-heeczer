# Security Policy

We take security seriously. Thank you for helping keep ai-heeczer and its users safe.

## Supported versions

| Version | Supported          |
| ------- | ------------------ |
| latest minor of latest major | ✅ |
| previous minor of latest major | ✅ for critical / high CVEs |
| earlier versions | ❌ |

## Reporting a vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Email **security@cognizhi.ai** (replace with the project's actual address) with:

1. A description of the issue and its impact.
2. Steps to reproduce, ideally with a minimal proof of concept.
3. The version(s) and configuration affected.
4. Your name and affiliation if you'd like to be credited.

You can also use [GitHub's private vulnerability reporting](https://docs.github.com/en/code-security/security-advisories/guidance-on-reporting-and-writing-information-about-vulnerabilities/privately-reporting-a-security-vulnerability) on this repository.

We aim to:

- acknowledge your report within **2 business days**
- provide an initial assessment within **5 business days**
- agree on a coordinated disclosure timeline (typically 30–90 days depending on severity)
- credit you in the security advisory unless you prefer otherwise

## Response process

1. Triage and severity classification (CVSS v3.1).
2. Patch development on a private branch.
3. Coordinated disclosure date set with the reporter.
4. Patch release out of the normal release train (ADR-0009 §6 partial-publish recovery applies).
5. Public GitHub Security Advisory published with CVE.
6. SBOM and provenance for the fixed release available per ADR-0009.

## Hardening defaults

Per PRD §22:

- Prompt and model output content are not stored.
- Authenticated ingestion endpoints are required (API key minimum; mTLS optional).
- TLS 1.2+ everywhere; HSTS on the dashboard.
- RBAC for admin features.
- Audit logs for sensitive changes.
- Rate limiting and per-workspace quotas.
- Container images signed with cosign keyless OIDC.
- SLSA Build Level 3 provenance attached to releases.
- CycloneDX SBOM attached to every GitHub Release.

## Out of scope

- Issues in third-party dependencies that have an upstream fix — please report upstream and tell us so we can pin/upgrade.
- Self-XSS in the dashboard requiring an attacker to paste code into their own console.
- Denial of service via unauthenticated unlimited request volume below the documented rate limits — these are expected to be handled by the operator's edge.

Thank you again. Responsible disclosure helps everyone.
