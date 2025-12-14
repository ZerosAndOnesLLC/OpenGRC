use super::config::AzureAdConfig;
use chrono::{Duration, Utc};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};

const GRAPH_API_BASE_URL: &str = "https://graph.microsoft.com/v1.0";
const GRAPH_API_BETA_URL: &str = "https://graph.microsoft.com/beta";

/// Azure AD (Microsoft Graph) client
pub struct AzureAdClient {
    client: Client,
    access_token: String,
}

impl AzureAdClient {
    pub async fn new(config: AzureAdConfig) -> Result<Self, String> {
        // Get access token
        let access_token = if let Some(ref token) = config.access_token {
            token.clone()
        } else if let Some(ref client_secret) = config.client_secret {
            get_client_credentials_token(&config, client_secret).await?
        } else {
            return Err("Either access token or client secret is required".to_string());
        };

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        Ok(Self {
            client,
            access_token,
        })
    }

    fn auth_header(&self) -> header::HeaderValue {
        header::HeaderValue::from_str(&format!("Bearer {}", self.access_token))
            .unwrap_or_else(|_| header::HeaderValue::from_static(""))
    }

    /// Get organization info
    pub async fn get_organization(&self) -> Result<AzureOrganization, String> {
        let url = format!("{}/organization", GRAPH_API_BASE_URL);

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Failed to get organization info: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("API error ({}): {}", status, body));
        }

        let result: GraphResponse<Vec<AzureOrganization>> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse organization info: {}", e))?;

        result
            .value
            .into_iter()
            .next()
            .ok_or_else(|| "No organization found".to_string())
    }

    /// List all users
    pub async fn list_users(&self) -> Result<Vec<AzureUser>, String> {
        self.paginate_all(&format!(
            "{}/users?$select=id,displayName,userPrincipalName,mail,givenName,surname,jobTitle,department,accountEnabled,createdDateTime,lastSignInDateTime,signInActivity,userType,assignedLicenses&$top=999",
            GRAPH_API_BETA_URL // Using beta for signInActivity
        ))
        .await
    }

    /// List all groups
    pub async fn list_groups(&self) -> Result<Vec<AzureGroup>, String> {
        self.paginate_all(&format!(
            "{}/groups?$select=id,displayName,description,groupTypes,securityEnabled,mailEnabled,mail,createdDateTime,membershipRule&$top=999",
            GRAPH_API_BASE_URL
        ))
        .await
    }

    /// Get group members
    pub async fn list_group_members(&self, group_id: &str) -> Result<Vec<AzureDirectoryObject>, String> {
        self.paginate_all(&format!(
            "{}/groups/{}/members?$select=id,displayName,userPrincipalName,mail&$top=999",
            GRAPH_API_BASE_URL, group_id
        ))
        .await
    }

    /// List conditional access policies
    pub async fn list_conditional_access_policies(&self) -> Result<Vec<AzureConditionalAccessPolicy>, String> {
        let url = format!("{}/identity/conditionalAccess/policies", GRAPH_API_BASE_URL);

        let response = self
            .client
            .get(&url)
            .header(header::AUTHORIZATION, self.auth_header())
            .send()
            .await
            .map_err(|e| format!("Failed to list conditional access policies: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            // Might not have permission for this
            if status.as_u16() == 403 {
                return Ok(Vec::new());
            }
            return Err(format!("API error ({}): {}", status, body));
        }

        let result: GraphResponse<Vec<AzureConditionalAccessPolicy>> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse conditional access policies: {}", e))?;

        Ok(result.value)
    }

    /// List sign-in logs
    pub async fn list_sign_in_logs(&self, days: u32) -> Result<Vec<AzureSignInLog>, String> {
        let since = (Utc::now() - Duration::days(days as i64))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();

        self.paginate_all(&format!(
            "{}/auditLogs/signIns?$filter=createdDateTime ge {}&$top=1000&$orderby=createdDateTime desc",
            GRAPH_API_BASE_URL,
            urlencoding::encode(&since)
        ))
        .await
    }

    /// List audit logs
    pub async fn list_audit_logs(&self, days: u32) -> Result<Vec<AzureAuditLog>, String> {
        let since = (Utc::now() - Duration::days(days as i64))
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();

        self.paginate_all(&format!(
            "{}/auditLogs/directoryAudits?$filter=activityDateTime ge {}&$top=1000&$orderby=activityDateTime desc",
            GRAPH_API_BASE_URL,
            urlencoding::encode(&since)
        ))
        .await
    }

    /// Paginate through all results
    async fn paginate_all<T: for<'de> Deserialize<'de>>(&self, initial_url: &str) -> Result<Vec<T>, String> {
        let mut all_items = Vec::new();
        let mut url = Some(initial_url.to_string());
        let mut page_count = 0;
        const MAX_PAGES: usize = 100;

        while let Some(current_url) = url {
            if page_count >= MAX_PAGES {
                tracing::warn!("Hit pagination limit of {} pages", MAX_PAGES);
                break;
            }

            let response = self
                .client
                .get(&current_url)
                .header(header::AUTHORIZATION, self.auth_header())
                .send()
                .await
                .map_err(|e| format!("Failed to fetch page: {}", e))?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(format!("API error ({}): {}", status, body));
            }

            let result: GraphResponse<Vec<T>> = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse response: {}", e))?;

            all_items.extend(result.value);
            url = result.odata_next_link;
            page_count += 1;
        }

        Ok(all_items)
    }
}

/// Get access token using client credentials flow
async fn get_client_credentials_token(config: &AzureAdConfig, client_secret: &str) -> Result<String, String> {
    let client = Client::new();

    let params = [
        ("client_id", config.client_id.as_str()),
        ("client_secret", client_secret),
        ("scope", "https://graph.microsoft.com/.default"),
        ("grant_type", "client_credentials"),
    ];

    let response = client
        .post(&config.token_url())
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("Failed to get access token: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Token error ({}): {}", status, body));
    }

    let token_response: TokenResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token response: {}", e))?;

    Ok(token_response.access_token)
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    #[allow(dead_code)]
    expires_in: i64,
}

// API Response types

#[derive(Debug, Deserialize)]
struct GraphResponse<T> {
    pub value: T,
    #[serde(rename = "@odata.nextLink")]
    pub odata_next_link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureOrganization {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "verifiedDomains")]
    pub verified_domains: Option<Vec<AzureVerifiedDomain>>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
    #[serde(rename = "tenantType")]
    pub tenant_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureVerifiedDomain {
    pub name: Option<String>,
    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,
    #[serde(rename = "isInitial")]
    pub is_initial: Option<bool>,
    #[serde(rename = "type")]
    pub domain_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureUser {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: Option<String>,
    pub mail: Option<String>,
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    pub surname: Option<String>,
    #[serde(rename = "jobTitle")]
    pub job_title: Option<String>,
    pub department: Option<String>,
    #[serde(rename = "accountEnabled")]
    pub account_enabled: Option<bool>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
    #[serde(rename = "userType")]
    pub user_type: Option<String>,
    #[serde(rename = "signInActivity")]
    pub sign_in_activity: Option<AzureSignInActivity>,
    #[serde(rename = "assignedLicenses")]
    pub assigned_licenses: Option<Vec<AzureLicense>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureSignInActivity {
    #[serde(rename = "lastSignInDateTime")]
    pub last_sign_in_date_time: Option<String>,
    #[serde(rename = "lastSignInRequestId")]
    pub last_sign_in_request_id: Option<String>,
    #[serde(rename = "lastNonInteractiveSignInDateTime")]
    pub last_non_interactive_sign_in_date_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureLicense {
    #[serde(rename = "skuId")]
    pub sku_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureGroup {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "groupTypes")]
    pub group_types: Option<Vec<String>>,
    #[serde(rename = "securityEnabled")]
    pub security_enabled: Option<bool>,
    #[serde(rename = "mailEnabled")]
    pub mail_enabled: Option<bool>,
    pub mail: Option<String>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
    #[serde(rename = "membershipRule")]
    pub membership_rule: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureDirectoryObject {
    #[serde(rename = "@odata.type")]
    pub odata_type: Option<String>,
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: Option<String>,
    pub mail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConditionalAccessPolicy {
    pub id: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub state: Option<String>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
    #[serde(rename = "modifiedDateTime")]
    pub modified_date_time: Option<String>,
    pub conditions: Option<AzureConditionalAccessConditions>,
    #[serde(rename = "grantControls")]
    pub grant_controls: Option<AzureGrantControls>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConditionalAccessConditions {
    pub users: Option<AzureConditionalAccessUsers>,
    pub applications: Option<AzureConditionalAccessApplications>,
    pub locations: Option<AzureConditionalAccessLocations>,
    #[serde(rename = "signInRiskLevels")]
    pub sign_in_risk_levels: Option<Vec<String>>,
    #[serde(rename = "userRiskLevels")]
    pub user_risk_levels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConditionalAccessUsers {
    #[serde(rename = "includeUsers")]
    pub include_users: Option<Vec<String>>,
    #[serde(rename = "excludeUsers")]
    pub exclude_users: Option<Vec<String>>,
    #[serde(rename = "includeGroups")]
    pub include_groups: Option<Vec<String>>,
    #[serde(rename = "excludeGroups")]
    pub exclude_groups: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConditionalAccessApplications {
    #[serde(rename = "includeApplications")]
    pub include_applications: Option<Vec<String>>,
    #[serde(rename = "excludeApplications")]
    pub exclude_applications: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConditionalAccessLocations {
    #[serde(rename = "includeLocations")]
    pub include_locations: Option<Vec<String>>,
    #[serde(rename = "excludeLocations")]
    pub exclude_locations: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureGrantControls {
    pub operator: Option<String>,
    #[serde(rename = "builtInControls")]
    pub built_in_controls: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureSignInLog {
    pub id: String,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
    #[serde(rename = "userDisplayName")]
    pub user_display_name: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "appDisplayName")]
    pub app_display_name: Option<String>,
    #[serde(rename = "appId")]
    pub app_id: Option<String>,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
    pub status: Option<AzureSignInStatus>,
    #[serde(rename = "conditionalAccessStatus")]
    pub conditional_access_status: Option<String>,
    #[serde(rename = "isInteractive")]
    pub is_interactive: Option<bool>,
    #[serde(rename = "riskDetail")]
    pub risk_detail: Option<String>,
    #[serde(rename = "riskLevelAggregated")]
    pub risk_level_aggregated: Option<String>,
    #[serde(rename = "riskLevelDuringSignIn")]
    pub risk_level_during_sign_in: Option<String>,
    pub location: Option<AzureSignInLocation>,
    #[serde(rename = "deviceDetail")]
    pub device_detail: Option<AzureDeviceDetail>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureSignInStatus {
    #[serde(rename = "errorCode")]
    pub error_code: Option<i64>,
    #[serde(rename = "failureReason")]
    pub failure_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureSignInLocation {
    pub city: Option<String>,
    pub state: Option<String>,
    #[serde(rename = "countryOrRegion")]
    pub country_or_region: Option<String>,
    #[serde(rename = "geoCoordinates")]
    pub geo_coordinates: Option<AzureGeoCoordinates>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureGeoCoordinates {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureDeviceDetail {
    #[serde(rename = "deviceId")]
    pub device_id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "operatingSystem")]
    pub operating_system: Option<String>,
    pub browser: Option<String>,
    #[serde(rename = "isCompliant")]
    pub is_compliant: Option<bool>,
    #[serde(rename = "isManaged")]
    pub is_managed: Option<bool>,
    #[serde(rename = "trustType")]
    pub trust_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureAuditLog {
    pub id: String,
    #[serde(rename = "activityDateTime")]
    pub activity_date_time: Option<String>,
    #[serde(rename = "activityDisplayName")]
    pub activity_display_name: Option<String>,
    #[serde(rename = "operationType")]
    pub operation_type: Option<String>,
    pub category: Option<String>,
    pub result: Option<String>,
    #[serde(rename = "resultReason")]
    pub result_reason: Option<String>,
    #[serde(rename = "initiatedBy")]
    pub initiated_by: Option<AzureAuditInitiatedBy>,
    #[serde(rename = "targetResources")]
    pub target_resources: Option<Vec<AzureTargetResource>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureAuditInitiatedBy {
    pub user: Option<AzureAuditUser>,
    pub app: Option<AzureAuditApp>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureAuditUser {
    pub id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "userPrincipalName")]
    pub user_principal_name: Option<String>,
    #[serde(rename = "ipAddress")]
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureAuditApp {
    #[serde(rename = "appId")]
    pub app_id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "servicePrincipalId")]
    pub service_principal_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureTargetResource {
    pub id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "type")]
    pub resource_type: Option<String>,
    #[serde(rename = "modifiedProperties")]
    pub modified_properties: Option<Vec<AzureModifiedProperty>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureModifiedProperty {
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    #[serde(rename = "oldValue")]
    pub old_value: Option<String>,
    #[serde(rename = "newValue")]
    pub new_value: Option<String>,
}
