use super::client::GoogleWorkspaceClient;
use super::config::GoogleWorkspaceConfig;
use super::services::{
    audit::AuditCollector,
    users::{GroupsCollector, UsersCollector},
};
use crate::integrations::provider::SyncError;
use crate::integrations::{SyncContext, SyncResult};

/// Run the full Google Workspace sync across all enabled services
pub async fn run_sync(
    client: &GoogleWorkspaceClient,
    config: &GoogleWorkspaceConfig,
    context: &SyncContext,
) -> Result<SyncResult, String> {
    let mut result = SyncResult::default();

    // Sync users
    if config.services.users || config.services.two_step_verification {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing Google Workspace users"
        );

        match UsersCollector::sync(client, context).await {
            Ok(users_result) => result.merge(users_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Google Workspace users");
                result = result.with_error(SyncError::new("users_sync_failed", e));
            }
        }
    }

    // Sync groups
    if config.services.groups {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing Google Workspace groups"
        );

        match GroupsCollector::sync(client, context).await {
            Ok(groups_result) => result.merge(groups_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Google Workspace groups");
                result = result.with_error(SyncError::new("groups_sync_failed", e));
            }
        }
    }

    // Sync login audit logs
    if config.services.login_audit {
        tracing::info!(
            integration_id = %context.integration_id,
            log_days = config.services.log_days,
            "Syncing Google Workspace login audit"
        );

        match AuditCollector::sync_login_audit(client, context, config.services.log_days).await {
            Ok(audit_result) => result.merge(audit_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Google Workspace login audit");
                result = result.with_error(SyncError::new("login_audit_sync_failed", e));
            }
        }
    }

    // Sync admin audit logs
    if config.services.admin_audit {
        tracing::info!(
            integration_id = %context.integration_id,
            log_days = config.services.log_days,
            "Syncing Google Workspace admin audit"
        );

        match AuditCollector::sync_admin_audit(client, context, config.services.log_days).await {
            Ok(audit_result) => result.merge(audit_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync Google Workspace admin audit");
                result = result.with_error(SyncError::new("admin_audit_sync_failed", e));
            }
        }
    }

    Ok(result)
}
