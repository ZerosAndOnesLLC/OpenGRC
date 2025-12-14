use axum::{
    extract::{Extension, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}

/// GET /api/v1/reports/:report_type/csv
/// Generates a CSV report for the specified type
pub async fn generate_csv_report(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(report_type): Path<String>,
) -> impl IntoResponse {
    let org_id = match get_org_id(&user) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                [(header::CONTENT_TYPE, "text/plain")],
                e.to_string(),
            )
                .into_response()
        }
    };

    let result = match report_type.as_str() {
        "controls" => services.reports.generate_controls_csv(org_id).await,
        "risks" => services.reports.generate_risks_csv(org_id).await,
        "evidence" => services.reports.generate_evidence_csv(org_id).await,
        "policies" => services.reports.generate_policies_csv(org_id).await,
        "vendors" => services.reports.generate_vendors_csv(org_id).await,
        "compliance-posture" => services.reports.generate_compliance_posture_csv(org_id).await,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                [(header::CONTENT_TYPE, "text/plain")],
                format!("Unknown report type: {}", report_type),
            )
                .into_response()
        }
    };

    match result {
        Ok(csv) => {
            let filename = format!("opengrc-{}-{}.csv", report_type, chrono::Utc::now().format("%Y%m%d"));
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
                    (
                        header::CONTENT_DISPOSITION,
                        &format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                csv,
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("Failed to generate report: {}", e),
        )
            .into_response(),
    }
}

/// GET /api/v1/reports/:report_type/pdf
/// Generates a PDF report for the specified type
pub async fn generate_pdf_report(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(report_type): Path<String>,
) -> impl IntoResponse {
    let org_id = match get_org_id(&user) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                [(header::CONTENT_TYPE, "text/plain")],
                e.to_string().into_bytes(),
            )
                .into_response()
        }
    };

    let result = match report_type.as_str() {
        "controls" => services.pdf.generate_controls_pdf(org_id).await,
        "risks" => services.pdf.generate_risks_pdf(org_id).await,
        "evidence" => services.pdf.generate_evidence_pdf(org_id).await,
        "policies" => services.pdf.generate_policies_pdf(org_id).await,
        "vendors" => services.pdf.generate_vendors_pdf(org_id).await,
        "compliance-posture" => services.pdf.generate_compliance_posture_pdf(org_id).await,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                [(header::CONTENT_TYPE, "text/plain")],
                format!("Unknown report type: {}", report_type).into_bytes(),
            )
                .into_response()
        }
    };

    match result {
        Ok(pdf_bytes) => {
            let filename = format!("opengrc-{}-{}.pdf", report_type, chrono::Utc::now().format("%Y%m%d"));
            (
                StatusCode::OK,
                [
                    (header::CONTENT_TYPE, "application/pdf"),
                    (
                        header::CONTENT_DISPOSITION,
                        &format!("attachment; filename=\"{}\"", filename),
                    ),
                ],
                pdf_bytes,
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            [(header::CONTENT_TYPE, "text/plain")],
            format!("Failed to generate PDF report: {}", e).into_bytes(),
        )
            .into_response(),
    }
}

/// GET /api/v1/reports/types
/// Lists available report types
pub async fn list_report_types() -> impl IntoResponse {
    let types = serde_json::json!({
        "reports": [
            {
                "id": "controls",
                "name": "Control Health",
                "description": "Control testing results and implementation status"
            },
            {
                "id": "risks",
                "name": "Risk Register",
                "description": "Complete risk register with scores and mitigation status"
            },
            {
                "id": "evidence",
                "name": "Evidence Summary",
                "description": "Evidence collection and control coverage"
            },
            {
                "id": "policies",
                "name": "Policy Acknowledgments",
                "description": "Policy review and acknowledgment status"
            },
            {
                "id": "vendors",
                "name": "Vendor Risk",
                "description": "Vendor risk assessment and contract status"
            },
            {
                "id": "compliance-posture",
                "name": "Compliance Posture",
                "description": "Framework coverage and compliance status"
            }
        ]
    });

    (StatusCode::OK, axum::Json(types))
}
