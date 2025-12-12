use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    Control, ControlStats, ControlTest, ControlTestResult, ControlWithMappings,
    CreateControl, CreateControlTest, CreateTestResult, ListControlsQuery,
    UpdateControl,
};
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

// ==================== Query Params ====================

#[derive(Debug, Deserialize)]
pub struct ListControlsParams {
    pub status: Option<String>,
    pub control_type: Option<String>,
    pub owner_id: Option<Uuid>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListControlsParams> for ListControlsQuery {
    fn from(params: ListControlsParams) -> Self {
        ListControlsQuery {
            status: params.status,
            control_type: params.control_type,
            owner_id: params.owner_id,
            search: params.search,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct MapRequirementsRequest {
    pub requirement_ids: Vec<Uuid>,
}

// ==================== Control Routes ====================

/// GET /api/v1/controls
pub async fn list_controls(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListControlsParams>,
) -> AppResult<Json<Vec<ControlWithMappings>>> {
    let org_id = get_org_id(&user)?;
    let controls = services.control.list_controls(org_id, params.into()).await?;
    Ok(Json(controls))
}

/// GET /api/v1/controls/stats
pub async fn get_control_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<ControlStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.control.get_stats(org_id).await?;
    Ok(Json(stats))
}

/// GET /api/v1/controls/:id
pub async fn get_control(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ControlWithMappings>> {
    let org_id = get_org_id(&user)?;
    let control = services.control.get_control(org_id, id).await?;
    Ok(Json(control))
}

/// POST /api/v1/controls
pub async fn create_control(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateControl>,
) -> AppResult<Json<Control>> {
    let org_id = get_org_id(&user)?;
    let control = services.control.create_control(org_id, input).await?;
    Ok(Json(control))
}

/// PUT /api/v1/controls/:id
pub async fn update_control(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateControl>,
) -> AppResult<Json<Control>> {
    let org_id = get_org_id(&user)?;
    let control = services.control.update_control(org_id, id, input).await?;
    Ok(Json(control))
}

/// DELETE /api/v1/controls/:id
pub async fn delete_control(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    services.control.delete_control(org_id, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ==================== Requirement Mapping Routes ====================

/// POST /api/v1/controls/:id/requirements
pub async fn map_requirements(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(control_id): Path<Uuid>,
    Json(input): Json<MapRequirementsRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let mappings = services
        .control
        .map_requirements(org_id, control_id, input.requirement_ids)
        .await?;
    Ok(Json(serde_json::json!({
        "mapped": mappings.len(),
        "mappings": mappings
    })))
}

/// DELETE /api/v1/controls/:id/requirements
pub async fn unmap_requirements(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(control_id): Path<Uuid>,
    Json(input): Json<MapRequirementsRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let deleted = services
        .control
        .unmap_requirements(org_id, control_id, input.requirement_ids)
        .await?;
    Ok(Json(serde_json::json!({ "unmapped": deleted })))
}

// ==================== Control Test Routes ====================

/// GET /api/v1/controls/:id/tests
pub async fn list_control_tests(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(control_id): Path<Uuid>,
) -> AppResult<Json<Vec<ControlTest>>> {
    let org_id = get_org_id(&user)?;
    let tests = services.control.list_tests(org_id, control_id).await?;
    Ok(Json(tests))
}

/// POST /api/v1/controls/:id/tests
pub async fn create_control_test(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(control_id): Path<Uuid>,
    Json(input): Json<CreateControlTest>,
) -> AppResult<Json<ControlTest>> {
    let org_id = get_org_id(&user)?;
    let test = services
        .control
        .create_test(org_id, control_id, input)
        .await?;
    Ok(Json(test))
}

/// POST /api/v1/controls/:control_id/tests/:test_id/results
pub async fn record_test_result(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path((control_id, test_id)): Path<(Uuid, Uuid)>,
    Json(input): Json<CreateTestResult>,
) -> AppResult<Json<ControlTestResult>> {
    let org_id = get_org_id(&user)?;
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;

    let result = services
        .control
        .record_test_result(org_id, control_id, test_id, user_id, input)
        .await?;
    Ok(Json(result))
}

// ==================== Helpers ====================

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}
