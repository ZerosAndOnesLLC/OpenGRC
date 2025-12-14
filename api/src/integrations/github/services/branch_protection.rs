use crate::integrations::github::client::{GitHubClient, GitHubRepository};
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;

/// Branch Protection Collector for GitHub
pub struct BranchProtectionCollector;

impl BranchProtectionCollector {
    /// Collect branch protection data from GitHub
    pub async fn sync(
        client: &GitHubClient,
        repos: &[GitHubRepository],
        _context: &SyncContext,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        let mut protected_repos = Vec::new();
        let mut unprotected_repos = Vec::new();
        let mut protection_details = Vec::new();

        for repo in repos {
            // Skip archived and disabled repos
            if repo.archived || repo.disabled {
                continue;
            }

            result.records_processed += 1;

            // Parse owner from full_name (format: owner/repo)
            let parts: Vec<&str> = repo.full_name.split('/').collect();
            if parts.len() != 2 {
                continue;
            }
            let owner = parts[0];

            // Get branch protection for default branch
            match client
                .get_branch_protection(owner, &repo.name, &repo.default_branch)
                .await
            {
                Ok(Some(protection)) => {
                    protected_repos.push(repo.full_name.clone());

                    let mut issues = Vec::new();

                    // Check for weak protection settings
                    if protection.enforce_admins.as_ref().map_or(true, |e| !e.enabled) {
                        issues.push("Admin enforcement disabled");
                    }
                    if protection.required_pull_request_reviews.is_none() {
                        issues.push("No PR review requirements");
                    }
                    if protection.required_status_checks.is_none() {
                        issues.push("No status check requirements");
                    }
                    if protection.allow_force_pushes.as_ref().map_or(false, |f| f.enabled) {
                        issues.push("Force pushes allowed");
                    }
                    if protection.allow_deletions.as_ref().map_or(false, |d| d.enabled) {
                        issues.push("Branch deletion allowed");
                    }

                    protection_details.push(json!({
                        "repository": repo.full_name,
                        "branch": repo.default_branch,
                        "protected": true,
                        "enforce_admins": protection.enforce_admins.as_ref().map(|e| e.enabled),
                        "required_reviews": protection.required_pull_request_reviews.as_ref().map(|r| json!({
                            "dismiss_stale_reviews": r.dismiss_stale_reviews,
                            "require_code_owner_reviews": r.require_code_owner_reviews,
                            "required_approving_review_count": r.required_approving_review_count,
                        })),
                        "required_status_checks": protection.required_status_checks.as_ref().map(|s| json!({
                            "strict": s.strict,
                            "contexts": s.contexts,
                        })),
                        "required_linear_history": protection.required_linear_history.as_ref().map(|l| l.enabled),
                        "allow_force_pushes": protection.allow_force_pushes.as_ref().map(|f| f.enabled),
                        "allow_deletions": protection.allow_deletions.as_ref().map(|d| d.enabled),
                        "required_signatures": protection.required_signatures.as_ref().map(|s| s.enabled),
                        "issues": issues,
                    }));
                }
                Ok(None) => {
                    unprotected_repos.push(repo.full_name.clone());
                    protection_details.push(json!({
                        "repository": repo.full_name,
                        "branch": repo.default_branch,
                        "protected": false,
                    }));
                }
                Err(e) => {
                    tracing::warn!(
                        repo = %repo.full_name,
                        error = %e,
                        "Failed to get branch protection"
                    );
                }
            }
        }

        let total_checked = protected_repos.len() + unprotected_repos.len();
        if total_checked == 0 {
            return Ok(result);
        }

        let protection_rate = protected_repos.len() as f64 / total_checked as f64 * 100.0;

        // Generate branch protection compliance evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Branch Protection Compliance Report".to_string(),
            description: Some(format!(
                "{:.1}% of repositories have branch protection enabled ({} of {})",
                protection_rate,
                protected_repos.len(),
                total_checked
            )),
            evidence_type: "automated".to_string(),
            source: "github".to_string(),
            source_reference: Some("github:branch-protection".to_string()),
            data: json!({
                "total_repositories_checked": total_checked,
                "protected_count": protected_repos.len(),
                "unprotected_count": unprotected_repos.len(),
                "protection_rate": protection_rate,
                "unprotected_repositories": unprotected_repos,
                "protection_details": protection_details,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.6".to_string(),
                "CC8.1".to_string(),
            ],
        });

        result.records_created = result.records_processed;
        Ok(result)
    }
}
