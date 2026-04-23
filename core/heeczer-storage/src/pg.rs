//! PostgreSQL backend. Uses `sqlx_postgres::PgPool`; migrations are embedded from
//! `migrations-pg/` which contains PostgreSQL-dialect SQL (PL/pgSQL triggers,
//! `NOW()` defaults, etc.). See ADR-0004 for the dialect-parity strategy.

use crate::error::Result;
use sqlx_core::connection::ConnectOptions;
use sqlx_core::migrate::Migrator;
use sqlx_core::query_as::query_as;
use sqlx_postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use std::str::FromStr;

/// Embedded PostgreSQL migrations (`core/heeczer-storage/migrations-pg/`).
pub static MIGRATOR: Migrator = sqlx_macros::migrate!("./migrations-pg");

/// Open a PostgreSQL pool.
///
/// `url` must be a valid `postgres://` or `postgresql://` DSN.
///
/// # Security
/// Always use `sqlx_postgres::PgConnectOptions::from_str` to parse the DSN rather than
/// constructing it by string interpolation; this prevents host/port injection
/// via URL special characters (CWE-88).
pub async fn open(url: &str) -> Result<PgPool> {
    let opts = PgConnectOptions::from_str(url)?.disable_statement_logging();
    let pool = PgPoolOptions::new()
        .max_connections(16)
        .connect_with(opts)
        .await?;
    Ok(pool)
}

/// Run all pending migrations.
pub async fn migrate(pool: &PgPool) -> Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

/// Return the current migration version (highest applied), or `None` if empty.
pub async fn current_version(pool: &PgPool) -> Result<Option<i64>> {
    let row: Option<(i64,)> =
        query_as("SELECT version FROM aih_schema_migrations ORDER BY version DESC LIMIT 1")
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(v,)| v))
}

#[cfg(test)]
mod tests {
    // Integration tests against a real PostgreSQL instance are run in CI via
    // `.github/workflows/integration.yml` with a `services: postgres:` container.
    // Unit-level compile checks live here; skip at runtime if DATABASE_URL is absent.

    use super::MIGRATOR;

    #[test]
    fn migrator_is_non_empty() {
        assert!(
            !MIGRATOR.migrations.is_empty(),
            "pg migrator must have at least one migration"
        );
    }
}
