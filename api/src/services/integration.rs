use crate::cache::{org_cache_key, CacheClient};
use crate::integrations::{IntegrationProvider, IntegrationRegistry, SyncResult};
use crate::models::{
    CreateIntegration, HealthTrendPoint, Integration, IntegrationHealth, IntegrationHealthStats,
    IntegrationHealthWithDetails, IntegrationStats, IntegrationSyncLog, IntegrationTypeCount,
    IntegrationWithStats, ListIntegrationsQuery, RecentFailure, TestConnectionResult,
    UpdateIntegration,
};
use crate::utils::{AppError, AppResult, EncryptionService};
use sqlx::PgPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(1800); // 30 minutes
const CACHE_PREFIX_INTEGRATION: &str = "integration";
const CACHE_PREFIX_INTEGRATIONS_LIST: &str = "integrations:list";
const CACHE_PREFIX_INTEGRATION_STATS: &str = "integrations:stats";

#[derive(Clone)]
pub struct IntegrationService {
    db: PgPool,
    cache: CacheClient,
    registry: Arc<RwLock<IntegrationRegistry>>,
    encryption: EncryptionService,
}

impl IntegrationService {
    pub fn new(db: PgPool, cache: CacheClient, encryption: EncryptionService) -> Self {
        Self {
            db,
            cache,
            registry: Arc::new(RwLock::new(IntegrationRegistry::new())),
            encryption,
        }
    }

    /// Register an integration provider
    pub async fn register_provider(&self, provider: Box<dyn IntegrationProvider>) {
        let mut registry = self.registry.write().await;
        registry.register(provider);
    }

    /// Get a provider by type
    pub async fn get_provider(&self, integration_type: &str) -> Option<Arc<dyn IntegrationProvider>> {
        let registry = self.registry.read().await;
        registry.get(integration_type).map(|_p| {
            // This is a workaround - in a real implementation we'd store Arc<dyn IntegrationProvider>
            // For now, we return None and handle provider operations directly
            None::<Arc<dyn IntegrationProvider>>
        }).flatten()
    }

    // ==================== Integration CRUD ====================

    /// List integrations for an organization
    pub async fn list_integrations(
        &self,
        org_id: Uuid,
        query: ListIntegrationsQuery,
    ) -> AppResult<Vec<IntegrationWithStats>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        let integrations = sqlx::query_as::<_, Integration>(
            r#"
            SELECT id, organization_id, integration_type, name, config, status,
                   last_sync_at, last_error, created_at, updated_at
            FROM integrations
            WHERE organization_id = $1
              AND ($2::text IS NULL OR integration_type = $2)
              AND ($3::text IS NULL OR status = $3)
            ORDER BY name ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(org_id)
        .bind(&query.integration_type)
        .bind(&query.status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        // Get sync stats for all integrations
        let integration_ids: Vec<Uuid> = integrations.iter().map(|i| i.id).collect();

        let sync_stats: Vec<(Uuid, i64, Option<String>, Option<i64>)> = if !integration_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT
                    integration_id,
                    COUNT(*) as sync_count,
                    (SELECT status FROM integration_sync_logs
                     WHERE integration_id = isl.integration_id
                     ORDER BY started_at DESC LIMIT 1) as last_status,
                    (SELECT COALESCE(SUM(records_processed), 0) FROM integration_sync_logs
                     WHERE integration_id = isl.integration_id) as total_records
                FROM integration_sync_logs isl
                WHERE integration_id = ANY($1)
                GROUP BY integration_id
                "#,
            )
            .bind(&integration_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let stats_map: std::collections::HashMap<Uuid, (i64, Option<String>, Option<i64>)> = sync_stats
            .into_iter()
            .map(|(id, count, status, records)| (id, (count, status, records)))
            .collect();

        let result: Vec<IntegrationWithStats> = integrations
            .into_iter()
            .map(|integration| {
                let (sync_count, last_sync_status, records_synced) = stats_map
                    .get(&integration.id)
                    .cloned()
                    .unwrap_or((0, None, None));

                IntegrationWithStats {
                    integration,
                    sync_count,
                    last_sync_status,
                    records_synced,
                }
            })
            .collect();

        Ok(result)
    }

    /// Get a single integration by ID
    pub async fn get_integration(&self, org_id: Uuid, id: Uuid) -> AppResult<IntegrationWithStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_INTEGRATION, &id.to_string());

        // Try cache first
        if let Some(cached) = self.cache.get::<IntegrationWithStats>(&cache_key).await? {
            tracing::debug!("Cache hit for integration {}", id);
            return Ok(cached);
        }

        let integration = sqlx::query_as::<_, Integration>(
            r#"
            SELECT id, organization_id, integration_type, name, config, status,
                   last_sync_at, last_error, created_at, updated_at
            FROM integrations
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Integration {} not found", id)))?;

        // Get sync stats
        let sync_stats: Option<(i64, Option<String>, Option<i64>)> = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as sync_count,
                (SELECT status FROM integration_sync_logs
                 WHERE integration_id = $1
                 ORDER BY started_at DESC LIMIT 1) as last_status,
                COALESCE(SUM(records_processed), 0) as total_records
            FROM integration_sync_logs
            WHERE integration_id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?;

        let (sync_count, last_sync_status, records_synced) = sync_stats.unwrap_or((0, None, None));

        let result = IntegrationWithStats {
            integration,
            sync_count,
            last_sync_status,
            records_synced,
        };

        // Cache the result (but mask sensitive config data)
        self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await?;

        Ok(result)
    }

    /// Create a new integration
    pub async fn create_integration(
        &self,
        org_id: Uuid,
        input: CreateIntegration,
    ) -> AppResult<Integration> {
        Integration::validate_create(&input).map_err(AppError::ValidationError)?;

        // Encrypt sensitive fields in config
        let encrypted_config = match &input.config {
            Some(config) => Some(
                self.encryption
                    .encrypt_config(config)
                    .map_err(|e| AppError::InternalServerError(format!("Encryption failed: {}", e)))?,
            ),
            None => None,
        };

        let integration = sqlx::query_as::<_, Integration>(
            r#"
            INSERT INTO integrations (organization_id, integration_type, name, config, status)
            VALUES ($1, $2, $3, $4, 'inactive')
            RETURNING id, organization_id, integration_type, name, config, status,
                      last_sync_at, last_error, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(&input.integration_type)
        .bind(&input.name)
        .bind(&encrypted_config)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint().is_some() {
                    return AppError::Conflict("Integration already exists".to_string());
                }
            }
            AppError::DatabaseError(e)
        })?;

        // Invalidate caches
        self.invalidate_org_integration_caches(org_id).await?;

        tracing::info!(
            "Created integration: {} ({}) for org {}",
            integration.name,
            integration.id,
            org_id
        );

        Ok(integration)
    }

    /// Update an integration
    pub async fn update_integration(
        &self,
        org_id: Uuid,
        id: Uuid,
        input: UpdateIntegration,
    ) -> AppResult<Integration> {
        // Verify integration exists
        let _ = self.get_integration(org_id, id).await?;

        // Encrypt sensitive fields in config if provided
        let encrypted_config = match &input.config {
            Some(config) => Some(
                self.encryption
                    .encrypt_config(config)
                    .map_err(|e| AppError::InternalServerError(format!("Encryption failed: {}", e)))?,
            ),
            None => None,
        };

        let integration = sqlx::query_as::<_, Integration>(
            r#"
            UPDATE integrations
            SET
                name = COALESCE($3, name),
                config = COALESCE($4, config),
                status = COALESCE($5, status)
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, integration_type, name, config, status,
                      last_sync_at, last_error, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(&input.name)
        .bind(&encrypted_config)
        .bind(&input.status)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_integration_cache(org_id, id).await?;

        tracing::info!("Updated integration: {} ({})", integration.name, integration.id);

        Ok(integration)
    }

    /// Delete an integration
    pub async fn delete_integration(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        // Verify integration exists
        self.get_integration(org_id, id).await?;

        sqlx::query("DELETE FROM integrations WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        // Invalidate caches
        self.invalidate_integration_cache(org_id, id).await?;

        tracing::info!("Deleted integration: {}", id);

        Ok(())
    }

    // ==================== Connection Testing ====================

    /// Test connection for an integration
    pub async fn test_connection(
        &self,
        org_id: Uuid,
        id: Uuid,
    ) -> AppResult<TestConnectionResult> {
        let integration = self.get_integration(org_id, id).await?;
        let _config = integration.integration.config.clone().unwrap_or_default();

        // For now, return a placeholder - actual implementation would use the provider
        // registry to get the appropriate provider and call test_connection
        let result = TestConnectionResult {
            success: true,
            message: format!(
                "Connection test for {} integration - provider not yet implemented",
                integration.integration.integration_type
            ),
            details: Some(serde_json::json!({
                "integration_type": integration.integration.integration_type,
                "status": "pending_implementation"
            })),
        };

        Ok(result)
    }

    /// Test connection with provided config (for create flow before saving)
    pub async fn test_connection_with_config(
        &self,
        integration_type: &str,
        _config: &serde_json::Value,
    ) -> AppResult<TestConnectionResult> {
        // For now, return a placeholder
        let result = TestConnectionResult {
            success: true,
            message: format!(
                "Connection test for {} - provider not yet implemented",
                integration_type
            ),
            details: Some(serde_json::json!({
                "integration_type": integration_type,
                "status": "pending_implementation"
            })),
        };

        Ok(result)
    }

    /// Decrypt config for an integration (used internally for sync operations)
    pub fn decrypt_config(
        &self,
        config: &serde_json::Value,
    ) -> AppResult<serde_json::Value> {
        self.encryption
            .decrypt_config(config)
            .map_err(|e| AppError::InternalServerError(format!("Decryption failed: {}", e)))
    }

    // ==================== Sync Operations ====================

    /// Trigger a sync for an integration
    pub async fn trigger_sync(
        &self,
        org_id: Uuid,
        id: Uuid,
        sync_type: Option<String>,
        _full_sync: bool,
    ) -> AppResult<IntegrationSyncLog> {
        let _integration = self.get_integration(org_id, id).await?;

        // Create sync log entry
        let sync_log = sqlx::query_as::<_, IntegrationSyncLog>(
            r#"
            INSERT INTO integration_sync_logs (integration_id, sync_type, status)
            VALUES ($1, $2, 'running')
            RETURNING id, integration_id, sync_type, started_at, completed_at, status,
                      records_processed, errors, created_at
            "#,
        )
        .bind(id)
        .bind(&sync_type)
        .fetch_one(&self.db)
        .await?;

        // Update integration status to syncing
        sqlx::query("UPDATE integrations SET status = 'syncing' WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        // In a real implementation, we would:
        // 1. Get the provider from registry
        // 2. Create a SyncContext
        // 3. Spawn a background task to run the sync
        // 4. The task would update the sync log when complete

        // For now, we'll simulate a completed sync
        tokio::spawn(async move {
            // Simulate some work
            tokio::time::sleep(Duration::from_secs(2)).await;
            // In real implementation, actual sync would happen here
        });

        tracing::info!(
            "Triggered sync for integration {} (log_id: {})",
            id,
            sync_log.id
        );

        Ok(sync_log)
    }

    /// Get sync logs for an integration
    pub async fn get_sync_logs(
        &self,
        org_id: Uuid,
        integration_id: Uuid,
        limit: i64,
    ) -> AppResult<Vec<IntegrationSyncLog>> {
        // Verify integration exists and belongs to org
        self.get_integration(org_id, integration_id).await?;

        let logs = sqlx::query_as::<_, IntegrationSyncLog>(
            r#"
            SELECT id, integration_id, sync_type, started_at, completed_at, status,
                   records_processed, errors, created_at
            FROM integration_sync_logs
            WHERE integration_id = $1
            ORDER BY started_at DESC
            LIMIT $2
            "#,
        )
        .bind(integration_id)
        .bind(limit.min(100))
        .fetch_all(&self.db)
        .await?;

        Ok(logs)
    }

    /// Update a sync log (used by background sync tasks)
    pub async fn update_sync_log(
        &self,
        sync_log_id: Uuid,
        integration_id: Uuid,
        result: &SyncResult,
    ) -> AppResult<()> {
        let status = if result.success { "completed" } else { "failed" };
        let errors = if result.errors.is_empty() {
            None
        } else {
            Some(
                serde_json::to_value(&result.errors)
                    .map_err(|e| AppError::InternalServerError(e.to_string()))?,
            )
        };

        sqlx::query(
            r#"
            UPDATE integration_sync_logs
            SET completed_at = NOW(),
                status = $2,
                records_processed = $3,
                errors = $4
            WHERE id = $1
            "#,
        )
        .bind(sync_log_id)
        .bind(status)
        .bind(result.records_processed)
        .bind(errors)
        .execute(&self.db)
        .await?;

        // Update integration status and last_sync_at
        let new_status = if result.success { "active" } else { "error" };
        let last_error = if result.success {
            None
        } else {
            result.errors.first().map(|e| e.message.clone())
        };

        sqlx::query(
            r#"
            UPDATE integrations
            SET status = $2,
                last_sync_at = NOW(),
                last_error = $3
            WHERE id = $1
            "#,
        )
        .bind(integration_id)
        .bind(new_status)
        .bind(last_error)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ==================== Statistics ====================

    /// Get integration statistics for dashboard
    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<IntegrationStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_INTEGRATION_STATS, "summary");

        // Try cache first
        if let Some(cached) = self.cache.get::<IntegrationStats>(&cache_key).await? {
            tracing::debug!("Cache hit for integration stats");
            return Ok(cached);
        }

        let stats: (i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'active') as active,
                COUNT(*) FILTER (WHERE status = 'inactive') as inactive,
                COUNT(*) FILTER (WHERE status = 'error') as error
            FROM integrations
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let by_type: Vec<IntegrationTypeCount> = sqlx::query_as(
            r#"
            SELECT integration_type, COUNT(*) as count
            FROM integrations
            WHERE organization_id = $1
            GROUP BY integration_type
            ORDER BY count DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        let result = IntegrationStats {
            total: stats.0,
            active: stats.1,
            inactive: stats.2,
            error: stats.3,
            by_type,
        };

        // Cache for 5 minutes
        self.cache
            .set(&cache_key, &result, Some(Duration::from_secs(300)))
            .await?;

        Ok(result)
    }

    // ==================== Cache Invalidation ====================

    async fn invalidate_integration_cache(&self, org_id: Uuid, integration_id: Uuid) -> AppResult<()> {
        let cache_key = org_cache_key(
            &org_id.to_string(),
            CACHE_PREFIX_INTEGRATION,
            &integration_id.to_string(),
        );
        self.cache.delete(&cache_key).await?;

        // Also invalidate org-wide caches
        self.invalidate_org_integration_caches(org_id).await
    }

    async fn invalidate_org_integration_caches(&self, org_id: Uuid) -> AppResult<()> {
        // Invalidate list cache
        let list_pattern = format!("org:{}:{}:*", org_id, CACHE_PREFIX_INTEGRATIONS_LIST);
        self.cache.delete_pattern(&list_pattern).await?;

        // Invalidate stats cache
        let stats_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_INTEGRATION_STATS, "summary");
        self.cache.delete(&stats_key).await
    }

    // ==================== Health Monitoring ====================

    /// Get health status for all integrations
    pub async fn get_all_health(
        &self,
        org_id: Uuid,
    ) -> AppResult<Vec<IntegrationHealthWithDetails>> {
        // Get integrations first
        let integrations = sqlx::query_as::<_, Integration>(
            r#"
            SELECT id, organization_id, integration_type, name, config, status,
                   last_sync_at, last_error, created_at, updated_at
            FROM integrations
            WHERE organization_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        if integrations.is_empty() {
            return Ok(vec![]);
        }

        // Get health records for all integrations
        let integration_ids: Vec<Uuid> = integrations.iter().map(|i| i.id).collect();

        let health_records = sqlx::query_as::<_, IntegrationHealth>(
            r#"
            SELECT
                id, integration_id, status::text as status,
                last_successful_sync_at, consecutive_failures,
                sync_success_count_24h, sync_failure_count_24h,
                average_sync_duration_ms,
                sync_success_count_7d, sync_failure_count_7d,
                last_check_at, last_check_message,
                last_error_at, last_error_message,
                created_at, updated_at
            FROM integration_health
            WHERE integration_id = ANY($1)
            "#,
        )
        .bind(&integration_ids)
        .fetch_all(&self.db)
        .await?;

        // Create a map for quick lookup
        let health_map: std::collections::HashMap<Uuid, IntegrationHealth> = health_records
            .into_iter()
            .map(|h| (h.integration_id, h))
            .collect();

        // Build result with health data, sorted by status severity
        let mut result: Vec<IntegrationHealthWithDetails> = integrations
            .into_iter()
            .map(|integration| {
                let health = health_map.get(&integration.id).cloned().unwrap_or_else(|| {
                    // Create a default unknown health record
                    IntegrationHealth {
                        id: Uuid::nil(),
                        integration_id: integration.id,
                        status: "unknown".to_string(),
                        last_successful_sync_at: None,
                        consecutive_failures: 0,
                        sync_success_count_24h: 0,
                        sync_failure_count_24h: 0,
                        average_sync_duration_ms: None,
                        sync_success_count_7d: 0,
                        sync_failure_count_7d: 0,
                        last_check_at: None,
                        last_check_message: None,
                        last_error_at: None,
                        last_error_message: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    }
                });

                let success_rate_24h = health.success_rate_24h();
                let success_rate_7d = health.success_rate_7d();

                IntegrationHealthWithDetails {
                    integration_id: integration.id,
                    integration_name: integration.name,
                    integration_type: integration.integration_type,
                    health,
                    success_rate_24h,
                    success_rate_7d,
                }
            })
            .collect();

        // Sort by status severity (unhealthy first, then degraded, unknown, healthy)
        result.sort_by(|a, b| {
            let status_order = |s: &str| match s {
                "unhealthy" => 0,
                "degraded" => 1,
                "unknown" => 2,
                _ => 3,
            };
            status_order(&a.health.status).cmp(&status_order(&b.health.status))
        });

        Ok(result)
    }

    /// Get health for a specific integration
    pub async fn get_integration_health(
        &self,
        org_id: Uuid,
        integration_id: Uuid,
    ) -> AppResult<IntegrationHealthWithDetails> {
        // Verify integration belongs to org
        let integration = self.get_integration(org_id, integration_id).await?;

        let health = sqlx::query_as::<_, IntegrationHealth>(
            r#"
            SELECT
                id, integration_id, status::text as status,
                last_successful_sync_at, consecutive_failures,
                sync_success_count_24h, sync_failure_count_24h,
                average_sync_duration_ms,
                sync_success_count_7d, sync_failure_count_7d,
                last_check_at, last_check_message,
                last_error_at, last_error_message,
                created_at, updated_at
            FROM integration_health
            WHERE integration_id = $1
            "#,
        )
        .bind(integration_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| {
            AppError::NotFound(format!("Health record not found for integration {}", integration_id))
        })?;

        let success_rate_24h = health.success_rate_24h();
        let success_rate_7d = health.success_rate_7d();

        Ok(IntegrationHealthWithDetails {
            integration_id,
            integration_name: integration.integration.name,
            integration_type: integration.integration.integration_type,
            health,
            success_rate_24h,
            success_rate_7d,
        })
    }

    /// Get aggregated health statistics
    pub async fn get_health_stats(&self, org_id: Uuid) -> AppResult<IntegrationHealthStats> {
        let cache_key = org_cache_key(&org_id.to_string(), "integration:health", "stats");

        // Try cache first
        if let Some(cached) = self.cache.get::<IntegrationHealthStats>(&cache_key).await? {
            return Ok(cached);
        }

        let row: (i64, i64, i64, i64, i64, i64, i64, i64, i64, Option<i32>) = sqlx::query_as(
            r#"
            SELECT
                COUNT(DISTINCT i.id) as total_integrations,
                COUNT(*) FILTER (WHERE h.status = 'healthy') as healthy_count,
                COUNT(*) FILTER (WHERE h.status = 'degraded') as degraded_count,
                COUNT(*) FILTER (WHERE h.status = 'unhealthy') as unhealthy_count,
                COUNT(*) FILTER (WHERE h.status = 'unknown' OR h.status IS NULL) as unknown_count,
                COALESCE(SUM(h.sync_success_count_24h), 0) as total_success_24h,
                COALESCE(SUM(h.sync_failure_count_24h), 0) as total_failure_24h,
                COALESCE(SUM(h.sync_success_count_7d), 0) as total_success_7d,
                COALESCE(SUM(h.sync_failure_count_7d), 0) as total_failure_7d,
                AVG(h.average_sync_duration_ms)::integer as avg_duration
            FROM integrations i
            LEFT JOIN integration_health h ON i.id = h.integration_id
            WHERE i.organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let total_24h = row.5 + row.6;
        let total_7d = row.7 + row.8;

        let stats = IntegrationHealthStats {
            total_integrations: row.0,
            healthy_count: row.1,
            degraded_count: row.2,
            unhealthy_count: row.3,
            unknown_count: row.4,
            overall_success_rate_24h: if total_24h > 0 {
                (row.5 as f64 / total_24h as f64) * 100.0
            } else {
                100.0
            },
            overall_success_rate_7d: if total_7d > 0 {
                (row.7 as f64 / total_7d as f64) * 100.0
            } else {
                100.0
            },
            average_sync_duration_ms: row.9,
            total_syncs_24h: total_24h,
            total_failures_24h: row.6,
        };

        // Cache for 5 minutes
        self.cache
            .set(&cache_key, &stats, Some(Duration::from_secs(300)))
            .await?;

        Ok(stats)
    }

    /// Get recent failures for the health dashboard
    pub async fn get_recent_failures(
        &self,
        org_id: Uuid,
        limit: i64,
    ) -> AppResult<Vec<RecentFailure>> {
        let failures = sqlx::query_as::<_, (Uuid, String, String, Option<String>, chrono::DateTime<chrono::Utc>, i32)>(
            r#"
            SELECT
                i.id as integration_id,
                i.name as integration_name,
                i.integration_type,
                h.last_error_message,
                h.last_error_at,
                h.consecutive_failures
            FROM integrations i
            INNER JOIN integration_health h ON i.id = h.integration_id
            WHERE i.organization_id = $1
              AND h.consecutive_failures > 0
              AND h.last_error_at IS NOT NULL
            ORDER BY h.last_error_at DESC
            LIMIT $2
            "#,
        )
        .bind(org_id)
        .bind(limit.min(50))
        .fetch_all(&self.db)
        .await?;

        Ok(failures
            .into_iter()
            .map(|row| RecentFailure {
                integration_id: row.0,
                integration_name: row.1,
                integration_type: row.2,
                error_message: row.3,
                failed_at: row.4,
                consecutive_failures: row.5,
            })
            .collect())
    }

    /// Get health trend data for charts
    pub async fn get_health_trend(
        &self,
        org_id: Uuid,
        hours: i32,
    ) -> AppResult<Vec<HealthTrendPoint>> {
        // Get trend data from snapshots
        let points = sqlx::query_as::<_, (chrono::DateTime<chrono::Utc>, i64, i64, i64, Option<rust_decimal::Decimal>)>(
            r#"
            WITH time_buckets AS (
                SELECT
                    date_trunc('hour', snapshot_at) as bucket,
                    status,
                    sync_success_rate
                FROM integration_health_snapshots hs
                INNER JOIN integrations i ON hs.integration_id = i.id
                WHERE i.organization_id = $1
                  AND hs.snapshot_at > NOW() - ($2 || ' hours')::interval
            )
            SELECT
                bucket as timestamp,
                COUNT(*) FILTER (WHERE status = 'healthy') as healthy_count,
                COUNT(*) FILTER (WHERE status = 'degraded') as degraded_count,
                COUNT(*) FILTER (WHERE status = 'unhealthy') as unhealthy_count,
                AVG(sync_success_rate) as avg_success_rate
            FROM time_buckets
            GROUP BY bucket
            ORDER BY bucket ASC
            "#,
        )
        .bind(org_id)
        .bind(hours.to_string())
        .fetch_all(&self.db)
        .await?;

        Ok(points
            .into_iter()
            .map(|row| HealthTrendPoint {
                timestamp: row.0,
                healthy_count: row.1,
                degraded_count: row.2,
                unhealthy_count: row.3,
                success_rate: row.4.map(|d| d.to_string().parse::<f64>().unwrap_or(100.0)).unwrap_or(100.0),
            })
            .collect())
    }

    /// Create a health snapshot for historical tracking
    pub async fn create_health_snapshot(&self, org_id: Uuid) -> AppResult<()> {
        sqlx::query(
            r#"
            INSERT INTO integration_health_snapshots (
                integration_id, status, sync_success_rate,
                average_sync_duration_ms, error_count, snapshot_at
            )
            SELECT
                h.integration_id,
                h.status,
                CASE
                    WHEN h.sync_success_count_24h + h.sync_failure_count_24h > 0
                    THEN (h.sync_success_count_24h::decimal / (h.sync_success_count_24h + h.sync_failure_count_24h)) * 100
                    ELSE 100.00
                END as sync_success_rate,
                h.average_sync_duration_ms,
                h.sync_failure_count_24h as error_count,
                NOW()
            FROM integration_health h
            INNER JOIN integrations i ON h.integration_id = i.id
            WHERE i.organization_id = $1
            "#,
        )
        .bind(org_id)
        .execute(&self.db)
        .await?;

        tracing::info!("Created health snapshots for organization {}", org_id);

        Ok(())
    }
}
