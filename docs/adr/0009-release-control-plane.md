# ADR-0009: Release Control Plane

- **Status:** Accepted
- **Date:** 2026-04-22
- **Related:** PRD §27, §12.10–§12.11

## Context
The repo must publish synchronized releases to npm, PyPI, crates.io, Maven Central, Go module tags, container registries, and GitHub Releases (PRD §27.4). All artifacts share one product version.

## Decision
- **`release-please` in manifest mode** is the release control plane (per PRD §27.1).
- One release PR per cycle aggregates conventional-commit-derived changelogs across all components and computes the next semver.
- Merging the release PR triggers a release workflow that:
  1. Tags the repo (`v<version>`).
  2. Builds and signs all artifacts (cosign keyless).
  3. Generates SBOMs (CycloneDX) and SLSA provenance.
  4. Publishes to each registry in parallel where independent; serialized only where ordering matters (e.g., Maven Central staging close).
  5. Updates the **release manifest** (`.github/release-manifest.json`) with intent, digests, and per-target publish status.
  6. Marks the release complete only when all required targets succeed (PRD §27.5).
- **Trusted publishing** is preferred wherever the registry supports it (PyPI OIDC, npm provenance via OIDC, GitHub Releases). Maven Central uses a Sonatype token in GitHub Secrets until OIDC trusted publishing is broadly available.
- Failed targets enter a partial-publish state. Resume is operator-driven via a `release-resume` workflow_dispatch and never mints a new version.

## Alternatives Considered
- **`semantic-release`** — JS-centric, weaker multi-language story.
- **Custom shell pipeline** — maximum flexibility, maximum maintenance.

## Consequences
- Positive: a single declarative release manifest; one source of truth for what's in a release.
- Negative: release-please's release notes need careful templating to read well across components.
- Follow-ups: document partial-publish recovery runbook.

## References
- PRD §27 CI/CD and Release Requirements
