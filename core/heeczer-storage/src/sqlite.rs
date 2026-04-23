//! SQLite backend. Uses `sqlx_sqlite::SqlitePool`; migrations are embedded.

use crate::error::Result;
use sqlx_core::connection::ConnectOptions;
use sqlx_core::executor::Executor;
use sqlx_core::migrate::Migrator;
use sqlx_core::query_as::query_as;
use sqlx_sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

/// Embedded migrations (`core/heeczer-storage/migrations/`).
pub static MIGRATOR: Migrator = sqlx_macros::migrate!("./migrations");

/// Open a SQLite pool. Pass `sqlite::memory:` for an ephemeral database.
///
/// `:memory:` databases are private per-connection in SQLite, so we cap the
/// pool to a single connection to avoid silent state fragmentation. We also
/// enforce `PRAGMA foreign_keys = ON` on every acquired connection because the
/// pragma is per-connection and resets on reconnect.
pub async fn open(url: &str) -> Result<SqlitePool> {
    let opts = SqliteConnectOptions::from_str(url)?.disable_statement_logging();
    let is_memory = url.contains(":memory:");
    let pool = SqlitePoolOptions::new()
        .max_connections(if is_memory { 1 } else { 8 })
        .after_connect(|conn, _| {
            Box::pin(async move {
                conn.execute("PRAGMA foreign_keys = ON;").await?;
                Ok(())
            })
        })
        .connect_with(opts)
        .await?;
    Ok(pool)
}

/// Open or create a SQLite database at the given filesystem path. Avoids
/// string DSN construction so that paths containing `?`, `#`, or `&` cannot be
/// reinterpreted as URL query/fragment by sqlx (CWE-88 hardening).
pub async fn open_path(path: &Path) -> Result<SqlitePool> {
    let opts = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .disable_statement_logging();
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .after_connect(|conn, _| {
            Box::pin(async move {
                conn.execute("PRAGMA foreign_keys = ON;").await?;
                Ok(())
            })
        })
        .connect_with(opts)
        .await?;
    Ok(pool)
}

/// Run all pending migrations.
pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

/// Return the current migration version (highest applied), or `None` if empty.
pub async fn current_version(pool: &SqlitePool) -> Result<Option<i64>> {
    let row: Option<(i64,)> =
        query_as("SELECT version FROM heec_schema_migrations ORDER BY version DESC LIMIT 1")
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(v,)| v))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx_core::query::query;

    #[tokio::test]
    async fn fresh_in_memory_database_runs_migrations() {
        let pool = open("sqlite::memory:").await.unwrap();
        migrate(&pool).await.unwrap();
        let v = current_version(&pool).await.unwrap();
        assert!(v.is_some(), "migrations did not produce a version row");
    }

    #[tokio::test]
    async fn migrations_are_idempotent() {
        let pool = open("sqlite::memory:").await.unwrap();
        migrate(&pool).await.unwrap();
        migrate(&pool).await.unwrap();
    }

    #[tokio::test]
    async fn heec_events_is_append_only_via_trigger() {
        let pool = open("sqlite::memory:").await.unwrap();
        migrate(&pool).await.unwrap();

        query(
            "INSERT INTO heec_workspaces (workspace_id, display_name) VALUES ('ws_test', 'Test')",
        )
        .execute(&pool)
        .await
        .unwrap();

        query(
            "INSERT INTO heec_events (event_id, workspace_id, spec_version, framework_source, payload, received_at)
             VALUES ('evt-1', 'ws_test', '1.0', 'test', '{}', '2026-04-22T10:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let update =
            query("UPDATE heec_events SET framework_source = 'tampered' WHERE event_id = 'evt-1'")
                .execute(&pool)
                .await;
        assert!(update.is_err(), "UPDATE on heec_events must be rejected");

        let delete = query("DELETE FROM heec_events WHERE event_id = 'evt-1'")
            .execute(&pool)
            .await;
        assert!(delete.is_err(), "DELETE on heec_events must be rejected");
    }
}
