use super::config::GoogleWorkspaceConfig;
use chrono::{Duration, Utc};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

const ADMIN_DIRECTORY_BASE_URL: &str = "https://admin.googleapis.com/admin/directory/v1";
const REPORTS_BASE_URL: &str = "https://admin.googleapis.com/admin/reports/v1";

/// Google Workspace Admin SDK client
pub struct GoogleWorkspaceClient {
    client: Client,
    config: GoogleWorkspaceConfig,
    access_token: String,
}

impl GoogleWorkspaceClient {
    pub async fn new(config: GoogleWorkspaceConfig) -> Result<Self, String> {
        // Get access token based on auth type
        let access_token = match &config.auth_type {
            super::config::GoogleAuthType::ServiceAccount => {
                get_service_account_token(&config).await?
            }
            super::config::GoogleAuthType::OAuth => {
                config.access_token.clone().ok_or("Access token required for OAuth auth")?
            }
        };

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            config,
            access_token,
        })
    }

    fn auth_header(&self) -> header::HeaderValue {
        header::HeaderValue::from_str(&format!("Bearer {}", self.access_token))
            .unwrap_or_else(|_| header::HeaderValue::from_static(""))
    }

    /// Get customer info to verify connection
    pub async fn get_customer(&self) -> Result<GoogleCustomer, String> {
        let url = format!(
            "{}/customers/{}",
            ADMIN_DIRECTORY_BASE_URL, self.config.customer_id
        );

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Failed to get customer info: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, body));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse customer info: {}", e))
    }

    /// List all users in the directory
    pub async fn list_users(&self) -> Result<Vec<GoogleUser>, String> {
        let mut all_users = Vec::new();
        let mut page_token: Option<String> = None;
        let max_pages = 50;
        let mut page_count = 0;

        loop {
            if page_count >= max_pages {
                tracing::warn!("Hit pagination limit of {} pages for users", max_pages);
                break;
            }

            let mut url = format!(
                "{}/users?customer={}&maxResults=500",
                ADMIN_DIRECTORY_BASE_URL, self.config.customer_id
            );

            if let Some(ref domain) = self.config.domain {
                url.push_str(&format!("&domain={}", domain));
            }

            if let Some(ref token) = page_token {
                url.push_str(&format!("&pageToken={}", token));
            }

            let response = self
                .client
                .get(&url)
                .header(header::AUTHORIZATION, self.auth_header())
                .send()
                .await
                .map_err(|e| format!("Failed to list users: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(format!("API error ({}): {}", status, body));
            }

            let result: GoogleUsersResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse users response: {}", e))?;

            if let Some(users) = result.users {
                all_users.extend(users);
            }

            page_token = result.next_page_token;
            if page_token.is_none() {
                break;
            }
            page_count += 1;
        }

        Ok(all_users)
    }

    /// List all groups
    pub async fn list_groups(&self) -> Result<Vec<GoogleGroup>, String> {
        let mut all_groups = Vec::new();
        let mut page_token: Option<String> = None;
        let max_pages = 50;
        let mut page_count = 0;

        loop {
            if page_count >= max_pages {
                tracing::warn!("Hit pagination limit of {} pages for groups", max_pages);
                break;
            }

            let mut url = format!(
                "{}/groups?customer={}&maxResults=200",
                ADMIN_DIRECTORY_BASE_URL, self.config.customer_id
            );

            if let Some(ref domain) = self.config.domain {
                url.push_str(&format!("&domain={}", domain));
            }

            if let Some(ref token) = page_token {
                url.push_str(&format!("&pageToken={}", token));
            }

            let response = self
                .client
                .get(&url)
                .header(header::AUTHORIZATION, self.auth_header())
                .send()
                .await
                .map_err(|e| format!("Failed to list groups: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(format!("API error ({}): {}", status, body));
            }

            let result: GoogleGroupsResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse groups response: {}", e))?;

            if let Some(groups) = result.groups {
                all_groups.extend(groups);
            }

            page_token = result.next_page_token;
            if page_token.is_none() {
                break;
            }
            page_count += 1;
        }

        Ok(all_groups)
    }

    /// Get group members
    pub async fn list_group_members(&self, group_key: &str) -> Result<Vec<GoogleGroupMember>, String> {
        let mut all_members = Vec::new();
        let mut page_token: Option<String> = None;
        let max_pages = 20;
        let mut page_count = 0;

        loop {
            if page_count >= max_pages {
                break;
            }

            let mut url = format!(
                "{}/groups/{}/members?maxResults=200",
                ADMIN_DIRECTORY_BASE_URL,
                urlencoding::encode(group_key)
            );

            if let Some(ref token) = page_token {
                url.push_str(&format!("&pageToken={}", token));
            }

            let response = self
                .client
                .get(&url)
                .header(header::AUTHORIZATION, self.auth_header())
                .send()
                .await
                .map_err(|e| format!("Failed to list group members: {}", e))?;

            if !response.status().is_success() {
                // Group might not have any members or permission denied
                if response.status().as_u16() == 404 {
                    return Ok(Vec::new());
                }
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(format!("API error ({}): {}", status, body));
            }

            let result: GoogleGroupMembersResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse group members response: {}", e))?;

            if let Some(members) = result.members {
                all_members.extend(members);
            }

            page_token = result.next_page_token;
            if page_token.is_none() {
                break;
            }
            page_count += 1;
        }

        Ok(all_members)
    }

    /// Get login audit activities
    pub async fn list_login_activities(&self, days: u32) -> Result<Vec<GoogleActivity>, String> {
        let start_time = (Utc::now() - Duration::days(days as i64))
            .format("%Y-%m-%dT%H:%M:%S.000Z")
            .to_string();

        self.list_activities("login", &start_time).await
    }

    /// Get admin audit activities
    pub async fn list_admin_activities(&self, days: u32) -> Result<Vec<GoogleActivity>, String> {
        let start_time = (Utc::now() - Duration::days(days as i64))
            .format("%Y-%m-%dT%H:%M:%S.000Z")
            .to_string();

        self.list_activities("admin", &start_time).await
    }

    async fn list_activities(&self, application_name: &str, start_time: &str) -> Result<Vec<GoogleActivity>, String> {
        let mut all_activities = Vec::new();
        let mut page_token: Option<String> = None;
        let max_pages = 50;
        let mut page_count = 0;

        loop {
            if page_count >= max_pages {
                tracing::warn!("Hit pagination limit of {} pages for activities", max_pages);
                break;
            }

            let mut url = format!(
                "{}/activity/users/all/applications/{}?startTime={}&maxResults=1000",
                REPORTS_BASE_URL,
                application_name,
                urlencoding::encode(start_time)
            );

            if let Some(ref token) = page_token {
                url.push_str(&format!("&pageToken={}", token));
            }

            let response = self
                .client
                .get(&url)
                .header(header::AUTHORIZATION, self.auth_header())
                .send()
                .await
                .map_err(|e| format!("Failed to list activities: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(format!("API error ({}): {}", status, body));
            }

            let result: GoogleActivitiesResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse activities response: {}", e))?;

            if let Some(items) = result.items {
                all_activities.extend(items);
            }

            page_token = result.next_page_token;
            if page_token.is_none() {
                break;
            }
            page_count += 1;
        }

        Ok(all_activities)
    }
}

/// Get an access token using service account credentials
async fn get_service_account_token(config: &GoogleWorkspaceConfig) -> Result<String, String> {
    let key_json = config.service_account_key.as_ref().ok_or("Service account key required")?;
    let admin_email = config.admin_email.as_ref().ok_or("Admin email required for impersonation")?;

    // Parse the service account key
    let key: ServiceAccountKey = serde_json::from_str(key_json)
        .map_err(|e| format!("Invalid service account key JSON: {}", e))?;

    // Create JWT assertion
    let now = Utc::now().timestamp();
    let claims = serde_json::json!({
        "iss": key.client_email,
        "sub": admin_email,
        "scope": "https://www.googleapis.com/auth/admin.directory.user.readonly https://www.googleapis.com/auth/admin.directory.group.readonly https://www.googleapis.com/auth/admin.reports.audit.readonly https://www.googleapis.com/auth/admin.directory.customer.readonly",
        "aud": "https://oauth2.googleapis.com/token",
        "iat": now,
        "exp": now + 3600,
    });

    // Service account JWT signing requires RSA private key handling
    // For now, return an error directing users to use OAuth flow
    // Full implementation would require `rsa` and `pkcs8` crates
    let _ = &key.client_email; // Mark as used
    let _ = claims; // Mark as used

    Err("Service account authentication requires JWT signing implementation. Please use OAuth flow instead.".to_string())
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct ServiceAccountKey {
    client_email: String,
    private_key: String,
    private_key_id: String,
    project_id: String,
}

// API Response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleCustomer {
    pub id: String,
    pub kind: Option<String>,
    #[serde(rename = "customerDomain")]
    pub customer_domain: Option<String>,
    #[serde(rename = "alternateEmail")]
    pub alternate_email: Option<String>,
    #[serde(rename = "postalAddress")]
    pub postal_address: Option<GooglePostalAddress>,
    #[serde(rename = "customerCreationTime")]
    pub customer_creation_time: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GooglePostalAddress {
    #[serde(rename = "organizationName")]
    pub organization_name: Option<String>,
    #[serde(rename = "addressLine1")]
    pub address_line1: Option<String>,
    pub locality: Option<String>,
    pub region: Option<String>,
    #[serde(rename = "postalCode")]
    pub postal_code: Option<String>,
    #[serde(rename = "countryCode")]
    pub country_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleUsersResponse {
    pub users: Option<Vec<GoogleUser>>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUser {
    pub id: String,
    #[serde(rename = "primaryEmail")]
    pub primary_email: String,
    pub name: GoogleUserName,
    #[serde(rename = "isAdmin")]
    pub is_admin: Option<bool>,
    #[serde(rename = "isDelegatedAdmin")]
    pub is_delegated_admin: Option<bool>,
    #[serde(rename = "isMailboxSetup")]
    pub is_mailbox_setup: Option<bool>,
    #[serde(rename = "isEnrolledIn2Sv")]
    pub is_enrolled_in_2sv: Option<bool>,
    #[serde(rename = "isEnforcedIn2Sv")]
    pub is_enforced_in_2sv: Option<bool>,
    pub suspended: Option<bool>,
    #[serde(rename = "suspensionReason")]
    pub suspension_reason: Option<String>,
    pub archived: Option<bool>,
    #[serde(rename = "creationTime")]
    pub creation_time: Option<String>,
    #[serde(rename = "lastLoginTime")]
    pub last_login_time: Option<String>,
    #[serde(rename = "agreedToTerms")]
    pub agreed_to_terms: Option<bool>,
    #[serde(rename = "changePasswordAtNextLogin")]
    pub change_password_at_next_login: Option<bool>,
    #[serde(rename = "orgUnitPath")]
    pub org_unit_path: Option<String>,
    #[serde(rename = "recoveryEmail")]
    pub recovery_email: Option<String>,
    #[serde(rename = "recoveryPhone")]
    pub recovery_phone: Option<String>,
    pub emails: Option<Vec<GoogleUserEmail>>,
    pub phones: Option<Vec<GoogleUserPhone>>,
    pub organizations: Option<Vec<GoogleUserOrganization>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserName {
    #[serde(rename = "fullName")]
    pub full_name: Option<String>,
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    #[serde(rename = "familyName")]
    pub family_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserEmail {
    pub address: String,
    pub primary: Option<bool>,
    #[serde(rename = "type")]
    pub email_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserPhone {
    pub value: String,
    #[serde(rename = "type")]
    pub phone_type: Option<String>,
    pub primary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleUserOrganization {
    pub name: Option<String>,
    pub title: Option<String>,
    pub department: Option<String>,
    pub primary: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleGroupsResponse {
    pub groups: Option<Vec<GoogleGroup>>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleGroup {
    pub id: String,
    pub email: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(rename = "directMembersCount")]
    pub direct_members_count: Option<String>,
    #[serde(rename = "adminCreated")]
    pub admin_created: Option<bool>,
    pub aliases: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleGroupMembersResponse {
    pub members: Option<Vec<GoogleGroupMember>>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleGroupMember {
    pub id: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    #[serde(rename = "type")]
    pub member_type: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleActivitiesResponse {
    pub items: Option<Vec<GoogleActivity>>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleActivity {
    pub kind: Option<String>,
    pub id: Option<GoogleActivityId>,
    pub actor: Option<GoogleActivityActor>,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    pub events: Option<Vec<GoogleActivityEvent>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleActivityId {
    pub time: Option<String>,
    #[serde(rename = "uniqueQualifier")]
    pub unique_qualifier: Option<String>,
    #[serde(rename = "applicationName")]
    pub application_name: Option<String>,
    #[serde(rename = "customerId")]
    pub customer_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleActivityActor {
    pub email: Option<String>,
    #[serde(rename = "profileId")]
    pub profile_id: Option<String>,
    #[serde(rename = "callerType")]
    pub caller_type: Option<String>,
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleActivityEvent {
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    pub name: Option<String>,
    pub parameters: Option<Vec<GoogleActivityParameter>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleActivityParameter {
    pub name: Option<String>,
    pub value: Option<String>,
    #[serde(rename = "boolValue")]
    pub bool_value: Option<bool>,
    #[serde(rename = "intValue")]
    pub int_value: Option<String>,
    #[serde(rename = "multiValue")]
    pub multi_value: Option<Vec<String>>,
}
