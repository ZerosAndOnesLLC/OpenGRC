use axum::{
    extract::{Extension, Path, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::services::soc2_parser::{Soc2ParseResult, Soc2ParsedReport, Soc2ReportSummary};
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}

#[derive(Debug, serde::Deserialize)]
pub struct DocumentPath {
    pub vendor_id: Uuid,
    pub document_id: Uuid,
}

/// Parse a vendor document (SOC 2 report PDF)
pub async fn parse_document(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<DocumentPath>,
) -> AppResult<Json<Soc2ParseResult>> {
    let org_id = get_org_id(&user)?;

    // Get the document
    let document = services
        .vendor
        .get_document(org_id, path.vendor_id, path.document_id)
        .await?;

    // Get the file path
    let file_path = document.file_path.ok_or_else(|| {
        AppError::BadRequest("Document has no file uploaded".to_string())
    })?;

    // Download the file
    let (data, content_type) = services.storage.download_evidence(&file_path).await?;

    // Verify it's a PDF
    if !content_type.contains("pdf") && !file_path.ends_with(".pdf") {
        return Err(AppError::BadRequest(
            "Document must be a PDF file to parse".to_string(),
        ));
    }

    // Parse the PDF
    let result = services
        .soc2_parser
        .parse_vendor_document(org_id, path.vendor_id, path.document_id, &data)
        .await?;

    Ok(Json(result))
}

/// Get parsed report for a document
pub async fn get_document_report(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<DocumentPath>,
) -> AppResult<Json<Option<Soc2ParseResult>>> {
    let org_id = get_org_id(&user)?;

    // Verify document exists and belongs to vendor/org
    services
        .vendor
        .get_document(org_id, path.vendor_id, path.document_id)
        .await?;

    let result = services
        .soc2_parser
        .get_parsed_report_by_document(org_id, path.document_id)
        .await?;

    Ok(Json(result))
}

/// Get a parsed report by ID
pub async fn get_parsed_report(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Soc2ParseResult>> {
    let org_id = get_org_id(&user)?;
    let result = services.soc2_parser.get_parsed_report(org_id, id).await?;
    Ok(Json(result))
}

/// List all parsed reports for a vendor
pub async fn list_vendor_reports(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(vendor_id): Path<Uuid>,
) -> AppResult<Json<Vec<Soc2ParsedReport>>> {
    let org_id = get_org_id(&user)?;
    let reports = services.soc2_parser.list_vendor_reports(org_id, vendor_id).await?;
    Ok(Json(reports))
}

/// Get all parsed report summaries for the organization
pub async fn get_report_summaries(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<Soc2ReportSummary>>> {
    let org_id = get_org_id(&user)?;
    let summaries = services.soc2_parser.get_report_summaries(org_id).await?;
    Ok(Json(summaries))
}

/// Delete a parsed report
pub async fn delete_parsed_report(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services.soc2_parser.delete_parsed_report(org_id, id).await?;
    Ok(Json(()))
}
