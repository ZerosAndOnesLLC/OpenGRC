use crate::cache::{org_cache_key, CacheClient};
use crate::models::{
    Asset, AssetControlMapping, AssetStats, AssetStatusCount, AssetTypeCount,
    AssetWithControls, ClassificationCount, CreateAsset, DiscoveredAsset, LifecycleStageCount,
    ListAssetsQuery, UpdateAsset,
};
use crate::utils::{AppError, AppResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
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
                       metadata, created_at, updated_at,
                       lifecycle_stage, commissioned_date, decommission_date,
                       last_maintenance_date, next_maintenance_due, maintenance_frequency,
                       end_of_life_date, end_of_support_date,
                       integration_source, integration_id, external_id, last_synced_at
                FROM assets
                WHERE organization_id = $1
                  AND (LOWER(name) LIKE $2 OR LOWER(description) LIKE $2 OR LOWER(ip_address) LIKE $2)
                  AND ($3::text IS NULL OR asset_type = $3)
                  AND ($4::text IS NULL OR category = $4)
                  AND ($5::text IS NULL OR classification = $5)
                  AND ($6::text IS NULL OR status = $6)
                  AND ($7::uuid IS NULL OR owner_id = $7)
                  AND ($8::text IS NULL OR lifecycle_stage = $8)
                  AND ($9::text IS NULL OR integration_source = $9)
                  AND ($10::bool IS NULL OR ($10 = true AND next_maintenance_due <= CURRENT_DATE + INTERVAL '30 days' AND next_maintenance_due >= CURRENT_DATE))
                ORDER BY name ASC
                LIMIT $11 OFFSET $12
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .bind(&query.asset_type)
            .bind(&query.category)
            .bind(&query.classification)
            .bind(&query.status)
            .bind(query.owner_id)
            .bind(&query.lifecycle_stage)
            .bind(&query.integration_source)
            .bind(query.maintenance_due)
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
                       metadata, created_at, updated_at,
                       lifecycle_stage, commissioned_date, decommission_date,
                       last_maintenance_date, next_maintenance_due, maintenance_frequency,
                       end_of_life_date, end_of_support_date,
                       integration_source, integration_id, external_id, last_synced_at
                FROM assets
                WHERE organization_id = $1
                  AND ($2::text IS NULL OR asset_type = $2)
                  AND ($3::text IS NULL OR category = $3)
                  AND ($4::text IS NULL OR classification = $4)
                  AND ($5::text IS NULL OR status = $5)
                  AND ($6::uuid IS NULL OR owner_id = $6)
                  AND ($7::text IS NULL OR lifecycle_stage = $7)
                  AND ($8::text IS NULL OR integration_source = $8)
                  AND ($9::bool IS NULL OR ($9 = true AND next_maintenance_due <= CURRENT_DATE + INTERVAL '30 days' AND next_maintenance_due >= CURRENT_DATE))
                ORDER BY name ASC
                LIMIT $10 OFFSET $11
                "#,
            )
            .bind(org_id)
            .bind(&query.asset_type)
            .bind(&query.category)
            .bind(&query.classification)
            .bind(&query.status)
            .bind(query.owner_id)
            .bind(&query.lifecycle_stage)
            .bind(&query.integration_source)
            .bind(query.maintenance_due)
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

        let (total, warranty_expiring_soon, maintenance_due_soon, from_integrations): (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE warranty_until IS NOT NULL AND warranty_until <= CURRENT_DATE + INTERVAL '90 days' AND warranty_until >= CURRENT_DATE) as warranty_expiring_soon,
                COUNT(*) FILTER (WHERE next_maintenance_due IS NOT NULL AND next_maintenance_due <= CURRENT_DATE + INTERVAL '30 days' AND next_maintenance_due >= CURRENT_DATE) as maintenance_due_soon,
                COUNT(*) FILTER (WHERE integration_source IS NOT NULL) as from_integrations
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

        let by_lifecycle_stage: Vec<LifecycleStageCount> = sqlx::query_as(
            r#"
            SELECT lifecycle_stage, COUNT(*) as count
            FROM assets
            WHERE organization_id = $1
            GROUP BY lifecycle_stage
            ORDER BY
                CASE lifecycle_stage
                    WHEN 'procurement' THEN 1
                    WHEN 'deployment' THEN 2
                    WHEN 'active' THEN 3
                    WHEN 'maintenance' THEN 4
                    WHEN 'decommissioning' THEN 5
                    WHEN 'decommissioned' THEN 6
                    ELSE 7
                END
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
            by_lifecycle_stage,
            warranty_expiring_soon,
            maintenance_due_soon,
            from_integrations,
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

    // ==================== Asset Discovery from Integrations ====================

    /// Discover assets from AWS EC2, RDS, and S3 resources
    pub async fn discover_assets_from_aws(
        &self,
        org_id: Uuid,
        integration_id: Uuid,
    ) -> AppResult<AssetDiscoveryResult> {
        let mut result = AssetDiscoveryResult::default();

        // Discover EC2 instances
        let ec2_instances = sqlx::query_as::<_, AwsEc2Row>(
            r#"
            SELECT instance_id, state, instance_type, region, private_ip, public_ip, vpc_id, tags
            FROM aws_ec2_instances
            WHERE organization_id = $1 AND integration_id = $2
            "#,
        )
        .bind(org_id)
        .bind(integration_id)
        .fetch_all(&self.db)
        .await?;

        for instance in ec2_instances {
            let name = instance.get_name().unwrap_or_else(|| instance.instance_id.clone());
            let is_public = instance.public_ip.is_some();
            let ip_address = instance.private_ip.clone().or(instance.public_ip.clone());
            let created = self
                .upsert_discovered_asset(
                    org_id,
                    integration_id,
                    &instance.instance_id,
                    DiscoveredAsset {
                        external_id: instance.instance_id.clone(),
                        name,
                        description: Some(format!(
                            "EC2 instance ({}) in {} - {}",
                            instance.instance_type,
                            instance.region,
                            instance.state.as_deref().unwrap_or("unknown")
                        )),
                        asset_type: "cloud".to_string(),
                        ip_address,
                        metadata: serde_json::json!({
                            "instance_type": instance.instance_type,
                            "region": instance.region,
                            "state": instance.state,
                            "vpc_id": instance.vpc_id,
                            "is_public": is_public,
                        }),
                    },
                    "aws",
                    "compute",
                )
                .await?;
            if created {
                result.created += 1;
            } else {
                result.updated += 1;
            }
        }

        // Discover RDS instances
        let rds_instances = sqlx::query_as::<_, AwsRdsRow>(
            r#"
            SELECT db_instance_identifier, engine, engine_version, db_instance_class, region,
                   endpoint_address, status, publicly_accessible, storage_encrypted, tags
            FROM aws_rds_instances
            WHERE organization_id = $1 AND integration_id = $2
            "#,
        )
        .bind(org_id)
        .bind(integration_id)
        .fetch_all(&self.db)
        .await?;

        for instance in rds_instances {
            let name = instance.get_name().unwrap_or_else(|| instance.db_instance_identifier.clone());
            let created = self
                .upsert_discovered_asset(
                    org_id,
                    integration_id,
                    &instance.db_instance_identifier,
                    DiscoveredAsset {
                        external_id: instance.db_instance_identifier.clone(),
                        name,
                        description: Some(format!(
                            "RDS {} ({}) - {}",
                            instance.engine.clone().unwrap_or_default(),
                            instance.db_instance_class.clone().unwrap_or_default(),
                            instance.status.clone().unwrap_or_default()
                        )),
                        asset_type: "cloud".to_string(),
                        ip_address: instance.endpoint_address.clone(),
                        metadata: serde_json::json!({
                            "engine": instance.engine,
                            "engine_version": instance.engine_version,
                            "instance_class": instance.db_instance_class,
                            "region": instance.region,
                            "publicly_accessible": instance.publicly_accessible,
                            "storage_encrypted": instance.storage_encrypted,
                        }),
                    },
                    "aws",
                    "database",
                )
                .await?;
            if created {
                result.created += 1;
            } else {
                result.updated += 1;
            }
        }

        // Discover S3 buckets
        let s3_buckets = sqlx::query_as::<_, AwsS3Row>(
            r#"
            SELECT bucket_name, region, encryption_enabled, versioning_enabled, is_public, tags
            FROM aws_s3_buckets
            WHERE organization_id = $1 AND integration_id = $2
            "#,
        )
        .bind(org_id)
        .bind(integration_id)
        .fetch_all(&self.db)
        .await?;

        for bucket in s3_buckets {
            let name = bucket.get_name().unwrap_or_else(|| bucket.bucket_name.clone());
            let created = self
                .upsert_discovered_asset(
                    org_id,
                    integration_id,
                    &bucket.bucket_name,
                    DiscoveredAsset {
                        external_id: bucket.bucket_name.clone(),
                        name,
                        description: Some(format!(
                            "S3 bucket in {}{}{}",
                            bucket.region.clone().unwrap_or_else(|| "unknown region".to_string()),
                            if bucket.is_public { " (PUBLIC)" } else { "" },
                            if bucket.encryption_enabled { "" } else { " (unencrypted)" }
                        )),
                        asset_type: "data".to_string(),
                        ip_address: None,
                        metadata: serde_json::json!({
                            "region": bucket.region,
                            "encryption_enabled": bucket.encryption_enabled,
                            "versioning_enabled": bucket.versioning_enabled,
                            "is_public": bucket.is_public,
                        }),
                    },
                    "aws",
                    "storage",
                )
                .await?;
            if created {
                result.created += 1;
            } else {
                result.updated += 1;
            }
        }

        // Invalidate caches
        self.invalidate_org_asset_caches(org_id).await?;

        tracing::info!(
            "Discovered {} new assets, updated {} existing from AWS integration {}",
            result.created,
            result.updated,
            integration_id
        );

        Ok(result)
    }

    /// Upsert a discovered asset, returns true if created, false if updated
    async fn upsert_discovered_asset(
        &self,
        org_id: Uuid,
        integration_id: Uuid,
        external_id: &str,
        asset: DiscoveredAsset,
        source: &str,
        category: &str,
    ) -> AppResult<bool> {
        // Check if asset already exists by external_id and integration
        let existing: Option<(Uuid,)> = sqlx::query_as(
            r#"
            SELECT id FROM assets
            WHERE organization_id = $1 AND integration_id = $2 AND external_id = $3
            "#,
        )
        .bind(org_id)
        .bind(integration_id)
        .bind(external_id)
        .fetch_optional(&self.db)
        .await?;

        let now = Utc::now();

        if let Some((existing_id,)) = existing {
            // Update existing asset
            sqlx::query(
                r#"
                UPDATE assets
                SET name = $1, description = $2, ip_address = $3, metadata = $4,
                    last_synced_at = $5, updated_at = $5
                WHERE id = $6
                "#,
            )
            .bind(&asset.name)
            .bind(&asset.description)
            .bind(&asset.ip_address)
            .bind(&asset.metadata)
            .bind(now)
            .bind(existing_id)
            .execute(&self.db)
            .await?;

            Ok(false)
        } else {
            // Create new asset
            sqlx::query(
                r#"
                INSERT INTO assets (
                    organization_id, name, description, asset_type, category,
                    classification, status, ip_address, metadata, lifecycle_stage,
                    integration_source, integration_id, external_id, last_synced_at
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                "#,
            )
            .bind(org_id)
            .bind(&asset.name)
            .bind(&asset.description)
            .bind(&asset.asset_type)
            .bind(category)
            .bind("internal") // Default classification
            .bind("active")   // Default status
            .bind(&asset.ip_address)
            .bind(&asset.metadata)
            .bind("active")   // Default lifecycle stage
            .bind(source)
            .bind(integration_id)
            .bind(external_id)
            .bind(now)
            .execute(&self.db)
            .await?;

            Ok(true)
        }
    }
}

/// Result of asset discovery
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AssetDiscoveryResult {
    pub created: i32,
    pub updated: i32,
}

/// AWS EC2 row for discovery
#[derive(Debug, sqlx::FromRow)]
struct AwsEc2Row {
    instance_id: String,
    state: Option<String>,
    instance_type: String,
    region: String,
    private_ip: Option<String>,
    public_ip: Option<String>,
    vpc_id: Option<String>,
    tags: serde_json::Value,
}

impl AwsEc2Row {
    fn get_name(&self) -> Option<String> {
        self.tags
            .get("Name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

/// AWS RDS row for discovery
#[derive(Debug, sqlx::FromRow)]
struct AwsRdsRow {
    db_instance_identifier: String,
    engine: Option<String>,
    engine_version: Option<String>,
    db_instance_class: Option<String>,
    region: String,
    endpoint_address: Option<String>,
    status: Option<String>,
    publicly_accessible: bool,
    storage_encrypted: bool,
    tags: serde_json::Value,
}

impl AwsRdsRow {
    fn get_name(&self) -> Option<String> {
        self.tags
            .get("Name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

/// AWS S3 row for discovery
#[derive(Debug, sqlx::FromRow)]
struct AwsS3Row {
    bucket_name: String,
    region: Option<String>,
    encryption_enabled: bool,
    versioning_enabled: bool,
    is_public: bool,
    tags: serde_json::Value,
}

impl AwsS3Row {
    fn get_name(&self) -> Option<String> {
        self.tags
            .get("Name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}
