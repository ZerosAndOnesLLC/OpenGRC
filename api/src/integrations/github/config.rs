use serde::{Deserialize, Serialize};
use serde_json::Value;

/// GitHub integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// Personal access token or OAuth token
    pub access_token: String,
    /// Organization name (optional - if not set, uses authenticated user's repos)
    pub organization: Option<String>,
    /// Specific repositories to sync (comma-separated, optional)
    pub repositories: Option<String>,
    /// Services to enable
    #[serde(default)]
    pub services: GitHubServicesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GitHubServicesConfig {
    /// Sync repositories
    #[serde(default = "default_true")]
    pub repositories: bool,
    /// Sync branch protection rules
    #[serde(default = "default_true")]
    pub branch_protection: bool,
    /// Sync Dependabot alerts
    #[serde(default = "default_true")]
    pub dependabot_alerts: bool,
    /// Sync code scanning alerts
    #[serde(default = "default_true")]
    pub code_scanning: bool,
    /// Sync secret scanning alerts
    #[serde(default = "default_true")]
    pub secret_scanning: bool,
    /// Sync organization members
    #[serde(default = "default_true")]
    pub members: bool,
}

fn default_true() -> bool {
    true
}

impl GitHubConfig {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        serde_json::from_value(value.clone())
            .map_err(|e| format!("Invalid GitHub configuration: {}", e))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.access_token.is_empty() {
            return Err("Access token is required".to_string());
        }
        Ok(())
    }

    /// Get list of repositories to sync
    pub fn get_repositories(&self) -> Vec<String> {
        self.repositories
            .as_ref()
            .map(|r| r.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
            .unwrap_or_default()
    }
}
