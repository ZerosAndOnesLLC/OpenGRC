use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    Permission, PermissionGroup, RoleWithPermissions, CreateRole, UpdateRole,
    UserWithRoles, AssignRolesRequest,
    SsoConfigurationResponse, CreateSsoConfiguration, UpdateSsoConfiguration,
    SsoDomain, AddSsoDomain, SamlSpMetadata,
    ScimConfigurationResponse, CreateScimConfiguration, UpdateScimConfiguration,
    GenerateScimTokenResponse,
    AuditExportConfigurationResponse, CreateAuditExportConfiguration,
    ActivityLogWithUser, ListActivityLogsQuery,
    BrandingConfiguration, UpdateBrandingConfiguration, SetCustomDomainRequest,
    DomainVerificationInstructions,
    ApiKeyResponse, CreateApiKey, CreateApiKeyResponse, RevokeApiKeyRequest,
    RateLimitStatus, UsageStats, EnterpriseStats,
};
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .ok_or_else(|| AppError::Unauthorized("No organization".to_string()))?
        .parse::<Uuid>()
        .map_err(|_| AppError::BadRequest("Invalid organization ID".to_string()))
}

fn get_user_id(user: &AuthUser) -> AppResult<Uuid> {
    user.id.parse::<Uuid>()
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))
}

// ============================================================================
// PERMISSIONS
// ============================================================================

pub async fn list_permissions(
    State(services): State<Arc<AppServices>>,
    Extension(_user): Extension<AuthUser>,
) -> AppResult<Json<Vec<Permission>>> {
    let permissions = services.enterprise.list_permissions().await?;
    Ok(Json(permissions))
}

pub async fn list_permissions_grouped(
    State(services): State<Arc<AppServices>>,
    Extension(_user): Extension<AuthUser>,
) -> AppResult<Json<Vec<PermissionGroup>>> {
    let groups = services.enterprise.list_permissions_grouped().await?;
    Ok(Json(groups))
}

// ============================================================================
// ROLES
// ============================================================================

pub async fn list_roles(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<RoleWithPermissions>>> {
    let org_id = get_org_id(&user)?;
    let roles = services.enterprise.list_roles(org_id).await?;
    Ok(Json(roles))
}

pub async fn get_role(
    State(services): State<Arc<AppServices>>,
    Path(role_id): Path<Uuid>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<RoleWithPermissions>> {
    let org_id = get_org_id(&user)?;
    let role = services.enterprise.get_role(org_id, role_id).await?;
    Ok(Json(role))
}

pub async fn create_role(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateRole>,
) -> AppResult<(StatusCode, Json<RoleWithPermissions>)> {
    let org_id = get_org_id(&user)?;
    let role = services.enterprise.create_role(org_id, input).await?;
    Ok((StatusCode::CREATED, Json(role)))
}

pub async fn update_role(
    State(services): State<Arc<AppServices>>,
    Path(role_id): Path<Uuid>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<UpdateRole>,
) -> AppResult<Json<RoleWithPermissions>> {
    let org_id = get_org_id(&user)?;
    let role = services.enterprise.update_role(org_id, role_id, input).await?;
    Ok(Json(role))
}

pub async fn delete_role(
    State(services): State<Arc<AppServices>>,
    Path(role_id): Path<Uuid>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<StatusCode> {
    let org_id = get_org_id(&user)?;
    services.enterprise.delete_role(org_id, role_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// USER ROLES
// ============================================================================

pub async fn get_user_roles(
    State(services): State<Arc<AppServices>>,
    Path(user_id): Path<Uuid>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<UserWithRoles>> {
    let org_id = get_org_id(&user)?;
    let user_with_roles = services.enterprise.get_user_with_roles(org_id, user_id).await?;
    Ok(Json(user_with_roles))
}

pub async fn assign_user_roles(
    State(services): State<Arc<AppServices>>,
    Path(target_user_id): Path<Uuid>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<AssignRolesRequest>,
) -> AppResult<Json<UserWithRoles>> {
    let org_id = get_org_id(&user)?;
    let assigner_id = get_user_id(&user)?;
    let user_with_roles = services.enterprise.assign_roles(org_id, target_user_id, input, assigner_id).await?;
    Ok(Json(user_with_roles))
}

pub async fn get_my_permissions(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<String>>> {
    let user_id = get_user_id(&user)?;
    let permissions = services.enterprise.get_user_permissions(user_id).await?;
    Ok(Json(permissions))
}

// ============================================================================
// SSO / SAML
// ============================================================================

pub async fn get_sso_configuration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Option<SsoConfigurationResponse>>> {
    let org_id = get_org_id(&user)?;
    let config = services.enterprise.get_sso_configuration(org_id).await?;
    Ok(Json(config))
}

pub async fn create_sso_configuration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateSsoConfiguration>,
) -> AppResult<(StatusCode, Json<SsoConfigurationResponse>)> {
    let org_id = get_org_id(&user)?;
    let config = services.enterprise.create_sso_configuration(org_id, input).await?;
    Ok((StatusCode::CREATED, Json(config)))
}

pub async fn update_sso_configuration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<UpdateSsoConfiguration>,
) -> AppResult<Json<SsoConfigurationResponse>> {
    let org_id = get_org_id(&user)?;
    let config = services.enterprise.update_sso_configuration(org_id, input).await?;
    Ok(Json(config))
}

pub async fn delete_sso_configuration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<StatusCode> {
    let org_id = get_org_id(&user)?;
    services.enterprise.delete_sso_configuration(org_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn add_sso_domain(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<AddSsoDomain>,
) -> AppResult<(StatusCode, Json<SsoDomain>)> {
    let org_id = get_org_id(&user)?;
    let domain = services.enterprise.add_sso_domain(org_id, input).await?;
    Ok((StatusCode::CREATED, Json(domain)))
}

pub async fn verify_sso_domain(
    State(services): State<Arc<AppServices>>,
    Path(domain_id): Path<Uuid>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<SsoDomain>> {
    let org_id = get_org_id(&user)?;
    let domain = services.enterprise.verify_sso_domain(org_id, domain_id).await?;
    Ok(Json(domain))
}

pub async fn get_saml_metadata(
    State(_services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<SamlSpMetadata>> {
    let org_id = get_org_id(&user)?;

    let base_url = std::env::var("API_BASE_URL").unwrap_or_else(|_| "https://api.opengrc.io".to_string());
    let sp_entity_id = format!("{}/saml/{}", base_url, org_id);
    let sp_acs_url = format!("{}/api/v1/sso/saml/acs", base_url);
    let sp_slo_url = format!("{}/api/v1/sso/saml/slo", base_url);

    let metadata_xml = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<md:EntityDescriptor xmlns:md="urn:oasis:names:tc:SAML:2.0:metadata" entityID="{}">
  <md:SPSSODescriptor AuthnRequestsSigned="false" WantAssertionsSigned="true" protocolSupportEnumeration="urn:oasis:names:tc:SAML:2.0:protocol">
    <md:NameIDFormat>urn:oasis:names:tc:SAML:1.1:nameid-format:emailAddress</md:NameIDFormat>
    <md:AssertionConsumerService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-POST" Location="{}" index="0" isDefault="true"/>
    <md:SingleLogoutService Binding="urn:oasis:names:tc:SAML:2.0:bindings:HTTP-Redirect" Location="{}"/>
  </md:SPSSODescriptor>
</md:EntityDescriptor>"#, sp_entity_id, sp_acs_url, sp_slo_url);

    Ok(Json(SamlSpMetadata {
        entity_id: sp_entity_id,
        acs_url: sp_acs_url,
        slo_url: sp_slo_url,
        metadata_xml,
    }))
}

// ============================================================================
// SCIM
// ============================================================================

pub async fn get_scim_configuration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Option<ScimConfigurationResponse>>> {
    let org_id = get_org_id(&user)?;
    let config = services.enterprise.get_scim_configuration(org_id).await?;
    Ok(Json(config))
}

pub async fn create_scim_configuration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateScimConfiguration>,
) -> AppResult<(StatusCode, Json<ScimConfigurationResponse>)> {
    let org_id = get_org_id(&user)?;
    let config = services.enterprise.create_scim_configuration(org_id, input).await?;
    Ok((StatusCode::CREATED, Json(config)))
}

pub async fn update_scim_configuration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<UpdateScimConfiguration>,
) -> AppResult<Json<ScimConfigurationResponse>> {
    let org_id = get_org_id(&user)?;
    let config = services.enterprise.update_scim_configuration(org_id, input).await?;
    Ok(Json(config))
}

pub async fn generate_scim_token(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<GenerateScimTokenResponse>> {
    let org_id = get_org_id(&user)?;
    let response = services.enterprise.generate_scim_token(org_id).await?;
    Ok(Json(response))
}

pub async fn revoke_scim_token(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<StatusCode> {
    let org_id = get_org_id(&user)?;
    services.enterprise.revoke_scim_token(org_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// AUDIT LOGS
// ============================================================================

#[derive(serde::Serialize)]
pub struct PaginatedActivityLogs {
    pub data: Vec<ActivityLogWithUser>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

pub async fn list_activity_logs(
    State(services): State<Arc<AppServices>>,
    Query(query): Query<ListActivityLogsQuery>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<PaginatedActivityLogs>> {
    let org_id = get_org_id(&user)?;
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(50);
    let (logs, total) = services.enterprise.list_activity_logs(org_id, query).await?;
    Ok(Json(PaginatedActivityLogs { data: logs, total, page, page_size }))
}

// ============================================================================
// AUDIT EXPORT CONFIGURATIONS
// ============================================================================

pub async fn list_audit_export_configurations(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<AuditExportConfigurationResponse>>> {
    let org_id = get_org_id(&user)?;
    let configs = services.enterprise.list_audit_export_configurations(org_id).await?;
    Ok(Json(configs))
}

pub async fn create_audit_export_configuration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateAuditExportConfiguration>,
) -> AppResult<(StatusCode, Json<AuditExportConfigurationResponse>)> {
    let org_id = get_org_id(&user)?;
    let config = services.enterprise.create_audit_export_configuration(org_id, input).await?;
    Ok((StatusCode::CREATED, Json(config)))
}

pub async fn delete_audit_export_configuration(
    State(services): State<Arc<AppServices>>,
    Path(config_id): Path<Uuid>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<StatusCode> {
    let org_id = get_org_id(&user)?;
    services.enterprise.delete_audit_export_configuration(org_id, config_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// BRANDING
// ============================================================================

pub async fn get_branding(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Option<BrandingConfiguration>>> {
    let org_id = get_org_id(&user)?;
    let branding = services.enterprise.get_branding(org_id).await?;
    Ok(Json(branding))
}

pub async fn update_branding(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<UpdateBrandingConfiguration>,
) -> AppResult<Json<BrandingConfiguration>> {
    let org_id = get_org_id(&user)?;
    let branding = services.enterprise.update_branding(org_id, input).await?;
    Ok(Json(branding))
}

pub async fn set_custom_domain(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<SetCustomDomainRequest>,
) -> AppResult<Json<DomainVerificationInstructions>> {
    let org_id = get_org_id(&user)?;
    let instructions = services.enterprise.set_custom_domain(org_id, input).await?;
    Ok(Json(instructions))
}

// ============================================================================
// API KEYS
// ============================================================================

pub async fn list_api_keys(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<ApiKeyResponse>>> {
    let org_id = get_org_id(&user)?;
    let keys = services.enterprise.list_api_keys(org_id).await?;
    Ok(Json(keys))
}

pub async fn create_api_key(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateApiKey>,
) -> AppResult<(StatusCode, Json<CreateApiKeyResponse>)> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let response = services.enterprise.create_api_key(org_id, user_id, input).await?;
    Ok((StatusCode::CREATED, Json(response)))
}

pub async fn revoke_api_key(
    State(services): State<Arc<AppServices>>,
    Path(key_id): Path<Uuid>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<RevokeApiKeyRequest>,
) -> AppResult<StatusCode> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    services.enterprise.revoke_api_key(org_id, key_id, user_id, input).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================================
// USAGE & RATE LIMITING
// ============================================================================

pub async fn get_rate_limit_status(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<RateLimitStatus>> {
    let org_id = get_org_id(&user)?;
    let limit_minute = 1000;
    let limit_hour = 10000;
    let status = services.enterprise.check_rate_limit(org_id, limit_minute, limit_hour).await?;
    Ok(Json(status))
}

pub async fn get_usage_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<UsageStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.enterprise.get_usage_stats(org_id).await?;
    Ok(Json(stats))
}

// ============================================================================
// ENTERPRISE STATS
// ============================================================================

pub async fn get_enterprise_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<EnterpriseStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.enterprise.get_enterprise_stats(org_id).await?;
    Ok(Json(stats))
}
