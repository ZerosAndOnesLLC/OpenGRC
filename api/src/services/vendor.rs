use crate::cache::{org_cache_key, CacheClient};
use crate::models::{
    CreateVendor, CreateVendorAssessment, CriticalityCount, ListVendorsQuery, UpdateVendor,
    Vendor, VendorAssessment, VendorCategoryCount, VendorStats, VendorWithAssessment,
};
use crate::utils::{AppError, AppResult};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(1800); // 30 minutes
const CACHE_PREFIX_VENDOR: &str = "vendor";
const CACHE_PREFIX_VENDOR_STATS: &str = "vendor:stats";

#[derive(Clone)]
pub struct VendorService {
    db: PgPool,
    cache: CacheClient,
}

impl VendorService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Vendor CRUD ====================

    /// List vendors for an organization with filtering
    pub async fn list_vendors(
        &self,
        org_id: Uuid,
        query: ListVendorsQuery,
    ) -> AppResult<Vec<VendorWithAssessment>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        let vendors = if let Some(ref search) = query.search {
            let search_pattern = format!("%{}%", search.to_lowercase());
            sqlx::query_as::<_, Vendor>(
                r#"
                SELECT id, organization_id, name, description, category, criticality,
                       data_classification, status, contract_start, contract_end,
                       owner_id, website, created_at, updated_at
                FROM vendors
                WHERE organization_id = $1
                  AND (LOWER(name) LIKE $2 OR LOWER(description) LIKE $2)
                  AND ($3::text IS NULL OR status = $3)
                  AND ($4::text IS NULL OR category = $4)
                  AND ($5::text IS NULL OR criticality = $5)
                  AND ($6::uuid IS NULL OR owner_id = $6)
                  AND ($7::bool IS NULL OR ($7 = true AND contract_end <= CURRENT_DATE + INTERVAL '90 days' AND contract_end >= CURRENT_DATE))
                ORDER BY criticality DESC NULLS LAST, name ASC
                LIMIT $8 OFFSET $9
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .bind(&query.status)
            .bind(&query.category)
            .bind(&query.criticality)
            .bind(query.owner_id)
            .bind(query.contract_expiring)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, Vendor>(
                r#"
                SELECT id, organization_id, name, description, category, criticality,
                       data_classification, status, contract_start, contract_end,
                       owner_id, website, created_at, updated_at
                FROM vendors
                WHERE organization_id = $1
                  AND ($2::text IS NULL OR status = $2)
                  AND ($3::text IS NULL OR category = $3)
                  AND ($4::text IS NULL OR criticality = $4)
                  AND ($5::uuid IS NULL OR owner_id = $5)
                  AND ($6::bool IS NULL OR ($6 = true AND contract_end <= CURRENT_DATE + INTERVAL '90 days' AND contract_end >= CURRENT_DATE))
                ORDER BY criticality DESC NULLS LAST, name ASC
                LIMIT $7 OFFSET $8
                "#,
            )
            .bind(org_id)
            .bind(&query.status)
            .bind(&query.category)
            .bind(&query.criticality)
            .bind(query.owner_id)
            .bind(query.contract_expiring)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        };

        // Get assessment counts in one query
        let vendor_ids: Vec<Uuid> = vendors.iter().map(|v| v.id).collect();

        let assessment_counts: Vec<(Uuid, i64)> = if !vendor_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT vendor_id, COUNT(*) as count
                FROM vendor_assessments
                WHERE vendor_id = ANY($1)
                GROUP BY vendor_id
                "#,
            )
            .bind(&vendor_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let count_map: std::collections::HashMap<Uuid, i64> =
            assessment_counts.into_iter().collect();

        let result: Vec<VendorWithAssessment> = vendors
            .into_iter()
            .map(|vendor| {
                let count = count_map.get(&vendor.id).copied().unwrap_or(0);
                VendorWithAssessment {
                    vendor,
                    latest_assessment: None,
                    assessment_count: count,
                }
            })
            .collect();

        Ok(result)
    }

    /// Get a single vendor by ID with latest assessment
    pub async fn get_vendor(&self, org_id: Uuid, id: Uuid) -> AppResult<VendorWithAssessment> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_VENDOR, &id.to_string());

        // Try cache first
        if let Some(cached) = self.cache.get::<VendorWithAssessment>(&cache_key).await? {
            tracing::debug!("Cache hit for vendor {}", id);
            return Ok(cached);
        }

        let vendor = sqlx::query_as::<_, Vendor>(
            r#"
            SELECT id, organization_id, name, description, category, criticality,
                   data_classification, status, contract_start, contract_end,
                   owner_id, website, created_at, updated_at
            FROM vendors
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Vendor {} not found", id)))?;

        // Get latest assessment
        let latest_assessment = sqlx::query_as::<_, VendorAssessment>(
            r#"
            SELECT id, vendor_id, assessment_type, assessed_by, assessed_at,
                   risk_rating, findings, recommendations, next_assessment_date, created_at
            FROM vendor_assessments
            WHERE vendor_id = $1
            ORDER BY assessed_at DESC
            LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        // Get assessment count
        let (assessment_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM vendor_assessments WHERE vendor_id = $1",
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        let result = VendorWithAssessment {
            vendor,
            latest_assessment,
            assessment_count,
        };

        // Cache the result
        self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await?;

        Ok(result)
    }

    /// Create a new vendor
    pub async fn create_vendor(&self, org_id: Uuid, input: CreateVendor) -> AppResult<Vendor> {
        Vendor::validate_create(&input).map_err(AppError::ValidationError)?;

        let vendor = sqlx::query_as::<_, Vendor>(
            r#"
            INSERT INTO vendors (organization_id, name, description, category, criticality,
                                 data_classification, contract_start, contract_end, owner_id, website)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING id, organization_id, name, description, category, criticality,
                      data_classification, status, contract_start, contract_end,
                      owner_id, website, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.category)
        .bind(input.criticality.as_deref().unwrap_or("medium"))
        .bind(&input.data_classification)
        .bind(input.contract_start)
        .bind(input.contract_end)
        .bind(input.owner_id)
        .bind(&input.website)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_org_vendor_caches(org_id).await?;

        tracing::info!("Created vendor: {} ({})", vendor.name, vendor.id);

        Ok(vendor)
    }

    /// Update a vendor
    pub async fn update_vendor(
        &self,
        org_id: Uuid,
        id: Uuid,
        input: UpdateVendor,
    ) -> AppResult<Vendor> {
        // Verify vendor exists
        self.get_vendor(org_id, id).await?;

        let vendor = sqlx::query_as::<_, Vendor>(
            r#"
            UPDATE vendors
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                category = COALESCE($5, category),
                criticality = COALESCE($6, criticality),
                data_classification = COALESCE($7, data_classification),
                status = COALESCE($8, status),
                contract_start = COALESCE($9, contract_start),
                contract_end = COALESCE($10, contract_end),
                owner_id = COALESCE($11, owner_id),
                website = COALESCE($12, website),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, name, description, category, criticality,
                      data_classification, status, contract_start, contract_end,
                      owner_id, website, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.category)
        .bind(&input.criticality)
        .bind(&input.data_classification)
        .bind(&input.status)
        .bind(input.contract_start)
        .bind(input.contract_end)
        .bind(input.owner_id)
        .bind(&input.website)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_vendor_cache(org_id, id).await?;

        tracing::info!("Updated vendor: {} ({})", vendor.name, vendor.id);

        Ok(vendor)
    }

    /// Delete a vendor
    pub async fn delete_vendor(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        // Verify vendor exists
        self.get_vendor(org_id, id).await?;

        sqlx::query("DELETE FROM vendors WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        // Invalidate caches
        self.invalidate_vendor_cache(org_id, id).await?;

        tracing::info!("Deleted vendor: {}", id);

        Ok(())
    }

    // ==================== Assessments ====================

    /// Create a vendor assessment
    pub async fn create_assessment(
        &self,
        org_id: Uuid,
        vendor_id: Uuid,
        user_id: Option<Uuid>,
        input: CreateVendorAssessment,
    ) -> AppResult<VendorAssessment> {
        // Verify vendor exists
        self.get_vendor(org_id, vendor_id).await?;

        let assessment = sqlx::query_as::<_, VendorAssessment>(
            r#"
            INSERT INTO vendor_assessments (vendor_id, assessment_type, assessed_by, risk_rating,
                                            findings, recommendations, next_assessment_date)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, vendor_id, assessment_type, assessed_by, assessed_at,
                      risk_rating, findings, recommendations, next_assessment_date, created_at
            "#,
        )
        .bind(vendor_id)
        .bind(input.assessment_type.as_deref().unwrap_or("periodic"))
        .bind(user_id)
        .bind(&input.risk_rating)
        .bind(&input.findings)
        .bind(&input.recommendations)
        .bind(input.next_assessment_date)
        .fetch_one(&self.db)
        .await?;

        // Invalidate vendor cache
        self.invalidate_vendor_cache(org_id, vendor_id).await?;

        tracing::info!(
            "Created assessment for vendor {}: {}",
            vendor_id,
            assessment.id
        );

        Ok(assessment)
    }

    /// Get assessments for a vendor
    pub async fn get_assessments(
        &self,
        org_id: Uuid,
        vendor_id: Uuid,
    ) -> AppResult<Vec<VendorAssessment>> {
        // Verify vendor exists
        self.get_vendor(org_id, vendor_id).await?;

        let assessments = sqlx::query_as::<_, VendorAssessment>(
            r#"
            SELECT id, vendor_id, assessment_type, assessed_by, assessed_at,
                   risk_rating, findings, recommendations, next_assessment_date, created_at
            FROM vendor_assessments
            WHERE vendor_id = $1
            ORDER BY assessed_at DESC
            "#,
        )
        .bind(vendor_id)
        .fetch_all(&self.db)
        .await?;

        Ok(assessments)
    }

    // ==================== Statistics ====================

    /// Get vendor statistics
    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<VendorStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_VENDOR_STATS, "summary");

        // Try cache first
        if let Some(cached) = self.cache.get::<VendorStats>(&cache_key).await? {
            tracing::debug!("Cache hit for vendor stats");
            return Ok(cached);
        }

        // Get basic counts
        let (total, active, inactive, contracts_expiring_soon, needs_assessment): (
            i64,
            i64,
            i64,
            i64,
            i64,
        ) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active,
                COUNT(*) FILTER (WHERE status = 'inactive' OR status = 'terminated') as inactive,
                COUNT(*) FILTER (WHERE contract_end IS NOT NULL AND contract_end <= CURRENT_DATE + INTERVAL '90 days' AND contract_end >= CURRENT_DATE) as contracts_expiring_soon,
                COUNT(*) FILTER (WHERE id NOT IN (
                    SELECT DISTINCT vendor_id FROM vendor_assessments
                    WHERE assessed_at >= NOW() - INTERVAL '1 year'
                )) as needs_assessment
            FROM vendors
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        // Get by criticality
        let by_criticality: Vec<CriticalityCount> = sqlx::query_as(
            r#"
            SELECT criticality, COUNT(*) as count
            FROM vendors
            WHERE organization_id = $1
            GROUP BY criticality
            ORDER BY
                CASE criticality
                    WHEN 'critical' THEN 1
                    WHEN 'high' THEN 2
                    WHEN 'medium' THEN 3
                    WHEN 'low' THEN 4
                    ELSE 5
                END
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        // Get by category
        let by_category: Vec<VendorCategoryCount> = sqlx::query_as(
            r#"
            SELECT category, COUNT(*) as count
            FROM vendors
            WHERE organization_id = $1
            GROUP BY category
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let stats = VendorStats {
            total,
            active,
            inactive,
            by_criticality,
            by_category,
            contracts_expiring_soon,
            needs_assessment,
        };

        // Cache for 5 minutes
        self.cache
            .set(&cache_key, &stats, Some(Duration::from_secs(300)))
            .await?;

        Ok(stats)
    }

    // ==================== Cache Invalidation ====================

    async fn invalidate_vendor_cache(&self, org_id: Uuid, vendor_id: Uuid) -> AppResult<()> {
        let cache_key = org_cache_key(
            &org_id.to_string(),
            CACHE_PREFIX_VENDOR,
            &vendor_id.to_string(),
        );
        self.cache.delete(&cache_key).await?;

        self.invalidate_org_vendor_caches(org_id).await
    }

    async fn invalidate_org_vendor_caches(&self, org_id: Uuid) -> AppResult<()> {
        let stats_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_VENDOR_STATS, "summary");
        self.cache.delete(&stats_key).await
    }
}
