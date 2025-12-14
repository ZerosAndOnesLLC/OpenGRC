use crate::cache::{org_cache_key, CacheClient};
use crate::services::notification::{CreateNotification, NotificationService};
use crate::utils::{AppError, AppResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

const CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes
const CACHE_PREFIX_TEMPLATES: &str = "control_test_templates";
const CACHE_PREFIX_MONITORING: &str = "control_monitoring";

#[derive(Clone)]
pub struct ControlTestAutomationService {
    db: PgPool,
    cache: CacheClient,
}

// ==================== Models ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ControlTestTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub subcategory: Option<String>,
    pub test_type: String,
    pub automation_config: serde_json::Value,
    pub default_frequency: Option<String>,
    pub applicable_frameworks: Option<Vec<String>>,
    pub applicable_controls: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTemplatesQuery {
    pub category: Option<String>,
    pub subcategory: Option<String>,
    pub framework: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ControlTestAlertConfig {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub control_test_id: Uuid,
    pub alert_on_failure: bool,
    pub consecutive_failures_threshold: i32,
    pub alert_on_recovery: bool,
    pub alert_recipients: Option<Vec<Uuid>>,
    pub alert_email_enabled: bool,
    pub alert_in_app_enabled: bool,
    pub escalation_after_hours: Option<i32>,
    pub escalation_recipients: Option<Vec<Uuid>>,
    pub is_muted: bool,
    pub muted_until: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAlertConfig {
    pub control_test_id: Uuid,
    pub alert_on_failure: Option<bool>,
    pub consecutive_failures_threshold: Option<i32>,
    pub alert_on_recovery: Option<bool>,
    pub alert_recipients: Option<Vec<Uuid>>,
    pub alert_email_enabled: Option<bool>,
    pub alert_in_app_enabled: Option<bool>,
    pub escalation_after_hours: Option<i32>,
    pub escalation_recipients: Option<Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAlertConfig {
    pub alert_on_failure: Option<bool>,
    pub consecutive_failures_threshold: Option<i32>,
    pub alert_on_recovery: Option<bool>,
    pub alert_recipients: Option<Vec<Uuid>>,
    pub alert_email_enabled: Option<bool>,
    pub alert_in_app_enabled: Option<bool>,
    pub escalation_after_hours: Option<i32>,
    pub escalation_recipients: Option<Vec<Uuid>>,
    pub is_muted: Option<bool>,
    pub muted_until: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ControlTestRemediation {
    pub id: Uuid,
    pub control_test_id: Option<Uuid>,
    pub failure_pattern: Option<String>,
    pub severity: String,
    pub title: String,
    pub description: String,
    pub remediation_steps: Vec<String>,
    pub documentation_url: Option<String>,
    pub estimated_effort: Option<String>,
    pub auto_generated: bool,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRemediation {
    pub control_test_id: Option<Uuid>,
    pub failure_pattern: Option<String>,
    pub severity: Option<String>,
    pub title: String,
    pub description: String,
    pub remediation_steps: Vec<String>,
    pub documentation_url: Option<String>,
    pub estimated_effort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ControlMonitoringStatus {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub control_id: Uuid,
    pub monitoring_enabled: bool,
    pub last_test_at: Option<DateTime<Utc>>,
    pub last_test_status: Option<String>,
    pub consecutive_failures: i32,
    pub consecutive_passes: i32,
    pub current_health: String,
    pub health_score: i32,
    pub total_tests: i32,
    pub passed_tests: i32,
    pub failed_tests: i32,
    pub last_failure_at: Option<DateTime<Utc>>,
    pub last_failure_reason: Option<String>,
    pub alert_status: String,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ControlMonitoringSummary {
    pub organization_id: Uuid,
    pub total_controls: i64,
    pub monitored_controls: i64,
    pub healthy_controls: i64,
    pub degraded_controls: i64,
    pub failing_controls: i64,
    pub unknown_controls: i64,
    pub alerting_controls: i64,
    pub avg_health_score: Option<f64>,
    pub total_test_runs: Option<i64>,
    pub total_passed: Option<i64>,
    pub total_failed: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ControlTestRun {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub control_test_id: Uuid,
    pub control_id: Uuid,
    pub run_type: String,
    pub triggered_by: Option<Uuid>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: String,
    pub execution_time_ms: Option<i32>,
    pub notes: Option<String>,
    pub raw_output: Option<String>,
    pub error_message: Option<String>,
    pub evidence_created_id: Option<Uuid>,
    pub alert_sent: bool,
    pub alert_sent_at: Option<DateTime<Utc>>,
    pub remediation_suggested_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListTestRunsQuery {
    pub control_id: Option<Uuid>,
    pub control_test_id: Option<Uuid>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ControlTestAlert {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub control_test_id: Uuid,
    pub test_run_id: Option<Uuid>,
    pub alert_type: String,
    pub severity: String,
    pub title: String,
    pub message: String,
    pub recipients: Vec<Uuid>,
    pub email_sent: bool,
    pub in_app_sent: bool,
    pub acknowledged_by: Option<Uuid>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAlertsQuery {
    pub control_test_id: Option<Uuid>,
    pub alert_type: Option<String>,
    pub unacknowledged_only: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl ControlTestAutomationService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ==================== Test Templates ====================

    /// List test templates with filtering
    pub async fn list_templates(&self, query: ListTemplatesQuery) -> AppResult<Vec<ControlTestTemplate>> {
        let limit = query.limit.unwrap_or(100).min(500);
        let offset = query.offset.unwrap_or(0);

        let templates = sqlx::query_as::<_, ControlTestTemplate>(
            r#"
            SELECT id, name, description, category, subcategory, test_type, automation_config,
                   default_frequency, applicable_frameworks, applicable_controls, tags,
                   is_active, created_at, updated_at
            FROM control_test_templates
            WHERE is_active = true
              AND ($1::text IS NULL OR category = $1)
              AND ($2::text IS NULL OR subcategory = $2)
              AND ($3::text IS NULL OR $3 = ANY(applicable_frameworks))
              AND ($4::text IS NULL OR
                   LOWER(name) LIKE '%' || LOWER($4) || '%' OR
                   LOWER(description) LIKE '%' || LOWER($4) || '%' OR
                   EXISTS (SELECT 1 FROM unnest(tags) t WHERE LOWER(t) LIKE '%' || LOWER($4) || '%'))
            ORDER BY category, name
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(&query.category)
        .bind(&query.subcategory)
        .bind(&query.framework)
        .bind(&query.search)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        Ok(templates)
    }

    /// Get template by ID
    pub async fn get_template(&self, id: Uuid) -> AppResult<ControlTestTemplate> {
        let cache_key = org_cache_key("global", CACHE_PREFIX_TEMPLATES, &id.to_string());

        if let Some(cached) = self.cache.get::<ControlTestTemplate>(&cache_key).await? {
            return Ok(cached);
        }

        let template = sqlx::query_as::<_, ControlTestTemplate>(
            r#"
            SELECT id, name, description, category, subcategory, test_type, automation_config,
                   default_frequency, applicable_frameworks, applicable_controls, tags,
                   is_active, created_at, updated_at
            FROM control_test_templates
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Template {} not found", id)))?;

        self.cache.set(&cache_key, &template, Some(CACHE_TTL)).await?;

        Ok(template)
    }

    /// Get template categories
    pub async fn get_template_categories(&self) -> AppResult<Vec<(String, i64)>> {
        let categories: Vec<(String, i64)> = sqlx::query_as(
            r#"
            SELECT category, COUNT(*) as count
            FROM control_test_templates
            WHERE is_active = true
            GROUP BY category
            ORDER BY count DESC
            "#,
        )
        .fetch_all(&self.db)
        .await?;

        Ok(categories)
    }

    // ==================== Alert Configuration ====================

    /// Get or create alert config for a test
    pub async fn get_alert_config(
        &self,
        org_id: Uuid,
        control_test_id: Uuid,
    ) -> AppResult<ControlTestAlertConfig> {
        let config = sqlx::query_as::<_, ControlTestAlertConfig>(
            r#"
            SELECT id, organization_id, control_test_id, alert_on_failure, consecutive_failures_threshold,
                   alert_on_recovery, alert_recipients, alert_email_enabled, alert_in_app_enabled,
                   escalation_after_hours, escalation_recipients, is_muted, muted_until,
                   created_at, updated_at
            FROM control_test_alert_configs
            WHERE organization_id = $1 AND control_test_id = $2
            "#,
        )
        .bind(org_id)
        .bind(control_test_id)
        .fetch_optional(&self.db)
        .await?;

        match config {
            Some(c) => Ok(c),
            None => {
                // Create default config
                let new_config = sqlx::query_as::<_, ControlTestAlertConfig>(
                    r#"
                    INSERT INTO control_test_alert_configs (organization_id, control_test_id)
                    VALUES ($1, $2)
                    RETURNING id, organization_id, control_test_id, alert_on_failure, consecutive_failures_threshold,
                              alert_on_recovery, alert_recipients, alert_email_enabled, alert_in_app_enabled,
                              escalation_after_hours, escalation_recipients, is_muted, muted_until,
                              created_at, updated_at
                    "#,
                )
                .bind(org_id)
                .bind(control_test_id)
                .fetch_one(&self.db)
                .await?;

                Ok(new_config)
            }
        }
    }

    /// Update alert config
    pub async fn update_alert_config(
        &self,
        org_id: Uuid,
        control_test_id: Uuid,
        input: UpdateAlertConfig,
    ) -> AppResult<ControlTestAlertConfig> {
        // Ensure config exists
        self.get_alert_config(org_id, control_test_id).await?;

        let config = sqlx::query_as::<_, ControlTestAlertConfig>(
            r#"
            UPDATE control_test_alert_configs
            SET
                alert_on_failure = COALESCE($3, alert_on_failure),
                consecutive_failures_threshold = COALESCE($4, consecutive_failures_threshold),
                alert_on_recovery = COALESCE($5, alert_on_recovery),
                alert_recipients = COALESCE($6, alert_recipients),
                alert_email_enabled = COALESCE($7, alert_email_enabled),
                alert_in_app_enabled = COALESCE($8, alert_in_app_enabled),
                escalation_after_hours = COALESCE($9, escalation_after_hours),
                escalation_recipients = COALESCE($10, escalation_recipients),
                is_muted = COALESCE($11, is_muted),
                muted_until = COALESCE($12, muted_until),
                updated_at = NOW()
            WHERE organization_id = $1 AND control_test_id = $2
            RETURNING id, organization_id, control_test_id, alert_on_failure, consecutive_failures_threshold,
                      alert_on_recovery, alert_recipients, alert_email_enabled, alert_in_app_enabled,
                      escalation_after_hours, escalation_recipients, is_muted, muted_until,
                      created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(control_test_id)
        .bind(input.alert_on_failure)
        .bind(input.consecutive_failures_threshold)
        .bind(input.alert_on_recovery)
        .bind(&input.alert_recipients)
        .bind(input.alert_email_enabled)
        .bind(input.alert_in_app_enabled)
        .bind(input.escalation_after_hours)
        .bind(&input.escalation_recipients)
        .bind(input.is_muted)
        .bind(input.muted_until)
        .fetch_one(&self.db)
        .await?;

        Ok(config)
    }

    // ==================== Remediation ====================

    /// List remediations
    pub async fn list_remediations(
        &self,
        control_test_id: Option<Uuid>,
    ) -> AppResult<Vec<ControlTestRemediation>> {
        let remediations = sqlx::query_as::<_, ControlTestRemediation>(
            r#"
            SELECT id, control_test_id, failure_pattern, severity, title, description,
                   remediation_steps, documentation_url, estimated_effort, auto_generated,
                   created_by, created_at, updated_at
            FROM control_test_remediations
            WHERE ($1::uuid IS NULL AND control_test_id IS NULL)
               OR control_test_id = $1
            ORDER BY severity DESC, created_at DESC
            "#,
        )
        .bind(control_test_id)
        .fetch_all(&self.db)
        .await?;

        Ok(remediations)
    }

    /// Create a remediation
    pub async fn create_remediation(
        &self,
        user_id: Uuid,
        input: CreateRemediation,
    ) -> AppResult<ControlTestRemediation> {
        let remediation = sqlx::query_as::<_, ControlTestRemediation>(
            r#"
            INSERT INTO control_test_remediations (
                control_test_id, failure_pattern, severity, title, description,
                remediation_steps, documentation_url, estimated_effort, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, control_test_id, failure_pattern, severity, title, description,
                      remediation_steps, documentation_url, estimated_effort, auto_generated,
                      created_by, created_at, updated_at
            "#,
        )
        .bind(input.control_test_id)
        .bind(&input.failure_pattern)
        .bind(input.severity.as_deref().unwrap_or("medium"))
        .bind(&input.title)
        .bind(&input.description)
        .bind(&input.remediation_steps)
        .bind(&input.documentation_url)
        .bind(&input.estimated_effort)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(remediation)
    }

    /// Get remediation by ID
    pub async fn get_remediation(&self, id: Uuid) -> AppResult<ControlTestRemediation> {
        let remediation = sqlx::query_as::<_, ControlTestRemediation>(
            r#"
            SELECT id, control_test_id, failure_pattern, severity, title, description,
                   remediation_steps, documentation_url, estimated_effort, auto_generated,
                   created_by, created_at, updated_at
            FROM control_test_remediations
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Remediation {} not found", id)))?;

        Ok(remediation)
    }

    /// Find matching remediation for a failure
    pub async fn find_matching_remediation(
        &self,
        control_test_id: Uuid,
        failure_message: &str,
    ) -> AppResult<Option<ControlTestRemediation>> {
        let remediation = sqlx::query_as::<_, ControlTestRemediation>(
            r#"
            SELECT id, control_test_id, failure_pattern, severity, title, description,
                   remediation_steps, documentation_url, estimated_effort, auto_generated,
                   created_by, created_at, updated_at
            FROM control_test_remediations
            WHERE (control_test_id = $1 OR control_test_id IS NULL)
              AND $2 ~* failure_pattern
            ORDER BY
                CASE WHEN control_test_id = $1 THEN 0 ELSE 1 END,
                CASE severity
                    WHEN 'critical' THEN 1
                    WHEN 'high' THEN 2
                    WHEN 'medium' THEN 3
                    ELSE 4
                END,
                created_at DESC
            LIMIT 1
            "#,
        )
        .bind(control_test_id)
        .bind(failure_message)
        .fetch_optional(&self.db)
        .await?;

        Ok(remediation)
    }

    // ==================== Monitoring Status ====================

    /// Get monitoring summary for organization
    pub async fn get_monitoring_summary(&self, org_id: Uuid) -> AppResult<ControlMonitoringSummary> {
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_MONITORING, "summary");

        if let Some(cached) = self.cache.get::<ControlMonitoringSummary>(&cache_key).await? {
            return Ok(cached);
        }

        let summary = sqlx::query_as::<_, ControlMonitoringSummary>(
            r#"
            SELECT * FROM v_control_monitoring_summary
            WHERE organization_id = $1
            "#,
        )
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .unwrap_or(ControlMonitoringSummary {
            organization_id: org_id,
            total_controls: 0,
            monitored_controls: 0,
            healthy_controls: 0,
            degraded_controls: 0,
            failing_controls: 0,
            unknown_controls: 0,
            alerting_controls: 0,
            avg_health_score: Some(100.0),
            total_test_runs: Some(0),
            total_passed: Some(0),
            total_failed: Some(0),
        });

        self.cache.set(&cache_key, &summary, Some(CACHE_TTL)).await?;

        Ok(summary)
    }

    /// Get monitoring status for a control
    pub async fn get_control_monitoring_status(
        &self,
        org_id: Uuid,
        control_id: Uuid,
    ) -> AppResult<ControlMonitoringStatus> {
        let status = sqlx::query_as::<_, ControlMonitoringStatus>(
            r#"
            SELECT id, organization_id, control_id, monitoring_enabled, last_test_at, last_test_status,
                   consecutive_failures, consecutive_passes, current_health, health_score,
                   total_tests, passed_tests, failed_tests, last_failure_at, last_failure_reason,
                   alert_status, acknowledged_by, acknowledged_at, created_at, updated_at
            FROM control_monitoring_status
            WHERE organization_id = $1 AND control_id = $2
            "#,
        )
        .bind(org_id)
        .bind(control_id)
        .fetch_optional(&self.db)
        .await?;

        match status {
            Some(s) => Ok(s),
            None => {
                // Create default status
                let new_status = sqlx::query_as::<_, ControlMonitoringStatus>(
                    r#"
                    INSERT INTO control_monitoring_status (organization_id, control_id)
                    VALUES ($1, $2)
                    RETURNING id, organization_id, control_id, monitoring_enabled, last_test_at, last_test_status,
                              consecutive_failures, consecutive_passes, current_health, health_score,
                              total_tests, passed_tests, failed_tests, last_failure_at, last_failure_reason,
                              alert_status, acknowledged_by, acknowledged_at, created_at, updated_at
                    "#,
                )
                .bind(org_id)
                .bind(control_id)
                .fetch_one(&self.db)
                .await?;

                Ok(new_status)
            }
        }
    }

    /// List controls by health status
    pub async fn list_controls_by_health(
        &self,
        org_id: Uuid,
        health: Option<String>,
        limit: i64,
    ) -> AppResult<Vec<ControlMonitoringStatus>> {
        let statuses = sqlx::query_as::<_, ControlMonitoringStatus>(
            r#"
            SELECT id, organization_id, control_id, monitoring_enabled, last_test_at, last_test_status,
                   consecutive_failures, consecutive_passes, current_health, health_score,
                   total_tests, passed_tests, failed_tests, last_failure_at, last_failure_reason,
                   alert_status, acknowledged_by, acknowledged_at, created_at, updated_at
            FROM control_monitoring_status
            WHERE organization_id = $1
              AND ($2::text IS NULL OR current_health = $2)
            ORDER BY health_score ASC, last_failure_at DESC NULLS LAST
            LIMIT $3
            "#,
        )
        .bind(org_id)
        .bind(&health)
        .bind(limit)
        .fetch_all(&self.db)
        .await?;

        Ok(statuses)
    }

    /// Acknowledge alert for a control
    pub async fn acknowledge_alert(
        &self,
        org_id: Uuid,
        control_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<ControlMonitoringStatus> {
        let status = sqlx::query_as::<_, ControlMonitoringStatus>(
            r#"
            UPDATE control_monitoring_status
            SET alert_status = 'acknowledged',
                acknowledged_by = $3,
                acknowledged_at = NOW(),
                updated_at = NOW()
            WHERE organization_id = $1 AND control_id = $2
            RETURNING id, organization_id, control_id, monitoring_enabled, last_test_at, last_test_status,
                      consecutive_failures, consecutive_passes, current_health, health_score,
                      total_tests, passed_tests, failed_tests, last_failure_at, last_failure_reason,
                      alert_status, acknowledged_by, acknowledged_at, created_at, updated_at
            "#,
        )
        .bind(org_id)
        .bind(control_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        // Invalidate cache
        let cache_key = org_cache_key(&org_id.to_string(), CACHE_PREFIX_MONITORING, "summary");
        self.cache.delete(&cache_key).await?;

        Ok(status)
    }

    // ==================== Test Runs ====================

    /// List test runs
    pub async fn list_test_runs(
        &self,
        org_id: Uuid,
        query: ListTestRunsQuery,
    ) -> AppResult<Vec<ControlTestRun>> {
        let limit = query.limit.unwrap_or(50).min(200);
        let offset = query.offset.unwrap_or(0);

        let runs = sqlx::query_as::<_, ControlTestRun>(
            r#"
            SELECT id, organization_id, control_test_id, control_id, run_type, triggered_by,
                   started_at, completed_at, status, execution_time_ms, notes, raw_output,
                   error_message, evidence_created_id, alert_sent, alert_sent_at,
                   remediation_suggested_id, created_at
            FROM control_test_runs
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR control_id = $2)
              AND ($3::uuid IS NULL OR control_test_id = $3)
              AND ($4::text IS NULL OR status = $4)
            ORDER BY started_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(org_id)
        .bind(query.control_id)
        .bind(query.control_test_id)
        .bind(&query.status)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        Ok(runs)
    }

    /// Get test run by ID
    pub async fn get_test_run(&self, org_id: Uuid, run_id: Uuid) -> AppResult<ControlTestRun> {
        let run = sqlx::query_as::<_, ControlTestRun>(
            r#"
            SELECT id, organization_id, control_test_id, control_id, run_type, triggered_by,
                   started_at, completed_at, status, execution_time_ms, notes, raw_output,
                   error_message, evidence_created_id, alert_sent, alert_sent_at,
                   remediation_suggested_id, created_at
            FROM control_test_runs
            WHERE organization_id = $1 AND id = $2
            "#,
        )
        .bind(org_id)
        .bind(run_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound(format!("Test run {} not found", run_id)))?;

        Ok(run)
    }

    /// Create a test run (usually called by worker)
    pub async fn create_test_run(
        &self,
        org_id: Uuid,
        control_test_id: Uuid,
        control_id: Uuid,
        run_type: &str,
        triggered_by: Option<Uuid>,
    ) -> AppResult<ControlTestRun> {
        let run = sqlx::query_as::<_, ControlTestRun>(
            r#"
            INSERT INTO control_test_runs (organization_id, control_test_id, control_id, run_type, triggered_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING id, organization_id, control_test_id, control_id, run_type, triggered_by,
                      started_at, completed_at, status, execution_time_ms, notes, raw_output,
                      error_message, evidence_created_id, alert_sent, alert_sent_at,
                      remediation_suggested_id, created_at
            "#,
        )
        .bind(org_id)
        .bind(control_test_id)
        .bind(control_id)
        .bind(run_type)
        .bind(triggered_by)
        .fetch_one(&self.db)
        .await?;

        Ok(run)
    }

    /// Complete a test run with results
    pub async fn complete_test_run(
        &self,
        run_id: Uuid,
        status: &str,
        notes: Option<String>,
        raw_output: Option<String>,
        error_message: Option<String>,
        evidence_id: Option<Uuid>,
        execution_time_ms: i32,
    ) -> AppResult<ControlTestRun> {
        let run = sqlx::query_as::<_, ControlTestRun>(
            r#"
            UPDATE control_test_runs
            SET completed_at = NOW(),
                status = $2,
                notes = $3,
                raw_output = $4,
                error_message = $5,
                evidence_created_id = $6,
                execution_time_ms = $7
            WHERE id = $1
            RETURNING id, organization_id, control_test_id, control_id, run_type, triggered_by,
                      started_at, completed_at, status, execution_time_ms, notes, raw_output,
                      error_message, evidence_created_id, alert_sent, alert_sent_at,
                      remediation_suggested_id, created_at
            "#,
        )
        .bind(run_id)
        .bind(status)
        .bind(&notes)
        .bind(&raw_output)
        .bind(&error_message)
        .bind(evidence_id)
        .bind(execution_time_ms)
        .fetch_one(&self.db)
        .await?;

        // Update monitoring status
        sqlx::query("SELECT update_control_monitoring_status($1, $2, $3)")
            .bind(run.organization_id)
            .bind(run.control_id)
            .bind(status)
            .execute(&self.db)
            .await?;

        // Invalidate monitoring cache
        let cache_key = org_cache_key(&run.organization_id.to_string(), CACHE_PREFIX_MONITORING, "summary");
        self.cache.delete(&cache_key).await?;

        Ok(run)
    }

    // ==================== Alerts ====================

    /// List alerts
    pub async fn list_alerts(
        &self,
        org_id: Uuid,
        query: ListAlertsQuery,
    ) -> AppResult<Vec<ControlTestAlert>> {
        let limit = query.limit.unwrap_or(50).min(200);
        let offset = query.offset.unwrap_or(0);

        let alerts = sqlx::query_as::<_, ControlTestAlert>(
            r#"
            SELECT id, organization_id, control_test_id, test_run_id, alert_type, severity,
                   title, message, recipients, email_sent, in_app_sent, acknowledged_by,
                   acknowledged_at, resolved_by, resolved_at, resolution_notes, created_at
            FROM control_test_alerts
            WHERE organization_id = $1
              AND ($2::uuid IS NULL OR control_test_id = $2)
              AND ($3::text IS NULL OR alert_type = $3)
              AND ($4::bool IS NULL OR $4 = false OR acknowledged_at IS NULL)
            ORDER BY created_at DESC
            LIMIT $5 OFFSET $6
            "#,
        )
        .bind(org_id)
        .bind(query.control_test_id)
        .bind(&query.alert_type)
        .bind(query.unacknowledged_only)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.db)
        .await?;

        Ok(alerts)
    }

    /// Create an alert
    pub async fn create_alert(
        &self,
        org_id: Uuid,
        control_test_id: Uuid,
        test_run_id: Option<Uuid>,
        alert_type: &str,
        severity: &str,
        title: &str,
        message: &str,
        recipients: Vec<Uuid>,
    ) -> AppResult<ControlTestAlert> {
        let alert = sqlx::query_as::<_, ControlTestAlert>(
            r#"
            INSERT INTO control_test_alerts (
                organization_id, control_test_id, test_run_id, alert_type, severity,
                title, message, recipients
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, organization_id, control_test_id, test_run_id, alert_type, severity,
                      title, message, recipients, email_sent, in_app_sent, acknowledged_by,
                      acknowledged_at, resolved_by, resolved_at, resolution_notes, created_at
            "#,
        )
        .bind(org_id)
        .bind(control_test_id)
        .bind(test_run_id)
        .bind(alert_type)
        .bind(severity)
        .bind(title)
        .bind(message)
        .bind(&recipients)
        .fetch_one(&self.db)
        .await?;

        // Mark alert sent on test run
        if let Some(run_id) = test_run_id {
            sqlx::query("UPDATE control_test_runs SET alert_sent = true, alert_sent_at = NOW() WHERE id = $1")
                .bind(run_id)
                .execute(&self.db)
                .await?;
        }

        Ok(alert)
    }

    /// Acknowledge an alert
    pub async fn acknowledge_test_alert(
        &self,
        org_id: Uuid,
        alert_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<ControlTestAlert> {
        let alert = sqlx::query_as::<_, ControlTestAlert>(
            r#"
            UPDATE control_test_alerts
            SET acknowledged_by = $3, acknowledged_at = NOW()
            WHERE organization_id = $1 AND id = $2
            RETURNING id, organization_id, control_test_id, test_run_id, alert_type, severity,
                      title, message, recipients, email_sent, in_app_sent, acknowledged_by,
                      acknowledged_at, resolved_by, resolved_at, resolution_notes, created_at
            "#,
        )
        .bind(org_id)
        .bind(alert_id)
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(alert)
    }

    /// Resolve an alert
    pub async fn resolve_alert(
        &self,
        org_id: Uuid,
        alert_id: Uuid,
        user_id: Uuid,
        resolution_notes: Option<String>,
    ) -> AppResult<ControlTestAlert> {
        let alert = sqlx::query_as::<_, ControlTestAlert>(
            r#"
            UPDATE control_test_alerts
            SET resolved_by = $3, resolved_at = NOW(), resolution_notes = $4
            WHERE organization_id = $1 AND id = $2
            RETURNING id, organization_id, control_test_id, test_run_id, alert_type, severity,
                      title, message, recipients, email_sent, in_app_sent, acknowledged_by,
                      acknowledged_at, resolved_by, resolved_at, resolution_notes, created_at
            "#,
        )
        .bind(org_id)
        .bind(alert_id)
        .bind(user_id)
        .bind(&resolution_notes)
        .fetch_one(&self.db)
        .await?;

        Ok(alert)
    }

    /// Get unacknowledged alert count
    pub async fn get_unacknowledged_alert_count(&self, org_id: Uuid) -> AppResult<i64> {
        let (count,): (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM control_test_alerts
            WHERE organization_id = $1 AND acknowledged_at IS NULL AND resolved_at IS NULL
            "#,
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        Ok(count)
    }

    /// Send alert notifications (in-app and email)
    pub async fn send_alert_notifications(
        &self,
        notification_service: &NotificationService,
        org_id: Uuid,
        alert: &ControlTestAlert,
    ) -> AppResult<()> {
        // Send in-app notifications to all recipients
        for recipient_id in &alert.recipients {
            let notification_data = serde_json::json!({
                "alert_id": alert.id,
                "control_test_id": alert.control_test_id,
                "alert_type": alert.alert_type,
                "severity": alert.severity,
            });

            notification_service
                .create_notification(
                    org_id,
                    CreateNotification {
                        user_id: *recipient_id,
                        notification_type: "control_test_alert".to_string(),
                        title: alert.title.clone(),
                        message: alert.message.clone(),
                        data: Some(notification_data),
                    },
                )
                .await?;
        }

        // Mark in-app as sent
        sqlx::query("UPDATE control_test_alerts SET in_app_sent = true WHERE id = $1")
            .bind(alert.id)
            .execute(&self.db)
            .await?;

        Ok(())
    }

    // ==================== Trigger Manual Test ====================

    /// Trigger a manual test run
    pub async fn trigger_test(
        &self,
        org_id: Uuid,
        control_test_id: Uuid,
        user_id: Uuid,
    ) -> AppResult<ControlTestRun> {
        // Get the control ID for this test
        let (control_id,): (Uuid,) = sqlx::query_as(
            "SELECT control_id FROM control_tests WHERE id = $1",
        )
        .bind(control_test_id)
        .fetch_one(&self.db)
        .await?;

        // Create a run with manual type
        let run = self
            .create_test_run(org_id, control_test_id, control_id, "manual", Some(user_id))
            .await?;

        tracing::info!(
            "Manual test run triggered for test {} by user {}",
            control_test_id,
            user_id
        );

        Ok(run)
    }
}
