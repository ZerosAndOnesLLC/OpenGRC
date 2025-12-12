use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub database_max_connections: u32,
    pub redis_url: String,
    #[allow(dead_code)]
    pub worker_concurrency: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .context("DATABASE_URL must be set")?,
            database_max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("DATABASE_MAX_CONNECTIONS must be a number")?,
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            worker_concurrency: std::env::var("WORKER_CONCURRENCY")
                .unwrap_or_else(|_| "4".to_string())
                .parse()
                .context("WORKER_CONCURRENCY must be a number")?,
        })
    }
}
