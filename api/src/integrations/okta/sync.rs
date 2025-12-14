use super::client::OktaClient;
use super::config::OktaConfig;
use super::services::{
    applications::ApplicationsCollector, logs::LogsCollector, mfa::MfaCollector,
    users::{GroupsCollector, UsersCollector},
};
use crate::integrations::provider::SyncError;
use crate::integrations::{SyncContext, SyncResult};

/// Run the full Okta sync across all enabled services
pub async fn run_sync(
    client: &OktaClient,
    config: &OktaConfig,
    context: &SyncContext,
) -> Result<SyncResult, String> {
    let mut result = SyncResult::default();

    // Sync users
    if config.services.users {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing Okta users"
        );

        match UsersCollector::sync(client, context).await {
            Ok(users_result) => result.merge(users_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Okta users");
                result = result.with_error(SyncError::new("users_sync_failed", e));
            }
        }
    }

    // Sync groups
    if config.services.groups {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing Okta groups"
        );

        match GroupsCollector::sync(client, context).await {
            Ok(groups_result) => result.merge(groups_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Okta groups");
                result = result.with_error(SyncError::new("groups_sync_failed", e));
            }
        }
    }

    // Sync MFA status
    if config.services.mfa {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing Okta MFA status"
        );

        match MfaCollector::sync(client, context).await {
            Ok(mfa_result) => result.merge(mfa_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Okta MFA status");
                result = result.with_error(SyncError::new("mfa_sync_failed", e));
            }
        }
    }

    // Sync applications
    if config.services.applications {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing Okta applications"
        );

        match ApplicationsCollector::sync(client, context).await {
            Ok(apps_result) => result.merge(apps_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Okta applications");
                result = result.with_error(SyncError::new("applications_sync_failed", e));
            }
        }
    }

    // Sync system logs
    if config.services.logs {
        tracing::info!(
            integration_id = %context.integration_id,
            log_days = config.services.log_days,
            "Syncing Okta system logs"
        );

        match LogsCollector::sync(client, context, config.services.log_days).await {
            Ok(logs_result) => result.merge(logs_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Okta system logs");
                result = result.with_error(SyncError::new("logs_sync_failed", e));
            }
        }
    }

    Ok(result)
}
