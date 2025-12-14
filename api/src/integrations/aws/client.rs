use super::config::{AwsAccountInfo, AwsAuthMethod, AwsConfig};
use aws_config::meta::region::RegionProviderChain;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_cloudtrail::Client as CloudTrailClient;
use aws_sdk_config::Client as ConfigClient;
use aws_sdk_ec2::Client as Ec2Client;
use aws_sdk_iam::Client as IamClient;
use aws_sdk_rds::Client as RdsClient;
use aws_sdk_s3::Client as S3Client;
use aws_sdk_securityhub::Client as SecurityHubClient;
use aws_sdk_sts::Client as StsClient;
use std::sync::Arc;

/// AWS client wrapper that handles authentication and multi-region support
#[derive(Clone)]
pub struct AwsClient {
    config: AwsConfig,
    sdk_config: aws_config::SdkConfig,
}

impl AwsClient {
    /// Create a new AWS client from configuration
    pub async fn new(config: AwsConfig) -> Result<Self, String> {
        let sdk_config = Self::build_sdk_config(&config).await?;
        Ok(Self { config, sdk_config })
    }

    /// Build AWS SDK config from our config
    async fn build_sdk_config(config: &AwsConfig) -> Result<aws_config::SdkConfig, String> {
        let region_provider =
            RegionProviderChain::first_try(aws_sdk_sts::config::Region::new(config.region.clone()))
                .or_default_provider();

        match &config.auth_method {
            AwsAuthMethod::AccessKeys => {
                let access_key = config
                    .access_key_id
                    .as_ref()
                    .ok_or("Access Key ID is required")?;
                let secret_key = config
                    .secret_access_key
                    .as_ref()
                    .ok_or("Secret Access Key is required")?;

                let credentials =
                    Credentials::new(access_key, secret_key, None, None, "opengrc-static");

                let sdk_config = aws_config::defaults(BehaviorVersion::latest())
                    .region(region_provider)
                    .credentials_provider(credentials)
                    .load()
                    .await;

                Ok(sdk_config)
            }
            AwsAuthMethod::AssumeRole => {
                let role_arn = config
                    .assume_role_arn
                    .as_ref()
                    .ok_or("Assume Role ARN is required")?;

                // First, load default credentials (could be from env, instance profile, etc.)
                let base_region_provider =
                    RegionProviderChain::first_try(aws_sdk_sts::config::Region::new(config.region.clone()))
                        .or_default_provider();
                let base_config = aws_config::defaults(BehaviorVersion::latest())
                    .region(base_region_provider)
                    .load()
                    .await;

                // Create STS client with base credentials
                let sts_client = StsClient::new(&base_config);

                // Assume the role
                let mut assume_role_req = sts_client
                    .assume_role()
                    .role_arn(role_arn)
                    .role_session_name("opengrc-session");

                if let Some(external_id) = &config.external_id {
                    assume_role_req = assume_role_req.external_id(external_id);
                }

                let assumed_role = assume_role_req
                    .send()
                    .await
                    .map_err(|e| format!("Failed to assume role: {}", e))?;

                let creds = assumed_role
                    .credentials()
                    .ok_or("No credentials returned from AssumeRole")?;

                let credentials = Credentials::new(
                    creds.access_key_id(),
                    creds.secret_access_key(),
                    Some(creds.session_token().to_string()),
                    Some(
                        std::time::SystemTime::try_from(creds.expiration().clone())
                            .unwrap_or(std::time::SystemTime::now()),
                    ),
                    "opengrc-assumed-role",
                );

                let sdk_config = aws_config::defaults(BehaviorVersion::latest())
                    .region(region_provider)
                    .credentials_provider(credentials)
                    .load()
                    .await;

                Ok(sdk_config)
            }
        }
    }

    /// Get account information using STS GetCallerIdentity
    pub async fn get_caller_identity(&self) -> Result<AwsAccountInfo, String> {
        let sts_client = StsClient::new(&self.sdk_config);

        let identity = sts_client
            .get_caller_identity()
            .send()
            .await
            .map_err(|e| format!("Failed to get caller identity: {}", e))?;

        Ok(AwsAccountInfo {
            account_id: identity.account().unwrap_or_default().to_string(),
            arn: identity.arn().unwrap_or_default().to_string(),
            user_id: identity.user_id().unwrap_or_default().to_string(),
        })
    }

    /// Get IAM client (global service, uses primary region)
    pub fn iam_client(&self) -> IamClient {
        IamClient::new(&self.sdk_config)
    }

    /// Get STS client
    pub fn sts_client(&self) -> StsClient {
        StsClient::new(&self.sdk_config)
    }

    /// Get S3 client (global bucket list, regional for bucket operations)
    pub fn s3_client(&self) -> S3Client {
        S3Client::new(&self.sdk_config)
    }

    /// Get S3 client for a specific region
    pub async fn s3_client_for_region(&self, region: &str) -> Result<S3Client, String> {
        let regional_config = self.regional_config(region).await?;
        Ok(S3Client::new(&regional_config))
    }

    /// Get CloudTrail client for a specific region
    pub async fn cloudtrail_client(&self, region: &str) -> Result<CloudTrailClient, String> {
        let regional_config = self.regional_config(region).await?;
        Ok(CloudTrailClient::new(&regional_config))
    }

    /// Get Security Hub client for a specific region
    pub async fn securityhub_client(&self, region: &str) -> Result<SecurityHubClient, String> {
        let regional_config = self.regional_config(region).await?;
        Ok(SecurityHubClient::new(&regional_config))
    }

    /// Get Config client for a specific region
    pub async fn config_client(&self, region: &str) -> Result<ConfigClient, String> {
        let regional_config = self.regional_config(region).await?;
        Ok(ConfigClient::new(&regional_config))
    }

    /// Get EC2 client for a specific region
    pub async fn ec2_client(&self, region: &str) -> Result<Ec2Client, String> {
        let regional_config = self.regional_config(region).await?;
        Ok(Ec2Client::new(&regional_config))
    }

    /// Get RDS client for a specific region
    pub async fn rds_client(&self, region: &str) -> Result<RdsClient, String> {
        let regional_config = self.regional_config(region).await?;
        Ok(RdsClient::new(&regional_config))
    }

    /// Build SDK config for a specific region
    async fn regional_config(&self, region: &str) -> Result<aws_config::SdkConfig, String> {
        // Clone the base config but override the region
        let region_provider =
            RegionProviderChain::first_try(aws_sdk_sts::config::Region::new(region.to_string()));

        // Build new config with same credentials but different region
        match &self.config.auth_method {
            AwsAuthMethod::AccessKeys => {
                let access_key = self
                    .config
                    .access_key_id
                    .as_ref()
                    .ok_or("Access Key ID is required")?;
                let secret_key = self
                    .config
                    .secret_access_key
                    .as_ref()
                    .ok_or("Secret Access Key is required")?;

                let credentials =
                    Credentials::new(access_key, secret_key, None, None, "opengrc-static");

                Ok(aws_config::defaults(BehaviorVersion::latest())
                    .region(region_provider)
                    .credentials_provider(credentials)
                    .load()
                    .await)
            }
            AwsAuthMethod::AssumeRole => {
                // For assumed roles, we need to re-assume for each region
                // In practice, the same assumed credentials work across regions
                // But we still need to set the correct region
                let role_arn = self
                    .config
                    .assume_role_arn
                    .as_ref()
                    .ok_or("Assume Role ARN is required")?;

                let base_region_provider =
                    RegionProviderChain::first_try(aws_sdk_sts::config::Region::new(region.to_string()));
                let base_config = aws_config::defaults(BehaviorVersion::latest())
                    .region(base_region_provider)
                    .load()
                    .await;

                let sts_client = StsClient::new(&base_config);

                let mut assume_role_req = sts_client
                    .assume_role()
                    .role_arn(role_arn)
                    .role_session_name(format!("opengrc-{}", region));

                if let Some(external_id) = &self.config.external_id {
                    assume_role_req = assume_role_req.external_id(external_id);
                }

                let assumed_role = assume_role_req
                    .send()
                    .await
                    .map_err(|e| format!("Failed to assume role for region {}: {}", region, e))?;

                let creds = assumed_role
                    .credentials()
                    .ok_or("No credentials returned from AssumeRole")?;

                let credentials = Credentials::new(
                    creds.access_key_id(),
                    creds.secret_access_key(),
                    Some(creds.session_token().to_string()),
                    Some(
                        std::time::SystemTime::try_from(creds.expiration().clone())
                            .unwrap_or(std::time::SystemTime::now()),
                    ),
                    "opengrc-assumed-role",
                );

                Ok(aws_config::defaults(BehaviorVersion::latest())
                    .region(region_provider)
                    .credentials_provider(credentials)
                    .load()
                    .await)
            }
        }
    }

    /// Get the AWS configuration
    pub fn config(&self) -> &AwsConfig {
        &self.config
    }

    /// Get all configured regions
    pub fn regions(&self) -> Vec<String> {
        self.config.all_regions()
    }

    /// Get the primary region
    pub fn primary_region(&self) -> &str {
        &self.config.region
    }
}

/// Thread-safe reference to AWS client
pub type SharedAwsClient = Arc<AwsClient>;
