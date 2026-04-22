//! SQLite backend. Uses `sqlx::SqlitePool`; migrations are embedded.

use crate::error::Result;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use std::path::Path;

/// Embedded migrations (`core/heeczer-storage/migrations/`).
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Open a SQLite pool. Pass `:memory:` for an ephemeral database.
pub async fn open(url: &str) -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
        .max_connections(8)
        .connect(url)
        .await?;
    Ok(pool)
}

/// Open or create a SQLite database at the given filesystem path.
pub async fn open_path(path: &Path) -> Result<SqlitePool> {
    let url = format!("sqlite://{}?mode=rwc", path.display());
    open(&url).await
}

/// Run all pending migrations.
pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

/// Return the current migration version (highest applied), or `None` if empty.
pub async fn current_version(pool: &SqlitePool) -> Result<Option<i64>> {
    let row: Option<(i64,)> =
        sqlx::query_as("SELECT version FROM aih_schema_migrations ORDER BY version DESC LIMIT 1")
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(v,)| v))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    async fn aih_events_is_append_only_via_trigger() {
        let pool = open("sqlite::memory:").await.unwrap();
        migrate(&pool).await.unwrap();

        sqlx::query(
            "INSERT INTO aih_workspaces (workspace_id, display_name) VALUES ('ws_test', 'Test')",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO aih_events (event_id, workspace_id, spec_version, framework_source, payload, received_at)
             VALUES ('evt-1', 'ws_test', '1.0', 'test', '{}', '2026-04-22T10:00:00Z')",
        )
        .execute(&pool)
        .await
        .unwrap();

        let update = sqlx::query(
            "UPDATE aih_events SET framework_source = 'tampered' WHERE event_id = 'evt-1'",
        )
        .execute(&pool)
        .await;
        assert!(update.is_err(), "UPDATE on aih_events must be rejected");

        let delete = sqlx::query("DELETE FROM aih_events WHERE event_id = 'evt-1'")
            .execute(&pool)
            .await;
        assert!(delete.is_err(), "DELETE on aih_events must be rejected");
    }
}
