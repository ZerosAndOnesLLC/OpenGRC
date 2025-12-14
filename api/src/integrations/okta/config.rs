use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Okta integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaConfig {
    /// Okta domain (e.g., "dev-123456.okta.com" or "your-company.okta.com")
    pub domain: String,
    /// API token for authentication
    pub api_token: String,
    /// Services to enable
    #[serde(default)]
    pub services: OktaServicesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaServicesConfig {
    /// Sync user directory
    #[serde(default = "default_true")]
    pub users: bool,
    /// Sync user MFA status
    #[serde(default = "default_true")]
    pub mfa: bool,
    /// Sync application assignments
    #[serde(default = "default_true")]
    pub applications: bool,
    /// Sync system logs
    #[serde(default = "default_true")]
    pub logs: bool,
    /// Sync groups
    #[serde(default = "default_true")]
    pub groups: bool,
    /// Number of days of logs to collect
    #[serde(default = "default_log_days")]
    pub log_days: u32,
}

impl Default for OktaServicesConfig {
    fn default() -> Self {
        Self {
            users: true,
            mfa: true,
            applications: true,
            logs: true,
            groups: true,
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

impl OktaConfig {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        serde_json::from_value(value.clone())
            .map_err(|e| format!("Invalid Okta configuration: {}", e))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.domain.is_empty() {
            return Err("Domain is required".to_string());
        }
        if self.api_token.is_empty() {
            return Err("API token is required".to_string());
        }
        // Validate domain format
        if !self.domain.contains("okta.com") && !self.domain.contains("oktapreview.com") {
            return Err("Domain must be an Okta domain (e.g., your-company.okta.com)".to_string());
        }
        Ok(())
    }

    /// Get the base URL for the Okta API
    pub fn base_url(&self) -> String {
        format!("https://{}", self.domain)
    }
}
