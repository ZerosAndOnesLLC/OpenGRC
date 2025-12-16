use crate::cache::{org_cache_key, CacheClient};
use crate::models::collaboration::{
    CollaborationEvent, CollaborationPresence, CommentEntityTypeCount,
    CommentStats, CreateEntityComment, DigestComment, DigestContent, DigestMention,
    DigestNotification, DigestTask, EmailDigest, EntityComment, EntityCommentWithUser,
    ListCommentsQuery, MentionInfo, NotificationPreferences, PresenceInfo, UpdateEntityComment,
    UpdateNotificationPreferences, UpdatePresence, UserSearchResult, WebSocketSession,
    COMMENTABLE_ENTITY_TYPES,
};
use crate::services::notification::{CreateNotification, NotificationService};
use crate::utils::{AppError, AppResult};
use chrono::{DateTime, Duration, Utc};
use regex::Regex;
use sqlx::PgPool;
use uuid::Uuid;

const CACHE_PREFIX_COMMENTS: &str = "comments";

#[derive(Clone)]
pub struct CollaborationService {
    db: PgPool,
    cache: CacheClient,
}

impl CollaborationService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    /// Invalidate comment caches for an entity
    async fn invalidate_comment_caches(&self, org_id: Uuid, entity_type: &str, entity_id: Uuid) {
        let pattern = org_cache_key(
            &org_id.to_string(),
            CACHE_PREFIX_COMMENTS,
            &format!("{}:{}:*", entity_type, entity_id),
        );
        let _ = self.cache.delete_pattern(&pattern).await;
    }

    // ==================== Entity Comments ====================

    /// List comments for an entity
    pub async fn list_comments(
        &self,
        org_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        query: ListCommentsQuery,
    ) -> AppResult<Vec<EntityCommentWithUser>> {
        // Validate entity type
        if !COMMENTABLE_ENTITY_TYPES.contains(&entity_type) {
            return Err(AppError::BadRequest(format!(
                "Invalid entity type: {}. Valid types are: {:?}",
                entity_type, COMMENTABLE_ENTITY_TYPES
            )));
        }

        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);
        let include_deleted = query.include_deleted.unwrap_or(false);
        let include_replies = query.include_replies.unwrap_or(true);

        // Get top-level comments (parent_comment_id IS NULL) or all comments based on query
        let comments: Vec<EntityComment> = if include_replies {
            if include_deleted {
                sqlx::query_as::<_, EntityComment>(
                    r#"
                    SELECT id, organization_id, entity_type, entity_id, user_id, content,
                           parent_comment_id, deleted_at, deleted_by, created_at, updated_at
                    FROM entity_comments
                    WHERE organization_id = $1 AND entity_type = $2 AND entity_id = $3
                    ORDER BY created_at ASC
                    LIMIT $4 OFFSET $5
                    "#,
                )
                .bind(org_id)
                .bind(entity_type)
                .bind(entity_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.db)
                .await?
            } else {
                sqlx::query_as::<_, EntityComment>(
                    r#"
                    SELECT id, organization_id, entity_type, entity_id, user_id, content,
                           parent_comment_id, deleted_at, deleted_by, created_at, updated_at
                    FROM entity_comments
                    WHERE organization_id = $1 AND entity_type = $2 AND entity_id = $3
                      AND deleted_at IS NULL
                    ORDER BY created_at ASC
                    LIMIT $4 OFFSET $5
                    "#,
                )
                .bind(org_id)
                .bind(entity_type)
                .bind(entity_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.db)
                .await?
            }
        } else {
            // Only top-level comments
            if include_deleted {
                sqlx::query_as::<_, EntityComment>(
                    r#"
                    SELECT id, organization_id, entity_type, entity_id, user_id, content,
                           parent_comment_id, deleted_at, deleted_by, created_at, updated_at
                    FROM entity_comments
                    WHERE organization_id = $1 AND entity_type = $2 AND entity_id = $3
                      AND parent_comment_id IS NULL
                    ORDER BY created_at ASC
                    LIMIT $4 OFFSET $5
                    "#,
                )
                .bind(org_id)
                .bind(entity_type)
                .bind(entity_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.db)
                .await?
            } else {
                sqlx::query_as::<_, EntityComment>(
                    r#"
                    SELECT id, organization_id, entity_type, entity_id, user_id, content,
                           parent_comment_id, deleted_at, deleted_by, created_at, updated_at
                    FROM entity_comments
                    WHERE organization_id = $1 AND entity_type = $2 AND entity_id = $3
                      AND parent_comment_id IS NULL AND deleted_at IS NULL
                    ORDER BY created_at ASC
                    LIMIT $4 OFFSET $5
                    "#,
                )
                .bind(org_id)
                .bind(entity_type)
                .bind(entity_id)
                .bind(limit)
                .bind(offset)
                .fetch_all(&self.db)
                .await?
            }
        };

        // Get user info and mentions for all comments
        self.enrich_comments(comments).await
    }

    /// Get a single comment by ID
    pub async fn get_comment(
        &self,
        org_id: Uuid,
        comment_id: Uuid,
    ) -> AppResult<EntityCommentWithUser> {
        let comment = sqlx::query_as::<_, EntityComment>(
            r#"
            SELECT id, organization_id, entity_type, entity_id, user_id, content,
                   parent_comment_id, deleted_at, deleted_by, created_at, updated_at
            FROM entity_comments
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(comment_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        let enriched = self.enrich_comments(vec![comment]).await?;
        enriched
            .into_iter()
            .next()
            .ok_or_else(|| AppError::InternalServerError("Failed to enrich comment".to_string()))
    }

    /// Create a new comment
    pub async fn create_comment(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        input: CreateEntityComment,
    ) -> AppResult<EntityCommentWithUser> {
        // Validate entity type
        if !COMMENTABLE_ENTITY_TYPES.contains(&entity_type) {
            return Err(AppError::BadRequest(format!(
                "Invalid entity type: {}. Valid types are: {:?}",
                entity_type, COMMENTABLE_ENTITY_TYPES
            )));
        }

        // Validate content
        if input.content.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Comment content cannot be empty".to_string(),
            ));
        }

        // If this is a reply, verify parent comment exists
        if let Some(parent_id) = input.parent_comment_id {
            let parent_exists = sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT id FROM entity_comments
                WHERE id = $1 AND organization_id = $2 AND entity_type = $3 AND entity_id = $4
                "#,
            )
            .bind(parent_id)
            .bind(org_id)
            .bind(entity_type)
            .bind(entity_id)
            .fetch_optional(&self.db)
            .await?;

            if parent_exists.is_none() {
                return Err(AppError::NotFound("Parent comment not found".to_string()));
            }
        }

        // Parse mentions from content
        let mentioned_user_ids = self.parse_mentions(&input.content, org_id).await?;

        // Create comment
        let comment = sqlx::query_as::<_, EntityComment>(
            r#"
            INSERT INTO entity_comments (organization_id, entity_type, entity_id, user_id, content, parent_comment_id)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, organization_id, entity_type, entity_id, user_id, content,
                      parent_comment_id, deleted_at, deleted_by, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(user_id)
        .bind(&input.content)
        .bind(input.parent_comment_id)
        .fetch_one(&self.db)
        .await?;

        // Create mention records
        for mentioned_user_id in &mentioned_user_ids {
            sqlx::query(
                r#"
                INSERT INTO comment_mentions (comment_id, mentioned_user_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(comment.id)
            .bind(mentioned_user_id)
            .execute(&self.db)
            .await?;
        }

        // Create collaboration event
        self.create_event(
            org_id,
            "comment_added",
            Some(entity_type),
            Some(entity_id),
            Some(user_id),
            serde_json::json!({
                "comment_id": comment.id,
                "content_preview": input.content.chars().take(100).collect::<String>(),
            }),
        )
        .await?;

        // Invalidate cache
        self.invalidate_comment_caches(org_id, entity_type, entity_id)
            .await;

        // Return enriched comment
        let enriched = self.enrich_comments(vec![comment]).await?;
        enriched
            .into_iter()
            .next()
            .ok_or_else(|| AppError::InternalServerError("Failed to enrich comment".to_string()))
    }

    /// Update a comment
    pub async fn update_comment(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        comment_id: Uuid,
        input: UpdateEntityComment,
    ) -> AppResult<EntityCommentWithUser> {
        // Validate content
        if input.content.trim().is_empty() {
            return Err(AppError::BadRequest(
                "Comment content cannot be empty".to_string(),
            ));
        }

        // Check comment exists and user owns it
        let existing = sqlx::query_as::<_, EntityComment>(
            r#"
            SELECT id, organization_id, entity_type, entity_id, user_id, content,
                   parent_comment_id, deleted_at, deleted_by, created_at, updated_at
            FROM entity_comments
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(comment_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        if existing.user_id != user_id {
            return Err(AppError::Forbidden(
                "You can only edit your own comments".to_string(),
            ));
        }

        if existing.deleted_at.is_some() {
            return Err(AppError::BadRequest(
                "Cannot edit a deleted comment".to_string(),
            ));
        }

        // Parse new mentions
        let mentioned_user_ids = self.parse_mentions(&input.content, org_id).await?;

        // Update comment
        let comment = sqlx::query_as::<_, EntityComment>(
            r#"
            UPDATE entity_comments SET content = $3, updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, entity_type, entity_id, user_id, content,
                      parent_comment_id, deleted_at, deleted_by, created_at, updated_at
            "#,
        )
        .bind(comment_id)
        .bind(org_id)
        .bind(&input.content)
        .fetch_one(&self.db)
        .await?;

        // Update mentions (remove old, add new)
        sqlx::query("DELETE FROM comment_mentions WHERE comment_id = $1")
            .bind(comment_id)
            .execute(&self.db)
            .await?;

        for mentioned_user_id in &mentioned_user_ids {
            sqlx::query(
                r#"
                INSERT INTO comment_mentions (comment_id, mentioned_user_id)
                VALUES ($1, $2)
                ON CONFLICT DO NOTHING
                "#,
            )
            .bind(comment_id)
            .bind(mentioned_user_id)
            .execute(&self.db)
            .await?;
        }

        // Create collaboration event
        self.create_event(
            org_id,
            "comment_updated",
            Some(&existing.entity_type),
            Some(existing.entity_id),
            Some(user_id),
            serde_json::json!({
                "comment_id": comment.id,
            }),
        )
        .await?;

        // Invalidate cache
        self.invalidate_comment_caches(org_id, &existing.entity_type, existing.entity_id)
            .await;

        // Return enriched comment
        let enriched = self.enrich_comments(vec![comment]).await?;
        enriched
            .into_iter()
            .next()
            .ok_or_else(|| AppError::InternalServerError("Failed to enrich comment".to_string()))
    }

    /// Delete a comment (soft delete)
    pub async fn delete_comment(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        comment_id: Uuid,
    ) -> AppResult<()> {
        // Check comment exists and user owns it (or is admin)
        let existing = sqlx::query_as::<_, EntityComment>(
            r#"
            SELECT id, organization_id, entity_type, entity_id, user_id, content,
                   parent_comment_id, deleted_at, deleted_by, created_at, updated_at
            FROM entity_comments
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(comment_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Comment not found".to_string()))?;

        // For now, only comment owner can delete (add admin check if needed)
        if existing.user_id != user_id {
            return Err(AppError::Forbidden(
                "You can only delete your own comments".to_string(),
            ));
        }

        if existing.deleted_at.is_some() {
            return Err(AppError::BadRequest("Comment is already deleted".to_string()));
        }

        // Soft delete
        sqlx::query(
            r#"
            UPDATE entity_comments SET deleted_at = NOW(), deleted_by = $3
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(comment_id)
        .bind(org_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        // Create collaboration event
        self.create_event(
            org_id,
            "comment_deleted",
            Some(&existing.entity_type),
            Some(existing.entity_id),
            Some(user_id),
            serde_json::json!({
                "comment_id": comment_id,
            }),
        )
        .await?;

        // Invalidate cache
        self.invalidate_comment_caches(org_id, &existing.entity_type, existing.entity_id)
            .await;

        Ok(())
    }

    /// Get comment statistics
    pub async fn get_comment_stats(&self, org_id: Uuid) -> AppResult<CommentStats> {
        let (total_comments, comments_today, comments_this_week, active_commenters): (
            i64,
            i64,
            i64,
            i64,
        ) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE deleted_at IS NULL),
                COUNT(*) FILTER (WHERE deleted_at IS NULL AND created_at::date = CURRENT_DATE),
                COUNT(*) FILTER (WHERE deleted_at IS NULL AND created_at >= NOW() - INTERVAL '7 days'),
                COUNT(DISTINCT user_id) FILTER (WHERE deleted_at IS NULL AND created_at >= NOW() - INTERVAL '7 days')
            FROM entity_comments
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let by_entity_type: Vec<CommentEntityTypeCount> = sqlx::query_as(
            r#"
            SELECT entity_type, COUNT(*) as count
            FROM entity_comments
            WHERE organization_id = $1 AND deleted_at IS NULL
            GROUP BY entity_type
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(CommentStats {
            total_comments,
            comments_today,
            comments_this_week,
            active_commenters,
            by_entity_type,
        })
    }

    /// Parse @mentions from content and return user IDs
    async fn parse_mentions(&self, content: &str, org_id: Uuid) -> AppResult<Vec<Uuid>> {
        // Match @username or @"Full Name" or @email@domain.com patterns
        let re = Regex::new(r#"@(?:"([^"]+)"|(\S+@\S+\.\S+)|(\w+))"#)
            .map_err(|e| AppError::InternalServerError(format!("Regex error: {}", e)))?;

        let mut mentioned_user_ids = Vec::new();

        for cap in re.captures_iter(content) {
            let identifier = cap
                .get(1)
                .or_else(|| cap.get(2))
                .or_else(|| cap.get(3))
                .map(|m| m.as_str())
                .unwrap_or("");

            if identifier.is_empty() {
                continue;
            }

            // Try to find user by email or name
            let user_id: Option<Uuid> = sqlx::query_scalar(
                r#"
                SELECT id FROM users
                WHERE organization_id = $1
                  AND (LOWER(email) = LOWER($2) OR LOWER(name) = LOWER($2))
                LIMIT 1
                "#,
            )
            .bind(org_id)
            .bind(identifier)
            .fetch_optional(&self.db)
            .await?;

            if let Some(id) = user_id {
                if !mentioned_user_ids.contains(&id) {
                    mentioned_user_ids.push(id);
                }
            }
        }

        Ok(mentioned_user_ids)
    }

    /// Enrich comments with user info and mentions
    async fn enrich_comments(
        &self,
        comments: Vec<EntityComment>,
    ) -> AppResult<Vec<EntityCommentWithUser>> {
        if comments.is_empty() {
            return Ok(vec![]);
        }

        // Get all user IDs
        let user_ids: Vec<Uuid> = comments.iter().map(|c| c.user_id).collect();
        let comment_ids: Vec<Uuid> = comments.iter().map(|c| c.id).collect();

        // Get user info
        let users: Vec<(Uuid, String, String)> = if !user_ids.is_empty() {
            sqlx::query_as("SELECT id, name, email FROM users WHERE id = ANY($1)")
                .bind(&user_ids)
                .fetch_all(&self.db)
                .await?
        } else {
            vec![]
        };

        let user_map: std::collections::HashMap<Uuid, (String, String)> = users
            .into_iter()
            .map(|(id, name, email)| (id, (name, email)))
            .collect();

        // Get mentions for all comments
        let mentions: Vec<(Uuid, Uuid, String, String)> = if !comment_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT cm.comment_id, u.id, u.name, u.email
                FROM comment_mentions cm
                JOIN users u ON cm.mentioned_user_id = u.id
                WHERE cm.comment_id = ANY($1)
                "#,
            )
            .bind(&comment_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let mut mentions_map: std::collections::HashMap<Uuid, Vec<MentionInfo>> =
            std::collections::HashMap::new();
        for (comment_id, user_id, name, email) in mentions {
            mentions_map
                .entry(comment_id)
                .or_default()
                .push(MentionInfo {
                    user_id,
                    user_name: name,
                    user_email: email,
                });
        }

        // Get reply counts
        let reply_counts: Vec<(Uuid, i64)> = if !comment_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT parent_comment_id, COUNT(*) as count
                FROM entity_comments
                WHERE parent_comment_id = ANY($1) AND deleted_at IS NULL
                GROUP BY parent_comment_id
                "#,
            )
            .bind(&comment_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let reply_count_map: std::collections::HashMap<Uuid, i64> =
            reply_counts.into_iter().collect();

        // Build result
        let result: Vec<EntityCommentWithUser> = comments
            .into_iter()
            .map(|comment| {
                let (user_name, user_email) = user_map
                    .get(&comment.user_id)
                    .cloned()
                    .map(|(n, e)| (Some(n), Some(e)))
                    .unwrap_or((None, None));

                let comment_mentions = mentions_map.remove(&comment.id).unwrap_or_default();
                let reply_count = reply_count_map.get(&comment.id).copied().unwrap_or(0);

                EntityCommentWithUser {
                    comment,
                    user_name,
                    user_email,
                    mentions: comment_mentions,
                    reply_count,
                }
            })
            .collect();

        Ok(result)
    }

    // ==================== Notification Preferences ====================

    /// Get user's notification preferences (creates default if not exists)
    pub async fn get_notification_preferences(
        &self,
        org_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<NotificationPreferences> {
        // Try to get existing preferences
        let prefs = sqlx::query_as::<_, NotificationPreferences>(
            r#"
            SELECT id, organization_id, user_id, in_app_enabled, email_enabled,
                   slack_enabled, teams_enabled, enabled_types, email_digest_enabled,
                   email_digest_frequency, email_digest_day_of_week, email_digest_hour,
                   quiet_hours_enabled, quiet_hours_start, quiet_hours_end, quiet_hours_timezone,
                   created_at, updated_at
            FROM notification_preferences
            WHERE organization_id = $1 AND user_id = $2
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(p) = prefs {
            return Ok(p);
        }

        // Create default preferences
        let prefs = sqlx::query_as::<_, NotificationPreferences>(
            r#"
            INSERT INTO notification_preferences (organization_id, user_id)
            VALUES ($1, $2)
            RETURNING id, organization_id, user_id, in_app_enabled, email_enabled,
                      slack_enabled, teams_enabled, enabled_types, email_digest_enabled,
                      email_digest_frequency, email_digest_day_of_week, email_digest_hour,
                      quiet_hours_enabled, quiet_hours_start, quiet_hours_end, quiet_hours_timezone,
                      created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(prefs)
    }

    /// Update user's notification preferences
    pub async fn update_notification_preferences(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        input: UpdateNotificationPreferences,
    ) -> AppResult<NotificationPreferences> {
        // Ensure preferences exist
        let _ = self.get_notification_preferences(org_id, user_id).await?;

        // Update
        let prefs = sqlx::query_as::<_, NotificationPreferences>(
            r#"
            UPDATE notification_preferences SET
                in_app_enabled = COALESCE($3, in_app_enabled),
                email_enabled = COALESCE($4, email_enabled),
                slack_enabled = COALESCE($5, slack_enabled),
                teams_enabled = COALESCE($6, teams_enabled),
                enabled_types = COALESCE($7, enabled_types),
                email_digest_enabled = COALESCE($8, email_digest_enabled),
                email_digest_frequency = COALESCE($9, email_digest_frequency),
                email_digest_day_of_week = COALESCE($10, email_digest_day_of_week),
                email_digest_hour = COALESCE($11, email_digest_hour),
                quiet_hours_enabled = COALESCE($12, quiet_hours_enabled),
                quiet_hours_start = COALESCE($13, quiet_hours_start),
                quiet_hours_end = COALESCE($14, quiet_hours_end),
                quiet_hours_timezone = COALESCE($15, quiet_hours_timezone),
                updated_at = NOW()
            WHERE organization_id = $1 AND user_id = $2
            RETURNING id, organization_id, user_id, in_app_enabled, email_enabled,
                      slack_enabled, teams_enabled, enabled_types, email_digest_enabled,
                      email_digest_frequency, email_digest_day_of_week, email_digest_hour,
                      quiet_hours_enabled, quiet_hours_start, quiet_hours_end, quiet_hours_timezone,
                      created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .bind(input.in_app_enabled)
        .bind(input.email_enabled)
        .bind(input.slack_enabled)
        .bind(input.teams_enabled)
        .bind(&input.enabled_types)
        .bind(input.email_digest_enabled)
        .bind(&input.email_digest_frequency)
        .bind(input.email_digest_day_of_week)
        .bind(input.email_digest_hour)
        .bind(input.quiet_hours_enabled)
        .bind(input.quiet_hours_start)
        .bind(input.quiet_hours_end)
        .bind(&input.quiet_hours_timezone)
        .fetch_one(&self.db)
        .await?;

        Ok(prefs)
    }

    // ==================== User Search (for @mentions) ====================

    /// Search users for @mention autocomplete
    pub async fn search_users(
        &self,
        org_id: Uuid,
        query: &str,
        limit: i64,
    ) -> AppResult<Vec<UserSearchResult>> {
        let search_pattern = format!("%{}%", query.to_lowercase());
        let limit = limit.min(20);

        let users: Vec<UserSearchResult> = sqlx::query_as(
            r#"
            SELECT id, name, email
            FROM users
            WHERE organization_id = $1
              AND (LOWER(name) LIKE $2 OR LOWER(email) LIKE $2)
            ORDER BY name ASC
            LIMIT $3
            "#,
        )
        .bind(org_id)
        .bind(&search_pattern)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    // ==================== Collaboration Events ====================

    /// Create a collaboration event
    pub async fn create_event(
        &self,
        org_id: Uuid,
        event_type: &str,
        entity_type: Option<&str>,
        entity_id: Option<Uuid>,
        user_id: Option<Uuid>,
        data: serde_json::Value,
    ) -> AppResult<CollaborationEvent> {
        let event = sqlx::query_as::<_, CollaborationEvent>(
            r#"
            INSERT INTO collaboration_events (organization_id, event_type, entity_type, entity_id, user_id, data)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, organization_id, event_type, entity_type, entity_id, user_id, data, created_at
            "#,
        )
        .bind(org_id)
        .bind(event_type)
        .bind(entity_type)
        .bind(entity_id)
        .bind(user_id)
        .bind(&data)
        .fetch_one(&self.db)
        .await?;

        Ok(event)
    }

    /// Get recent collaboration events for an entity
    pub async fn get_entity_events(
        &self,
        org_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
        since: Option<DateTime<Utc>>,
        limit: i64,
    ) -> AppResult<Vec<CollaborationEvent>> {
        let since = since.unwrap_or_else(|| Utc::now() - Duration::hours(24));

        let events = sqlx::query_as::<_, CollaborationEvent>(
            r#"
            SELECT id, organization_id, event_type, entity_type, entity_id, user_id, data, created_at
            FROM collaboration_events
            WHERE organization_id = $1 AND entity_type = $2 AND entity_id = $3 AND created_at > $4
            ORDER BY created_at DESC
            LIMIT $5
            "#,
        )
        .bind(org_id)
        .bind(entity_type)
        .bind(entity_id)
        .bind(since)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(events)
    }

    // ==================== WebSocket Sessions ====================

    /// Create a new WebSocket session
    pub async fn create_websocket_session(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<WebSocketSession> {
        let session_token = format!("ws_{}_{}", user_id, Uuid::new_v4());

        let session = sqlx::query_as::<_, WebSocketSession>(
            r#"
            INSERT INTO websocket_sessions (organization_id, user_id, session_token, ip_address, user_agent)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, organization_id, user_id, session_token, connected_at, last_heartbeat_at,
                      ip_address, user_agent, status
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .bind(&session_token)
        .bind(&ip_address)
        .bind(&user_agent)
        .fetch_one(&self.db)
        .await?;

        Ok(session)
    }

    /// Update session heartbeat
    pub async fn update_session_heartbeat(&self, session_token: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE websocket_sessions SET last_heartbeat_at = NOW()
            WHERE session_token = $1 AND status = 'connected'
            "#,
        )
        .bind(session_token)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Mark session as disconnected
    pub async fn disconnect_session(&self, session_token: &str) -> AppResult<()> {
        sqlx::query(
            r#"
            UPDATE websocket_sessions SET status = 'disconnected'
            WHERE session_token = $1
            "#,
        )
        .bind(session_token)
        .execute(&self.db)
        .await?;

        // Also clean up presence
        sqlx::query(
            r#"
            DELETE FROM collaboration_presence
            WHERE session_id = (SELECT id FROM websocket_sessions WHERE session_token = $1)
            "#,
        )
        .bind(session_token)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ==================== Presence ====================

    /// Update user presence (what they're viewing/editing)
    pub async fn update_presence(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        session_id: Uuid,
        input: UpdatePresence,
    ) -> AppResult<CollaborationPresence> {
        // Upsert presence
        let presence = sqlx::query_as::<_, CollaborationPresence>(
            r#"
            INSERT INTO collaboration_presence (organization_id, user_id, session_id, entity_type, entity_id, is_editing, editing_field)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (session_id, entity_type, entity_id) DO UPDATE SET
                last_activity_at = NOW(),
                is_editing = COALESCE($6, collaboration_presence.is_editing),
                editing_field = COALESCE($7, collaboration_presence.editing_field)
            RETURNING id, organization_id, user_id, session_id, entity_type, entity_id,
                      started_at, last_activity_at, is_editing, editing_field
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .bind(session_id)
        .bind(&input.entity_type)
        .bind(input.entity_id)
        .bind(input.is_editing)
        .bind(&input.editing_field)
        .fetch_one(&self.db)
        .await?;

        Ok(presence)
    }

    /// Remove user presence from an entity
    pub async fn remove_presence(
        &self,
        session_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
    ) -> AppResult<()> {
        sqlx::query(
            r#"
            DELETE FROM collaboration_presence
            WHERE session_id = $1 AND entity_type = $2 AND entity_id = $3
            "#,
        )
        .bind(session_id)
        .bind(entity_type)
        .bind(entity_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get all users currently viewing an entity
    pub async fn get_entity_presence(
        &self,
        org_id: Uuid,
        entity_type: &str,
        entity_id: Uuid,
    ) -> AppResult<Vec<PresenceInfo>> {
        let presence: Vec<(Uuid, String, String, bool, Option<String>, DateTime<Utc>)> =
            sqlx::query_as(
                r#"
            SELECT cp.user_id, u.name, u.email, cp.is_editing, cp.editing_field, cp.started_at
            FROM collaboration_presence cp
            JOIN users u ON cp.user_id = u.id
            JOIN websocket_sessions ws ON cp.session_id = ws.id
            WHERE cp.organization_id = $1
              AND cp.entity_type = $2
              AND cp.entity_id = $3
              AND ws.status = 'connected'
              AND cp.last_activity_at > NOW() - INTERVAL '5 minutes'
            ORDER BY cp.started_at ASC
            "#,
            )
            .bind(org_id)
            .bind(entity_type)
            .bind(entity_id)
            .fetch_all(&self.db)
            .await?;

        let result = presence
            .into_iter()
            .map(
                |(user_id, user_name, user_email, is_editing, editing_field, started_at)| {
                    PresenceInfo {
                        user_id,
                        user_name,
                        user_email,
                        is_editing,
                        editing_field,
                        started_at,
                    }
                },
            )
            .collect();

        Ok(result)
    }

    // ==================== Email Digests ====================

    /// Get users who need a digest sent
    pub async fn get_users_needing_digest(
        &self,
        digest_type: &str,
    ) -> AppResult<Vec<(Uuid, Uuid, String, String)>> {
        // Get users with digest enabled who haven't received one in the appropriate period
        let interval = match digest_type {
            "daily" => "24 hours",
            "weekly" => "7 days",
            _ => "24 hours",
        };

        let users: Vec<(Uuid, Uuid, String, String)> = sqlx::query_as(&format!(
            r#"
            SELECT np.organization_id, np.user_id, u.name, u.email
            FROM notification_preferences np
            JOIN users u ON np.user_id = u.id
            WHERE np.email_digest_enabled = TRUE
              AND np.email_digest_frequency = $1
              AND NOT EXISTS (
                  SELECT 1 FROM email_digests ed
                  WHERE ed.user_id = np.user_id
                    AND ed.digest_type = $1
                    AND ed.created_at > NOW() - INTERVAL '{}'
              )
            "#,
            interval
        ))
        .bind(digest_type)
        .fetch_all(&self.db)
        .await?;

        Ok(users)
    }

    /// Create email digest content for a user
    pub async fn create_digest_content(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
    ) -> AppResult<DigestContent> {
        // Get tasks due
        let tasks_due: Vec<DigestTask> = sqlx::query_as(
            r#"
            SELECT id, title, due_at, priority
            FROM tasks
            WHERE organization_id = $1 AND assignee_id = $2
              AND due_at BETWEEN $3 AND $4
              AND status != 'completed'
            ORDER BY due_at ASC
            LIMIT 10
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(&self.db)
        .await?;

        // Get overdue tasks
        let tasks_overdue: Vec<DigestTask> = sqlx::query_as(
            r#"
            SELECT id, title, due_at, priority
            FROM tasks
            WHERE organization_id = $1 AND assignee_id = $2
              AND due_at < $3
              AND status != 'completed'
            ORDER BY due_at ASC
            LIMIT 10
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .bind(period_start)
        .fetch_all(&self.db)
        .await?;

        // Get completed tasks count
        let (tasks_completed,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*)
            FROM tasks
            WHERE organization_id = $1 AND assignee_id = $2
              AND completed_at BETWEEN $3 AND $4
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_one(&self.db)
        .await?;

        // Get mentions
        let mentions: Vec<DigestMention> = sqlx::query_as(
            r#"
            SELECT ec.entity_type, ec.entity_id, '' as entity_title, u.name as mentioned_by,
                   SUBSTRING(ec.content FROM 1 FOR 100) as comment_preview, ec.created_at
            FROM comment_mentions cm
            JOIN entity_comments ec ON cm.comment_id = ec.id
            JOIN users u ON ec.user_id = u.id
            WHERE cm.mentioned_user_id = $1
              AND ec.organization_id = $2
              AND ec.created_at BETWEEN $3 AND $4
              AND ec.deleted_at IS NULL
            ORDER BY ec.created_at DESC
            LIMIT 10
            "#,
        )
        .bind(user_id)
        .bind(org_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(&self.db)
        .await?;

        // Get recent comments on entities the user owns/is assigned to
        let comments: Vec<DigestComment> = sqlx::query_as(
            r#"
            SELECT ec.entity_type, ec.entity_id, '' as entity_title, u.name as user_name,
                   SUBSTRING(ec.content FROM 1 FOR 100) as comment_preview, ec.created_at
            FROM entity_comments ec
            JOIN users u ON ec.user_id = u.id
            WHERE ec.organization_id = $1
              AND ec.created_at BETWEEN $2 AND $3
              AND ec.deleted_at IS NULL
              AND ec.user_id != $4
              AND (
                  (ec.entity_type = 'task' AND ec.entity_id IN (
                      SELECT id FROM tasks WHERE assignee_id = $4 OR created_by = $4
                  ))
                  OR (ec.entity_type = 'control' AND ec.entity_id IN (
                      SELECT id FROM controls WHERE owner_id = $4
                  ))
                  OR (ec.entity_type = 'risk' AND ec.entity_id IN (
                      SELECT id FROM risks WHERE owner_id = $4
                  ))
              )
            ORDER BY ec.created_at DESC
            LIMIT 10
            "#,
        )
        .bind(org_id)
        .bind(period_start)
        .bind(period_end)
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        // Get notifications
        let notifications: Vec<DigestNotification> = sqlx::query_as(
            r#"
            SELECT title, message, notification_type, created_at
            FROM notifications
            WHERE organization_id = $1 AND user_id = $2
              AND created_at BETWEEN $3 AND $4
            ORDER BY created_at DESC
            LIMIT 20
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .bind(period_start)
        .bind(period_end)
        .fetch_all(&self.db)
        .await?;

        Ok(DigestContent {
            tasks_due,
            tasks_overdue,
            tasks_completed: tasks_completed as i32,
            mentions,
            comments,
            notifications,
        })
    }

    /// Create an email digest record
    pub async fn create_digest(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        digest_type: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        content: DigestContent,
    ) -> AppResult<EmailDigest> {
        let digest = sqlx::query_as::<_, EmailDigest>(
            r#"
            INSERT INTO email_digests (organization_id, user_id, digest_type, period_start, period_end,
                                       notification_count, task_count, comment_count, mention_count, content)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, organization_id, user_id, digest_type, period_start, period_end,
                      notification_count, task_count, comment_count, mention_count, content,
                      status, sent_at, error_message, created_at
            "#,
        )
        .bind(org_id)
        .bind(user_id)
        .bind(digest_type)
        .bind(period_start)
        .bind(period_end)
        .bind(content.notifications.len() as i32)
        .bind((content.tasks_due.len() + content.tasks_overdue.len()) as i32)
        .bind(content.comments.len() as i32)
        .bind(content.mentions.len() as i32)
        .bind(serde_json::to_value(&content).unwrap_or_default())
        .fetch_one(&self.db)
        .await?;

        Ok(digest)
    }

    /// Mark digest as sent
    pub async fn mark_digest_sent(&self, digest_id: Uuid) -> AppResult<()> {
        sqlx::query("UPDATE email_digests SET status = 'sent', sent_at = NOW() WHERE id = $1")
            .bind(digest_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Mark digest as failed
    pub async fn mark_digest_failed(&self, digest_id: Uuid, error: &str) -> AppResult<()> {
        sqlx::query(
            "UPDATE email_digests SET status = 'failed', error_message = $2 WHERE id = $1",
        )
        .bind(digest_id)
        .bind(error)
        .execute(&self.db)
        .await?;

        Ok(())
    }
}

/// Create mention notifications
pub async fn create_mention_notifications(
    notification_service: &NotificationService,
    org_id: Uuid,
    commenter_user_id: Uuid,
    commenter_name: &str,
    entity_type: &str,
    entity_id: Uuid,
    entity_title: &str,
    comment_id: Uuid,
    mentioned_user_ids: &[Uuid],
) -> AppResult<()> {
    for mentioned_user_id in mentioned_user_ids {
        // Don't notify self
        if *mentioned_user_id == commenter_user_id {
            continue;
        }

        let notification_data = serde_json::json!({
            "comment_id": comment_id,
            "entity_type": entity_type,
            "entity_id": entity_id,
            "mentioned_by": commenter_user_id,
        });

        notification_service
            .create_notification(
                org_id,
                CreateNotification {
                    user_id: *mentioned_user_id,
                    notification_type: "comment_mention".to_string(),
                    title: format!("{} mentioned you in {}", commenter_name, entity_title),
                    message: format!(
                        "You were mentioned in a comment on {} \"{}\".",
                        entity_type, entity_title
                    ),
                    data: Some(notification_data),
                },
            )
            .await?;
    }

    Ok(())
}
