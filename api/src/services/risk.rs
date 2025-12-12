use crate::cache::{org_cache_key, CacheClient};
use crate::models::{
    CreateRisk, LinkedControlSummary, ListRisksQuery, Risk, RiskCategoryCount,
    RiskControlMapping, RiskStats, RiskWithControls, StatusCount, UpdateRisk,
};
use crate::utils::{AppError, AppResult};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(1800); // 30 minutes
const CACHE_PREFIX_RISK: &str = "risk";
const CACHE_PREFIX_RISK_STATS: &str = "risk:stats";

#[derive(Clone)]
pub struct RiskService {
    db: PgPool,
    cache: CacheClient,
}

impl RiskService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Risk CRUD ====================

    /// List risks for an organization with filtering
    pub async fn list_risks(
        &self,
        org_id: Uuid,
        query: ListRisksQuery,
    ) -> AppResult<Vec<RiskWithControls>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        let risks = if let Some(ref search) = query.search {
            let search_pattern = format!("%{}%", search.to_lowercase());
            sqlx::query_as::<_, Risk>(
                r#"
                SELECT id, organization_id, code, title, description, category, source,
                       likelihood, impact, inherent_score, residual_likelihood, residual_impact,
                       residual_score, status, owner_id, treatment_plan, identified_at,
                       review_date, created_at, updated_at
                FROM risks
                WHERE organization_id = $1
                  AND (LOWER(title) LIKE $2 OR LOWER(code) LIKE $2 OR LOWER(description) LIKE $2)
                  AND ($3::text IS NULL OR status = $3)
                  AND ($4::text IS NULL OR category = $4)
                  AND ($5::text IS NULL OR source = $5)
                  AND ($6::uuid IS NULL OR owner_id = $6)
                  AND ($7::int IS NULL OR inherent_score >= $7)
                  AND ($8::int IS NULL OR inherent_score <= $8)
                  AND ($9::bool IS NULL OR ($9 = true AND review_date <= CURRENT_DATE))
                ORDER BY inherent_score DESC NULLS LAST, updated_at DESC
                LIMIT $10 OFFSET $11
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .bind(&query.status)
            .bind(&query.category)
            .bind(&query.source)
            .bind(query.owner_id)
            .bind(query.min_score)
            .bind(query.max_score)
            .bind(query.needs_review)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, Risk>(
                r#"
                SELECT id, organization_id, code, title, description, category, source,
                       likelihood, impact, inherent_score, residual_likelihood, residual_impact,
                       residual_score, status, owner_id, treatment_plan, identified_at,
                       review_date, created_at, updated_at
                FROM risks
                WHERE organization_id = $1
                  AND ($2::text IS NULL OR status = $2)
                  AND ($3::text IS NULL OR category = $3)
                  AND ($4::text IS NULL OR source = $4)
                  AND ($5::uuid IS NULL OR owner_id = $5)
                  AND ($6::int IS NULL OR inherent_score >= $6)
                  AND ($7::int IS NULL OR inherent_score <= $7)
                  AND ($8::bool IS NULL OR ($8 = true AND review_date <= CURRENT_DATE))
                ORDER BY inherent_score DESC NULLS LAST, updated_at DESC
                LIMIT $9 OFFSET $10
                "#,
            )
            .bind(org_id)
            .bind(&query.status)
            .bind(&query.category)
            .bind(&query.source)
            .bind(query.owner_id)
            .bind(query.min_score)
            .bind(query.max_score)
            .bind(query.needs_review)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        };

        // Get linked control counts in one query
        let risk_ids: Vec<Uuid> = risks.iter().map(|r| r.id).collect();

        let counts: Vec<(Uuid, i64)> = if !risk_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT risk_id, COUNT(*) as count
                FROM risk_control_mappings
                WHERE risk_id = ANY($1)
                GROUP BY risk_id
                "#,
            )
            .bind(&risk_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let count_map: std::collections::HashMap<Uuid, i64> = counts.into_iter().collect();

        let result: Vec<RiskWithControls> = risks
            .into_iter()
            .map(|risk| {
                let count = count_map.get(&risk.id).copied().unwrap_or(0);
                RiskWithControls {
                    risk,
                    linked_control_count: count,
                    linked_controls: None,
                }
            })
            .collect();

        Ok(result)
    }

    /// Get a single risk by ID with linked controls
    pub async fn get_risk(&self, org_id: Uuid, id: Uuid) -> AppResult<RiskWithControls> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_RISK, &id.to_string());

        // Try cache first
        if let Some(cached) = self.cache.get::<RiskWithControls>(&cache_key).await? {
            tracing::debug!("Cache hit for risk {}", id);
            return Ok(cached);
        }

        let risk = sqlx::query_as::<_, Risk>(
            r#"
            SELECT id, organization_id, code, title, description, category, source,
                   likelihood, impact, inherent_score, residual_likelihood, residual_impact,
                   residual_score, status, owner_id, treatment_plan, identified_at,
                   review_date, created_at, updated_at
            FROM risks
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Risk {} not found", id)))?;

        // Get linked controls
        let linked_controls = sqlx::query_as::<_, LinkedControlSummary>(
            r#"
            SELECT c.id, c.code, c.name, rcm.effectiveness
            FROM risk_control_mappings rcm
            JOIN controls c ON rcm.control_id = c.id
            WHERE rcm.risk_id = $1
            ORDER BY c.code
            "#,
        )
        .bind(id)
        .fetch_all(&self.db)
        .await?;

        let linked_control_count = linked_controls.len() as i64;

        let result = RiskWithControls {
            risk,
            linked_control_count,
            linked_controls: Some(linked_controls),
        };

        // Cache the result
        self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await?;

        Ok(result)
    }

    /// Create a new risk
    pub async fn create_risk(&self, org_id: Uuid, input: CreateRisk) -> AppResult<Risk> {
        Risk::validate_create(&input).map_err(AppError::ValidationError)?;

        // Calculate inherent score
        let inherent_score = Risk::calculate_inherent_score(input.likelihood, input.impact);

        let risk = sqlx::query_as::<_, Risk>(
            r#"
            INSERT INTO risks (organization_id, code, title, description, category, source,
                               likelihood, impact, inherent_score, owner_id, treatment_plan, review_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, organization_id, code, title, description, category, source,
                      likelihood, impact, inherent_score, residual_likelihood, residual_impact,
                      residual_score, status, owner_id, treatment_plan, identified_at,
                      review_date, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(&input.code)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.category)
        .bind(&input.source)
        .bind(input.likelihood)
        .bind(input.impact)
        .bind(inherent_score)
        .bind(input.owner_id)
        .bind(&input.treatment_plan)
        .bind(input.review_date)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_org_risk_caches(org_id).await?;

        tracing::info!("Created risk: {} ({})", risk.code, risk.id);

        Ok(risk)
    }

    /// Update a risk
    pub async fn update_risk(
        &self,
        org_id: Uuid,
        id: Uuid,
        input: UpdateRisk,
    ) -> AppResult<Risk> {
        // Verify risk exists
        let existing = self.get_risk(org_id, id).await?;

        // Calculate scores if likelihood/impact changed
        let new_likelihood = input.likelihood.or(existing.risk.likelihood);
        let new_impact = input.impact.or(existing.risk.impact);
        let inherent_score = Risk::calculate_inherent_score(new_likelihood, new_impact);

        let new_residual_likelihood = input.residual_likelihood.or(existing.risk.residual_likelihood);
        let new_residual_impact = input.residual_impact.or(existing.risk.residual_impact);
        let residual_score = Risk::calculate_residual_score(new_residual_likelihood, new_residual_impact);

        let risk = sqlx::query_as::<_, Risk>(
            r#"
            UPDATE risks
            SET
                code = COALESCE($3, code),
                title = COALESCE($4, title),
                description = COALESCE($5, description),
                category = COALESCE($6, category),
                source = COALESCE($7, source),
                likelihood = COALESCE($8, likelihood),
                impact = COALESCE($9, impact),
                inherent_score = $10,
                residual_likelihood = COALESCE($11, residual_likelihood),
                residual_impact = COALESCE($12, residual_impact),
                residual_score = $13,
                status = COALESCE($14, status),
                owner_id = COALESCE($15, owner_id),
                treatment_plan = COALESCE($16, treatment_plan),
                review_date = COALESCE($17, review_date),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, code, title, description, category, source,
                      likelihood, impact, inherent_score, residual_likelihood, residual_impact,
                      residual_score, status, owner_id, treatment_plan, identified_at,
                      review_date, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(&input.code)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.category)
        .bind(&input.source)
        .bind(input.likelihood)
        .bind(input.impact)
        .bind(inherent_score)
        .bind(input.residual_likelihood)
        .bind(input.residual_impact)
        .bind(residual_score)
        .bind(&input.status)
        .bind(input.owner_id)
        .bind(&input.treatment_plan)
        .bind(input.review_date)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_risk_cache(org_id, id).await?;

        tracing::info!("Updated risk: {} ({})", risk.code, risk.id);

        Ok(risk)
    }

    /// Delete a risk
    pub async fn delete_risk(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        // Verify risk exists
        self.get_risk(org_id, id).await?;

        sqlx::query("DELETE FROM risks WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        // Invalidate caches
        self.invalidate_risk_cache(org_id, id).await?;

        tracing::info!("Deleted risk: {}", id);

        Ok(())
    }

    // ==================== Control Mappings ====================

    /// Link controls to a risk
    pub async fn link_controls(
        &self,
        org_id: Uuid,
        risk_id: Uuid,
        control_ids: Vec<Uuid>,
        effectiveness: Option<String>,
    ) -> AppResult<Vec<RiskControlMapping>> {
        // Verify risk exists
        self.get_risk(org_id, risk_id).await?;

        let mut tx = self.db.begin().await?;
        let mut mappings = Vec::new();

        for control_id in control_ids {
            // Check if link already exists
            let existing: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM risk_control_mappings WHERE risk_id = $1 AND control_id = $2",
            )
            .bind(risk_id)
            .bind(control_id)
            .fetch_optional(&mut *tx)
            .await?;

            if existing.is_none() {
                let mapping = sqlx::query_as::<_, RiskControlMapping>(
                    r#"
                    INSERT INTO risk_control_mappings (risk_id, control_id, effectiveness)
                    VALUES ($1, $2, $3)
                    RETURNING id, risk_id, control_id, effectiveness, created_at
                    "#,
                )
                .bind(risk_id)
                .bind(control_id)
                .bind(&effectiveness)
                .fetch_one(&mut *tx)
                .await?;

                mappings.push(mapping);
            }
        }

        tx.commit().await?;

        // Invalidate risk cache
        self.invalidate_risk_cache(org_id, risk_id).await?;

        tracing::info!("Linked {} controls to risk {}", mappings.len(), risk_id);

        Ok(mappings)
    }

    /// Unlink controls from a risk
    pub async fn unlink_controls(
        &self,
        org_id: Uuid,
        risk_id: Uuid,
        control_ids: Vec<Uuid>,
    ) -> AppResult<i64> {
        // Verify risk exists
        self.get_risk(org_id, risk_id).await?;

        let result = sqlx::query(
            "DELETE FROM risk_control_mappings WHERE risk_id = $1 AND control_id = ANY($2)",
        )
        .bind(risk_id)
        .bind(&control_ids)
        .execute(&self.db)
        .await?;

        let deleted = result.rows_affected() as i64;

        // Invalidate risk cache
        self.invalidate_risk_cache(org_id, risk_id).await?;

        tracing::info!("Unlinked {} controls from risk {}", deleted, risk_id);

        Ok(deleted)
    }

    // ==================== Statistics ====================

    /// Get risk statistics
    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<RiskStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_RISK_STATS, "summary");

        // Try cache first
        if let Some(cached) = self.cache.get::<RiskStats>(&cache_key).await? {
            tracing::debug!("Cache hit for risk stats");
            return Ok(cached);
        }

        // Get basic counts
        let (total, high_risks, medium_risks, low_risks, needs_review, avg_inherent, avg_residual): (
            i64,
            i64,
            i64,
            i64,
            i64,
            Option<f64>,
            Option<f64>,
        ) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE inherent_score >= 15) as high_risks,
                COUNT(*) FILTER (WHERE inherent_score >= 5 AND inherent_score < 15) as medium_risks,
                COUNT(*) FILTER (WHERE inherent_score < 5) as low_risks,
                COUNT(*) FILTER (WHERE review_date IS NOT NULL AND review_date <= CURRENT_DATE) as needs_review,
                AVG(inherent_score)::float8 as avg_inherent,
                AVG(residual_score)::float8 as avg_residual
            FROM risks
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        // Get by status
        let by_status: Vec<StatusCount> = sqlx::query_as(
            r#"
            SELECT status, COUNT(*) as count
            FROM risks
            WHERE organization_id = $1
            GROUP BY status
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        // Get by category
        let by_category: Vec<RiskCategoryCount> = sqlx::query_as(
            r#"
            SELECT category, COUNT(*) as count
            FROM risks
            WHERE organization_id = $1
            GROUP BY category
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let stats = RiskStats {
            total,
            by_status,
            by_category,
            high_risks,
            medium_risks,
            low_risks,
            needs_review,
            average_inherent_score: avg_inherent.unwrap_or(0.0),
            average_residual_score: avg_residual.unwrap_or(0.0),
        };

        // Cache for 5 minutes
        self.cache
            .set(&cache_key, &stats, Some(Duration::from_secs(300)))
            .await?;

        Ok(stats)
    }

    // ==================== Cache Invalidation ====================

    async fn invalidate_risk_cache(&self, org_id: Uuid, risk_id: Uuid) -> AppResult<()> {
        let cache_key = org_cache_key(
            &org_id.to_string(),
            CACHE_PREFIX_RISK,
            &risk_id.to_string(),
        );
        self.cache.delete(&cache_key).await?;

        self.invalidate_org_risk_caches(org_id).await
    }

    async fn invalidate_org_risk_caches(&self, org_id: Uuid) -> AppResult<()> {
        let stats_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_RISK_STATS, "summary");
        self.cache.delete(&stats_key).await
    }
}
