pub mod asset;
pub mod audit;
pub mod control;
pub mod evidence;
pub mod framework;
pub mod integration;
pub mod policy;
pub mod questionnaire;
pub mod risk;
pub mod vendor;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub settings: serde_json::Value,
    pub subscription_tier: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub tv_user_id: String,
    pub email: String,
    pub name: String,
    pub role: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub use framework::{
    Framework, FrameworkRequirement, CreateFramework, UpdateFramework,
    CreateFrameworkRequirement, UpdateFrameworkRequirement, FrameworkWithRequirements,
    FrameworkGapAnalysis, CategoryGapAnalysis, RequirementGapAnalysis,
};

pub use control::{
    Control, ControlWithMappings, MappedRequirement, ControlRequirementMapping,
    CreateControl, UpdateControl, ControlTest, CreateControlTest, UpdateControlTest,
    ControlTestResult, CreateTestResult, ListControlsQuery, ControlStats,
};

pub use evidence::{
    Evidence, EvidenceWithLinks, LinkedControl, EvidenceControlLink,
    CreateEvidence, UpdateEvidence, ListEvidenceQuery, EvidenceStats,
    TypeCount, SourceCount,
    // Evidence automation types
    EvidenceWithFreshness, EvidenceCollectionTask, CreateEvidenceCollectionTask,
    UpdateEvidenceCollectionTask, EvidenceCollectionRun, EvidenceChange,
    EvidenceChangeWithDetails, EvidenceControlMappingRule, CreateMappingRule,
    UpdateMappingRule, EvidenceFreshnessSla, CreateFreshnessSla,
    EvidenceFreshnessSummary, StaleEvidenceBySource, EvidenceCollectionTaskWithStats,
    ListCollectionTasksQuery, ListEvidenceChangesQuery,
};

pub use policy::{
    Policy, PolicyVersion, PolicyAcknowledgment, PolicyWithStats,
    CreatePolicy, UpdatePolicy, ListPoliciesQuery, PolicyStats, CategoryCount,
};

pub use risk::{
    Risk, RiskWithControls, LinkedControlSummary, RiskControlMapping,
    CreateRisk, UpdateRisk, ListRisksQuery, RiskStats, StatusCount, RiskCategoryCount,
    LinkControlsRequest, RiskHeatmapData, HeatmapCell,
};

pub use vendor::{
    Vendor, VendorWithAssessment, VendorAssessment, CreateVendor, UpdateVendor,
    CreateVendorAssessment, ListVendorsQuery, VendorStats, CriticalityCount, VendorCategoryCount,
    VendorDocument, CreateVendorDocument, UpdateVendorDocument,
};

pub use asset::{
    Asset, AssetWithControls, AssetControlMapping, CreateAsset, UpdateAsset,
    ListAssetsQuery, AssetStats, AssetTypeCount, ClassificationCount, AssetStatusCount,
    LifecycleStageCount, AssetLifecycleEvent, CreateLifecycleEvent, DiscoveredAsset,
};

pub use audit::{
    Audit, AuditWithStats, AuditRequest, AuditRequestResponse, AuditFinding,
    CreateAudit, UpdateAudit, CreateAuditRequest, CreateAuditFinding, UpdateAuditFinding,
    CreateRequestResponse, ListAuditsQuery, AuditStats, AuditTypeCount,
};

pub use integration::{
    Integration, IntegrationWithStats, IntegrationSyncLog, IntegrationType, IntegrationStatus,
    SyncStatus, CreateIntegration, UpdateIntegration, ListIntegrationsQuery, IntegrationStats,
    IntegrationTypeCount, AvailableIntegration, TestConnectionResult, TriggerSyncRequest,
    get_available_integrations,
    // Health monitoring types
    HealthStatus, IntegrationHealth, IntegrationHealthWithDetails, IntegrationHealthSnapshot,
    IntegrationHealthStats, HealthTrendPoint, RecentFailure,
    // OAuth types
    AuthMethod, IntegrationOAuthState, OAuthAuthorizeRequest, OAuthAuthorizeResponse,
    OAuthCallbackParams, OAuthTokenResponse, OAuthRefreshRequest, OAuthProviderConfig,
    // Error handling and retry types
    SyncErrorCategory, CircuitBreakerState,
};

pub use questionnaire::{
    QuestionnaireTemplate, QuestionnaireTemplateWithDetails, CreateQuestionnaireTemplate,
    UpdateQuestionnaireTemplate, QuestionnaireSection, QuestionnaireSectionWithQuestions,
    CreateQuestionnaireSection, UpdateQuestionnaireSection, QuestionnaireQuestion,
    CreateQuestionnaireQuestion, UpdateQuestionnaireQuestion, QuestionnaireAssignment,
    QuestionnaireAssignmentWithDetails, CreateQuestionnaireAssignment,
    ReviewQuestionnaireAssignment, ListQuestionnaireAssignmentsQuery, QuestionnaireResponse,
    QuestionnaireResponseWithQuestion, SaveQuestionnaireResponse, BulkSaveQuestionnaireResponses,
    QuestionnaireResponseComment, CreateResponseComment, VendorPortalAccess,
    VendorPortalSubmission, QuestionnaireStats,
};
