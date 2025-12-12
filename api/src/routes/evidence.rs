use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    CreateEvidence, Evidence, EvidenceStats, EvidenceWithLinks, ListEvidenceQuery,
    UpdateEvidence,
};
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

// ==================== Query Params ====================

#[derive(Debug, Deserialize)]
pub struct ListEvidenceParams {
    pub evidence_type: Option<String>,
    pub source: Option<String>,
    pub control_id: Option<Uuid>,
    pub search: Option<String>,
    pub expired: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListEvidenceParams> for ListEvidenceQuery {
    fn from(params: ListEvidenceParams) -> Self {
        ListEvidenceQuery {
            evidence_type: params.evidence_type,
            source: params.source,
            control_id: params.control_id,
            search: params.search,
            expired: params.expired,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LinkControlsRequest {
    pub control_ids: Vec<Uuid>,
}

// ==================== Evidence Routes ====================

/// GET /api/v1/evidence
pub async fn list_evidence(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListEvidenceParams>,
) -> AppResult<Json<Vec<EvidenceWithLinks>>> {
    let org_id = get_org_id(&user)?;
    let evidence = services.evidence.list_evidence(org_id, params.into()).await?;
    Ok(Json(evidence))
}

/// GET /api/v1/evidence/stats
pub async fn get_evidence_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<EvidenceStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.evidence.get_stats(org_id).await?;
    Ok(Json(stats))
}

/// GET /api/v1/evidence/:id
pub async fn get_evidence(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<EvidenceWithLinks>> {
    let org_id = get_org_id(&user)?;
    let evidence = services.evidence.get_evidence(org_id, id).await?;
    Ok(Json(evidence))
}

/// POST /api/v1/evidence
pub async fn create_evidence(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateEvidence>,
) -> AppResult<Json<Evidence>> {
    let org_id = get_org_id(&user)?;
    let user_id = Uuid::parse_str(&user.id).ok();
    let evidence = services.evidence.create_evidence(org_id, user_id, input).await?;
    Ok(Json(evidence))
}

/// PUT /api/v1/evidence/:id
pub async fn update_evidence(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateEvidence>,
) -> AppResult<Json<Evidence>> {
    let org_id = get_org_id(&user)?;
    let evidence = services.evidence.update_evidence(org_id, id, input).await?;
    Ok(Json(evidence))
}

/// DELETE /api/v1/evidence/:id
pub async fn delete_evidence(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    services.evidence.delete_evidence(org_id, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ==================== Control Links ====================

/// POST /api/v1/evidence/:id/controls
pub async fn link_controls(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(evidence_id): Path<Uuid>,
    Json(input): Json<LinkControlsRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let user_id = Uuid::parse_str(&user.id).ok();
    let links = services
        .evidence
        .link_to_controls(org_id, evidence_id, input.control_ids, user_id)
        .await?;
    Ok(Json(serde_json::json!({
        "linked": links.len(),
        "links": links
    })))
}

/// DELETE /api/v1/evidence/:id/controls
pub async fn unlink_controls(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(evidence_id): Path<Uuid>,
    Json(input): Json<LinkControlsRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let deleted = services
        .evidence
        .unlink_from_controls(org_id, evidence_id, input.control_ids)
        .await?;
    Ok(Json(serde_json::json!({ "unlinked": deleted })))
}

// ==================== Helpers ====================

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}
