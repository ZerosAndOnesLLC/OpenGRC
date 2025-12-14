use super::client::GoogleWorkspaceClient;
use super::config::GoogleWorkspaceConfig;
use crate::integrations::{
    IntegrationCapability, IntegrationProvider, SyncContext, SyncResult, TestConnectionDetails,
};
use async_trait::async_trait;
use serde_json::Value;

/// Google Workspace integration provider
pub struct GoogleWorkspaceProvider;

impl GoogleWorkspaceProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GoogleWorkspaceProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationProvider for GoogleWorkspaceProvider {
    fn integration_type(&self) -> &'static str {
        "google_workspace"
    }

    fn capabilities(&self) -> Vec<IntegrationCapability> {
        vec![
            IntegrationCapability::UserSync,           // User directory
            IntegrationCapability::AccessSync,         // Groups and memberships
            IntegrationCapability::AuditLogs,          // Login and admin audit logs
            IntegrationCapability::ComplianceStatus,   // 2-Step Verification status
        ]
    }

    fn validate_config(&self, config: &Value) -> Result<(), String> {
        let gw_config = GoogleWorkspaceConfig::from_value(config)?;
        gw_config.validate()
    }

    async fn test_connection(&self, config: &Value) -> Result<TestConnectionDetails, String> {
        let gw_config = GoogleWorkspaceConfig::from_value(config)?;
        gw_config.validate()?;

        // Create Google Workspace client and test connection
        let client = GoogleWorkspaceClient::new(gw_config.clone()).await?;

        // Get customer info to verify credentials
        let customer = client.get_customer().await?;

        // Build permissions list based on what services are enabled
        let mut permissions = vec!["admin.directory.customer.readonly".to_string()];

        if gw_config.services.users || gw_config.services.two_step_verification {
            permissions.push("admin.directory.user.readonly".to_string());
        }
        if gw_config.services.groups {
            permissions.push("admin.directory.group.readonly".to_string());
        }
        if gw_config.services.login_audit || gw_config.services.admin_audit {
            permissions.push("admin.reports.audit.readonly".to_string());
        }

        Ok(TestConnectionDetails::success(format!(
            "Successfully connected to {}",
            customer.customer_domain.as_deref().unwrap_or(&gw_config.customer_id)
        ))
        .with_account_info(serde_json::json!({
            "customer_id": customer.id,
            "customer_domain": customer.customer_domain,
            "organization_name": customer.postal_address.as_ref().and_then(|a| a.organization_name.clone()),
            "language": customer.language,
            "customer_creation_time": customer.customer_creation_time,
            "services_enabled": {
                "users": gw_config.services.users,
                "groups": gw_config.services.groups,
                "two_step_verification": gw_config.services.two_step_verification,
                "login_audit": gw_config.services.login_audit,
                "admin_audit": gw_config.services.admin_audit,
                "log_days": gw_config.services.log_days,
            }
        }))
        .with_permissions(permissions))
    }

    async fn sync(&self, config: &Value, context: SyncContext) -> Result<SyncResult, String> {
        let gw_config = GoogleWorkspaceConfig::from_value(config)?;
        gw_config.validate()?;

        // Create Google Workspace client
        let client = GoogleWorkspaceClient::new(gw_config.clone()).await?;

        // Get customer info for logging
        let customer = client.get_customer().await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            google_customer = ?customer.customer_domain,
            full_sync = context.full_sync,
            "Starting Google Workspace sync"
        );

        // Run the sync orchestrator
        let result = super::sync::run_sync(&client, &gw_config, &context).await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            google_customer = ?customer.customer_domain,
            records_processed = result.records_processed,
            records_created = result.records_created,
            errors = result.errors.len(),
            evidence_collected = result.evidence_collected.len(),
            "Completed Google Workspace sync"
        );

        Ok(result)
    }

    fn required_fields(&self) -> Vec<&'static str> {
        vec!["auth_type"]
    }

    fn optional_fields(&self) -> Vec<&'static str> {
        vec!["service_account_key", "access_token", "refresh_token", "customer_id", "domain", "admin_email", "services"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_type() {
        let provider = GoogleWorkspaceProvider::new();
        assert_eq!(provider.integration_type(), "google_workspace");
    }

    #[test]
    fn test_capabilities() {
        let provider = GoogleWorkspaceProvider::new();
        let caps = provider.capabilities();
        assert!(caps.contains(&IntegrationCapability::UserSync));
        assert!(caps.contains(&IntegrationCapability::AccessSync));
        assert!(caps.contains(&IntegrationCapability::AuditLogs));
        assert!(caps.contains(&IntegrationCapability::ComplianceStatus));
    }

    #[test]
    fn test_validate_config_service_account_missing_key() {
        let provider = GoogleWorkspaceProvider::new();
        let config = serde_json::json!({
            "auth_type": "service_account",
            "admin_email": "admin@example.com"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_oauth_missing_tokens() {
        let provider = GoogleWorkspaceProvider::new();
        let config = serde_json::json!({
            "auth_type": "oauth"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_oauth_valid() {
        let provider = GoogleWorkspaceProvider::new();
        let config = serde_json::json!({
            "auth_type": "oauth",
            "access_token": "ya29.xxx"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_ok());
    }
}
