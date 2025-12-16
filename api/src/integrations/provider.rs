use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Capabilities that an integration can provide
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntegrationCapability {
    /// Can sync user/identity data
    UserSync,
    /// Can sync access permissions
    AccessSync,
    /// Can collect audit logs
    AuditLogs,
    /// Can collect security findings
    SecurityFindings,
    /// Can collect compliance status
    ComplianceStatus,
    /// Can collect asset inventory
    AssetInventory,
    /// Can collect configuration state
    ConfigurationState,
    /// Can receive webhooks for real-time updates
    Webhooks,
}

/// Context passed to sync operations
#[derive(Debug, Clone)]
pub struct SyncContext {
    pub organization_id: Uuid,
    pub integration_id: Uuid,
    pub sync_log_id: Uuid,
    pub full_sync: bool,
    pub sync_type: Option<String>,
}

/// Result of a sync operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub success: bool,
    pub records_processed: i32,
    pub records_created: i32,
    pub records_updated: i32,
    pub records_deleted: i32,
    pub errors: Vec<SyncError>,
    pub evidence_collected: Vec<CollectedEvidence>,
    /// Security alert data for CloudTrail events
    pub security_alerts: Option<SecurityAlertInfo>,
}

/// Security alert information collected during sync
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityAlertInfo {
    pub root_actions: Vec<serde_json::Value>,
    pub sensitive_actions: Vec<serde_json::Value>,
    pub failed_actions: Vec<serde_json::Value>,
    pub critical_findings_count: i32,
    pub high_findings_count: i32,
    pub critical_findings: Vec<serde_json::Value>,
}

impl Default for SyncResult {
    fn default() -> Self {
        Self {
            success: true,
            records_processed: 0,
            records_created: 0,
            records_updated: 0,
            records_deleted: 0,
            errors: Vec::new(),
            evidence_collected: Vec::new(),
            security_alerts: None,
        }
    }
}

impl SyncResult {
    pub fn with_error(mut self, error: SyncError) -> Self {
        self.errors.push(error);
        self.success = false;
        self
    }

    pub fn merge(&mut self, other: SyncResult) {
        self.records_processed += other.records_processed;
        self.records_created += other.records_created;
        self.records_updated += other.records_updated;
        self.records_deleted += other.records_deleted;
        self.errors.extend(other.errors);
        self.evidence_collected.extend(other.evidence_collected);
        if !other.success {
            self.success = false;
        }
        // Merge security alerts
        if let Some(other_alerts) = other.security_alerts {
            if let Some(ref mut alerts) = self.security_alerts {
                alerts.root_actions.extend(other_alerts.root_actions);
                alerts.sensitive_actions.extend(other_alerts.sensitive_actions);
                alerts.failed_actions.extend(other_alerts.failed_actions);
                alerts.critical_findings_count += other_alerts.critical_findings_count;
                alerts.high_findings_count += other_alerts.high_findings_count;
                alerts.critical_findings.extend(other_alerts.critical_findings);
            } else {
                self.security_alerts = Some(other_alerts);
            }
        }
    }
}

/// Error that occurred during sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    pub code: String,
    pub message: String,
    pub resource: Option<String>,
    pub recoverable: bool,
    /// Error category for retry logic
    pub category: Option<String>,
}

impl SyncError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        let code_str = code.into();
        let message_str = message.into();
        let category = crate::models::SyncErrorCategory::classify(&code_str, &message_str);

        Self {
            code: code_str,
            message: message_str,
            resource: None,
            recoverable: category.should_retry(),
            category: Some(category.as_str().to_string()),
        }
    }

    pub fn recoverable(mut self) -> Self {
        self.recoverable = true;
        self
    }

    pub fn not_recoverable(mut self) -> Self {
        self.recoverable = false;
        self.category = Some("permanent".to_string());
        self
    }

    pub fn with_resource(mut self, resource: impl Into<String>) -> Self {
        self.resource = Some(resource.into());
        self
    }

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Create a rate limited error
    pub fn rate_limited(message: impl Into<String>) -> Self {
        Self {
            code: "429".to_string(),
            message: message.into(),
            resource: None,
            recoverable: true,
            category: Some("rate_limited".to_string()),
        }
    }

    /// Create an authentication error
    pub fn auth_failure(message: impl Into<String>) -> Self {
        Self {
            code: "401".to_string(),
            message: message.into(),
            resource: None,
            recoverable: false, // Needs manual re-auth
            category: Some("auth_failure".to_string()),
        }
    }

    /// Create a transient error (network, timeout)
    pub fn transient(message: impl Into<String>) -> Self {
        Self {
            code: "transient".to_string(),
            message: message.into(),
            resource: None,
            recoverable: true,
            category: Some("transient".to_string()),
        }
    }

    /// Create a configuration error
    pub fn config_error(message: impl Into<String>) -> Self {
        Self {
            code: "config_error".to_string(),
            message: message.into(),
            resource: None,
            recoverable: false,
            category: Some("config_error".to_string()),
        }
    }
}

/// Evidence collected during sync
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectedEvidence {
    pub title: String,
    pub description: Option<String>,
    pub evidence_type: String,
    pub source: String,
    pub source_reference: Option<String>,
    pub data: Value,
    pub control_codes: Vec<String>,
}

/// Trait that all integration providers must implement
#[async_trait]
pub trait IntegrationProvider: Send + Sync {
    /// Get the integration type identifier
    fn integration_type(&self) -> &'static str;

    /// Get the capabilities this integration provides
    fn capabilities(&self) -> Vec<IntegrationCapability>;

    /// Validate the configuration
    fn validate_config(&self, config: &Value) -> Result<(), String>;

    /// Test the connection with the given configuration
    async fn test_connection(&self, config: &Value) -> Result<TestConnectionDetails, String>;

    /// Run a sync operation
    async fn sync(&self, config: &Value, context: SyncContext) -> Result<SyncResult, String>;

    /// Get the required configuration fields
    fn required_fields(&self) -> Vec<&'static str>;

    /// Get optional configuration fields
    fn optional_fields(&self) -> Vec<&'static str> {
        vec![]
    }
}

/// Details returned from a test connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConnectionDetails {
    pub connected: bool,
    pub message: String,
    pub account_info: Option<Value>,
    pub permissions: Option<Vec<String>>,
}

impl TestConnectionDetails {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            connected: true,
            message: message.into(),
            account_info: None,
            permissions: None,
        }
    }

    pub fn failure(message: impl Into<String>) -> Self {
        Self {
            connected: false,
            message: message.into(),
            account_info: None,
            permissions: None,
        }
    }

    pub fn with_account_info(mut self, info: Value) -> Self {
        self.account_info = Some(info);
        self
    }

    pub fn with_permissions(mut self, perms: Vec<String>) -> Self {
        self.permissions = Some(perms);
        self
    }
}

/// Registry of integration providers
pub struct IntegrationRegistry {
    providers: std::collections::HashMap<String, Box<dyn IntegrationProvider>>,
}

impl IntegrationRegistry {
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
        }
    }

    pub fn register(&mut self, provider: Box<dyn IntegrationProvider>) {
        let type_name = provider.integration_type().to_string();
        self.providers.insert(type_name, provider);
    }

    pub fn get(&self, integration_type: &str) -> Option<&dyn IntegrationProvider> {
        self.providers.get(integration_type).map(|p| p.as_ref())
    }

    pub fn list_types(&self) -> Vec<&str> {
        self.providers.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for IntegrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}
