use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    CreateFramework, CreateFrameworkRequirement, Framework, FrameworkRequirement,
    FrameworkWithRequirements, UpdateFramework, UpdateFrameworkRequirement,
    FrameworkGapAnalysis,
};
use crate::models::framework::build_requirement_tree;
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct ListFrameworksQuery {
    pub category: Option<String>,
    pub is_system: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListRequirementsQuery {
    pub tree: Option<bool>,
}

// ==================== Framework Routes ====================

/// GET /api/v1/frameworks
pub async fn list_frameworks(
    State(services): State<Arc<AppServices>>,
    Query(query): Query<ListFrameworksQuery>,
) -> AppResult<Json<Vec<Framework>>> {
    let frameworks = services
        .framework
        .list_frameworks(query.category.as_deref(), query.is_system)
        .await?;

    Ok(Json(frameworks))
}

/// GET /api/v1/frameworks/:id
pub async fn get_framework(
    State(services): State<Arc<AppServices>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<FrameworkWithRequirements>> {
    let framework = services.framework.get_framework_with_requirements(id).await?;
    Ok(Json(framework))
}

/// POST /api/v1/frameworks
pub async fn create_framework(
    State(services): State<Arc<AppServices>>,
    Json(input): Json<CreateFramework>,
) -> AppResult<Json<Framework>> {
    let framework = services.framework.create_framework(input).await?;
    Ok(Json(framework))
}

/// PUT /api/v1/frameworks/:id
pub async fn update_framework(
    State(services): State<Arc<AppServices>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateFramework>,
) -> AppResult<Json<Framework>> {
    let framework = services.framework.update_framework(id, input).await?;
    Ok(Json(framework))
}

/// DELETE /api/v1/frameworks/:id
pub async fn delete_framework(
    State(services): State<Arc<AppServices>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    services.framework.delete_framework(id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ==================== Requirement Routes ====================

/// GET /api/v1/frameworks/:framework_id/requirements
pub async fn list_requirements(
    State(services): State<Arc<AppServices>>,
    Path(framework_id): Path<Uuid>,
    Query(query): Query<ListRequirementsQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let requirements = services.framework.list_requirements(framework_id).await?;

    // Return tree structure if requested
    if query.tree.unwrap_or(false) {
        let tree = build_requirement_tree(requirements);
        Ok(Json(serde_json::json!({ "tree": tree })))
    } else {
        Ok(Json(serde_json::json!({ "requirements": requirements })))
    }
}

/// GET /api/v1/frameworks/:framework_id/requirements/:id
pub async fn get_requirement(
    State(services): State<Arc<AppServices>>,
    Path((_framework_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<FrameworkRequirement>> {
    let requirement = services.framework.get_requirement(id).await?;
    Ok(Json(requirement))
}

/// POST /api/v1/frameworks/:framework_id/requirements
pub async fn create_requirement(
    State(services): State<Arc<AppServices>>,
    Path(framework_id): Path<Uuid>,
    Json(input): Json<CreateFrameworkRequirement>,
) -> AppResult<Json<FrameworkRequirement>> {
    let requirement = services
        .framework
        .create_requirement(framework_id, input)
        .await?;
    Ok(Json(requirement))
}

/// POST /api/v1/frameworks/:framework_id/requirements/batch
pub async fn batch_create_requirements(
    State(services): State<Arc<AppServices>>,
    Path(framework_id): Path<Uuid>,
    Json(input): Json<Vec<CreateFrameworkRequirement>>,
) -> AppResult<Json<Vec<FrameworkRequirement>>> {
    let requirements = services
        .framework
        .batch_create_requirements(framework_id, input)
        .await?;
    Ok(Json(requirements))
}

/// PUT /api/v1/frameworks/:framework_id/requirements/:id
pub async fn update_requirement(
    State(services): State<Arc<AppServices>>,
    Path((_framework_id, id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateFrameworkRequirement>,
) -> AppResult<Json<FrameworkRequirement>> {
    let requirement = services.framework.update_requirement(id, input).await?;
    Ok(Json(requirement))
}

/// DELETE /api/v1/frameworks/:framework_id/requirements/:id
pub async fn delete_requirement(
    State(services): State<Arc<AppServices>>,
    Path((_framework_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    services.framework.delete_requirement(id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ==================== Gap Analysis ====================

/// GET /api/v1/frameworks/:framework_id/gap-analysis
pub async fn get_gap_analysis(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(framework_id): Path<Uuid>,
) -> AppResult<Json<FrameworkGapAnalysis>> {
    let org_id = user
        .organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))?;

    let analysis = services.framework.get_gap_analysis(org_id, framework_id).await?;
    Ok(Json(analysis))
}
