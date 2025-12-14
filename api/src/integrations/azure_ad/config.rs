use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Azure AD (Microsoft Entra ID) integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureAdConfig {
    /// Azure AD tenant ID
    pub tenant_id: String,
    /// Application (client) ID
    pub client_id: String,
    /// Client secret (for client credentials flow)
    pub client_secret: Option<String>,
    /// OAuth access token (for OAuth auth)
    pub access_token: Option<String>,
    /// Services to enable
    #[serde(default)]
    pub services: AzureAdServicesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureAdServicesConfig {
    /// Sync users
    #[serde(default = "default_true")]
    pub users: bool,
    /// Sync groups
    #[serde(default = "default_true")]
    pub groups: bool,
    /// Sync conditional access policies
    #[serde(default = "default_true")]
    pub conditional_access: bool,
    /// Sync sign-in logs
    #[serde(default = "default_true")]
    pub sign_in_logs: bool,
    /// Sync audit logs
    #[serde(default = "default_true")]
    pub audit_logs: bool,
    /// Number of days of logs to collect
    #[serde(default = "default_log_days")]
    pub log_days: u32,
}

impl Default for AzureAdServicesConfig {
    fn default() -> Self {
        Self {
            users: true,
            groups: true,
            conditional_access: true,
            sign_in_logs: true,
            audit_logs: true,
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

impl AzureAdConfig {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        serde_json::from_value(value.clone())
            .map_err(|e| format!("Invalid Azure AD configuration: {}", e))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.tenant_id.is_empty() {
            return Err("Tenant ID is required".to_string());
        }
        if self.client_id.is_empty() {
            return Err("Client ID is required".to_string());
        }
        // Either client_secret (for app auth) or access_token (for OAuth) required
        if self.client_secret.is_none() && self.access_token.is_none() {
            return Err("Either client secret or access token is required".to_string());
        }
        if let Some(ref secret) = self.client_secret {
            if secret.is_empty() {
                if self.access_token.is_none() {
                    return Err("Either client secret or access token is required".to_string());
                }
            }
        }
        Ok(())
    }

    /// Get the token endpoint URL
    pub fn token_url(&self) -> String {
        format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.tenant_id
        )
    }
}
