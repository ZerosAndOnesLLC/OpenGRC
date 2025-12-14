use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    CreateEvidenceCollectionTask, CreateMappingRule, EvidenceChangeWithDetails,
    EvidenceCollectionTaskWithStats, EvidenceControlMappingRule, EvidenceFreshnessSla,
    EvidenceFreshnessSummary, EvidenceWithFreshness, ListCollectionTasksQuery,
    ListEvidenceChangesQuery, StaleEvidenceBySource, UpdateEvidenceCollectionTask, UpdateMappingRule,
};
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}

fn get_user_id(user: &AuthUser) -> AppResult<Uuid> {
    Uuid::parse_str(&user.id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))
}

// ==================== Freshness ====================

/// GET /api/v1/evidence/freshness/summary
/// Get freshness summary for dashboard
pub async fn get_freshness_summary(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<EvidenceFreshnessSummary>> {
    let org_id = get_org_id(&user)?;
    let summary = services.evidence_automation.get_freshness_summary(org_id).await?;
    Ok(Json(summary))
}

/// GET /api/v1/evidence/freshness/stale
/// Get stale evidence grouped by source
#[derive(Debug, serde::Deserialize)]
pub struct StaleQuery {
    pub min_staleness: Option<i32>,
}

pub async fn get_stale_by_source(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<StaleQuery>,
) -> AppResult<Json<Vec<StaleEvidenceBySource>>> {
    let org_id = get_org_id(&user)?;
    let min_staleness = query.min_staleness.unwrap_or(0);
    let result = services.evidence_automation.get_stale_by_source(org_id, min_staleness).await?;
    Ok(Json(result))
}

/// POST /api/v1/evidence/freshness/update
/// Trigger freshness score update
pub async fn update_freshness_scores(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let updated = services.evidence_automation.update_freshness_scores(Some(org_id)).await?;
    Ok(Json(serde_json::json!({ "updated": updated })))
}

/// GET /api/v1/evidence/:id/freshness
/// Get evidence with freshness information
pub async fn get_evidence_freshness(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<EvidenceWithFreshness>> {
    let org_id = get_org_id(&user)?;
    let evidence = services.evidence_automation.get_evidence_with_freshness(org_id, id).await?;
    Ok(Json(evidence))
}

/// GET /api/v1/evidence/freshness/slas
/// List freshness SLAs
pub async fn list_freshness_slas(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<EvidenceFreshnessSla>>> {
    let org_id = get_org_id(&user)?;
    let slas = services.evidence_automation.list_freshness_slas(org_id).await?;
    Ok(Json(slas))
}

// ==================== Collection Tasks ====================

/// GET /api/v1/evidence/collection-tasks
/// List collection tasks
pub async fn list_collection_tasks(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<ListCollectionTasksQuery>,
) -> AppResult<Json<Vec<EvidenceCollectionTaskWithStats>>> {
    let org_id = get_org_id(&user)?;
    let tasks = services.evidence_automation.list_collection_tasks(org_id, query).await?;
    Ok(Json(tasks))
}

/// POST /api/v1/evidence/collection-tasks
/// Create a new collection task
pub async fn create_collection_task(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateEvidenceCollectionTask>,
) -> AppResult<Json<crate::models::EvidenceCollectionTask>> {
    let org_id = get_org_id(&user)?;
    let task = services.evidence_automation.create_collection_task(org_id, input).await?;
    Ok(Json(task))
}

/// PUT /api/v1/evidence/collection-tasks/:id
/// Update a collection task
pub async fn update_collection_task(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateEvidenceCollectionTask>,
) -> AppResult<Json<crate::models::EvidenceCollectionTask>> {
    let org_id = get_org_id(&user)?;
    let task = services.evidence_automation.update_collection_task(org_id, id, input).await?;
    Ok(Json(task))
}

/// DELETE /api/v1/evidence/collection-tasks/:id
/// Delete a collection task
pub async fn delete_collection_task(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    services.evidence_automation.delete_collection_task(org_id, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ==================== Changes ====================

/// GET /api/v1/evidence/changes
/// List evidence changes
pub async fn list_evidence_changes(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<ListEvidenceChangesQuery>,
) -> AppResult<Json<Vec<EvidenceChangeWithDetails>>> {
    let org_id = get_org_id(&user)?;
    let changes = services.evidence_automation.list_evidence_changes(org_id, query).await?;
    Ok(Json(changes))
}

/// PUT /api/v1/evidence/changes/:id/acknowledge
/// Acknowledge a change
pub async fn acknowledge_change(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    services.evidence_automation.acknowledge_change(org_id, id, user_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

/// GET /api/v1/evidence/changes/count
/// Get pending change count
pub async fn get_pending_change_count(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let count = services.evidence_automation.get_pending_change_count(org_id).await?;
    Ok(Json(serde_json::json!({ "pending_count": count })))
}

// ==================== Mapping Rules ====================

/// GET /api/v1/evidence/mapping-rules
/// List mapping rules
pub async fn list_mapping_rules(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<EvidenceControlMappingRule>>> {
    let org_id = get_org_id(&user)?;
    let rules = services.evidence_automation.list_mapping_rules(org_id).await?;
    Ok(Json(rules))
}

/// POST /api/v1/evidence/mapping-rules
/// Create a mapping rule
pub async fn create_mapping_rule(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateMappingRule>,
) -> AppResult<Json<EvidenceControlMappingRule>> {
    let org_id = get_org_id(&user)?;
    let rule = services.evidence_automation.create_mapping_rule(org_id, input).await?;
    Ok(Json(rule))
}

/// PUT /api/v1/evidence/mapping-rules/:id
/// Update a mapping rule
pub async fn update_mapping_rule(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateMappingRule>,
) -> AppResult<Json<EvidenceControlMappingRule>> {
    let org_id = get_org_id(&user)?;
    let rule = services.evidence_automation.update_mapping_rule(org_id, id, input).await?;
    Ok(Json(rule))
}

/// DELETE /api/v1/evidence/mapping-rules/:id
/// Delete a mapping rule
pub async fn delete_mapping_rule(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    services.evidence_automation.delete_mapping_rule(org_id, id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}
