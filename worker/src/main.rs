use anyhow::Result;
use sqlx::postgres::PgPoolOptions;
use tracing::{info, Level};
use tracing_subscriber::EnvFilter;

mod config;
mod jobs;
mod queue;

use config::Config;
use queue::JobQueue;

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(Level::INFO.into())
                .from_env_lossy(),
        )
        .json()
        .init();

    info!("Starting OpenGRC Worker");

    // Load configuration
    let config = Config::from_env()?;

    // Initialize database pool
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await?;

    info!("Connected to database");

    // Initialize Redis connection
    let redis_client = redis::Client::open(config.redis_url.as_str())?;
    let redis_conn = redis::aio::ConnectionManager::new(redis_client).await?;

    info!("Connected to Redis");

    // Initialize job queue
    let job_queue = JobQueue::new(redis_conn, db_pool);

    // Set up graceful shutdown
    let shutdown = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Shutdown signal received");
    };

    // Run the worker
    tokio::select! {
        result = job_queue.run() => {
            if let Err(e) = result {
                tracing::error!("Worker error: {}", e);
            }
        }
        _ = shutdown => {
            info!("Shutting down gracefully");
        }
    }

    info!("Worker stopped");
    Ok(())
}
