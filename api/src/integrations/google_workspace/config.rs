use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Google Workspace integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleWorkspaceConfig {
    /// Service account email or OAuth access token
    #[serde(default)]
    pub auth_type: GoogleAuthType,
    /// Service account JSON key (for service account auth)
    pub service_account_key: Option<String>,
    /// OAuth access token (for OAuth auth)
    pub access_token: Option<String>,
    /// OAuth refresh token (for OAuth auth)
    pub refresh_token: Option<String>,
    /// Customer ID (usually "my_customer" for the primary domain)
    #[serde(default = "default_customer_id")]
    pub customer_id: String,
    /// Domain to query
    pub domain: Option<String>,
    /// Admin user email (for service account impersonation)
    pub admin_email: Option<String>,
    /// Services to enable
    #[serde(default)]
    pub services: GoogleWorkspaceServicesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum GoogleAuthType {
    #[default]
    ServiceAccount,
    OAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleWorkspaceServicesConfig {
    /// Sync user directory
    #[serde(default = "default_true")]
    pub users: bool,
    /// Sync groups
    #[serde(default = "default_true")]
    pub groups: bool,
    /// Sync 2-step verification status
    #[serde(default = "default_true")]
    pub two_step_verification: bool,
    /// Sync login audit logs
    #[serde(default = "default_true")]
    pub login_audit: bool,
    /// Sync admin audit logs
    #[serde(default = "default_true")]
    pub admin_audit: bool,
    /// Number of days of logs to collect
    #[serde(default = "default_log_days")]
    pub log_days: u32,
}

impl Default for GoogleWorkspaceServicesConfig {
    fn default() -> Self {
        Self {
            users: true,
            groups: true,
            two_step_verification: true,
            login_audit: true,
            admin_audit: true,
            log_days: 7,
        }
    }
}

fn default_true() -> bool {
    true
}

fn default_log_days() -> u32 {
    7
}

fn default_customer_id() -> String {
    "my_customer".to_string()
}

impl GoogleWorkspaceConfig {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        serde_json::from_value(value.clone())
            .map_err(|e| format!("Invalid Google Workspace configuration: {}", e))
    }

    pub fn validate(&self) -> Result<(), String> {
        match self.auth_type {
            GoogleAuthType::ServiceAccount => {
                if self.service_account_key.is_none() || self.service_account_key.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    return Err("Service account key is required for service account authentication".to_string());
                }
                if self.admin_email.is_none() || self.admin_email.as_ref().map(|s| s.is_empty()).unwrap_or(true) {
                    return Err("Admin email is required for service account impersonation".to_string());
                }
            }
            GoogleAuthType::OAuth => {
                if (self.access_token.is_none() || self.access_token.as_ref().map(|s| s.is_empty()).unwrap_or(true))
                    && (self.refresh_token.is_none() || self.refresh_token.as_ref().map(|s| s.is_empty()).unwrap_or(true))
                {
                    return Err("Access token or refresh token is required for OAuth authentication".to_string());
                }
            }
        }
        Ok(())
    }
}
