use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

/// Supported integration types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationType {
    // Cloud Providers
    Aws,
    Gcp,
    Azure,
    // Identity Providers
    Okta,
    GoogleWorkspace,
    AzureAd,
    // DevOps
    Github,
    Gitlab,
    Jira,
    // Infrastructure
    Cloudflare,
    Datadog,
    Pagerduty,
    // Custom webhook
    Webhook,
}

impl IntegrationType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Aws => "aws",
            Self::Gcp => "gcp",
            Self::Azure => "azure",
            Self::Okta => "okta",
            Self::GoogleWorkspace => "google_workspace",
            Self::AzureAd => "azure_ad",
            Self::Github => "github",
            Self::Gitlab => "gitlab",
            Self::Jira => "jira",
            Self::Cloudflare => "cloudflare",
            Self::Datadog => "datadog",
            Self::Pagerduty => "pagerduty",
            Self::Webhook => "webhook",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "aws" => Some(Self::Aws),
            "gcp" => Some(Self::Gcp),
            "azure" => Some(Self::Azure),
            "okta" => Some(Self::Okta),
            "google_workspace" => Some(Self::GoogleWorkspace),
            "azure_ad" => Some(Self::AzureAd),
            "github" => Some(Self::Github),
            "gitlab" => Some(Self::Gitlab),
            "jira" => Some(Self::Jira),
            "cloudflare" => Some(Self::Cloudflare),
            "datadog" => Some(Self::Datadog),
            "pagerduty" => Some(Self::Pagerduty),
            "webhook" => Some(Self::Webhook),
            _ => None,
        }
    }

    pub fn all() -> Vec<Self> {
        vec![
            Self::Aws,
            Self::Gcp,
            Self::Azure,
            Self::Okta,
            Self::GoogleWorkspace,
            Self::AzureAd,
            Self::Github,
            Self::Gitlab,
            Self::Jira,
            Self::Cloudflare,
            Self::Datadog,
            Self::Pagerduty,
            Self::Webhook,
        ]
    }
}

/// Integration status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationStatus {
    Active,
    Inactive,
    Error,
    Syncing,
}

impl IntegrationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Error => "error",
            Self::Syncing => "syncing",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "active" => Self::Active,
            "inactive" => Self::Inactive,
            "error" => Self::Error,
            "syncing" => Self::Syncing,
            _ => Self::Inactive,
        }
    }
}

/// Sync status for integration sync logs
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl SyncStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Integration database model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Integration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub integration_type: String,
    pub name: String,
    pub config: Option<serde_json::Value>,
    pub status: String,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Integration with additional computed fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationWithStats {
    #[serde(flatten)]
    pub integration: Integration,
    pub sync_count: i64,
    pub last_sync_status: Option<String>,
    pub records_synced: Option<i64>,
}

/// Integration sync log
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationSyncLog {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub sync_type: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub records_processed: Option<i32>,
    pub errors: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Create integration request
#[derive(Debug, Clone, Deserialize)]
pub struct CreateIntegration {
    pub integration_type: String,
    pub name: String,
    pub config: Option<serde_json::Value>,
}

/// Update integration request
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateIntegration {
    pub name: Option<String>,
    pub config: Option<serde_json::Value>,
    pub status: Option<String>,
}

/// List integrations query params
#[derive(Debug, Clone, Deserialize, Default)]
pub struct ListIntegrationsQuery {
    pub integration_type: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Integration statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStats {
    pub total: i64,
    pub active: i64,
    pub inactive: i64,
    pub error: i64,
    pub by_type: Vec<IntegrationTypeCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationTypeCount {
    pub integration_type: String,
    pub count: i64,
}

/// Available integration type definition (for UI)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableIntegration {
    pub integration_type: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub capabilities: Vec<String>,
    pub config_schema: serde_json::Value,
    pub logo_url: Option<String>,
}

/// Test connection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionResult {
    pub success: bool,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

/// Trigger sync request
#[derive(Debug, Clone, Deserialize)]
pub struct TriggerSyncRequest {
    pub sync_type: Option<String>,
    pub full_sync: Option<bool>,
}

impl Integration {
    pub fn validate_create(input: &CreateIntegration) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return Err("Name is required".to_string());
        }
        if input.name.len() > 255 {
            return Err("Name must be less than 255 characters".to_string());
        }
        if IntegrationType::from_str(&input.integration_type).is_none() {
            return Err(format!("Invalid integration type: {}", input.integration_type));
        }
        Ok(())
    }

    pub fn get_type(&self) -> Option<IntegrationType> {
        IntegrationType::from_str(&self.integration_type)
    }

    pub fn get_status(&self) -> IntegrationStatus {
        IntegrationStatus::from_str(&self.status)
    }
}

/// Health status for integrations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl HealthStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Degraded => "degraded",
            Self::Unhealthy => "unhealthy",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "healthy" => Self::Healthy,
            "degraded" => Self::Degraded,
            "unhealthy" => Self::Unhealthy,
            _ => Self::Unknown,
        }
    }
}

/// Integration health record
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationHealth {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub status: String,
    pub last_successful_sync_at: Option<DateTime<Utc>>,
    pub consecutive_failures: i32,
    pub sync_success_count_24h: i32,
    pub sync_failure_count_24h: i32,
    pub average_sync_duration_ms: Option<i32>,
    pub sync_success_count_7d: i32,
    pub sync_failure_count_7d: i32,
    pub last_check_at: Option<DateTime<Utc>>,
    pub last_check_message: Option<String>,
    pub last_error_at: Option<DateTime<Utc>>,
    pub last_error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl IntegrationHealth {
    pub fn get_status(&self) -> HealthStatus {
        HealthStatus::from_str(&self.status)
    }

    /// Calculate success rate for 24h window
    pub fn success_rate_24h(&self) -> f64 {
        let total = self.sync_success_count_24h + self.sync_failure_count_24h;
        if total == 0 {
            return 100.0;
        }
        (self.sync_success_count_24h as f64 / total as f64) * 100.0
    }

    /// Calculate success rate for 7d window
    pub fn success_rate_7d(&self) -> f64 {
        let total = self.sync_success_count_7d + self.sync_failure_count_7d;
        if total == 0 {
            return 100.0;
        }
        (self.sync_success_count_7d as f64 / total as f64) * 100.0
    }
}

/// Integration health with integration details (for dashboard)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationHealthWithDetails {
    pub integration_id: Uuid,
    pub integration_name: String,
    pub integration_type: String,
    pub health: IntegrationHealth,
    pub success_rate_24h: f64,
    pub success_rate_7d: f64,
}

/// Health snapshot for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationHealthSnapshot {
    pub id: Uuid,
    pub integration_id: Uuid,
    pub status: String,
    pub sync_success_rate: Option<rust_decimal::Decimal>,
    pub average_sync_duration_ms: Option<i32>,
    pub error_count: i32,
    pub snapshot_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Aggregated health statistics for all integrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationHealthStats {
    pub total_integrations: i64,
    pub healthy_count: i64,
    pub degraded_count: i64,
    pub unhealthy_count: i64,
    pub unknown_count: i64,
    pub overall_success_rate_24h: f64,
    pub overall_success_rate_7d: f64,
    pub average_sync_duration_ms: Option<i32>,
    pub total_syncs_24h: i64,
    pub total_failures_24h: i64,
}

/// Health trend data point for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthTrendPoint {
    pub timestamp: DateTime<Utc>,
    pub healthy_count: i64,
    pub degraded_count: i64,
    pub unhealthy_count: i64,
    pub success_rate: f64,
}

/// Recent failure for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentFailure {
    pub integration_id: Uuid,
    pub integration_name: String,
    pub integration_type: String,
    pub error_message: Option<String>,
    pub failed_at: DateTime<Utc>,
    pub consecutive_failures: i32,
}

/// Get all available integration definitions
pub fn get_available_integrations() -> Vec<AvailableIntegration> {
    vec![
        AvailableIntegration {
            integration_type: "aws".to_string(),
            name: "Amazon Web Services".to_string(),
            description: "Sync IAM users, roles, CloudTrail logs, Security Hub findings, and more".to_string(),
            category: "Cloud Provider".to_string(),
            capabilities: vec![
                "IAM Users & Roles".to_string(),
                "CloudTrail Logs".to_string(),
                "Security Hub Findings".to_string(),
                "Config Compliance".to_string(),
                "S3 Bucket Policies".to_string(),
                "EC2/RDS Inventory".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["access_key_id", "secret_access_key", "region"],
                "properties": {
                    "access_key_id": { "type": "string", "title": "Access Key ID" },
                    "secret_access_key": { "type": "string", "title": "Secret Access Key", "secret": true },
                    "region": { "type": "string", "title": "Region", "default": "us-east-1" },
                    "assume_role_arn": { "type": "string", "title": "Assume Role ARN (optional)" }
                }
            }),
            logo_url: Some("/integrations/aws.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "github".to_string(),
            name: "GitHub".to_string(),
            description: "Sync repositories, branch protection rules, security alerts, and access permissions".to_string(),
            category: "DevOps".to_string(),
            capabilities: vec![
                "Repository Inventory".to_string(),
                "Branch Protection Rules".to_string(),
                "Dependabot Alerts".to_string(),
                "Code Scanning Alerts".to_string(),
                "Access Permissions".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["access_token"],
                "properties": {
                    "access_token": { "type": "string", "title": "Personal Access Token", "secret": true },
                    "organization": { "type": "string", "title": "Organization (optional)" },
                    "include_private": { "type": "boolean", "title": "Include Private Repos", "default": true }
                }
            }),
            logo_url: Some("/integrations/github.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "okta".to_string(),
            name: "Okta".to_string(),
            description: "Sync users, groups, MFA status, application assignments, and system logs".to_string(),
            category: "Identity Provider".to_string(),
            capabilities: vec![
                "User Inventory".to_string(),
                "MFA Status".to_string(),
                "Application Assignments".to_string(),
                "System Logs".to_string(),
                "Group Memberships".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["domain", "api_token"],
                "properties": {
                    "domain": { "type": "string", "title": "Okta Domain", "placeholder": "your-org.okta.com" },
                    "api_token": { "type": "string", "title": "API Token", "secret": true }
                }
            }),
            logo_url: Some("/integrations/okta.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "gcp".to_string(),
            name: "Google Cloud Platform".to_string(),
            description: "Sync IAM, audit logs, Security Command Center, and asset inventory".to_string(),
            category: "Cloud Provider".to_string(),
            capabilities: vec![
                "IAM & Admin".to_string(),
                "Cloud Audit Logs".to_string(),
                "Security Command Center".to_string(),
                "Asset Inventory".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["service_account_json", "project_id"],
                "properties": {
                    "service_account_json": { "type": "string", "title": "Service Account JSON", "secret": true, "multiline": true },
                    "project_id": { "type": "string", "title": "Project ID" }
                }
            }),
            logo_url: Some("/integrations/gcp.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "azure".to_string(),
            name: "Microsoft Azure".to_string(),
            description: "Sync Azure AD, activity logs, Security Center, and resource inventory".to_string(),
            category: "Cloud Provider".to_string(),
            capabilities: vec![
                "Azure AD Users/Groups".to_string(),
                "Activity Logs".to_string(),
                "Security Center".to_string(),
                "Resource Inventory".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["tenant_id", "client_id", "client_secret"],
                "properties": {
                    "tenant_id": { "type": "string", "title": "Tenant ID" },
                    "client_id": { "type": "string", "title": "Client ID" },
                    "client_secret": { "type": "string", "title": "Client Secret", "secret": true },
                    "subscription_id": { "type": "string", "title": "Subscription ID (optional)" }
                }
            }),
            logo_url: Some("/integrations/azure.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "google_workspace".to_string(),
            name: "Google Workspace".to_string(),
            description: "Sync user directory, security settings, and login audit".to_string(),
            category: "Identity Provider".to_string(),
            capabilities: vec![
                "User Directory".to_string(),
                "Security Settings".to_string(),
                "Login Audit".to_string(),
                "Group Memberships".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["service_account_json", "admin_email"],
                "properties": {
                    "service_account_json": { "type": "string", "title": "Service Account JSON", "secret": true, "multiline": true },
                    "admin_email": { "type": "string", "title": "Admin Email" },
                    "customer_id": { "type": "string", "title": "Customer ID (optional)" }
                }
            }),
            logo_url: Some("/integrations/google-workspace.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "azure_ad".to_string(),
            name: "Azure Active Directory / Entra ID".to_string(),
            description: "Sync users, groups, conditional access policies, and sign-in logs".to_string(),
            category: "Identity Provider".to_string(),
            capabilities: vec![
                "Users and Groups".to_string(),
                "Conditional Access".to_string(),
                "Sign-in Logs".to_string(),
                "MFA Status".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["tenant_id", "client_id", "client_secret"],
                "properties": {
                    "tenant_id": { "type": "string", "title": "Tenant ID" },
                    "client_id": { "type": "string", "title": "Client ID" },
                    "client_secret": { "type": "string", "title": "Client Secret", "secret": true }
                }
            }),
            logo_url: Some("/integrations/azure-ad.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "gitlab".to_string(),
            name: "GitLab".to_string(),
            description: "Sync projects, security scanning results, and access permissions".to_string(),
            category: "DevOps".to_string(),
            capabilities: vec![
                "Project Inventory".to_string(),
                "Branch Protection".to_string(),
                "Security Scanning".to_string(),
                "Access Permissions".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["access_token"],
                "properties": {
                    "access_token": { "type": "string", "title": "Personal Access Token", "secret": true },
                    "base_url": { "type": "string", "title": "GitLab URL", "default": "https://gitlab.com" },
                    "group_id": { "type": "string", "title": "Group ID (optional)" }
                }
            }),
            logo_url: Some("/integrations/gitlab.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "jira".to_string(),
            name: "Jira".to_string(),
            description: "Sync security tickets and change management records".to_string(),
            category: "DevOps".to_string(),
            capabilities: vec![
                "Security Ticket Tracking".to_string(),
                "Change Management".to_string(),
                "Issue Export".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["base_url", "email", "api_token"],
                "properties": {
                    "base_url": { "type": "string", "title": "Jira URL", "placeholder": "https://your-org.atlassian.net" },
                    "email": { "type": "string", "title": "Email" },
                    "api_token": { "type": "string", "title": "API Token", "secret": true },
                    "project_keys": { "type": "string", "title": "Project Keys (comma-separated)" }
                }
            }),
            logo_url: Some("/integrations/jira.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "cloudflare".to_string(),
            name: "Cloudflare".to_string(),
            description: "Sync WAF rules and DDoS protection status".to_string(),
            category: "Infrastructure".to_string(),
            capabilities: vec![
                "WAF Rules".to_string(),
                "DDoS Protection".to_string(),
                "DNS Records".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["api_token"],
                "properties": {
                    "api_token": { "type": "string", "title": "API Token", "secret": true },
                    "zone_ids": { "type": "string", "title": "Zone IDs (comma-separated, optional)" }
                }
            }),
            logo_url: Some("/integrations/cloudflare.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "datadog".to_string(),
            name: "Datadog".to_string(),
            description: "Sync monitoring configuration and alert policies".to_string(),
            category: "Infrastructure".to_string(),
            capabilities: vec![
                "Monitoring Configuration".to_string(),
                "Alert Policies".to_string(),
                "Dashboard Export".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["api_key", "app_key"],
                "properties": {
                    "api_key": { "type": "string", "title": "API Key", "secret": true },
                    "app_key": { "type": "string", "title": "Application Key", "secret": true },
                    "site": { "type": "string", "title": "Site", "default": "datadoghq.com" }
                }
            }),
            logo_url: Some("/integrations/datadog.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "pagerduty".to_string(),
            name: "PagerDuty".to_string(),
            description: "Sync incident response data and on-call schedules".to_string(),
            category: "Infrastructure".to_string(),
            capabilities: vec![
                "Incident Response".to_string(),
                "On-call Schedules".to_string(),
                "Escalation Policies".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["api_key"],
                "properties": {
                    "api_key": { "type": "string", "title": "API Key", "secret": true }
                }
            }),
            logo_url: Some("/integrations/pagerduty.svg".to_string()),
        },
        AvailableIntegration {
            integration_type: "webhook".to_string(),
            name: "Custom Webhook".to_string(),
            description: "Send compliance events to custom endpoints".to_string(),
            category: "Custom".to_string(),
            capabilities: vec![
                "Event Notifications".to_string(),
                "Custom Payloads".to_string(),
            ],
            config_schema: serde_json::json!({
                "type": "object",
                "required": ["url"],
                "properties": {
                    "url": { "type": "string", "title": "Webhook URL" },
                    "secret": { "type": "string", "title": "Secret (for HMAC signing)", "secret": true },
                    "events": { "type": "array", "title": "Events to Send", "items": { "type": "string" } }
                }
            }),
            logo_url: Some("/integrations/webhook.svg".to_string()),
        },
    ]
}
