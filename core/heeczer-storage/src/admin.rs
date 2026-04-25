//! Admin-only storage operations (PRD §12.17, plan 0003).
//!
//! Provides hard-deletion for GDPR/CCPA-style subject deletion. Callers are
//! responsible for authorization (admin-only). All operations run in a single
//! transaction and write an audit log entry.
//!
//! # How tombstone-authorized deletion works
//!
//! Migration 0004 replaces the blanket `heec_events_no_delete` and
//! `heec_scores_no_delete` triggers with tombstone-aware equivalents. The new
//! triggers abort the delete *unless* a `heec_tombstones` row already exists
//! for `(workspace_id, event_id)`. [`hard_delete_event`] exploits this by
//! inserting the tombstone row first, within the same transaction, before
//! issuing the `DELETE` statements.
//!
//! # Known limitation — audit-log PII redaction
//!
//! PRD §12.17 requires removal of "audit-log identifiers." Pre-existing
//! `heec_audit_log` rows that reference the deleted `event_id` in their
//! `target_id` column are **not redacted** by this function. Redacting them
//! requires loosening the `heec_audit_log_no_update` trigger, which has its
//! own security implications. This is tracked as a follow-up item in plan 0003
//! and requires a separate migration with a dedicated security review.
//! Operators completing a GDPR/CCPA erasure request must account for this gap.

use crate::error::Result;
use sqlx_core::query::query;
use sqlx_core::query_as::query_as;
use sqlx_postgres::PgPool;
use sqlx_sqlite::SqlitePool;
use uuid::Uuid;

/// Outcome of a [`hard_delete_event`] or [`hard_delete_event_pg`] call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HardDeleteOutcome {
    /// Number of `heec_scores` rows removed.
    pub scores_deleted: u64,
    /// `true` if the event was already tombstoned before this call.
    /// When `true` the function returns immediately; no second deletion is
    /// attempted so the call is safe to retry.
    pub already_tombstoned: bool,
}

/// Hard-delete an event and all associated scores for a workspace (SQLite).
///
/// See [`hard_delete_event_pg`] for the PostgreSQL equivalent.
///
/// # Parameters
///
/// - `workspace_id` — tenant workspace that owns the event.
/// - `event_id` — identifier of the event to delete.
/// - `actor` — identity of the admin performing the deletion, written verbatim
///   to the audit log (e.g. `"admin@example.com"` or a service-account name).
/// - `reason` — human-readable deletion reason. Use a structured reference
///   where possible (e.g. `"GDPR erasure SR-2026-001"`).
///
/// # Invariants upheld
///
/// - **Atomicity** — tombstone insert, score deletion, event deletion, and
///   audit-log write all occur in a single transaction.
/// - **Idempotency** — a second call with the same `(workspace_id, event_id)`
///   returns `already_tombstoned: true` without modifying any data.
/// - **Aggregates preserved** — `heec_daily_aggregates` rows are never
///   touched; anonymized rollups remain intact per PRD §12.17.
/// - **Append-only audit trail** — a new `heec_audit_log` row is inserted for
///   the deletion event; existing audit rows are not modified.
///
/// # Authorization
///
/// The caller **must** verify admin authority before invoking this function.
/// This is a storage primitive; RBAC enforcement belongs at the service layer.
///
/// # Audit-log PII redaction caveat
///
/// Pre-existing `heec_audit_log` rows that reference `event_id` in `target_id`
/// are **not redacted**. See module-level documentation for details.
pub async fn hard_delete_event(
    pool: &SqlitePool,
    workspace_id: &str,
    event_id: &str,
    actor: &str,
    reason: &str,
) -> Result<HardDeleteOutcome> {
    let mut tx = pool.begin().await?;

    // 1. Idempotency check — return early if already tombstoned.
    let existing: Option<(String,)> =
        query_as("SELECT event_id FROM heec_tombstones WHERE workspace_id = ?1 AND event_id = ?2")
            .bind(workspace_id)
            .bind(event_id)
            .fetch_optional(&mut *tx)
            .await?;

    if existing.is_some() {
        return Ok(HardDeleteOutcome {
            scores_deleted: 0,
            already_tombstoned: true,
        });
    }

    // 2. Insert tombstone — this satisfies the migration-0004 trigger guards
    //    on heec_events and heec_scores for the remainder of this transaction.
    query("INSERT INTO heec_tombstones (workspace_id, event_id, reason) VALUES (?1, ?2, ?3)")
        .bind(workspace_id)
        .bind(event_id)
        .bind(reason)
        .execute(&mut *tx)
        .await?;

    // 3. Delete scores (tombstone now satisfies the append-only guard).
    let scores_result = query("DELETE FROM heec_scores WHERE workspace_id = ?1 AND event_id = ?2")
        .bind(workspace_id)
        .bind(event_id)
        .execute(&mut *tx)
        .await?;
    let scores_deleted = scores_result.rows_affected();

    // 4. Delete the event row (tombstone now satisfies the append-only guard).
    query("DELETE FROM heec_events WHERE workspace_id = ?1 AND event_id = ?2")
        .bind(workspace_id)
        .bind(event_id)
        .execute(&mut *tx)
        .await?;

    // 5. Append audit log entry for this deletion.
    //    target_id is set to event_id so standard audit queries for this event
    //    locate the deletion record.
    let audit_id = Uuid::new_v4().to_string();
    let payload_json = serde_json::json!({
        "reason": reason,
        "scores_deleted": scores_deleted,
    })
    .to_string();
    query(
        "INSERT INTO heec_audit_log
             (audit_id, workspace_id, actor, action, target_table, target_id, payload_json)
         VALUES (?1, ?2, ?3, 'hard_delete_event', 'heec_events', ?4, ?5)",
    )
    .bind(&audit_id)
    .bind(workspace_id)
    .bind(actor)
    .bind(event_id)
    .bind(&payload_json)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HardDeleteOutcome {
        scores_deleted,
        already_tombstoned: false,
    })
}

/// Hard-delete an event and all associated scores for a workspace (PostgreSQL).
///
/// Semantically identical to [`hard_delete_event`]; see that function's
/// documentation for full invariants, parameter descriptions, and the
/// audit-log PII redaction caveat.
pub async fn hard_delete_event_pg(
    pool: &PgPool,
    workspace_id: &str,
    event_id: &str,
    actor: &str,
    reason: &str,
) -> Result<HardDeleteOutcome> {
    let mut tx = pool.begin().await?;

    // 1. Idempotency check.
    let existing: Option<(String,)> =
        query_as("SELECT event_id FROM heec_tombstones WHERE workspace_id = $1 AND event_id = $2")
            .bind(workspace_id)
            .bind(event_id)
            .fetch_optional(&mut *tx)
            .await?;

    if existing.is_some() {
        return Ok(HardDeleteOutcome {
            scores_deleted: 0,
            already_tombstoned: true,
        });
    }

    // 2. Insert tombstone.
    query("INSERT INTO heec_tombstones (workspace_id, event_id, reason) VALUES ($1, $2, $3)")
        .bind(workspace_id)
        .bind(event_id)
        .bind(reason)
        .execute(&mut *tx)
        .await?;

    // 3. Delete scores.
    let scores_result = query("DELETE FROM heec_scores WHERE workspace_id = $1 AND event_id = $2")
        .bind(workspace_id)
        .bind(event_id)
        .execute(&mut *tx)
        .await?;
    let scores_deleted = scores_result.rows_affected();

    // 4. Delete event.
    query("DELETE FROM heec_events WHERE workspace_id = $1 AND event_id = $2")
        .bind(workspace_id)
        .bind(event_id)
        .execute(&mut *tx)
        .await?;

    // 5. Append audit log entry.
    let audit_id = Uuid::new_v4().to_string();
    let payload_json = serde_json::json!({
        "reason": reason,
        "scores_deleted": scores_deleted,
    })
    .to_string();
    query(
        "INSERT INTO heec_audit_log
             (audit_id, workspace_id, actor, action, target_table, target_id, payload_json)
         VALUES ($1, $2, $3, 'hard_delete_event', 'heec_events', $4, $5)",
    )
    .bind(&audit_id)
    .bind(workspace_id)
    .bind(actor)
    .bind(event_id)
    .bind(&payload_json)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HardDeleteOutcome {
        scores_deleted,
        already_tombstoned: false,
    })
}
