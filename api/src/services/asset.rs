use crate::cache::{org_cache_key, CacheClient};
use crate::models::{
    Asset, AssetControlMapping, AssetStats, AssetStatusCount, AssetTypeCount,
    AssetWithControls, ClassificationCount, CreateAsset, ListAssetsQuery, UpdateAsset,
};
use crate::utils::{AppError, AppResult};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(1800); // 30 minutes
const CACHE_PREFIX_ASSET: &str = "asset";
const CACHE_PREFIX_ASSET_STATS: &str = "asset:stats";

#[derive(Clone)]
pub struct AssetService {
    db: PgPool,
    cache: CacheClient,
}

impl AssetService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Asset CRUD ====================

    /// List assets for an organization with filtering
    pub async fn list_assets(
        &self,
        org_id: Uuid,
        query: ListAssetsQuery,
    ) -> AppResult<Vec<AssetWithControls>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        let assets = if let Some(ref search) = query.search {
            let search_pattern = format!("%{}%", search.to_lowercase());
            sqlx::query_as::<_, Asset>(
                r#"
                SELECT id, organization_id, name, description, asset_type, category,
                       classification, status, owner_id, custodian_id, location,
                       ip_address, mac_address, purchase_date, warranty_until,
                       metadata, created_at, updated_at
                FROM assets
                WHERE organization_id = $1
                  AND (LOWER(name) LIKE $2 OR LOWER(description) LIKE $2 OR LOWER(ip_address) LIKE $2)
                  AND ($3::text IS NULL OR asset_type = $3)
                  AND ($4::text IS NULL OR category = $4)
                  AND ($5::text IS NULL OR classification = $5)
                  AND ($6::text IS NULL OR status = $6)
                  AND ($7::uuid IS NULL OR owner_id = $7)
                ORDER BY name ASC
                LIMIT $8 OFFSET $9
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .bind(&query.asset_type)
            .bind(&query.category)
            .bind(&query.classification)
            .bind(&query.status)
            .bind(query.owner_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, Asset>(
                r#"
                SELECT id, organization_id, name, description, asset_type, category,
                       classification, status, owner_id, custodian_id, location,
                       ip_address, mac_address, purchase_date, warranty_until,
                       metadata, created_at, updated_at
                FROM assets
                WHERE organization_id = $1
                  AND ($2::text IS NULL OR asset_type = $2)
                  AND ($3::text IS NULL OR category = $3)
                  AND ($4::text IS NULL OR classification = $4)
                  AND ($5::text IS NULL OR status = $5)
                  AND ($6::uuid IS NULL OR owner_id = $6)
                ORDER BY name ASC
                LIMIT $7 OFFSET $8
                "#,
            )
            .bind(org_id)
            .bind(&query.asset_type)
            .bind(&query.category)
            .bind(&query.classification)
            .bind(&query.status)
            .bind(query.owner_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        };

        // Get control counts
        let asset_ids: Vec<Uuid> = assets.iter().map(|a| a.id).collect();

        let counts: Vec<(Uuid, i64)> = if !asset_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT asset_id, COUNT(*) as count
                FROM asset_control_mappings
                WHERE asset_id = ANY($1)
                GROUP BY asset_id
                "#,
            )
            .bind(&asset_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let count_map: std::collections::HashMap<Uuid, i64> = counts.into_iter().collect();

        let result: Vec<AssetWithControls> = assets
            .into_iter()
            .map(|asset| {
                let count = count_map.get(&asset.id).copied().unwrap_or(0);
                AssetWithControls {
                    asset,
                    linked_control_count: count,
                }
            })
            .collect();

        Ok(result)
    }

    /// Get a single asset by ID
    pub async fn get_asset(&self, org_id: Uuid, id: Uuid) -> AppResult<AssetWithControls> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_ASSET, &id.to_string());

        // Try cache first
        if let Some(cached) = self.cache.get::<AssetWithControls>(&cache_key).await? {
            tracing::debug!("Cache hit for asset {}", id);
            return Ok(cached);
        }

        let asset = sqlx::query_as::<_, Asset>(
            r#"
            SELECT id, organization_id, name, description, asset_type, category,
                   classification, status, owner_id, custodian_id, location,
                   ip_address, mac_address, purchase_date, warranty_until,
                   metadata, created_at, updated_at
            FROM assets
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Asset {} not found", id)))?;

        let (linked_control_count,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM asset_control_mappings WHERE asset_id = $1",
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        let result = AssetWithControls {
            asset,
            linked_control_count,
        };

        // Cache the result
        self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await?;

        Ok(result)
    }

    /// Create a new asset
    pub async fn create_asset(&self, org_id: Uuid, input: CreateAsset) -> AppResult<Asset> {
        Asset::validate_create(&input).map_err(AppError::ValidationError)?;

        let metadata = input.metadata.unwrap_or(serde_json::json!({}));

        let asset = sqlx::query_as::<_, Asset>(
            r#"
            INSERT INTO assets (organization_id, name, description, asset_type, category,
                                classification, owner_id, custodian_id, location, ip_address,
                                mac_address, purchase_date, warranty_until, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            RETURNING id, organization_id, name, description, asset_type, category,
                      classification, status, owner_id, custodian_id, location,
                      ip_address, mac_address, purchase_date, warranty_until,
                      metadata, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.asset_type)
        .bind(&input.category)
        .bind(input.classification.as_deref().unwrap_or("internal"))
        .bind(input.owner_id)
        .bind(input.custodian_id)
        .bind(&input.location)
        .bind(&input.ip_address)
        .bind(&input.mac_address)
        .bind(input.purchase_date)
        .bind(input.warranty_until)
        .bind(&metadata)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_org_asset_caches(org_id).await?;

        tracing::info!("Created asset: {} ({})", asset.name, asset.id);

        Ok(asset)
    }

    /// Update an asset
    pub async fn update_asset(
        &self,
        org_id: Uuid,
        id: Uuid,
        input: UpdateAsset,
    ) -> AppResult<Asset> {
        // Verify asset exists
        self.get_asset(org_id, id).await?;

        let asset = sqlx::query_as::<_, Asset>(
            r#"
            UPDATE assets
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                asset_type = COALESCE($5, asset_type),
                category = COALESCE($6, category),
                classification = COALESCE($7, classification),
                status = COALESCE($8, status),
                owner_id = COALESCE($9, owner_id),
                custodian_id = COALESCE($10, custodian_id),
                location = COALESCE($11, location),
                ip_address = COALESCE($12, ip_address),
                mac_address = COALESCE($13, mac_address),
                purchase_date = COALESCE($14, purchase_date),
                warranty_until = COALESCE($15, warranty_until),
                metadata = COALESCE($16, metadata),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, name, description, asset_type, category,
                      classification, status, owner_id, custodian_id, location,
                      ip_address, mac_address, purchase_date, warranty_until,
                      metadata, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.asset_type)
        .bind(&input.category)
        .bind(&input.classification)
        .bind(&input.status)
        .bind(input.owner_id)
        .bind(input.custodian_id)
        .bind(&input.location)
        .bind(&input.ip_address)
        .bind(&input.mac_address)
        .bind(input.purchase_date)
        .bind(input.warranty_until)
        .bind(&input.metadata)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_asset_cache(org_id, id).await?;

        tracing::info!("Updated asset: {} ({})", asset.name, asset.id);

        Ok(asset)
    }

    /// Delete an asset
    pub async fn delete_asset(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        // Verify asset exists
        self.get_asset(org_id, id).await?;

        sqlx::query("DELETE FROM assets WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        // Invalidate caches
        self.invalidate_asset_cache(org_id, id).await?;

        tracing::info!("Deleted asset: {}", id);

        Ok(())
    }

    // ==================== Control Mappings ====================

    /// Link controls to an asset
    pub async fn link_controls(
        &self,
        org_id: Uuid,
        asset_id: Uuid,
        control_ids: Vec<Uuid>,
    ) -> AppResult<Vec<AssetControlMapping>> {
        // Verify asset exists
        self.get_asset(org_id, asset_id).await?;

        let mut tx = self.db.begin().await?;
        let mut mappings = Vec::new();

        for control_id in control_ids {
            let existing: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM asset_control_mappings WHERE asset_id = $1 AND control_id = $2",
            )
            .bind(asset_id)
            .bind(control_id)
            .fetch_optional(&mut *tx)
            .await?;

            if existing.is_none() {
                let mapping = sqlx::query_as::<_, AssetControlMapping>(
                    r#"
                    INSERT INTO asset_control_mappings (asset_id, control_id)
                    VALUES ($1, $2)
                    RETURNING id, asset_id, control_id, created_at
                    "#,
                )
                .bind(asset_id)
                .bind(control_id)
                .fetch_one(&mut *tx)
                .await?;

                mappings.push(mapping);
            }
        }

        tx.commit().await?;

        // Invalidate cache
        self.invalidate_asset_cache(org_id, asset_id).await?;

        tracing::info!("Linked {} controls to asset {}", mappings.len(), asset_id);

        Ok(mappings)
    }

    /// Unlink controls from an asset
    pub async fn unlink_controls(
        &self,
        org_id: Uuid,
        asset_id: Uuid,
        control_ids: Vec<Uuid>,
    ) -> AppResult<i64> {
        // Verify asset exists
        self.get_asset(org_id, asset_id).await?;

        let result = sqlx::query(
            "DELETE FROM asset_control_mappings WHERE asset_id = $1 AND control_id = ANY($2)",
        )
        .bind(asset_id)
        .bind(&control_ids)
        .execute(&self.db)
        .await?;

        let deleted = result.rows_affected() as i64;

        // Invalidate cache
        self.invalidate_asset_cache(org_id, asset_id).await?;

        tracing::info!("Unlinked {} controls from asset {}", deleted, asset_id);

        Ok(deleted)
    }

    // ==================== Statistics ====================

    /// Get asset statistics
    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<AssetStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_ASSET_STATS, "summary");

        // Try cache first
        if let Some(cached) = self.cache.get::<AssetStats>(&cache_key).await? {
            tracing::debug!("Cache hit for asset stats");
            return Ok(cached);
        }

        let (total, warranty_expiring_soon): (i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE warranty_until IS NOT NULL AND warranty_until <= CURRENT_DATE + INTERVAL '90 days' AND warranty_until >= CURRENT_DATE) as warranty_expiring_soon
            FROM assets
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let by_type: Vec<AssetTypeCount> = sqlx::query_as(
            r#"
            SELECT asset_type, COUNT(*) as count
            FROM assets
            WHERE organization_id = $1
            GROUP BY asset_type
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let by_classification: Vec<ClassificationCount> = sqlx::query_as(
            r#"
            SELECT classification, COUNT(*) as count
            FROM assets
            WHERE organization_id = $1
            GROUP BY classification
            ORDER BY
                CASE classification
                    WHEN 'restricted' THEN 1
                    WHEN 'confidential' THEN 2
                    WHEN 'internal' THEN 3
                    WHEN 'public' THEN 4
                    ELSE 5
                END
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let by_status: Vec<AssetStatusCount> = sqlx::query_as(
            r#"
            SELECT status, COUNT(*) as count
            FROM assets
            WHERE organization_id = $1
            GROUP BY status
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let stats = AssetStats {
            total,
            by_type,
            by_classification,
            by_status,
            warranty_expiring_soon,
        };

        // Cache for 5 minutes
        self.cache
            .set(&cache_key, &stats, Some(Duration::from_secs(300)))
            .await?;

        Ok(stats)
    }

    // ==================== Cache Invalidation ====================

    async fn invalidate_asset_cache(&self, org_id: Uuid, asset_id: Uuid) -> AppResult<()> {
        let cache_key = org_cache_key(
            &org_id.to_string(),
            CACHE_PREFIX_ASSET,
            &asset_id.to_string(),
        );
        self.cache.delete(&cache_key).await?;

        self.invalidate_org_asset_caches(org_id).await
    }

    async fn invalidate_org_asset_caches(&self, org_id: Uuid) -> AppResult<()> {
        let stats_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_ASSET_STATS, "summary");
        self.cache.delete(&stats_key).await
    }
}
