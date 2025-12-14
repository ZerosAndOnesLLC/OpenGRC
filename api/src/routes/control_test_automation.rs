use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::services::control_test_automation::{
    ControlMonitoringStatus, ControlMonitoringSummary, ControlTestAlert, ControlTestRemediation,
    ControlTestRun, ControlTestTemplate, CreateRemediation, ListAlertsQuery, ListTemplatesQuery,
    ListTestRunsQuery, UpdateAlertConfig,
};
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

// ==================== Query Params ====================

#[derive(Debug, Deserialize)]
pub struct ListTemplatesParams {
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub framework: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListTemplatesParams> for ListTemplatesQuery {
    fn from(params: ListTemplatesParams) -> Self {
        ListTemplatesQuery {
            category: params.category,
            subcategory: params.subcategory,
            framework: params.framework,
            search: params.search,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListTestRunsParams {
    pub control_id: Option<Uuid>,
    pub control_test_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListTestRunsParams> for ListTestRunsQuery {
    fn from(params: ListTestRunsParams) -> Self {
        ListTestRunsQuery {
            control_id: params.control_id,
            control_test_id: params.control_test_id,
            status: params.status,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListAlertsParams {
    pub control_test_id: Option<Uuid>,
    pub alert_type: Option<String>,
    pub unacknowledged_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListAlertsParams> for ListAlertsQuery {
    fn from(params: ListAlertsParams) -> Self {
        ListAlertsQuery {
            control_test_id: params.control_test_id,
            alert_type: params.alert_type,
            unacknowledged_only: params.unacknowledged_only,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListByHealthParams {
    pub health: Option<String>,
    pub limit: Option<i64>,
}

// ==================== Response Types ====================

#[derive(Debug, Serialize)]
pub struct TemplateCategory {
    pub category: String,
    pub count: i64,
}

#[derive(Debug, Serialize)]
pub struct AlertCountResponse {
    pub unacknowledged: i64,
}

#[derive(Debug, Deserialize)]
pub struct ResolveAlertRequest {
    pub resolution_notes: Option<String>,
}

// ==================== Template Routes ====================

/// GET /api/v1/control-testing/templates
pub async fn list_templates(
    State(services): State<Arc<AppServices>>,
    Query(params): Query<ListTemplatesParams>,
) -> AppResult<Json<Vec<ControlTestTemplate>>> {
    let templates = services
        .control_test_automation
        .list_templates(params.into())
        .await?;
    Ok(Json(templates))
}

/// GET /api/v1/control-testing/templates/categories
pub async fn get_template_categories(
    State(services): State<Arc<AppServices>>,
) -> AppResult<Json<Vec<TemplateCategory>>> {
    let categories = services
        .control_test_automation
        .get_template_categories()
        .await?;

    let result: Vec<TemplateCategory> = categories
        .into_iter()
        .map(|(category, count)| TemplateCategory { category, count })
        .collect();

    Ok(Json(result))
}

/// GET /api/v1/control-testing/templates/:id
pub async fn get_template(
    State(services): State<Arc<AppServices>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ControlTestTemplate>> {
    let template = services.control_test_automation.get_template(id).await?;
    Ok(Json(template))
}

// ==================== Alert Config Routes ====================

/// GET /api/v1/control-testing/tests/:test_id/alert-config
pub async fn get_alert_config(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(test_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let config = services
        .control_test_automation
        .get_alert_config(org_id, test_id)
        .await?;
    Ok(Json(serde_json::to_value(config).unwrap()))
}

/// PUT /api/v1/control-testing/tests/:test_id/alert-config
pub async fn update_alert_config(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(test_id): Path<Uuid>,
    Json(input): Json<UpdateAlertConfig>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let config = services
        .control_test_automation
        .update_alert_config(org_id, test_id, input)
        .await?;
    Ok(Json(serde_json::to_value(config).unwrap()))
}

// ==================== Remediation Routes ====================

/// GET /api/v1/control-testing/remediations
pub async fn list_global_remediations(
    State(services): State<Arc<AppServices>>,
) -> AppResult<Json<Vec<ControlTestRemediation>>> {
    let remediations = services
        .control_test_automation
        .list_remediations(None)
        .await?;
    Ok(Json(remediations))
}

/// GET /api/v1/control-testing/tests/:test_id/remediations
pub async fn list_test_remediations(
    State(services): State<Arc<AppServices>>,
    Path(test_id): Path<Uuid>,
) -> AppResult<Json<Vec<ControlTestRemediation>>> {
    let remediations = services
        .control_test_automation
        .list_remediations(Some(test_id))
        .await?;
    Ok(Json(remediations))
}

/// POST /api/v1/control-testing/remediations
pub async fn create_remediation(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateRemediation>,
) -> AppResult<Json<ControlTestRemediation>> {
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let remediation = services
        .control_test_automation
        .create_remediation(user_id, input)
        .await?;
    Ok(Json(remediation))
}

/// GET /api/v1/control-testing/remediations/:id
pub async fn get_remediation(
    State(services): State<Arc<AppServices>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<ControlTestRemediation>> {
    let remediation = services.control_test_automation.get_remediation(id).await?;
    Ok(Json(remediation))
}

// ==================== Monitoring Routes ====================

/// GET /api/v1/control-testing/monitoring/summary
pub async fn get_monitoring_summary(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<ControlMonitoringSummary>> {
    let org_id = get_org_id(&user)?;
    let summary = services
        .control_test_automation
        .get_monitoring_summary(org_id)
        .await?;
    Ok(Json(summary))
}

/// GET /api/v1/control-testing/monitoring/controls
pub async fn list_monitored_controls(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListByHealthParams>,
) -> AppResult<Json<Vec<ControlMonitoringStatus>>> {
    let org_id = get_org_id(&user)?;
    let statuses = services
        .control_test_automation
        .list_controls_by_health(org_id, params.health, params.limit.unwrap_or(50))
        .await?;
    Ok(Json(statuses))
}

/// GET /api/v1/control-testing/monitoring/controls/:control_id
pub async fn get_control_monitoring_status(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(control_id): Path<Uuid>,
) -> AppResult<Json<ControlMonitoringStatus>> {
    let org_id = get_org_id(&user)?;
    let status = services
        .control_test_automation
        .get_control_monitoring_status(org_id, control_id)
        .await?;
    Ok(Json(status))
}

/// PUT /api/v1/control-testing/monitoring/controls/:control_id/acknowledge
pub async fn acknowledge_control_alert(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(control_id): Path<Uuid>,
) -> AppResult<Json<ControlMonitoringStatus>> {
    let org_id = get_org_id(&user)?;
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let status = services
        .control_test_automation
        .acknowledge_alert(org_id, control_id, user_id)
        .await?;
    Ok(Json(status))
}

// ==================== Test Run Routes ====================

/// GET /api/v1/control-testing/runs
pub async fn list_test_runs(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListTestRunsParams>,
) -> AppResult<Json<Vec<ControlTestRun>>> {
    let org_id = get_org_id(&user)?;
    let runs = services
        .control_test_automation
        .list_test_runs(org_id, params.into())
        .await?;
    Ok(Json(runs))
}

/// GET /api/v1/control-testing/runs/:run_id
pub async fn get_test_run(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(run_id): Path<Uuid>,
) -> AppResult<Json<ControlTestRun>> {
    let org_id = get_org_id(&user)?;
    let run = services
        .control_test_automation
        .get_test_run(org_id, run_id)
        .await?;
    Ok(Json(run))
}

/// POST /api/v1/control-testing/tests/:test_id/trigger
pub async fn trigger_test(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(test_id): Path<Uuid>,
) -> AppResult<Json<ControlTestRun>> {
    let org_id = get_org_id(&user)?;
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let run = services
        .control_test_automation
        .trigger_test(org_id, test_id, user_id)
        .await?;
    Ok(Json(run))
}

// ==================== Alert Routes ====================

/// GET /api/v1/control-testing/alerts
pub async fn list_alerts(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListAlertsParams>,
) -> AppResult<Json<Vec<ControlTestAlert>>> {
    let org_id = get_org_id(&user)?;
    let alerts = services
        .control_test_automation
        .list_alerts(org_id, params.into())
        .await?;
    Ok(Json(alerts))
}

/// GET /api/v1/control-testing/alerts/count
pub async fn get_alert_count(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<AlertCountResponse>> {
    let org_id = get_org_id(&user)?;
    let count = services
        .control_test_automation
        .get_unacknowledged_alert_count(org_id)
        .await?;
    Ok(Json(AlertCountResponse { unacknowledged: count }))
}

/// PUT /api/v1/control-testing/alerts/:alert_id/acknowledge
pub async fn acknowledge_alert(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(alert_id): Path<Uuid>,
) -> AppResult<Json<ControlTestAlert>> {
    let org_id = get_org_id(&user)?;
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let alert = services
        .control_test_automation
        .acknowledge_test_alert(org_id, alert_id, user_id)
        .await?;
    Ok(Json(alert))
}

/// PUT /api/v1/control-testing/alerts/:alert_id/resolve
pub async fn resolve_alert(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(alert_id): Path<Uuid>,
    Json(input): Json<ResolveAlertRequest>,
) -> AppResult<Json<ControlTestAlert>> {
    let org_id = get_org_id(&user)?;
    let user_id = Uuid::parse_str(&user.id)
        .map_err(|_| AppError::BadRequest("Invalid user ID".to_string()))?;
    let alert = services
        .control_test_automation
        .resolve_alert(org_id, alert_id, user_id, input.resolution_notes)
        .await?;
    Ok(Json(alert))
}

// ==================== Helpers ====================

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}
