use axum::{
    extract::{Multipart, Path, Query, State},
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
use crate::services::evidence::{PresignedDownloadResponse, PresignedUploadResponse};
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

#[derive(Debug, Deserialize)]
pub struct UploadUrlRequest {
    pub filename: String,
    pub content_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ConfirmUploadRequest {
    pub file_key: String,
    pub file_size: i64,
    pub mime_type: String,
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

// ==================== File Upload/Download ====================

/// POST /api/v1/evidence/:id/upload-url
/// Get a presigned URL for uploading a file directly to S3
pub async fn get_upload_url(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(evidence_id): Path<Uuid>,
    Json(input): Json<UploadUrlRequest>,
) -> AppResult<Json<PresignedUploadResponse>> {
    let org_id = get_org_id(&user)?;
    let response = services
        .evidence
        .get_upload_url(org_id, evidence_id, &input.filename, &input.content_type)
        .await?;
    Ok(Json(response))
}

/// POST /api/v1/evidence/:id/confirm-upload
/// Confirm that a file has been uploaded and update the evidence record
pub async fn confirm_upload(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(evidence_id): Path<Uuid>,
    Json(input): Json<ConfirmUploadRequest>,
) -> AppResult<Json<Evidence>> {
    let org_id = get_org_id(&user)?;
    let evidence = services
        .evidence
        .confirm_upload(org_id, evidence_id, &input.file_key, input.file_size, &input.mime_type)
        .await?;
    Ok(Json(evidence))
}

/// GET /api/v1/evidence/:id/download-url
/// Get a presigned URL for downloading the file
pub async fn get_download_url(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(evidence_id): Path<Uuid>,
) -> AppResult<Json<PresignedDownloadResponse>> {
    let org_id = get_org_id(&user)?;
    let response = services.evidence.get_download_url(org_id, evidence_id).await?;
    Ok(Json(response))
}

/// POST /api/v1/evidence/:id/upload
/// Upload a file directly via multipart form (for smaller files)
pub async fn upload_file(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(evidence_id): Path<Uuid>,
    mut multipart: Multipart,
) -> AppResult<Json<Evidence>> {
    let org_id = get_org_id(&user)?;

    // Extract file from multipart
    let mut filename: Option<String> = None;
    let mut content_type: Option<String> = None;
    let mut data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        if field.name() == Some("file") {
            filename = field.file_name().map(|s| s.to_string());
            content_type = field.content_type().map(|s| s.to_string());
            data = Some(field.bytes().await.map_err(|e| {
                AppError::BadRequest(format!("Failed to read file data: {}", e))
            })?.to_vec());
            break;
        }
    }

    let filename = filename.ok_or_else(|| AppError::BadRequest("No file provided".to_string()))?;
    let content_type = content_type.unwrap_or_else(|| {
        mime_guess::from_path(&filename)
            .first_or_octet_stream()
            .to_string()
    });
    let data = data.ok_or_else(|| AppError::BadRequest("No file data".to_string()))?;

    // Limit file size to 50MB for direct upload
    const MAX_SIZE: usize = 50 * 1024 * 1024;
    if data.len() > MAX_SIZE {
        return Err(AppError::BadRequest(
            "File too large. Use presigned upload for files over 50MB.".to_string(),
        ));
    }

    let evidence = services
        .evidence
        .upload_file(org_id, evidence_id, &filename, &content_type, data)
        .await?;

    Ok(Json(evidence))
}

// ==================== Helpers ====================

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}
