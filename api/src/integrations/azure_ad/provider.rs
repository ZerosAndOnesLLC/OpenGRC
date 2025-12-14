use super::client::AzureAdClient;
use super::config::AzureAdConfig;
use crate::integrations::{
    IntegrationCapability, IntegrationProvider, SyncContext, SyncResult, TestConnectionDetails,
};
use async_trait::async_trait;
use serde_json::Value;

/// Azure AD (Microsoft Entra ID) integration provider
pub struct AzureAdProvider;

impl AzureAdProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AzureAdProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationProvider for AzureAdProvider {
    fn integration_type(&self) -> &'static str {
        "azure_ad"
    }

    fn capabilities(&self) -> Vec<IntegrationCapability> {
        vec![
            IntegrationCapability::UserSync,           // User directory
            IntegrationCapability::AccessSync,         // Groups and memberships
            IntegrationCapability::AuditLogs,          // Sign-in and audit logs
            IntegrationCapability::ComplianceStatus,   // Conditional access policies
        ]
    }

    fn validate_config(&self, config: &Value) -> Result<(), String> {
        let azure_config = AzureAdConfig::from_value(config)?;
        azure_config.validate()
    }

    async fn test_connection(&self, config: &Value) -> Result<TestConnectionDetails, String> {
        let azure_config = AzureAdConfig::from_value(config)?;
        azure_config.validate()?;

        // Create Azure AD client and test connection
        let client = AzureAdClient::new(azure_config.clone()).await?;

        // Get organization info to verify credentials
        let org = client.get_organization().await?;

        // Build permissions list based on what services are enabled
        let mut permissions = vec!["Organization.Read.All".to_string()];

        if azure_config.services.users {
            permissions.push("User.Read.All".to_string());
        }
        if azure_config.services.groups {
            permissions.push("Group.Read.All".to_string());
        }
        if azure_config.services.conditional_access {
            permissions.push("Policy.Read.All".to_string());
        }
        if azure_config.services.sign_in_logs {
            permissions.push("AuditLog.Read.All".to_string());
        }
        if azure_config.services.audit_logs {
            permissions.push("AuditLog.Read.All".to_string());
        }

        let primary_domain = org.verified_domains
            .as_ref()
            .and_then(|d| d.iter().find(|v| v.is_default.unwrap_or(false)))
            .and_then(|d| d.name.clone())
            .unwrap_or_else(|| org.display_name.clone().unwrap_or_default());

        Ok(TestConnectionDetails::success(format!(
            "Successfully connected to {}",
            primary_domain
        ))
        .with_account_info(serde_json::json!({
            "tenant_id": org.id,
            "display_name": org.display_name,
            "verified_domains": org.verified_domains,
            "tenant_type": org.tenant_type,
            "created_date_time": org.created_date_time,
            "services_enabled": {
                "users": azure_config.services.users,
                "groups": azure_config.services.groups,
                "conditional_access": azure_config.services.conditional_access,
                "sign_in_logs": azure_config.services.sign_in_logs,
                "audit_logs": azure_config.services.audit_logs,
                "log_days": azure_config.services.log_days,
            }
        }))
        .with_permissions(permissions))
    }

    async fn sync(&self, config: &Value, context: SyncContext) -> Result<SyncResult, String> {
        let azure_config = AzureAdConfig::from_value(config)?;
        azure_config.validate()?;

        // Create Azure AD client
        let client = AzureAdClient::new(azure_config.clone()).await?;

        // Get organization info for logging
        let org = client.get_organization().await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            azure_tenant = ?org.display_name,
            full_sync = context.full_sync,
            "Starting Azure AD sync"
        );

        // Run the sync orchestrator
        let result = super::sync::run_sync(&client, &azure_config, &context).await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            azure_tenant = ?org.display_name,
            records_processed = result.records_processed,
            records_created = result.records_created,
            errors = result.errors.len(),
            evidence_collected = result.evidence_collected.len(),
            "Completed Azure AD sync"
        );

        Ok(result)
    }

    fn required_fields(&self) -> Vec<&'static str> {
        vec!["tenant_id", "client_id"]
    }

    fn optional_fields(&self) -> Vec<&'static str> {
        vec!["client_secret", "access_token", "services"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_type() {
        let provider = AzureAdProvider::new();
        assert_eq!(provider.integration_type(), "azure_ad");
    }

    #[test]
    fn test_capabilities() {
        let provider = AzureAdProvider::new();
        let caps = provider.capabilities();
        assert!(caps.contains(&IntegrationCapability::UserSync));
        assert!(caps.contains(&IntegrationCapability::AccessSync));
        assert!(caps.contains(&IntegrationCapability::AuditLogs));
        assert!(caps.contains(&IntegrationCapability::ComplianceStatus));
    }

    #[test]
    fn test_validate_config_missing_tenant() {
        let provider = AzureAdProvider::new();
        let config = serde_json::json!({
            "client_id": "test-client-id",
            "client_secret": "test-secret"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_missing_auth() {
        let provider = AzureAdProvider::new();
        let config = serde_json::json!({
            "tenant_id": "test-tenant-id",
            "client_id": "test-client-id"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_valid() {
        let provider = AzureAdProvider::new();
        let config = serde_json::json!({
            "tenant_id": "test-tenant-id",
            "client_id": "test-client-id",
            "client_secret": "test-secret"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_ok());
    }
}
