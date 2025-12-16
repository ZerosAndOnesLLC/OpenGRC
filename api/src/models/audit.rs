use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Audit type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditType {
    Internal,
    External,
    Certification,
    Compliance,
    Readiness,
}

/// Audit status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuditStatus {
    Planning,
    InProgress,
    Fieldwork,
    Reporting,
    Completed,
    Cancelled,
}

impl Default for AuditStatus {
    fn default() -> Self {
        Self::Planning
    }
}

/// Audit entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Audit {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub framework_id: Option<Uuid>,
    pub audit_type: Option<String>,
    pub auditor_firm: Option<String>,
    pub auditor_contact: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Audit with stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditWithStats {
    #[serde(flatten)]
    pub audit: Audit,
    pub request_count: i64,
    pub open_requests: i64,
    pub finding_count: i64,
    pub open_findings: i64,
}

/// Audit request
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditRequest {
    pub id: Uuid,
    pub audit_id: Uuid,
    pub request_type: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub due_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Audit request response
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditRequestResponse {
    pub id: Uuid,
    pub audit_request_id: Uuid,
    pub response_text: Option<String>,
    pub evidence_ids: Option<Vec<Uuid>>,
    pub responded_by: Option<Uuid>,
    pub responded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Audit finding
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditFinding {
    pub id: Uuid,
    pub audit_id: Uuid,
    pub finding_type: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub recommendation: Option<String>,
    pub control_ids: Option<Vec<Uuid>>,
    pub status: Option<String>,
    pub remediation_plan: Option<String>,
    pub remediation_due: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create audit request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAudit {
    pub name: String,
    pub framework_id: Option<Uuid>,
    pub audit_type: Option<String>,
    pub auditor_firm: Option<String>,
    pub auditor_contact: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
}

/// Update audit request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAudit {
    pub name: Option<String>,
    pub framework_id: Option<Uuid>,
    pub audit_type: Option<String>,
    pub auditor_firm: Option<String>,
    pub auditor_contact: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub status: Option<String>,
}

/// Create audit request (the request entity)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditRequest {
    pub request_type: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub due_at: Option<DateTime<Utc>>,
}

/// Create audit finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAuditFinding {
    pub finding_type: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub recommendation: Option<String>,
    pub control_ids: Option<Vec<Uuid>>,
    pub remediation_plan: Option<String>,
    pub remediation_due: Option<NaiveDate>,
}

/// Update audit finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAuditFinding {
    pub finding_type: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub recommendation: Option<String>,
    pub control_ids: Option<Vec<Uuid>>,
    pub status: Option<String>,
    pub remediation_plan: Option<String>,
    pub remediation_due: Option<NaiveDate>,
}

/// Create request response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRequestResponse {
    pub response_text: Option<String>,
    pub evidence_ids: Option<Vec<Uuid>>,
}

/// List audits query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListAuditsQuery {
    pub status: Option<String>,
    pub audit_type: Option<String>,
    pub framework_id: Option<Uuid>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStats {
    pub total: i64,
    pub in_progress: i64,
    pub completed: i64,
    pub by_type: Vec<AuditTypeCount>,
    pub open_findings: i64,
    pub overdue_requests: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditTypeCount {
    pub audit_type: Option<String>,
    pub count: i64,
}

// ==================== Evidence Packaging ====================

/// Evidence item for audit package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvidenceItem {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub evidence_type: String,
    pub source: String,
    pub file_path: Option<String>,
    pub file_size: Option<i64>,
    pub mime_type: Option<String>,
    pub collected_at: DateTime<Utc>,
    pub linked_controls: Vec<String>,
    pub linked_requests: Vec<String>,
}

/// Audit evidence package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvidencePackage {
    pub audit_id: Uuid,
    pub audit_name: String,
    pub framework_name: Option<String>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub evidence_count: usize,
    pub total_file_size: i64,
    pub evidence: Vec<AuditEvidenceItem>,
    pub generated_at: DateTime<Utc>,
}

impl Audit {
    pub fn validate_create(input: &CreateAudit) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return Err("Audit name is required".to_string());
        }
        if input.name.len() > 255 {
            return Err("Audit name must be 255 characters or less".to_string());
        }

        if let Some(ref audit_type) = input.audit_type {
            if !["internal", "external", "certification", "compliance", "readiness"]
                .contains(&audit_type.as_str())
            {
                return Err("Invalid audit type".to_string());
            }
        }

        if let (Some(start), Some(end)) = (input.period_start, input.period_end) {
            if start > end {
                return Err("Period start must be before period end".to_string());
            }
        }

        Ok(())
    }
}

impl AuditRequest {
    pub fn is_overdue(&self) -> bool {
        if let Some(due_at) = self.due_at {
            self.status.as_deref() == Some("open") && due_at < chrono::Utc::now()
        } else {
            false
        }
    }
}

impl AuditFinding {
    pub fn remediation_overdue(&self) -> bool {
        if let Some(due_date) = self.remediation_due {
            self.status.as_deref() != Some("closed")
                && due_date < chrono::Utc::now().date_naive()
        } else {
            false
        }
    }
}
