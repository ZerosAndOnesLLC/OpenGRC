use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub titanium_vault: TitaniumVaultConfig,
    pub cors: CorsConfig,
    pub s3: S3Config,
    pub meilisearch: MeilisearchConfig,
    pub encryption: EncryptionConfig,
    pub environment: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    /// Base URL for the API (used for OAuth callbacks)
    pub api_base_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TitaniumVaultConfig {
    pub api_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CorsConfig {
    pub origins: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StorageConfig {
    /// Storage type: "local" or "s3"
    pub storage_type: String,
    /// Local storage path (used when storage_type = "local")
    pub local_path: Option<String>,
    /// S3 bucket (used when storage_type = "s3")
    pub bucket: Option<String>,
    /// S3 region
    pub region: String,
    /// Custom S3 endpoint (for MinIO, LocalStack)
    pub endpoint: Option<String>,
    /// AWS access key (optional, uses IAM role if not set)
    pub access_key_id: Option<String>,
    /// AWS secret key
    pub secret_access_key: Option<String>,
}

impl StorageConfig {
    pub fn is_local(&self) -> bool {
        self.storage_type.to_lowercase() == "local"
    }

    pub fn is_s3(&self) -> bool {
        self.storage_type.to_lowercase() == "s3"
    }
}

// Backwards compatibility alias
pub type S3Config = StorageConfig;

#[derive(Debug, Clone, Deserialize)]
pub struct MeilisearchConfig {
    pub host: String,
    pub api_key: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EncryptionConfig {
    /// 256-bit encryption key as hex string (64 characters).
    /// Generate with: openssl rand -hex 32
    pub key: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "http://localhost:3000".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        Ok(Config {
            server: ServerConfig {
                host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("PORT")
                    .unwrap_or_else(|_| "8080".to_string())
                    .parse()
                    .expect("PORT must be a valid u16"),
                api_base_url: env::var("API_BASE_URL")
                    .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .expect("DATABASE_MAX_CONNECTIONS must be a valid u32"),
                min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "2".to_string())
                    .parse()
                    .expect("DATABASE_MIN_CONNECTIONS must be a valid u32"),
                acquire_timeout: env::var("DATABASE_ACQUIRE_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .expect("DATABASE_ACQUIRE_TIMEOUT must be a valid u64"),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                pool_size: env::var("REDIS_POOL_SIZE")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .expect("REDIS_POOL_SIZE must be a valid u32"),
            },
            titanium_vault: TitaniumVaultConfig {
                api_url: env::var("TV_API_URL")
                    .unwrap_or_else(|_| "https://api.titanium-vault.com".to_string()),
                client_id: env::var("TV_CLIENT_ID")
                    .unwrap_or_else(|_| "opengrc".to_string()),
                client_secret: env::var("TV_CLIENT_SECRET")
                    .unwrap_or_else(|_| "".to_string()),
                redirect_uri: env::var("TV_REDIRECT_URI")
                    .unwrap_or_else(|_| "http://localhost:3000/sso/callback/".to_string()),
            },
            cors: CorsConfig {
                origins: cors_origins,
            },
            s3: StorageConfig {
                storage_type: env::var("STORAGE_TYPE").unwrap_or_else(|_| "local".to_string()),
                local_path: env::var("STORAGE_LOCAL_PATH").ok(),
                bucket: env::var("S3_BUCKET").ok(),
                region: env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
                endpoint: env::var("S3_ENDPOINT").ok(),
                access_key_id: env::var("AWS_ACCESS_KEY_ID").ok(),
                secret_access_key: env::var("AWS_SECRET_ACCESS_KEY").ok(),
            },
            meilisearch: MeilisearchConfig {
                host: env::var("MEILISEARCH_HOST")
                    .unwrap_or_else(|_| "http://localhost:7700".to_string()),
                api_key: env::var("MEILISEARCH_API_KEY").ok(),
                enabled: env::var("MEILISEARCH_ENABLED")
                    .map(|v| v.to_lowercase() == "true")
                    .unwrap_or(false),
            },
            encryption: EncryptionConfig {
                // Generate a random key for development if not set
                // In production, this MUST be set via environment variable
                key: env::var("ENCRYPTION_KEY").unwrap_or_else(|_| {
                    let key = crate::utils::EncryptionService::generate_key();
                    tracing::warn!(
                        "ENCRYPTION_KEY not set, using generated key. Set ENCRYPTION_KEY in production!"
                    );
                    key
                }),
            },
            environment: env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string()),
        })
    }

    pub fn database_url(&self) -> &str {
        &self.database.url
    }

    pub fn redis_url(&self) -> &str {
        &self.redis.url
    }

    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }
}
