use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

// ============================================================================
// AUTH METHOD
// ============================================================================

/// Authentication method for integrations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    ApiKey,
    OAuth2,
    ServiceAccount,
}

impl AuthMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ApiKey => "api_key",
            Self::OAuth2 => "oauth2",
            Self::ServiceAccount => "service_account",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "oauth2" => Self::OAuth2,
            "service_account" => Self::ServiceAccount,
            _ => Self::ApiKey,
        }
    }
}

impl Default for AuthMethod {
    fn default() -> Self {
        Self::ApiKey
    }
}

// ============================================================================
// ERROR CATEGORIES FOR RETRY LOGIC
// ============================================================================

/// Error category for sync failures - determines retry behavior
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncErrorCategory {
    /// Temporary errors (network, timeout) - should retry
    Transient,
    /// Rate limit hit - retry with backoff
    RateLimited,
    /// Authentication/authorization error - may need re-auth
    AuthFailure,
    /// Configuration problem - user needs to fix
    ConfigError,
    /// Permanent error - don't retry
    Permanent,
    /// Unclassified error
    Unknown,
}

impl SyncErrorCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Transient => "transient",
            Self::RateLimited => "rate_limited",
            Self::AuthFailure => "auth_failure",
            Self::ConfigError => "config_error",
            Self::Permanent => "permanent",
            Self::Unknown => "unknown",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "transient" => Self::Transient,
            "rate_limited" => Self::RateLimited,
            "auth_failure" => Self::AuthFailure,
            "config_error" => Self::ConfigError,
            "permanent" => Self::Permanent,
            _ => Self::Unknown,
        }
    }

    /// Whether this error category should be retried
    pub fn should_retry(&self) -> bool {
        matches!(self, Self::Transient | Self::RateLimited | Self::Unknown)
    }

    /// Classify an error based on code and message
    pub fn classify(code: &str, message: &str) -> Self {
        let code_lower = code.to_lowercase();
        let msg_lower = message.to_lowercase();

        // Rate limiting
        if code_lower == "429" || code_lower == "rate_limit" || code_lower == "too_many_requests"
            || msg_lower.contains("rate limit")
            || msg_lower.contains("too many requests")
            || msg_lower.contains("quota exceeded")
        {
            return Self::RateLimited;
        }

        // Authentication failures
        if code_lower == "401" || code_lower == "403" || code_lower == "unauthorized"
            || code_lower == "forbidden" || code_lower == "invalid_token"
            || msg_lower.contains("unauthorized")
            || msg_lower.contains("forbidden")
            || msg_lower.contains("invalid token")
            || msg_lower.contains("token expired")
            || msg_lower.contains("authentication")
        {
            return Self::AuthFailure;
        }

        // Configuration errors
        if code_lower == "400" || code_lower == "404" || code_lower == "not_found"
            || code_lower == "invalid_config"
            || msg_lower.contains("not found")
            || msg_lower.contains("invalid config")
            || msg_lower.contains("missing required")
        {
            return Self::ConfigError;
        }

        // Transient errors
        if code_lower == "408" || code_lower == "500" || code_lower == "502"
            || code_lower == "503" || code_lower == "504" || code_lower == "timeout"
            || code_lower == "connection"
            || msg_lower.contains("timeout")
            || msg_lower.contains("connection")
            || msg_lower.contains("network")
            || msg_lower.contains("temporary")
            || msg_lower.contains("server error")
        {
            return Self::Transient;
        }

        Self::Unknown
    }
}

// ============================================================================
// CIRCUIT BREAKER
// ============================================================================

/// Circuit breaker state for integration health
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CircuitBreakerState {
    /// Normal operation - requests allowed
    Closed,
    /// Failures exceeded threshold - requests blocked
    Open,
    /// Testing if service recovered - limited requests allowed
    HalfOpen,
}

impl CircuitBreakerState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Closed => "closed",
            Self::Open => "open",
            Self::HalfOpen => "half_open",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "open" => Self::Open,
            "half_open" => Self::HalfOpen,
            _ => Self::Closed,
        }
    }
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self::Closed
    }
}

// ============================================================================
// INTEGRATION TYPES
// ============================================================================

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
    // OAuth fields
    pub auth_method: String,
    pub oauth_access_token: Option<String>,
    pub oauth_refresh_token: Option<String>,
    pub oauth_token_expires_at: Option<DateTime<Utc>>,
    pub oauth_scopes: Option<Vec<String>>,
    pub oauth_metadata: Option<serde_json::Value>,
    // Retry configuration
    pub retry_enabled: bool,
    pub max_retry_attempts: i32,
    pub retry_backoff_base_ms: i32,
    pub retry_backoff_max_ms: i32,
    pub circuit_breaker_threshold: i32,
    pub circuit_breaker_reset_ms: i32,
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

    pub fn get_auth_method(&self) -> AuthMethod {
        AuthMethod::from_str(&self.auth_method)
    }

    /// Check if OAuth tokens need refresh (within 5 minutes of expiry)
    pub fn needs_token_refresh(&self) -> bool {
        if self.auth_method != "oauth2" {
            return false;
        }
        match self.oauth_token_expires_at {
            Some(expires_at) => {
                let refresh_threshold = chrono::Duration::minutes(5);
                Utc::now() + refresh_threshold >= expires_at
            }
            None => false,
        }
    }

    /// Check if OAuth tokens are expired
    pub fn is_token_expired(&self) -> bool {
        if self.auth_method != "oauth2" {
            return false;
        }
        match self.oauth_token_expires_at {
            Some(expires_at) => Utc::now() >= expires_at,
            None => false,
        }
    }
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

/// Integration sync log with retry tracking
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
    // Retry tracking
    pub retry_attempt: i32,
    pub max_retries: i32,
    pub error_category: Option<String>,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub retry_backoff_ms: Option<i32>,
    pub parent_sync_id: Option<Uuid>,
}

impl IntegrationSyncLog {
    pub fn get_error_category(&self) -> Option<SyncErrorCategory> {
        self.error_category.as_ref().map(|c| SyncErrorCategory::from_str(c))
    }

    /// Check if this sync can be retried
    pub fn can_retry(&self) -> bool {
        if self.retry_attempt >= self.max_retries {
            return false;
        }
        match self.get_error_category() {
            Some(category) => category.should_retry(),
            None => true, // Default to allowing retry if no category
        }
    }

    /// Get duration of this sync in milliseconds
    pub fn duration_ms(&self) -> Option<i64> {
        self.completed_at.map(|completed| {
            (completed - self.started_at).num_milliseconds()
        })
    }
}

// ============================================================================
// OAUTH STATE
// ============================================================================

/// OAuth state for CSRF protection during authorization flow
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IntegrationOAuthState {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub integration_type: String,
    pub state: String,
    pub code_verifier: Option<String>,
    pub redirect_uri: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub metadata: Option<serde_json::Value>,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Request to start OAuth flow
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthAuthorizeRequest {
    pub integration_name: Option<String>,
    pub scopes: Option<Vec<String>>,
    pub redirect_uri: Option<String>,
}

/// Response with OAuth authorization URL
#[derive(Debug, Clone, Serialize)]
pub struct OAuthAuthorizeResponse {
    pub authorization_url: String,
    pub state: String,
    pub expires_at: DateTime<Utc>,
}

/// OAuth callback parameters
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthCallbackParams {
    pub code: String,
    pub state: String,
    pub error: Option<String>,
    pub error_description: Option<String>,
}

/// OAuth token response from provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: Option<i64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Request to refresh OAuth tokens
#[derive(Debug, Clone, Deserialize)]
pub struct OAuthRefreshRequest {
    pub force: Option<bool>,
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
    /// Supported authentication methods (api_key, oauth2, service_account)
    pub auth_methods: Vec<String>,
    /// OAuth2 specific configuration (if supported)
    pub oauth_config: Option<OAuthProviderConfig>,
}

/// OAuth provider configuration for available integrations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderConfig {
    /// OAuth2 scopes required for this integration
    pub scopes: Vec<String>,
    /// Whether PKCE is required
    pub pkce_required: bool,
    /// Additional authorization URL parameters
    pub extra_params: Option<serde_json::Value>,
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
    // Circuit breaker fields
    pub circuit_breaker_state: String,
    pub circuit_breaker_opened_at: Option<DateTime<Utc>>,
    pub circuit_breaker_half_open_at: Option<DateTime<Utc>>,
}

impl IntegrationHealth {
    pub fn get_status(&self) -> HealthStatus {
        HealthStatus::from_str(&self.status)
    }

    pub fn get_circuit_breaker_state(&self) -> CircuitBreakerState {
        CircuitBreakerState::from_str(&self.circuit_breaker_state)
    }

    /// Check if circuit breaker is allowing requests
    pub fn is_circuit_open(&self) -> bool {
        self.circuit_breaker_state == "open"
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
            auth_methods: vec!["api_key".to_string()],
            oauth_config: None,
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
                "required": [],
                "properties": {
                    "access_token": { "type": "string", "title": "Personal Access Token", "secret": true },
                    "organization": { "type": "string", "title": "Organization (optional)" },
                    "include_private": { "type": "boolean", "title": "Include Private Repos", "default": true }
                }
            }),
            logo_url: Some("/integrations/github.svg".to_string()),
            auth_methods: vec!["api_key".to_string(), "oauth2".to_string()],
            oauth_config: Some(OAuthProviderConfig {
                scopes: vec![
                    "read:user".to_string(),
                    "read:org".to_string(),
                    "repo".to_string(),
                    "security_events".to_string(),
                ],
                pkce_required: false,
                extra_params: None,
            }),
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
                "required": ["domain"],
                "properties": {
                    "domain": { "type": "string", "title": "Okta Domain", "placeholder": "your-org.okta.com" },
                    "api_token": { "type": "string", "title": "API Token", "secret": true }
                }
            }),
            logo_url: Some("/integrations/okta.svg".to_string()),
            auth_methods: vec!["api_key".to_string(), "oauth2".to_string()],
            oauth_config: Some(OAuthProviderConfig {
                scopes: vec![
                    "okta.users.read".to_string(),
                    "okta.groups.read".to_string(),
                    "okta.apps.read".to_string(),
                    "okta.logs.read".to_string(),
                ],
                pkce_required: true,
                extra_params: None,
            }),
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
                "required": ["project_id"],
                "properties": {
                    "service_account_json": { "type": "string", "title": "Service Account JSON", "secret": true, "multiline": true },
                    "project_id": { "type": "string", "title": "Project ID" }
                }
            }),
            logo_url: Some("/integrations/gcp.svg".to_string()),
            auth_methods: vec!["service_account".to_string(), "oauth2".to_string()],
            oauth_config: Some(OAuthProviderConfig {
                scopes: vec![
                    "https://www.googleapis.com/auth/cloud-platform.read-only".to_string(),
                    "https://www.googleapis.com/auth/cloudplatformprojects.readonly".to_string(),
                ],
                pkce_required: true,
                extra_params: Some(serde_json::json!({
                    "access_type": "offline",
                    "prompt": "consent"
                })),
            }),
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
                "required": ["tenant_id"],
                "properties": {
                    "tenant_id": { "type": "string", "title": "Tenant ID" },
                    "client_id": { "type": "string", "title": "Client ID" },
                    "client_secret": { "type": "string", "title": "Client Secret", "secret": true },
                    "subscription_id": { "type": "string", "title": "Subscription ID (optional)" }
                }
            }),
            logo_url: Some("/integrations/azure.svg".to_string()),
            auth_methods: vec!["api_key".to_string(), "oauth2".to_string()],
            oauth_config: Some(OAuthProviderConfig {
                scopes: vec![
                    "https://management.azure.com/.default".to_string(),
                ],
                pkce_required: true,
                extra_params: None,
            }),
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
                "required": ["admin_email"],
                "properties": {
                    "service_account_json": { "type": "string", "title": "Service Account JSON", "secret": true, "multiline": true },
                    "admin_email": { "type": "string", "title": "Admin Email" },
                    "customer_id": { "type": "string", "title": "Customer ID (optional)" }
                }
            }),
            logo_url: Some("/integrations/google-workspace.svg".to_string()),
            auth_methods: vec!["service_account".to_string(), "oauth2".to_string()],
            oauth_config: Some(OAuthProviderConfig {
                scopes: vec![
                    "https://www.googleapis.com/auth/admin.directory.user.readonly".to_string(),
                    "https://www.googleapis.com/auth/admin.directory.group.readonly".to_string(),
                    "https://www.googleapis.com/auth/admin.reports.audit.readonly".to_string(),
                ],
                pkce_required: true,
                extra_params: Some(serde_json::json!({
                    "access_type": "offline",
                    "prompt": "consent"
                })),
            }),
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
                "required": ["tenant_id"],
                "properties": {
                    "tenant_id": { "type": "string", "title": "Tenant ID" },
                    "client_id": { "type": "string", "title": "Client ID" },
                    "client_secret": { "type": "string", "title": "Client Secret", "secret": true }
                }
            }),
            logo_url: Some("/integrations/azure-ad.svg".to_string()),
            auth_methods: vec!["api_key".to_string(), "oauth2".to_string()],
            oauth_config: Some(OAuthProviderConfig {
                scopes: vec![
                    "https://graph.microsoft.com/.default".to_string(),
                ],
                pkce_required: true,
                extra_params: None,
            }),
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
                "required": [],
                "properties": {
                    "access_token": { "type": "string", "title": "Personal Access Token", "secret": true },
                    "base_url": { "type": "string", "title": "GitLab URL", "default": "https://gitlab.com" },
                    "group_id": { "type": "string", "title": "Group ID (optional)" }
                }
            }),
            logo_url: Some("/integrations/gitlab.svg".to_string()),
            auth_methods: vec!["api_key".to_string(), "oauth2".to_string()],
            oauth_config: Some(OAuthProviderConfig {
                scopes: vec![
                    "read_user".to_string(),
                    "read_api".to_string(),
                    "read_repository".to_string(),
                ],
                pkce_required: true,
                extra_params: None,
            }),
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
                "required": ["base_url"],
                "properties": {
                    "base_url": { "type": "string", "title": "Jira URL", "placeholder": "https://your-org.atlassian.net" },
                    "email": { "type": "string", "title": "Email" },
                    "api_token": { "type": "string", "title": "API Token", "secret": true },
                    "project_keys": { "type": "string", "title": "Project Keys (comma-separated)" }
                }
            }),
            logo_url: Some("/integrations/jira.svg".to_string()),
            auth_methods: vec!["api_key".to_string(), "oauth2".to_string()],
            oauth_config: Some(OAuthProviderConfig {
                scopes: vec![
                    "read:jira-work".to_string(),
                    "read:jira-user".to_string(),
                ],
                pkce_required: true,
                extra_params: Some(serde_json::json!({
                    "audience": "api.atlassian.com",
                    "prompt": "consent"
                })),
            }),
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
            auth_methods: vec!["api_key".to_string()],
            oauth_config: None,
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
            auth_methods: vec!["api_key".to_string()],
            oauth_config: None,
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
            auth_methods: vec!["api_key".to_string()],
            oauth_config: None,
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
            auth_methods: vec!["api_key".to_string()],
            oauth_config: None,
        },
    ]
}
