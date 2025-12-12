use crate::cache::{org_cache_key, CacheClient};
use crate::models::{
    Control, ControlRequirementMapping, ControlStats, ControlTest, ControlTestResult,
    ControlWithMappings, CreateControl, CreateControlTest, CreateTestResult,
    ListControlsQuery, MappedRequirement, UpdateControl,
};
use crate::utils::{AppError, AppResult};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(1800); // 30 minutes
const CACHE_PREFIX_CONTROL: &str = "control";
const CACHE_PREFIX_CONTROLS_LIST: &str = "controls:list";
const CACHE_PREFIX_CONTROL_STATS: &str = "controls:stats";

#[derive(Clone)]
pub struct ControlService {
    db: PgPool,
    cache: CacheClient,
}

impl ControlService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Control CRUD ====================

    /// List controls for an organization with filtering
    pub async fn list_controls(
        &self,
        org_id: Uuid,
        query: ListControlsQuery,
    ) -> AppResult<Vec<ControlWithMappings>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        // Build dynamic query based on filters
        let controls = if let Some(ref search) = query.search {
            let search_pattern = format!("%{}%", search.to_lowercase());
            sqlx::query_as::<_, Control>(
                r#"
                SELECT id, organization_id, code, name, description, control_type,
                       frequency, owner_id, status, implementation_notes, created_at, updated_at
                FROM controls
                WHERE organization_id = $1
                  AND (LOWER(code) LIKE $2 OR LOWER(name) LIKE $2 OR LOWER(description) LIKE $2)
                  AND ($3::text IS NULL OR status = $3)
                  AND ($4::text IS NULL OR control_type = $4)
                  AND ($5::uuid IS NULL OR owner_id = $5)
                ORDER BY code ASC
                LIMIT $6 OFFSET $7
                "#,
            )
            .bind(org_id)
            .bind(&search_pattern)
            .bind(&query.status)
            .bind(&query.control_type)
            .bind(query.owner_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        } else {
            sqlx::query_as::<_, Control>(
                r#"
                SELECT id, organization_id, code, name, description, control_type,
                       frequency, owner_id, status, implementation_notes, created_at, updated_at
                FROM controls
                WHERE organization_id = $1
                  AND ($2::text IS NULL OR status = $2)
                  AND ($3::text IS NULL OR control_type = $3)
                  AND ($4::uuid IS NULL OR owner_id = $4)
                ORDER BY code ASC
                LIMIT $5 OFFSET $6
                "#,
            )
            .bind(org_id)
            .bind(&query.status)
            .bind(&query.control_type)
            .bind(query.owner_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.db)
            .await?
        };

        // Get requirement counts for all controls in one query
        let control_ids: Vec<Uuid> = controls.iter().map(|c| c.id).collect();

        let counts: Vec<(Uuid, i64)> = if !control_ids.is_empty() {
            sqlx::query_as(
                r#"
                SELECT control_id, COUNT(*) as count
                FROM control_requirement_mappings
                WHERE control_id = ANY($1)
                GROUP BY control_id
                "#,
            )
            .bind(&control_ids)
            .fetch_all(&self.db)
            .await?
        } else {
            vec![]
        };

        let count_map: std::collections::HashMap<Uuid, i64> = counts.into_iter().collect();

        let result: Vec<ControlWithMappings> = controls
            .into_iter()
            .map(|control| {
                let count = count_map.get(&control.id).copied().unwrap_or(0);
                ControlWithMappings {
                    control,
                    requirement_count: count,
                    mapped_requirements: None,
                }
            })
            .collect();

        Ok(result)
    }

    /// Get a single control by ID with mapped requirements
    pub async fn get_control(&self, org_id: Uuid, id: Uuid) -> AppResult<ControlWithMappings> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_CONTROL, &id.to_string());

        // Try cache first
        if let Some(cached) = self.cache.get::<ControlWithMappings>(&cache_key).await? {
            tracing::debug!("Cache hit for control {}", id);
            return Ok(cached);
        }

        let control = sqlx::query_as::<_, Control>(
            r#"
            SELECT id, organization_id, code, name, description, control_type,
                   frequency, owner_id, status, implementation_notes, created_at, updated_at
            FROM controls
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Control {} not found", id)))?;

        // Get mapped requirements
        let mapped_requirements = sqlx::query_as::<_, MappedRequirement>(
            r#"
            SELECT fr.id, fr.framework_id, f.name as framework_name, fr.code, fr.name
            FROM control_requirement_mappings crm
            JOIN framework_requirements fr ON crm.framework_requirement_id = fr.id
            JOIN frameworks f ON fr.framework_id = f.id
            WHERE crm.control_id = $1
            ORDER BY f.name, fr.code
            "#,
        )
        .bind(id)
        .fetch_all(&self.db)
        .await?;

        let requirement_count = mapped_requirements.len() as i64;

        let result = ControlWithMappings {
            control,
            requirement_count,
            mapped_requirements: Some(mapped_requirements),
        };

        // Cache the result
        self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await?;

        Ok(result)
    }

    /// Create a new control
    pub async fn create_control(&self, org_id: Uuid, input: CreateControl) -> AppResult<Control> {
        Control::validate_create(&input).map_err(|e| AppError::ValidationError(e))?;

        let control = sqlx::query_as::<_, Control>(
            r#"
            INSERT INTO controls (organization_id, code, name, description, control_type,
                                  frequency, owner_id, status, implementation_notes)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, organization_id, code, name, description, control_type,
                      frequency, owner_id, status, implementation_notes, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(&input.code)
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.control_type.as_deref().unwrap_or("preventive"))
        .bind(input.frequency.as_deref().unwrap_or("continuous"))
        .bind(input.owner_id)
        .bind(input.status.as_deref().unwrap_or("not_implemented"))
        .bind(&input.implementation_notes)
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("controls_organization_id_code_key") {
                    return AppError::Conflict(format!(
                        "Control with code '{}' already exists",
                        input.code
                    ));
                }
            }
            AppError::DatabaseError(e)
        })?;

        // Invalidate caches
        self.invalidate_org_control_caches(org_id).await?;

        tracing::info!("Created control: {} ({})", control.code, control.id);

        Ok(control)
    }

    /// Update a control
    pub async fn update_control(
        &self,
        org_id: Uuid,
        id: Uuid,
        input: UpdateControl,
    ) -> AppResult<Control> {
        // Verify control exists and belongs to org
        let _ = self.get_control(org_id, id).await?;

        let control = sqlx::query_as::<_, Control>(
            r#"
            UPDATE controls
            SET
                code = COALESCE($3, code),
                name = COALESCE($4, name),
                description = COALESCE($5, description),
                control_type = COALESCE($6, control_type),
                frequency = COALESCE($7, frequency),
                owner_id = COALESCE($8, owner_id),
                status = COALESCE($9, status),
                implementation_notes = COALESCE($10, implementation_notes)
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, code, name, description, control_type,
                      frequency, owner_id, status, implementation_notes, created_at, updated_at
            "#,
        )
        .bind(id)
        .bind(org_id)
        .bind(&input.code)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.control_type)
        .bind(&input.frequency)
        .bind(input.owner_id)
        .bind(&input.status)
        .bind(&input.implementation_notes)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_control_cache(org_id, id).await?;

        tracing::info!("Updated control: {} ({})", control.code, control.id);

        Ok(control)
    }

    /// Delete a control
    pub async fn delete_control(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        // Verify control exists
        self.get_control(org_id, id).await?;

        // Check for evidence links
        let evidence_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM evidence_control_links WHERE control_id = $1",
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        if evidence_count.0 > 0 {
            return Err(AppError::Conflict(format!(
                "Cannot delete control with {} linked evidence items",
                evidence_count.0
            )));
        }

        sqlx::query("DELETE FROM controls WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        // Invalidate caches
        self.invalidate_control_cache(org_id, id).await?;

        tracing::info!("Deleted control: {}", id);

        Ok(())
    }

    // ==================== Requirement Mappings ====================

    /// Map a control to framework requirements
    pub async fn map_requirements(
        &self,
        org_id: Uuid,
        control_id: Uuid,
        requirement_ids: Vec<Uuid>,
    ) -> AppResult<Vec<ControlRequirementMapping>> {
        // Verify control exists
        self.get_control(org_id, control_id).await?;

        let mut tx = self.db.begin().await?;
        let mut mappings = Vec::new();

        for req_id in requirement_ids {
            // Check if mapping already exists
            let existing: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM control_requirement_mappings WHERE control_id = $1 AND framework_requirement_id = $2",
            )
            .bind(control_id)
            .bind(req_id)
            .fetch_optional(&mut *tx)
            .await?;

            if existing.is_none() {
                let mapping = sqlx::query_as::<_, ControlRequirementMapping>(
                    r#"
                    INSERT INTO control_requirement_mappings (control_id, framework_requirement_id)
                    VALUES ($1, $2)
                    RETURNING id, control_id, framework_requirement_id, created_at
                    "#,
                )
                .bind(control_id)
                .bind(req_id)
                .fetch_one(&mut *tx)
                .await?;

                mappings.push(mapping);
            }
        }

        tx.commit().await?;

        // Invalidate control cache
        self.invalidate_control_cache(org_id, control_id).await?;

        tracing::info!(
            "Mapped {} requirements to control {}",
            mappings.len(),
            control_id
        );

        Ok(mappings)
    }

    /// Remove requirement mappings from a control
    pub async fn unmap_requirements(
        &self,
        org_id: Uuid,
        control_id: Uuid,
        requirement_ids: Vec<Uuid>,
    ) -> AppResult<i64> {
        // Verify control exists
        self.get_control(org_id, control_id).await?;

        let result = sqlx::query(
            "DELETE FROM control_requirement_mappings WHERE control_id = $1 AND framework_requirement_id = ANY($2)",
        )
        .bind(control_id)
        .bind(&requirement_ids)
        .execute(&self.db)
        .await?;

        let deleted = result.rows_affected() as i64;

        // Invalidate control cache
        self.invalidate_control_cache(org_id, control_id).await?;

        tracing::info!(
            "Unmapped {} requirements from control {}",
            deleted,
            control_id
        );

        Ok(deleted)
    }

    // ==================== Control Tests ====================

    /// List tests for a control
    pub async fn list_tests(&self, org_id: Uuid, control_id: Uuid) -> AppResult<Vec<ControlTest>> {
        // Verify control exists
        self.get_control(org_id, control_id).await?;

        let tests = sqlx::query_as::<_, ControlTest>(
            r#"
            SELECT id, control_id, name, description, test_type, automation_config,
                   frequency, next_due_at, created_at
            FROM control_tests
            WHERE control_id = $1
            ORDER BY name ASC
            "#,
        )
        .bind(control_id)
        .fetch_all(&self.db)
        .await?;

        Ok(tests)
    }

    /// Create a control test
    pub async fn create_test(
        &self,
        org_id: Uuid,
        control_id: Uuid,
        input: CreateControlTest,
    ) -> AppResult<ControlTest> {
        // Verify control exists
        self.get_control(org_id, control_id).await?;

        ControlTest::validate_create(&input).map_err(|e| AppError::ValidationError(e))?;

        let test = sqlx::query_as::<_, ControlTest>(
            r#"
            INSERT INTO control_tests (control_id, name, description, test_type,
                                       automation_config, frequency, next_due_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, control_id, name, description, test_type, automation_config,
                      frequency, next_due_at, created_at
            "#,
        )
        .bind(control_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(input.test_type.as_deref().unwrap_or("manual"))
        .bind(&input.automation_config)
        .bind(&input.frequency)
        .bind(input.next_due_at)
        .fetch_one(&self.db)
        .await?;

        tracing::info!("Created control test: {} ({})", test.name, test.id);

        Ok(test)
    }

    /// Record a test result
    pub async fn record_test_result(
        &self,
        org_id: Uuid,
        control_id: Uuid,
        test_id: Uuid,
        user_id: Uuid,
        input: CreateTestResult,
    ) -> AppResult<ControlTestResult> {
        // Verify control and test exist
        self.get_control(org_id, control_id).await?;

        let result = sqlx::query_as::<_, ControlTestResult>(
            r#"
            INSERT INTO control_test_results (control_test_id, performed_by, status, notes, evidence_ids)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, control_test_id, performed_by, performed_at, status, notes, evidence_ids, created_at
            "#,
        )
        .bind(test_id)
        .bind(user_id)
        .bind(&input.status)
        .bind(&input.notes)
        .bind(&input.evidence_ids)
        .fetch_one(&self.db)
        .await?;

        tracing::info!("Recorded test result: {} for test {}", result.id, test_id);

        Ok(result)
    }

    // ==================== Statistics ====================

    /// Get control statistics for dashboard
    pub async fn get_stats(&self, org_id: Uuid) -> AppResult<ControlStats> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_CONTROL_STATS, "summary");

        // Try cache first
        if let Some(cached) = self.cache.get::<ControlStats>(&cache_key).await? {
            tracing::debug!("Cache hit for control stats");
            return Ok(cached);
        }

        let stats: (i64, i64, i64, i64, i64) = sqlx::query_as(
            r#"
            SELECT
                COUNT(*) as total,
                COUNT(*) FILTER (WHERE status = 'implemented') as implemented,
                COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress,
                COUNT(*) FILTER (WHERE status = 'not_implemented') as not_implemented,
                COUNT(*) FILTER (WHERE status = 'not_applicable') as not_applicable
            FROM controls
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        let implementation_percentage = if stats.0 > 0 {
            (stats.1 as f64 / (stats.0 - stats.4) as f64) * 100.0
        } else {
            0.0
        };

        let result = ControlStats {
            total: stats.0,
            implemented: stats.1,
            in_progress: stats.2,
            not_implemented: stats.3,
            not_applicable: stats.4,
            implementation_percentage,
        };

        // Cache for 5 minutes
        self.cache.set(&cache_key, &result, Some(Duration::from_secs(300))).await?;

        Ok(result)
    }

    // ==================== Cache Invalidation ====================

    async fn invalidate_control_cache(&self, org_id: Uuid, control_id: Uuid) -> AppResult<()> {
        let cache_key = org_cache_key(
            &org_id.to_string(),
            CACHE_PREFIX_CONTROL,
            &control_id.to_string(),
        );
        self.cache.delete(&cache_key).await?;

        // Also invalidate org-wide caches
        self.invalidate_org_control_caches(org_id).await
    }

    async fn invalidate_org_control_caches(&self, org_id: Uuid) -> AppResult<()> {
        // Invalidate list cache
        let list_pattern = format!("org:{}:{}:*", org_id, CACHE_PREFIX_CONTROLS_LIST);
        self.cache.delete_pattern(&list_pattern).await?;

        // Invalidate stats cache
        let stats_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_CONTROL_STATS, "summary");
        self.cache.delete(&stats_key).await
    }
}
