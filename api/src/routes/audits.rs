use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    Audit, AuditFinding, AuditRequest, AuditRequestResponse, AuditStats, AuditWithStats,
    CreateAudit, CreateAuditFinding, CreateAuditRequest, CreateRequestResponse, ListAuditsQuery,
    UpdateAudit, UpdateAuditFinding,
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
    Uuid::parse_str(&user.id).map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))
}

// ==================== Query Params ====================

#[derive(Debug, Deserialize)]
pub struct ListAuditsParams {
    pub status: Option<String>,
    pub audit_type: Option<String>,
    pub framework_id: Option<Uuid>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListAuditsParams> for ListAuditsQuery {
    fn from(params: ListAuditsParams) -> Self {
        ListAuditsQuery {
            status: params.status,
            audit_type: params.audit_type,
            framework_id: params.framework_id,
            search: params.search,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

// ==================== Audit CRUD ====================

pub async fn list_audits(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListAuditsParams>,
) -> AppResult<Json<Vec<AuditWithStats>>> {
    let org_id = get_org_id(&user)?;
    let audits = services.audit.list_audits(org_id, params.into()).await?;
    Ok(Json(audits))
}

pub async fn get_audit(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<AuditWithStats>> {
    let org_id = get_org_id(&user)?;
    let audit = services.audit.get_audit(org_id, id).await?;
    Ok(Json(audit))
}

pub async fn create_audit(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateAudit>,
) -> AppResult<Json<Audit>> {
    let org_id = get_org_id(&user)?;
    let audit = services.audit.create_audit(org_id, input).await?;
    Ok(Json(audit))
}

pub async fn update_audit(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAudit>,
) -> AppResult<Json<Audit>> {
    let org_id = get_org_id(&user)?;
    let audit = services.audit.update_audit(org_id, id, input).await?;
    Ok(Json(audit))
}

pub async fn delete_audit(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services.audit.delete_audit(org_id, id).await?;
    Ok(Json(()))
}

// ==================== Audit Statistics ====================

pub async fn get_audit_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<AuditStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.audit.get_stats(org_id).await?;
    Ok(Json(stats))
}

// ==================== Audit Requests ====================

pub async fn list_requests(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(audit_id): Path<Uuid>,
) -> AppResult<Json<Vec<AuditRequest>>> {
    let org_id = get_org_id(&user)?;
    let requests = services.audit.list_requests(org_id, audit_id).await?;
    Ok(Json(requests))
}

pub async fn create_request(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(audit_id): Path<Uuid>,
    Json(input): Json<CreateAuditRequest>,
) -> AppResult<Json<AuditRequest>> {
    let org_id = get_org_id(&user)?;
    let request = services.audit.create_request(org_id, audit_id, input).await?;
    Ok(Json(request))
}

#[derive(Debug, Deserialize)]
pub struct RequestResponsePath {
    pub audit_id: Uuid,
    pub request_id: Uuid,
}

pub async fn add_response(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<RequestResponsePath>,
    Json(input): Json<CreateRequestResponse>,
) -> AppResult<Json<AuditRequestResponse>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let response = services
        .audit
        .add_response(org_id, path.audit_id, path.request_id, Some(user_id), input)
        .await?;
    Ok(Json(response))
}

// ==================== Audit Findings ====================

pub async fn list_findings(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(audit_id): Path<Uuid>,
) -> AppResult<Json<Vec<AuditFinding>>> {
    let org_id = get_org_id(&user)?;
    let findings = services.audit.list_findings(org_id, audit_id).await?;
    Ok(Json(findings))
}

pub async fn create_finding(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(audit_id): Path<Uuid>,
    Json(input): Json<CreateAuditFinding>,
) -> AppResult<Json<AuditFinding>> {
    let org_id = get_org_id(&user)?;
    let finding = services.audit.create_finding(org_id, audit_id, input).await?;
    Ok(Json(finding))
}

#[derive(Debug, Deserialize)]
pub struct FindingPath {
    pub audit_id: Uuid,
    pub finding_id: Uuid,
}

pub async fn update_finding(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<FindingPath>,
    Json(input): Json<UpdateAuditFinding>,
) -> AppResult<Json<AuditFinding>> {
    let org_id = get_org_id(&user)?;
    let finding = services
        .audit
        .update_finding(org_id, path.audit_id, path.finding_id, input)
        .await?;
    Ok(Json(finding))
}
