use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::services::access_review::{
    AccessRemovalLog, AccessReviewCampaign, AccessReviewItem, AccessReviewStats,
    BulkReviewDecision, CampaignWithStats, CreateCampaign, CreateReviewItem, ListCampaignsQuery,
    ListItemsQuery, RequestRemoval, ReviewDecision, UpdateCampaign,
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
pub struct ListCampaignsParams {
    pub status: Option<String>,
    pub integration_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListCampaignsParams> for ListCampaignsQuery {
    fn from(params: ListCampaignsParams) -> Self {
        ListCampaignsQuery {
            status: params.status,
            integration_type: params.integration_type,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ListItemsParams {
    pub review_status: Option<String>,
    pub risk_level: Option<String>,
    pub is_admin: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListItemsParams> for ListItemsQuery {
    fn from(params: ListItemsParams) -> Self {
        ListItemsQuery {
            review_status: params.review_status,
            risk_level: params.risk_level,
            is_admin: params.is_admin,
            search: params.search,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

// ==================== Campaigns ====================

pub async fn list_campaigns(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListCampaignsParams>,
) -> AppResult<Json<Vec<CampaignWithStats>>> {
    let org_id = get_org_id(&user)?;
    let campaigns = services
        .access_review
        .list_campaigns(org_id, params.into())
        .await?;
    Ok(Json(campaigns))
}

pub async fn get_campaign(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<CampaignWithStats>> {
    let org_id = get_org_id(&user)?;
    let campaign = services.access_review.get_campaign(org_id, id).await?;
    Ok(Json(campaign))
}

pub async fn create_campaign(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateCampaign>,
) -> AppResult<Json<AccessReviewCampaign>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let campaign = services
        .access_review
        .create_campaign(org_id, Some(user_id), input)
        .await?;
    Ok(Json(campaign))
}

pub async fn update_campaign(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateCampaign>,
) -> AppResult<Json<AccessReviewCampaign>> {
    let org_id = get_org_id(&user)?;
    let campaign = services
        .access_review
        .update_campaign(org_id, id, input)
        .await?;
    Ok(Json(campaign))
}

pub async fn delete_campaign(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services.access_review.delete_campaign(org_id, id).await?;
    Ok(Json(()))
}

// ==================== Review Items ====================

#[derive(Debug, Deserialize)]
pub struct CampaignItemPath {
    pub campaign_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct ItemPath {
    pub campaign_id: Uuid,
    pub item_id: Uuid,
}

pub async fn list_items(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<CampaignItemPath>,
    Query(params): Query<ListItemsParams>,
) -> AppResult<Json<Vec<AccessReviewItem>>> {
    let org_id = get_org_id(&user)?;
    let items = services
        .access_review
        .list_items(org_id, path.campaign_id, params.into())
        .await?;
    Ok(Json(items))
}

pub async fn get_item(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<ItemPath>,
) -> AppResult<Json<AccessReviewItem>> {
    let org_id = get_org_id(&user)?;
    let item = services
        .access_review
        .get_item(org_id, path.campaign_id, path.item_id)
        .await?;
    Ok(Json(item))
}

pub async fn add_items(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<CampaignItemPath>,
    Json(items): Json<Vec<CreateReviewItem>>,
) -> AppResult<Json<Vec<AccessReviewItem>>> {
    let org_id = get_org_id(&user)?;
    let created = services
        .access_review
        .add_items(org_id, path.campaign_id, items)
        .await?;
    Ok(Json(created))
}

pub async fn review_item(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<ItemPath>,
    Json(decision): Json<ReviewDecision>,
) -> AppResult<Json<AccessReviewItem>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let item = services
        .access_review
        .review_item(org_id, path.campaign_id, path.item_id, user_id, decision)
        .await?;
    Ok(Json(item))
}

pub async fn bulk_review(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<CampaignItemPath>,
    Json(decision): Json<BulkReviewDecision>,
) -> AppResult<Json<i64>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let count = services
        .access_review
        .bulk_review(org_id, path.campaign_id, user_id, decision)
        .await?;
    Ok(Json(count))
}

// ==================== Access Removal ====================

pub async fn request_removal(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<ItemPath>,
    Json(input): Json<RequestRemoval>,
) -> AppResult<Json<AccessRemovalLog>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let log = services
        .access_review
        .request_removal(org_id, path.campaign_id, path.item_id, user_id, input)
        .await?;
    Ok(Json(log))
}

#[derive(Debug, Deserialize)]
pub struct CompleteRemovalInput {
    pub error_message: Option<String>,
}

pub async fn complete_removal(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(log_id): Path<Uuid>,
    Json(input): Json<CompleteRemovalInput>,
) -> AppResult<Json<AccessRemovalLog>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let log = services
        .access_review
        .complete_removal(org_id, log_id, user_id, input.error_message)
        .await?;
    Ok(Json(log))
}

pub async fn get_removal_logs(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(campaign_id): Path<Uuid>,
) -> AppResult<Json<Vec<AccessRemovalLog>>> {
    let org_id = get_org_id(&user)?;
    let logs = services
        .access_review
        .get_removal_logs(org_id, campaign_id)
        .await?;
    Ok(Json(logs))
}

// ==================== Sync ====================

pub async fn sync_from_integration(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(campaign_id): Path<Uuid>,
) -> AppResult<Json<i64>> {
    let org_id = get_org_id(&user)?;
    let count = services
        .access_review
        .sync_from_integration(org_id, campaign_id, &services.integration)
        .await?;
    Ok(Json(count))
}

// ==================== Statistics ====================

pub async fn get_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<AccessReviewStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.access_review.get_stats(org_id).await?;
    Ok(Json(stats))
}

// ==================== Certification Reports ====================

/// Get certification report data as JSON
pub async fn get_certification_report(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(campaign_id): Path<Uuid>,
) -> AppResult<Json<crate::services::reports::AccessReviewCertificationReport>> {
    let org_id = get_org_id(&user)?;
    let report = services
        .reports
        .generate_access_review_certification(org_id, campaign_id)
        .await?;
    Ok(Json(report))
}

/// Download certification report as CSV
pub async fn download_certification_csv(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(campaign_id): Path<Uuid>,
) -> AppResult<axum::response::Response> {
    let org_id = get_org_id(&user)?;

    let csv = services
        .reports
        .generate_access_review_csv(org_id, campaign_id)
        .await?;

    let filename = format!("access-review-certification-{}.csv", campaign_id);

    Ok(axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header(axum::http::header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(
            axum::http::header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"{}\"", filename),
        )
        .body(axum::body::Body::from(csv))
        .unwrap())
}
