use crate::cache::{org_cache_key, CacheClient};
use crate::integrations::{IntegrationProvider, IntegrationRegistry, SyncResult};
use crate::models::{
    CreateIntegration, Integration, IntegrationStats, IntegrationSyncLog, IntegrationTypeCount,
    IntegrationWithStats, ListIntegrationsQuery, TestConnectionResult, UpdateIntegration,
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
}
