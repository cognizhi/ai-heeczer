//! Storage layer (plan 0003). SQLite is the MVP backend; PostgreSQL parity arrives in Phase 2.
//!
//! Tables follow PRD §20. Migrations live under `migrations/` (SQLite) and
//! `migrations-pg/` (PostgreSQL) and are embedded at compile time via
//! [`sqlx_macros::migrate!`]. Append-only invariants for `aih_events` and `aih_scores`
//! are enforced by SQL triggers in the migration scripts; typed Rust insert
//! helpers will land alongside the ingestion service in plan 0004.

#![cfg_attr(not(test), warn(missing_docs))]

extern crate sqlx_core as sqlx;

pub mod error;
pub mod pg;
pub mod sqlite;

pub use error::{Error, Result};
