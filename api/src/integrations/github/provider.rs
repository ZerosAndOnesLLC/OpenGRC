use super::client::GitHubClient;
use super::config::GitHubConfig;
use crate::integrations::{
    IntegrationCapability, IntegrationProvider, SyncContext, SyncResult, TestConnectionDetails,
};
use async_trait::async_trait;
use serde_json::Value;

/// GitHub integration provider
pub struct GitHubProvider;

impl GitHubProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitHubProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationProvider for GitHubProvider {
    fn integration_type(&self) -> &'static str {
        "github"
    }

    fn capabilities(&self) -> Vec<IntegrationCapability> {
        vec![
            IntegrationCapability::AssetInventory,      // Repositories
            IntegrationCapability::SecurityFindings,   // Dependabot, code scanning, secret scanning
            IntegrationCapability::ConfigurationState, // Branch protection
            IntegrationCapability::UserSync,           // Organization members
        ]
    }

    fn validate_config(&self, config: &Value) -> Result<(), String> {
        let github_config = GitHubConfig::from_value(config)?;
        github_config.validate()
    }

    async fn test_connection(&self, config: &Value) -> Result<TestConnectionDetails, String> {
        let github_config = GitHubConfig::from_value(config)?;
        github_config.validate()?;

        // Create GitHub client and test connection
        let client = GitHubClient::new(github_config.clone()).await?;

        // Get authenticated user to verify credentials
        let user = client.get_authenticated_user().await?;

        // Build permissions list based on what services are enabled
        let mut permissions = vec!["user:read".to_string()];

        if github_config.services.repositories {
            permissions.push("repo:read".to_string());
        }
        if github_config.services.branch_protection {
            permissions.push("repo:read (admin for protection rules)".to_string());
        }
        if github_config.services.dependabot_alerts {
            permissions.push("security_events:read".to_string());
        }
        if github_config.services.code_scanning {
            permissions.push("security_events:read".to_string());
        }
        if github_config.services.secret_scanning {
            permissions.push("secret_scanning_alerts:read".to_string());
        }
        if github_config.services.members {
            permissions.push("org:read".to_string());
        }

        Ok(TestConnectionDetails::success(format!(
            "Successfully connected as {}",
            user.login
        ))
        .with_account_info(serde_json::json!({
            "login": user.login,
            "id": user.id,
            "name": user.name,
            "email": user.email,
            "avatar_url": user.avatar_url,
            "html_url": user.html_url,
            "type": user.user_type,
            "organization": github_config.organization,
            "services_enabled": {
                "repositories": github_config.services.repositories,
                "branch_protection": github_config.services.branch_protection,
                "dependabot_alerts": github_config.services.dependabot_alerts,
                "code_scanning": github_config.services.code_scanning,
                "secret_scanning": github_config.services.secret_scanning,
                "members": github_config.services.members,
            }
        }))
        .with_permissions(permissions))
    }

    async fn sync(&self, config: &Value, context: SyncContext) -> Result<SyncResult, String> {
        let github_config = GitHubConfig::from_value(config)?;
        github_config.validate()?;

        // Create GitHub client
        let client = GitHubClient::new(github_config.clone()).await?;

        // Get authenticated user for logging
        let user = client.get_authenticated_user().await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            github_user = %user.login,
            github_org = ?github_config.organization,
            full_sync = context.full_sync,
            "Starting GitHub sync"
        );

        // Run the sync orchestrator
        let result = super::sync::run_sync(&client, &github_config, &context).await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            github_user = %user.login,
            records_processed = result.records_processed,
            records_created = result.records_created,
            errors = result.errors.len(),
            evidence_collected = result.evidence_collected.len(),
            "Completed GitHub sync"
        );

        Ok(result)
    }

    fn required_fields(&self) -> Vec<&'static str> {
        vec!["access_token"]
    }

    fn optional_fields(&self) -> Vec<&'static str> {
        vec!["organization", "repositories", "services"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_type() {
        let provider = GitHubProvider::new();
        assert_eq!(provider.integration_type(), "github");
    }

    #[test]
    fn test_capabilities() {
        let provider = GitHubProvider::new();
        let caps = provider.capabilities();
        assert!(caps.contains(&IntegrationCapability::AssetInventory));
        assert!(caps.contains(&IntegrationCapability::SecurityFindings));
        assert!(caps.contains(&IntegrationCapability::ConfigurationState));
        assert!(caps.contains(&IntegrationCapability::UserSync));
    }

    #[test]
    fn test_validate_config_missing_token() {
        let provider = GitHubProvider::new();
        let config = serde_json::json!({});
        let result = provider.validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_valid() {
        let provider = GitHubProvider::new();
        let config = serde_json::json!({
            "access_token": "ghp_xxxxxxxxxxxx",
            "organization": "my-org"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_ok());
    }
}
