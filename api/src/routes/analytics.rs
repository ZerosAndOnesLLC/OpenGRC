use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::services::analytics::*;
use crate::services::AppServices;
use crate::utils::AppError;

// Helper functions for parsing user IDs
fn get_org_id(user: &AuthUser) -> Result<Uuid, AppError> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}

fn get_user_id(user: &AuthUser) -> Result<Uuid, AppError> {
    Uuid::parse_str(&user.id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))
}

// ============================================================================
// Query Parameters
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct TrendQuery {
    pub days: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct DashboardQuery {
    pub dashboard_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SnapshotQuery {
    pub snapshot_type: Option<String>,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct MessageResponse {
    pub message: String,
    pub id: Option<Uuid>,
}

// ============================================================================
// Compliance Trends Endpoints
// ============================================================================

/// Capture a new compliance snapshot
pub async fn capture_snapshot(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<SnapshotQuery>,
) -> Result<Json<MessageResponse>, AppError> {
    let org_id = get_org_id(&user)?;
    let snapshot_type = query.snapshot_type.as_deref().unwrap_or("daily");

    let id = services.analytics
        .capture_snapshot(org_id, snapshot_type)
        .await?;

    Ok(Json(MessageResponse {
        message: "Snapshot captured successfully".to_string(),
        id: Some(id),
    }))
}

/// Get current compliance snapshot
pub async fn get_current_snapshot(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Option<ComplianceSnapshot>>>, AppError> {
    let org_id = get_org_id(&user)?;

    let snapshot = services.analytics
        .get_current_snapshot(org_id)
        .await?;

    Ok(Json(ApiResponse { data: snapshot }))
}

/// Get compliance trends
pub async fn get_compliance_trends(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<TrendQuery>,
) -> Result<Json<ApiResponse<ComplianceTrendResponse>>, AppError> {
    let org_id = get_org_id(&user)?;
    let days = query.days.unwrap_or(30);

    let trends = services.analytics
        .get_compliance_trends(org_id, days)
        .await?;

    Ok(Json(ApiResponse { data: trends }))
}

// ============================================================================
// Risk Predictions Endpoints
// ============================================================================

/// Get all risk predictions
pub async fn get_risk_predictions(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Vec<RiskPredictionWithDetails>>>, AppError> {
    let org_id = get_org_id(&user)?;

    let predictions = services.analytics
        .get_risk_predictions(org_id)
        .await?;

    Ok(Json(ApiResponse { data: predictions }))
}

/// Get prediction for a specific risk
pub async fn get_risk_prediction(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(risk_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Option<RiskPrediction>>>, AppError> {
    let org_id = get_org_id(&user)?;

    let prediction = services.analytics
        .get_risk_prediction(org_id, risk_id)
        .await?;

    Ok(Json(ApiResponse { data: prediction }))
}

/// Recompute prediction for a risk
pub async fn recompute_risk_prediction(
    State(services): State<Arc<AppServices>>,
    Path(risk_id): Path<Uuid>,
) -> Result<Json<MessageResponse>, AppError> {
    let id = services.analytics
        .recompute_risk_prediction(risk_id)
        .await?;

    Ok(Json(MessageResponse {
        message: "Prediction recomputed".to_string(),
        id,
    }))
}

/// Get risk prediction summary
pub async fn get_risk_prediction_summary(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<RiskPredictionSummary>>, AppError> {
    let org_id = get_org_id(&user)?;

    let summary = services.analytics
        .get_risk_prediction_summary(org_id)
        .await?;

    Ok(Json(ApiResponse { data: summary }))
}

/// Get prediction factors
pub async fn get_prediction_factors(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Vec<RiskPredictionFactor>>>, AppError> {
    let org_id = get_org_id(&user)?;

    let factors = services.analytics
        .get_prediction_factors(org_id)
        .await?;

    Ok(Json(ApiResponse { data: factors }))
}

// ============================================================================
// Benchmarks Endpoints
// ============================================================================

/// Get available benchmarks
pub async fn get_available_benchmarks(
    State(services): State<Arc<AppServices>>,
) -> Result<Json<ApiResponse<Vec<IndustryBenchmark>>>, AppError> {
    let benchmarks = services.analytics
        .get_available_benchmarks()
        .await?;

    Ok(Json(ApiResponse { data: benchmarks }))
}

/// Compare organization to a benchmark
pub async fn compare_to_benchmark(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(benchmark_id): Path<Uuid>,
) -> Result<Json<ApiResponse<BenchmarkComparisonWithDetails>>, AppError> {
    let org_id = get_org_id(&user)?;

    let comparison = services.analytics
        .compare_to_benchmark(org_id, benchmark_id)
        .await?;

    Ok(Json(ApiResponse { data: comparison }))
}

/// Get latest benchmark comparison
pub async fn get_latest_benchmark_comparison(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Option<BenchmarkComparisonWithDetails>>>, AppError> {
    let org_id = get_org_id(&user)?;

    let comparison = services.analytics
        .get_latest_benchmark_comparison(org_id)
        .await?;

    Ok(Json(ApiResponse { data: comparison }))
}

// ============================================================================
// Custom Reports Endpoints
// ============================================================================

/// List saved reports
pub async fn list_saved_reports(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Vec<SavedReport>>>, AppError> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;

    let reports = services.analytics
        .list_saved_reports(org_id, user_id)
        .await?;

    Ok(Json(ApiResponse { data: reports }))
}

/// Get a saved report
pub async fn get_saved_report(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(report_id): Path<Uuid>,
) -> Result<Json<ApiResponse<SavedReport>>, AppError> {
    let org_id = get_org_id(&user)?;

    let report = services.analytics
        .get_saved_report(org_id, report_id)
        .await?;

    Ok(Json(ApiResponse { data: report }))
}

/// Create a saved report
pub async fn create_saved_report(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateSavedReport>,
) -> Result<(StatusCode, Json<ApiResponse<SavedReport>>), AppError> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;

    let report = services.analytics
        .create_saved_report(org_id, user_id, input)
        .await?;

    Ok((StatusCode::CREATED, Json(ApiResponse { data: report })))
}

/// Update a saved report
pub async fn update_saved_report(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(report_id): Path<Uuid>,
    Json(input): Json<UpdateSavedReport>,
) -> Result<Json<ApiResponse<SavedReport>>, AppError> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;

    let report = services.analytics
        .update_saved_report(org_id, report_id, user_id, input)
        .await?;

    Ok(Json(ApiResponse { data: report }))
}

/// Delete a saved report
pub async fn delete_saved_report(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(report_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let org_id = get_org_id(&user)?;

    services.analytics
        .delete_saved_report(org_id, report_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get report templates
pub async fn get_report_templates(
    State(services): State<Arc<AppServices>>,
) -> Result<Json<ApiResponse<Vec<ReportTemplate>>>, AppError> {
    let templates = services.analytics
        .get_report_templates()
        .await?;

    Ok(Json(ApiResponse { data: templates }))
}

// ============================================================================
// Executive Dashboard Endpoints
// ============================================================================

/// Get executive metrics
pub async fn get_executive_metrics(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<Vec<ExecutiveMetric>>>, AppError> {
    let org_id = get_org_id(&user)?;

    let metrics = services.analytics
        .get_executive_metrics(org_id)
        .await?;

    Ok(Json(ApiResponse { data: metrics }))
}

/// Get dashboard widgets
pub async fn get_dashboard_widgets(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<ApiResponse<Vec<DashboardWidget>>>, AppError> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let dashboard_type = query.dashboard_type.as_deref().unwrap_or("executive");

    let widgets = services.analytics
        .get_dashboard_widgets(org_id, Some(user_id), dashboard_type)
        .await?;

    Ok(Json(ApiResponse { data: widgets }))
}

/// Create dashboard widget
pub async fn create_dashboard_widget(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateDashboardWidget>,
) -> Result<(StatusCode, Json<ApiResponse<DashboardWidget>>), AppError> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;

    let widget = services.analytics
        .create_dashboard_widget(org_id, Some(user_id), input)
        .await?;

    Ok((StatusCode::CREATED, Json(ApiResponse { data: widget })))
}

/// Update dashboard widget
pub async fn update_dashboard_widget(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(widget_id): Path<Uuid>,
    Json(input): Json<UpdateDashboardWidget>,
) -> Result<Json<ApiResponse<DashboardWidget>>, AppError> {
    let org_id = get_org_id(&user)?;

    let widget = services.analytics
        .update_dashboard_widget(org_id, widget_id, input)
        .await?;

    Ok(Json(ApiResponse { data: widget }))
}

/// Delete dashboard widget
pub async fn delete_dashboard_widget(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(widget_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let org_id = get_org_id(&user)?;

    services.analytics
        .delete_dashboard_widget(org_id, widget_id)
        .await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get full executive dashboard
pub async fn get_executive_dashboard(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> Result<Json<ApiResponse<ExecutiveDashboard>>, AppError> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;

    let dashboard = services.analytics
        .get_executive_dashboard(org_id, Some(user_id))
        .await?;

    Ok(Json(ApiResponse { data: dashboard }))
}
