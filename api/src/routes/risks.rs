use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    CreateRisk, LinkControlsRequest, ListRisksQuery, Risk, RiskControlMapping,
    RiskStats, RiskWithControls, UpdateRisk,
};
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}

// ==================== Query Params ====================

#[derive(Debug, Deserialize)]
pub struct ListRisksParams {
    pub status: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    pub owner_id: Option<Uuid>,
    pub min_score: Option<i32>,
    pub max_score: Option<i32>,
    pub search: Option<String>,
    pub needs_review: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListRisksParams> for ListRisksQuery {
    fn from(params: ListRisksParams) -> Self {
        ListRisksQuery {
            status: params.status,
            category: params.category,
            source: params.source,
            owner_id: params.owner_id,
            min_score: params.min_score,
            max_score: params.max_score,
            search: params.search,
            needs_review: params.needs_review,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UnlinkControlsRequest {
    pub control_ids: Vec<Uuid>,
}

// ==================== Risk CRUD ====================

pub async fn list_risks(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListRisksParams>,
) -> AppResult<Json<Vec<RiskWithControls>>> {
    let org_id = get_org_id(&user)?;
    let risks = services.risk.list_risks(org_id, params.into()).await?;
    Ok(Json(risks))
}

pub async fn get_risk(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<RiskWithControls>> {
    let org_id = get_org_id(&user)?;
    let risk = services.risk.get_risk(org_id, id).await?;
    Ok(Json(risk))
}

pub async fn create_risk(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateRisk>,
) -> AppResult<Json<Risk>> {
    let org_id = get_org_id(&user)?;
    let risk = services.risk.create_risk(org_id, input).await?;
    Ok(Json(risk))
}

pub async fn update_risk(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateRisk>,
) -> AppResult<Json<Risk>> {
    let org_id = get_org_id(&user)?;
    let risk = services.risk.update_risk(org_id, id, input).await?;
    Ok(Json(risk))
}

pub async fn delete_risk(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services.risk.delete_risk(org_id, id).await?;
    Ok(Json(()))
}

// ==================== Risk Statistics ====================

pub async fn get_risk_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<RiskStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.risk.get_stats(org_id).await?;
    Ok(Json(stats))
}

// ==================== Control Mappings ====================

pub async fn link_controls(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<LinkControlsRequest>,
) -> AppResult<Json<Vec<RiskControlMapping>>> {
    let org_id = get_org_id(&user)?;
    let mappings = services
        .risk
        .link_controls(org_id, id, input.control_ids, input.effectiveness)
        .await?;
    Ok(Json(mappings))
}

pub async fn unlink_controls(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UnlinkControlsRequest>,
) -> AppResult<Json<i64>> {
    let org_id = get_org_id(&user)?;
    let deleted = services
        .risk
        .unlink_controls(org_id, id, input.control_ids)
        .await?;
    Ok(Json(deleted))
}
