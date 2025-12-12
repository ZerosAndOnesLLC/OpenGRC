use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    Asset, AssetControlMapping, AssetStats, AssetWithControls, CreateAsset, ListAssetsQuery,
    UpdateAsset,
};
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))
}

// ==================== Query Params ====================

#[derive(Debug, Deserialize)]
pub struct ListAssetsParams {
    pub asset_type: Option<String>,
    pub category: Option<String>,
    pub classification: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListAssetsParams> for ListAssetsQuery {
    fn from(params: ListAssetsParams) -> Self {
        ListAssetsQuery {
            asset_type: params.asset_type,
            category: params.category,
            classification: params.classification,
            status: params.status,
            owner_id: params.owner_id,
            search: params.search,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LinkControlsRequest {
    pub control_ids: Vec<Uuid>,
}

// ==================== Asset CRUD ====================

pub async fn list_assets(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListAssetsParams>,
) -> AppResult<Json<Vec<AssetWithControls>>> {
    let org_id = get_org_id(&user)?;
    let assets = services.asset.list_assets(org_id, params.into()).await?;
    Ok(Json(assets))
}

pub async fn get_asset(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<AssetWithControls>> {
    let org_id = get_org_id(&user)?;
    let asset = services.asset.get_asset(org_id, id).await?;
    Ok(Json(asset))
}

pub async fn create_asset(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateAsset>,
) -> AppResult<Json<Asset>> {
    let org_id = get_org_id(&user)?;
    let asset = services.asset.create_asset(org_id, input).await?;
    Ok(Json(asset))
}

pub async fn update_asset(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateAsset>,
) -> AppResult<Json<Asset>> {
    let org_id = get_org_id(&user)?;
    let asset = services.asset.update_asset(org_id, id, input).await?;
    Ok(Json(asset))
}

pub async fn delete_asset(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services.asset.delete_asset(org_id, id).await?;
    Ok(Json(()))
}

// ==================== Asset Statistics ====================

pub async fn get_asset_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<AssetStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.asset.get_stats(org_id).await?;
    Ok(Json(stats))
}

// ==================== Control Mappings ====================

pub async fn link_controls(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<LinkControlsRequest>,
) -> AppResult<Json<Vec<AssetControlMapping>>> {
    let org_id = get_org_id(&user)?;
    let mappings = services
        .asset
        .link_controls(org_id, id, input.control_ids)
        .await?;
    Ok(Json(mappings))
}

pub async fn unlink_controls(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<LinkControlsRequest>,
) -> AppResult<Json<i64>> {
    let org_id = get_org_id(&user)?;
    let deleted = services
        .asset
        .unlink_controls(org_id, id, input.control_ids)
        .await?;
    Ok(Json(deleted))
}
