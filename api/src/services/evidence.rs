use crate::cache::{org_cache_key, CacheClient};
use crate::models::{
    CreateEvidence, Evidence, EvidenceControlLink, EvidenceStats, EvidenceWithLinks,
    LinkedControl, ListEvidenceQuery, SourceCount, TypeCount, UpdateEvidence,
};
use crate::utils::{AppError, AppResult};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(1800); // 30 minutes
const CACHE_PREFIX_EVIDENCE: &str = "evidence";
const CACHE_PREFIX_EVIDENCE_STATS: &str = "evidence:stats";

#[derive(Clone)]
pub struct EvidenceService {
    db: PgPool,
    cache: CacheClient,
}

impl EvidenceService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Evidence CRUD ====================

    /// List evidence for an organization with filtering
    pub async fn list_evidence(
        &self,
        org_id: Uuid,
        query: ListEvidenceQuery,
    ) -> AppResult<Vec<EvidenceWithLinks>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        // Handle control_id filter with a join
        let evidence = if let Some(control_id) = query.control_id {
            sqlx::query_as::<_, Evidence>(
                r#"
                SELECT DISTINCT e.id, e.organization_id, e.title, e.description, e.evidence_type,
                       e.source, e.source_reference, e.file_path, e.file_size, e.mime_type,
                       e.collected_at, e.valid_from, e.valid_until, e.uploaded_by, e.created_at
                FROM evidence e
                JOIN evidence_control_links ecl ON e.id = ecl.evidence_id
                WHERE e.organization_id = $1
                  AND ecl.control_id = $2
                  AND ($3::text IS NULL OR e.evidence_type = $3)
                  AND ($4::text IS NULL OR e.source = $4)
                  AND ($5::bool IS NULL OR ($5 = true AND e.valid_until < NOW()) OR ($5 = false AND (e.valid_until IS NULL OR e.valid_until >= NOW())))
                ORDER BY e.collected_at DESC
                LIMIT $6 OFFSET $7
                "#,
            )
            .bind(org_id)
            .bind(control_id)
            .bind(&query.evidence_type)
            .bind(&query.source)
            .bind(query.expired)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else if let Some(ref search) = query.search {
            let search_pattern = format!("%{}%", search.to_lowercase());
            sqlx::query_as::<_, Evidence>(
                r#"
                SELECT id, organization_id, title, description, evidence_type, source,
                       source_reference, file_path, file_size, mime_type, collected_at,
                       valid_from, valid_until, uploaded_by, created_at
                FROM evidence
                WHERE organization_id = $1
                  AND (LOWER(title) LIKE $2 OR LOWER(description) LIKE $2)
                  AND ($3::text IS NULL OR evidence_type = $3)
                  AND ($4::text IS NULL OR source = $4)
                  AND ($5::bool IS NULL OR ($5 = true AND valid_until < NOW()) OR ($5 = false AND (valid_until IS NULL OR valid_until >= NOW())))
                ORDER BY collected_at DESC
                LIMIT $6 OFFSET $7
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .bind(&query.evidence_type)
            .bind(&query.source)
            .bind(query.expired)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, Evidence>(
                r#"
                SELECT id, organization_id, title, description, evidence_type, source,
                       source_reference, file_path, file_size, mime_type, collected_at,
                       valid_from, valid_until, uploaded_by, created_at
                FROM evidence
                WHERE organization_id = $1
                  AND ($2::text IS NULL OR evidence_type = $2)
                  AND ($3::text IS NULL OR source = $3)
                  AND ($4::bool IS NULL OR ($4 = true AND valid_until < NOW()) OR ($4 = false AND (valid_until IS NULL OR valid_until >= NOW())))
                ORDER BY collected_at DESC
                LIMIT $5 OFFSET $6
                "#,
            )
            .bind(org_id)
            .bind(&query.evidence_type)
            .bind(&query.source)
            .bind(query.expired)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        };

        // Get linked control counts in one query
        let evidence_ids: Vec<Uuid> = evidence.iter().map(|e| e.id).collect();

        let counts: Vec<(Uuid, i64)> = if !evidence_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT evidence_id, COUNT(*) as count
                FROM evidence_control_links
                WHERE evidence_id = ANY($1)
                GROUP BY evidence_id
                "#,
            )
            .bind(&evidence_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let count_map: std::collections::HashMap<Uuid, i64> = counts.into_iter().collect();

        let result: Vec<EvidenceWithLinks> = evidence
            .into_iter()
            .map(|ev| {
                let count = count_map.get(&ev.id).copied().unwrap_or(0);
                EvidenceWithLinks {
                    evidence: ev,
                    linked_control_count: count,
                    linked_controls: None,
                }
            })
            .collect();

        Ok(result)
    }

    /// Get a single evidence by ID with linked controls
    pub async fn get_evidence(&self, org_id: Uuid, id: Uuid) -> AppResult<EvidenceWithLinks> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_EVIDENCE, &id.to_string());

        // Try cache first
        if let Some(cached) = self.cache.get::<EvidenceWithLinks>(&cache_key).await? {
            tracing::debug!("Cache hit for evidence {}", id);
            return Ok(cached);
        }

        let evidence = sqlx::query_as::<_, Evidence>(
            r#"
            SELECT id, organization_id, title, description, evidence_type, source,
                   source_reference, file_path, file_size, mime_type, collected_at,
                   valid_from, valid_until, uploaded_by, created_at
            FROM evidence
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Evidence {} not found", id)))?;

        // Get linked controls
        let linked_controls = sqlx::query_as::<_, LinkedControl>(
            r#"
            SELECT c.id, c.code, c.name
            FROM evidence_control_links ecl
            JOIN controls c ON ecl.control_id = c.id
            WHERE ecl.evidence_id = $1
            ORDER BY c.code
            "#,
        )
        .bind(id)
        .fetch_all(&self.db)
        .await?;

        let linked_control_count = linked_controls.len() as i64;

        let result = EvidenceWithLinks {
            evidence,
            linked_control_count,
            linked_controls: Some(linked_controls),
        };

        // Cache the result
        self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await?;

        Ok(result)
    }

    /// Create new evidence
    pub async fn create_evidence(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        input: CreateEvidence,
    ) -> AppResult<Evidence> {
        Evidence::validate_create(&input).map_err(|e| AppError::ValidationError(e))?;

        let evidence = sqlx::query_as::<_, Evidence>(
            r#"
            INSERT INTO evidence (organization_id, title, description, evidence_type, source,
                                  source_reference, file_path, file_size, mime_type,
                                  valid_from, valid_until, uploaded_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, organization_id, title, description, evidence_type, source,
                      source_reference, file_path, file_size, mime_type, collected_at,
                      valid_from, valid_until, uploaded_by, created_at
            "#,
        )
        .bind(org_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(input.evidence_type.as_deref().unwrap_or("document"))
        .bind(input.source.as_deref().unwrap_or("manual"))
        .bind(&input.source_reference)
        .bind(&input.file_path)
        .bind(input.file_size)
        .bind(&input.mime_type)
        .bind(input.valid_from)
        .bind(input.valid_until)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_org_evidence_caches(org_id).await?;

        tracing::info!("Created evidence: {} ({})", evidence.title, evidence.id);

        Ok(evidence)
    }

    /// Update evidence
    pub async fn update_evidence(
        &self,
        org_id: Uuid,
        id: Uuid,
        input: UpdateEvidence,
    ) -> AppResult<Evidence> {
        // Verify evidence exists
        let _ = self.get_evidence(org_id, id).await?;

        let evidence = sqlx::query_as::<_, Evidence>(
            r#"
            UPDATE evidence
            SET
                title = COALESCE($3, title),
                description = COALESCE($4, description),
                evidence_type = COALESCE($5, evidence_type),
                source = COALESCE($6, source),
                source_reference = COALESCE($7, source_reference),
                valid_from = COALESCE($8, valid_from),
                valid_until = COALESCE($9, valid_until)
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, title, description, evidence_type, source,
                      source_reference, file_path, file_size, mime_type, collected_at,
                      valid_from, valid_until, uploaded_by, created_at
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.evidence_type)
        .bind(&input.source)
        .bind(&input.source_reference)
        .bind(input.valid_from)
        .bind(input.valid_until)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_evidence_cache(org_id, id).await?;

        tracing::info!("Updated evidence: {} ({})", evidence.title, evidence.id);

        Ok(evidence)
    }

    /// Delete evidence
    pub async fn delete_evidence(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        // Verify evidence exists
        self.get_evidence(org_id, id).await?;

        sqlx::query("DELETE FROM evidence WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        // Invalidate caches
        self.invalidate_evidence_cache(org_id, id).await?;

        tracing::info!("Deleted evidence: {}", id);

        Ok(())
    }

    // ==================== Control Links ====================

    /// Link evidence to controls
    pub async fn link_to_controls(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
        control_ids: Vec<Uuid>,
        user_id: Option<Uuid>,
    ) -> AppResult<Vec<EvidenceControlLink>> {
        // Verify evidence exists
        self.get_evidence(org_id, evidence_id).await?;

        let mut tx = self.db.begin().await?;
        let mut links = Vec::new();

        for control_id in control_ids {
            // Check if link already exists
            let existing: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM evidence_control_links WHERE evidence_id = $1 AND control_id = $2",
            )
            .bind(evidence_id)
            .bind(control_id)
            .fetch_optional(&mut *tx)
            .await?;

            if existing.is_none() {
                let link = sqlx::query_as::<_, EvidenceControlLink>(
                    r#"
                    INSERT INTO evidence_control_links (evidence_id, control_id, linked_by)
                    VALUES ($1, $2, $3)
                    RETURNING id, evidence_id, control_id, control_test_result_id, linked_by, linked_at
                    "#,
                )
                .bind(evidence_id)
                .bind(control_id)
                .bind(user_id)
                .fetch_one(&mut *tx)
                .await?;

                links.push(link);
            }
        }

        tx.commit().await?;

        // Invalidate evidence cache
        self.invalidate_evidence_cache(org_id, evidence_id).await?;

        tracing::info!(
            "Linked {} controls to evidence {}",
            links.len(),
            evidence_id
        );

        Ok(links)
    }

    /// Unlink evidence from controls
    pub async fn unlink_from_controls(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
        control_ids: Vec<Uuid>,
    ) -> AppResult<i64> {
        // Verify evidence exists
        self.get_evidence(org_id, evidence_id).await?;

        let result = sqlx::query(
            "DELETE FROM evidence_control_links WHERE evidence_id = $1 AND control_id = ANY($2)",
        )
        .bind(evidence_id)
        .bind(&control_ids)
        .execute(&self.db)
        .await?;

        let deleted = result.rows_affected() as i64;

        // Invalidate evidence cache
        self.invalidate_evidence_cache(org_id, evidence_id).await?;

        tracing::info!(
            "Unlinked {} controls from evidence {}",
            deleted,
            evidence_id
        );

        Ok(deleted)
    }

    // ==================== Statistics ====================

    /// Get evidence statistics
    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<EvidenceStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_EVIDENCE_STATS, "summary");

        // Try cache first
        if let Some(cached) = self.cache.get::<EvidenceStats>(&cache_key).await? {
            tracing::debug!("Cache hit for evidence stats");
            return Ok(cached);
        }

        // Get total and expiry stats
        let (total, expiring_soon, expired): (i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE valid_until IS NOT NULL AND valid_until <= NOW() + INTERVAL '30 days' AND valid_until > NOW()) as expiring_soon,
                COUNT(*) FILTER (WHERE valid_until IS NOT NULL AND valid_until < NOW()) as expired
            FROM evidence
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        // Get by type
        let by_type: Vec<TypeCount> = sqlx::query_as(
            r#"
            SELECT evidence_type, COUNT(*) as count
            FROM evidence
            WHERE organization_id = $1
            GROUP BY evidence_type
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        // Get by source
        let by_source: Vec<SourceCount> = sqlx::query_as(
            r#"
            SELECT source, COUNT(*) as count
            FROM evidence
            WHERE organization_id = $1
            GROUP BY source
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let stats = EvidenceStats {
            total,
            by_type,
            by_source,
            expiring_soon,
            expired,
        };

        // Cache for 5 minutes
        self.cache
            .set(&cache_key, &stats, Some(Duration::from_secs(300)))
            .await?;

        Ok(stats)
    }

    // ==================== Cache Invalidation ====================

    async fn invalidate_evidence_cache(&self, org_id: Uuid, evidence_id: Uuid) -> AppResult<()> {
        let cache_key = org_cache_key(
            &org_id.to_string(),
            CACHE_PREFIX_EVIDENCE,
            &evidence_id.to_string(),
        );
        self.cache.delete(&cache_key).await?;

        self.invalidate_org_evidence_caches(org_id).await
    }

    async fn invalidate_org_evidence_caches(&self, org_id: Uuid) -> AppResult<()> {
        // Invalidate stats cache
        let stats_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_EVIDENCE_STATS, "summary");
        self.cache.delete(&stats_key).await
    }
}
