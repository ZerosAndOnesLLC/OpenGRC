use crate::integrations::aws::client::AwsClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// RDS instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsRdsInstance {
    pub db_instance_identifier: String,
    pub db_instance_class: String,
    pub engine: String,
    pub engine_version: String,
    pub status: String,
    pub endpoint: Option<String>,
    pub port: Option<i32>,
    pub availability_zone: Option<String>,
    pub multi_az: bool,
    pub publicly_accessible: bool,
    pub storage_encrypted: bool,
    pub kms_key_id: Option<String>,
    pub vpc_id: Option<String>,
    pub security_groups: Vec<String>,
    pub backup_retention_period: i32,
    pub deletion_protection: bool,
    pub iam_auth_enabled: bool,
    pub auto_minor_version_upgrade: bool,
    pub allocated_storage: i32,
    pub storage_type: Option<String>,
    pub tags: HashMap<String, String>,
}

/// RDS collector
pub struct RdsCollector;

impl RdsCollector {
    /// Sync RDS instances for a region
    pub async fn sync(
        client: &AwsClient,
        _context: &SyncContext,
        region: &str,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        let rds_client = client.rds_client(region).await?;

        // Collect instances
        let instances = Self::collect_instances(&rds_client).await?;
        result.records_processed = instances.len() as i32;

        // Analyze
        let public_instances: Vec<_> = instances.iter().filter(|i| i.publicly_accessible).collect();
        let unencrypted_instances: Vec<_> =
            instances.iter().filter(|i| !i.storage_encrypted).collect();
        let no_backup_instances: Vec<_> =
            instances.iter().filter(|i| i.backup_retention_period == 0).collect();
        let _no_deletion_protection: Vec<_> =
            instances.iter().filter(|i| !i.deletion_protection).collect();

        // Generate inventory evidence
        result.evidence_collected.push(CollectedEvidence {
            title: format!("RDS Instance Inventory - {}", region),
            description: Some(format!(
                "{} RDS instances ({} encrypted, {} publicly accessible)",
                instances.len(),
                instances.iter().filter(|i| i.storage_encrypted).count(),
                public_instances.len()
            )),
            evidence_type: "automated".to_string(),
            source: "aws".to_string(),
            source_reference: Some(format!("rds:{}:instances", region)),
            data: json!({
                "region": region,
                "total_instances": instances.len(),
                "encrypted_instances": instances.iter().filter(|i| i.storage_encrypted).count(),
                "public_instances": public_instances.len(),
                "multi_az_instances": instances.iter().filter(|i| i.multi_az).count(),
                "instances": instances.iter().map(|i| json!({
                    "identifier": i.db_instance_identifier,
                    "engine": format!("{} {}", i.engine, i.engine_version),
                    "status": i.status,
                    "multi_az": i.multi_az,
                    "encrypted": i.storage_encrypted,
                    "public": i.publicly_accessible,
                    "backup_days": i.backup_retention_period,
                    "deletion_protection": i.deletion_protection,
                })).collect::<Vec<_>>(),
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec!["A1.1".to_string(), "CC6.6".to_string(), "CC6.7".to_string()],
        });

        // Public instances evidence
        if !public_instances.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: format!("Publicly Accessible RDS Instances - {}", region),
                description: Some(format!(
                    "{} RDS instances are publicly accessible",
                    public_instances.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some(format!("rds:{}:public", region)),
                data: json!({
                    "public_instances": public_instances.iter().map(|i| json!({
                        "identifier": i.db_instance_identifier,
                        "engine": i.engine,
                        "endpoint": i.endpoint,
                    })).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC6.6".to_string(), "CC6.7".to_string()],
            });
        }

        // Unencrypted instances evidence
        if !unencrypted_instances.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: format!("Unencrypted RDS Instances - {}", region),
                description: Some(format!(
                    "{} RDS instances without encryption at rest",
                    unencrypted_instances.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some(format!("rds:{}:unencrypted", region)),
                data: json!({
                    "unencrypted_instances": unencrypted_instances.iter().map(|i| json!({
                        "identifier": i.db_instance_identifier,
                        "engine": i.engine,
                    })).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC6.6".to_string(), "CC6.7".to_string()],
            });
        }

        // Backup compliance evidence
        if !no_backup_instances.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: format!("RDS Instances Without Backups - {}", region),
                description: Some(format!(
                    "{} RDS instances have no automated backups",
                    no_backup_instances.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some(format!("rds:{}:no-backup", region)),
                data: json!({
                    "no_backup_instances": no_backup_instances.iter().map(|i| json!({
                        "identifier": i.db_instance_identifier,
                        "engine": i.engine,
                    })).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["A1.2".to_string()],
            });
        }

        result.records_created = result.records_processed;
        Ok(result)
    }

    async fn collect_instances(
        rds_client: &aws_sdk_rds::Client,
    ) -> Result<Vec<AwsRdsInstance>, String> {
        let mut instances = Vec::new();
        let mut marker: Option<String> = None;

        loop {
            let mut request = rds_client.describe_db_instances();
            if let Some(m) = &marker {
                request = request.marker(m);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format!("Failed to describe RDS instances: {}", e))?;

            for db in response.db_instances() {
                let tags: HashMap<String, String> = db
                    .tag_list()
                    .iter()
                    .filter_map(|t| {
                        Some((t.key()?.to_string(), t.value()?.to_string()))
                    })
                    .collect();

                instances.push(AwsRdsInstance {
                    db_instance_identifier: db
                        .db_instance_identifier()
                        .unwrap_or_default()
                        .to_string(),
                    db_instance_class: db.db_instance_class().unwrap_or_default().to_string(),
                    engine: db.engine().unwrap_or_default().to_string(),
                    engine_version: db.engine_version().unwrap_or_default().to_string(),
                    status: db.db_instance_status().unwrap_or_default().to_string(),
                    endpoint: db.endpoint().and_then(|e| e.address()).map(|s| s.to_string()),
                    port: db.endpoint().and_then(|e| e.port()),
                    availability_zone: db.availability_zone().map(|s| s.to_string()),
                    multi_az: db.multi_az().unwrap_or(false),
                    publicly_accessible: db.publicly_accessible().unwrap_or(false),
                    storage_encrypted: db.storage_encrypted().unwrap_or(false),
                    kms_key_id: db.kms_key_id().map(|s| s.to_string()),
                    vpc_id: db
                        .db_subnet_group()
                        .and_then(|sg| sg.vpc_id())
                        .map(|s| s.to_string()),
                    security_groups: db
                        .vpc_security_groups()
                        .iter()
                        .filter_map(|sg| sg.vpc_security_group_id().map(|s| s.to_string()))
                        .collect(),
                    backup_retention_period: db.backup_retention_period().unwrap_or(0),
                    deletion_protection: db.deletion_protection().unwrap_or(false),
                    iam_auth_enabled: db.iam_database_authentication_enabled().unwrap_or(false),
                    auto_minor_version_upgrade: db.auto_minor_version_upgrade().unwrap_or(false),
                    allocated_storage: db.allocated_storage().unwrap_or(0),
                    storage_type: db.storage_type().map(|s| s.to_string()),
                    tags,
                });
            }

            marker = response.marker().map(|s| s.to_string());
            if marker.is_none() {
                break;
            }
        }

        Ok(instances)
    }
}
