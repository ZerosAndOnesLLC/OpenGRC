pub mod control;
pub mod framework;

use sqlx::PgPool;
use crate::cache::CacheClient;

pub use control::ControlService;
pub use framework::FrameworkService;

#[derive(Clone)]
pub struct AppServices {
    pub db: PgPool,
    pub cache: CacheClient,
    pub framework: FrameworkService,
    pub control: ControlService,
}

impl AppServices {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        let framework = FrameworkService::new(db.clone(), cache.clone());
        let control = ControlService::new(db.clone(), cache.clone());
        Self { db, cache, framework, control }
    }
}
