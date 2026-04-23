//! Storage hardening tests added by foundation-backlog burndown.
//!
//! Covers the gaps called out in /memories/repo/foundation-backlog.md:
//! - heec_scores append-only triggers (symmetry with heec_events)
//! - heec_audit_log append-only triggers (added in migration 0002)
//! - Unique-with-COALESCE indexes for global (workspace_id IS NULL) rows
//! - current_version matches the embedded migration count
//! - Foreign keys are enforced (PRAGMA on every connection)
//! - CHECK constraints reject invalid heec_jobs.state values
//! - open_path round-trip through a filesystem path

use heeczer_storage::sqlite::{current_version, migrate, open, open_path, MIGRATOR};
use sqlx_core::query::query;
use sqlx_core::query_as::query_as;
use sqlx_sqlite::SqlitePool;
use tempfile::tempdir;

async fn fresh_pool() -> SqlitePool {
    let pool = open("sqlite::memory:").await.expect("open in-memory");
    migrate(&pool).await.expect("migrate");
    query("INSERT INTO heec_workspaces (workspace_id, display_name) VALUES ('ws_t', 'T')")
        .execute(&pool)
        .await
        .expect("seed workspace");
    pool
}

#[tokio::test]
async fn heec_scores_rejects_update_and_delete() {
    let pool = fresh_pool().await;
    query(
        "INSERT INTO heec_events (event_id, workspace_id, spec_version, framework_source, payload, received_at)
         VALUES ('evt-1', 'ws_t', '1.0', 'test', '{}', '2026-04-22T10:00:00Z')",
    )
    .execute(&pool)
    .await
    .unwrap();
    query(
        "INSERT INTO heec_scores
            (workspace_id, event_id, scoring_version, scoring_profile_id, profile_version,
             tier_id, tier_version, rates_version, result_json, final_minutes, final_fec,
             confidence, confidence_band)
         VALUES ('ws_t','evt-1','1.0','default','1.0','default','1.0','1.0','{}','5.0','0.10','0.85','high')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let upd = query("UPDATE heec_scores SET final_fec = '9.99' WHERE event_id = 'evt-1'")
        .execute(&pool)
        .await;
    assert!(upd.is_err(), "UPDATE on heec_scores must be rejected");

    let del = query("DELETE FROM heec_scores WHERE event_id = 'evt-1'")
        .execute(&pool)
        .await;
    assert!(del.is_err(), "DELETE on heec_scores must be rejected");
}

#[tokio::test]
async fn heec_audit_log_is_append_only() {
    let pool = fresh_pool().await;
    query(
        "INSERT INTO heec_audit_log (audit_id, workspace_id, actor, action)
         VALUES ('a1','ws_t','tester','noop')",
    )
    .execute(&pool)
    .await
    .unwrap();

    let upd = query("UPDATE heec_audit_log SET action = 'tampered' WHERE audit_id = 'a1'")
        .execute(&pool)
        .await;
    assert!(upd.is_err(), "UPDATE on heec_audit_log must be rejected");

    let del = query("DELETE FROM heec_audit_log WHERE audit_id = 'a1'")
        .execute(&pool)
        .await;
    assert!(del.is_err(), "DELETE on heec_audit_log must be rejected");
}

#[tokio::test]
async fn global_scoring_profile_rows_cannot_duplicate() {
    let pool = open("sqlite::memory:").await.unwrap();
    migrate(&pool).await.unwrap();
    let insert = |i: i32| {
        let pool = pool.clone();
        async move {
            query(
                "INSERT INTO heec_scoring_profiles
                    (scoring_profile_id, version, workspace_id, profile_json, effective_at)
                 VALUES ('default','1.0', NULL, ?, '2026-04-22T00:00:00Z')",
            )
            .bind(format!("{{\"n\":{i}}}"))
            .execute(&pool)
            .await
        }
    };
    insert(1).await.expect("first global row");
    let dup = insert(2).await;
    assert!(dup.is_err(), "second global row must fail unique index");
}

#[tokio::test]
async fn current_version_matches_embedded_migration_count() {
    let pool = open("sqlite::memory:").await.unwrap();
    migrate(&pool).await.unwrap();
    let v = current_version(&pool).await.unwrap().unwrap();
    let highest = MIGRATOR
        .migrations
        .iter()
        .map(|m| m.version)
        .max()
        .expect("at least one migration");
    assert_eq!(
        v, highest,
        "current_version must equal max embedded version"
    );
}

#[tokio::test]
async fn foreign_keys_are_enforced() {
    let pool = open("sqlite::memory:").await.unwrap();
    migrate(&pool).await.unwrap();
    // Insert an event for a workspace that doesn't exist — must be rejected.
    let res = query(
        "INSERT INTO heec_events (event_id, workspace_id, spec_version, framework_source, payload, received_at)
         VALUES ('evt-x', 'nope', '1.0', 'test', '{}', '2026-04-22T10:00:00Z')",
    )
    .execute(&pool)
    .await;
    assert!(res.is_err(), "FK to heec_workspaces must be enforced");
}

#[tokio::test]
async fn heec_jobs_check_constraint_rejects_unknown_state() {
    let pool = fresh_pool().await;
    let res =
        query("INSERT INTO heec_jobs (job_id, workspace_id, state) VALUES ('j1','ws_t','bogus')")
            .execute(&pool)
            .await;
    assert!(
        res.is_err(),
        "CHECK constraint on heec_jobs.state must reject unknown values"
    );
}

#[tokio::test]
async fn open_path_round_trips_to_disk() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("heec.sqlite3");
    {
        let pool = open_path(&path).await.unwrap();
        migrate(&pool).await.unwrap();
        query("INSERT INTO heec_workspaces (workspace_id, display_name) VALUES ('ws_d','D')")
            .execute(&pool)
            .await
            .unwrap();
    }
    // Reopen — data must persist.
    let pool = open_path(&path).await.unwrap();
    let row: (String,) =
        query_as("SELECT display_name FROM heec_workspaces WHERE workspace_id = 'ws_d'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row.0, "D");
}
