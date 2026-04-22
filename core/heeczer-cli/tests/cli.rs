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

#[test]
fn fixtures_list_includes_canonical_fixture() {
    aih()
        .args(["fixtures", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "events/valid/01-prd-canonical.json",
        ));
}

#[test]
fn fixtures_show_prints_embedded_body() {
    aih()
        .args(["fixtures", "show", "events/valid/01-prd-canonical.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"event_id\""));
}

#[test]
fn fixtures_show_unknown_name_fails() {
    aih()
        .args(["fixtures", "show", "events/valid/does-not-exist.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("fixture not found"));
}

#[test]
fn score_detail_emits_explainability_block() {
    aih()
        .args(["score", CANONICAL, "--detail"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Explainability trace"))
        .stdout(predicate::str::contains("BCU breakdown"))
        .stdout(predicate::str::contains("baseline_minutes"))
        .stdout(predicate::str::contains("confidence"));
}

#[test]
fn validate_profile_accepts_default_profile() {
    aih()
        .args(["validate", "profile", "../schema/profiles/default.v1.json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok"));
}

#[test]
fn validate_profile_rejects_event_payload() {
    aih()
        .args(["validate", "profile", CANONICAL])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "scoring profile schema validation failed",
        ));
}

#[test]
fn validate_tier_surface_is_reserved_until_schema_lands() {
    aih()
        .args(["validate", "tier", "../schema/tiers/default.v1.json"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("schema not yet shipped"));
}

#[test]
fn bench_runs_and_reports_percentiles() {
    aih()
        .args(["bench", "--fixture", CANONICAL, "--iter", "32"])
        .assert()
        .success()
        .stdout(predicate::str::contains("score() iter=32"))
        .stdout(predicate::str::contains("p50="))
        .stdout(predicate::str::contains("p95="))
        .stdout(predicate::str::contains("p99="));
}

#[test]
fn bench_zero_iter_rejected() {
    aih()
        .args(["bench", "--fixture", CANONICAL, "--iter", "0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--iter must be at least 1"));
}

#[test]
fn bench_p95_budget_breach_exits_nonzero() {
    aih()
        .args([
            "bench",
            "--fixture",
            CANONICAL,
            "--iter",
            "16",
            "--budget-ms",
            "0.0000001",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("exceeds budget"));
}

#[test]
fn replay_missing_event_id_errors() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("aih.sqlite");
    let url = format!("sqlite://{}?mode=rwc", db.display());

    aih()
        .args(["migrate", "up", "--database-url", &url])
        .assert()
        .success();

    aih()
        .args([
            "replay",
            "--database-url",
            &url,
            "--workspace",
            "default",
            "--event-id",
            "does-not-exist",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no aih_events row"));
}

#[test]
fn replay_persisted_event_emits_score_and_no_prior_row_marker() {
    let dir = tempfile::tempdir().expect("tempdir");
    let db = dir.path().join("aih.sqlite");
    let url = format!("sqlite://{}?mode=rwc", db.display());

    aih()
        .args(["migrate", "up", "--database-url", &url])
        .assert()
        .success();

    // Seed workspace + event using the canonical fixture as the persisted payload.
    let payload = std::fs::read_to_string(CANONICAL).expect("canonical fixture");
    let event_id = "evt-replay-1";
    let workspace = "default";
    seed_event(&url, workspace, event_id, &payload);

    aih()
        .args([
            "replay",
            "--database-url",
            &url,
            "--workspace",
            workspace,
            "--event-id",
            event_id,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"final_estimated_minutes\""))
        .stdout(predicate::str::contains("no prior score row"));
}

fn seed_event(url: &str, workspace: &str, event_id: &str, payload: &str) {
    use sqlx::Executor;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let pool = heeczer_storage::sqlite::open(url).await.unwrap();
        pool.execute(
            sqlx::query("INSERT INTO aih_workspaces (workspace_id, display_name) VALUES (?1, ?1)")
                .bind(workspace),
        )
        .await
        .unwrap();
        pool.execute(
            sqlx::query(
                "INSERT INTO aih_events
                 (event_id, workspace_id, spec_version, framework_source, payload, received_at)
                 VALUES (?1, ?2, '1.0', 'test', ?3, '2026-04-23T10:00:00Z')",
            )
            .bind(event_id)
            .bind(workspace)
            .bind(payload),
        )
        .await
        .unwrap();
    });
}
