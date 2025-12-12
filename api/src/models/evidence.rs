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
