use anyhow::Result;
use redis::aio::ConnectionManager;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use tracing::{error, info, warn};

use crate::jobs::{
    evidence_collection, control_testing, integration_sync,
    notifications, access_review
};

const QUEUE_NAME: &str = "opengrc:jobs";
const PROCESSING_QUEUE: &str = "opengrc:jobs:processing";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub job_type: JobType,
    pub payload: serde_json::Value,
    pub attempts: u32,
    pub max_attempts: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobType {
    EvidenceCollection,
    ControlTesting,
    IntegrationSync,
    SendNotification,
    AccessReviewReminder,
    PolicyAcknowledgmentReminder,
    ReportGeneration,
}

pub struct JobQueue {
    redis: ConnectionManager,
    db: PgPool,
}

impl JobQueue {
    pub fn new(redis: ConnectionManager, db: PgPool) -> Self {
        Self { redis, db }
    }

    pub async fn run(&self) -> Result<()> {
        info!("Job queue worker started");

        loop {
            match self.process_next_job().await {
                Ok(true) => {
                    // Job processed, continue immediately
                }
                Ok(false) => {
                    // No job available, wait before polling again
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                Err(e) => {
                    error!("Error processing job: {}", e);
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn process_next_job(&self) -> Result<bool> {
        let mut redis = self.redis.clone();

        // Pop job from queue (blocking with timeout)
        let job_data: Option<String> = redis
            .rpoplpush(QUEUE_NAME, PROCESSING_QUEUE)
            .await?;

        let Some(job_json) = job_data else {
            return Ok(false);
        };

        let job: Job = serde_json::from_str(&job_json)?;
        info!(job_id = %job.id, job_type = ?job.job_type, "Processing job");

        let result = self.execute_job(&job).await;

        // Remove from processing queue
        let _: () = redis.lrem(PROCESSING_QUEUE, 1, &job_json).await?;

        match result {
            Ok(()) => {
                info!(job_id = %job.id, "Job completed successfully");
            }
            Err(e) => {
                error!(job_id = %job.id, error = %e, "Job failed");

                // Retry if attempts remaining
                if job.attempts < job.max_attempts {
                    let mut retry_job = job;
                    retry_job.attempts += 1;
                    let retry_json = serde_json::to_string(&retry_job)?;
                    let _: () = redis.lpush(QUEUE_NAME, retry_json).await?;
                    warn!(job_id = %retry_job.id, attempt = retry_job.attempts, "Job queued for retry");
                }
            }
        }

        Ok(true)
    }

    async fn execute_job(&self, job: &Job) -> Result<()> {
        match job.job_type {
            JobType::EvidenceCollection => {
                evidence_collection::execute(&self.db, &job.payload).await
            }
            JobType::ControlTesting => {
                control_testing::execute(&self.db, &job.payload).await
            }
            JobType::IntegrationSync => {
                integration_sync::execute(&self.db, &job.payload).await
            }
            JobType::SendNotification => {
                notifications::execute(&self.db, &job.payload).await
            }
            JobType::AccessReviewReminder => {
                access_review::execute(&self.db, &job.payload).await
            }
            JobType::PolicyAcknowledgmentReminder => {
                // TODO: Implement
                Ok(())
            }
            JobType::ReportGeneration => {
                // TODO: Implement
                Ok(())
            }
        }
    }
}

/// Helper to enqueue a job (used by API)
#[allow(dead_code)]
pub async fn enqueue_job(
    redis: &mut ConnectionManager,
    job_type: JobType,
    payload: serde_json::Value,
) -> Result<String> {
    let job = Job {
        id: uuid::Uuid::new_v4().to_string(),
        job_type,
        payload,
        attempts: 0,
        max_attempts: 3,
        created_at: chrono::Utc::now(),
    };

    let job_json = serde_json::to_string(&job)?;
    let _: () = redis.lpush(QUEUE_NAME, job_json).await?;

    Ok(job.id)
}
