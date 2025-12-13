use crate::cache::{cache_key, CacheClient};
use crate::models::{
    CreateFramework, CreateFrameworkRequirement, Framework, FrameworkRequirement,
    FrameworkWithRequirements, UpdateFramework, UpdateFrameworkRequirement,
    FrameworkGapAnalysis, CategoryGapAnalysis, RequirementGapAnalysis,
};
use crate::utils::{AppError, AppResult};
use sqlx::PgPool;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(3600); // 1 hour
const CACHE_PREFIX_FRAMEWORK: &str = "framework";
const CACHE_PREFIX_FRAMEWORKS_LIST: &str = "frameworks:list";
const CACHE_PREFIX_REQUIREMENT: &str = "framework_req";
const CACHE_PREFIX_REQUIREMENTS_LIST: &str = "framework_reqs";

#[derive(Clone)]
pub struct FrameworkService {
    db: PgPool,
    cache: CacheClient,
}

impl FrameworkService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Framework CRUD ====================

    /// List all frameworks with optional filtering
    pub async fn list_frameworks(
        &self,
        category: Option<&str>,
        is_system: Option<bool>,
    ) -> AppResult<Vec<Framework>> {
        // Build cache key based on filters
        let cache_key = format!(
            "{}:cat={:?}:sys={:?}",
            CACHE_PREFIX_FRAMEWORKS_LIST, category, is_system
        );

        // Try cache first
        if let Some(cached) = self.cache.get::<Vec<Framework>>(&cache_key).await? {
            tracing::debug!("Cache hit for frameworks list");
            return Ok(cached);
        }

        // Build query dynamically based on filters
        let frameworks = match (category, is_system) {
            (Some(cat), Some(sys)) => {
                sqlx::query_as::<_, Framework>(
                    r#"
                    SELECT id, name, version, description, category, is_system, created_at
                    FROM frameworks
                    WHERE category = $1 AND is_system = $2
                    ORDER BY is_system DESC, name ASC
                    "#,
                )
                .bind(cat)
                .bind(sys)
                .fetch_all(&self.db)
                .await?
            }
            (Some(cat), None) => {
                sqlx::query_as::<_, Framework>(
                    r#"
                    SELECT id, name, version, description, category, is_system, created_at
                    FROM frameworks
                    WHERE category = $1
                    ORDER BY is_system DESC, name ASC
                    "#,
                )
                .bind(cat)
                .fetch_all(&self.db)
                .await?
            }
            (None, Some(sys)) => {
                sqlx::query_as::<_, Framework>(
                    r#"
                    SELECT id, name, version, description, category, is_system, created_at
                    FROM frameworks
                    WHERE is_system = $1
                    ORDER BY is_system DESC, name ASC
                    "#,
                )
                .bind(sys)
                .fetch_all(&self.db)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, Framework>(
                    r#"
                    SELECT id, name, version, description, category, is_system, created_at
                    FROM frameworks
                    ORDER BY is_system DESC, name ASC
                    "#,
                )
                .fetch_all(&self.db)
                .await?
            }
        };

        // Cache the result
        self.cache.set(&cache_key, &frameworks, Some(CACHE_TTL)).await?;

        Ok(frameworks)
    }

    /// Get a single framework by ID
    pub async fn get_framework(&self, id: Uuid) -> AppResult<Framework> {
        let cache_key = cache_key(CACHE_PREFIX_FRAMEWORK, &id.to_string());

        // Try cache first
        if let Some(cached) = self.cache.get::<Framework>(&cache_key).await? {
            tracing::debug!("Cache hit for framework {}", id);
            return Ok(cached);
        }

        let framework = sqlx::query_as::<_, Framework>(
            r#"
            SELECT id, name, version, description, category, is_system, created_at
            FROM frameworks
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Framework {} not found", id)))?;

        // Cache the result
        self.cache.set(&cache_key, &framework, Some(CACHE_TTL)).await?;

        Ok(framework)
    }

    /// Get framework with all requirements and count
    pub async fn get_framework_with_requirements(&self, id: Uuid) -> AppResult<FrameworkWithRequirements> {
        let cache_key = format!("{}:with_reqs:{}", CACHE_PREFIX_FRAMEWORK, id);

        // Try cache first
        if let Some(cached) = self.cache.get::<FrameworkWithRequirements>(&cache_key).await? {
            tracing::debug!("Cache hit for framework with requirements {}", id);
            return Ok(cached);
        }

        // Fetch framework and requirements in parallel for optimization
        let (framework, requirements) = tokio::try_join!(
            self.get_framework(id),
            self.list_requirements(id)
        )?;

        let requirement_count = requirements.len() as i64;

        let result = FrameworkWithRequirements {
            framework,
            requirements,
            requirement_count,
        };

        // Cache the result
        self.cache.set(&cache_key, &result, Some(CACHE_TTL)).await?;

        Ok(result)
    }

    /// Create a new framework
    pub async fn create_framework(&self, input: CreateFramework) -> AppResult<Framework> {
        Framework::validate_create(&input)
            .map_err(|e| AppError::ValidationError(e))?;

        let framework = sqlx::query_as::<_, Framework>(
            r#"
            INSERT INTO frameworks (name, version, description, category, is_system)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, name, version, description, category, is_system, created_at
            "#,
        )
        .bind(&input.name)
        .bind(&input.version)
        .bind(&input.description)
        .bind(&input.category)
        .bind(input.is_system.unwrap_or(false))
        .fetch_one(&self.db)
        .await?;

        // Invalidate list cache
        self.invalidate_framework_list_cache().await?;

        // Cache the new framework
        let cache_key = cache_key(CACHE_PREFIX_FRAMEWORK, &framework.id.to_string());
        self.cache.set(&cache_key, &framework, Some(CACHE_TTL)).await?;

        tracing::info!("Created framework: {} ({})", framework.name, framework.id);

        Ok(framework)
    }

    /// Update an existing framework
    pub async fn update_framework(&self, id: Uuid, input: UpdateFramework) -> AppResult<Framework> {
        // Check if framework exists and is not system
        let existing = self.get_framework(id).await?;
        if existing.is_system {
            return Err(AppError::Forbidden(
                "Cannot modify system frameworks".to_string(),
            ));
        }

        let framework = sqlx::query_as::<_, Framework>(
            r#"
            UPDATE frameworks
            SET
                name = COALESCE($2, name),
                version = COALESCE($3, version),
                description = COALESCE($4, description),
                category = COALESCE($5, category)
            WHERE id = $1
            RETURNING id, name, version, description, category, is_system, created_at
            "#,
        )
        .bind(id)
        .bind(&input.name)
        .bind(&input.version)
        .bind(&input.description)
        .bind(&input.category)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_framework_cache(id).await?;

        // Cache updated framework
        let cache_key = cache_key(CACHE_PREFIX_FRAMEWORK, &framework.id.to_string());
        self.cache.set(&cache_key, &framework, Some(CACHE_TTL)).await?;

        tracing::info!("Updated framework: {} ({})", framework.name, framework.id);

        Ok(framework)
    }

    /// Delete a framework
    pub async fn delete_framework(&self, id: Uuid) -> AppResult<()> {
        // Check if framework exists and is not system
        let existing = self.get_framework(id).await?;
        if existing.is_system {
            return Err(AppError::Forbidden(
                "Cannot delete system frameworks".to_string(),
            ));
        }

        // Check if framework has any control mappings
        let mapping_count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM control_requirement_mappings crm
            JOIN framework_requirements fr ON crm.framework_requirement_id = fr.id
            WHERE fr.framework_id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        if mapping_count.0 > 0 {
            return Err(AppError::Conflict(format!(
                "Cannot delete framework with {} existing control mappings",
                mapping_count.0
            )));
        }

        sqlx::query("DELETE FROM frameworks WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        // Invalidate caches
        self.invalidate_framework_cache(id).await?;

        tracing::info!("Deleted framework: {}", id);

        Ok(())
    }

    // ==================== Requirement CRUD ====================

    /// List all requirements for a framework
    pub async fn list_requirements(&self, framework_id: Uuid) -> AppResult<Vec<FrameworkRequirement>> {
        let cache_key = format!("{}:{}", CACHE_PREFIX_REQUIREMENTS_LIST, framework_id);

        // Try cache first
        if let Some(cached) = self.cache.get::<Vec<FrameworkRequirement>>(&cache_key).await? {
            tracing::debug!("Cache hit for framework requirements {}", framework_id);
            return Ok(cached);
        }

        let requirements = sqlx::query_as::<_, FrameworkRequirement>(
            r#"
            SELECT id, framework_id, code, name, description, category, parent_id, sort_order
            FROM framework_requirements
            WHERE framework_id = $1
            ORDER BY sort_order ASC, code ASC
            "#,
        )
        .bind(framework_id)
        .fetch_all(&self.db)
        .await?;

        // Cache the result
        self.cache.set(&cache_key, &requirements, Some(CACHE_TTL)).await?;

        Ok(requirements)
    }

    /// Get a single requirement by ID
    pub async fn get_requirement(&self, id: Uuid) -> AppResult<FrameworkRequirement> {
        let cache_key = cache_key(CACHE_PREFIX_REQUIREMENT, &id.to_string());

        // Try cache first
        if let Some(cached) = self.cache.get::<FrameworkRequirement>(&cache_key).await? {
            tracing::debug!("Cache hit for requirement {}", id);
            return Ok(cached);
        }

        let requirement = sqlx::query_as::<_, FrameworkRequirement>(
            r#"
            SELECT id, framework_id, code, name, description, category, parent_id, sort_order
            FROM framework_requirements
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Requirement {} not found", id)))?;

        // Cache the result
        self.cache.set(&cache_key, &requirement, Some(CACHE_TTL)).await?;

        Ok(requirement)
    }

    /// Create a new requirement
    pub async fn create_requirement(
        &self,
        framework_id: Uuid,
        input: CreateFrameworkRequirement,
    ) -> AppResult<FrameworkRequirement> {
        // Verify framework exists
        self.get_framework(framework_id).await?;

        FrameworkRequirement::validate_create(&input)
            .map_err(|e| AppError::ValidationError(e))?;

        // Validate parent exists if specified
        if let Some(parent_id) = input.parent_id {
            let parent = self.get_requirement(parent_id).await?;
            if parent.framework_id != framework_id {
                return Err(AppError::ValidationError(
                    "Parent requirement must belong to the same framework".to_string(),
                ));
            }
        }

        let requirement = sqlx::query_as::<_, FrameworkRequirement>(
            r#"
            INSERT INTO framework_requirements (framework_id, code, name, description, category, parent_id, sort_order)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, framework_id, code, name, description, category, parent_id, sort_order
            "#,
        )
        .bind(framework_id)
        .bind(&input.code)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.category)
        .bind(input.parent_id)
        .bind(input.sort_order.unwrap_or(0))
        .fetch_one(&self.db)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.constraint() == Some("framework_requirements_framework_id_code_key") {
                    return AppError::Conflict(format!(
                        "Requirement with code '{}' already exists in this framework",
                        input.code
                    ));
                }
            }
            AppError::DatabaseError(e)
        })?;

        // Invalidate requirement list cache
        self.invalidate_requirement_list_cache(framework_id).await?;

        // Cache the new requirement
        let cache_key = cache_key(CACHE_PREFIX_REQUIREMENT, &requirement.id.to_string());
        self.cache.set(&cache_key, &requirement, Some(CACHE_TTL)).await?;

        tracing::info!(
            "Created requirement: {} ({}) for framework {}",
            requirement.code,
            requirement.id,
            framework_id
        );

        Ok(requirement)
    }

    /// Update a requirement
    pub async fn update_requirement(
        &self,
        id: Uuid,
        input: UpdateFrameworkRequirement,
    ) -> AppResult<FrameworkRequirement> {
        let existing = self.get_requirement(id).await?;

        // Check framework is not system
        let framework = self.get_framework(existing.framework_id).await?;
        if framework.is_system {
            return Err(AppError::Forbidden(
                "Cannot modify requirements of system frameworks".to_string(),
            ));
        }

        // Validate parent if specified
        if let Some(parent_id) = input.parent_id {
            if parent_id == id {
                return Err(AppError::ValidationError(
                    "Requirement cannot be its own parent".to_string(),
                ));
            }
            let parent = self.get_requirement(parent_id).await?;
            if parent.framework_id != existing.framework_id {
                return Err(AppError::ValidationError(
                    "Parent requirement must belong to the same framework".to_string(),
                ));
            }
        }

        let requirement = sqlx::query_as::<_, FrameworkRequirement>(
            r#"
            UPDATE framework_requirements
            SET
                code = COALESCE($2, code),
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                category = COALESCE($5, category),
                parent_id = COALESCE($6, parent_id),
                sort_order = COALESCE($7, sort_order)
            WHERE id = $1
            RETURNING id, framework_id, code, name, description, category, parent_id, sort_order
            "#,
        )
        .bind(id)
        .bind(&input.code)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.category)
        .bind(input.parent_id)
        .bind(input.sort_order)
        .fetch_one(&self.db)
        .await?;

        // Invalidate caches
        self.invalidate_requirement_cache(id, existing.framework_id).await?;

        // Cache updated requirement
        let cache_key = cache_key(CACHE_PREFIX_REQUIREMENT, &requirement.id.to_string());
        self.cache.set(&cache_key, &requirement, Some(CACHE_TTL)).await?;

        tracing::info!("Updated requirement: {} ({})", requirement.code, requirement.id);

        Ok(requirement)
    }

    /// Delete a requirement
    pub async fn delete_requirement(&self, id: Uuid) -> AppResult<()> {
        let existing = self.get_requirement(id).await?;

        // Check framework is not system
        let framework = self.get_framework(existing.framework_id).await?;
        if framework.is_system {
            return Err(AppError::Forbidden(
                "Cannot delete requirements of system frameworks".to_string(),
            ));
        }

        // Check for control mappings
        let mapping_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM control_requirement_mappings WHERE framework_requirement_id = $1",
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        if mapping_count.0 > 0 {
            return Err(AppError::Conflict(format!(
                "Cannot delete requirement with {} existing control mappings",
                mapping_count.0
            )));
        }

        // Check for child requirements
        let child_count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM framework_requirements WHERE parent_id = $1",
        )
        .bind(id)
        .fetch_one(&self.db)
        .await?;

        if child_count.0 > 0 {
            return Err(AppError::Conflict(format!(
                "Cannot delete requirement with {} child requirements",
                child_count.0
            )));
        }

        sqlx::query("DELETE FROM framework_requirements WHERE id = $1")
            .bind(id)
            .execute(&self.db)
            .await?;

        // Invalidate caches
        self.invalidate_requirement_cache(id, existing.framework_id).await?;

        tracing::info!("Deleted requirement: {}", id);

        Ok(())
    }

    /// Batch create requirements (optimized for seeding)
    pub async fn batch_create_requirements(
        &self,
        framework_id: Uuid,
        requirements: Vec<CreateFrameworkRequirement>,
    ) -> AppResult<Vec<FrameworkRequirement>> {
        if requirements.is_empty() {
            return Ok(vec![]);
        }

        // Verify framework exists
        self.get_framework(framework_id).await?;

        // Validate all requirements first
        for req in &requirements {
            FrameworkRequirement::validate_create(req)
                .map_err(|e| AppError::ValidationError(e))?;
        }

        // Use transaction for batch insert
        let mut tx = self.db.begin().await?;

        let mut created = Vec::with_capacity(requirements.len());

        for input in requirements {
            let requirement = sqlx::query_as::<_, FrameworkRequirement>(
                r#"
                INSERT INTO framework_requirements (framework_id, code, name, description, category, parent_id, sort_order)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                RETURNING id, framework_id, code, name, description, category, parent_id, sort_order
                "#,
            )
            .bind(framework_id)
            .bind(&input.code)
            .bind(&input.name)
            .bind(&input.description)
            .bind(&input.category)
            .bind(input.parent_id)
            .bind(input.sort_order.unwrap_or(0))
            .fetch_one(&mut *tx)
            .await?;

            created.push(requirement);
        }

        tx.commit().await?;

        // Invalidate caches
        self.invalidate_requirement_list_cache(framework_id).await?;

        tracing::info!(
            "Batch created {} requirements for framework {}",
            created.len(),
            framework_id
        );

        Ok(created)
    }

    // ==================== Gap Analysis ====================

    /// Get gap analysis for a framework within an organization
    pub async fn get_gap_analysis(
        &self,
        org_id: Uuid,
        framework_id: Uuid,
    ) -> AppResult<FrameworkGapAnalysis> {
        // Get framework info
        let framework = self.get_framework(framework_id).await?;

        // Get all requirements for this framework
        let requirements = self.list_requirements(framework_id).await?;

        // Get control mappings count for each requirement (for this org)
        let mapping_counts: Vec<(Uuid, i64)> = sqlx::query_as(
            r#"
            SELECT fr.id, COUNT(crm.id) as count
            FROM framework_requirements fr
            LEFT JOIN control_requirement_mappings crm ON fr.id = crm.framework_requirement_id
            LEFT JOIN controls c ON crm.control_id = c.id AND c.organization_id = $1
            WHERE fr.framework_id = $2
            GROUP BY fr.id
            "#,
        )
        .bind(org_id)
        .bind(framework_id)
        .fetch_all(&self.db)
        .await?;

        let count_map: HashMap<Uuid, i64> = mapping_counts.into_iter().collect();

        // Build requirement analysis
        let req_analysis: Vec<RequirementGapAnalysis> = requirements
            .iter()
            .map(|req| {
                let control_count = count_map.get(&req.id).copied().unwrap_or(0);
                RequirementGapAnalysis {
                    id: req.id,
                    code: req.code.clone(),
                    name: req.name.clone(),
                    category: req.category.clone(),
                    control_count,
                    is_covered: control_count > 0,
                }
            })
            .collect();

        // Calculate totals
        let total_requirements = req_analysis.len() as i64;
        let covered_requirements = req_analysis.iter().filter(|r| r.is_covered).count() as i64;
        let uncovered_requirements = total_requirements - covered_requirements;
        let coverage_percentage = if total_requirements > 0 {
            (covered_requirements as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };

        // Group by category
        let mut category_map: HashMap<Option<String>, (i64, i64)> = HashMap::new();
        for req in &req_analysis {
            let entry = category_map.entry(req.category.clone()).or_insert((0, 0));
            entry.0 += 1; // total
            if req.is_covered {
                entry.1 += 1; // covered
            }
        }

        let by_category: Vec<CategoryGapAnalysis> = category_map
            .into_iter()
            .map(|(category, (total, covered))| CategoryGapAnalysis {
                category,
                total,
                covered,
                coverage_percentage: if total > 0 {
                    (covered as f64 / total as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect();

        Ok(FrameworkGapAnalysis {
            framework_id,
            framework_name: framework.name,
            total_requirements,
            covered_requirements,
            uncovered_requirements,
            coverage_percentage,
            by_category,
            requirements: req_analysis,
        })
    }

    // ==================== Cache Invalidation ====================

    async fn invalidate_framework_cache(&self, id: Uuid) -> AppResult<()> {
        // Delete specific framework cache
        let cache_key = cache_key(CACHE_PREFIX_FRAMEWORK, &id.to_string());
        self.cache.delete(&cache_key).await?;

        // Delete framework with requirements cache
        let with_reqs_key = format!("{}:with_reqs:{}", CACHE_PREFIX_FRAMEWORK, id);
        self.cache.delete(&with_reqs_key).await?;

        // Invalidate list caches
        self.invalidate_framework_list_cache().await?;

        Ok(())
    }

    async fn invalidate_framework_list_cache(&self) -> AppResult<()> {
        self.cache.delete_pattern(&format!("{}:*", CACHE_PREFIX_FRAMEWORKS_LIST)).await
    }

    async fn invalidate_requirement_cache(&self, id: Uuid, framework_id: Uuid) -> AppResult<()> {
        // Delete specific requirement cache
        let cache_key = cache_key(CACHE_PREFIX_REQUIREMENT, &id.to_string());
        self.cache.delete(&cache_key).await?;

        // Delete list cache
        self.invalidate_requirement_list_cache(framework_id).await?;

        // Invalidate framework with requirements cache
        let with_reqs_key = format!("{}:with_reqs:{}", CACHE_PREFIX_FRAMEWORK, framework_id);
        self.cache.delete(&with_reqs_key).await?;

        Ok(())
    }

    async fn invalidate_requirement_list_cache(&self, framework_id: Uuid) -> AppResult<()> {
        let cache_key = format!("{}:{}", CACHE_PREFIX_REQUIREMENTS_LIST, framework_id);
        self.cache.delete(&cache_key).await
    }
}
