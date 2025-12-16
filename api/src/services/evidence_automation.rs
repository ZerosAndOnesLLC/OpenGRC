use crate::cache::{org_cache_key, CacheClient};
use crate::models::{
    CreateEvidenceCollectionTask, CreateMappingRule, EvidenceChange, EvidenceChangeWithDetails,
    EvidenceCollectionRun, EvidenceCollectionTask, EvidenceCollectionTaskWithStats,
    EvidenceControlMappingRule, EvidenceFreshnessSla, EvidenceFreshnessSummary,
    EvidenceWithFreshness, ListCollectionTasksQuery, ListEvidenceChangesQuery,
    StaleEvidenceBySource, UpdateEvidenceCollectionTask, UpdateMappingRule,
};
use crate::utils::{AppError, AppResult};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use regex::Regex;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

/// Parse a cron schedule and return the next run time
fn parse_cron_next_run(cron_expr: &str) -> Option<DateTime<Utc>> {
    // Use cron_parser to get next occurrence
    cron_parser::parse(cron_expr, &Utc::now()).ok()
}

/// Validate a cron expression
fn validate_cron_expression(cron_expr: &str) -> bool {
    cron_parser::parse(cron_expr, &Utc::now()).is_ok()
}

const CACHE_PREFIX_FRESHNESS: &str = "evidence:freshness";

#[derive(Clone)]
pub struct EvidenceAutomationService {
    db: PgPool,
    cache: CacheClient,
}

impl EvidenceAutomationService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Freshness Scoring ====================

    /// Get freshness summary for organization dashboard
    pub async fn get_freshness_summary(&self, org_id: Uuid) -> AppResult<EvidenceFreshnessSummary> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_FRESHNESS, "summary");

        if let Some(cached) = self.cache.get::<EvidenceFreshnessSummary>(&cache_key).await? {
            return Ok(cached);
        }

        let summary = sqlx::query_as::<_, EvidenceFreshnessSummary>(
            r#"
            SELECT * FROM v_evidence_freshness_summary
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .unwrap_or_else(|| EvidenceFreshnessSummary {
            organization_id: org_id,
            total_evidence: 0,
            fresh_count: 0,
            aging_count: 0,
            stale_count: 0,
            critical_count: 0,
            expired_count: 0,
            expiring_soon_count: 0,
            avg_freshness_score: None,
            max_days_stale: None,
        });

        self.cache
            .set(&cache_key, &summary, Some(Duration::from_secs(300)))
            .await?;

        Ok(summary)
    }

    /// Get stale evidence grouped by source
    pub async fn get_stale_by_source(
        &self,
        org_id: Uuid,
        min_staleness: i32,
    ) -> AppResult<Vec<StaleEvidenceBySource>> {
        let results = sqlx::query_as::<_, StaleEvidenceBySource>(
            r#"
            SELECT * FROM get_stale_evidence_by_source($1, $2)
            "#,
        )
        .bind(org_id)
        .bind(min_staleness)
        .fetch_all(&self.db)
        .await?;

        Ok(results)
    }

    /// Update freshness scores for all evidence in an organization
    pub async fn update_freshness_scores(&self, org_id: Option<Uuid>) -> AppResult<i32> {
        let result: (i32,) = sqlx::query_as("SELECT update_evidence_freshness($1)")
            .bind(org_id)
            .fetch_one(&self.db)
            .await?;

        // Invalidate freshness cache
        if let Some(oid) = org_id {
            self.invalidate_freshness_cache(oid).await?;
        }

        tracing::info!("Updated freshness scores for {} evidence records", result.0);
        Ok(result.0)
    }

    /// Get evidence with freshness information
    pub async fn get_evidence_with_freshness(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
    ) -> AppResult<EvidenceWithFreshness> {
        let evidence = sqlx::query_as::<_, EvidenceWithFreshness>(
            r#"
            SELECT id, organization_id, title, description, evidence_type, source,
                   source_reference, file_path, file_size, mime_type, collected_at,
                   valid_from, valid_until, uploaded_by, created_at,
                   freshness_score, days_stale, freshness_sla_days, content_hash, last_verified_at
            FROM evidence
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(evidence_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Evidence {} not found", evidence_id)))?;

        Ok(evidence)
    }

    // ==================== Collection Tasks ====================

    /// List collection tasks for organization
    pub async fn list_collection_tasks(
        &self,
        org_id: Uuid,
        query: ListCollectionTasksQuery,
    ) -> AppResult<Vec<EvidenceCollectionTaskWithStats>> {
        let limit = query.limit.unwrap_or(50).min(100);
        let offset = query.offset.unwrap_or(0);

        // Get tasks first
        let tasks = sqlx::query_as::<_, EvidenceCollectionTask>(
            r#"
            SELECT id, organization_id, integration_id, name, description,
                   schedule, collection_config, last_run_at, next_run_at, status,
                   enabled, cron_schedule, timezone, evidence_type, auto_link_controls,
                   retention_days, last_error, error_count, success_count, created_at
            FROM evidence_collection_tasks
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR integration_id = $2)
              AND ($3::bool IS NULL OR enabled = $3)
            ORDER BY name ASC
            LIMIT $4 OFFSET $5
            "#,
        )
        .bind(org_id)
        .bind(query.integration_id)
        .bind(query.enabled)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        if tasks.is_empty() {
            return Ok(vec![]);
        }

        // Get integration info
        let task_ids: Vec<Uuid> = tasks.iter().map(|t| t.id).collect();
        let integration_ids: Vec<Uuid> = tasks.iter().map(|t| t.integration_id).collect();

        let integration_info: Vec<(Uuid, String, String)> = sqlx::query_as(
            "SELECT id, name, integration_type FROM integrations WHERE id = ANY($1)",
        )
        .bind(&integration_ids)
        .fetch_all(&self.db)
        .await?;

        let integration_map: std::collections::HashMap<Uuid, (String, String)> = integration_info
            .into_iter()
            .map(|(id, name, itype)| (id, (name, itype)))
            .collect();

        // Get last run stats
        let run_stats: Vec<(Uuid, Option<i32>, Option<i32>, Option<i32>, Option<i32>)> = sqlx::query_as(
            r#"
            SELECT DISTINCT ON (task_id)
                task_id, evidence_created, evidence_updated, changes_detected, duration_ms
            FROM evidence_collection_runs
            WHERE task_id = ANY($1)
            ORDER BY task_id, started_at DESC
            "#,
        )
        .bind(&task_ids)
        .fetch_all(&self.db)
        .await?;

        let run_map: std::collections::HashMap<Uuid, (Option<i32>, Option<i32>, Option<i32>, Option<i32>)> =
            run_stats
                .into_iter()
                .map(|(tid, c, u, ch, d)| (tid, (c, u, ch, d)))
                .collect();

        let result = tasks
            .into_iter()
            .map(|task| {
                let (integration_name, integration_type) = integration_map
                    .get(&task.integration_id)
                    .cloned()
                    .unwrap_or_default();

                let (last_run_created, last_run_updated, last_run_changes, last_run_duration_ms) =
                    run_map.get(&task.id).cloned().unwrap_or_default();

                EvidenceCollectionTaskWithStats {
                    task,
                    integration_name: Some(integration_name),
                    integration_type: Some(integration_type),
                    last_run_created,
                    last_run_updated,
                    last_run_changes,
                    last_run_duration_ms,
                }
            })
            .collect();

        Ok(result)
    }

    /// Create a new collection task
    pub async fn create_collection_task(
        &self,
        org_id: Uuid,
        input: CreateEvidenceCollectionTask,
    ) -> AppResult<EvidenceCollectionTask> {
        // Validate cron schedule if provided
        if let Some(ref cron) = input.cron_schedule {
            if !validate_cron_expression(cron) {
                return Err(AppError::ValidationError("Invalid cron schedule".to_string()));
            }
        }

        // Calculate next run time
        let next_run_at = input.cron_schedule.as_ref().and_then(|cron| {
            parse_cron_next_run(cron).or_else(|| Some(Utc::now() + ChronoDuration::hours(1)))
        });

        let task = sqlx::query_as::<_, EvidenceCollectionTask>(
            r#"
            INSERT INTO evidence_collection_tasks (
                organization_id, integration_id, name, description, cron_schedule,
                timezone, collection_config, evidence_type, auto_link_controls,
                retention_days, next_run_at, status
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'pending')
            RETURNING id, organization_id, integration_id, name, description,
                      schedule, collection_config, last_run_at, next_run_at, status,
                      enabled, cron_schedule, timezone, evidence_type, auto_link_controls,
                      retention_days, last_error, error_count, success_count, created_at
            "#,
        )
        .bind(org_id)
        .bind(input.integration_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.cron_schedule)
        .bind(input.timezone.as_deref().unwrap_or("UTC"))
        .bind(input.collection_config.unwrap_or_else(|| serde_json::json!({})))
        .bind(&input.evidence_type)
        .bind(input.auto_link_controls.unwrap_or(true))
        .bind(input.retention_days)
        .bind(next_run_at)
        .fetch_one(&self.db)
        .await?;

        tracing::info!("Created evidence collection task: {} ({})", task.name, task.id);
        Ok(task)
    }

    /// Update a collection task
    pub async fn update_collection_task(
        &self,
        org_id: Uuid,
        task_id: Uuid,
        input: UpdateEvidenceCollectionTask,
    ) -> AppResult<EvidenceCollectionTask> {
        // Validate cron schedule if provided
        if let Some(ref cron) = input.cron_schedule {
            if !validate_cron_expression(cron) {
                return Err(AppError::ValidationError("Invalid cron schedule".to_string()));
            }
        }

        let task = sqlx::query_as::<_, EvidenceCollectionTask>(
            r#"
            UPDATE evidence_collection_tasks
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                cron_schedule = COALESCE($5, cron_schedule),
                timezone = COALESCE($6, timezone),
                collection_config = COALESCE($7, collection_config),
                evidence_type = COALESCE($8, evidence_type),
                auto_link_controls = COALESCE($9, auto_link_controls),
                retention_days = COALESCE($10, retention_days),
                enabled = COALESCE($11, enabled)
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, integration_id, name, description,
                      schedule, collection_config, last_run_at, next_run_at, status,
                      enabled, cron_schedule, timezone, evidence_type, auto_link_controls,
                      retention_days, last_error, error_count, success_count, created_at
            "#,
        )
        .bind(task_id)
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.cron_schedule)
        .bind(&input.timezone)
        .bind(&input.collection_config)
        .bind(&input.evidence_type)
        .bind(input.auto_link_controls)
        .bind(input.retention_days)
        .bind(input.enabled)
        .fetch_one(&self.db)
        .await?;

        Ok(task)
    }

    /// Delete a collection task
    pub async fn delete_collection_task(&self, org_id: Uuid, task_id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM evidence_collection_tasks WHERE id = $1 AND organization_id = $2")
            .bind(task_id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    /// Get tasks due for execution
    pub async fn get_due_tasks(&self) -> AppResult<Vec<EvidenceCollectionTask>> {
        let tasks = sqlx::query_as::<_, EvidenceCollectionTask>(
            r#"
            SELECT id, organization_id, integration_id, name, description,
                   schedule, collection_config, last_run_at, next_run_at, status,
                   enabled, cron_schedule, timezone, evidence_type, auto_link_controls,
                   retention_days, last_error, error_count, success_count, created_at
            FROM evidence_collection_tasks
            WHERE enabled = true
              AND next_run_at <= NOW()
              AND status != 'running'
            ORDER BY next_run_at ASC
            LIMIT 100
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(tasks)
    }

    /// Start a collection run
    pub async fn start_collection_run(
        &self,
        task: &EvidenceCollectionTask,
    ) -> AppResult<EvidenceCollectionRun> {
        // Mark task as running
        sqlx::query("UPDATE evidence_collection_tasks SET status = 'running' WHERE id = $1")
            .bind(task.id)
            .execute(&self.db)
            .await?;

        // Create run record
        let run = sqlx::query_as::<_, EvidenceCollectionRun>(
            r#"
            INSERT INTO evidence_collection_runs (task_id, organization_id, integration_id, status)
            VALUES ($1, $2, $3, 'running')
            RETURNING id, task_id, organization_id, integration_id, status, started_at,
                      completed_at, evidence_created, evidence_updated, evidence_unchanged,
                      changes_detected, controls_linked, error_message, duration_ms, created_at
            "#,
        )
        .bind(task.id)
        .bind(task.organization_id)
        .bind(task.integration_id)
        .fetch_one(&self.db)
        .await?;

        Ok(run)
    }

    /// Complete a collection run
    pub async fn complete_collection_run(
        &self,
        run: &EvidenceCollectionRun,
        evidence_created: i32,
        evidence_updated: i32,
        evidence_unchanged: i32,
        changes_detected: i32,
        controls_linked: i32,
        error_message: Option<String>,
    ) -> AppResult<()> {
        let status = if error_message.is_some() { "failed" } else { "completed" };
        let duration_ms = (Utc::now() - run.started_at).num_milliseconds() as i32;

        // Update run record
        sqlx::query(
            r#"
            UPDATE evidence_collection_runs
            SET status = $2, completed_at = NOW(), evidence_created = $3, evidence_updated = $4,
                evidence_unchanged = $5, changes_detected = $6, controls_linked = $7,
                error_message = $8, duration_ms = $9
            WHERE id = $1
            "#,
        )
        .bind(run.id)
        .bind(status)
        .bind(evidence_created)
        .bind(evidence_updated)
        .bind(evidence_unchanged)
        .bind(changes_detected)
        .bind(controls_linked)
        .bind(&error_message)
        .bind(duration_ms)
        .execute(&self.db)
        .await?;

        // Update task status and calculate next run
        let task = sqlx::query_as::<_, EvidenceCollectionTask>(
            "SELECT * FROM evidence_collection_tasks WHERE id = $1",
        )
        .bind(run.task_id)
        .fetch_one(&self.db)
        .await?;

        let next_run_at = task.cron_schedule.as_ref().and_then(|cron| {
            parse_cron_next_run(cron)
        });

        let (new_success, new_error) = if error_message.is_some() {
            (task.success_count, task.error_count + 1)
        } else {
            (task.success_count + 1, task.error_count)
        };

        sqlx::query(
            r#"
            UPDATE evidence_collection_tasks
            SET status = 'pending', last_run_at = NOW(), next_run_at = $2,
                success_count = $3, error_count = $4, last_error = $5
            WHERE id = $1
            "#,
        )
        .bind(run.task_id)
        .bind(next_run_at)
        .bind(new_success)
        .bind(new_error)
        .bind(&error_message)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    // ==================== Change Detection ====================

    /// List evidence changes
    pub async fn list_evidence_changes(
        &self,
        org_id: Uuid,
        query: ListEvidenceChangesQuery,
    ) -> AppResult<Vec<EvidenceChangeWithDetails>> {
        let limit = query.limit.unwrap_or(50).min(100);
        let offset = query.offset.unwrap_or(0);

        let changes = sqlx::query_as::<_, (
            Uuid, Uuid, Uuid, String, Option<String>, Option<String>,
            Option<serde_json::Value>, Option<serde_json::Value>, Option<String>,
            DateTime<Utc>, Option<DateTime<Utc>>, Option<Uuid>, DateTime<Utc>,
            String, String, String
        )>(
            r#"
            SELECT
                c.id, c.evidence_id, c.organization_id, c.change_type,
                c.previous_hash, c.new_hash, c.previous_data, c.new_data,
                c.change_summary, c.detected_at, c.acknowledged_at, c.acknowledged_by,
                c.created_at, e.title, e.source, e.evidence_type
            FROM evidence_changes c
            JOIN evidence e ON c.evidence_id = e.id
            WHERE c.organization_id = $1
              AND ($2::uuid IS NULL OR c.evidence_id = $2)
              AND ($3::text IS NULL OR c.change_type = $3)
              AND ($4::bool IS NULL OR ($4 = true AND c.acknowledged_at IS NOT NULL)
                   OR ($4 = false AND c.acknowledged_at IS NULL))
            ORDER BY c.detected_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(org_id)
        .bind(query.evidence_id)
        .bind(&query.change_type)
        .bind(query.acknowledged)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        let result = changes
            .into_iter()
            .map(|row| {
                let change = EvidenceChange {
                    id: row.0,
                    evidence_id: row.1,
                    organization_id: row.2,
                    change_type: row.3,
                    previous_hash: row.4,
                    new_hash: row.5,
                    previous_data: row.6,
                    new_data: row.7,
                    change_summary: row.8,
                    detected_at: row.9,
                    acknowledged_at: row.10,
                    acknowledged_by: row.11,
                    created_at: row.12,
                };

                EvidenceChangeWithDetails {
                    change,
                    evidence_title: row.13,
                    evidence_source: row.14,
                    evidence_type: row.15,
                }
            })
            .collect();

        Ok(result)
    }

    /// Acknowledge a change
    pub async fn acknowledge_change(
        &self,
        org_id: Uuid,
        change_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<()> {
        let result = sqlx::query(
            r#"
            UPDATE evidence_changes
            SET acknowledged_at = NOW(), acknowledged_by = $3
            WHERE id = $1 AND organization_id = $2 AND acknowledged_at IS NULL
            "#,
        )
        .bind(change_id)
        .bind(org_id)
        .bind(user_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Change not found or already acknowledged".to_string()));
        }

        Ok(())
    }

    /// Get pending change count
    pub async fn get_pending_change_count(&self, org_id: Uuid) -> AppResult<i64> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM evidence_changes
            WHERE organization_id = $1 AND acknowledged_at IS NULL
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        Ok(count)
    }

    /// Create a change record (for manual detection)
    pub async fn record_change(
        &self,
        evidence_id: Uuid,
        org_id: Uuid,
        change_type: &str,
        previous_hash: Option<&str>,
        new_hash: Option<&str>,
        change_summary: Option<&str>,
    ) -> AppResult<EvidenceChange> {
        let change = sqlx::query_as::<_, EvidenceChange>(
            r#"
            INSERT INTO evidence_changes (evidence_id, organization_id, change_type,
                                         previous_hash, new_hash, change_summary)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, evidence_id, organization_id, change_type, previous_hash, new_hash,
                      previous_data, new_data, change_summary, detected_at, acknowledged_at,
                      acknowledged_by, created_at
            "#,
        )
        .bind(evidence_id)
        .bind(org_id)
        .bind(change_type)
        .bind(previous_hash)
        .bind(new_hash)
        .bind(change_summary)
        .fetch_one(&self.db)
        .await?;

        Ok(change)
    }

    // ==================== Auto-Linking ====================

    /// List mapping rules
    pub async fn list_mapping_rules(
        &self,
        org_id: Uuid,
    ) -> AppResult<Vec<EvidenceControlMappingRule>> {
        let rules = sqlx::query_as::<_, EvidenceControlMappingRule>(
            r#"
            SELECT id, organization_id, name, description, enabled, priority,
                   source_pattern, type_pattern, title_pattern, control_codes,
                   created_at, updated_at
            FROM evidence_control_mapping_rules
            WHERE organization_id = $1
            ORDER BY priority DESC, name ASC
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(rules)
    }

    /// Create a mapping rule
    pub async fn create_mapping_rule(
        &self,
        org_id: Uuid,
        input: CreateMappingRule,
    ) -> AppResult<EvidenceControlMappingRule> {
        // Validate regex patterns
        if let Some(ref pattern) = input.source_pattern {
            Regex::new(pattern).map_err(|_| {
                AppError::ValidationError("Invalid source_pattern regex".to_string())
            })?;
        }
        if let Some(ref pattern) = input.type_pattern {
            Regex::new(pattern).map_err(|_| {
                AppError::ValidationError("Invalid type_pattern regex".to_string())
            })?;
        }
        if let Some(ref pattern) = input.title_pattern {
            Regex::new(pattern).map_err(|_| {
                AppError::ValidationError("Invalid title_pattern regex".to_string())
            })?;
        }

        let rule = sqlx::query_as::<_, EvidenceControlMappingRule>(
            r#"
            INSERT INTO evidence_control_mapping_rules (
                organization_id, name, description, source_pattern, type_pattern,
                title_pattern, control_codes, priority
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, organization_id, name, description, enabled, priority,
                      source_pattern, type_pattern, title_pattern, control_codes,
                      created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.source_pattern)
        .bind(&input.type_pattern)
        .bind(&input.title_pattern)
        .bind(&input.control_codes)
        .bind(input.priority.unwrap_or(0))
        .fetch_one(&self.db)
        .await?;

        Ok(rule)
    }

    /// Update a mapping rule
    pub async fn update_mapping_rule(
        &self,
        org_id: Uuid,
        rule_id: Uuid,
        input: UpdateMappingRule,
    ) -> AppResult<EvidenceControlMappingRule> {
        let rule = sqlx::query_as::<_, EvidenceControlMappingRule>(
            r#"
            UPDATE evidence_control_mapping_rules
            SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                source_pattern = COALESCE($5, source_pattern),
                type_pattern = COALESCE($6, type_pattern),
                title_pattern = COALESCE($7, title_pattern),
                control_codes = COALESCE($8, control_codes),
                priority = COALESCE($9, priority),
                enabled = COALESCE($10, enabled),
                updated_at = NOW()
            WHERE id = $1 AND organization_id = $2
            RETURNING id, organization_id, name, description, enabled, priority,
                      source_pattern, type_pattern, title_pattern, control_codes,
                      created_at, updated_at
            "#,
        )
        .bind(rule_id)
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.source_pattern)
        .bind(&input.type_pattern)
        .bind(&input.title_pattern)
        .bind(&input.control_codes)
        .bind(input.priority)
        .bind(input.enabled)
        .fetch_one(&self.db)
        .await?;

        Ok(rule)
    }

    /// Delete a mapping rule
    pub async fn delete_mapping_rule(&self, org_id: Uuid, rule_id: Uuid) -> AppResult<()> {
        sqlx::query(
            "DELETE FROM evidence_control_mapping_rules WHERE id = $1 AND organization_id = $2",
        )
        .bind(rule_id)
        .bind(org_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Apply mapping rules to evidence and return control codes to link
    pub async fn get_control_codes_for_evidence(
        &self,
        org_id: Uuid,
        source: &str,
        evidence_type: &str,
        title: &str,
    ) -> AppResult<Vec<String>> {
        let rules = self.list_mapping_rules(org_id).await?;

        let mut control_codes = Vec::new();

        for rule in rules.iter().filter(|r| r.enabled) {
            let mut matches = true;

            if let Some(ref pattern) = rule.source_pattern {
                if let Ok(re) = Regex::new(pattern) {
                    matches = matches && re.is_match(source);
                }
            }

            if let Some(ref pattern) = rule.type_pattern {
                if let Ok(re) = Regex::new(pattern) {
                    matches = matches && re.is_match(evidence_type);
                }
            }

            if let Some(ref pattern) = rule.title_pattern {
                if let Ok(re) = Regex::new(pattern) {
                    matches = matches && re.is_match(title);
                }
            }

            if matches {
                control_codes.extend(rule.control_codes.clone());
            }
        }

        // Deduplicate
        control_codes.sort();
        control_codes.dedup();

        Ok(control_codes)
    }

    // ==================== Content Hashing ====================

    /// Calculate content hash for evidence
    pub fn calculate_content_hash(content: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content);
        format!("{:x}", hasher.finalize())
    }

    /// Update content hash for evidence
    pub async fn update_content_hash(
        &self,
        org_id: Uuid,
        evidence_id: Uuid,
        content_hash: &str,
    ) -> AppResult<Option<String>> {
        // Get current hash
        let current: Option<(Option<String>,)> = sqlx::query_as(
            "SELECT content_hash FROM evidence WHERE id = $1 AND organization_id = $2",
        )
        .bind(evidence_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?;

        let previous_hash = current.and_then(|c| c.0);

        // Update hash
        sqlx::query(
            r#"
            UPDATE evidence
            SET content_hash = $3, last_verified_at = NOW()
            WHERE id = $1 AND organization_id = $2
            "#,
        )
        .bind(evidence_id)
        .bind(org_id)
        .bind(content_hash)
        .execute(&self.db)
        .await?;

        Ok(previous_hash)
    }

    // ==================== Freshness SLAs ====================

    /// List SLAs for organization (including system defaults)
    pub async fn list_freshness_slas(&self, org_id: Uuid) -> AppResult<Vec<EvidenceFreshnessSla>> {
        let slas = sqlx::query_as::<_, EvidenceFreshnessSla>(
            r#"
            SELECT id, organization_id, evidence_type, source, max_age_days,
                   warning_days, critical_days, auto_expire, created_at
            FROM evidence_freshness_slas
            WHERE organization_id = $1 OR organization_id IS NULL
            ORDER BY organization_id DESC NULLS LAST, evidence_type, source
            "#,
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(slas)
    }

    // ==================== Cache ====================

    async fn invalidate_freshness_cache(&self, org_id: Uuid) -> AppResult<()> {
        let key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_FRESHNESS, "summary");
        self.cache.delete(&key).await
    }
}
