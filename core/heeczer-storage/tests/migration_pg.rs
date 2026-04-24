//! PostgreSQL migration file presence tests.
//!
//! Full integration tests (fresh install + incremental upgrade on a real
//! PostgreSQL instance) run in `.github/workflows/migration.yml` with a
//! `postgres:16-alpine` service container.
//!
//! These unit-level tests verify that the migration files are present and
//! well-formed without requiring a live database.

use std::path::PathBuf;

fn migrations_pg_dir() -> PathBuf {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest.join("migrations-pg")
}

#[test]
fn migrations_pg_directory_exists() {
    let dir = migrations_pg_dir();
    assert!(
        dir.exists(),
        "migrations-pg/ directory not found at {}",
        dir.display()
    );
}

#[test]
fn migrations_pg_contains_at_least_one_sql_file() {
    let dir = migrations_pg_dir();
    let sql_files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("read_dir {}: {e}", dir.display()))
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("sql"))
        .collect();
    assert!(
        !sql_files.is_empty(),
        "migrations-pg/ contains no .sql files"
    );
}

#[test]
fn migrations_pg_files_are_well_formed_sql() {
    let dir = migrations_pg_dir();
    for entry in std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("sql"))
    {
        let content = std::fs::read_to_string(entry.path()).unwrap();
        // Basic sanity: non-empty and contains at least one SQL keyword.
        assert!(
            !content.trim().is_empty(),
            "{} is empty",
            entry.path().display()
        );
        let upper = content.to_uppercase();
        let has_sql = upper.contains("CREATE")
            || upper.contains("ALTER")
            || upper.contains("INSERT")
            || upper.contains("DROP");
        assert!(
            has_sql,
            "{} contains no recognisable SQL DDL",
            entry.path().display()
        );
    }
}

#[test]
fn migrations_pg_has_matching_sqlite_migration() {
    // Every PG migration should have a corresponding SQLite migration.
    let pg_dir = migrations_pg_dir();
    let sqlite_dir = pg_dir.parent().unwrap().join("migrations");

    for entry in std::fs::read_dir(&pg_dir)
        .unwrap()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("sql"))
    {
        let filename = entry.file_name();
        let sqlite_file = sqlite_dir.join(&filename);
        assert!(
            sqlite_file.exists(),
            "PG migration {} has no matching SQLite migration at {}",
            entry.path().display(),
            sqlite_file.display()
        );
    }
}
