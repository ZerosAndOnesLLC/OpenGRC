use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Jira integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraConfig {
    /// Jira instance URL (e.g., https://your-domain.atlassian.net)
    pub instance_url: String,
    /// API token or OAuth access token
    pub access_token: String,
    /// Email associated with the API token (for Basic Auth)
    pub email: Option<String>,
    /// Authentication method
    #[serde(default)]
    pub auth_method: JiraAuthMethod,
    /// Specific projects to sync (comma-separated, optional)
    pub projects: Option<String>,
    /// Services to enable
    #[serde(default)]
    pub services: JiraServicesConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum JiraAuthMethod {
    /// API Token with email (Basic Auth)
    #[default]
    ApiToken,
    /// OAuth 2.0 access token
    OAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraServicesConfig {
    /// Sync projects
    #[serde(default = "default_true")]
    pub projects: bool,
    /// Sync issues
    #[serde(default = "default_true")]
    pub issues: bool,
    /// Sync users
    #[serde(default = "default_true")]
    pub users: bool,
    /// Sync project permissions
    #[serde(default = "default_true")]
    pub permissions: bool,
}

fn default_true() -> bool {
    true
}

impl JiraConfig {
    pub fn from_value(value: &Value) -> Result<Self, String> {
        serde_json::from_value(value.clone())
            .map_err(|e| format!("Invalid Jira configuration: {}", e))
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.instance_url.is_empty() {
            return Err("Instance URL is required".to_string());
        }

        // Validate URL format
        if !self.instance_url.starts_with("https://") {
            return Err("Instance URL must use HTTPS".to_string());
        }

        if self.access_token.is_empty() {
            return Err("Access token is required".to_string());
        }

        // API token auth requires email
        if self.auth_method == JiraAuthMethod::ApiToken && self.email.is_none() {
            return Err("Email is required for API token authentication".to_string());
        }

        Ok(())
    }

    /// Get list of projects to sync
    pub fn get_projects(&self) -> Vec<String> {
        self.projects
            .as_ref()
            .map(|p| {
                p.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the base API URL
    pub fn api_url(&self) -> String {
        format!("{}/rest/api/3", self.instance_url.trim_end_matches('/'))
    }
}
