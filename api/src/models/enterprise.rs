use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// PERMISSIONS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Permission {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub resource: String,
    pub action: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionGroup {
    pub resource: String,
    pub permissions: Vec<Permission>,
}

// ============================================================================
// ROLES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Role {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleWithPermissions {
    #[serde(flatten)]
    pub role: Role,
    pub permissions: Vec<Permission>,
    pub user_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRole {
    pub name: String,
    pub description: Option<String>,
    pub permission_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRole {
    pub name: Option<String>,
    pub description: Option<String>,
    pub permission_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RolePermission {
    pub id: Uuid,
    pub role_id: Uuid,
    pub permission_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// USER ROLES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: Uuid,
    pub role_id: Uuid,
    pub assigned_by: Option<Uuid>,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithRoles {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub tv_user_id: String,
    pub email: String,
    pub name: String,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub roles: Vec<Role>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignRolesRequest {
    pub role_ids: Vec<Uuid>,
}

// ============================================================================
// SSO / SAML CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SsoConfiguration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub provider_type: String, // 'saml', 'oidc'
    pub name: String,
    pub is_enabled: bool,
    pub is_enforced: bool,

    // SAML fields
    pub idp_entity_id: Option<String>,
    pub idp_sso_url: Option<String>,
    pub idp_slo_url: Option<String>,
    pub idp_certificate: Option<String>,
    pub sp_entity_id: Option<String>,
    pub sp_acs_url: Option<String>,
    pub sp_slo_url: Option<String>,

    pub attribute_mappings: serde_json::Value,

    // OIDC fields
    pub oidc_client_id: Option<String>,
    pub oidc_client_secret_encrypted: Option<String>,
    pub oidc_issuer_url: Option<String>,
    pub oidc_scopes: Option<Vec<String>>,

    // Security
    pub require_signed_assertions: Option<bool>,
    pub require_encrypted_assertions: Option<bool>,
    pub allowed_clock_skew_seconds: Option<i32>,

    // Auto-provisioning
    pub auto_provision_users: Option<bool>,
    pub default_role_id: Option<Uuid>,

    // Metadata
    pub metadata_url: Option<String>,
    pub metadata_xml: Option<String>,
    pub last_metadata_refresh: Option<DateTime<Utc>>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoConfigurationResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub provider_type: String,
    pub name: String,
    pub is_enabled: bool,
    pub is_enforced: bool,
    pub idp_entity_id: Option<String>,
    pub idp_sso_url: Option<String>,
    pub idp_slo_url: Option<String>,
    pub has_certificate: bool,
    pub sp_entity_id: Option<String>,
    pub sp_acs_url: Option<String>,
    pub sp_slo_url: Option<String>,
    pub attribute_mappings: serde_json::Value,
    pub auto_provision_users: Option<bool>,
    pub default_role_id: Option<Uuid>,
    pub metadata_url: Option<String>,
    pub last_metadata_refresh: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub domains: Vec<SsoDomain>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSsoConfiguration {
    pub provider_type: String,
    pub name: String,
    pub idp_entity_id: Option<String>,
    pub idp_sso_url: Option<String>,
    pub idp_slo_url: Option<String>,
    pub idp_certificate: Option<String>,
    pub attribute_mappings: Option<serde_json::Value>,
    pub auto_provision_users: Option<bool>,
    pub default_role_id: Option<Uuid>,
    pub metadata_url: Option<String>,
    pub metadata_xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSsoConfiguration {
    pub name: Option<String>,
    pub is_enabled: Option<bool>,
    pub is_enforced: Option<bool>,
    pub idp_entity_id: Option<String>,
    pub idp_sso_url: Option<String>,
    pub idp_slo_url: Option<String>,
    pub idp_certificate: Option<String>,
    pub attribute_mappings: Option<serde_json::Value>,
    pub auto_provision_users: Option<bool>,
    pub default_role_id: Option<Uuid>,
    pub metadata_url: Option<String>,
    pub metadata_xml: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SsoDomain {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub sso_configuration_id: Uuid,
    pub domain: String,
    pub is_verified: bool,
    pub verification_token: Option<String>,
    pub verification_method: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddSsoDomain {
    pub domain: String,
    pub verification_method: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SsoSession {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub session_index: Option<String>,
    pub name_id: Option<String>,
    pub name_id_format: Option<String>,
    pub idp_session_id: Option<String>,
    pub attributes: Option<serde_json::Value>,
    pub valid_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// SAML SP Metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamlSpMetadata {
    pub entity_id: String,
    pub acs_url: String,
    pub slo_url: String,
    pub metadata_xml: String,
}

// ============================================================================
// SCIM CONFIGURATION
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScimConfiguration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub is_enabled: bool,
    pub base_url: Option<String>,
    pub bearer_token_hash: Option<String>,
    pub token_created_at: Option<DateTime<Utc>>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub auto_activate_users: Option<bool>,
    pub default_role_id: Option<Uuid>,
    pub sync_groups_as_roles: Option<bool>,
    pub group_role_mappings: serde_json::Value,
    pub on_user_deactivate: Option<String>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub total_users_provisioned: Option<i32>,
    pub total_groups_provisioned: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimConfigurationResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub is_enabled: bool,
    pub base_url: Option<String>,
    pub has_token: bool,
    pub token_created_at: Option<DateTime<Utc>>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub auto_activate_users: Option<bool>,
    pub default_role_id: Option<Uuid>,
    pub sync_groups_as_roles: Option<bool>,
    pub group_role_mappings: serde_json::Value,
    pub on_user_deactivate: Option<String>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub total_users_provisioned: Option<i32>,
    pub total_groups_provisioned: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateScimConfiguration {
    pub auto_activate_users: Option<bool>,
    pub default_role_id: Option<Uuid>,
    pub sync_groups_as_roles: Option<bool>,
    pub group_role_mappings: Option<serde_json::Value>,
    pub on_user_deactivate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateScimConfiguration {
    pub is_enabled: Option<bool>,
    pub auto_activate_users: Option<bool>,
    pub default_role_id: Option<Uuid>,
    pub sync_groups_as_roles: Option<bool>,
    pub group_role_mappings: Option<serde_json::Value>,
    pub on_user_deactivate: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateScimTokenResponse {
    pub token: String, // Only returned once
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScimUser {
    pub id: Uuid,
    pub scim_configuration_id: Uuid,
    pub user_id: Option<Uuid>,
    pub external_id: String,
    pub scim_id: String,
    pub user_name: String,
    pub active: bool,
    pub display_name: Option<String>,
    pub emails: Option<serde_json::Value>,
    pub name: Option<serde_json::Value>,
    pub groups: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ScimGroup {
    pub id: Uuid,
    pub scim_configuration_id: Uuid,
    pub role_id: Option<Uuid>,
    pub external_id: Option<String>,
    pub scim_id: String,
    pub display_name: String,
    pub members: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// SCIM 2.0 Request/Response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimListResponse<T> {
    pub schemas: Vec<String>,
    #[serde(rename = "totalResults")]
    pub total_results: i64,
    #[serde(rename = "startIndex")]
    pub start_index: i64,
    #[serde(rename = "itemsPerPage")]
    pub items_per_page: i64,
    #[serde(rename = "Resources")]
    pub resources: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimUserResource {
    pub schemas: Vec<String>,
    pub id: String,
    #[serde(rename = "externalId")]
    pub external_id: Option<String>,
    #[serde(rename = "userName")]
    pub user_name: String,
    pub name: Option<ScimName>,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub emails: Option<Vec<ScimEmail>>,
    pub active: bool,
    pub groups: Option<Vec<ScimGroupRef>>,
    pub meta: ScimMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimName {
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    #[serde(rename = "familyName")]
    pub family_name: Option<String>,
    pub formatted: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimEmail {
    pub value: String,
    pub primary: Option<bool>,
    #[serde(rename = "type")]
    pub email_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroupRef {
    pub value: String,
    #[serde(rename = "$ref")]
    pub ref_url: Option<String>,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroupResource {
    pub schemas: Vec<String>,
    pub id: String,
    #[serde(rename = "externalId")]
    pub external_id: Option<String>,
    #[serde(rename = "displayName")]
    pub display_name: String,
    pub members: Option<Vec<ScimMemberRef>>,
    pub meta: ScimMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimMemberRef {
    pub value: String,
    #[serde(rename = "$ref")]
    pub ref_url: Option<String>,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimMeta {
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    pub created: String,
    #[serde(rename = "lastModified")]
    pub last_modified: String,
    pub location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimError {
    pub schemas: Vec<String>,
    pub status: String,
    pub detail: Option<String>,
    #[serde(rename = "scimType")]
    pub scim_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimPatchRequest {
    pub schemas: Vec<String>,
    #[serde(rename = "Operations")]
    pub operations: Vec<ScimPatchOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimPatchOperation {
    pub op: String, // "add", "remove", "replace"
    pub path: Option<String>,
    pub value: Option<serde_json::Value>,
}

// ============================================================================
// AUDIT LOG EXPORTS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditExportConfiguration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub is_enabled: bool,
    pub export_type: String, // 'webhook', 's3', 'splunk', 'elastic'
    pub webhook_url: Option<String>,
    pub webhook_secret_encrypted: Option<String>,
    pub webhook_headers: serde_json::Value,
    pub s3_bucket: Option<String>,
    pub s3_prefix: Option<String>,
    pub s3_region: Option<String>,
    pub s3_access_key_encrypted: Option<String>,
    pub s3_secret_key_encrypted: Option<String>,
    pub format: Option<String>, // 'json', 'cef', 'leef', 'syslog'
    pub include_pii: Option<bool>,
    pub event_types: Option<Vec<String>>,
    pub min_severity: Option<String>,
    pub batch_size: Option<i32>,
    pub flush_interval_seconds: Option<i32>,
    pub retry_count: Option<i32>,
    pub last_export_at: Option<DateTime<Utc>>,
    pub total_events_exported: Option<i64>,
    pub total_failures: Option<i64>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditExportConfigurationResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub is_enabled: bool,
    pub export_type: String,
    pub webhook_url: Option<String>,
    pub has_webhook_secret: bool,
    pub webhook_headers: serde_json::Value,
    pub s3_bucket: Option<String>,
    pub s3_prefix: Option<String>,
    pub s3_region: Option<String>,
    pub has_s3_credentials: bool,
    pub format: Option<String>,
    pub include_pii: Option<bool>,
    pub event_types: Option<Vec<String>>,
    pub min_severity: Option<String>,
    pub batch_size: Option<i32>,
    pub flush_interval_seconds: Option<i32>,
    pub last_export_at: Option<DateTime<Utc>>,
    pub total_events_exported: Option<i64>,
    pub total_failures: Option<i64>,
    pub last_error: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditExportConfiguration {
    pub name: String,
    pub export_type: String,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub webhook_headers: Option<serde_json::Value>,
    pub s3_bucket: Option<String>,
    pub s3_prefix: Option<String>,
    pub s3_region: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
    pub format: Option<String>,
    pub include_pii: Option<bool>,
    pub event_types: Option<Vec<String>>,
    pub min_severity: Option<String>,
    pub batch_size: Option<i32>,
    pub flush_interval_seconds: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAuditExportConfiguration {
    pub name: Option<String>,
    pub is_enabled: Option<bool>,
    pub webhook_url: Option<String>,
    pub webhook_secret: Option<String>,
    pub webhook_headers: Option<serde_json::Value>,
    pub s3_bucket: Option<String>,
    pub s3_prefix: Option<String>,
    pub s3_region: Option<String>,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,
    pub format: Option<String>,
    pub include_pii: Option<bool>,
    pub event_types: Option<Vec<String>>,
    pub min_severity: Option<String>,
    pub batch_size: Option<i32>,
    pub flush_interval_seconds: Option<i32>,
}

// Activity log with extended fields
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivityLog {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub outcome: Option<String>,
    pub duration_ms: Option<i32>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub resource_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ActivityLogWithUser {
    // ActivityLog fields (flattened)
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub outcome: Option<String>,
    pub duration_ms: Option<i32>,
    pub request_id: Option<String>,
    pub session_id: Option<String>,
    pub resource_name: Option<String>,
    pub created_at: DateTime<Utc>,
    // Joined user fields
    pub user_name: Option<String>,
    pub user_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateActivityLog {
    pub action: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub old_values: Option<serde_json::Value>,
    pub new_values: Option<serde_json::Value>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub outcome: Option<String>,
    pub duration_ms: Option<i32>,
    pub resource_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListActivityLogsQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub severity: Option<String>,
    pub category: Option<String>,
    pub outcome: Option<String>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportActivityLogsRequest {
    pub format: String, // 'json', 'csv', 'cef', 'leef'
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub event_types: Option<Vec<String>>,
    pub include_pii: Option<bool>,
}

// SIEM formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CefEvent {
    pub version: String,
    pub device_vendor: String,
    pub device_product: String,
    pub device_version: String,
    pub signature_id: String,
    pub name: String,
    pub severity: String,
    pub extension: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeefEvent {
    pub version: String,
    pub vendor: String,
    pub product: String,
    pub product_version: String,
    pub event_id: String,
    pub attributes: std::collections::HashMap<String, String>,
}

// ============================================================================
// BRANDING / WHITE-LABELING
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BrandingConfiguration {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub logo_url: Option<String>,
    pub logo_dark_url: Option<String>,
    pub favicon_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
    pub dark_mode_enabled: Option<bool>,
    pub default_theme: Option<String>,
    pub font_family: Option<String>,
    pub heading_font_family: Option<String>,
    pub custom_css: Option<String>,
    pub login_background_url: Option<String>,
    pub login_message: Option<String>,
    pub show_powered_by: Option<bool>,
    pub email_header_html: Option<String>,
    pub email_footer_html: Option<String>,
    pub email_logo_url: Option<String>,
    pub pdf_header_html: Option<String>,
    pub pdf_footer_html: Option<String>,
    pub pdf_cover_image_url: Option<String>,
    pub custom_domain: Option<String>,
    pub custom_domain_verified: Option<bool>,
    pub custom_domain_ssl_status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateBrandingConfiguration {
    pub logo_url: Option<String>,
    pub logo_dark_url: Option<String>,
    pub favicon_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub accent_color: Option<String>,
    pub dark_mode_enabled: Option<bool>,
    pub default_theme: Option<String>,
    pub font_family: Option<String>,
    pub heading_font_family: Option<String>,
    pub custom_css: Option<String>,
    pub login_background_url: Option<String>,
    pub login_message: Option<String>,
    pub show_powered_by: Option<bool>,
    pub email_header_html: Option<String>,
    pub email_footer_html: Option<String>,
    pub email_logo_url: Option<String>,
    pub pdf_header_html: Option<String>,
    pub pdf_footer_html: Option<String>,
    pub pdf_cover_image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetCustomDomainRequest {
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DomainVerification {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub domain: String,
    pub verification_type: String,
    pub verification_token: String,
    pub verification_value: Option<String>,
    pub is_verified: bool,
    pub verified_at: Option<DateTime<Utc>>,
    pub last_check_at: Option<DateTime<Utc>>,
    pub check_error: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainVerificationInstructions {
    pub domain: String,
    pub verification_type: String,
    pub dns_record_type: String,
    pub dns_record_name: String,
    pub dns_record_value: String,
    pub verification_token: String,
}

// ============================================================================
// API KEYS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub key_prefix: String,
    pub key_hash: String,
    pub scopes: Vec<String>,
    pub role_id: Option<Uuid>,
    pub rate_limit_per_minute: Option<i32>,
    pub rate_limit_per_hour: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub revoked_by: Option<Uuid>,
    pub revoke_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub role_id: Option<Uuid>,
    pub rate_limit_per_minute: Option<i32>,
    pub rate_limit_per_hour: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKey {
    pub name: String,
    pub description: Option<String>,
    pub scopes: Vec<String>,
    pub role_id: Option<Uuid>,
    pub rate_limit_per_minute: Option<i32>,
    pub rate_limit_per_hour: Option<i32>,
    pub expires_in_days: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateApiKeyResponse {
    pub api_key: ApiKeyResponse,
    pub key: String, // Full key, only shown once
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeApiKeyRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ApiKeyUsageDaily {
    pub id: Uuid,
    pub api_key_id: Uuid,
    pub usage_date: NaiveDate,
    pub request_count: i64,
    pub error_count: i64,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// RATE LIMITING (Redis structures, not DB)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub requests_per_minute: i32,
    pub requests_per_hour: i32,
    pub burst_size: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitStatus {
    pub remaining_minute: i64,
    pub remaining_hour: i64,
    pub reset_minute: i64, // Unix timestamp
    pub reset_hour: i64,
    pub is_limited: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub requests_today: i64,
    pub requests_this_hour: i64,
    pub requests_this_minute: i64,
    pub quota_remaining: Option<i64>,
    pub quota_reset_at: Option<DateTime<Utc>>,
}

// ============================================================================
// ORGANIZATION USAGE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrganizationUsageDaily {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub usage_date: NaiveDate,
    pub api_requests: i64,
    pub storage_bytes: i64,
    pub users_count: i32,
    pub integrations_count: i32,
    pub evidence_count: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizationUsageSummary {
    pub total_api_requests_30d: i64,
    pub total_storage_bytes: i64,
    pub current_users: i32,
    pub current_integrations: i32,
    pub current_evidence: i32,
    pub daily_usage: Vec<OrganizationUsageDaily>,
}

// ============================================================================
// ENTERPRISE STATS
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseStats {
    pub roles_count: i64,
    pub custom_roles_count: i64,
    pub sso_enabled: bool,
    pub scim_enabled: bool,
    pub audit_exports_count: i64,
    pub api_keys_count: i64,
    pub has_custom_branding: bool,
    pub has_custom_domain: bool,
}
