pub mod control;
pub mod evidence;
pub mod framework;

use sqlx::PgPool;
use crate::cache::CacheClient;

pub use control::ControlService;
pub use evidence::EvidenceService;
pub use framework::FrameworkService;

#[derive(Clone)]
pub struct AppServices {
    pub db: PgPool,
    pub cache: CacheClient,
    pub framework: FrameworkService,
    pub control: ControlService,
    pub evidence: EvidenceService,
}

impl AppServices {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        let framework = FrameworkService::new(db.clone(), cache.clone());
        let control = ControlService::new(db.clone(), cache.clone());
        let evidence = EvidenceService::new(db.clone(), cache.clone());
        Self { db, cache, framework, control, evidence }
    }
}
