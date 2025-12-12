use crate::cache::{org_cache_key, CacheClient};
use crate::models::{
    CategoryCount, CreatePolicy, ListPoliciesQuery, Policy, PolicyAcknowledgment,
    PolicyStats, PolicyVersion, PolicyWithStats, UpdatePolicy,
};
use crate::utils::{AppError, AppResult};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(1800); // 30 minutes
const CACHE_PREFIX_POLICY: &str = "policy";
const CACHE_PREFIX_POLICY_STATS: &str = "policy:stats";

#[derive(Clone)]
pub struct PolicyService {
    db: PgPool,
    cache: CacheClient,
}

impl PolicyService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Policy CRUD ====================

    /// List policies for an organization with filtering
    pub async fn list_policies(
        &self,
        org_id: Uuid,
        query: ListPoliciesQuery,
    ) -> AppResult<Vec<PolicyWithStats>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        let policies = if let Some(ref search) = query.search {
            let search_pattern = format!("%{}%", search.to_lowercase());
            sqlx::query_as::<_, Policy>(
                r#"
                SELECT id, organization_id, code, title, category, content, version, status,
                       owner_id, approver_id, approved_at, effective_date, review_date,
                       created_at, updated_at
                FROM policies
                WHERE organization_id = $1
                  AND (LOWER(title) LIKE $2 OR LOWER(code) LIKE $2 OR LOWER(content) LIKE $2)
                  AND ($3::text IS NULL OR status = $3)
                  AND ($4::text IS NULL OR category = $4)
                  AND ($5::uuid IS NULL OR owner_id = $5)
                  AND ($6::bool IS NULL OR ($6 = true AND review_date <= CURRENT_DATE))
                ORDER BY updated_at DESC
                LIMIT $7 OFFSET $8
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .bind(&query.status)
            .bind(&query.category)
            .bind(query.owner_id)
            .bind(query.needs_review)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, Policy>(
                r#"
                SELECT id, organization_id, code, title, category, content, version, status,
                       owner_id, approver_id, approved_at, effective_date, review_date,
                       created_at, updated_at
                FROM policies
                WHERE organization_id = $1
                  AND ($2::text IS NULL OR status = $2)
                  AND ($3::text IS NULL OR category = $3)
                  AND ($4::uuid IS NULL OR owner_id = $4)
                  AND ($5::bool IS NULL OR ($5 = true AND review_date <= CURRENT_DATE))
                ORDER BY updated_at DESC
                LIMIT $6 OFFSET $7
                "#,
            )
            .bind(org_id)
            .bind(&query.status)
            .bind(&query.category)
            .bind(query.owner_id)
            .bind(query.needs_review)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        };

        // Get acknowledgment counts for all policies in one query
        let policy_ids: Vec<Uuid> = policies.iter().map(|p| p.id).collect();

        let ack_counts: Vec<(Uuid, i64, i64)> = if !policy_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT
                    p.id,
                    COUNT(pa.id) as acknowledged,
                    COALESCE((
                        SELECT COUNT(*) FROM users u
                        WHERE u.organization_id = p.organization_id
                    ), 0) - COUNT(pa.id) as pending
                FROM policies p
                LEFT JOIN policy_acknowledgments pa ON p.id = pa.policy_id AND pa.policy_version = p.version
                WHERE p.id = ANY($1)
                GROUP BY p.id, p.organization_id
                "#,
            )
            .bind(&policy_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let ack_map: std::collections::HashMap<Uuid, (i64, i64)> =
            ack_counts.into_iter().map(|(id, ack, pending)| (id, (ack, pending))).collect();

        let result: Vec<PolicyWithStats> = policies
            .into_iter()
            .map(|policy| {
                let (ack_count, pending) = ack_map.get(&policy.id).copied().unwrap_or((0, 0));
                PolicyWithStats {
                    policy,
                    acknowledgment_count: ack_count,
                    pending_acknowledgments: pending.max(0),
                }
            })
            .collect();

        Ok(result)
    }

    /// Get a single policy by ID
    pub async fn get_policy(&self, org_id: Uuid, id: Uuid) -> AppResult<PolicyWithStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_POLICY, &id.to_string());

        // Try cache first
        if let Some(cached) = self.cache.get::<PolicyWithStats>(&cache_key).await? {
            tracing::debug!("Cache hit for policy {}", id);
            return Ok(cached);
        }

        let policy = sqlx::query_as::<_, Policy>(
            r#"
            SELECT id, organization_id, code, title, category, content, version, status,
                   owner_id, approver_id, approved_at, effective_date, review_date,
                   created_at, updated_at
            FROM policies
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Policy {} not found", id)))?;

        // Get acknowledgment stats
        let (ack_count, total_users): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                (SELECT COUNT(*) FROM policy_acknowledgments WHERE policy_id = $1 AND policy_version = $2),
                (SELECT COUNT(*) FROM users WHERE organization_id = $3)
            "#,
        )
        .bind(id)
        .bind(policy.version)
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let result = PolicyWithStats {
            policy,
            acknowledgment_count: ack_count,
            pending_acknowledgments: (total_users - ack_count).max(0),
        };

        // Cache the result
        self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await?;

        Ok(result)
    }

    /// Create a new policy
    pub async fn create_policy(
        &self,
        org_id: Uuid,
        input: CreatePolicy,
    ) -> AppResult<Policy> {
        Policy::validate_create(&input).map_err(AppError::ValidationError)?;

        let policy = sqlx::query_as::<_, Policy>(
            r#"
            INSERT INTO policies (organization_id, code, title, category, content, owner_id,
                                  effective_date, review_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, organization_id, code, title, category, content, version, status,
                      owner_id, approver_id, approved_at, effective_date, review_date,
                      created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(&input.code)
        .bind(&input.title)
        .bind(&input.category)
        .bind(&input.content)
        .bind(input.owner_id)
        .bind(input.effective_date)
        .bind(input.review_date)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_org_policy_caches(org_id).await?;

        tracing::info!("Created policy: {} ({})", policy.code, policy.id);

        Ok(policy)
    }

    /// Update a policy (creates new version if content changes)
    pub async fn update_policy(
        &self,
        org_id: Uuid,
        id: Uuid,
        user_id: Option<Uuid>,
        input: UpdatePolicy,
    ) -> AppResult<Policy> {
        // Verify policy exists
        let existing = self.get_policy(org_id, id).await?;

        // Check if content is changing - if so, create a version entry
        let content_changed = input.content.is_some()
            && input.content.as_ref() != existing.policy.content.as_ref();

        let mut tx = self.db.begin().await?;

        // If content changed, create version history entry and increment version
        let new_version = if content_changed {
            // Save current version to history
            sqlx::query(
                r#"
                INSERT INTO policy_versions (policy_id, version, content, changed_by, change_summary)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(id)
            .bind(existing.policy.version)
            .bind(&existing.policy.content)
            .bind(user_id)
            .bind(&input.change_summary)
            .execute(&mut *tx)
            .await?;

            existing.policy.version + 1
        } else {
            existing.policy.version
        };

        let policy = sqlx::query_as::<_, Policy>(
            r#"
            UPDATE policies
            SET
                code = COALESCE($3, code),
                title = COALESCE($4, title),
                category = COALESCE($5, category),
                content = COALESCE($6, content),
                status = COALESCE($7, status),
                owner_id = COALESCE($8, owner_id),
                effective_date = COALESCE($9, effective_date),
                review_date = COALESCE($10, review_date),
                version = $11,
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, code, title, category, content, version, status,
                      owner_id, approver_id, approved_at, effective_date, review_date,
                      created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(&input.code)
        .bind(&input.title)
        .bind(&input.category)
        .bind(&input.content)
        .bind(&input.status)
        .bind(input.owner_id)
        .bind(input.effective_date)
        .bind(input.review_date)
        .bind(new_version)
        .fetch_one(&mut *tx)
        .await?;

        tx.commit().await?;

        // Invalidate caches
        self.invalidate_policy_cache(org_id, id).await?;

        tracing::info!("Updated policy: {} ({})", policy.code, policy.id);

        Ok(policy)
    }

    /// Delete a policy
    pub async fn delete_policy(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        // Verify policy exists
        self.get_policy(org_id, id).await?;

        sqlx::query("DELETE FROM policies WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        // Invalidate caches
        self.invalidate_policy_cache(org_id, id).await?;

        tracing::info!("Deleted policy: {}", id);

        Ok(())
    }

    // ==================== Policy Versions ====================

    /// Get version history for a policy
    pub async fn get_versions(&self, org_id: Uuid, policy_id: Uuid) -> AppResult<Vec<PolicyVersion>> {
        // Verify policy exists
        self.get_policy(org_id, policy_id).await?;

        let versions = sqlx::query_as::<_, PolicyVersion>(
            r#"
            SELECT id, policy_id, version, content, changed_by, change_summary, created_at
            FROM policy_versions
            WHERE policy_id = $1
            ORDER BY version DESC
            "#,
        )
        .bind(policy_id)
        .fetch_all(&self.db)
        .await?;

        Ok(versions)
    }

    // ==================== Policy Acknowledgments ====================

    /// Acknowledge a policy
    pub async fn acknowledge_policy(
        &self,
        org_id: Uuid,
        policy_id: Uuid,
        user_id: Uuid,
        ip_address: Option<String>,
    ) -> AppResult<PolicyAcknowledgment> {
        // Get policy and its current version
        let policy = self.get_policy(org_id, policy_id).await?;

        // Check if already acknowledged this version
        let existing: Option<PolicyAcknowledgment> = sqlx::query_as(
            r#"
            SELECT id, policy_id, policy_version, user_id, acknowledged_at, ip_address
            FROM policy_acknowledgments
            WHERE policy_id = $1 AND user_id = $2 AND policy_version = $3
            "#,
        )
        .bind(policy_id)
        .bind(user_id)
        .bind(policy.policy.version)
        .fetch_optional(&self.db)
        .await?;

        if let Some(ack) = existing {
            return Ok(ack);
        }

        let acknowledgment = sqlx::query_as::<_, PolicyAcknowledgment>(
            r#"
            INSERT INTO policy_acknowledgments (policy_id, policy_version, user_id, ip_address)
            VALUES ($1, $2, $3, $4)
            RETURNING id, policy_id, policy_version, user_id, acknowledged_at, ip_address
            "#,
        )
        .bind(policy_id)
        .bind(policy.policy.version)
        .bind(user_id)
        .bind(ip_address)
        .fetch_one(&self.db)
        .await?;

        // Invalidate cache
        self.invalidate_policy_cache(org_id, policy_id).await?;

        tracing::info!(
            "User {} acknowledged policy {} version {}",
            user_id,
            policy_id,
            policy.policy.version
        );

        Ok(acknowledgment)
    }

    /// Get acknowledgments for a policy
    pub async fn get_acknowledgments(
        &self,
        org_id: Uuid,
        policy_id: Uuid,
    ) -> AppResult<Vec<PolicyAcknowledgment>> {
        // Verify policy exists
        self.get_policy(org_id, policy_id).await?;

        let acknowledgments = sqlx::query_as::<_, PolicyAcknowledgment>(
            r#"
            SELECT id, policy_id, policy_version, user_id, acknowledged_at, ip_address
            FROM policy_acknowledgments
            WHERE policy_id = $1
            ORDER BY acknowledged_at DESC
            "#,
        )
        .bind(policy_id)
        .fetch_all(&self.db)
        .await?;

        Ok(acknowledgments)
    }

    // ==================== Statistics ====================

    /// Get policy statistics
    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<PolicyStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_POLICY_STATS, "summary");

        // Try cache first
        if let Some(cached) = self.cache.get::<PolicyStats>(&cache_key).await? {
            tracing::debug!("Cache hit for policy stats");
            return Ok(cached);
        }

        // Get counts by status
        let (total, published, draft, pending_approval, needs_review): (i64, i64, i64, i64, i64) =
            sqlx::query_as(
                r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'published') as published,
                COUNT(*) FILTER (WHERE status = 'draft') as draft,
                COUNT(*) FILTER (WHERE status = 'pending_approval') as pending_approval,
                COUNT(*) FILTER (WHERE review_date IS NOT NULL AND review_date <= CURRENT_DATE) as needs_review
            FROM policies
            WHERE organization_id = $1
            "#,
            )
            .bind(org_id)
            .fetch_one(&self.db)
            .await?;

        // Get by category
        let by_category: Vec<CategoryCount> = sqlx::query_as(
            r#"
            SELECT category, COUNT(*) as count
            FROM policies
            WHERE organization_id = $1
            GROUP BY category
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let stats = PolicyStats {
            total,
            published,
            draft,
            pending_approval,
            needs_review,
            by_category,
        };

        // Cache for 5 minutes
        self.cache
            .set(&cache_key, &stats, Some(Duration::from_secs(300)))
            .await?;

        Ok(stats)
    }

    // ==================== Cache Invalidation ====================

    async fn invalidate_policy_cache(&self, org_id: Uuid, policy_id: Uuid) -> AppResult<()> {
        let cache_key = org_cache_key(
            &org_id.to_string(),
            CACHE_PREFIX_POLICY,
            &policy_id.to_string(),
        );
        self.cache.delete(&cache_key).await?;

        self.invalidate_org_policy_caches(org_id).await
    }

    async fn invalidate_org_policy_caches(&self, org_id: Uuid) -> AppResult<()> {
        let stats_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_POLICY_STATS, "summary");
        self.cache.delete(&stats_key).await
    }
}
