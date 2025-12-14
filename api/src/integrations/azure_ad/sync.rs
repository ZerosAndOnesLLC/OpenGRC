use super::client::AzureAdClient;
use super::config::AzureAdConfig;
use super::services::{
    audit::AuditCollector,
    users::{ConditionalAccessCollector, GroupsCollector, UsersCollector},
};
use crate::integrations::provider::SyncError;
use crate::integrations::{SyncContext, SyncResult};

/// Run the full Azure AD sync across all enabled services
pub async fn run_sync(
    client: &AzureAdClient,
    config: &AzureAdConfig,
    context: &SyncContext,
) -> Result<SyncResult, String> {
    let mut result = SyncResult::default();

    // Sync users
    if config.services.users {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing Azure AD users"
        );

        match UsersCollector::sync(client, context).await {
            Ok(users_result) => result.merge(users_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Azure AD users");
                result = result.with_error(SyncError::new("users_sync_failed", e));
            }
        }
    }

    // Sync groups
    if config.services.groups {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing Azure AD groups"
        );

        match GroupsCollector::sync(client, context).await {
            Ok(groups_result) => result.merge(groups_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Azure AD groups");
                result = result.with_error(SyncError::new("groups_sync_failed", e));
            }
        }
    }

    // Sync conditional access policies
    if config.services.conditional_access {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing Azure AD conditional access policies"
        );

        match ConditionalAccessCollector::sync(client, context).await {
            Ok(ca_result) => result.merge(ca_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Azure AD conditional access policies");
                result = result.with_error(SyncError::new("conditional_access_sync_failed", e));
            }
        }
    }

    // Sync sign-in logs
    if config.services.sign_in_logs {
        tracing::info!(
            integration_id = %context.integration_id,
            log_days = config.services.log_days,
            "Syncing Azure AD sign-in logs"
        );

        match AuditCollector::sync_sign_in_logs(client, context, config.services.log_days).await {
            Ok(logs_result) => result.merge(logs_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Azure AD sign-in logs");
                result = result.with_error(SyncError::new("sign_in_logs_sync_failed", e));
            }
        }
    }

    // Sync audit logs
    if config.services.audit_logs {
        tracing::info!(
            integration_id = %context.integration_id,
            log_days = config.services.log_days,
            "Syncing Azure AD audit logs"
        );

        match AuditCollector::sync_audit_logs(client, context, config.services.log_days).await {
            Ok(logs_result) => result.merge(logs_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Azure AD audit logs");
                result = result.with_error(SyncError::new("audit_logs_sync_failed", e));
            }
        }
    }

    Ok(result)
}
