use super::config::{JiraAuthMethod, JiraConfig};
use base64::Engine;
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

/// Jira API client
#[derive(Clone)]
pub struct JiraClient {
    client: Client,
    config: JiraConfig,
}

// API Response types
#[derive(Debug, Serialize, Deserialize)]
pub struct JiraUser {
    #[serde(rename = "accountId")]
    pub account_id: String,
    #[serde(rename = "accountType")]
    pub account_type: Option<String>,
    #[serde(rename = "emailAddress")]
    pub email_address: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub active: bool,
    #[serde(rename = "avatarUrls")]
    pub avatar_urls: Option<AvatarUrls>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AvatarUrls {
    #[serde(rename = "48x48")]
    pub large: Option<String>,
    #[serde(rename = "24x24")]
    pub small: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraProject {
    pub id: String,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "projectTypeKey")]
    pub project_type_key: String,
    pub lead: Option<JiraUser>,
    #[serde(rename = "avatarUrls")]
    pub avatar_urls: Option<AvatarUrls>,
    pub simplified: Option<bool>,
    pub style: Option<String>,
    #[serde(rename = "isPrivate")]
    pub is_private: Option<bool>,
    pub archived: Option<bool>,
    #[serde(rename = "archivedDate")]
    pub archived_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectsResponse {
    pub values: Vec<JiraProject>,
    #[serde(rename = "maxResults")]
    pub max_results: i32,
    #[serde(rename = "startAt")]
    pub start_at: i32,
    pub total: i32,
    #[serde(rename = "isLast")]
    pub is_last: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JiraIssue {
    pub id: String,
    pub key: String,
    #[serde(rename = "self")]
    pub self_link: String,
    pub fields: IssueFields,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueFields {
    pub summary: String,
    pub description: Option<serde_json::Value>,
    #[serde(rename = "issuetype")]
    pub issue_type: IssueType,
    pub status: IssueStatus,
    pub priority: Option<IssuePriority>,
    pub project: IssueProject,
    pub assignee: Option<JiraUser>,
    pub reporter: Option<JiraUser>,
    pub created: String,
    pub updated: String,
    pub labels: Option<Vec<String>>,
    #[serde(rename = "securitylevel")]
    pub security_level: Option<SecurityLevel>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueType {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
    pub subtask: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueStatus {
    pub id: String,
    pub name: String,
    #[serde(rename = "statusCategory")]
    pub status_category: StatusCategory,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StatusCategory {
    pub id: i32,
    pub key: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssuePriority {
    pub id: String,
    pub name: String,
    #[serde(rename = "iconUrl")]
    pub icon_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IssueProject {
    pub id: String,
    pub key: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityLevel {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SearchResponse {
    pub issues: Vec<JiraIssue>,
    #[serde(rename = "maxResults")]
    pub max_results: i32,
    #[serde(rename = "startAt")]
    pub start_at: i32,
    pub total: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsersResponse {
    pub values: Option<Vec<JiraUser>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectPermissions {
    pub permissions: Vec<PermissionGrant>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionGrant {
    pub id: i64,
    pub holder: Option<PermissionHolder>,
    pub permission: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PermissionHolder {
    #[serde(rename = "type")]
    pub holder_type: String,
    pub parameter: Option<String>,
    pub value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectRole {
    pub id: i64,
    pub name: String,
    pub description: Option<String>,
    pub actors: Option<Vec<RoleActor>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleActor {
    pub id: i64,
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "type")]
    pub actor_type: String,
    pub name: Option<String>,
    #[serde(rename = "actorUser")]
    pub actor_user: Option<ActorUser>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActorUser {
    #[serde(rename = "accountId")]
    pub account_id: String,
}

impl JiraClient {
    pub async fn new(config: JiraConfig) -> Result<Self, String> {
        let mut headers = header::HeaderMap::new();

        // Set up authentication based on method
        match config.auth_method {
            JiraAuthMethod::ApiToken => {
                let email = config
                    .email
                    .as_ref()
                    .ok_or("Email required for API token auth")?;
                let credentials = format!("{}:{}", email, config.access_token);
                let encoded =
                    base64::engine::general_purpose::STANDARD.encode(credentials.as_bytes());
                headers.insert(
                    header::AUTHORIZATION,
                    header::HeaderValue::from_str(&format!("Basic {}", encoded))
                        .map_err(|e| format!("Invalid credentials: {}", e))?,
                );
            }
            JiraAuthMethod::OAuth => {
                headers.insert(
                    header::AUTHORIZATION,
                    header::HeaderValue::from_str(&format!("Bearer {}", config.access_token))
                        .map_err(|e| format!("Invalid token: {}", e))?,
                );
            }
        }

        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self { client, config })
    }

    /// Get the authenticated user
    pub async fn get_myself(&self) -> Result<JiraUser, String> {
        let url = format!("{}/myself", self.config.api_url());
        self.get(&url).await
    }

    /// List all accessible projects
    pub async fn list_projects(&self) -> Result<Vec<JiraProject>, String> {
        let mut all_projects = Vec::new();
        let mut start_at = 0;
        let max_results = 50;

        loop {
            let url = format!(
                "{}/project/search?startAt={}&maxResults={}&expand=lead,description",
                self.config.api_url(),
                start_at,
                max_results
            );

            let response: ProjectsResponse = self.get(&url).await?;
            all_projects.extend(response.values);

            if response.is_last.unwrap_or(true)
                || all_projects.len() >= response.total as usize
            {
                break;
            }

            start_at += max_results;

            // Safety limit
            if all_projects.len() > 500 {
                break;
            }
        }

        Ok(all_projects)
    }

    /// Search for issues using JQL
    pub async fn search_issues(&self, jql: &str, max_results: i32) -> Result<Vec<JiraIssue>, String> {
        let mut all_issues = Vec::new();
        let mut start_at = 0;
        let page_size = max_results.min(100);

        loop {
            let url = format!(
                "{}/search?jql={}&startAt={}&maxResults={}&fields=summary,description,issuetype,status,priority,project,assignee,reporter,created,updated,labels,securitylevel",
                self.config.api_url(),
                urlencoding::encode(jql),
                start_at,
                page_size
            );

            let response: SearchResponse = self.get(&url).await?;
            all_issues.extend(response.issues);

            if all_issues.len() >= response.total as usize || all_issues.len() >= max_results as usize {
                break;
            }

            start_at += page_size;

            // Safety limit
            if all_issues.len() > 1000 {
                break;
            }
        }

        Ok(all_issues)
    }

    /// List users assignable to a project
    pub async fn list_project_users(&self, project_key: &str) -> Result<Vec<JiraUser>, String> {
        let url = format!(
            "{}/user/assignable/search?project={}",
            self.config.api_url(),
            project_key
        );

        // This endpoint returns an array directly
        self.get(&url).await
    }

    /// Get project roles for a project
    pub async fn get_project_roles(&self, project_key: &str) -> Result<Vec<ProjectRole>, String> {
        // First get the list of roles (returns a map of name -> URL)
        let url = format!(
            "{}/project/{}/role",
            self.config.api_url(),
            project_key
        );

        let role_urls: std::collections::HashMap<String, String> = self.get(&url).await?;
        let mut roles = Vec::new();

        // Fetch each role's details
        for (_name, role_url) in role_urls {
            match self.get::<ProjectRole>(&role_url).await {
                Ok(role) => roles.push(role),
                Err(e) => {
                    tracing::warn!(error = %e, url = %role_url, "Failed to get project role details");
                }
            }
        }

        Ok(roles)
    }

    /// Get server info (useful for connection test)
    pub async fn get_server_info(&self) -> Result<ServerInfo, String> {
        let url = format!("{}/serverInfo", self.config.api_url());
        self.get(&url).await
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<T, String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let status = response.status();
        let body = response
            .text()
            .await
            .map_err(|e| format!("Failed to read response: {}", e))?;

        if !status.is_success() {
            return Err(format!("Jira API error ({}): {}", status, body));
        }

        serde_json::from_str(&body).map_err(|e| {
            format!(
                "Failed to parse response: {} - Body: {}",
                e,
                &body[..body.len().min(200)]
            )
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerInfo {
    #[serde(rename = "baseUrl")]
    pub base_url: String,
    pub version: String,
    #[serde(rename = "deploymentType")]
    pub deployment_type: String,
    #[serde(rename = "scmInfo")]
    pub scm_info: Option<String>,
    #[serde(rename = "serverTitle")]
    pub server_title: Option<String>,
}
