use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    CreateTask, CreateTaskComment, ListTasksQuery, Task, TaskComment, TaskCommentWithUser,
    TaskRecurrenceHistory, TaskStats, TaskWithAssignee, UpdateTask,
};
use chrono::{DateTime, Utc};
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
pub struct ListTasksParams {
    pub status: Option<String>,
    pub task_type: Option<String>,
    pub priority: Option<String>,
    pub assignee_id: Option<Uuid>,
    pub related_entity_type: Option<String>,
    pub related_entity_id: Option<Uuid>,
    pub search: Option<String>,
    pub overdue_only: Option<bool>,
    pub recurring_only: Option<bool>,
    pub exclude_recurring_templates: Option<bool>,
    pub parent_task_id: Option<Uuid>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl From<ListTasksParams> for ListTasksQuery {
    fn from(params: ListTasksParams) -> Self {
        ListTasksQuery {
            status: params.status,
            task_type: params.task_type,
            priority: params.priority,
            assignee_id: params.assignee_id,
            related_entity_type: params.related_entity_type,
            related_entity_id: params.related_entity_id,
            search: params.search,
            overdue_only: params.overdue_only,
            recurring_only: params.recurring_only,
            exclude_recurring_templates: params.exclude_recurring_templates,
            parent_task_id: params.parent_task_id,
            limit: params.limit,
            offset: params.offset,
        }
    }
}

// ==================== Task CRUD ====================

pub async fn list_tasks(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(params): Query<ListTasksParams>,
) -> AppResult<Json<Vec<TaskWithAssignee>>> {
    let org_id = get_org_id(&user)?;
    let tasks = services.task.list_tasks(org_id, params.into()).await?;
    Ok(Json(tasks))
}

pub async fn get_task(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<TaskWithAssignee>> {
    let org_id = get_org_id(&user)?;
    let task = services.task.get_task(org_id, id).await?;
    Ok(Json(task))
}

pub async fn create_task(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<CreateTask>,
) -> AppResult<Json<Task>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user).ok();
    let task = services.task.create_task(org_id, user_id, input).await?;
    Ok(Json(task))
}

pub async fn update_task(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateTask>,
) -> AppResult<Json<Task>> {
    let org_id = get_org_id(&user)?;
    let task = services.task.update_task(org_id, id, input).await?;
    Ok(Json(task))
}

pub async fn delete_task(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<()>> {
    let org_id = get_org_id(&user)?;
    services.task.delete_task(org_id, id).await?;
    Ok(Json(()))
}

pub async fn complete_task(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Task>> {
    let org_id = get_org_id(&user)?;
    let task = services.task.complete_task(org_id, id).await?;
    Ok(Json(task))
}

// ==================== Task Statistics ====================

pub async fn get_task_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<TaskStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.task.get_stats(org_id).await?;
    Ok(Json(stats))
}

// ==================== My Tasks ====================

pub async fn get_my_tasks(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<Task>>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let tasks = services.task.get_my_tasks(org_id, user_id).await?;
    Ok(Json(tasks))
}

pub async fn get_overdue_tasks(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<TaskWithAssignee>>> {
    let org_id = get_org_id(&user)?;
    let tasks = services.task.get_overdue_tasks(org_id).await?;
    Ok(Json(tasks))
}

// ==================== Task Comments ====================

pub async fn list_comments(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<Vec<TaskCommentWithUser>>> {
    let org_id = get_org_id(&user)?;
    let comments = services.task.list_comments(org_id, task_id).await?;
    Ok(Json(comments))
}

pub async fn add_comment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
    Json(input): Json<CreateTaskComment>,
) -> AppResult<Json<TaskComment>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let comment = services.task.add_comment(org_id, task_id, user_id, input).await?;
    Ok(Json(comment))
}

// ==================== Recurring Tasks ====================

/// GET /api/v1/tasks/recurring
/// Get all recurring task templates
pub async fn list_recurring_tasks(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<Task>>> {
    let org_id = get_org_id(&user)?;
    let tasks = services.task.get_recurring_tasks(org_id).await?;
    Ok(Json(tasks))
}

/// POST /api/v1/tasks/recurring/process
/// Process recurring tasks and create new occurrences
pub async fn process_recurring_tasks(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;
    let count = services.task.process_recurring_tasks(org_id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "occurrences_created": count
    })))
}

/// GET /api/v1/tasks/:id/occurrences
/// Get task occurrences for a recurring task template
pub async fn get_task_occurrences(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<Vec<TaskWithAssignee>>> {
    let org_id = get_org_id(&user)?;
    let occurrences = services.task.get_task_occurrences(org_id, task_id).await?;
    Ok(Json(occurrences))
}

/// GET /api/v1/tasks/:id/recurrence-history
/// Get recurrence history for a recurring task template
pub async fn get_recurrence_history(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<Vec<TaskRecurrenceHistory>>> {
    let org_id = get_org_id(&user)?;
    let history = services.task.get_recurrence_history(org_id, task_id).await?;
    Ok(Json(history))
}

#[derive(Debug, Deserialize)]
pub struct SkipOccurrenceInput {
    pub reason: Option<String>,
}

/// POST /api/v1/tasks/:id/skip
/// Skip the next occurrence of a recurring task
pub async fn skip_next_occurrence(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
    Json(input): Json<SkipOccurrenceInput>,
) -> AppResult<Json<Task>> {
    let org_id = get_org_id(&user)?;
    let task = services.task.skip_next_occurrence(org_id, task_id, input.reason).await?;
    Ok(Json(task))
}

/// POST /api/v1/tasks/:id/pause
/// Pause a recurring task
pub async fn pause_recurring_task(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
) -> AppResult<Json<Task>> {
    let org_id = get_org_id(&user)?;
    let task = services.task.pause_recurring_task(org_id, task_id).await?;
    Ok(Json(task))
}

#[derive(Debug, Deserialize)]
pub struct ResumeTaskInput {
    pub resume_from: Option<DateTime<Utc>>,
}

/// POST /api/v1/tasks/:id/resume
/// Resume a paused recurring task
pub async fn resume_recurring_task(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(task_id): Path<Uuid>,
    Json(input): Json<ResumeTaskInput>,
) -> AppResult<Json<Task>> {
    let org_id = get_org_id(&user)?;
    let task = services.task.resume_recurring_task(org_id, task_id, input.resume_from).await?;
    Ok(Json(task))
}
