use super::client::OktaClient;
use super::config::OktaConfig;
use crate::integrations::{
    IntegrationCapability, IntegrationProvider, SyncContext, SyncResult, TestConnectionDetails,
};
use async_trait::async_trait;
use serde_json::Value;

/// Okta integration provider
pub struct OktaProvider;

impl OktaProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OktaProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationProvider for OktaProvider {
    fn integration_type(&self) -> &'static str {
        "okta"
    }

    fn capabilities(&self) -> Vec<IntegrationCapability> {
        vec![
            IntegrationCapability::UserSync,           // User directory
            IntegrationCapability::AccessSync,         // Application assignments
            IntegrationCapability::AuditLogs,          // System logs
            IntegrationCapability::ComplianceStatus,   // MFA status
        ]
    }

    fn validate_config(&self, config: &Value) -> Result<(), String> {
        let okta_config = OktaConfig::from_value(config)?;
        okta_config.validate()
    }

    async fn test_connection(&self, config: &Value) -> Result<TestConnectionDetails, String> {
        let okta_config = OktaConfig::from_value(config)?;
        okta_config.validate()?;

        // Create Okta client and test connection
        let client = OktaClient::new(okta_config.clone()).await?;

        // Get organization info to verify credentials
        let org = client.get_org_info().await?;

        // Build permissions list based on what services are enabled
        let mut permissions = vec!["org:read".to_string()];

        if okta_config.services.users || okta_config.services.mfa {
            permissions.push("users:read".to_string());
        }
        if okta_config.services.groups {
            permissions.push("groups:read".to_string());
        }
        if okta_config.services.applications {
            permissions.push("apps:read".to_string());
        }
        if okta_config.services.logs {
            permissions.push("logs:read".to_string());
        }

        Ok(TestConnectionDetails::success(format!(
            "Successfully connected to {}",
            org.company_name.as_deref().unwrap_or(&okta_config.domain)
        ))
        .with_account_info(serde_json::json!({
            "org_id": org.id,
            "company_name": org.company_name,
            "subdomain": org.subdomain,
            "website": org.website,
            "status": org.status,
            "domain": okta_config.domain,
            "services_enabled": {
                "users": okta_config.services.users,
                "mfa": okta_config.services.mfa,
                "groups": okta_config.services.groups,
                "applications": okta_config.services.applications,
                "logs": okta_config.services.logs,
                "log_days": okta_config.services.log_days,
            }
        }))
        .with_permissions(permissions))
    }

    async fn sync(&self, config: &Value, context: SyncContext) -> Result<SyncResult, String> {
        let okta_config = OktaConfig::from_value(config)?;
        okta_config.validate()?;

        // Create Okta client
        let client = OktaClient::new(okta_config.clone()).await?;

        // Get organization info for logging
        let org = client.get_org_info().await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            okta_org = ?org.company_name,
            okta_domain = %okta_config.domain,
            full_sync = context.full_sync,
            "Starting Okta sync"
        );

        // Run the sync orchestrator
        let result = super::sync::run_sync(&client, &okta_config, &context).await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            okta_org = ?org.company_name,
            records_processed = result.records_processed,
            records_created = result.records_created,
            errors = result.errors.len(),
            evidence_collected = result.evidence_collected.len(),
            "Completed Okta sync"
        );

        Ok(result)
    }

    fn required_fields(&self) -> Vec<&'static str> {
        vec!["domain", "api_token"]
    }

    fn optional_fields(&self) -> Vec<&'static str> {
        vec!["services"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_type() {
        let provider = OktaProvider::new();
        assert_eq!(provider.integration_type(), "okta");
    }

    #[test]
    fn test_capabilities() {
        let provider = OktaProvider::new();
        let caps = provider.capabilities();
        assert!(caps.contains(&IntegrationCapability::UserSync));
        assert!(caps.contains(&IntegrationCapability::AccessSync));
        assert!(caps.contains(&IntegrationCapability::AuditLogs));
        assert!(caps.contains(&IntegrationCapability::ComplianceStatus));
    }

    #[test]
    fn test_validate_config_missing_domain() {
        let provider = OktaProvider::new();
        let config = serde_json::json!({
            "api_token": "test-token"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_missing_token() {
        let provider = OktaProvider::new();
        let config = serde_json::json!({
            "domain": "test.okta.com"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_invalid_domain() {
        let provider = OktaProvider::new();
        let config = serde_json::json!({
            "domain": "test.example.com",
            "api_token": "test-token"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_valid() {
        let provider = OktaProvider::new();
        let config = serde_json::json!({
            "domain": "dev-123456.okta.com",
            "api_token": "test-token"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_ok());
    }
}
