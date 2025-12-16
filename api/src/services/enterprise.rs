use crate::cache::CacheClient;
use crate::models::{
    Permission, PermissionGroup, Role, RoleWithPermissions, CreateRole, UpdateRole,
    UserWithRoles, AssignRolesRequest,
    SsoConfiguration, SsoConfigurationResponse, CreateSsoConfiguration, UpdateSsoConfiguration,
    SsoDomain, AddSsoDomain,
    ScimConfiguration, ScimConfigurationResponse, CreateScimConfiguration, UpdateScimConfiguration,
    GenerateScimTokenResponse,
    AuditExportConfiguration, AuditExportConfigurationResponse,
    CreateAuditExportConfiguration,
    ActivityLog, ActivityLogWithUser, CreateActivityLog, ListActivityLogsQuery,
    BrandingConfiguration, UpdateBrandingConfiguration, SetCustomDomainRequest,
    DomainVerificationInstructions,
    ApiKey, ApiKeyResponse, CreateApiKey, CreateApiKeyResponse, RevokeApiKeyRequest,
    RateLimitStatus, UsageStats, EnterpriseStats, User,
};
use crate::utils::{AppError, AppResult};
use chrono::{Duration, Utc};
use sha2::{Sha256, Digest};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration as StdDuration;
use uuid::Uuid;

#[derive(Clone)]
pub struct EnterpriseService {
    db: PgPool,
    cache: Arc<CacheClient>,
}

impl EnterpriseService {
    pub fn new(db: PgPool, cache: Arc<CacheClient>) -> Self {
        Self { db, cache }
    }

    // ========================================================================
    // PERMISSIONS
    // ========================================================================

    pub async fn list_permissions(&self) -> AppResult<Vec<Permission>> {
        let permissions = sqlx::query_as::<_, Permission>(
            "SELECT * FROM permissions ORDER BY resource, action"
        )
        .fetch_all(&self.db)
        .await?;

        Ok(permissions)
    }

    pub async fn list_permissions_grouped(&self) -> AppResult<Vec<PermissionGroup>> {
        let permissions = self.list_permissions().await?;

        let mut groups: std::collections::HashMap<String, Vec<Permission>> = std::collections::HashMap::new();
        for perm in permissions {
            groups.entry(perm.resource.clone()).or_default().push(perm);
        }

        let mut result: Vec<PermissionGroup> = groups
            .into_iter()
            .map(|(resource, permissions)| PermissionGroup { resource, permissions })
            .collect();
        result.sort_by(|a, b| a.resource.cmp(&b.resource));

        Ok(result)
    }

    // ========================================================================
    // ROLES
    // ========================================================================

    pub async fn list_roles(&self, org_id: Uuid) -> AppResult<Vec<RoleWithPermissions>> {
        let roles = sqlx::query_as::<_, Role>(
            "SELECT * FROM roles WHERE organization_id = $1 ORDER BY is_system DESC, name"
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let mut result = Vec::new();
        for role in roles {
            let permissions = sqlx::query_as::<_, Permission>(
                r#"
                SELECT p.* FROM permissions p
                JOIN role_permissions rp ON rp.permission_id = p.id
                WHERE rp.role_id = $1
                ORDER BY p.resource, p.action
                "#
            )
            .bind(role.id)
            .fetch_all(&self.db)
            .await?;

            let user_count: (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM user_roles WHERE role_id = $1"
            )
            .bind(role.id)
            .fetch_one(&self.db)
            .await?;

            result.push(RoleWithPermissions {
                role,
                permissions,
                user_count: user_count.0,
            });
        }

        Ok(result)
    }

    pub async fn get_role(&self, org_id: Uuid, role_id: Uuid) -> AppResult<RoleWithPermissions> {
        let role = sqlx::query_as::<_, Role>(
            "SELECT * FROM roles WHERE id = $1 AND organization_id = $2"
        )
        .bind(role_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Role not found".to_string()))?;

        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT p.* FROM permissions p
            JOIN role_permissions rp ON rp.permission_id = p.id
            WHERE rp.role_id = $1
            ORDER BY p.resource, p.action
            "#
        )
        .bind(role_id)
        .fetch_all(&self.db)
        .await?;

        let user_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM user_roles WHERE role_id = $1"
        )
        .bind(role_id)
        .fetch_one(&self.db)
        .await?;

        Ok(RoleWithPermissions {
            role,
            permissions,
            user_count: user_count.0,
        })
    }

    pub async fn create_role(&self, org_id: Uuid, input: CreateRole) -> AppResult<RoleWithPermissions> {
        let mut tx = self.db.begin().await?;

        let role = sqlx::query_as::<_, Role>(
            r#"
            INSERT INTO roles (organization_id, name, description, is_system, is_default)
            VALUES ($1, $2, $3, FALSE, FALSE)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .fetch_one(&mut *tx)
        .await?;

        // Assign permissions
        for perm_id in &input.permission_ids {
            sqlx::query(
                "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)"
            )
            .bind(role.id)
            .bind(perm_id)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        self.get_role(org_id, role.id).await
    }

    pub async fn update_role(&self, org_id: Uuid, role_id: Uuid, input: UpdateRole) -> AppResult<RoleWithPermissions> {
        let existing = self.get_role(org_id, role_id).await?;

        if existing.role.is_system {
            return Err(AppError::BadRequest("Cannot modify system roles".to_string()));
        }

        let mut tx = self.db.begin().await?;

        sqlx::query(
            r#"
            UPDATE roles SET
                name = COALESCE($1, name),
                description = COALESCE($2, description),
                updated_at = NOW()
            WHERE id = $3 AND organization_id = $4
            "#
        )
        .bind(&input.name)
        .bind(&input.description)
        .bind(role_id)
        .bind(org_id)
        .execute(&mut *tx)
        .await?;

        // Update permissions if provided
        if let Some(permission_ids) = &input.permission_ids {
            sqlx::query("DELETE FROM role_permissions WHERE role_id = $1")
                .bind(role_id)
                .execute(&mut *tx)
                .await?;

            for perm_id in permission_ids {
                sqlx::query(
                    "INSERT INTO role_permissions (role_id, permission_id) VALUES ($1, $2)"
                )
                .bind(role_id)
                .bind(perm_id)
                .execute(&mut *tx)
                .await?;
            }
        }

        tx.commit().await?;

        // Invalidate cache for users with this role
        self.cache.delete_pattern(&format!("user_permissions:*")).await?;

        self.get_role(org_id, role_id).await
    }

    pub async fn delete_role(&self, org_id: Uuid, role_id: Uuid) -> AppResult<()> {
        let existing = self.get_role(org_id, role_id).await?;

        if existing.role.is_system {
            return Err(AppError::BadRequest("Cannot delete system roles".to_string()));
        }

        if existing.user_count > 0 {
            return Err(AppError::BadRequest(
                format!("Cannot delete role with {} assigned users", existing.user_count)
            ));
        }

        sqlx::query("DELETE FROM roles WHERE id = $1 AND organization_id = $2")
            .bind(role_id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    // ========================================================================
    // USER ROLES
    // ========================================================================

    pub async fn get_user_with_roles(&self, org_id: Uuid, user_id: Uuid) -> AppResult<UserWithRoles> {
        let user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND organization_id = $2"
        )
        .bind(user_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        let roles = sqlx::query_as::<_, Role>(
            r#"
            SELECT r.* FROM roles r
            JOIN user_roles ur ON ur.role_id = r.id
            WHERE ur.user_id = $1
            ORDER BY r.name
            "#
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(UserWithRoles {
            id: user.id,
            organization_id: user.organization_id,
            tv_user_id: user.tv_user_id,
            email: user.email,
            name: user.name,
            last_login_at: user.last_login_at,
            created_at: user.created_at,
            roles,
        })
    }

    pub async fn assign_roles(&self, org_id: Uuid, user_id: Uuid, input: AssignRolesRequest, assigned_by: Uuid) -> AppResult<UserWithRoles> {
        // Verify user exists
        let _user = sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE id = $1 AND organization_id = $2"
        )
        .bind(user_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

        // Verify all roles belong to the org
        for role_id in &input.role_ids {
            let _role = sqlx::query_as::<_, Role>(
                "SELECT * FROM roles WHERE id = $1 AND organization_id = $2"
            )
            .bind(role_id)
            .bind(org_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("Role {} not found", role_id)))?;
        }

        let mut tx = self.db.begin().await?;

        // Remove existing roles
        sqlx::query("DELETE FROM user_roles WHERE user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;

        // Assign new roles
        for role_id in &input.role_ids {
            sqlx::query(
                "INSERT INTO user_roles (user_id, role_id, assigned_by) VALUES ($1, $2, $3)"
            )
            .bind(user_id)
            .bind(role_id)
            .bind(assigned_by)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // Invalidate user permissions cache
        self.cache.delete(&format!("user_permissions:{}", user_id)).await?;

        self.get_user_with_roles(org_id, user_id).await
    }

    pub async fn get_user_permissions(&self, user_id: Uuid) -> AppResult<Vec<String>> {
        // Check cache first
        let cache_key = format!("user_permissions:{}", user_id);
        if let Ok(Some(cached)) = self.cache.get::<Vec<String>>(&cache_key).await {
            return Ok(cached);
        }

        let permissions: Vec<(String,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT p.code
            FROM permissions p
            JOIN role_permissions rp ON rp.permission_id = p.id
            JOIN user_roles ur ON ur.role_id = rp.role_id
            WHERE ur.user_id = $1
            "#
        )
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        let perms: Vec<String> = permissions.into_iter().map(|(code,)| code).collect();

        // Cache for 5 minutes
        self.cache.set(&cache_key, &perms, Some(StdDuration::from_secs(300))).await?;

        Ok(perms)
    }

    pub async fn user_has_permission(&self, user_id: Uuid, permission_code: &str) -> AppResult<bool> {
        let permissions = self.get_user_permissions(user_id).await?;
        Ok(permissions.contains(&permission_code.to_string()))
    }

    pub async fn initialize_org_roles(&self, org_id: Uuid) -> AppResult<()> {
        sqlx::query("SELECT initialize_organization_roles($1)")
            .bind(org_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    // ========================================================================
    // SSO / SAML
    // ========================================================================

    pub async fn get_sso_configuration(&self, org_id: Uuid) -> AppResult<Option<SsoConfigurationResponse>> {
        let config = sqlx::query_as::<_, SsoConfiguration>(
            "SELECT * FROM sso_configurations WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?;

        match config {
            Some(c) => {
                let domains = sqlx::query_as::<_, SsoDomain>(
                    "SELECT * FROM sso_domains WHERE sso_configuration_id = $1 ORDER BY domain"
                )
                .bind(c.id)
                .fetch_all(&self.db)
                .await?;

                Ok(Some(SsoConfigurationResponse {
                    id: c.id,
                    organization_id: c.organization_id,
                    provider_type: c.provider_type,
                    name: c.name,
                    is_enabled: c.is_enabled,
                    is_enforced: c.is_enforced,
                    idp_entity_id: c.idp_entity_id,
                    idp_sso_url: c.idp_sso_url,
                    idp_slo_url: c.idp_slo_url,
                    has_certificate: c.idp_certificate.is_some(),
                    sp_entity_id: c.sp_entity_id,
                    sp_acs_url: c.sp_acs_url,
                    sp_slo_url: c.sp_slo_url,
                    attribute_mappings: c.attribute_mappings,
                    auto_provision_users: c.auto_provision_users,
                    default_role_id: c.default_role_id,
                    metadata_url: c.metadata_url,
                    last_metadata_refresh: c.last_metadata_refresh,
                    created_at: c.created_at,
                    updated_at: c.updated_at,
                    domains,
                }))
            }
            None => Ok(None),
        }
    }

    pub async fn create_sso_configuration(&self, org_id: Uuid, input: CreateSsoConfiguration) -> AppResult<SsoConfigurationResponse> {
        // Check if config already exists
        let existing = self.get_sso_configuration(org_id).await?;
        if existing.is_some() {
            return Err(AppError::BadRequest("SSO configuration already exists".to_string()));
        }

        // Generate SP URLs based on org
        let base_url = std::env::var("API_BASE_URL").unwrap_or_else(|_| "https://api.opengrc.io".to_string());
        let sp_entity_id = format!("{}/saml/{}", base_url, org_id);
        let sp_acs_url = format!("{}/api/v1/sso/saml/acs", base_url);
        let sp_slo_url = format!("{}/api/v1/sso/saml/slo", base_url);

        let _config = sqlx::query_as::<_, SsoConfiguration>(
            r#"
            INSERT INTO sso_configurations (
                organization_id, provider_type, name, idp_entity_id, idp_sso_url, idp_slo_url,
                idp_certificate, sp_entity_id, sp_acs_url, sp_slo_url, attribute_mappings,
                auto_provision_users, default_role_id, metadata_url, metadata_xml
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(&input.provider_type)
        .bind(&input.name)
        .bind(&input.idp_entity_id)
        .bind(&input.idp_sso_url)
        .bind(&input.idp_slo_url)
        .bind(&input.idp_certificate)
        .bind(&sp_entity_id)
        .bind(&sp_acs_url)
        .bind(&sp_slo_url)
        .bind(&input.attribute_mappings.unwrap_or(serde_json::json!({"email": "email", "name": "name"})))
        .bind(input.auto_provision_users.unwrap_or(true))
        .bind(&input.default_role_id)
        .bind(&input.metadata_url)
        .bind(&input.metadata_xml)
        .fetch_one(&self.db)
        .await?;

        self.get_sso_configuration(org_id).await?.ok_or_else(|| AppError::InternalServerError("Failed to retrieve created SSO config".to_string()))
    }

    pub async fn update_sso_configuration(&self, org_id: Uuid, input: UpdateSsoConfiguration) -> AppResult<SsoConfigurationResponse> {
        let existing = self.get_sso_configuration(org_id).await?
            .ok_or_else(|| AppError::NotFound("SSO configuration not found".to_string()))?;

        sqlx::query(
            r#"
            UPDATE sso_configurations SET
                name = COALESCE($1, name),
                is_enabled = COALESCE($2, is_enabled),
                is_enforced = COALESCE($3, is_enforced),
                idp_entity_id = COALESCE($4, idp_entity_id),
                idp_sso_url = COALESCE($5, idp_sso_url),
                idp_slo_url = COALESCE($6, idp_slo_url),
                idp_certificate = COALESCE($7, idp_certificate),
                attribute_mappings = COALESCE($8, attribute_mappings),
                auto_provision_users = COALESCE($9, auto_provision_users),
                default_role_id = COALESCE($10, default_role_id),
                metadata_url = COALESCE($11, metadata_url),
                metadata_xml = COALESCE($12, metadata_xml),
                updated_at = NOW()
            WHERE id = $13
            "#
        )
        .bind(&input.name)
        .bind(&input.is_enabled)
        .bind(&input.is_enforced)
        .bind(&input.idp_entity_id)
        .bind(&input.idp_sso_url)
        .bind(&input.idp_slo_url)
        .bind(&input.idp_certificate)
        .bind(&input.attribute_mappings)
        .bind(&input.auto_provision_users)
        .bind(&input.default_role_id)
        .bind(&input.metadata_url)
        .bind(&input.metadata_xml)
        .bind(existing.id)
        .execute(&self.db)
        .await?;

        self.get_sso_configuration(org_id).await?.ok_or_else(|| AppError::InternalServerError("Failed to retrieve updated SSO config".to_string()))
    }

    pub async fn delete_sso_configuration(&self, org_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM sso_configurations WHERE organization_id = $1")
            .bind(org_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    pub async fn add_sso_domain(&self, org_id: Uuid, input: AddSsoDomain) -> AppResult<SsoDomain> {
        let config = self.get_sso_configuration(org_id).await?
            .ok_or_else(|| AppError::NotFound("SSO configuration not found".to_string()))?;

        // Generate verification token
        let token = Uuid::new_v4().to_string().replace("-", "");

        let domain = sqlx::query_as::<_, SsoDomain>(
            r#"
            INSERT INTO sso_domains (organization_id, sso_configuration_id, domain, verification_token, verification_method)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(config.id)
        .bind(&input.domain)
        .bind(&token)
        .bind(input.verification_method.unwrap_or_else(|| "dns_txt".to_string()))
        .fetch_one(&self.db)
        .await?;

        Ok(domain)
    }

    pub async fn verify_sso_domain(&self, org_id: Uuid, domain_id: Uuid) -> AppResult<SsoDomain> {
        // In a real implementation, this would check DNS records
        // For now, we'll just mark it as verified
        let domain = sqlx::query_as::<_, SsoDomain>(
            r#"
            UPDATE sso_domains SET is_verified = TRUE, verified_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING *
            "#
        )
        .bind(domain_id)
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        Ok(domain)
    }

    // ========================================================================
    // SCIM
    // ========================================================================

    pub async fn get_scim_configuration(&self, org_id: Uuid) -> AppResult<Option<ScimConfigurationResponse>> {
        let config = sqlx::query_as::<_, ScimConfiguration>(
            "SELECT * FROM scim_configurations WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?;

        match config {
            Some(c) => Ok(Some(ScimConfigurationResponse {
                id: c.id,
                organization_id: c.organization_id,
                is_enabled: c.is_enabled,
                base_url: c.base_url,
                has_token: c.bearer_token_hash.is_some(),
                token_created_at: c.token_created_at,
                token_expires_at: c.token_expires_at,
                auto_activate_users: c.auto_activate_users,
                default_role_id: c.default_role_id,
                sync_groups_as_roles: c.sync_groups_as_roles,
                group_role_mappings: c.group_role_mappings,
                on_user_deactivate: c.on_user_deactivate,
                last_sync_at: c.last_sync_at,
                total_users_provisioned: c.total_users_provisioned,
                total_groups_provisioned: c.total_groups_provisioned,
                created_at: c.created_at,
                updated_at: c.updated_at,
            })),
            None => Ok(None),
        }
    }

    pub async fn create_scim_configuration(&self, org_id: Uuid, input: CreateScimConfiguration) -> AppResult<ScimConfigurationResponse> {
        let existing = self.get_scim_configuration(org_id).await?;
        if existing.is_some() {
            return Err(AppError::BadRequest("SCIM configuration already exists".to_string()));
        }

        let base_url = std::env::var("API_BASE_URL").unwrap_or_else(|_| "https://api.opengrc.io".to_string());
        let scim_base_url = format!("{}/scim/v2/{}", base_url, org_id);

        sqlx::query(
            r#"
            INSERT INTO scim_configurations (
                organization_id, base_url, auto_activate_users, default_role_id,
                sync_groups_as_roles, group_role_mappings, on_user_deactivate
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#
        )
        .bind(org_id)
        .bind(&scim_base_url)
        .bind(input.auto_activate_users.unwrap_or(true))
        .bind(&input.default_role_id)
        .bind(input.sync_groups_as_roles.unwrap_or(false))
        .bind(&input.group_role_mappings.unwrap_or(serde_json::json!({})))
        .bind(input.on_user_deactivate.unwrap_or_else(|| "suspend".to_string()))
        .execute(&self.db)
        .await?;

        self.get_scim_configuration(org_id).await?.ok_or_else(|| AppError::InternalServerError("Failed to create SCIM config".to_string()))
    }

    pub async fn update_scim_configuration(&self, org_id: Uuid, input: UpdateScimConfiguration) -> AppResult<ScimConfigurationResponse> {
        sqlx::query(
            r#"
            UPDATE scim_configurations SET
                is_enabled = COALESCE($1, is_enabled),
                auto_activate_users = COALESCE($2, auto_activate_users),
                default_role_id = COALESCE($3, default_role_id),
                sync_groups_as_roles = COALESCE($4, sync_groups_as_roles),
                group_role_mappings = COALESCE($5, group_role_mappings),
                on_user_deactivate = COALESCE($6, on_user_deactivate),
                updated_at = NOW()
            WHERE organization_id = $7
            "#
        )
        .bind(&input.is_enabled)
        .bind(&input.auto_activate_users)
        .bind(&input.default_role_id)
        .bind(&input.sync_groups_as_roles)
        .bind(&input.group_role_mappings)
        .bind(&input.on_user_deactivate)
        .bind(org_id)
        .execute(&self.db)
        .await?;

        self.get_scim_configuration(org_id).await?.ok_or_else(|| AppError::NotFound("SCIM config not found".to_string()))
    }

    pub async fn generate_scim_token(&self, org_id: Uuid) -> AppResult<GenerateScimTokenResponse> {
        // Generate a secure token
        let token = format!("scim_{}_{}", org_id.to_string().replace("-", "")[..8].to_string(), Uuid::new_v4().to_string().replace("-", ""));

        // Hash the token for storage
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let token_hash = format!("{:x}", hasher.finalize());

        let expires_at = Utc::now() + Duration::days(365);

        sqlx::query(
            r#"
            UPDATE scim_configurations SET
                bearer_token_hash = $1,
                token_created_at = NOW(),
                token_expires_at = $2,
                updated_at = NOW()
            WHERE organization_id = $3
            "#
        )
        .bind(&token_hash)
        .bind(expires_at)
        .bind(org_id)
        .execute(&self.db)
        .await?;

        Ok(GenerateScimTokenResponse {
            token,
            expires_at: Some(expires_at),
        })
    }

    pub async fn revoke_scim_token(&self, org_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE scim_configurations SET
                bearer_token_hash = NULL,
                token_created_at = NULL,
                token_expires_at = NULL,
                updated_at = NOW()
            WHERE organization_id = $1
            "#
        )
        .bind(org_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ========================================================================
    // AUDIT LOGS
    // ========================================================================

    pub async fn create_activity_log(&self, org_id: Uuid, user_id: Option<Uuid>, input: CreateActivityLog, ip: Option<String>, user_agent: Option<String>, request_id: Option<String>) -> AppResult<ActivityLog> {
        let log = sqlx::query_as::<_, ActivityLog>(
            r#"
            INSERT INTO activity_logs (
                organization_id, user_id, action, entity_type, entity_id,
                old_values, new_values, severity, category, outcome,
                duration_ms, resource_name, ip_address, user_agent, request_id
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(user_id)
        .bind(&input.action)
        .bind(&input.entity_type)
        .bind(&input.entity_id)
        .bind(&input.old_values)
        .bind(&input.new_values)
        .bind(input.severity.unwrap_or_else(|| "info".to_string()))
        .bind(&input.category)
        .bind(input.outcome.unwrap_or_else(|| "success".to_string()))
        .bind(&input.duration_ms)
        .bind(&input.resource_name)
        .bind(&ip)
        .bind(&user_agent)
        .bind(&request_id)
        .fetch_one(&self.db)
        .await?;

        Ok(log)
    }

    pub async fn list_activity_logs(&self, org_id: Uuid, query: ListActivityLogsQuery) -> AppResult<(Vec<ActivityLogWithUser>, i64)> {
        let page = query.page.unwrap_or(1);
        let page_size = query.page_size.unwrap_or(50).min(100);
        let offset = (page - 1) * page_size;

        let mut sql = String::from(
            r#"
            SELECT al.*, u.name as user_name, u.email as user_email
            FROM activity_logs al
            LEFT JOIN users u ON u.id = al.user_id
            WHERE al.organization_id = $1
            "#
        );
        let mut count_sql = String::from(
            "SELECT COUNT(*) FROM activity_logs al WHERE al.organization_id = $1"
        );

        let mut conditions = Vec::new();
        if query.user_id.is_some() { conditions.push("al.user_id = $2"); }
        if query.action.is_some() { conditions.push("al.action ILIKE $3"); }
        if query.entity_type.is_some() { conditions.push("al.entity_type = $4"); }
        if query.severity.is_some() { conditions.push("al.severity = $5"); }
        if query.start_date.is_some() { conditions.push("al.created_at >= $6"); }
        if query.end_date.is_some() { conditions.push("al.created_at <= $7"); }

        if !conditions.is_empty() {
            let cond_str = format!(" AND {}", conditions.join(" AND "));
            sql.push_str(&cond_str);
            count_sql.push_str(&cond_str);
        }

        sql.push_str(" ORDER BY al.created_at DESC LIMIT $8 OFFSET $9");

        // Execute with dynamic binding - simplified version
        let logs: Vec<ActivityLogWithUser> = sqlx::query_as(
            r#"
            SELECT al.id, al.organization_id, al.user_id, al.action, al.entity_type, al.entity_id,
                   al.old_values, al.new_values, al.ip_address, al.user_agent, al.severity,
                   al.category, al.outcome, al.duration_ms, al.request_id, al.session_id,
                   al.resource_name, al.created_at, u.name as user_name, u.email as user_email
            FROM activity_logs al
            LEFT JOIN users u ON u.id = al.user_id
            WHERE al.organization_id = $1
            ORDER BY al.created_at DESC
            LIMIT $2 OFFSET $3
            "#
        )
        .bind(org_id)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let total: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM activity_logs WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        Ok((logs, total.0))
    }

    // ========================================================================
    // AUDIT EXPORT CONFIGURATIONS
    // ========================================================================

    pub async fn list_audit_export_configurations(&self, org_id: Uuid) -> AppResult<Vec<AuditExportConfigurationResponse>> {
        let configs = sqlx::query_as::<_, AuditExportConfiguration>(
            "SELECT * FROM audit_export_configurations WHERE organization_id = $1 ORDER BY name"
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(configs.into_iter().map(|c| AuditExportConfigurationResponse {
            id: c.id,
            organization_id: c.organization_id,
            name: c.name,
            is_enabled: c.is_enabled,
            export_type: c.export_type,
            webhook_url: c.webhook_url,
            has_webhook_secret: c.webhook_secret_encrypted.is_some(),
            webhook_headers: c.webhook_headers,
            s3_bucket: c.s3_bucket,
            s3_prefix: c.s3_prefix,
            s3_region: c.s3_region,
            has_s3_credentials: c.s3_access_key_encrypted.is_some(),
            format: c.format,
            include_pii: c.include_pii,
            event_types: c.event_types,
            min_severity: c.min_severity,
            batch_size: c.batch_size,
            flush_interval_seconds: c.flush_interval_seconds,
            last_export_at: c.last_export_at,
            total_events_exported: c.total_events_exported,
            total_failures: c.total_failures,
            last_error: c.last_error,
            created_at: c.created_at,
            updated_at: c.updated_at,
        }).collect())
    }

    pub async fn create_audit_export_configuration(&self, org_id: Uuid, input: CreateAuditExportConfiguration) -> AppResult<AuditExportConfigurationResponse> {
        // Encrypt secrets if provided
        let webhook_secret_encrypted = input.webhook_secret.as_ref().map(|s| self.encrypt_secret(s));
        let s3_access_key_encrypted = input.s3_access_key.as_ref().map(|s| self.encrypt_secret(s));
        let s3_secret_key_encrypted = input.s3_secret_key.as_ref().map(|s| self.encrypt_secret(s));

        let config = sqlx::query_as::<_, AuditExportConfiguration>(
            r#"
            INSERT INTO audit_export_configurations (
                organization_id, name, export_type, webhook_url, webhook_secret_encrypted,
                webhook_headers, s3_bucket, s3_prefix, s3_region, s3_access_key_encrypted,
                s3_secret_key_encrypted, format, include_pii, event_types, min_severity,
                batch_size, flush_interval_seconds
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.export_type)
        .bind(&input.webhook_url)
        .bind(&webhook_secret_encrypted)
        .bind(&input.webhook_headers.unwrap_or(serde_json::json!({})))
        .bind(&input.s3_bucket)
        .bind(&input.s3_prefix)
        .bind(&input.s3_region)
        .bind(&s3_access_key_encrypted)
        .bind(&s3_secret_key_encrypted)
        .bind(input.format.unwrap_or_else(|| "json".to_string()))
        .bind(input.include_pii.unwrap_or(false))
        .bind(&input.event_types.unwrap_or_else(|| vec!["*".to_string()]))
        .bind(input.min_severity.unwrap_or_else(|| "info".to_string()))
        .bind(input.batch_size.unwrap_or(100))
        .bind(input.flush_interval_seconds.unwrap_or(60))
        .fetch_one(&self.db)
        .await?;

        Ok(AuditExportConfigurationResponse {
            id: config.id,
            organization_id: config.organization_id,
            name: config.name,
            is_enabled: config.is_enabled,
            export_type: config.export_type,
            webhook_url: config.webhook_url,
            has_webhook_secret: config.webhook_secret_encrypted.is_some(),
            webhook_headers: config.webhook_headers,
            s3_bucket: config.s3_bucket,
            s3_prefix: config.s3_prefix,
            s3_region: config.s3_region,
            has_s3_credentials: config.s3_access_key_encrypted.is_some(),
            format: config.format,
            include_pii: config.include_pii,
            event_types: config.event_types,
            min_severity: config.min_severity,
            batch_size: config.batch_size,
            flush_interval_seconds: config.flush_interval_seconds,
            last_export_at: config.last_export_at,
            total_events_exported: config.total_events_exported,
            total_failures: config.total_failures,
            last_error: config.last_error,
            created_at: config.created_at,
            updated_at: config.updated_at,
        })
    }

    pub async fn delete_audit_export_configuration(&self, org_id: Uuid, config_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM audit_export_configurations WHERE id = $1 AND organization_id = $2")
            .bind(config_id)
            .bind(org_id)
            .execute(&self.db)
            .await?;
        Ok(())
    }

    // ========================================================================
    // BRANDING
    // ========================================================================

    pub async fn get_branding(&self, org_id: Uuid) -> AppResult<Option<BrandingConfiguration>> {
        let branding = sqlx::query_as::<_, BrandingConfiguration>(
            "SELECT * FROM branding_configurations WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(branding)
    }

    pub async fn update_branding(&self, org_id: Uuid, input: UpdateBrandingConfiguration) -> AppResult<BrandingConfiguration> {
        // Upsert branding configuration
        let branding = sqlx::query_as::<_, BrandingConfiguration>(
            r#"
            INSERT INTO branding_configurations (organization_id, logo_url, logo_dark_url, favicon_url,
                primary_color, secondary_color, accent_color, dark_mode_enabled, default_theme,
                font_family, heading_font_family, custom_css, login_background_url, login_message,
                show_powered_by, email_header_html, email_footer_html, email_logo_url,
                pdf_header_html, pdf_footer_html, pdf_cover_image_url)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21)
            ON CONFLICT (organization_id) DO UPDATE SET
                logo_url = COALESCE($2, branding_configurations.logo_url),
                logo_dark_url = COALESCE($3, branding_configurations.logo_dark_url),
                favicon_url = COALESCE($4, branding_configurations.favicon_url),
                primary_color = COALESCE($5, branding_configurations.primary_color),
                secondary_color = COALESCE($6, branding_configurations.secondary_color),
                accent_color = COALESCE($7, branding_configurations.accent_color),
                dark_mode_enabled = COALESCE($8, branding_configurations.dark_mode_enabled),
                default_theme = COALESCE($9, branding_configurations.default_theme),
                font_family = COALESCE($10, branding_configurations.font_family),
                heading_font_family = COALESCE($11, branding_configurations.heading_font_family),
                custom_css = COALESCE($12, branding_configurations.custom_css),
                login_background_url = COALESCE($13, branding_configurations.login_background_url),
                login_message = COALESCE($14, branding_configurations.login_message),
                show_powered_by = COALESCE($15, branding_configurations.show_powered_by),
                email_header_html = COALESCE($16, branding_configurations.email_header_html),
                email_footer_html = COALESCE($17, branding_configurations.email_footer_html),
                email_logo_url = COALESCE($18, branding_configurations.email_logo_url),
                pdf_header_html = COALESCE($19, branding_configurations.pdf_header_html),
                pdf_footer_html = COALESCE($20, branding_configurations.pdf_footer_html),
                pdf_cover_image_url = COALESCE($21, branding_configurations.pdf_cover_image_url),
                updated_at = NOW()
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(&input.logo_url)
        .bind(&input.logo_dark_url)
        .bind(&input.favicon_url)
        .bind(&input.primary_color)
        .bind(&input.secondary_color)
        .bind(&input.accent_color)
        .bind(&input.dark_mode_enabled)
        .bind(&input.default_theme)
        .bind(&input.font_family)
        .bind(&input.heading_font_family)
        .bind(&input.custom_css)
        .bind(&input.login_background_url)
        .bind(&input.login_message)
        .bind(&input.show_powered_by)
        .bind(&input.email_header_html)
        .bind(&input.email_footer_html)
        .bind(&input.email_logo_url)
        .bind(&input.pdf_header_html)
        .bind(&input.pdf_footer_html)
        .bind(&input.pdf_cover_image_url)
        .fetch_one(&self.db)
        .await?;

        Ok(branding)
    }

    pub async fn set_custom_domain(&self, org_id: Uuid, input: SetCustomDomainRequest) -> AppResult<DomainVerificationInstructions> {
        let token = Uuid::new_v4().to_string().replace("-", "");
        let verification_value = format!("opengrc-verify={}", token);

        sqlx::query(
            r#"
            INSERT INTO domain_verifications (organization_id, domain, verification_type, verification_token, verification_value)
            VALUES ($1, $2, 'dns_txt', $3, $4)
            ON CONFLICT (organization_id, domain) DO UPDATE SET
                verification_token = $3,
                verification_value = $4,
                is_verified = FALSE,
                verified_at = NULL,
                last_check_at = NULL
            "#
        )
        .bind(org_id)
        .bind(&input.domain)
        .bind(&token)
        .bind(&verification_value)
        .execute(&self.db)
        .await?;

        Ok(DomainVerificationInstructions {
            domain: input.domain.clone(),
            verification_type: "dns_txt".to_string(),
            dns_record_type: "TXT".to_string(),
            dns_record_name: format!("_opengrc.{}", input.domain),
            dns_record_value: verification_value,
            verification_token: token,
        })
    }

    // ========================================================================
    // API KEYS
    // ========================================================================

    pub async fn list_api_keys(&self, org_id: Uuid) -> AppResult<Vec<ApiKeyResponse>> {
        let keys = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE organization_id = $1 ORDER BY created_at DESC"
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(keys.into_iter().map(|k| ApiKeyResponse {
            id: k.id,
            organization_id: k.organization_id,
            user_id: k.user_id,
            name: k.name,
            description: k.description,
            key_prefix: k.key_prefix,
            scopes: k.scopes,
            role_id: k.role_id,
            rate_limit_per_minute: k.rate_limit_per_minute,
            rate_limit_per_hour: k.rate_limit_per_hour,
            expires_at: k.expires_at,
            last_used_at: k.last_used_at,
            is_active: k.is_active,
            revoked_at: k.revoked_at,
            created_at: k.created_at,
        }).collect())
    }

    pub async fn create_api_key(&self, org_id: Uuid, user_id: Uuid, input: CreateApiKey) -> AppResult<CreateApiKeyResponse> {
        // Generate the API key
        let key_id = Uuid::new_v4().to_string().replace("-", "")[..16].to_string();
        let key_secret = Uuid::new_v4().to_string().replace("-", "");
        let full_key = format!("ogrc_{}_{}", key_id, key_secret);
        let key_prefix = format!("ogrc_{}...", &key_id[..8]);

        // Hash the full key
        let mut hasher = Sha256::new();
        hasher.update(full_key.as_bytes());
        let key_hash = format!("{:x}", hasher.finalize());

        let expires_at = input.expires_in_days.map(|days| Utc::now() + Duration::days(days as i64));

        let key = sqlx::query_as::<_, ApiKey>(
            r#"
            INSERT INTO api_keys (
                organization_id, user_id, name, description, key_prefix, key_hash,
                scopes, role_id, rate_limit_per_minute, rate_limit_per_hour, expires_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(user_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&key_prefix)
        .bind(&key_hash)
        .bind(&input.scopes)
        .bind(&input.role_id)
        .bind(input.rate_limit_per_minute.unwrap_or(60))
        .bind(input.rate_limit_per_hour.unwrap_or(1000))
        .bind(expires_at)
        .fetch_one(&self.db)
        .await?;

        Ok(CreateApiKeyResponse {
            api_key: ApiKeyResponse {
                id: key.id,
                organization_id: key.organization_id,
                user_id: key.user_id,
                name: key.name,
                description: key.description,
                key_prefix: key.key_prefix,
                scopes: key.scopes,
                role_id: key.role_id,
                rate_limit_per_minute: key.rate_limit_per_minute,
                rate_limit_per_hour: key.rate_limit_per_hour,
                expires_at: key.expires_at,
                last_used_at: key.last_used_at,
                is_active: key.is_active,
                revoked_at: key.revoked_at,
                created_at: key.created_at,
            },
            key: full_key,
        })
    }

    pub async fn revoke_api_key(&self, org_id: Uuid, key_id: Uuid, user_id: Uuid, input: RevokeApiKeyRequest) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE api_keys SET
                is_active = FALSE,
                revoked_at = NOW(),
                revoked_by = $1,
                revoke_reason = $2
            WHERE id = $3 AND organization_id = $4
            "#
        )
        .bind(user_id)
        .bind(&input.reason)
        .bind(key_id)
        .bind(org_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    pub async fn validate_api_key(&self, key: &str) -> AppResult<Option<ApiKey>> {
        // Hash the provided key
        let mut hasher = Sha256::new();
        hasher.update(key.as_bytes());
        let key_hash = format!("{:x}", hasher.finalize());

        let api_key = sqlx::query_as::<_, ApiKey>(
            r#"
            SELECT * FROM api_keys
            WHERE key_hash = $1 AND is_active = TRUE
            AND (expires_at IS NULL OR expires_at > NOW())
            "#
        )
        .bind(&key_hash)
        .fetch_optional(&self.db)
        .await?;

        if let Some(ref key) = api_key {
            // Update last used
            sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = $1")
                .bind(key.id)
                .execute(&self.db)
                .await?;
        }

        Ok(api_key)
    }

    // ========================================================================
    // RATE LIMITING (Redis-based)
    // ========================================================================

    pub async fn check_rate_limit(&self, org_id: Uuid, limit_minute: i32, limit_hour: i32) -> AppResult<RateLimitStatus> {
        let now = Utc::now();
        let minute_key = format!("rate_limit:{}:minute:{}", org_id, now.format("%Y%m%d%H%M"));
        let hour_key = format!("rate_limit:{}:hour:{}", org_id, now.format("%Y%m%d%H"));

        // Get current counts
        let minute_count = self.cache.increment(&minute_key).await?;
        let hour_count = self.cache.increment(&hour_key).await?;

        // Set expiry if this is the first request
        if minute_count == 1 {
            self.cache.expire(&minute_key, StdDuration::from_secs(60)).await?;
        }
        if hour_count == 1 {
            self.cache.expire(&hour_key, StdDuration::from_secs(3600)).await?;
        }

        let is_limited = minute_count > limit_minute as i64 || hour_count > limit_hour as i64;

        Ok(RateLimitStatus {
            remaining_minute: (limit_minute as i64 - minute_count).max(0),
            remaining_hour: (limit_hour as i64 - hour_count).max(0),
            reset_minute: (now + Duration::minutes(1)).timestamp(),
            reset_hour: (now + Duration::hours(1)).timestamp(),
            is_limited,
        })
    }

    pub async fn get_usage_stats(&self, org_id: Uuid) -> AppResult<UsageStats> {
        let now = Utc::now();
        let minute_key = format!("rate_limit:{}:minute:{}", org_id, now.format("%Y%m%d%H%M"));
        let hour_key = format!("rate_limit:{}:hour:{}", org_id, now.format("%Y%m%d%H"));
        let day_key = format!("rate_limit:{}:day:{}", org_id, now.format("%Y%m%d"));

        let minute_count: i64 = self.cache.get(&minute_key).await?.unwrap_or(0);
        let hour_count: i64 = self.cache.get(&hour_key).await?.unwrap_or(0);
        let day_count: i64 = self.cache.get(&day_key).await?.unwrap_or(0);

        Ok(UsageStats {
            requests_today: day_count,
            requests_this_hour: hour_count,
            requests_this_minute: minute_count,
            quota_remaining: None, // Could be implemented based on subscription
            quota_reset_at: None,
        })
    }

    // ========================================================================
    // ENTERPRISE STATS
    // ========================================================================

    pub async fn get_enterprise_stats(&self, org_id: Uuid) -> AppResult<EnterpriseStats> {
        let roles_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM roles WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let custom_roles_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM roles WHERE organization_id = $1 AND is_system = FALSE"
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let sso_config = self.get_sso_configuration(org_id).await?;
        let scim_config = self.get_scim_configuration(org_id).await?;

        let audit_exports_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_export_configurations WHERE organization_id = $1"
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let api_keys_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM api_keys WHERE organization_id = $1 AND is_active = TRUE"
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let branding = self.get_branding(org_id).await?;

        Ok(EnterpriseStats {
            roles_count: roles_count.0,
            custom_roles_count: custom_roles_count.0,
            sso_enabled: sso_config.map(|c| c.is_enabled).unwrap_or(false),
            scim_enabled: scim_config.map(|c| c.is_enabled).unwrap_or(false),
            audit_exports_count: audit_exports_count.0,
            api_keys_count: api_keys_count.0,
            has_custom_branding: branding.as_ref().map(|b| b.logo_url.is_some()).unwrap_or(false),
            has_custom_domain: branding.map(|b| b.custom_domain.is_some()).unwrap_or(false),
        })
    }

    // ========================================================================
    // HELPERS
    // ========================================================================

    fn encrypt_secret(&self, secret: &str) -> String {
        // In a real implementation, use proper encryption
        // For now, just base64 encode (NOT SECURE - just placeholder)
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(secret)
    }
}
