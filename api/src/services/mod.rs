pub mod asset;
pub mod audit;
pub mod aws;
pub mod control;
pub mod evidence;
pub mod framework;
pub mod integration;
pub mod notification;
pub mod pdf;
pub mod policy;
pub mod reports;
pub mod risk;
pub mod vendor;

use sqlx::PgPool;
use crate::cache::CacheClient;
use crate::config::Config;
use crate::integrations::{AwsProvider, GitHubProvider, JiraProvider, OAuthService};
use crate::search::SearchClient;
use crate::storage::StorageClient;
use crate::utils::EncryptionService;

pub use asset::AssetService;
pub use audit::AuditService;
pub use aws::AwsService;
pub use control::ControlService;
pub use evidence::EvidenceService;
pub use framework::FrameworkService;
pub use integration::IntegrationService;
pub use notification::NotificationService;
pub use pdf::PdfService;
pub use policy::PolicyService;
pub use reports::ReportsService;
pub use risk::RiskService;
pub use vendor::VendorService;

#[derive(Clone)]
pub struct AppServices {
    pub db: PgPool,
    pub cache: CacheClient,
    pub storage: StorageClient,
    pub search: SearchClient,
    pub framework: FrameworkService,
    pub control: ControlService,
    pub evidence: EvidenceService,
    pub policy: PolicyService,
    pub risk: RiskService,
    pub vendor: VendorService,
    pub asset: AssetService,
    pub audit: AuditService,
    pub reports: ReportsService,
    pub pdf: PdfService,
    pub notification: NotificationService,
    pub integration: IntegrationService,
    pub aws: AwsService,
}

impl AppServices {
    pub async fn new(
        db: PgPool,
        cache: CacheClient,
        storage: StorageClient,
        search: SearchClient,
        encryption: EncryptionService,
        oauth_redirect_base_url: String,
        config: &Config,
    ) -> Self {
        let framework = FrameworkService::new(db.clone(), cache.clone());
        let control = ControlService::new(db.clone(), cache.clone());
        let evidence = EvidenceService::new(db.clone(), cache.clone(), storage.clone());
        let policy = PolicyService::new(db.clone(), cache.clone());
        let risk = RiskService::new(db.clone(), cache.clone());
        let vendor = VendorService::new(db.clone(), cache.clone());
        let asset = AssetService::new(db.clone(), cache.clone());
        let audit = AuditService::new(db.clone(), cache.clone());
        let reports = ReportsService::new(db.clone());
        let pdf = PdfService::new(db.clone());
        let notification = NotificationService::new(db.clone(), cache.clone(), config).await;

        // Initialize OAuth service from environment
        let oauth = OAuthService::from_env(oauth_redirect_base_url);
        let integration = IntegrationService::new(db.clone(), cache.clone(), encryption, oauth);

        // Register integration providers
        integration.register_provider(Box::new(AwsProvider::new())).await;
        tracing::info!("Registered AWS integration provider");

        integration.register_provider(Box::new(GitHubProvider::new())).await;
        tracing::info!("Registered GitHub integration provider");

        integration.register_provider(Box::new(JiraProvider::new())).await;
        tracing::info!("Registered Jira integration provider");

        // AWS-specific service for querying synced data
        let aws = AwsService::new(db.clone(), cache.clone());

        Self { db, cache, storage, search, framework, control, evidence, policy, risk, vendor, asset, audit, reports, pdf, notification, integration, aws }
    }
}
