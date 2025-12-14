use super::client::GitHubClient;
use super::config::GitHubConfig;
use super::services::{
    branch_protection::BranchProtectionCollector, members::MembersCollector,
    repositories::RepositoryCollector, security_alerts::SecurityAlertsCollector,
};
use crate::integrations::provider::SyncError;
use crate::integrations::{SyncContext, SyncResult};

/// Run the full GitHub sync across all enabled services
pub async fn run_sync(
    client: &GitHubClient,
    config: &GitHubConfig,
    context: &SyncContext,
) -> Result<SyncResult, String> {
    let mut result = SyncResult::default();

    // First, collect repositories (needed for other collectors)
    let repos = if config.services.repositories
        || config.services.branch_protection
        || config.services.dependabot_alerts
        || config.services.code_scanning
        || config.services.secret_scanning
    {
        tracing::info!(
            integration_id = %context.integration_id,
            "Syncing GitHub repositories"
        );

        let repos = client.list_repositories().await?;

        // Filter repos if specific ones are configured
        let configured_repos = config.get_repositories();
        let filtered_repos = if configured_repos.is_empty() {
            repos
        } else {
            repos
                .into_iter()
                .filter(|r| {
                    configured_repos
                        .iter()
                        .any(|cr| r.name == *cr || r.full_name == *cr)
                })
                .collect()
        };

        filtered_repos
    } else {
        Vec::new()
    };

    // Sync repositories
    if config.services.repositories {
        match RepositoryCollector::sync(client, context).await {
            Ok(repo_result) => result.merge(repo_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync repositories");
                result = result.with_error(SyncError::new("repository_sync_failed", e));
            }
        }
    }

    // Sync branch protection
    if config.services.branch_protection && !repos.is_empty() {
        tracing::info!(
            integration_id = %context.integration_id,
            repo_count = repos.len(),
            "Syncing GitHub branch protection"
        );

        match BranchProtectionCollector::sync(client, &repos, context).await {
            Ok(bp_result) => result.merge(bp_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync branch protection");
                result = result.with_error(SyncError::new("branch_protection_sync_failed", e));
            }
        }
    }

    // Sync security alerts
    if (config.services.dependabot_alerts
        || config.services.code_scanning
        || config.services.secret_scanning)
        && !repos.is_empty()
    {
        tracing::info!(
            integration_id = %context.integration_id,
            repo_count = repos.len(),
            dependabot = config.services.dependabot_alerts,
            code_scanning = config.services.code_scanning,
            secret_scanning = config.services.secret_scanning,
            "Syncing GitHub security alerts"
        );

        match SecurityAlertsCollector::sync(
            client,
            &repos,
            context,
            config.services.dependabot_alerts,
            config.services.code_scanning,
            config.services.secret_scanning,
        )
        .await
        {
            Ok(alerts_result) => result.merge(alerts_result),
            Err(e) => {
                tracing::error!(error = %e, "Failed to sync security alerts");
                result = result.with_error(SyncError::new("security_alerts_sync_failed", e));
            }
        }
    }

    // Sync organization members
    if config.services.members {
        if let Some(ref org) = config.organization {
            tracing::info!(
                integration_id = %context.integration_id,
                organization = %org,
                "Syncing GitHub organization members"
            );

            match MembersCollector::sync(client, org, context).await {
                Ok(members_result) => result.merge(members_result),
                Err(e) => {
                    tracing::error!(error = %e, organization = %org, "Failed to sync organization members");
                    result = result.with_error(
                        SyncError::new("members_sync_failed", e).with_resource(org.clone()),
                    );
                }
            }
        } else {
            tracing::info!(
                integration_id = %context.integration_id,
                "Skipping organization members sync - no organization configured"
            );
        }
    }

    Ok(result)
}
