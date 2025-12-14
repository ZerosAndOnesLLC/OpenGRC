use crate::integrations::github::client::GitHubClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;

/// Repository Collector for GitHub
pub struct RepositoryCollector;

impl RepositoryCollector {
    /// Collect repository data from GitHub
    pub async fn sync(client: &GitHubClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get all repositories
        let repos = client.list_repositories().await?;
        result.records_processed = repos.len() as i32;

        if repos.is_empty() {
            return Ok(result);
        }

        // Analyze repositories
        let public_repos: Vec<_> = repos.iter().filter(|r| !r.private).collect();
        let archived_repos: Vec<_> = repos.iter().filter(|r| r.archived).collect();
        let fork_repos: Vec<_> = repos.iter().filter(|r| r.fork).collect();

        // Categorize by language
        let mut langs: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for repo in &repos {
            if let Some(lang) = &repo.language {
                *langs.entry(lang.clone()).or_insert(0) += 1;
            }
        }

        // Generate repository inventory evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "GitHub Repository Inventory".to_string(),
            description: Some(format!(
                "Inventory of {} GitHub repositories ({} public, {} private)",
                repos.len(),
                public_repos.len(),
                repos.len() - public_repos.len()
            )),
            evidence_type: "automated".to_string(),
            source: "github".to_string(),
            source_reference: Some("github:repositories".to_string()),
            data: json!({
                "total_repositories": repos.len(),
                "public_count": public_repos.len(),
                "private_count": repos.len() - public_repos.len(),
                "archived_count": archived_repos.len(),
                "fork_count": fork_repos.len(),
                "languages": langs,
                "repositories": repos.iter().map(|r| json!({
                    "name": r.name,
                    "full_name": r.full_name,
                    "private": r.private,
                    "archived": r.archived,
                    "fork": r.fork,
                    "default_branch": r.default_branch,
                    "language": r.language,
                    "visibility": r.visibility,
                    "forks_count": r.forks_count,
                    "stargazers_count": r.stargazers_count,
                    "open_issues_count": r.open_issues_count,
                    "created_at": r.created_at,
                    "updated_at": r.updated_at,
                    "pushed_at": r.pushed_at,
                    "html_url": r.html_url,
                })).collect::<Vec<_>>(),
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec!["CC6.1".to_string(), "CC6.7".to_string(), "A1.1".to_string()],
        });

        // Generate public repository visibility report
        if !public_repos.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Public Repository Report".to_string(),
                description: Some(format!(
                    "{} repositories are publicly accessible",
                    public_repos.len()
                )),
                evidence_type: "automated".to_string(),
                source: "github".to_string(),
                source_reference: Some("github:public-repos".to_string()),
                data: json!({
                    "public_repositories": public_repos.iter().map(|r| json!({
                        "name": r.name,
                        "full_name": r.full_name,
                        "description": r.description,
                        "html_url": r.html_url,
                        "created_at": r.created_at,
                        "pushed_at": r.pushed_at,
                    })).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC6.6".to_string(), "CC6.7".to_string()],
            });
        }

        result.records_created = result.records_processed;
        Ok(result)
    }
}
