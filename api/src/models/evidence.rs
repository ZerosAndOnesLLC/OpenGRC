use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Evidence types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Document,
    Screenshot,
    Log,
    Automated,
    Config,
    Report,
}

impl Default for EvidenceType {
    fn default() -> Self {
        Self::Document
    }
}

/// Evidence sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    Manual,
    Aws,
    Github,
    Okta,
    Azure,
    Gcp,
    Datadog,
    Other,
}

impl Default for EvidenceSource {
    fn default() -> Self {
        Self::Manual
    }
}

/// Evidence entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Evidence {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub evidence_type: String,
    pub source: String,
    pub source_reference: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub collected_at: DateTime<Utc>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Evidence with linked controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceWithLinks {
    #[serde(flatten)]
    pub evidence: Evidence,
    pub linked_control_count: i64,
    pub linked_controls: Option<Vec<LinkedControl>>,
}

/// Linked control info
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LinkedControl {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}

/// Evidence-Control link
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EvidenceControlLink {
    pub id: Uuid,
    pub evidence_id: Uuid,
    pub control_id: Uuid,
    pub control_test_result_id: Option<Uuid>,
    pub linked_by: Option<Uuid>,
    pub linked_at: DateTime<Utc>,
}

/// Create evidence request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvidence {
    pub title: String,
    pub description: Option<String>,
    pub evidence_type: Option<String>,
    pub source: Option<String>,
    pub source_reference: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
}

/// Update evidence request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEvidence {
    pub title: Option<String>,
    pub description: Option<String>,
    pub evidence_type: Option<String>,
    pub source: Option<String>,
    pub source_reference: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
}

/// List evidence query params
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListEvidenceQuery {
    pub evidence_type: Option<String>,
    pub source: Option<String>,
    pub control_id: Option<Uuid>,
    pub search: Option<String>,
    pub expired: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Evidence statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceStats {
    pub total: i64,
    pub by_type: Vec<TypeCount>,
    pub by_source: Vec<SourceCount>,
    pub expiring_soon: i64,
    pub expired: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TypeCount {
    pub evidence_type: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SourceCount {
    pub source: String,
    pub count: i64,
}

impl Evidence {
    pub fn validate_create(input: &CreateEvidence) -> Result<(), String> {
        if input.title.trim().is_empty() {
            return Err("Evidence title is required".to_string());
        }
        if input.title.len() > 255 {
            return Err("Evidence title must be 255 characters or less".to_string());
        }

        // Validate evidence_type if provided
        if let Some(ref et) = input.evidence_type {
            if !["document", "screenshot", "log", "automated", "config", "report"]
                .contains(&et.as_str())
            {
                return Err("Invalid evidence type".to_string());
            }
        }

        // Validate source if provided
        if let Some(ref src) = input.source {
            if !["manual", "aws", "github", "okta", "azure", "gcp", "datadog", "other"]
                .contains(&src.as_str())
            {
                return Err("Invalid evidence source".to_string());
            }
        }

        // Validate date range
        if let (Some(from), Some(until)) = (&input.valid_from, &input.valid_until) {
            if from > until {
                return Err("valid_from must be before valid_until".to_string());
            }
        }

        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        if let Some(valid_until) = self.valid_until {
            valid_until < Utc::now()
        } else {
            false
        }
    }

    pub fn is_expiring_soon(&self, days: i64) -> bool {
        if let Some(valid_until) = self.valid_until {
            let threshold = Utc::now() + chrono::Duration::days(days);
            valid_until <= threshold && valid_until > Utc::now()
        } else {
            false
        }
    }
}

// ==================== Evidence with Freshness ====================

/// Evidence entity with freshness fields
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EvidenceWithFreshness {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub evidence_type: String,
    pub source: String,
    pub source_reference: Option<String>,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub collected_at: DateTime<Utc>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub uploaded_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    // Freshness fields
    pub freshness_score: Option<i32>,
    pub days_stale: Option<i32>,
    pub freshness_sla_days: Option<i32>,
    pub content_hash: Option<String>,
    pub last_verified_at: Option<DateTime<Utc>>,
}

impl EvidenceWithFreshness {
    pub fn freshness_status(&self) -> &'static str {
        match self.freshness_score {
            Some(score) if score >= 80 => "fresh",
            Some(score) if score >= 50 => "aging",
            Some(score) if score >= 20 => "stale",
            Some(_) => "critical",
            None => "unknown",
        }
    }
}

// ==================== Evidence Collection Task ====================

/// Evidence collection task for scheduled collection
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EvidenceCollectionTask {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub integration_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub schedule: Option<String>,
    pub collection_config: serde_json::Value,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub status: String,
    pub enabled: bool,
    pub cron_schedule: Option<String>,
    pub timezone: Option<String>,
    pub evidence_type: Option<String>,
    pub auto_link_controls: bool,
    pub retention_days: Option<i32>,
    pub last_error: Option<String>,
    pub error_count: i32,
    pub success_count: i32,
    pub created_at: DateTime<Utc>,
}

/// Create evidence collection task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEvidenceCollectionTask {
    pub integration_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub cron_schedule: Option<String>,
    pub timezone: Option<String>,
    pub collection_config: Option<serde_json::Value>,
    pub evidence_type: Option<String>,
    pub auto_link_controls: Option<bool>,
    pub retention_days: Option<i32>,
}

/// Update evidence collection task request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEvidenceCollectionTask {
    pub name: Option<String>,
    pub description: Option<String>,
    pub cron_schedule: Option<String>,
    pub timezone: Option<String>,
    pub collection_config: Option<serde_json::Value>,
    pub evidence_type: Option<String>,
    pub auto_link_controls: Option<bool>,
    pub retention_days: Option<i32>,
    pub enabled: Option<bool>,
}

// ==================== Evidence Collection Run ====================

/// Record of a collection task execution
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EvidenceCollectionRun {
    pub id: Uuid,
    pub task_id: Uuid,
    pub organization_id: Uuid,
    pub integration_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub evidence_created: i32,
    pub evidence_updated: i32,
    pub evidence_unchanged: i32,
    pub changes_detected: i32,
    pub controls_linked: i32,
    pub error_message: Option<String>,
    pub duration_ms: Option<i32>,
    pub created_at: DateTime<Utc>,
}

// ==================== Evidence Change ====================

/// Evidence change record for change detection
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EvidenceChange {
    pub id: Uuid,
    pub evidence_id: Uuid,
    pub organization_id: Uuid,
    pub change_type: String,
    pub previous_hash: Option<String>,
    pub new_hash: Option<String>,
    pub previous_data: Option<serde_json::Value>,
    pub new_data: Option<serde_json::Value>,
    pub change_summary: Option<String>,
    pub detected_at: DateTime<Utc>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Evidence change with evidence details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceChangeWithDetails {
    #[serde(flatten)]
    pub change: EvidenceChange,
    pub evidence_title: String,
    pub evidence_source: String,
    pub evidence_type: String,
}

// ==================== Evidence Control Mapping Rule ====================

/// Rule for auto-linking evidence to controls
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EvidenceControlMappingRule {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub priority: i32,
    pub source_pattern: Option<String>,
    pub type_pattern: Option<String>,
    pub title_pattern: Option<String>,
    pub control_codes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create mapping rule request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMappingRule {
    pub name: String,
    pub description: Option<String>,
    pub source_pattern: Option<String>,
    pub type_pattern: Option<String>,
    pub title_pattern: Option<String>,
    pub control_codes: Vec<String>,
    pub priority: Option<i32>,
}

/// Update mapping rule request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMappingRule {
    pub name: Option<String>,
    pub description: Option<String>,
    pub source_pattern: Option<String>,
    pub type_pattern: Option<String>,
    pub title_pattern: Option<String>,
    pub control_codes: Option<Vec<String>>,
    pub priority: Option<i32>,
    pub enabled: Option<bool>,
}

// ==================== Evidence Freshness SLA ====================

/// SLA definition for evidence freshness
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EvidenceFreshnessSla {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub evidence_type: Option<String>,
    pub source: Option<String>,
    pub max_age_days: i32,
    pub warning_days: i32,
    pub critical_days: i32,
    pub auto_expire: bool,
    pub created_at: DateTime<Utc>,
}

/// Create SLA request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFreshnessSla {
    pub evidence_type: Option<String>,
    pub source: Option<String>,
    pub max_age_days: i32,
    pub warning_days: i32,
    pub critical_days: i32,
    pub auto_expire: Option<bool>,
}

// ==================== Freshness Statistics ====================

/// Evidence freshness summary for dashboard
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EvidenceFreshnessSummary {
    pub organization_id: Uuid,
    pub total_evidence: i64,
    pub fresh_count: i64,
    pub aging_count: i64,
    pub stale_count: i64,
    pub critical_count: i64,
    pub expired_count: i64,
    pub expiring_soon_count: i64,
    pub avg_freshness_score: Option<i32>,
    pub max_days_stale: Option<i32>,
}

/// Stale evidence by source
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StaleEvidenceBySource {
    pub source: String,
    pub integration_id: Option<Uuid>,
    pub stale_count: i64,
    pub oldest_days: Option<i32>,
    pub avg_freshness: Option<f64>,
}

/// Evidence collection task with last run info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceCollectionTaskWithStats {
    #[serde(flatten)]
    pub task: EvidenceCollectionTask,
    pub integration_name: Option<String>,
    pub integration_type: Option<String>,
    pub last_run_created: Option<i32>,
    pub last_run_updated: Option<i32>,
    pub last_run_changes: Option<i32>,
    pub last_run_duration_ms: Option<i32>,
}

/// List collection tasks query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListCollectionTasksQuery {
    pub integration_id: Option<Uuid>,
    pub enabled: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List evidence changes query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListEvidenceChangesQuery {
    pub evidence_id: Option<Uuid>,
    pub change_type: Option<String>,
    pub acknowledged: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}
