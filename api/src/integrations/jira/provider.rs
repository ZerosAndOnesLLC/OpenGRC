use super::client::JiraClient;
use super::config::JiraConfig;
use crate::integrations::{
    IntegrationCapability, IntegrationProvider, SyncContext, SyncResult, TestConnectionDetails,
};
use async_trait::async_trait;
use serde_json::Value;

/// Jira integration provider
pub struct JiraProvider;

impl JiraProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for JiraProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationProvider for JiraProvider {
    fn integration_type(&self) -> &'static str {
        "jira"
    }

    fn capabilities(&self) -> Vec<IntegrationCapability> {
        vec![
            IntegrationCapability::AssetInventory,      // Projects
            IntegrationCapability::ConfigurationState, // Project settings, permissions
            IntegrationCapability::UserSync,           // Project users
        ]
    }

    fn validate_config(&self, config: &Value) -> Result<(), String> {
        let jira_config = JiraConfig::from_value(config)?;
        jira_config.validate()
    }

    async fn test_connection(&self, config: &Value) -> Result<TestConnectionDetails, String> {
        let jira_config = JiraConfig::from_value(config)?;
        jira_config.validate()?;

        // Create Jira client and test connection
        let client = JiraClient::new(jira_config.clone()).await?;

        // Get authenticated user to verify credentials
        let user = client.get_myself().await?;

        // Get server info
        let server_info = client.get_server_info().await?;

        // Build permissions list based on what services are enabled
        let mut permissions = vec!["user:read".to_string()];

        if jira_config.services.projects {
            permissions.push("project:read".to_string());
        }
        if jira_config.services.issues {
            permissions.push("issue:read".to_string());
        }
        if jira_config.services.users {
            permissions.push("user:search".to_string());
        }
        if jira_config.services.permissions {
            permissions.push("project:admin".to_string());
        }

        Ok(TestConnectionDetails::success(format!(
            "Successfully connected to {} as {}",
            server_info.server_title.unwrap_or_else(|| "Jira".to_string()),
            user.display_name
        ))
        .with_account_info(serde_json::json!({
            "account_id": user.account_id,
            "display_name": user.display_name,
            "email": user.email_address,
            "active": user.active,
            "instance_url": jira_config.instance_url,
            "server_version": server_info.version,
            "deployment_type": server_info.deployment_type,
            "services_enabled": {
                "projects": jira_config.services.projects,
                "issues": jira_config.services.issues,
                "users": jira_config.services.users,
                "permissions": jira_config.services.permissions,
            }
        }))
        .with_permissions(permissions))
    }

    async fn sync(&self, config: &Value, context: SyncContext) -> Result<SyncResult, String> {
        let jira_config = JiraConfig::from_value(config)?;
        jira_config.validate()?;

        // Create Jira client
        let client = JiraClient::new(jira_config.clone()).await?;

        // Get authenticated user for logging
        let user = client.get_myself().await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            jira_user = %user.display_name,
            instance_url = %jira_config.instance_url,
            full_sync = context.full_sync,
            "Starting Jira sync"
        );

        // Run the sync orchestrator
        let result = super::sync::run_sync(&client, &jira_config, &context).await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            jira_user = %user.display_name,
            records_processed = result.records_processed,
            records_created = result.records_created,
            errors = result.errors.len(),
            evidence_collected = result.evidence_collected.len(),
            "Completed Jira sync"
        );

        Ok(result)
    }

    fn required_fields(&self) -> Vec<&'static str> {
        vec!["instance_url", "access_token"]
    }

    fn optional_fields(&self) -> Vec<&'static str> {
        vec!["email", "auth_method", "projects", "services"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_type() {
        let provider = JiraProvider::new();
        assert_eq!(provider.integration_type(), "jira");
    }

    #[test]
    fn test_capabilities() {
        let provider = JiraProvider::new();
        let caps = provider.capabilities();
        assert!(caps.contains(&IntegrationCapability::AssetInventory));
        assert!(caps.contains(&IntegrationCapability::ConfigurationState));
        assert!(caps.contains(&IntegrationCapability::UserSync));
    }

    #[test]
    fn test_validate_config_missing_url() {
        let provider = JiraProvider::new();
        let config = serde_json::json!({
            "access_token": "test-token"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_valid_oauth() {
        let provider = JiraProvider::new();
        let config = serde_json::json!({
            "instance_url": "https://example.atlassian.net",
            "access_token": "oauth-token",
            "auth_method": "oauth"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_api_token_needs_email() {
        let provider = JiraProvider::new();
        let config = serde_json::json!({
            "instance_url": "https://example.atlassian.net",
            "access_token": "api-token",
            "auth_method": "api_token"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_valid_api_token() {
        let provider = JiraProvider::new();
        let config = serde_json::json!({
            "instance_url": "https://example.atlassian.net",
            "access_token": "api-token",
            "email": "user@example.com",
            "auth_method": "api_token"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_ok());
    }
}
