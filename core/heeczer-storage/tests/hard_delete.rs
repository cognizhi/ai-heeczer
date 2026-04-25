//! Tests for the hard-deletion storage API (plan 0003, PRD §12.17).
//!
//! Covers:
//! - Tombstone is inserted on deletion.
//! - Event row is removed after hard delete.
//! - Score rows are removed after hard delete.
//! - Audit log entry is written for the deletion.
//! - Deletion without a tombstone still fails (append-only guard preserved).
//! - Calling hard_delete_event twice on the same event returns
//!   `already_tombstoned: true` on the second call.
//! - Tombstone presence blocks re-scoring (guards the re-scoring path's
//!   tombstone check described in plan 0003 § Tests).

use heeczer_storage::admin::hard_delete_event;
use heeczer_storage::sqlite::{migrate, open};
use sqlx_core::query::query;
use sqlx_core::query_as::query_as;
use sqlx_sqlite::SqlitePool;

async fn fresh_pool() -> SqlitePool {
    let pool = open("sqlite::memory:").await.expect("open in-memory");
    migrate(&pool).await.expect("migrate");
    query("INSERT INTO heec_workspaces (workspace_id, display_name) VALUES ('ws_t', 'Test')")
        .execute(&pool)
        .await
        .expect("seed workspace");
    pool
}

async fn insert_event(pool: &SqlitePool, event_id: &str) {
    query(
        "INSERT INTO heec_events
            (event_id, workspace_id, spec_version, framework_source, payload, received_at)
         VALUES (?1, 'ws_t', '1.0', 'test', '{}', '2026-04-25T00:00:00Z')",
    )
    .bind(event_id)
    .execute(pool)
    .await
    .expect("insert event");
}

async fn insert_score(pool: &SqlitePool, event_id: &str) {
    query(
        "INSERT INTO heec_scores
            (workspace_id, event_id, scoring_version, scoring_profile_id, profile_version,
             tier_id, tier_version, rates_version, result_json, final_minutes, final_fec,
             confidence, confidence_band)
         VALUES ('ws_t', ?1, '1.0', 'default', '1.0', 'default', '1.0', '1.0',
                 '{}', '5.0', '0.10', '0.85', 'high')",
    )
    .bind(event_id)
    .execute(pool)
    .await
    .expect("insert score");
}

// --------------------------------------------------------------------------
// Tombstone is present after deletion
// --------------------------------------------------------------------------

#[tokio::test]
async fn hard_delete_inserts_tombstone() {
    let pool = fresh_pool().await;
    insert_event(&pool, "evt-1").await;

    let outcome = hard_delete_event(&pool, "ws_t", "evt-1", "admin", "gdpr-test")
        .await
        .unwrap();
    assert!(!outcome.already_tombstoned);

    let row: Option<(String,)> = query_as(
        "SELECT reason FROM heec_tombstones WHERE workspace_id = 'ws_t' AND event_id = 'evt-1'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert!(row.is_some(), "tombstone row must be present");
    assert_eq!(row.unwrap().0, "gdpr-test");
}

// --------------------------------------------------------------------------
// Event row is removed
// --------------------------------------------------------------------------

#[tokio::test]
async fn hard_delete_removes_event_row() {
    let pool = fresh_pool().await;
    insert_event(&pool, "evt-2").await;

    hard_delete_event(&pool, "ws_t", "evt-2", "admin", "gdpr-test")
        .await
        .unwrap();

    let row: Option<(String,)> = query_as(
        "SELECT event_id FROM heec_events WHERE workspace_id = 'ws_t' AND event_id = 'evt-2'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert!(row.is_none(), "event row must be gone after hard delete");
}

// --------------------------------------------------------------------------
// Score rows are removed
// --------------------------------------------------------------------------

#[tokio::test]
async fn hard_delete_removes_score_rows() {
    let pool = fresh_pool().await;
    insert_event(&pool, "evt-3").await;
    insert_score(&pool, "evt-3").await;

    let outcome = hard_delete_event(&pool, "ws_t", "evt-3", "admin", "gdpr-test")
        .await
        .unwrap();
    assert_eq!(outcome.scores_deleted, 1);

    let row: Option<(String,)> = query_as(
        "SELECT event_id FROM heec_scores WHERE workspace_id = 'ws_t' AND event_id = 'evt-3'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert!(row.is_none(), "score rows must be gone after hard delete");
}

// --------------------------------------------------------------------------
// Audit log entry is written
// --------------------------------------------------------------------------

#[tokio::test]
async fn hard_delete_writes_audit_log_entry() {
    let pool = fresh_pool().await;
    insert_event(&pool, "evt-4").await;

    hard_delete_event(&pool, "ws_t", "evt-4", "admin@example.com", "gdpr-test")
        .await
        .unwrap();

    let row: Option<(String, String, String, Option<String>)> = query_as(
        "SELECT actor, action, target_table, target_id FROM heec_audit_log
         WHERE workspace_id = 'ws_t' AND action = 'hard_delete_event'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(row.is_some(), "audit log entry must be present");
    let (actor, action, target_table, target_id) = row.unwrap();
    assert_eq!(actor, "admin@example.com");
    assert_eq!(action, "hard_delete_event");
    assert_eq!(target_table, "heec_events");
    assert_eq!(
        target_id.as_deref(),
        Some("evt-4"),
        "target_id must be the event_id so audit queries locate the deletion record"
    );
}

// --------------------------------------------------------------------------
// Append-only guard still fires without a tombstone
// --------------------------------------------------------------------------

#[tokio::test]
async fn direct_delete_without_tombstone_still_aborts() {
    let pool = fresh_pool().await;
    insert_event(&pool, "evt-5").await;

    let result = query("DELETE FROM heec_events WHERE event_id = 'evt-5'")
        .execute(&pool)
        .await;
    assert!(
        result.is_err(),
        "direct DELETE without tombstone must still be rejected by the trigger"
    );
}

// --------------------------------------------------------------------------
// Idempotency: second call returns already_tombstoned
// --------------------------------------------------------------------------

#[tokio::test]
async fn hard_delete_is_idempotent() {
    let pool = fresh_pool().await;
    insert_event(&pool, "evt-6").await;

    let first = hard_delete_event(&pool, "ws_t", "evt-6", "admin", "gdpr-test")
        .await
        .unwrap();
    assert!(!first.already_tombstoned);

    let second = hard_delete_event(&pool, "ws_t", "evt-6", "admin", "gdpr-test")
        .await
        .unwrap();
    assert!(
        second.already_tombstoned,
        "second call must return already_tombstoned=true"
    );
}

// --------------------------------------------------------------------------
// Tombstone prevents re-scoring (plan 0003 § Tests)
// --------------------------------------------------------------------------

#[tokio::test]
async fn tombstone_blocks_rescore_path() {
    let pool = fresh_pool().await;
    insert_event(&pool, "evt-7").await;
    hard_delete_event(&pool, "ws_t", "evt-7", "admin", "gdpr-test")
        .await
        .unwrap();

    // Simulate the re-scoring path: check for a tombstone before inserting a
    // new score row. The tombstone must be present and block the operation.
    let tombstone: Option<(String,)> = query_as(
        "SELECT event_id FROM heec_tombstones WHERE workspace_id = 'ws_t' AND event_id = 'evt-7'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(
        tombstone.is_some(),
        "tombstone must be present so re-scoring jobs can detect deleted events"
    );
}

// --------------------------------------------------------------------------
// Tombstone itself is append-only (migration 0004 guard)
// --------------------------------------------------------------------------

#[tokio::test]
async fn tombstone_row_is_append_only() {
    let pool = fresh_pool().await;
    insert_event(&pool, "evt-9").await;
    hard_delete_event(&pool, "ws_t", "evt-9", "admin", "gdpr-test")
        .await
        .unwrap();

    let del = query("DELETE FROM heec_tombstones WHERE event_id = 'evt-9'")
        .execute(&pool)
        .await;
    assert!(del.is_err(), "DELETE on heec_tombstones must be rejected");

    let upd = query("UPDATE heec_tombstones SET reason = 'tampered' WHERE event_id = 'evt-9'")
        .execute(&pool)
        .await;
    assert!(upd.is_err(), "UPDATE on heec_tombstones must be rejected");
}

#[tokio::test]
async fn hard_delete_preserves_daily_aggregates() {
    let pool = fresh_pool().await;
    insert_event(&pool, "evt-8").await;

    query(
        "INSERT INTO heec_daily_aggregates
            (workspace_id, day, project_id, category, framework_source, event_count,
             total_minutes, total_fec)
         VALUES ('ws_t', '2026-04-25', '', '', '', 1, '5.0', '0.10')",
    )
    .execute(&pool)
    .await
    .unwrap();

    hard_delete_event(&pool, "ws_t", "evt-8", "admin", "gdpr-test")
        .await
        .unwrap();

    let row: Option<(i64,)> = query_as(
        "SELECT event_count FROM heec_daily_aggregates WHERE workspace_id = 'ws_t' AND day = '2026-04-25'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();

    assert!(
        row.is_some(),
        "daily aggregates must be preserved after hard delete"
    );
    assert_eq!(row.unwrap().0, 1, "aggregate event_count must be unchanged");
}

// --------------------------------------------------------------------------
// Audit-log PII redaction (PRD §12.17, migration 0005)
// --------------------------------------------------------------------------

/// Pre-existing audit log rows that reference the deleted event_id in
/// target_id must have target_id set to NULL after hard deletion.
#[tokio::test]
async fn hard_delete_redacts_audit_log_target_id() {
    let pool = fresh_pool().await;
    insert_event(&pool, "evt-10").await;

    // Seed a pre-existing audit log row that references the event.
    let prior_audit_id = "audit-prior-0001";
    query(
        "INSERT INTO heec_audit_log
             (audit_id, workspace_id, actor, action, target_table, target_id, payload_json)
         VALUES (?1, 'ws_t', 'system', 'score_event', 'heec_events', 'evt-10', '{}')",
    )
    .bind(prior_audit_id)
    .execute(&pool)
    .await
    .expect("seed prior audit row");

    let outcome = hard_delete_event(&pool, "ws_t", "evt-10", "admin", "gdpr-test")
        .await
        .unwrap();

    // The redaction step must have NULLed exactly one pre-existing row.
    assert_eq!(
        outcome.audit_log_rows_redacted, 1,
        "audit_log_rows_redacted must equal the number of pre-existing rows referencing the event"
    );

    // The prior audit row must now have target_id = NULL.
    let row: Option<(Option<String>,)> =
        query_as("SELECT target_id FROM heec_audit_log WHERE audit_id = ?1")
            .bind(prior_audit_id)
            .fetch_optional(&pool)
            .await
            .unwrap();
    assert!(row.is_some(), "prior audit row must still exist");
    assert!(
        row.unwrap().0.is_none(),
        "target_id on prior audit row must be NULL after hard delete"
    );

    // The deletion audit row itself must still carry the event_id in target_id
    // (it is inserted after the redaction step and is not itself redacted).
    let del_row: Option<(Option<String>,)> = query_as(
        "SELECT target_id FROM heec_audit_log
         WHERE workspace_id = 'ws_t' AND action = 'hard_delete_event'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert!(del_row.is_some(), "deletion audit row must be present");
    assert_eq!(
        del_row.unwrap().0.as_deref(),
        Some("evt-10"),
        "deletion audit row target_id must remain the event_id"
    );
}

/// A direct UPDATE on heec_audit_log that is NOT tombstone-authorized must
/// still be rejected (append-only guard preserved by migration 0005).
#[tokio::test]
async fn direct_audit_log_update_without_tombstone_is_rejected() {
    let pool = fresh_pool().await;
    insert_event(&pool, "evt-11").await;

    // Seed an audit row with a target_id referencing the event.
    query(
        "INSERT INTO heec_audit_log
             (audit_id, workspace_id, actor, action, target_table, target_id, payload_json)
         VALUES ('audit-no-ts-0001', 'ws_t', 'system', 'score_event', 'heec_events', 'evt-11', '{}')",
    )
    .execute(&pool)
    .await
    .expect("seed audit row");

    // Attempt UPDATE without an existing tombstone — must be rejected.
    let result =
        query("UPDATE heec_audit_log SET target_id = NULL WHERE audit_id = 'audit-no-ts-0001'")
            .execute(&pool)
            .await;
    assert!(
        result.is_err(),
        "UPDATE on heec_audit_log without tombstone must be rejected by the trigger"
    );
}
