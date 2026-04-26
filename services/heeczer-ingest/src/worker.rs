//! Queue worker loop with backoff, retry, DLQ, and graceful shutdown hooks.

use std::future::Future;
use std::time::Duration;

use tokio::time::sleep;

use crate::error::ApiResult;
use crate::queue::{JobQueue, JobRecord};

/// Run a queue worker until `shutdown` resolves.
pub async fn run_worker<Q, F, Fut, S>(
    queue: Q,
    idle_backoff: Duration,
    process: F,
    shutdown: S,
) -> ApiResult<()>
where
    Q: JobQueue,
    F: Fn(JobRecord) -> Fut,
    Fut: Future<Output = ApiResult<()>>,
    S: Future<Output = ()>,
{
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            () = &mut shutdown => return Ok(()),
            claimed = queue.claim_next() => {
                match claimed? {
                    Some(job) => {
                        let job_id = job.job_id.clone();
                        match process(job).await {
                            Ok(()) => queue.complete(&job_id).await?,
                            Err(err) => queue.fail(&job_id, &err.to_string(), 5).await?,
                        }
                    }
                    None => sleep(idle_backoff).await,
                }
            }
        }
    }
}
