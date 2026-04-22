//! End-to-end CLI tests using the binary under test. Complements `make cli-smoke`.

use assert_cmd::Command;
use predicates::prelude::*;

fn aih() -> Command {
    Command::cargo_bin("aih").expect("aih binary built")
}

const CANONICAL: &str = "../schema/fixtures/events/valid/01-prd-canonical.json";
const INVALID_UUID: &str = "../schema/fixtures/events/invalid/05-invalid-uuid.json";

#[test]
fn version_prints_scoring_and_spec_versions() {
    aih()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("scoring_version=1.0.0"))
        .stdout(predicate::str::contains("spec_version=1.0"));
}

#[test]
fn schema_validate_accepts_canonical_fixture() {
    aih()
        .args(["schema", "validate", CANONICAL])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));
}

#[test]
fn schema_validate_rejects_invalid_fixture() {
    aih()
        .args(["schema", "validate", INVALID_UUID])
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema validation failed"));
}

#[test]
fn score_emits_canonical_pretty_output() {
    aih()
        .args(["score", CANONICAL, "--format", "pretty"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "\"final_estimated_minutes\": \"75.98\"",
        ))
        .stdout(predicate::str::contains(
            "\"financial_equivalent_cost\": \"94.97\"",
        ))
        .stdout(predicate::str::contains("\"confidence_score\": \"0.9000\""));
}

#[test]
fn score_invalid_event_exits_nonzero() {
    aih().args(["score", INVALID_UUID]).assert().failure();
}

#[test]
fn migrate_status_requires_database_url() {
    aih().args(["migrate", "status"]).assert().failure();
}

#[test]
fn migrate_up_then_status_reports_latest_version() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("aih.sqlite");
    let url = format!("sqlite://{}?mode=rwc", db.display());

    // Resolve the highest embedded migration version dynamically so the
    // assertion does not need to be touched every time we add one.
    let latest = heeczer_storage::sqlite::MIGRATOR
        .migrations
        .iter()
        .map(|m| m.version)
        .max()
        .expect("at least one migration");

    aih()
        .args(["migrate", "up", "--database-url", &url])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("migrated to {latest}")));

    aih()
        .args(["migrate", "status", "--database-url", &url])
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Some({latest})")));
}
