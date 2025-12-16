use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::collaboration::{
    CommentStats, CreateEntityComment, EntityCommentWithUser, ListCommentsQuery,
    NotificationPreferences, PresenceInfo, UpdateEntityComment, UpdateNotificationPreferences,
    UpdatePresence, UserSearchResult, NOTIFICATION_TYPES,
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

// =====================================================
// ENTITY COMMENTS
// =====================================================

#[derive(Debug, Deserialize)]
pub struct EntityPath {
    pub entity_type: String,
    pub entity_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct CommentPath {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub comment_id: Uuid,
}

/// List comments for an entity
pub async fn list_comments(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<EntityPath>,
    Query(query): Query<ListCommentsQuery>,
) -> AppResult<Json<Vec<EntityCommentWithUser>>> {
    let org_id = get_org_id(&user)?;
    let comments = services
        .collaboration
        .list_comments(org_id, &path.entity_type, path.entity_id, query)
        .await?;
    Ok(Json(comments))
}

/// Get a single comment
pub async fn get_comment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<CommentPath>,
) -> AppResult<Json<EntityCommentWithUser>> {
    let org_id = get_org_id(&user)?;
    let comment = services.collaboration.get_comment(org_id, path.comment_id).await?;
    Ok(Json(comment))
}

/// Create a new comment
pub async fn create_comment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<EntityPath>,
    Json(input): Json<CreateEntityComment>,
) -> AppResult<(StatusCode, Json<EntityCommentWithUser>)> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;

    let comment = services
        .collaboration
        .create_comment(org_id, user_id, &path.entity_type, path.entity_id, input)
        .await?;

    // TODO: Send mention notifications via NotificationService
    // This would require getting user info and entity title

    Ok((StatusCode::CREATED, Json(comment)))
}

/// Update a comment
pub async fn update_comment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<CommentPath>,
    Json(input): Json<UpdateEntityComment>,
) -> AppResult<Json<EntityCommentWithUser>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;

    let comment = services
        .collaboration
        .update_comment(org_id, user_id, path.comment_id, input)
        .await?;
    Ok(Json(comment))
}

/// Delete a comment (soft delete)
pub async fn delete_comment(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<CommentPath>,
) -> AppResult<StatusCode> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;

    services
        .collaboration
        .delete_comment(org_id, user_id, path.comment_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get comment statistics
pub async fn get_comment_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<CommentStats>> {
    let org_id = get_org_id(&user)?;
    let stats = services.collaboration.get_comment_stats(org_id).await?;
    Ok(Json(stats))
}

// =====================================================
// USER SEARCH (for @mentions)
// =====================================================

#[derive(Debug, Deserialize)]
pub struct UserSearchQuery {
    pub q: String,
    pub limit: Option<i64>,
}

/// Search users for @mention autocomplete
pub async fn search_users(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<UserSearchQuery>,
) -> AppResult<Json<Vec<UserSearchResult>>> {
    let org_id = get_org_id(&user)?;
    let limit = query.limit.unwrap_or(10);

    let users = services
        .collaboration
        .search_users(org_id, &query.q, limit)
        .await?;
    Ok(Json(users))
}

// =====================================================
// NOTIFICATION PREFERENCES
// =====================================================

/// Get notification preferences
pub async fn get_notification_preferences(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<NotificationPreferences>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let prefs = services
        .collaboration
        .get_notification_preferences(org_id, user_id)
        .await?;
    Ok(Json(prefs))
}

/// Update notification preferences
pub async fn update_notification_preferences(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(input): Json<UpdateNotificationPreferences>,
) -> AppResult<Json<NotificationPreferences>> {
    let org_id = get_org_id(&user)?;
    let user_id = get_user_id(&user)?;
    let prefs = services
        .collaboration
        .update_notification_preferences(org_id, user_id, input)
        .await?;
    Ok(Json(prefs))
}

/// Get available notification types
pub async fn get_notification_types() -> Json<Vec<&'static str>> {
    Json(NOTIFICATION_TYPES.to_vec())
}

// =====================================================
// PRESENCE
// =====================================================

/// Get users currently viewing an entity
pub async fn get_entity_presence(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(path): Path<EntityPath>,
) -> AppResult<Json<Vec<PresenceInfo>>> {
    let org_id = get_org_id(&user)?;
    let presence = services
        .collaboration
        .get_entity_presence(org_id, &path.entity_type, path.entity_id)
        .await?;
    Ok(Json(presence))
}

/// Update presence (for REST fallback when WebSocket not available)
pub async fn update_presence(
    State(_services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Json(_input): Json<UpdatePresence>,
) -> AppResult<StatusCode> {
    let _org_id = get_org_id(&user)?;

    // For REST updates, we need a session - create a temporary one if needed
    // In practice, most presence updates will go through WebSocket
    // This is just a fallback

    Ok(StatusCode::OK)
}

// =====================================================
// COLLABORATION STATS
// =====================================================

#[derive(Debug, Serialize)]
pub struct CollaborationStats {
    pub comment_stats: CommentStats,
    pub active_sessions: i64,
    pub online_users: i64,
}

/// Get overall collaboration statistics
pub async fn get_collaboration_stats(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<CollaborationStats>> {
    let org_id = get_org_id(&user)?;

    let comment_stats = services.collaboration.get_comment_stats(org_id).await?;

    // Get active session and online user counts
    let (active_sessions, online_users): (i64, i64) = sqlx::query_as(
        r#"
        SELECT
            COUNT(*),
            COUNT(DISTINCT user_id)
        FROM websocket_sessions
        WHERE organization_id = $1 AND status = 'connected'
          AND last_heartbeat_at > NOW() - INTERVAL '5 minutes'
        "#,
    )
    .bind(org_id)
    .fetch_one(&services.db)
    .await?;

    Ok(Json(CollaborationStats {
        comment_stats,
        active_sessions,
        online_users,
    }))
}

// =====================================================
// SLACK INTEGRATION
// =====================================================

#[derive(Debug, Deserialize)]
pub struct SlackOAuthQuery {
    pub code: String,
    pub state: Option<String>,
}

/// Initiate Slack OAuth flow
pub async fn slack_oauth_start(
    State(_services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;

    // Generate OAuth URL
    let client_id = std::env::var("SLACK_CLIENT_ID")
        .map_err(|_| AppError::InternalServerError("Slack client ID not configured".to_string()))?;

    let redirect_uri = std::env::var("SLACK_REDIRECT_URI")
        .unwrap_or_else(|_| "https://api.opengrc.com/api/v1/collaboration/slack/callback".to_string());

    let state = format!("{}:{}", org_id, Uuid::new_v4());

    let scopes = "channels:read,chat:write,users:read,users:read.email,incoming-webhook";

    let oauth_url = format!(
        "https://slack.com/oauth/v2/authorize?client_id={}&scope={}&redirect_uri={}&state={}",
        client_id, scopes, redirect_uri, state
    );

    Ok(Json(serde_json::json!({
        "oauth_url": oauth_url,
        "state": state
    })))
}

/// Handle Slack OAuth callback
pub async fn slack_oauth_callback(
    State(services): State<Arc<AppServices>>,
    Query(query): Query<SlackOAuthQuery>,
) -> AppResult<Json<serde_json::Value>> {
    // Parse state to get org_id
    let state = query.state.ok_or_else(|| AppError::BadRequest("Missing state parameter".to_string()))?;
    let parts: Vec<&str> = state.split(':').collect();
    if parts.len() != 2 {
        return Err(AppError::BadRequest("Invalid state parameter".to_string()));
    }

    let org_id = Uuid::parse_str(parts[0])
        .map_err(|_| AppError::BadRequest("Invalid organization ID".to_string()))?;

    // Exchange code for token
    let client_id = std::env::var("SLACK_CLIENT_ID")
        .map_err(|_| AppError::InternalServerError("Slack client ID not configured".to_string()))?;
    let client_secret = std::env::var("SLACK_CLIENT_SECRET")
        .map_err(|_| AppError::InternalServerError("Slack client secret not configured".to_string()))?;
    let redirect_uri = std::env::var("SLACK_REDIRECT_URI")
        .unwrap_or_else(|_| "https://api.opengrc.com/api/v1/collaboration/slack/callback".to_string());

    let client = reqwest::Client::new();
    let response = client
        .post("https://slack.com/api/oauth.v2.access")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", query.code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Slack OAuth error: {}", e)))?;

    let token_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse Slack response: {}", e)))?;

    if !token_response.get("ok").and_then(|v| v.as_bool()).unwrap_or(false) {
        let error = token_response.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
        return Err(AppError::ExternalServiceError(format!("Slack OAuth failed: {}", error)));
    }

    // Extract tokens and team info
    let access_token = token_response.get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ExternalServiceError("Missing access token".to_string()))?;

    let team = token_response.get("team")
        .ok_or_else(|| AppError::ExternalServiceError("Missing team info".to_string()))?;

    let team_id = team.get("id").and_then(|v| v.as_str()).unwrap_or("");
    let team_name = team.get("name").and_then(|v| v.as_str()).unwrap_or("");

    let bot_user_id = token_response.get("bot_user_id").and_then(|v| v.as_str());

    // Store workspace connection
    sqlx::query(
        r#"
        INSERT INTO slack_workspaces (organization_id, team_id, team_name, access_token, bot_user_id, status)
        VALUES ($1, $2, $3, $4, $5, 'active')
        ON CONFLICT (organization_id, team_id) DO UPDATE SET
            team_name = EXCLUDED.team_name,
            access_token = EXCLUDED.access_token,
            bot_user_id = EXCLUDED.bot_user_id,
            status = 'active',
            updated_at = NOW()
        "#,
    )
    .bind(org_id)
    .bind(team_id)
    .bind(team_name)
    .bind(access_token)
    .bind(bot_user_id)
    .execute(&services.db)
    .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "team_id": team_id,
        "team_name": team_name
    })))
}

/// Get Slack workspaces
pub async fn get_slack_workspaces(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    let org_id = get_org_id(&user)?;

    let workspaces: Vec<(Uuid, String, String, Option<String>, Option<String>, String, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r#"
        SELECT id, team_id, team_name, bot_user_id, default_channel_name, status, last_activity_at, created_at
        FROM slack_workspaces
        WHERE organization_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(org_id)
    .fetch_all(&services.db)
    .await?;

    let result: Vec<serde_json::Value> = workspaces
        .into_iter()
        .map(|(id, team_id, team_name, bot_user_id, default_channel_name, status, last_activity_at, created_at)| {
            serde_json::json!({
                "id": id,
                "team_id": team_id,
                "team_name": team_name,
                "bot_user_id": bot_user_id,
                "default_channel_name": default_channel_name,
                "status": status,
                "last_activity_at": last_activity_at,
                "created_at": created_at
            })
        })
        .collect();

    Ok(Json(result))
}

/// Disconnect Slack workspace
pub async fn disconnect_slack_workspace(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(workspace_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let org_id = get_org_id(&user)?;

    let result = sqlx::query(
        r#"
        UPDATE slack_workspaces SET status = 'inactive', updated_at = NOW()
        WHERE id = $1 AND organization_id = $2
        "#,
    )
    .bind(workspace_id)
    .bind(org_id)
    .execute(&services.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Slack workspace not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// =====================================================
// MICROSOFT TEAMS INTEGRATION
// =====================================================

/// Initiate Teams OAuth flow
pub async fn teams_oauth_start(
    State(_services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;

    let client_id = std::env::var("TEAMS_CLIENT_ID")
        .map_err(|_| AppError::InternalServerError("Teams client ID not configured".to_string()))?;

    let redirect_uri = std::env::var("TEAMS_REDIRECT_URI")
        .unwrap_or_else(|_| "https://api.opengrc.com/api/v1/collaboration/teams/callback".to_string());

    let state = format!("{}:{}", org_id, Uuid::new_v4());

    let scopes = "https://graph.microsoft.com/.default offline_access";

    let oauth_url = format!(
        "https://login.microsoftonline.com/common/oauth2/v2.0/authorize?client_id={}&response_type=code&redirect_uri={}&scope={}&state={}",
        client_id, redirect_uri, urlencoding::encode(scopes), state
    );

    Ok(Json(serde_json::json!({
        "oauth_url": oauth_url,
        "state": state
    })))
}

/// Handle Teams OAuth callback
pub async fn teams_oauth_callback(
    State(services): State<Arc<AppServices>>,
    Query(query): Query<SlackOAuthQuery>,
) -> AppResult<Json<serde_json::Value>> {
    // Parse state to get org_id
    let state = query.state.ok_or_else(|| AppError::BadRequest("Missing state parameter".to_string()))?;
    let parts: Vec<&str> = state.split(':').collect();
    if parts.len() != 2 {
        return Err(AppError::BadRequest("Invalid state parameter".to_string()));
    }

    let org_id = Uuid::parse_str(parts[0])
        .map_err(|_| AppError::BadRequest("Invalid organization ID".to_string()))?;

    let client_id = std::env::var("TEAMS_CLIENT_ID")
        .map_err(|_| AppError::InternalServerError("Teams client ID not configured".to_string()))?;
    let client_secret = std::env::var("TEAMS_CLIENT_SECRET")
        .map_err(|_| AppError::InternalServerError("Teams client secret not configured".to_string()))?;
    let redirect_uri = std::env::var("TEAMS_REDIRECT_URI")
        .unwrap_or_else(|_| "https://api.opengrc.com/api/v1/collaboration/teams/callback".to_string());

    let client = reqwest::Client::new();
    let response = client
        .post("https://login.microsoftonline.com/common/oauth2/v2.0/token")
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", query.code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Teams OAuth error: {}", e)))?;

    let token_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse Teams response: {}", e)))?;

    if token_response.get("error").is_some() {
        let error = token_response.get("error_description")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error");
        return Err(AppError::ExternalServiceError(format!("Teams OAuth failed: {}", error)));
    }

    let access_token = token_response.get("access_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::ExternalServiceError("Missing access token".to_string()))?;

    let refresh_token = token_response.get("refresh_token")
        .and_then(|v| v.as_str());

    let expires_in = token_response.get("expires_in")
        .and_then(|v| v.as_i64())
        .unwrap_or(3600);

    let token_expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in);

    // Get tenant info from Graph API
    let tenant_response = client
        .get("https://graph.microsoft.com/v1.0/organization")
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to get tenant info: {}", e)))?;

    let tenant_info: serde_json::Value = tenant_response
        .json()
        .await
        .map_err(|e| AppError::ExternalServiceError(format!("Failed to parse tenant info: {}", e)))?;

    let tenant_id = tenant_info.get("value")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|org| org.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let tenant_name = tenant_info.get("value")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|org| org.get("displayName"))
        .and_then(|v| v.as_str());

    // Store tenant connection
    sqlx::query(
        r#"
        INSERT INTO teams_tenants (organization_id, tenant_id, tenant_name, access_token, refresh_token, token_expires_at, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'active')
        ON CONFLICT (organization_id, tenant_id) DO UPDATE SET
            tenant_name = EXCLUDED.tenant_name,
            access_token = EXCLUDED.access_token,
            refresh_token = EXCLUDED.refresh_token,
            token_expires_at = EXCLUDED.token_expires_at,
            status = 'active',
            updated_at = NOW()
        "#,
    )
    .bind(org_id)
    .bind(tenant_id)
    .bind(tenant_name)
    .bind(access_token)
    .bind(refresh_token)
    .bind(token_expires_at)
    .execute(&services.db)
    .await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "tenant_id": tenant_id,
        "tenant_name": tenant_name
    })))
}

/// Get Teams tenants
pub async fn get_teams_tenants(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<Vec<serde_json::Value>>> {
    let org_id = get_org_id(&user)?;

    let tenants: Vec<(Uuid, String, Option<String>, Option<String>, String, Option<chrono::DateTime<chrono::Utc>>, chrono::DateTime<chrono::Utc>)> = sqlx::query_as(
        r#"
        SELECT id, tenant_id, tenant_name, bot_id, status, last_activity_at, created_at
        FROM teams_tenants
        WHERE organization_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(org_id)
    .fetch_all(&services.db)
    .await?;

    let result: Vec<serde_json::Value> = tenants
        .into_iter()
        .map(|(id, tenant_id, tenant_name, bot_id, status, last_activity_at, created_at)| {
            serde_json::json!({
                "id": id,
                "tenant_id": tenant_id,
                "tenant_name": tenant_name,
                "bot_id": bot_id,
                "status": status,
                "last_activity_at": last_activity_at,
                "created_at": created_at
            })
        })
        .collect();

    Ok(Json(result))
}

/// Disconnect Teams tenant
pub async fn disconnect_teams_tenant(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(tenant_id): Path<Uuid>,
) -> AppResult<StatusCode> {
    let org_id = get_org_id(&user)?;

    let result = sqlx::query(
        r#"
        UPDATE teams_tenants SET status = 'inactive', updated_at = NOW()
        WHERE id = $1 AND organization_id = $2
        "#,
    )
    .bind(tenant_id)
    .bind(org_id)
    .execute(&services.db)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Teams tenant not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// =====================================================
// EMAIL DIGESTS
// =====================================================

/// Manually trigger digest processing (admin only)
pub async fn process_digests(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> AppResult<Json<serde_json::Value>> {
    let org_id = get_org_id(&user)?;

    // Process daily digests
    let daily_users = services.collaboration.get_users_needing_digest("daily").await?;
    let mut daily_count = 0;

    let now = chrono::Utc::now();
    let period_start = now - chrono::Duration::hours(24);

    for (user_org_id, user_id, _name, _email) in &daily_users {
        if *user_org_id != org_id {
            continue;
        }

        let content = services
            .collaboration
            .create_digest_content(*user_org_id, *user_id, period_start, now)
            .await?;

        if content.notifications.is_empty()
            && content.tasks_due.is_empty()
            && content.tasks_overdue.is_empty()
            && content.mentions.is_empty()
            && content.comments.is_empty()
        {
            continue;
        }

        let _digest = services
            .collaboration
            .create_digest(*user_org_id, *user_id, "daily", period_start, now, content)
            .await?;

        daily_count += 1;
    }

    Ok(Json(serde_json::json!({
        "daily_digests_created": daily_count
    })))
}
