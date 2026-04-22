//! Public version constants. Any change here MUST ship with a fixture diff (PRD §14.7).

/// Canonical event schema version. Mirrors `event.v1.json` `spec_version`.
pub const SPEC_VERSION: &str = "1.0";

/// Scoring engine version. Bumping this requires:
/// 1. updating every golden fixture under `core/schema/fixtures/scoring/`,
/// 2. an ADR amendment (ADR-0003),
/// 3. a release note flagged as a behavior change.
pub const SCORING_VERSION: &str = "1.0.0";
