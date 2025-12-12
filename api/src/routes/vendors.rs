use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    CreateVendor, CreateVendorAssessment, ListVendorsQuery, UpdateVendor, Vendor,
    VendorAssessment, VendorStats, VendorWithAssessment,
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
pub struct ListVendorsParams {
    pub status: Option<String>,
    pub category: Option<String>,
    pub criticality: Option<String>,
    pub owner_id: Option<Uuid>,
    pub search: Option<String>,
    pub contract_expiring: Option<bool>,
    pub needs_assessment: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListVendorsParams> for ListVendorsQuery {
    fn from(params: ListVendorsParams) -> Self {
        ListVendorsQuery {
            status: params.status,
            category: params.category,
            criticality: params.criticality,
            owner_id: params.owner_id,
            search: params.search,
            contract_expiring: params.contract_expiring,
            needs_assessment: params.needs_assessment,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

// ==================== Vendor CRUD ====================

pub async fn list_vendors(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListVendorsParams>,
) -> AppResult<Json<Vec<VendorWithAssessment>>> {
    let org_id = get_org_id(&user)?;
    let vendors = services.vendor.list_vendors(org_id, params.into()).await?;
    Ok(Json(vendors))
}

pub async fn get_vendor(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<VendorWithAssessment>> {
    let org_id = get_org_id(&user)?;
    let vendor = services.vendor.get_vendor(org_id, id).await?;
    Ok(Json(vendor))
}

pub async fn create_vendor(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateVendor>,
) -> AppResult<Json<Vendor>> {
    let org_id = get_org_id(&user)?;
    let vendor = services.vendor.create_vendor(org_id, input).await?;
    Ok(Json(vendor))
}

pub async fn update_vendor(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateVendor>,
) -> AppResult<Json<Vendor>> {
    let org_id = get_org_id(&user)?;
    let vendor = services.vendor.update_vendor(org_id, id, input).await?;
    Ok(Json(vendor))
}

pub async fn delete_vendor(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services.vendor.delete_vendor(org_id, id).await?;
    Ok(Json(()))
}

// ==================== Vendor Statistics ====================

pub async fn get_vendor_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<VendorStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.vendor.get_stats(org_id).await?;
    Ok(Json(stats))
}

// ==================== Assessments ====================

pub async fn create_assessment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<CreateVendorAssessment>,
) -> AppResult<Json<VendorAssessment>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let assessment = services
        .vendor
        .create_assessment(org_id, id, Some(user_id), input)
        .await?;
    Ok(Json(assessment))
}

pub async fn get_assessments(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<VendorAssessment>>> {
    let org_id = get_org_id(&user)?;
    let assessments = services.vendor.get_assessments(org_id, id).await?;
    Ok(Json(assessments))
}
