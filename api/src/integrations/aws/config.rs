use serde::{Deserialize, Serialize};
use serde_json::Value;

/// AWS authentication method
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AwsAuthMethod {
    /// Use access key ID and secret access key
    AccessKeys,
    /// Assume an IAM role using STS
    AssumeRole,
}

impl Default for AwsAuthMethod {
    fn default() -> Self {
        Self::AccessKeys
    }
}

/// AWS integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    /// Authentication method
    #[serde(default)]
    pub auth_method: AwsAuthMethod,

    /// AWS Access Key ID (required for AccessKeys auth)
    pub access_key_id: Option<String>,

    /// AWS Secret Access Key (required for AccessKeys auth)
    pub secret_access_key: Option<String>,

    /// IAM Role ARN to assume (required for AssumeRole auth)
    pub assume_role_arn: Option<String>,

    /// External ID for role assumption (optional, for security)
    pub external_id: Option<String>,

    /// Primary region for global services like IAM
    #[serde(default = "default_region")]
    pub region: String,

    /// Additional regions to sync (for regional services)
    #[serde(default)]
    pub regions: Vec<String>,

    /// Service-specific configuration
    #[serde(default)]
    pub services: AwsServicesConfig,

    /// Sync options
    #[serde(default)]
    pub sync_options: AwsSyncOptions,
}

fn default_region() -> String {
    "us-east-1".to_string()
}

/// Configuration for individual AWS services
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AwsServicesConfig {
    #[serde(default = "default_true")]
    pub iam: bool,
    #[serde(default = "default_true")]
    pub cloudtrail: bool,
    #[serde(default = "default_true")]
    pub securityhub: bool,
    #[serde(default = "default_true")]
    pub config: bool,
    #[serde(default = "default_true")]
    pub s3: bool,
    #[serde(default = "default_true")]
    pub ec2: bool,
    #[serde(default = "default_true")]
    pub rds: bool,
}

fn default_true() -> bool {
    true
}

/// Sync options for AWS integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsSyncOptions {
    /// Hours of CloudTrail events to fetch
    #[serde(default = "default_cloudtrail_hours")]
    pub cloudtrail_hours: u32,

    /// Maximum findings to fetch from Security Hub
    #[serde(default = "default_max_findings")]
    pub max_findings: u32,

    /// Whether to sync inactive/terminated resources
    #[serde(default)]
    pub include_inactive: bool,
}

fn default_cloudtrail_hours() -> u32 {
    24
}

fn default_max_findings() -> u32 {
    1000
}

impl Default for AwsSyncOptions {
    fn default() -> Self {
        Self {
            cloudtrail_hours: default_cloudtrail_hours(),
            max_findings: default_max_findings(),
            include_inactive: false,
        }
    }
}

impl AwsConfig {
    /// Parse AWS configuration from JSON value
    pub fn from_value(value: &Value) -> Result<Self, String> {
        serde_json::from_value(value.clone()).map_err(|e| format!("Invalid AWS config: {}", e))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        match self.auth_method {
            AwsAuthMethod::AccessKeys => {
                if self.access_key_id.as_ref().map_or(true, |s| s.is_empty()) {
                    return Err("Access Key ID is required for access key authentication".into());
                }
                if self
                    .secret_access_key
                    .as_ref()
                    .map_or(true, |s| s.is_empty())
                {
                    return Err(
                        "Secret Access Key is required for access key authentication".into(),
                    );
                }
            }
            AwsAuthMethod::AssumeRole => {
                if self.assume_role_arn.as_ref().map_or(true, |s| s.is_empty()) {
                    return Err("Assume Role ARN is required for role assumption".into());
                }
                // Validate ARN format
                if let Some(arn) = &self.assume_role_arn {
                    if !arn.starts_with("arn:aws:iam::") || !arn.contains(":role/") {
                        return Err("Invalid IAM Role ARN format".into());
                    }
                }
            }
        }

        if self.region.is_empty() {
            return Err("Primary region is required".into());
        }

        Ok(())
    }

    /// Get all regions to sync (primary + additional)
    pub fn all_regions(&self) -> Vec<String> {
        let mut regions = vec![self.region.clone()];
        for r in &self.regions {
            if !regions.contains(r) {
                regions.push(r.clone());
            }
        }
        regions
    }

    /// Check if a specific service is enabled
    pub fn is_service_enabled(&self, service: &str) -> bool {
        match service {
            "iam" => self.services.iam,
            "cloudtrail" => self.services.cloudtrail,
            "securityhub" => self.services.securityhub,
            "config" => self.services.config,
            "s3" => self.services.s3,
            "ec2" => self.services.ec2,
            "rds" => self.services.rds,
            _ => false,
        }
    }
}

/// AWS account information returned from STS GetCallerIdentity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsAccountInfo {
    pub account_id: String,
    pub arn: String,
    pub user_id: String,
}
