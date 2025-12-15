use crate::cache::{org_cache_key, CacheClient};
use crate::utils::{AppError, AppResult};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(1800);
const CACHE_PREFIX_CAMPAIGN: &str = "access_review:campaign";
const CACHE_PREFIX_STATS: &str = "access_review:stats";

// ==================== Models ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccessReviewCampaign {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub due_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub integration_type: Option<String>,
    pub integration_id: Option<Uuid>,
    pub scope: Option<serde_json::Value>,
    pub review_type: Option<String>,
    pub reminder_sent_at: Option<DateTime<Utc>>,
    pub last_sync_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignWithStats {
    #[serde(flatten)]
    pub campaign: AccessReviewCampaign,
    pub total_items: i64,
    pub pending_items: i64,
    pub approved_items: i64,
    pub revoked_items: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccessReviewItem {
    pub id: Uuid,
    pub campaign_id: Uuid,
    pub user_identifier: String,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub access_details: Option<serde_json::Value>,
    pub reviewer_id: Option<Uuid>,
    pub review_status: Option<String>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub integration_user_id: Option<String>,
    pub department: Option<String>,
    pub manager: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub risk_level: Option<String>,
    pub mfa_enabled: Option<bool>,
    pub is_admin: Option<bool>,
    pub applications: Option<serde_json::Value>,
    pub removal_requested_at: Option<DateTime<Utc>>,
    pub removal_completed_at: Option<DateTime<Utc>>,
    pub removal_ticket_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AccessRemovalLog {
    pub id: Uuid,
    pub access_review_item_id: Uuid,
    pub campaign_id: Uuid,
    pub organization_id: Uuid,
    pub user_identifier: String,
    pub user_name: Option<String>,
    pub access_type: Option<String>,
    pub access_description: Option<String>,
    pub action: String,
    pub action_reason: Option<String>,
    pub requested_by: Option<Uuid>,
    pub requested_at: DateTime<Utc>,
    pub executed_by: Option<Uuid>,
    pub executed_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub error_message: Option<String>,
    pub external_ticket_id: Option<String>,
    pub external_ticket_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCampaign {
    pub name: String,
    pub description: Option<String>,
    pub due_at: Option<NaiveDate>,
    pub integration_type: Option<String>,
    pub integration_id: Option<Uuid>,
    pub scope: Option<serde_json::Value>,
    pub review_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCampaign {
    pub name: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub due_at: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateReviewItem {
    pub user_identifier: String,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub access_details: Option<serde_json::Value>,
    pub integration_user_id: Option<String>,
    pub department: Option<String>,
    pub manager: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub risk_level: Option<String>,
    pub mfa_enabled: Option<bool>,
    pub is_admin: Option<bool>,
    pub applications: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewDecision {
    pub status: String,  // 'approved', 'revoked', 'flagged'
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkReviewDecision {
    pub item_ids: Vec<Uuid>,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestRemoval {
    pub access_type: Option<String>,
    pub access_description: Option<String>,
    pub action: String,  // 'disabled', 'deleted', 'role_removed', 'downgraded'
    pub action_reason: Option<String>,
    pub external_ticket_id: Option<String>,
    pub external_ticket_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessReviewStats {
    pub total_campaigns: i64,
    pub active_campaigns: i64,
    pub completed_campaigns: i64,
    pub total_items: i64,
    pub pending_reviews: i64,
    pub approved_accesses: i64,
    pub revoked_accesses: i64,
    pub pending_removals: i64,
    pub completed_removals: i64,
    pub high_risk_users: i64,
    pub admin_users: i64,
    pub users_without_mfa: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListCampaignsQuery {
    pub status: Option<String>,
    pub integration_type: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItemsQuery {
    pub review_status: Option<String>,
    pub risk_level: Option<String>,
    pub is_admin: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ==================== Service ====================

#[derive(Clone)]
pub struct AccessReviewService {
    db: PgPool,
    cache: CacheClient,
}

impl AccessReviewService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Campaigns ====================

    /// List campaigns for an organization
    pub async fn list_campaigns(
        &self,
        org_id: Uuid,
        query: ListCampaignsQuery,
    ) -> AppResult<Vec<CampaignWithStats>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        let campaigns = sqlx::query_as::<_, AccessReviewCampaign>(
            r#"
            SELECT id, organization_id, name, description, status,
                   started_at, due_at, completed_at, created_by, created_at,
                   integration_type, integration_id, scope, review_type,
                   reminder_sent_at, last_sync_at
            FROM access_review_campaigns
            WHERE organization_id = $1
              AND ($2::text IS NULL OR status = $2)
              AND ($3::text IS NULL OR integration_type = $3)
            ORDER BY created_at DESC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(org_id)
        .bind(&query.status)
        .bind(&query.integration_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        // Get stats for each campaign
        let mut results = vec![];
        for campaign in campaigns {
            let (total, pending, approved, revoked): (i64, i64, i64, i64) = sqlx::query_as(
                r#"
                SELECT
                    COUNT(*) as total,
                    COUNT(*) FILTER (WHERE review_status IS NULL OR review_status = 'pending') as pending,
                    COUNT(*) FILTER (WHERE review_status = 'approved') as approved,
                    COUNT(*) FILTER (WHERE review_status = 'revoked') as revoked
                FROM access_review_items
                WHERE campaign_id = $1
                "#,
            )
            .bind(campaign.id)
            .fetch_one(&self.db)
            .await?;

            results.push(CampaignWithStats {
                campaign,
                total_items: total,
                pending_items: pending,
                approved_items: approved,
                revoked_items: revoked,
            });
        }

        Ok(results)
    }

    /// Get a single campaign
    pub async fn get_campaign(&self, org_id: Uuid, id: Uuid) -> AppResult<CampaignWithStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_CAMPAIGN, &id.to_string());

        if let Some(cached) = self.cache.get::<CampaignWithStats>(&cache_key).await? {
            return Ok(cached);
        }

        let campaign = sqlx::query_as::<_, AccessReviewCampaign>(
            r#"
            SELECT id, organization_id, name, description, status,
                   started_at, due_at, completed_at, created_by, created_at,
                   integration_type, integration_id, scope, review_type,
                   reminder_sent_at, last_sync_at
            FROM access_review_campaigns
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Campaign not found".to_string()))?;

        let (total, pending, approved, revoked): (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE review_status IS NULL OR review_status = 'pending') as pending,
                COUNT(*) FILTER (WHERE review_status = 'approved') as approved,
                COUNT(*) FILTER (WHERE review_status = 'revoked') as revoked
            FROM access_review_items
            WHERE campaign_id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        let result = CampaignWithStats {
            campaign,
            total_items: total,
            pending_items: pending,
            approved_items: approved,
            revoked_items: revoked,
        };

        self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await?;

        Ok(result)
    }

    /// Create a campaign
    pub async fn create_campaign(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        input: CreateCampaign,
    ) -> AppResult<AccessReviewCampaign> {
        if input.name.trim().is_empty() {
            return Err(AppError::ValidationError("Campaign name is required".to_string()));
        }

        let campaign = sqlx::query_as::<_, AccessReviewCampaign>(
            r#"
            INSERT INTO access_review_campaigns (
                organization_id, name, description, due_at, created_by,
                integration_type, integration_id, scope, review_type, status
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, 'draft')
            RETURNING id, organization_id, name, description, status,
                      started_at, due_at, completed_at, created_by, created_at,
                      integration_type, integration_id, scope, review_type,
                      reminder_sent_at, last_sync_at
            "#,
        )
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.due_at)
        .bind(user_id)
        .bind(&input.integration_type)
        .bind(input.integration_id)
        .bind(&input.scope)
        .bind(input.review_type.as_deref().unwrap_or("periodic"))
        .fetch_one(&self.db)
        .await?;

        self.invalidate_org_cache(org_id).await?;

        tracing::info!("Created access review campaign: {} ({})", campaign.name, campaign.id);

        Ok(campaign)
    }

    /// Update a campaign
    pub async fn update_campaign(
        &self,
        org_id: Uuid,
        id: Uuid,
        input: UpdateCampaign,
    ) -> AppResult<AccessReviewCampaign> {
        // Verify exists
        self.get_campaign(org_id, id).await?;

        let campaign = sqlx::query_as::<_, AccessReviewCampaign>(
            r#"
            UPDATE access_review_campaigns
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                status = COALESCE($5, status),
                due_at = COALESCE($6, due_at),
                started_at = CASE WHEN $5 = 'active' AND started_at IS NULL THEN NOW() ELSE started_at END,
                completed_at = CASE WHEN $5 = 'completed' AND completed_at IS NULL THEN NOW() ELSE completed_at END
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, name, description, status,
                      started_at, due_at, completed_at, created_by, created_at,
                      integration_type, integration_id, scope, review_type,
                      reminder_sent_at, last_sync_at
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.status)
        .bind(input.due_at)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_campaign_cache(org_id, id).await?;

        Ok(campaign)
    }

    /// Delete a campaign
    pub async fn delete_campaign(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        self.get_campaign(org_id, id).await?;

        sqlx::query("DELETE FROM access_review_campaigns WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        self.invalidate_campaign_cache(org_id, id).await?;

        tracing::info!("Deleted access review campaign: {}", id);

        Ok(())
    }

    // ==================== Review Items ====================

    /// List items for a campaign
    pub async fn list_items(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        query: ListItemsQuery,
    ) -> AppResult<Vec<AccessReviewItem>> {
        // Verify campaign exists
        self.get_campaign(org_id, campaign_id).await?;

        let limit = query.limit.unwrap_or(200).min(1000);
        let offset = query.offset.unwrap_or(0);

        let items = if let Some(ref search) = query.search {
            let search_pattern = format!("%{}%", search.to_lowercase());
            sqlx::query_as::<_, AccessReviewItem>(
                r#"
                SELECT *
                FROM access_review_items
                WHERE campaign_id = $1
                  AND (LOWER(user_identifier) LIKE $2 OR LOWER(COALESCE(user_name, '')) LIKE $2 OR LOWER(COALESCE(user_email, '')) LIKE $2)
                  AND ($3::text IS NULL OR COALESCE(review_status, 'pending') = $3)
                  AND ($4::text IS NULL OR risk_level = $4)
                  AND ($5::bool IS NULL OR is_admin = $5)
                ORDER BY
                    CASE risk_level WHEN 'high' THEN 1 WHEN 'medium' THEN 2 ELSE 3 END,
                    user_name ASC
                LIMIT $6 OFFSET $7
                "#,
            )
            .bind(campaign_id)
            .bind(&search_pattern)
            .bind(&query.review_status)
            .bind(&query.risk_level)
            .bind(query.is_admin)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, AccessReviewItem>(
                r#"
                SELECT *
                FROM access_review_items
                WHERE campaign_id = $1
                  AND ($2::text IS NULL OR COALESCE(review_status, 'pending') = $2)
                  AND ($3::text IS NULL OR risk_level = $3)
                  AND ($4::bool IS NULL OR is_admin = $4)
                ORDER BY
                    CASE risk_level WHEN 'high' THEN 1 WHEN 'medium' THEN 2 ELSE 3 END,
                    user_name ASC
                LIMIT $5 OFFSET $6
                "#,
            )
            .bind(campaign_id)
            .bind(&query.review_status)
            .bind(&query.risk_level)
            .bind(query.is_admin)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        };

        Ok(items)
    }

    /// Get a single item
    pub async fn get_item(&self, org_id: Uuid, campaign_id: Uuid, id: Uuid) -> AppResult<AccessReviewItem> {
        // Verify campaign exists
        self.get_campaign(org_id, campaign_id).await?;

        let item = sqlx::query_as::<_, AccessReviewItem>(
            "SELECT * FROM access_review_items WHERE id = $1 AND campaign_id = $2",
        )
        .bind(id)
        .bind(campaign_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Review item not found".to_string()))?;

        Ok(item)
    }

    /// Add items to a campaign (for manual creation or after sync)
    pub async fn add_items(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        items: Vec<CreateReviewItem>,
    ) -> AppResult<Vec<AccessReviewItem>> {
        // Verify campaign exists
        self.get_campaign(org_id, campaign_id).await?;

        let mut created = vec![];
        for input in items {
            let item = sqlx::query_as::<_, AccessReviewItem>(
                r#"
                INSERT INTO access_review_items (
                    campaign_id, user_identifier, user_name, user_email,
                    access_details, integration_user_id, department, manager,
                    last_login_at, risk_level, mfa_enabled, is_admin, applications
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                ON CONFLICT (campaign_id, user_identifier) DO UPDATE
                SET
                    user_name = EXCLUDED.user_name,
                    user_email = EXCLUDED.user_email,
                    access_details = EXCLUDED.access_details,
                    last_login_at = EXCLUDED.last_login_at,
                    risk_level = EXCLUDED.risk_level,
                    mfa_enabled = EXCLUDED.mfa_enabled,
                    is_admin = EXCLUDED.is_admin,
                    applications = EXCLUDED.applications
                RETURNING *
                "#,
            )
            .bind(campaign_id)
            .bind(&input.user_identifier)
            .bind(&input.user_name)
            .bind(&input.user_email)
            .bind(&input.access_details)
            .bind(&input.integration_user_id)
            .bind(&input.department)
            .bind(&input.manager)
            .bind(input.last_login_at)
            .bind(&input.risk_level)
            .bind(input.mfa_enabled)
            .bind(input.is_admin)
            .bind(&input.applications)
            .fetch_one(&self.db)
            .await?;

            created.push(item);
        }

        self.invalidate_campaign_cache(org_id, campaign_id).await?;

        Ok(created)
    }

    /// Review an item (approve/revoke/flag)
    pub async fn review_item(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        item_id: Uuid,
        reviewer_id: Uuid,
        decision: ReviewDecision,
    ) -> AppResult<AccessReviewItem> {
        // Verify item exists
        self.get_item(org_id, campaign_id, item_id).await?;

        let valid_statuses = ["approved", "revoked", "flagged"];
        if !valid_statuses.contains(&decision.status.as_str()) {
            return Err(AppError::ValidationError(
                "Status must be 'approved', 'revoked', or 'flagged'".to_string(),
            ));
        }

        let item = sqlx::query_as::<_, AccessReviewItem>(
            r#"
            UPDATE access_review_items
            SET
                review_status = $3,
                reviewer_id = $4,
                reviewed_at = NOW(),
                review_notes = $5
            WHERE id = $1 AND campaign_id = $2
            RETURNING *
            "#,
        )
        .bind(item_id)
        .bind(campaign_id)
        .bind(&decision.status)
        .bind(reviewer_id)
        .bind(&decision.notes)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_campaign_cache(org_id, campaign_id).await?;

        tracing::info!(
            "Reviewed access item {}: {} for user {}",
            item_id, decision.status, item.user_identifier
        );

        Ok(item)
    }

    /// Bulk review items
    pub async fn bulk_review(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        reviewer_id: Uuid,
        decision: BulkReviewDecision,
    ) -> AppResult<i64> {
        // Verify campaign exists
        self.get_campaign(org_id, campaign_id).await?;

        let valid_statuses = ["approved", "revoked", "flagged"];
        if !valid_statuses.contains(&decision.status.as_str()) {
            return Err(AppError::ValidationError(
                "Status must be 'approved', 'revoked', or 'flagged'".to_string(),
            ));
        }

        let result = sqlx::query(
            r#"
            UPDATE access_review_items
            SET
                review_status = $3,
                reviewer_id = $4,
                reviewed_at = NOW(),
                review_notes = $5
            WHERE campaign_id = $1 AND id = ANY($2)
            "#,
        )
        .bind(campaign_id)
        .bind(&decision.item_ids)
        .bind(&decision.status)
        .bind(reviewer_id)
        .bind(&decision.notes)
        .execute(&self.db)
        .await?;

        self.invalidate_campaign_cache(org_id, campaign_id).await?;

        let count = result.rows_affected() as i64;
        tracing::info!(
            "Bulk reviewed {} access items in campaign {}: {}",
            count, campaign_id, decision.status
        );

        Ok(count)
    }

    // ==================== Access Removal ====================

    /// Request access removal for a revoked item
    pub async fn request_removal(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        item_id: Uuid,
        requester_id: Uuid,
        input: RequestRemoval,
    ) -> AppResult<AccessRemovalLog> {
        let item = self.get_item(org_id, campaign_id, item_id).await?;

        // Update item with removal request
        sqlx::query(
            r#"
            UPDATE access_review_items
            SET
                removal_requested_at = NOW(),
                removal_ticket_id = $3
            WHERE id = $1 AND campaign_id = $2
            "#,
        )
        .bind(item_id)
        .bind(campaign_id)
        .bind(&input.external_ticket_id)
        .execute(&self.db)
        .await?;

        // Create removal log
        let log = sqlx::query_as::<_, AccessRemovalLog>(
            r#"
            INSERT INTO access_removal_logs (
                access_review_item_id, campaign_id, organization_id,
                user_identifier, user_name, access_type, access_description,
                action, action_reason, requested_by,
                external_ticket_id, external_ticket_url
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING *
            "#,
        )
        .bind(item_id)
        .bind(campaign_id)
        .bind(org_id)
        .bind(&item.user_identifier)
        .bind(&item.user_name)
        .bind(&input.access_type)
        .bind(&input.access_description)
        .bind(&input.action)
        .bind(&input.action_reason)
        .bind(requester_id)
        .bind(&input.external_ticket_id)
        .bind(&input.external_ticket_url)
        .fetch_one(&self.db)
        .await?;

        self.invalidate_campaign_cache(org_id, campaign_id).await?;

        tracing::info!(
            "Requested access removal for user {}: {}",
            item.user_identifier, input.action
        );

        Ok(log)
    }

    /// Mark removal as completed
    pub async fn complete_removal(
        &self,
        org_id: Uuid,
        log_id: Uuid,
        executor_id: Uuid,
        error_message: Option<String>,
    ) -> AppResult<AccessRemovalLog> {
        let status = if error_message.is_some() { "failed" } else { "completed" };

        let log = sqlx::query_as::<_, AccessRemovalLog>(
            r#"
            UPDATE access_removal_logs
            SET
                status = $3,
                executed_by = $4,
                executed_at = NOW(),
                error_message = $5
            WHERE id = $1 AND organization_id = $2
            RETURNING *
            "#,
        )
        .bind(log_id)
        .bind(org_id)
        .bind(status)
        .bind(executor_id)
        .bind(&error_message)
        .fetch_one(&self.db)
        .await?;

        // Update the review item if successful
        if error_message.is_none() {
            sqlx::query(
                "UPDATE access_review_items SET removal_completed_at = NOW() WHERE id = $1",
            )
            .bind(log.access_review_item_id)
            .execute(&self.db)
            .await?;
        }

        self.invalidate_org_cache(org_id).await?;

        tracing::info!(
            "Completed access removal {}: {} (status: {})",
            log_id, log.user_identifier, status
        );

        Ok(log)
    }

    /// Get removal logs for a campaign
    pub async fn get_removal_logs(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
    ) -> AppResult<Vec<AccessRemovalLog>> {
        let logs = sqlx::query_as::<_, AccessRemovalLog>(
            "SELECT * FROM access_removal_logs WHERE campaign_id = $1 AND organization_id = $2 ORDER BY requested_at DESC",
        )
        .bind(campaign_id)
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(logs)
    }

    // ==================== Statistics ====================

    /// Get access review statistics
    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<AccessReviewStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_STATS, "summary");

        if let Some(cached) = self.cache.get::<AccessReviewStats>(&cache_key).await? {
            return Ok(cached);
        }

        let (total_campaigns, active_campaigns, completed_campaigns): (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active,
                COUNT(*) FILTER (WHERE status = 'completed') as completed
            FROM access_review_campaigns
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let (total_items, pending, approved, revoked, high_risk, admins, no_mfa): (i64, i64, i64, i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE review_status IS NULL OR review_status = 'pending') as pending,
                COUNT(*) FILTER (WHERE review_status = 'approved') as approved,
                COUNT(*) FILTER (WHERE review_status = 'revoked') as revoked,
                COUNT(*) FILTER (WHERE risk_level = 'high') as high_risk,
                COUNT(*) FILTER (WHERE is_admin = true) as admins,
                COUNT(*) FILTER (WHERE mfa_enabled = false) as no_mfa
            FROM access_review_items ari
            JOIN access_review_campaigns arc ON ari.campaign_id = arc.id
            WHERE arc.organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let (pending_removals, completed_removals): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) FILTER (WHERE status = 'pending' OR status = 'in_progress') as pending,
                COUNT(*) FILTER (WHERE status = 'completed') as completed
            FROM access_removal_logs
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let stats = AccessReviewStats {
            total_campaigns,
            active_campaigns,
            completed_campaigns,
            total_items,
            pending_reviews: pending,
            approved_accesses: approved,
            revoked_accesses: revoked,
            pending_removals,
            completed_removals,
            high_risk_users: high_risk,
            admin_users: admins,
            users_without_mfa: no_mfa,
        };

        self.cache
            .set(&cache_key, &stats, Some(Duration::from_secs(300)))
            .await?;

        Ok(stats)
    }

    // ==================== Integration Sync ====================

    /// Sync users from an integration into a campaign
    pub async fn sync_from_integration(
        &self,
        org_id: Uuid,
        campaign_id: Uuid,
        integration_service: &crate::services::IntegrationService,
    ) -> AppResult<i64> {
        let campaign = self.get_campaign(org_id, campaign_id).await?;

        let integration_id = campaign.campaign.integration_id
            .ok_or_else(|| AppError::BadRequest("Campaign has no integration configured".to_string()))?;

        let integration_type = campaign.campaign.integration_type.as_deref()
            .ok_or_else(|| AppError::BadRequest("Campaign has no integration type".to_string()))?;

        // Get users based on integration type
        let items = match integration_type {
            "okta" => self.get_okta_users(org_id, integration_id, integration_service).await?,
            "google_workspace" => self.get_google_users(org_id, integration_id, integration_service).await?,
            "azure_ad" => self.get_azure_users(org_id, integration_id, integration_service).await?,
            "github" => self.get_github_users(org_id, integration_id, integration_service).await?,
            _ => return Err(AppError::BadRequest(format!(
                "Unsupported integration type for access reviews: {}", integration_type
            ))),
        };

        // Add/update items
        let created = self.add_items(org_id, campaign_id, items).await?;

        // Update campaign last_sync_at
        sqlx::query("UPDATE access_review_campaigns SET last_sync_at = NOW() WHERE id = $1")
            .bind(campaign_id)
            .execute(&self.db)
            .await?;

        self.invalidate_campaign_cache(org_id, campaign_id).await?;

        Ok(created.len() as i64)
    }

    async fn get_okta_users(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        _integration_service: &crate::services::IntegrationService,
    ) -> AppResult<Vec<CreateReviewItem>> {
        // Query synced Okta users from database
        let users: Vec<(String, String, String, Option<DateTime<Utc>>, bool, Option<String>)> = sqlx::query_as(
            r#"
            SELECT
                ou.okta_user_id,
                ou.email,
                COALESCE(ou.first_name || ' ' || ou.last_name, ou.email) as name,
                ou.last_login,
                ou.mfa_enrolled,
                ou.department
            FROM okta_users ou
            WHERE ou.integration_id = $1
              AND ou.status = 'ACTIVE'
            "#,
        )
        .bind(integration_id)
        .fetch_all(&self.db)
        .await?;

        // Also get stale users (high risk)
        let stale_threshold = chrono::Utc::now() - chrono::Duration::days(90);

        Ok(users.into_iter().map(|(id, email, name, last_login, mfa, dept)| {
            let is_stale = last_login.map(|l| l < stale_threshold).unwrap_or(true);
            CreateReviewItem {
                user_identifier: email.clone(),
                user_name: Some(name),
                user_email: Some(email),
                access_details: None,
                integration_user_id: Some(id),
                department: dept,
                manager: None,
                last_login_at: last_login,
                risk_level: Some(if is_stale { "high".to_string() } else if !mfa { "medium".to_string() } else { "low".to_string() }),
                mfa_enabled: Some(mfa),
                is_admin: Some(false), // Would need to check Okta groups for admin status
                applications: None,
            }
        }).collect())
    }

    async fn get_google_users(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        _integration_service: &crate::services::IntegrationService,
    ) -> AppResult<Vec<CreateReviewItem>> {
        let users: Vec<(String, String, Option<DateTime<Utc>>, bool, bool, Option<String>)> = sqlx::query_as(
            r#"
            SELECT
                gu.google_user_id,
                gu.email,
                gu.last_login_time,
                COALESCE(gu.two_step_enrolled, false),
                COALESCE(gu.is_admin, false),
                gu.org_unit_path
            FROM google_workspace_users gu
            WHERE gu.integration_id = $1
              AND gu.suspended = false
            "#,
        )
        .bind(integration_id)
        .fetch_all(&self.db)
        .await?;

        let stale_threshold = chrono::Utc::now() - chrono::Duration::days(90);

        Ok(users.into_iter().map(|(id, email, last_login, mfa, is_admin, org_unit)| {
            let is_stale = last_login.map(|l| l < stale_threshold).unwrap_or(true);
            CreateReviewItem {
                user_identifier: email.clone(),
                user_name: None,
                user_email: Some(email),
                access_details: None,
                integration_user_id: Some(id),
                department: org_unit,
                manager: None,
                last_login_at: last_login,
                risk_level: Some(if is_admin { "high".to_string() } else if is_stale { "high".to_string() } else if !mfa { "medium".to_string() } else { "low".to_string() }),
                mfa_enabled: Some(mfa),
                is_admin: Some(is_admin),
                applications: None,
            }
        }).collect())
    }

    async fn get_azure_users(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        _integration_service: &crate::services::IntegrationService,
    ) -> AppResult<Vec<CreateReviewItem>> {
        let users: Vec<(String, String, Option<String>, Option<DateTime<Utc>>, Option<String>, bool)> = sqlx::query_as(
            r#"
            SELECT
                au.azure_user_id,
                au.user_principal_name,
                au.display_name,
                au.last_sign_in,
                au.department,
                COALESCE(au.account_enabled, true)
            FROM azure_ad_users au
            WHERE au.integration_id = $1
              AND COALESCE(au.account_enabled, true) = true
            "#,
        )
        .bind(integration_id)
        .fetch_all(&self.db)
        .await?;

        let stale_threshold = chrono::Utc::now() - chrono::Duration::days(90);

        Ok(users.into_iter().map(|(id, upn, name, last_login, dept, _enabled)| {
            let is_stale = last_login.map(|l| l < stale_threshold).unwrap_or(true);
            let is_guest = upn.contains("#EXT#");
            CreateReviewItem {
                user_identifier: upn.clone(),
                user_name: name,
                user_email: Some(upn),
                access_details: None,
                integration_user_id: Some(id),
                department: dept,
                manager: None,
                last_login_at: last_login,
                risk_level: Some(if is_guest { "high".to_string() } else if is_stale { "high".to_string() } else { "low".to_string() }),
                mfa_enabled: None, // Would need separate MFA status query
                is_admin: Some(false),
                applications: None,
            }
        }).collect())
    }

    async fn get_github_users(
        &self,
        _org_id: Uuid,
        integration_id: Uuid,
        _integration_service: &crate::services::IntegrationService,
    ) -> AppResult<Vec<CreateReviewItem>> {
        let members: Vec<(String, String, Option<String>, Option<bool>)> = sqlx::query_as(
            r#"
            SELECT
                gm.github_user_id,
                gm.login,
                gm.role,
                gm.two_factor_enabled
            FROM github_members gm
            WHERE gm.integration_id = $1
            "#,
        )
        .bind(integration_id)
        .fetch_all(&self.db)
        .await?;

        Ok(members.into_iter().map(|(id, login, role, mfa)| {
            let is_admin = role.as_deref() == Some("admin");
            let mfa_enabled = mfa.unwrap_or(false);
            CreateReviewItem {
                user_identifier: login.clone(),
                user_name: Some(login.clone()),
                user_email: None,
                access_details: Some(serde_json::json!({ "role": role })),
                integration_user_id: Some(id),
                department: None,
                manager: None,
                last_login_at: None,
                risk_level: Some(if is_admin && !mfa_enabled { "high".to_string() } else if is_admin { "medium".to_string() } else { "low".to_string() }),
                mfa_enabled: Some(mfa_enabled),
                is_admin: Some(is_admin),
                applications: None,
            }
        }).collect())
    }

    // ==================== Cache Helpers ====================

    async fn invalidate_campaign_cache(&self, org_id: Uuid, campaign_id: Uuid) -> AppResult<()> {
        let key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_CAMPAIGN, &campaign_id.to_string());
        self.cache.delete(&key).await?;
        self.invalidate_org_cache(org_id).await
    }

    async fn invalidate_org_cache(&self, org_id: Uuid) -> AppResult<()> {
        let stats_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_STATS, "summary");
        self.cache.delete(&stats_key).await
    }
}
