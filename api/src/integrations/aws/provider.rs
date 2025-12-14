use super::client::AwsClient;
use super::config::AwsConfig;
use crate::integrations::{
    IntegrationCapability, IntegrationProvider, SyncContext, SyncResult, TestConnectionDetails,
};
use async_trait::async_trait;
use serde_json::Value;

/// AWS integration provider
pub struct AwsProvider;

impl AwsProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AwsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationProvider for AwsProvider {
    fn integration_type(&self) -> &'static str {
        "aws"
    }

    fn capabilities(&self) -> Vec<IntegrationCapability> {
        vec![
            IntegrationCapability::UserSync,
            IntegrationCapability::AccessSync,
            IntegrationCapability::AuditLogs,
            IntegrationCapability::SecurityFindings,
            IntegrationCapability::ComplianceStatus,
            IntegrationCapability::AssetInventory,
            IntegrationCapability::ConfigurationState,
        ]
    }

    fn validate_config(&self, config: &Value) -> Result<(), String> {
        let aws_config = AwsConfig::from_value(config)?;
        aws_config.validate()
    }

    async fn test_connection(&self, config: &Value) -> Result<TestConnectionDetails, String> {
        let aws_config = AwsConfig::from_value(config)?;
        aws_config.validate()?;

        // Create AWS client and test connection
        let client = AwsClient::new(aws_config.clone()).await?;

        // Get caller identity to verify credentials
        let identity = client.get_caller_identity().await?;

        // Build permissions list based on what services are enabled
        let mut permissions = vec!["sts:GetCallerIdentity".to_string()];

        if aws_config.services.iam {
            permissions.push("iam:List*".to_string());
            permissions.push("iam:Get*".to_string());
        }
        if aws_config.services.cloudtrail {
            permissions.push("cloudtrail:LookupEvents".to_string());
        }
        if aws_config.services.securityhub {
            permissions.push("securityhub:GetFindings".to_string());
        }
        if aws_config.services.config {
            permissions.push("config:Describe*".to_string());
        }
        if aws_config.services.s3 {
            permissions.push("s3:ListAllMyBuckets".to_string());
            permissions.push("s3:GetBucket*".to_string());
        }
        if aws_config.services.ec2 {
            permissions.push("ec2:Describe*".to_string());
        }
        if aws_config.services.rds {
            permissions.push("rds:Describe*".to_string());
        }

        Ok(TestConnectionDetails::success(format!(
            "Successfully connected to AWS account {}",
            identity.account_id
        ))
        .with_account_info(serde_json::json!({
            "account_id": identity.account_id,
            "arn": identity.arn,
            "user_id": identity.user_id,
            "regions": aws_config.all_regions(),
            "services_enabled": {
                "iam": aws_config.services.iam,
                "cloudtrail": aws_config.services.cloudtrail,
                "securityhub": aws_config.services.securityhub,
                "config": aws_config.services.config,
                "s3": aws_config.services.s3,
                "ec2": aws_config.services.ec2,
                "rds": aws_config.services.rds,
            }
        }))
        .with_permissions(permissions))
    }

    async fn sync(&self, config: &Value, context: SyncContext) -> Result<SyncResult, String> {
        let aws_config = AwsConfig::from_value(config)?;
        aws_config.validate()?;

        // Create AWS client
        let client = AwsClient::new(aws_config.clone()).await?;

        // Get account info for logging
        let identity = client.get_caller_identity().await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            aws_account_id = %identity.account_id,
            full_sync = context.full_sync,
            "Starting AWS sync"
        );

        // Run the sync orchestrator
        let result = super::sync::run_sync(&client, &aws_config, &context).await?;

        tracing::info!(
            organization_id = %context.organization_id,
            integration_id = %context.integration_id,
            aws_account_id = %identity.account_id,
            records_processed = result.records_processed,
            records_created = result.records_created,
            errors = result.errors.len(),
            "Completed AWS sync"
        );

        Ok(result)
    }

    fn required_fields(&self) -> Vec<&'static str> {
        vec!["region"]
    }

    fn optional_fields(&self) -> Vec<&'static str> {
        vec![
            "access_key_id",
            "secret_access_key",
            "assume_role_arn",
            "external_id",
            "regions",
            "services",
            "sync_options",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integration_type() {
        let provider = AwsProvider::new();
        assert_eq!(provider.integration_type(), "aws");
    }

    #[test]
    fn test_capabilities() {
        let provider = AwsProvider::new();
        let caps = provider.capabilities();
        assert!(caps.contains(&IntegrationCapability::UserSync));
        assert!(caps.contains(&IntegrationCapability::SecurityFindings));
        assert!(caps.contains(&IntegrationCapability::AssetInventory));
    }

    #[test]
    fn test_validate_config_missing_auth() {
        let provider = AwsProvider::new();
        let config = serde_json::json!({
            "region": "us-east-1"
        });
        // Should fail because no auth method credentials provided
        let result = provider.validate_config(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_config_access_keys() {
        let provider = AwsProvider::new();
        let config = serde_json::json!({
            "auth_method": "access_keys",
            "access_key_id": "AKIAIOSFODNN7EXAMPLE",
            "secret_access_key": "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "region": "us-east-1"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_assume_role() {
        let provider = AwsProvider::new();
        let config = serde_json::json!({
            "auth_method": "assume_role",
            "assume_role_arn": "arn:aws:iam::123456789012:role/OpenGRCRole",
            "region": "us-east-1"
        });
        let result = provider.validate_config(&config);
        assert!(result.is_ok());
    }
}
