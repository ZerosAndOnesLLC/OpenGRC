use sqlx::PgPool;
use crate::cache::CacheClient;

#[derive(Clone)]
pub struct AppServices {
    pub db: PgPool,
    pub cache: CacheClient,
}

impl AppServices {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }
}
