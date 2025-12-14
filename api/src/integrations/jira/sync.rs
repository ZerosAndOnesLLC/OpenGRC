use super::client::JiraClient;
use super::config::JiraConfig;
use super::services::{
    issues::IssueCollector, permissions::PermissionsCollector, projects::ProjectCollector,
    users::UserCollector,
};
use crate::integrations::provider::SyncError;
use crate::integrations::{SyncContext, SyncResult};

/// Run the full Jira sync across all enabled services
pub async fn run_sync(
    client: &JiraClient,
    config: &JiraConfig,
    context: &SyncContext,
) -> Result<SyncResult, String> {
    let mut result = SyncResult::default();

    // First, collect projects (needed for other collectors)
    let projects = if config.services.projects
        || config.services.issues
        || config.services.users
        || config.services.permissions
    {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing Jira projects"
        );

        let projects = client.list_projects().await?;

        // Filter projects if specific ones are configured
        let configured_projects = config.get_projects();
        let filtered_projects = if configured_projects.is_empty() {
            projects
        } else {
            projects
                .into_iter()
                .filter(|p| {
                    configured_projects
                        .iter()
                        .any(|cp| p.key == *cp || p.name == *cp)
                })
                .collect()
        };

        filtered_projects
    } else {
        Vec::new()
    };

    // Sync projects
    if config.services.projects {
        match ProjectCollector::sync(client, context).await {
            Ok(project_result) => result.merge(project_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync projects");
                result = result.with_error(SyncError::new("project_sync_failed", e));
            }
        }
    }

    // Sync issues
    if config.services.issues && !projects.is_empty() {
        tracing::info!(
            integration_id = %context.integration_id,
            project_count = projects.len(),
            "Syncing Jira issues"
        );

        match IssueCollector::sync(client, &projects, context).await {
            Ok(issue_result) => result.merge(issue_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync issues");
                result = result.with_error(SyncError::new("issue_sync_failed", e));
            }
        }
    }

    // Sync users
    if config.services.users && !projects.is_empty() {
        tracing::info!(
            integration_id = %context.integration_id,
            project_count = projects.len(),
            "Syncing Jira users"
        );

        match UserCollector::sync(client, &projects, context).await {
            Ok(user_result) => result.merge(user_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync users");
                result = result.with_error(SyncError::new("user_sync_failed", e));
            }
        }
    }

    // Sync permissions
    if config.services.permissions && !projects.is_empty() {
        tracing::info!(
            integration_id = %context.integration_id,
            project_count = projects.len(),
            "Syncing Jira project permissions"
        );

        match PermissionsCollector::sync(client, &projects, context).await {
            Ok(perm_result) => result.merge(perm_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync permissions");
                result = result.with_error(SyncError::new("permissions_sync_failed", e));
            }
        }
    }

    Ok(result)
}
