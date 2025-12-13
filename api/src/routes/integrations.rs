use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    get_available_integrations, CreateIntegration, ListIntegrationsQuery, TriggerSyncRequest,
    UpdateIntegration,
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
