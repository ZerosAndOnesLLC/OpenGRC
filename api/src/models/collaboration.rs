use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// =====================================================
// ENTITY COMMENTS
// =====================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntityComment {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub parent_comment_id: Option<Uuid>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCommentWithUser {
    #[serde(flatten)]
    pub comment: EntityComment,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub mentions: Vec<MentionInfo>,
    pub reply_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentionInfo {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEntityComment {
    pub content: String,
    pub parent_comment_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEntityComment {
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCommentsQuery {
    pub include_deleted: Option<bool>,
    pub include_replies: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl Default for ListCommentsQuery {
    fn default() -> Self {
        Self {
            include_deleted: None,
            include_replies: Some(true),
            limit: Some(100),
            offset: Some(0),
        }
    }
}

// Valid entity types for comments
pub const COMMENTABLE_ENTITY_TYPES: &[&str] = &[
    "control",
    "evidence",
    "policy",
    "risk",
    "audit",
    "audit_request",
    "audit_finding",
    "vendor",
    "asset",
    "task",
];

// =====================================================
// COMMENT MENTIONS
// =====================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommentMention {
    pub id: Uuid,
    pub comment_id: Uuid,
    pub mentioned_user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// =====================================================
// NOTIFICATION PREFERENCES
// =====================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NotificationPreferences {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub in_app_enabled: bool,
    pub email_enabled: bool,
    pub slack_enabled: bool,
    pub teams_enabled: bool,
    pub enabled_types: serde_json::Value,
    pub email_digest_enabled: bool,
    pub email_digest_frequency: Option<String>,
    pub email_digest_day_of_week: Option<i32>,
    pub email_digest_hour: Option<i32>,
    pub quiet_hours_enabled: bool,
    pub quiet_hours_start: Option<i32>,
    pub quiet_hours_end: Option<i32>,
    pub quiet_hours_timezone: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotificationPreferences {
    pub in_app_enabled: Option<bool>,
    pub email_enabled: Option<bool>,
    pub slack_enabled: Option<bool>,
    pub teams_enabled: Option<bool>,
    pub enabled_types: Option<serde_json::Value>,
    pub email_digest_enabled: Option<bool>,
    pub email_digest_frequency: Option<String>,
    pub email_digest_day_of_week: Option<i32>,
    pub email_digest_hour: Option<i32>,
    pub quiet_hours_enabled: Option<bool>,
    pub quiet_hours_start: Option<i32>,
    pub quiet_hours_end: Option<i32>,
    pub quiet_hours_timezone: Option<String>,
}

// Notification types that can be enabled/disabled
pub const NOTIFICATION_TYPES: &[&str] = &[
    "task_assigned",
    "task_due_soon",
    "task_due_today",
    "task_overdue",
    "comment_added",
    "comment_mention",
    "comment_reply",
    "policy_reminder",
    "policy_update",
    "security_alert",
    "control_test_failed",
    "evidence_expiring",
    "audit_request",
    "access_review_assigned",
];

// =====================================================
// EMAIL DIGESTS
// =====================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EmailDigest {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub digest_type: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub notification_count: i32,
    pub task_count: i32,
    pub comment_count: i32,
    pub mention_count: i32,
    pub content: serde_json::Value,
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigestContent {
    pub tasks_due: Vec<DigestTask>,
    pub tasks_overdue: Vec<DigestTask>,
    pub tasks_completed: i32,
    pub mentions: Vec<DigestMention>,
    pub comments: Vec<DigestComment>,
    pub notifications: Vec<DigestNotification>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DigestTask {
    pub id: Uuid,
    pub title: String,
    pub due_at: Option<DateTime<Utc>>,
    pub priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DigestMention {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_title: String,
    pub mentioned_by: String,
    pub comment_preview: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DigestComment {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub entity_title: String,
    pub user_name: String,
    pub comment_preview: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DigestNotification {
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub created_at: DateTime<Utc>,
}

// =====================================================
// SLACK INTEGRATION
// =====================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SlackWorkspace {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub team_id: String,
    pub team_name: String,
    pub access_token: String,
    pub bot_user_id: Option<String>,
    pub bot_access_token: Option<String>,
    pub incoming_webhook_url: Option<String>,
    pub incoming_webhook_channel: Option<String>,
    pub default_channel_id: Option<String>,
    pub default_channel_name: Option<String>,
    pub status: String,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackWorkspaceResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub team_id: String,
    pub team_name: String,
    pub bot_user_id: Option<String>,
    pub default_channel_id: Option<String>,
    pub default_channel_name: Option<String>,
    pub status: String,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectSlackWorkspace {
    pub code: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SlackChannelMapping {
    pub id: Uuid,
    pub workspace_id: Uuid,
    pub notification_type: String,
    pub channel_id: String,
    pub channel_name: String,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSlackChannelMapping {
    pub notification_type: String,
    pub channel_id: String,
    pub channel_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSlackChannelMapping {
    pub channel_id: Option<String>,
    pub channel_name: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SlackUserConnection {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub workspace_id: Uuid,
    pub slack_user_id: String,
    pub slack_username: Option<String>,
    pub dm_channel_id: Option<String>,
    pub connected_at: DateTime<Utc>,
}

// =====================================================
// MICROSOFT TEAMS INTEGRATION
// =====================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamsTenant {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub tenant_id: String,
    pub tenant_name: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub bot_id: Option<String>,
    pub service_url: Option<String>,
    pub incoming_webhook_url: Option<String>,
    pub default_team_id: Option<String>,
    pub default_channel_id: Option<String>,
    pub status: String,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamsTenantResponse {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub tenant_id: String,
    pub tenant_name: Option<String>,
    pub bot_id: Option<String>,
    pub default_team_id: Option<String>,
    pub default_channel_id: Option<String>,
    pub status: String,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectTeamsTenant {
    pub code: String,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamsChannelMapping {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub notification_type: String,
    pub team_id: String,
    pub channel_id: String,
    pub channel_name: Option<String>,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTeamsChannelMapping {
    pub notification_type: String,
    pub team_id: String,
    pub channel_id: String,
    pub channel_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TeamsUserConnection {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub teams_user_id: String,
    pub conversation_id: Option<String>,
    pub connected_at: DateTime<Utc>,
}

// =====================================================
// REAL-TIME COLLABORATION
// =====================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebSocketSession {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub session_token: String,
    pub connected_at: DateTime<Utc>,
    pub last_heartbeat_at: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWebSocketSession {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CollaborationPresence {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub is_editing: bool,
    pub editing_field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceInfo {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub is_editing: bool,
    pub editing_field: Option<String>,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePresence {
    pub entity_type: String,
    pub entity_id: Uuid,
    pub is_editing: Option<bool>,
    pub editing_field: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CollaborationEvent {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub event_type: String,
    pub entity_type: Option<String>,
    pub entity_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

// Event types for real-time collaboration
pub const COLLABORATION_EVENT_TYPES: &[&str] = &[
    "comment_added",
    "comment_updated",
    "comment_deleted",
    "entity_updated",
    "user_joined",
    "user_left",
    "typing_started",
    "typing_stopped",
    "presence_updated",
];

// =====================================================
// WEBSOCKET MESSAGES
// =====================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    // Client -> Server
    Subscribe {
        entity_type: String,
        entity_id: Uuid,
    },
    Unsubscribe {
        entity_type: String,
        entity_id: Uuid,
    },
    Heartbeat,
    TypingStart {
        entity_type: String,
        entity_id: Uuid,
        field: Option<String>,
    },
    TypingStop {
        entity_type: String,
        entity_id: Uuid,
    },

    // Server -> Client
    Subscribed {
        entity_type: String,
        entity_id: Uuid,
        presence: Vec<PresenceInfo>,
    },
    Unsubscribed {
        entity_type: String,
        entity_id: Uuid,
    },
    UserJoined {
        entity_type: String,
        entity_id: Uuid,
        user: PresenceInfo,
    },
    UserLeft {
        entity_type: String,
        entity_id: Uuid,
        user_id: Uuid,
    },
    UserTyping {
        entity_type: String,
        entity_id: Uuid,
        user_id: Uuid,
        user_name: String,
        field: Option<String>,
    },
    UserStoppedTyping {
        entity_type: String,
        entity_id: Uuid,
        user_id: Uuid,
    },
    CommentAdded {
        entity_type: String,
        entity_id: Uuid,
        comment: EntityCommentWithUser,
    },
    CommentUpdated {
        entity_type: String,
        entity_id: Uuid,
        comment: EntityCommentWithUser,
    },
    CommentDeleted {
        entity_type: String,
        entity_id: Uuid,
        comment_id: Uuid,
    },
    EntityUpdated {
        entity_type: String,
        entity_id: Uuid,
        updated_by: Uuid,
        updated_by_name: String,
        changes: serde_json::Value,
    },
    Notification {
        id: Uuid,
        notification_type: String,
        title: String,
        message: String,
        data: serde_json::Value,
    },
    Error {
        message: String,
    },
}

// =====================================================
// COMMENT STATS
// =====================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentStats {
    pub total_comments: i64,
    pub comments_today: i64,
    pub comments_this_week: i64,
    pub active_commenters: i64,
    pub by_entity_type: Vec<CommentEntityTypeCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommentEntityTypeCount {
    pub entity_type: String,
    pub count: i64,
}

// =====================================================
// USER SEARCH (for @mentions)
// =====================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchQuery {
    pub query: String,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSearchResult {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}
