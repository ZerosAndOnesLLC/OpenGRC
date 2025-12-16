use axum::{
    extract::{Path, Query, State},
    response::Redirect,
    Extension, Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    get_available_integrations, CreateIntegration, ListIntegrationsQuery, OAuthAuthorizeRequest,
    OAuthCallbackParams, OAuthRefreshRequest, TriggerSyncRequest, UpdateIntegration,
};
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

/// List all integrations for the organization
pub async fn list_integrations(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<ListIntegrationsQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let integrations = services
        .integration
        .list_integrations(org_id, query)
        .await?;

    Ok(Json(json!({
        "data": integrations,
        "count": integrations.len()
    })))
}

/// Get a single integration by ID
pub async fn get_integration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let integration = services
        .integration
        .get_integration(org_id, id)
        .await?;

    // Mask sensitive config fields before returning
    let mut response = serde_json::to_value(&integration)
        .map_err(|e| AppError::InternalServerError(e.to_string()))?;
    if let Some(config) = response
        .get_mut("integration")
        .and_then(|i| i.get_mut("config"))
    {
        mask_sensitive_fields(config);
    }

    Ok(Json(json!({ "data": response })))
}

/// Create a new integration
pub async fn create_integration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateIntegration>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let integration = services
        .integration
        .create_integration(org_id, input)
        .await?;

    Ok(Json(json!({
        "data": integration,
        "message": "Integration created successfully"
    })))
}

/// Update an integration
pub async fn update_integration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateIntegration>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let integration = services
        .integration
        .update_integration(org_id, id, input)
        .await?;

    Ok(Json(json!({
        "data": integration,
        "message": "Integration updated successfully"
    })))
}

/// Delete an integration
pub async fn delete_integration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    services
        .integration
        .delete_integration(org_id, id)
        .await?;

    Ok(Json(json!({
        "message": "Integration deleted successfully"
    })))
}

/// List available integration types
pub async fn list_available_integrations() -> Json<Value> {
    let available = get_available_integrations();
    Json(json!({
        "data": available,
        "count": available.len()
    }))
}

/// Get integration statistics
pub async fn get_integration_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let stats = services
        .integration
        .get_stats(org_id)
        .await?;

    Ok(Json(json!({ "data": stats })))
}

/// Test connection for an integration
pub async fn test_connection(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let result = services
        .integration
        .test_connection(org_id, id)
        .await?;

    Ok(Json(json!({ "data": result })))
}

/// Test connection with config (before creating)
#[derive(Debug, Deserialize)]
pub struct TestConnectionRequest {
    pub integration_type: String,
    pub config: Value,
}

pub async fn test_connection_preview(
    State(services): State<Arc<AppServices>>,
    Extension(_user): Extension<AuthUser>,
    Json(input): Json<TestConnectionRequest>,
) -> AppResult<Json<Value>> {
    let result = services
        .integration
        .test_connection_with_config(&input.integration_type, &input.config)
        .await?;

    Ok(Json(json!({ "data": result })))
}

/// Trigger a sync for an integration
pub async fn trigger_sync(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<TriggerSyncRequest>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let sync_log = services
        .integration
        .trigger_sync(
            org_id,
            id,
            input.sync_type,
            input.full_sync.unwrap_or(false),
        )
        .await?;

    Ok(Json(json!({
        "data": sync_log,
        "message": "Sync triggered successfully"
    })))
}

/// Collect evidence from an integration (runs sync and persists evidence)
pub async fn collect_evidence(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    use crate::integrations::{AwsProvider, GitHubProvider, JiraProvider, IntegrationProvider, SyncContext};

    let org_id = get_org_id(&user)?;

    // Get integration details
    let integration_with_stats = services.integration.get_integration(org_id, id).await?;
    let integration = &integration_with_stats.integration;
    let integration_name = integration.name.clone();
    let integration_type = integration.integration_type.clone();

    // Get the appropriate provider based on integration type
    let provider: Box<dyn IntegrationProvider> = match integration_type.as_str() {
        "aws" => Box::new(AwsProvider::new()),
        "github" => Box::new(GitHubProvider::new()),
        "jira" => Box::new(JiraProvider::new()),
        _ => {
            return Err(AppError::BadRequest(format!(
                "Evidence collection is not supported for {} integrations",
                integration_type
            )));
        }
    };

    // Decrypt config
    let encrypted_config = integration.config.as_ref().ok_or_else(|| {
        AppError::BadRequest("Integration has no configuration".to_string())
    })?;
    let config = services.integration.decrypt_config(encrypted_config)?;

    // Create sync context
    let context = SyncContext {
        organization_id: org_id,
        integration_id: id,
        sync_log_id: Uuid::new_v4(), // Temporary ID for evidence collection
        full_sync: true,
        sync_type: Some("evidence_collection".to_string()),
    };

    // Run sync with the appropriate provider
    let sync_result = provider
        .sync(&config, context)
        .await
        .map_err(|e| AppError::InternalServerError(format!("{} sync failed: {}", integration_type, e)))?;

    // Store security alerts before consuming evidence_collected
    let security_alerts = sync_result.security_alerts.clone();

    // Persist collected evidence
    let evidence_count = services
        .evidence
        .create_from_integration(org_id, id, sync_result.evidence_collected)
        .await?;

    // Process security alerts for AWS integrations
    let mut alerts_created = 0;
    if integration_type == "aws" {
        if let Some(alerts) = security_alerts {
            // CloudTrail security alerts
            if !alerts.root_actions.is_empty() || alerts.sensitive_actions.len() >= 5 || alerts.failed_actions.len() >= 10 {
                match services
                    .notification
                    .create_cloudtrail_security_alerts(
                        org_id,
                        id,
                        &integration_name,
                        alerts.root_actions,
                        alerts.sensitive_actions,
                        alerts.failed_actions,
                    )
                    .await
                {
                    Ok(count) => alerts_created += count,
                    Err(e) => tracing::warn!("Failed to create CloudTrail security alerts: {}", e),
                }
            }

            // Security Hub alerts
            if alerts.critical_findings_count > 0 || alerts.high_findings_count >= 10 {
                match services
                    .notification
                    .create_securityhub_alerts(
                        org_id,
                        id,
                        &integration_name,
                        alerts.critical_findings_count,
                        alerts.high_findings_count,
                        alerts.critical_findings,
                    )
                    .await
                {
                    Ok(count) => alerts_created += count,
                    Err(e) => tracing::warn!("Failed to create Security Hub alerts: {}", e),
                }
            }
        }
    }

    // Update integration last_sync_at
    sqlx::query("UPDATE integrations SET last_sync_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(&services.db)
        .await?;

    Ok(Json(json!({
        "data": {
            "evidence_created": evidence_count,
            "records_processed": sync_result.records_processed,
            "errors": sync_result.errors.len(),
            "success": sync_result.success,
            "alerts_created": alerts_created
        },
        "message": format!("Collected {} evidence records from {}", evidence_count, integration_type)
    })))
}

/// Get sync logs for an integration
#[derive(Debug, Deserialize, Default)]
pub struct SyncLogsQuery {
    pub limit: Option<i64>,
}

pub async fn get_sync_logs(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Query(query): Query<SyncLogsQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let logs = services
        .integration
        .get_sync_logs(org_id, id, query.limit.unwrap_or(20))
        .await?;

    Ok(Json(json!({
        "data": logs,
        "count": logs.len()
    })))
}

// ==================== Health Monitoring ====================

/// Get health for all integrations
pub async fn get_all_health(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let health = services
        .integration
        .get_all_health(org_id)
        .await?;

    Ok(Json(json!({
        "data": health,
        "count": health.len()
    })))
}

/// Get health for a specific integration
pub async fn get_integration_health(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let health = services
        .integration
        .get_integration_health(org_id, id)
        .await?;

    Ok(Json(json!({ "data": health })))
}

/// Get aggregated health statistics
pub async fn get_health_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let stats = services
        .integration
        .get_health_stats(org_id)
        .await?;

    Ok(Json(json!({ "data": stats })))
}

/// Get recent failures
#[derive(Debug, Deserialize, Default)]
pub struct RecentFailuresQuery {
    pub limit: Option<i64>,
}

pub async fn get_recent_failures(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<RecentFailuresQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let failures = services
        .integration
        .get_recent_failures(org_id, query.limit.unwrap_or(10))
        .await?;

    Ok(Json(json!({
        "data": failures,
        "count": failures.len()
    })))
}

/// Get health trend data for charts
#[derive(Debug, Deserialize, Default)]
pub struct HealthTrendQuery {
    pub hours: Option<i32>,
}

pub async fn get_health_trend(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<HealthTrendQuery>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let trend = services
        .integration
        .get_health_trend(org_id, query.hours.unwrap_or(24))
        .await?;

    Ok(Json(json!({
        "data": trend,
        "count": trend.len()
    })))
}

// ==================== OAuth Operations ====================

/// Start OAuth authorization flow for an integration type
pub async fn oauth_authorize(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(integration_type): Path<String>,
    Json(input): Json<OAuthAuthorizeRequest>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;

    // Check if OAuth is supported for this type
    let available = get_available_integrations();
    let integration_info = available
        .iter()
        .find(|i| i.integration_type == integration_type)
        .ok_or_else(|| AppError::NotFound(format!("Unknown integration type: {}", integration_type)))?;

    if !integration_info.auth_methods.contains(&"oauth2".to_string()) {
        return Err(AppError::ValidationError(format!(
            "{} does not support OAuth2 authentication",
            integration_type
        )));
    }

    // Build metadata for the OAuth state
    let mut metadata = serde_json::json!({});
    if let Some(name) = &input.integration_name {
        metadata["integration_name"] = serde_json::json!(name);
    }

    let response = services
        .integration
        .create_oauth_authorization(
            org_id,
            &integration_type,
            input.scopes,
            input.redirect_uri,
            Some(metadata),
        )
        .await?;

    Ok(Json(json!({
        "data": response,
        "message": "Redirect user to authorization_url"
    })))
}

/// Handle OAuth callback from provider
pub async fn oauth_callback(
    State(services): State<Arc<AppServices>>,
    Query(params): Query<OAuthCallbackParams>,
) -> Result<Redirect, AppError> {
    // Check for error from provider
    if let Some(error) = &params.error {
        let error_desc = params.error_description.as_deref().unwrap_or("Unknown error");
        tracing::error!("OAuth callback error: {} - {}", error, error_desc);

        // Redirect to UI with error
        let redirect_url = format!(
            "/integrations/oauth/error?error={}&description={}",
            urlencoding::encode(error),
            urlencoding::encode(error_desc)
        );
        return Ok(Redirect::temporary(&redirect_url));
    }

    // Validate and consume the OAuth state
    let oauth_state = services
        .integration
        .validate_oauth_state(&params.state)
        .await?;

    // Extract integration name from metadata
    let integration_name = oauth_state
        .metadata
        .as_ref()
        .and_then(|m| m.get("integration_name"))
        .and_then(|n| n.as_str());

    // Complete the OAuth flow
    let integration = services
        .integration
        .complete_oauth_flow(
            oauth_state.organization_id,
            &oauth_state.integration_type,
            &params.code,
            oauth_state.code_verifier.as_deref(),
            integration_name,
            oauth_state.metadata.as_ref(),
        )
        .await?;

    // Redirect to UI with success
    let redirect_url = format!(
        "/integrations/{}?oauth=success",
        integration.id
    );
    Ok(Redirect::temporary(&redirect_url))
}

/// Refresh OAuth tokens for an integration
pub async fn oauth_refresh(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<OAuthRefreshRequest>,
) -> AppResult<Json<Value>> {
    let org_id = get_org_id(&user)?;
    let force = input.force.unwrap_or(false);

    let integration = services
        .integration
        .refresh_oauth_tokens(org_id, id, force)
        .await?;

    Ok(Json(json!({
        "data": {
            "id": integration.id,
            "token_expires_at": integration.oauth_token_expires_at,
            "refreshed": true
        },
        "message": "OAuth tokens refreshed successfully"
    })))
}

/// Check OAuth configuration for a provider
pub async fn oauth_check(
    State(services): State<Arc<AppServices>>,
    Path(integration_type): Path<String>,
) -> Json<Value> {
    let oauth_service = services.integration.oauth_service();
    let is_configured = oauth_service.is_configured(&integration_type);

    // Get available info
    let available = get_available_integrations();
    let supports_oauth = available
        .iter()
        .find(|i| i.integration_type == integration_type)
        .map(|i| i.auth_methods.contains(&"oauth2".to_string()))
        .unwrap_or(false);

    Json(json!({
        "data": {
            "integration_type": integration_type,
            "supports_oauth": supports_oauth,
            "oauth_configured": is_configured
        }
    }))
}

/// Mask sensitive fields in config (passwords, tokens, secrets)
fn mask_sensitive_fields(config: &mut Value) {
    if let Value::Object(map) = config {
        for (key, value) in map.iter_mut() {
            let key_lower = key.to_lowercase();
            if key_lower.contains("secret")
                || key_lower.contains("password")
                || key_lower.contains("token")
                || key_lower.contains("key")
                || key_lower.contains("credential")
            {
                if let Value::String(s) = value {
                    if !s.is_empty() {
                        *value = Value::String("••••••••".to_string());
                    }
                }
            }
        }
    }
}

// ==================== Helpers ====================

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}
