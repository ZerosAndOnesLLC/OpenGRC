pub mod control;
pub mod evidence;
pub mod framework;
pub mod policy;
pub mod risk;

use sqlx::PgPool;
use crate::cache::CacheClient;

pub use control::ControlService;
pub use evidence::EvidenceService;
pub use framework::FrameworkService;
pub use policy::PolicyService;
pub use risk::RiskService;

#[derive(Clone)]
pub struct AppServices {
    pub db: PgPool,
    pub cache: CacheClient,
    pub framework: FrameworkService,
    pub control: ControlService,
    pub evidence: EvidenceService,
    pub policy: PolicyService,
    pub risk: RiskService,
}

impl AppServices {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        let framework = FrameworkService::new(db.clone(), cache.clone());
        let control = ControlService::new(db.clone(), cache.clone());
        let evidence = EvidenceService::new(db.clone(), cache.clone());
        let policy = PolicyService::new(db.clone(), cache.clone());
        let risk = RiskService::new(db.clone(), cache.clone());
        Self { db, cache, framework, control, evidence, policy, risk }
    }
}
