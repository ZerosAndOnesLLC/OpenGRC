use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    CreatePolicy, ListPoliciesQuery, Policy, PolicyAcknowledgment, PolicyStats,
    PolicyVersion, PolicyWithStats, UpdatePolicy,
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

// ==================== Policy CRUD ====================

pub async fn list_policies(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<ListPoliciesQuery>,
) -> AppResult<Json<Vec<PolicyWithStats>>> {
    let org_id = get_org_id(&user)?;
    let policies = services.policy.list_policies(org_id, query).await?;
    Ok(Json(policies))
}

pub async fn get_policy(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PolicyWithStats>> {
    let org_id = get_org_id(&user)?;
    let policy = services.policy.get_policy(org_id, id).await?;
    Ok(Json(policy))
}

pub async fn create_policy(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreatePolicy>,
) -> AppResult<Json<Policy>> {
    let org_id = get_org_id(&user)?;
    let policy = services.policy.create_policy(org_id, input).await?;
    Ok(Json(policy))
}

pub async fn update_policy(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdatePolicy>,
) -> AppResult<Json<Policy>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let policy = services
        .policy
        .update_policy(org_id, id, Some(user_id), input)
        .await?;
    Ok(Json(policy))
}

pub async fn delete_policy(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services.policy.delete_policy(org_id, id).await?;
    Ok(Json(()))
}

// ==================== Policy Statistics ====================

pub async fn get_policy_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<PolicyStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.policy.get_stats(org_id).await?;
    Ok(Json(stats))
}

// ==================== Policy Versions ====================

pub async fn get_policy_versions(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<PolicyVersion>>> {
    let org_id = get_org_id(&user)?;
    let versions = services.policy.get_versions(org_id, id).await?;
    Ok(Json(versions))
}

// ==================== Policy Acknowledgments ====================

pub async fn acknowledge_policy(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<PolicyAcknowledgment>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    // TODO: Extract IP from request headers in production
    let ack = services
        .policy
        .acknowledge_policy(org_id, id, user_id, None)
        .await?;
    Ok(Json(ack))
}

pub async fn get_policy_acknowledgments(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Vec<PolicyAcknowledgment>>> {
    let org_id = get_org_id(&user)?;
    let acknowledgments = services.policy.get_acknowledgments(org_id, id).await?;
    Ok(Json(acknowledgments))
}

/// GET /api/v1/policies/pending
/// Get all policies pending acknowledgment for the current user
pub async fn get_pending_policies(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<PolicyWithStats>>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let policies = services.policy.get_pending_policies(org_id, user_id).await?;
    Ok(Json(policies))
}
