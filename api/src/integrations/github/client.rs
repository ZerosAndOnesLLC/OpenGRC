use super::config::GitHubConfig;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

const GITHUB_API_URL: &str = "https://api.github.com";

/// GitHub API client
#[derive(Clone)]
pub struct GitHubClient {
    client: Client,
    config: GitHubConfig,
}

pub type SharedGitHubClient = Arc<GitHubClient>;

// API Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: i64,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: Option<String>,
    pub html_url: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub site_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitHubRepository {
    pub id: i64,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub private: bool,
    pub html_url: String,
    pub default_branch: String,
    pub archived: bool,
    pub disabled: bool,
    pub visibility: Option<String>,
    pub language: Option<String>,
    pub fork: bool,
    pub forks_count: i64,
    pub stargazers_count: i64,
    pub open_issues_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub pushed_at: Option<String>,
    pub permissions: Option<RepoPermissions>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepoPermissions {
    pub admin: bool,
    pub maintain: Option<bool>,
    pub push: bool,
    pub triage: Option<bool>,
    pub pull: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BranchProtection {
    pub url: String,
    pub required_status_checks: Option<RequiredStatusChecks>,
    pub enforce_admins: Option<EnforceAdmins>,
    pub required_pull_request_reviews: Option<RequiredPullRequestReviews>,
    pub restrictions: Option<Value>,
    pub required_linear_history: Option<RequiredLinearHistory>,
    pub allow_force_pushes: Option<AllowForcePushes>,
    pub allow_deletions: Option<AllowDeletions>,
    pub required_signatures: Option<RequiredSignatures>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequiredStatusChecks {
    pub strict: bool,
    pub contexts: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnforceAdmins {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequiredPullRequestReviews {
    pub dismiss_stale_reviews: bool,
    pub require_code_owner_reviews: bool,
    pub required_approving_review_count: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequiredLinearHistory {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllowForcePushes {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AllowDeletions {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequiredSignatures {
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependabotAlert {
    pub number: i64,
    pub state: String,
    pub dependency: DependabotDependency,
    pub security_advisory: SecurityAdvisory,
    pub security_vulnerability: SecurityVulnerability,
    pub url: String,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub dismissed_at: Option<String>,
    pub dismissed_by: Option<GitHubUser>,
    pub dismissed_reason: Option<String>,
    pub fixed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DependabotDependency {
    pub package: Package,
    pub manifest_path: String,
    pub scope: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Package {
    pub ecosystem: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityAdvisory {
    pub ghsa_id: String,
    pub cve_id: Option<String>,
    pub summary: String,
    pub description: String,
    pub severity: String,
    pub cvss: Option<Cvss>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cvss {
    pub score: f64,
    pub vector_string: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub package: Package,
    pub severity: String,
    pub vulnerable_version_range: String,
    pub first_patched_version: Option<FirstPatchedVersion>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FirstPatchedVersion {
    pub identifier: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeScanningAlert {
    pub number: i64,
    pub state: String,
    pub rule: CodeScanningRule,
    pub tool: CodeScanningTool,
    pub most_recent_instance: Option<CodeScanningInstance>,
    pub html_url: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub dismissed_at: Option<String>,
    pub dismissed_by: Option<GitHubUser>,
    pub dismissed_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeScanningRule {
    pub id: String,
    pub severity: Option<String>,
    pub security_severity_level: Option<String>,
    pub description: String,
    pub name: String,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeScanningTool {
    pub name: String,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeScanningInstance {
    pub ref_name: Option<String>,
    pub state: String,
    pub location: Option<CodeScanningLocation>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeScanningLocation {
    pub path: Option<String>,
    pub start_line: Option<i64>,
    pub end_line: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecretScanningAlert {
    pub number: i64,
    pub state: String,
    pub secret_type: String,
    pub secret_type_display_name: String,
    pub secret: Option<String>,
    pub html_url: String,
    pub url: String,
    pub created_at: String,
    pub updated_at: Option<String>,
    pub resolved_at: Option<String>,
    pub resolved_by: Option<GitHubUser>,
    pub resolution: Option<String>,
    pub resolution_comment: Option<String>,
    pub push_protection_bypassed: Option<bool>,
    pub push_protection_bypassed_by: Option<GitHubUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrgMember {
    pub login: String,
    pub id: i64,
    pub avatar_url: String,
    pub html_url: String,
    #[serde(rename = "type")]
    pub user_type: String,
    pub site_admin: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrgMembership {
    pub state: String,
    pub role: String,
    pub user: GitHubUser,
}

impl GitHubClient {
    pub async fn new(config: GitHubConfig) -> Result<Self, String> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", config.access_token))
                .map_err(|e| format!("Invalid token: {}", e))?,
        );
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            header::HeaderValue::from_static("2022-11-28"),
        );
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("OpenGRC/1.0"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client, config })
    }

    /// Get the authenticated user
    pub async fn get_authenticated_user(&self) -> Result<GitHubUser, String> {
        let url = format!("{}/user", GITHUB_API_URL);
        self.get(&url).await
    }

    /// List repositories for the organization or authenticated user
    pub async fn list_repositories(&self) -> Result<Vec<GitHubRepository>, String> {
        let url = if let Some(ref org) = self.config.organization {
            format!("{}/orgs/{}/repos?per_page=100&type=all", GITHUB_API_URL, org)
        } else {
            format!("{}/user/repos?per_page=100&type=all", GITHUB_API_URL)
        };

        self.get_paginated(&url).await
    }

    /// Get branch protection for a repository
    pub async fn get_branch_protection(
        &self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Option<BranchProtection>, String> {
        let url = format!(
            "{}/repos/{}/{}/branches/{}/protection",
            GITHUB_API_URL, owner, repo, branch
        );

        match self.get::<BranchProtection>(&url).await {
            Ok(protection) => Ok(Some(protection)),
            Err(e) if e.contains("404") || e.contains("Branch not protected") => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// List Dependabot alerts for a repository
    pub async fn list_dependabot_alerts(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<DependabotAlert>, String> {
        let url = format!(
            "{}/repos/{}/{}/dependabot/alerts?per_page=100&state=open",
            GITHUB_API_URL, owner, repo
        );

        match self.get_paginated::<DependabotAlert>(&url).await {
            Ok(alerts) => Ok(alerts),
            Err(e) if e.contains("403") || e.contains("404") => Ok(vec![]), // Not enabled or no access
            Err(e) => Err(e),
        }
    }

    /// List code scanning alerts for a repository
    pub async fn list_code_scanning_alerts(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<CodeScanningAlert>, String> {
        let url = format!(
            "{}/repos/{}/{}/code-scanning/alerts?per_page=100&state=open",
            GITHUB_API_URL, owner, repo
        );

        match self.get_paginated::<CodeScanningAlert>(&url).await {
            Ok(alerts) => Ok(alerts),
            Err(e) if e.contains("403") || e.contains("404") => Ok(vec![]), // Not enabled or no access
            Err(e) => Err(e),
        }
    }

    /// List secret scanning alerts for a repository
    pub async fn list_secret_scanning_alerts(
        &self,
        owner: &str,
        repo: &str,
    ) -> Result<Vec<SecretScanningAlert>, String> {
        let url = format!(
            "{}/repos/{}/{}/secret-scanning/alerts?per_page=100&state=open",
            GITHUB_API_URL, owner, repo
        );

        match self.get_paginated::<SecretScanningAlert>(&url).await {
            Ok(alerts) => Ok(alerts),
            Err(e) if e.contains("403") || e.contains("404") => Ok(vec![]), // Not enabled or no access
            Err(e) => Err(e),
        }
    }

    /// List organization members
    pub async fn list_org_members(&self, org: &str) -> Result<Vec<OrgMember>, String> {
        let url = format!("{}/orgs/{}/members?per_page=100", GITHUB_API_URL, org);
        self.get_paginated(&url).await
    }

    /// Get organization membership for a user
    pub async fn get_org_membership(
        &self,
        org: &str,
        username: &str,
    ) -> Result<OrgMembership, String> {
        let url = format!("{}/orgs/{}/memberships/{}", GITHUB_API_URL, org, username);
        self.get(&url).await
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, String> {
        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();
        let body = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("GitHub API error ({}): {}", status, body));
        }

        serde_json::from_str(&body)
            .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, &body[..body.len().min(200)]))
    }

    async fn get_paginated<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<Vec<T>, String> {
        let mut all_items = Vec::new();
        let mut next_url = Some(url.to_string());

        while let Some(current_url) = next_url {
            let response = self.client
                .get(&current_url)
                .send()
                .await
                .map_err(|e| format!("Request failed: {}", e))?;

            let status = response.status();

            // Get Link header for pagination
            let link_header = response.headers()
                .get("link")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            let body = response.text().await.map_err(|e| format!("Failed to read response: {}", e))?;

            if !status.is_success() {
                return Err(format!("GitHub API error ({}): {}", status, body));
            }

            let items: Vec<T> = serde_json::from_str(&body)
                .map_err(|e| format!("Failed to parse response: {} - Body: {}", e, &body[..body.len().min(200)]))?;

            all_items.extend(items);

            // Parse next URL from Link header
            next_url = link_header.and_then(|link| {
                for part in link.split(',') {
                    let parts: Vec<&str> = part.split(';').collect();
                    if parts.len() == 2 && parts[1].contains("rel=\"next\"") {
                        let url = parts[0].trim().trim_start_matches('<').trim_end_matches('>');
                        return Some(url.to_string());
                    }
                }
                None
            });

            // Limit to prevent runaway pagination
            if all_items.len() > 1000 {
                break;
            }
        }

        Ok(all_items)
    }
}
