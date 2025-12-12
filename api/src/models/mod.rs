pub mod control;
pub mod evidence;
pub mod framework;
pub mod policy;
pub mod risk;

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
};

pub use policy::{
    Policy, PolicyVersion, PolicyAcknowledgment, PolicyWithStats,
    CreatePolicy, UpdatePolicy, ListPoliciesQuery, PolicyStats, CategoryCount,
};

pub use risk::{
    Risk, RiskWithControls, LinkedControlSummary, RiskControlMapping,
    CreateRisk, UpdateRisk, ListRisksQuery, RiskStats, StatusCount, RiskCategoryCount,
    LinkControlsRequest,
};
