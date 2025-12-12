use crate::utils::AppResult;
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct CacheClient {
    manager: ConnectionManager,
}

impl CacheClient {
    pub async fn new(redis_url: &str) -> AppResult<Self> {
        let client = Client::open(redis_url)?;
        let manager = ConnectionManager::new(client).await?;

        Ok(Self { manager })
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> AppResult<Option<T>> {
        let mut conn = self.manager.clone();
        let value: Option<String> = conn.get(key).await?;

        match value {
            Some(v) => {
                let deserialized = serde_json::from_str(&v)
                    .map_err(|e| crate::utils::AppError::InternalServerError(format!("Failed to deserialize cache value: {}", e)))?;
                Ok(Some(deserialized))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> AppResult<()> {
        let mut conn = self.manager.clone();
        let serialized = serde_json::to_string(value)
            .map_err(|e| crate::utils::AppError::InternalServerError(format!("Failed to serialize cache value: {}", e)))?;

        if let Some(ttl) = ttl {
            conn.set_ex::<_, _, ()>(key, serialized, ttl.as_secs()).await?;
        } else {
            conn.set::<_, _, ()>(key, serialized).await?;
        }

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> AppResult<()> {
        let mut conn = self.manager.clone();
        conn.del::<_, ()>(key).await?;
        Ok(())
    }

    pub async fn delete_pattern(&self, pattern: &str) -> AppResult<()> {
        let mut conn = self.manager.clone();
        let keys: Vec<String> = conn.keys(pattern).await?;

        if !keys.is_empty() {
            conn.del::<_, ()>(keys).await?;
        }

        Ok(())
    }

    pub async fn exists(&self, key: &str) -> AppResult<bool> {
        let mut conn = self.manager.clone();
        let exists: bool = conn.exists(key).await?;
        Ok(exists)
    }

    pub async fn increment(&self, key: &str) -> AppResult<i64> {
        let mut conn = self.manager.clone();
        let value: i64 = conn.incr(key, 1).await?;
        Ok(value)
    }

    pub async fn expire(&self, key: &str, ttl: Duration) -> AppResult<()> {
        let mut conn = self.manager.clone();
        conn.expire::<_, ()>(key, ttl.as_secs() as i64).await?;
        Ok(())
    }
}

pub fn cache_key(prefix: &str, id: &str) -> String {
    format!("{}:{}", prefix, id)
}

pub fn org_cache_key(org_id: &str, entity: &str, id: &str) -> String {
    format!("org:{}:{}:{}", org_id, entity, id)
}
