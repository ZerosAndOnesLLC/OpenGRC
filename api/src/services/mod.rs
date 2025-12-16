pub mod access_review;
pub mod ai;
pub mod analytics;
pub mod asset;
pub mod audit;
pub mod aws;
pub mod collaboration;
pub mod control;
pub mod control_test_automation;
pub mod enterprise;
pub mod evidence;
pub mod evidence_automation;
pub mod framework;
pub mod integration;
pub mod notification;
pub mod pdf;
pub mod policy;
pub mod questionnaire;
pub mod reports;
pub mod risk;
pub mod soc2_parser;
pub mod task;
pub mod vendor;

use sqlx::PgPool;
use crate::cache::CacheClient;
use crate::config::Config;
use crate::integrations::{AwsProvider, AzureAdProvider, GitHubProvider, GoogleWorkspaceProvider, JiraProvider, OAuthService, OktaProvider};
use crate::search::SearchClient;
use crate::storage::StorageClient;
use crate::utils::EncryptionService;

pub use access_review::AccessReviewService;
pub use ai::AiService;
pub use analytics::AnalyticsService;
pub use asset::AssetService;
pub use audit::AuditService;
pub use aws::AwsService;
pub use collaboration::CollaborationService;
pub use control::ControlService;
pub use control_test_automation::ControlTestAutomationService;
pub use enterprise::EnterpriseService;
pub use evidence::EvidenceService;
pub use evidence_automation::EvidenceAutomationService;
pub use framework::FrameworkService;
pub use integration::IntegrationService;
pub use notification::NotificationService;
pub use pdf::PdfService;
pub use policy::PolicyService;
pub use questionnaire::QuestionnaireService;
pub use reports::ReportsService;
pub use risk::RiskService;
pub use soc2_parser::Soc2ParserService;
pub use task::TaskService;
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
    pub task: TaskService,
    pub reports: ReportsService,
    pub pdf: PdfService,
    pub notification: NotificationService,
    pub integration: IntegrationService,
    pub aws: AwsService,
    pub evidence_automation: EvidenceAutomationService,
    pub control_test_automation: ControlTestAutomationService,
    pub questionnaire: QuestionnaireService,
    pub soc2_parser: Soc2ParserService,
    pub access_review: AccessReviewService,
    pub ai: AiService,
    pub analytics: AnalyticsService,
    pub enterprise: EnterpriseService,
    pub collaboration: CollaborationService,
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
        let task = TaskService::new(db.clone(), cache.clone());
        let reports = ReportsService::new(db.clone());
        let pdf = PdfService::new(db.clone());
        let notification = NotificationService::new(db.clone(), cache.clone(), config).await;

        // Initialize OAuth service from environment
        let oauth = OAuthService::from_env(oauth_redirect_base_url);
        let integration = IntegrationService::new(db.clone(), cache.clone(), encryption.clone(), oauth);

        // Register integration providers
        integration.register_provider(Box::new(AwsProvider::new())).await;
        tracing::info!("Registered AWS integration provider");

        integration.register_provider(Box::new(GitHubProvider::new())).await;
        tracing::info!("Registered GitHub integration provider");

        integration.register_provider(Box::new(JiraProvider::new())).await;
        tracing::info!("Registered Jira integration provider");

        integration.register_provider(Box::new(OktaProvider::new())).await;
        tracing::info!("Registered Okta integration provider");

        integration.register_provider(Box::new(GoogleWorkspaceProvider::new())).await;
        tracing::info!("Registered Google Workspace integration provider");

        integration.register_provider(Box::new(AzureAdProvider::new())).await;
        tracing::info!("Registered Azure AD integration provider");

        // AWS-specific service for querying synced data
        let aws = AwsService::new(db.clone(), cache.clone());

        // Evidence automation service
        let evidence_automation = EvidenceAutomationService::new(db.clone(), cache.clone());

        // Control test automation service
        let control_test_automation = ControlTestAutomationService::new(db.clone(), cache.clone());

        // Questionnaire service
        let questionnaire = QuestionnaireService::new(db.clone(), cache.clone());

        // SOC 2 parser service
        let soc2_parser = Soc2ParserService::new(db.clone(), cache.clone());

        // Access review service
        let access_review = AccessReviewService::new(db.clone(), cache.clone());

        // AI service
        let ai = AiService::new(db.clone(), cache.clone(), encryption);

        // Analytics service
        let analytics = AnalyticsService::new(db.clone(), cache.clone());

        // Enterprise service (RBAC, SSO, SCIM, audit exports, branding, API keys)
        let enterprise = EnterpriseService::new(db.clone(), std::sync::Arc::new(cache.clone()));

        // Collaboration service (comments, mentions, presence, Slack/Teams, digests)
        let collaboration = CollaborationService::new(db.clone(), cache.clone());

        Self { db, cache, storage, search, framework, control, evidence, policy, risk, vendor, asset, audit, task, reports, pdf, notification, integration, aws, evidence_automation, control_test_automation, questionnaire, soc2_parser, access_review, ai, analytics, enterprise, collaboration }
    }
}
