use super::client::AwsClient;
use super::config::AwsConfig;
use super::services::{
    cloudtrail::CloudTrailCollector, config_service::ConfigCollector, ec2::Ec2Collector,
    iam::IamCollector, rds::RdsCollector, s3::S3Collector, securityhub::SecurityHubCollector,
};
use crate::integrations::{SyncContext, SyncResult};

/// Run the full AWS sync across all enabled services
pub async fn run_sync(
    client: &AwsClient,
    config: &AwsConfig,
    context: &SyncContext,
) -> Result<SyncResult, String> {
    let mut result = SyncResult::default();

    // Sync global services (IAM)
    if config.services.iam {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing AWS IAM"
        );
        match IamCollector::sync(client, context).await {
            Ok(iam_result) => result.merge(iam_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync IAM");
                result = result.with_error(crate::integrations::provider::SyncError::new(
                    "iam_sync_failed",
                    e,
                ));
            }
        }
    }

    // Sync regional services in parallel across all regions
    let regions = config.all_regions();

    for region in &regions {
        tracing::info!(
            integration_id = %context.integration_id,
            region = %region,
            "Syncing regional AWS services"
        );

        // Security Hub
        if config.services.securityhub {
            match SecurityHubCollector::sync(client, context, region).await {
                Ok(sh_result) => result.merge(sh_result),
                Err(e) => {
                    tracing::warn!(error = %e, region = %region, "Failed to sync Security Hub");
                    result = result.with_error(
                        crate::integrations::provider::SyncError::new("securityhub_sync_failed", e)
                            .with_resource(region.clone()),
                    );
                }
            }
        }

        // AWS Config
        if config.services.config {
            match ConfigCollector::sync(client, context, region).await {
                Ok(cfg_result) => result.merge(cfg_result),
                Err(e) => {
                    tracing::warn!(error = %e, region = %region, "Failed to sync AWS Config");
                    result = result.with_error(
                        crate::integrations::provider::SyncError::new("config_sync_failed", e)
                            .with_resource(region.clone()),
                    );
                }
            }
        }

        // CloudTrail
        if config.services.cloudtrail {
            match CloudTrailCollector::sync(client, context, region, config.sync_options.cloudtrail_hours).await {
                Ok(ct_result) => result.merge(ct_result),
                Err(e) => {
                    tracing::warn!(error = %e, region = %region, "Failed to sync CloudTrail");
                    result = result.with_error(
                        crate::integrations::provider::SyncError::new("cloudtrail_sync_failed", e)
                            .with_resource(region.clone()),
                    );
                }
            }
        }

        // EC2
        if config.services.ec2 {
            match Ec2Collector::sync(client, context, region).await {
                Ok(ec2_result) => result.merge(ec2_result),
                Err(e) => {
                    tracing::warn!(error = %e, region = %region, "Failed to sync EC2");
                    result = result.with_error(
                        crate::integrations::provider::SyncError::new("ec2_sync_failed", e)
                            .with_resource(region.clone()),
                    );
                }
            }
        }

        // RDS
        if config.services.rds {
            match RdsCollector::sync(client, context, region).await {
                Ok(rds_result) => result.merge(rds_result),
                Err(e) => {
                    tracing::warn!(error = %e, region = %region, "Failed to sync RDS");
                    result = result.with_error(
                        crate::integrations::provider::SyncError::new("rds_sync_failed", e)
                            .with_resource(region.clone()),
                    );
                }
            }
        }
    }

    // S3 (global list, but bucket locations vary)
    if config.services.s3 {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing AWS S3"
        );
        match S3Collector::sync(client, context).await {
            Ok(s3_result) => result.merge(s3_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync S3");
                result = result.with_error(crate::integrations::provider::SyncError::new(
                    "s3_sync_failed",
                    e,
                ));
            }
        }
    }

    Ok(result)
}
