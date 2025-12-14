use super::config::OktaConfig;
use chrono::{Duration, Utc};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

/// Okta API client
pub struct OktaClient {
    client: Client,
    base_url: String,
}

impl OktaClient {
    pub async fn new(config: OktaConfig) -> Result<Self, String> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("SSWS {}", config.api_token))
                .map_err(|e| format!("Invalid API token format: {}", e))?,
        );
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
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            base_url: config.base_url(),
        })
    }

    /// Get organization information
    pub async fn get_org_info(&self) -> Result<OktaOrg, String> {
        let url = format!("{}/api/v1/org", self.base_url);
        let response = self.client.get(&url).send().await.map_err(|e| {
            format!("Failed to get organization info: {}", e)
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, body));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse organization info: {}", e))
    }

    /// List all users
    pub async fn list_users(&self) -> Result<Vec<OktaUser>, String> {
        self.paginate_all(&format!("{}/api/v1/users?limit=200", self.base_url))
            .await
    }

    /// List all groups
    pub async fn list_groups(&self) -> Result<Vec<OktaGroup>, String> {
        self.paginate_all(&format!("{}/api/v1/groups?limit=200", self.base_url))
            .await
    }

    /// List members of a group
    pub async fn list_group_members(&self, group_id: &str) -> Result<Vec<OktaUser>, String> {
        self.paginate_all(&format!(
            "{}/api/v1/groups/{}/users?limit=200",
            self.base_url, group_id
        ))
        .await
    }

    /// Get MFA factors for a user
    pub async fn list_user_factors(&self, user_id: &str) -> Result<Vec<OktaFactor>, String> {
        let url = format!("{}/api/v1/users/{}/factors", self.base_url, user_id);
        let response = self.client.get(&url).send().await.map_err(|e| {
            format!("Failed to get user factors: {}", e)
        })?;

        if !response.status().is_success() {
            if response.status().as_u16() == 404 {
                return Ok(Vec::new());
            }
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, body));
        }

        response
            .json()
            .await
            .map_err(|e| format!("Failed to parse user factors: {}", e))
    }

    /// List all applications
    pub async fn list_applications(&self) -> Result<Vec<OktaApplication>, String> {
        self.paginate_all(&format!(
            "{}/api/v1/apps?limit=200&expand=user",
            self.base_url
        ))
        .await
    }

    /// List application users
    pub async fn list_app_users(&self, app_id: &str) -> Result<Vec<OktaAppUser>, String> {
        self.paginate_all(&format!(
            "{}/api/v1/apps/{}/users?limit=200",
            self.base_url, app_id
        ))
        .await
    }

    /// List system logs
    pub async fn list_system_logs(&self, since_days: u32) -> Result<Vec<OktaLogEvent>, String> {
        let since = Utc::now() - Duration::days(since_days as i64);
        let since_str = since.format("%Y-%m-%dT%H:%M:%S.000Z").to_string();

        self.paginate_all(&format!(
            "{}/api/v1/logs?since={}&limit=1000",
            self.base_url, since_str
        ))
        .await
    }

    /// List system logs for security events only
    pub async fn list_security_logs(&self, since_days: u32) -> Result<Vec<OktaLogEvent>, String> {
        let since = Utc::now() - Duration::days(since_days as i64);
        let since_str = since.format("%Y-%m-%dT%H:%M:%S.000Z").to_string();

        // Filter for security-relevant events
        let filter = r#"eventType sw "user." or eventType sw "security." or eventType sw "policy.""#;
        let encoded_filter = urlencoding::encode(filter);

        self.paginate_all(&format!(
            "{}/api/v1/logs?since={}&filter={}&limit=1000",
            self.base_url, since_str, encoded_filter
        ))
        .await
    }

    /// Paginate through all results
    async fn paginate_all<T: for<'de> Deserialize<'de>>(&self, initial_url: &str) -> Result<Vec<T>, String> {
        let mut all_items = Vec::new();
        let mut url = Some(initial_url.to_string());
        let mut page_count = 0;
        const MAX_PAGES: usize = 100; // Safety limit

        while let Some(current_url) = url {
            if page_count >= MAX_PAGES {
                tracing::warn!("Hit pagination limit of {} pages", MAX_PAGES);
                break;
            }

            let response = self.client.get(&current_url).send().await.map_err(|e| {
                format!("Failed to fetch page: {}", e)
            })?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(format!("API error ({}): {}", status, body));
            }

            // Check for next link
            url = response
                .headers()
                .get("link")
                .and_then(|h| h.to_str().ok())
                .and_then(|links| parse_next_link(links));

            let items: Vec<T> = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            all_items.extend(items);
            page_count += 1;
        }

        Ok(all_items)
    }
}

/// Parse the next link from the Link header
fn parse_next_link(link_header: &str) -> Option<String> {
    for link in link_header.split(',') {
        let parts: Vec<&str> = link.split(';').collect();
        if parts.len() >= 2 {
            let url_part = parts[0].trim();
            let rel_part = parts[1].trim();
            if rel_part.contains("rel=\"next\"") {
                // Extract URL from <url>
                let url = url_part.trim_start_matches('<').trim_end_matches('>');
                return Some(url.to_string());
            }
        }
    }
    None
}

// Okta API response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaOrg {
    pub id: String,
    #[serde(rename = "companyName")]
    pub company_name: Option<String>,
    pub status: Option<String>,
    pub subdomain: Option<String>,
    pub website: Option<String>,
    #[serde(rename = "endUserSupportHelpURL")]
    pub end_user_support_help_url: Option<String>,
    #[serde(rename = "supportPhoneNumber")]
    pub support_phone_number: Option<String>,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "postalCode")]
    pub postal_code: Option<String>,
    pub created: Option<String>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaUser {
    pub id: String,
    pub status: String,
    pub created: String,
    #[serde(rename = "lastLogin")]
    pub last_login: Option<String>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
    #[serde(rename = "passwordChanged")]
    pub password_changed: Option<String>,
    pub profile: OktaUserProfile,
    #[serde(rename = "statusChanged")]
    pub status_changed: Option<String>,
    #[serde(rename = "type")]
    pub user_type: Option<OktaUserType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaUserType {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaUserProfile {
    pub login: String,
    pub email: String,
    #[serde(rename = "firstName")]
    pub first_name: Option<String>,
    #[serde(rename = "lastName")]
    pub last_name: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "nickName")]
    pub nick_name: Option<String>,
    #[serde(rename = "mobilePhone")]
    pub mobile_phone: Option<String>,
    #[serde(rename = "secondEmail")]
    pub second_email: Option<String>,
    pub department: Option<String>,
    pub title: Option<String>,
    pub manager: Option<String>,
    #[serde(rename = "employeeNumber")]
    pub employee_number: Option<String>,
    #[serde(rename = "organization")]
    pub organization: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaGroup {
    pub id: String,
    pub created: String,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
    #[serde(rename = "lastMembershipUpdated")]
    pub last_membership_updated: Option<String>,
    #[serde(rename = "type")]
    pub group_type: String,
    pub profile: OktaGroupProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaGroupProfile {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaFactor {
    pub id: String,
    #[serde(rename = "factorType")]
    pub factor_type: String,
    pub provider: String,
    #[serde(rename = "vendorName")]
    pub vendor_name: Option<String>,
    pub status: String,
    pub created: Option<String>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
    pub profile: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaApplication {
    pub id: String,
    pub name: String,
    pub label: String,
    pub status: String,
    pub created: Option<String>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
    pub features: Option<Vec<String>>,
    #[serde(rename = "signOnMode")]
    pub sign_on_mode: Option<String>,
    pub visibility: Option<OktaAppVisibility>,
    pub credentials: Option<OktaAppCredentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaAppVisibility {
    #[serde(rename = "autoSubmitToolbar")]
    pub auto_submit_toolbar: Option<bool>,
    pub hide: Option<OktaAppHide>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaAppHide {
    #[serde(rename = "iOS")]
    pub ios: Option<bool>,
    pub web: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaAppCredentials {
    pub scheme: Option<String>,
    #[serde(rename = "userNameTemplate")]
    pub user_name_template: Option<OktaUserNameTemplate>,
    pub signing: Option<OktaAppSigning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaUserNameTemplate {
    pub template: Option<String>,
    #[serde(rename = "type")]
    pub template_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaAppSigning {
    pub kid: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaAppUser {
    pub id: String,
    pub scope: String,
    pub status: String,
    #[serde(rename = "statusChanged")]
    pub status_changed: Option<String>,
    pub created: Option<String>,
    #[serde(rename = "lastUpdated")]
    pub last_updated: Option<String>,
    #[serde(rename = "syncState")]
    pub sync_state: Option<String>,
    pub credentials: Option<OktaAppUserCredentials>,
    pub profile: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaAppUserCredentials {
    #[serde(rename = "userName")]
    pub user_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogEvent {
    pub uuid: String,
    pub published: String,
    #[serde(rename = "eventType")]
    pub event_type: String,
    pub version: String,
    pub severity: String,
    #[serde(rename = "legacyEventType")]
    pub legacy_event_type: Option<String>,
    #[serde(rename = "displayMessage")]
    pub display_message: Option<String>,
    pub actor: Option<OktaLogActor>,
    pub client: Option<OktaLogClient>,
    pub outcome: Option<OktaLogOutcome>,
    pub target: Option<Vec<OktaLogTarget>>,
    pub transaction: Option<OktaLogTransaction>,
    #[serde(rename = "debugContext")]
    pub debug_context: Option<OktaLogDebugContext>,
    #[serde(rename = "authenticationContext")]
    pub authentication_context: Option<OktaLogAuthenticationContext>,
    #[serde(rename = "securityContext")]
    pub security_context: Option<OktaLogSecurityContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogActor {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub actor_type: Option<String>,
    #[serde(rename = "alternateId")]
    pub alternate_id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogClient {
    #[serde(rename = "userAgent")]
    pub user_agent: Option<OktaLogUserAgent>,
    pub zone: Option<String>,
    pub device: Option<String>,
    pub id: Option<String>,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    #[serde(rename = "geographicalContext")]
    pub geographical_context: Option<OktaLogGeographicalContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogUserAgent {
    #[serde(rename = "rawUserAgent")]
    pub raw_user_agent: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogGeographicalContext {
    pub city: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    #[serde(rename = "postalCode")]
    pub postal_code: Option<String>,
    pub geolocation: Option<OktaLogGeolocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogGeolocation {
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogOutcome {
    pub result: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogTarget {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub target_type: Option<String>,
    #[serde(rename = "alternateId")]
    pub alternate_id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogTransaction {
    #[serde(rename = "type")]
    pub transaction_type: Option<String>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogDebugContext {
    #[serde(rename = "debugData")]
    pub debug_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogAuthenticationContext {
    #[serde(rename = "authenticationProvider")]
    pub authentication_provider: Option<String>,
    #[serde(rename = "credentialProvider")]
    pub credential_provider: Option<String>,
    #[serde(rename = "credentialType")]
    pub credential_type: Option<String>,
    #[serde(rename = "externalSessionId")]
    pub external_session_id: Option<String>,
    pub interface: Option<String>,
    #[serde(rename = "authenticationStep")]
    pub authentication_step: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OktaLogSecurityContext {
    #[serde(rename = "asNumber")]
    pub as_number: Option<i64>,
    #[serde(rename = "asOrg")]
    pub as_org: Option<String>,
    pub isp: Option<String>,
    pub domain: Option<String>,
    #[serde(rename = "isProxy")]
    pub is_proxy: Option<bool>,
}
