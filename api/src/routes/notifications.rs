use axum::{
    extract::{Extension, Path, Query, State},
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::services::notification::Notification;
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

#[derive(Debug, Deserialize)]
pub struct ListNotificationsQuery {
    pub unread_only: Option<bool>,
    pub limit: Option<i64>,
}

/// GET /api/v1/notifications
/// Get all notifications for the current user
pub async fn list_notifications(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<ListNotificationsQuery>,
) -> AppResult<Json<Vec<Notification>>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let unread_only = query.unread_only.unwrap_or(false);
    let limit = query.limit.unwrap_or(50).min(100);

    let notifications = services
        .notification
        .get_user_notifications(org_id, user_id, unread_only, limit)
        .await?;

    Ok(Json(notifications))
}

/// GET /api/v1/notifications/count
/// Get unread notification count
pub async fn get_unread_count(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;

    let count = services.notification.get_unread_count(org_id, user_id).await?;

    Ok(Json(serde_json::json!({ "unread_count": count })))
}

/// PUT /api/v1/notifications/:id/read
/// Mark a notification as read
pub async fn mark_as_read(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;

    services.notification.mark_as_read(org_id, user_id, id).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// PUT /api/v1/notifications/read-all
/// Mark all notifications as read
pub async fn mark_all_as_read(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;

    services.notification.mark_all_as_read(org_id, user_id).await?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// POST /api/v1/notifications/process-task-reminders
/// Process and send task due date reminders
/// This endpoint is designed to be called by a scheduled job
pub async fn process_task_reminders(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;

    let count = services.notification.process_task_reminders(org_id).await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "reminders_sent": count
    })))
}
