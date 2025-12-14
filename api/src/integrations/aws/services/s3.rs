use crate::integrations::aws::client::AwsClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// S3 bucket information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsS3Bucket {
    pub name: String,
    pub region: Option<String>,
    pub creation_date: Option<DateTime<Utc>>,
    pub encryption_enabled: bool,
    pub encryption_type: Option<String>,
    pub versioning_enabled: bool,
    pub logging_enabled: bool,
    pub public_access_blocked: bool,
    pub is_public: bool,
}

/// S3 collector
pub struct S3Collector;

impl S3Collector {
    /// Sync S3 buckets
    pub async fn sync(client: &AwsClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        let s3_client = client.s3_client();

        // List all buckets
        let buckets = Self::collect_buckets(&s3_client, client).await?;
        result.records_processed = buckets.len() as i32;

        // Analyze buckets
        let public_buckets: Vec<_> = buckets.iter().filter(|b| b.is_public).collect();
        let unencrypted_buckets: Vec<_> = buckets.iter().filter(|b| !b.encryption_enabled).collect();
        let _no_versioning: Vec<_> = buckets.iter().filter(|b| !b.versioning_enabled).collect();

        // Generate inventory evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "S3 Bucket Inventory".to_string(),
            description: Some(format!("Inventory of {} S3 buckets", buckets.len())),
            evidence_type: "automated".to_string(),
            source: "aws".to_string(),
            source_reference: Some("s3:buckets".to_string()),
            data: json!({
                "total_buckets": buckets.len(),
                "encrypted_buckets": buckets.iter().filter(|b| b.encryption_enabled).count(),
                "public_buckets": public_buckets.len(),
                "versioning_enabled": buckets.iter().filter(|b| b.versioning_enabled).count(),
                "logging_enabled": buckets.iter().filter(|b| b.logging_enabled).count(),
                "buckets": buckets.iter().map(|b| json!({
                    "name": b.name,
                    "region": b.region,
                    "encrypted": b.encryption_enabled,
                    "versioning": b.versioning_enabled,
                    "public": b.is_public,
                })).collect::<Vec<_>>(),
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec!["CC6.6".to_string(), "CC6.7".to_string(), "A1.1".to_string()],
        });

        // Public bucket warning
        if !public_buckets.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Public S3 Buckets".to_string(),
                description: Some(format!(
                    "{} buckets are publicly accessible",
                    public_buckets.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some("s3:public".to_string()),
                data: json!({
                    "public_buckets": public_buckets.iter().map(|b| json!({
                        "name": b.name,
                        "region": b.region,
                    })).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC6.6".to_string(), "CC6.7".to_string()],
            });
        }

        // Encryption compliance
        if !unencrypted_buckets.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Unencrypted S3 Buckets".to_string(),
                description: Some(format!(
                    "{} buckets without encryption",
                    unencrypted_buckets.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some("s3:unencrypted".to_string()),
                data: json!({
                    "unencrypted_buckets": unencrypted_buckets.iter().map(|b| b.name.clone()).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC6.6".to_string(), "CC6.7".to_string()],
            });
        }

        result.records_created = result.records_processed;
        Ok(result)
    }

    async fn collect_buckets(
        s3_client: &aws_sdk_s3::Client,
        _aws_client: &AwsClient,
    ) -> Result<Vec<AwsS3Bucket>, String> {
        let mut buckets = Vec::new();

        let response = s3_client
            .list_buckets()
            .send()
            .await
            .map_err(|e| format!("Failed to list S3 buckets: {}", e))?;

        for bucket in response.buckets() {
            let bucket_name = bucket.name().unwrap_or_default().to_string();

            // Get bucket location
            let region = s3_client
                .get_bucket_location()
                .bucket(&bucket_name)
                .send()
                .await
                .ok()
                .and_then(|r| {
                    r.location_constraint()
                        .map(|l| l.as_str().to_string())
                        .or(Some("us-east-1".to_string()))
                });

            // Get encryption status
            let (encryption_enabled, encryption_type) = s3_client
                .get_bucket_encryption()
                .bucket(&bucket_name)
                .send()
                .await
                .ok()
                .map(|r| {
                    let enc_type = r
                        .server_side_encryption_configuration()
                        .and_then(|c| c.rules().first())
                        .and_then(|rule| rule.apply_server_side_encryption_by_default())
                        .map(|default| default.sse_algorithm().as_str().to_string());
                    (true, enc_type)
                })
                .unwrap_or((false, None));

            // Get versioning status
            let versioning_enabled = s3_client
                .get_bucket_versioning()
                .bucket(&bucket_name)
                .send()
                .await
                .ok()
                .map(|r| {
                    r.status()
                        .map(|s| s.as_str() == "Enabled")
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            // Get logging status
            let logging_enabled = s3_client
                .get_bucket_logging()
                .bucket(&bucket_name)
                .send()
                .await
                .ok()
                .map(|r| r.logging_enabled().is_some())
                .unwrap_or(false);

            // Get public access block
            let public_access_blocked = s3_client
                .get_public_access_block()
                .bucket(&bucket_name)
                .send()
                .await
                .ok()
                .map(|r| {
                    r.public_access_block_configuration()
                        .map(|c| {
                            c.block_public_acls() == Some(true)
                                && c.block_public_policy() == Some(true)
                                && c.ignore_public_acls() == Some(true)
                                && c.restrict_public_buckets() == Some(true)
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            buckets.push(AwsS3Bucket {
                name: bucket_name,
                region,
                creation_date: bucket.creation_date().map(|d| {
                    DateTime::from_timestamp(d.secs(), d.subsec_nanos()).unwrap_or_else(Utc::now)
                }),
                encryption_enabled,
                encryption_type,
                versioning_enabled,
                logging_enabled,
                public_access_blocked,
                is_public: !public_access_blocked, // Simplified check
            });
        }

        Ok(buckets)
    }
}
