//! Queue abstraction and PostgreSQL `SKIP LOCKED` implementation (ADR-0006).

use async_trait::async_trait;
use sqlx_core::query::query;
use sqlx_core::query_as::query_as;
use sqlx_postgres::PgPool;
use uuid::Uuid;

use crate::error::{ApiError, ApiResult};

/// Queue row claimed by a worker.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobRecord {
    pub job_id: String,
    pub workspace_id: String,
    pub event_id: Option<String>,
    pub state: String,
    pub attempts: i64,
    pub last_error: Option<String>,
}

/// Queue visibility metrics used by `/metrics` collectors and dashboards.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QueueStats {
    pub pending: i64,
    pub running: i64,
    pub failed: i64,
    pub dead_letter: i64,
    pub retries: i64,
}

#[async_trait]
pub trait JobQueue: Send + Sync {
    async fn enqueue(&self, workspace_id: &str, event_id: Option<&str>) -> ApiResult<String>;
    async fn claim_next(&self) -> ApiResult<Option<JobRecord>>;
    async fn complete(&self, job_id: &str) -> ApiResult<()>;
    async fn fail(&self, job_id: &str, error: &str, retry_after_seconds: i64) -> ApiResult<()>;
    async fn stats(&self) -> ApiResult<QueueStats>;
}

/// Default image-mode queue backend: PostgreSQL row queue with `SKIP LOCKED`.
#[derive(Clone)]
pub struct PostgresJobQueue {
    pool: PgPool,
    max_attempts: i64,
}

impl PostgresJobQueue {
    pub fn new(pool: PgPool, max_attempts: i64) -> Self {
        Self { pool, max_attempts }
    }
}

#[async_trait]
impl JobQueue for PostgresJobQueue {
    async fn enqueue(&self, workspace_id: &str, event_id: Option<&str>) -> ApiResult<String> {
        let job_id = Uuid::new_v4().to_string();
        query(
            "INSERT INTO heec_jobs (job_id, workspace_id, event_id, state) \
             VALUES ($1, $2, $3, 'pending') \
             ON CONFLICT (job_id) DO NOTHING",
        )
        .bind(&job_id)
        .bind(workspace_id)
        .bind(event_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;
        Ok(job_id)
    }

    async fn claim_next(&self) -> ApiResult<Option<JobRecord>> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;

        let row: Option<(String, String, Option<String>, String, i64, Option<String>)> = query_as(
            "SELECT job_id, workspace_id, event_id, state, attempts, last_error \
             FROM heec_jobs \
             WHERE state IN ('pending','failed') \
               AND available_at <= to_char(clock_timestamp() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') \
             ORDER BY available_at ASC, enqueued_at ASC \
             LIMIT 1 \
             FOR UPDATE SKIP LOCKED",
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;

        let Some(row) = row else {
            tx.commit()
                .await
                .map_err(|e| ApiError::Storage(e.to_string()))?;
            return Ok(None);
        };

        query(
            "UPDATE heec_jobs \
             SET state = 'running', attempts = attempts + 1, last_error = NULL \
             WHERE job_id = $1",
        )
        .bind(&row.0)
        .execute(&mut *tx)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;

        Ok(Some(JobRecord {
            job_id: row.0,
            workspace_id: row.1,
            event_id: row.2,
            state: "running".into(),
            attempts: row.4 + 1,
            last_error: None,
        }))
    }

    async fn complete(&self, job_id: &str) -> ApiResult<()> {
        query(
            "UPDATE heec_jobs \
             SET state = 'succeeded', finished_at = to_char(clock_timestamp() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') \
             WHERE job_id = $1",
        )
        .bind(job_id)
        .execute(&self.pool)
        .await
        .map_err(|e| ApiError::Storage(e.to_string()))?;
        Ok(())
    }

    async fn fail(&self, job_id: &str, error: &str, retry_after_seconds: i64) -> ApiResult<()> {
        let attempts: Option<(i64,)> = query_as("SELECT attempts FROM heec_jobs WHERE job_id = $1")
            .bind(job_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
        let Some((attempts,)) = attempts else {
            return Err(ApiError::NotFound(format!("job {job_id} not found")));
        };
        if attempts >= self.max_attempts {
            query(
                "UPDATE heec_jobs \
                 SET state = 'dead_letter', last_error = $2, \
                     finished_at = to_char(clock_timestamp() AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') \
                 WHERE job_id = $1",
            )
            .bind(job_id)
            .bind(error)
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
        } else {
            query(
                "UPDATE heec_jobs \
                 SET state = 'failed', last_error = $2, \
                     available_at = to_char((clock_timestamp() + ($3 || ' seconds')::interval) AT TIME ZONE 'UTC', 'YYYY-MM-DD\"T\"HH24:MI:SS.MS\"Z\"') \
                 WHERE job_id = $1",
            )
            .bind(job_id)
            .bind(error)
            .bind(retry_after_seconds.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    async fn stats(&self) -> ApiResult<QueueStats> {
        let (pending,): (i64,) = query_as("SELECT COUNT(*) FROM heec_jobs WHERE state = 'pending'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
        let (running,): (i64,) = query_as("SELECT COUNT(*) FROM heec_jobs WHERE state = 'running'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
        let (failed,): (i64,) = query_as("SELECT COUNT(*) FROM heec_jobs WHERE state = 'failed'")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
        let (dead_letter,): (i64,) =
            query_as("SELECT COUNT(*) FROM heec_jobs WHERE state = 'dead_letter'")
                .fetch_one(&self.pool)
                .await
                .map_err(|e| ApiError::Storage(e.to_string()))?;
        let (retries,): (i64,) = query_as("SELECT COALESCE(SUM(attempts), 0) FROM heec_jobs")
            .fetch_one(&self.pool)
            .await
            .map_err(|e| ApiError::Storage(e.to_string()))?;
        Ok(QueueStats {
            pending,
            running,
            failed,
            dead_letter,
            retries,
        })
    }
}
