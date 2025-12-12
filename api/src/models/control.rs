use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Control types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ControlType {
    Preventive,
    Detective,
    Corrective,
}

impl Default for ControlType {
    fn default() -> Self {
        Self::Preventive
    }
}

impl std::fmt::Display for ControlType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Preventive => write!(f, "preventive"),
            Self::Detective => write!(f, "detective"),
            Self::Corrective => write!(f, "corrective"),
        }
    }
}

/// Control frequency
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ControlFrequency {
    Continuous,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annual,
}

impl Default for ControlFrequency {
    fn default() -> Self {
        Self::Continuous
    }
}

impl std::fmt::Display for ControlFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Continuous => write!(f, "continuous"),
            Self::Daily => write!(f, "daily"),
            Self::Weekly => write!(f, "weekly"),
            Self::Monthly => write!(f, "monthly"),
            Self::Quarterly => write!(f, "quarterly"),
            Self::Annual => write!(f, "annual"),
        }
    }
}

/// Control status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ControlStatus {
    NotImplemented,
    InProgress,
    Implemented,
    NotApplicable,
}

impl Default for ControlStatus {
    fn default() -> Self {
        Self::NotImplemented
    }
}

impl std::fmt::Display for ControlStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented => write!(f, "not_implemented"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Implemented => write!(f, "implemented"),
            Self::NotApplicable => write!(f, "not_applicable"),
        }
    }
}

/// Control entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Control {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub control_type: String,
    pub frequency: String,
    pub owner_id: Option<Uuid>,
    pub status: String,
    pub implementation_notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Control with mapped requirements count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlWithMappings {
    #[serde(flatten)]
    pub control: Control,
    pub requirement_count: i64,
    pub mapped_requirements: Option<Vec<MappedRequirement>>,
}

/// Mapped requirement info
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MappedRequirement {
    pub id: Uuid,
    pub framework_id: Uuid,
    pub framework_name: String,
    pub code: String,
    pub name: String,
}

/// Control-Requirement mapping
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ControlRequirementMapping {
    pub id: Uuid,
    pub control_id: Uuid,
    pub framework_requirement_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Create control request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateControl {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub control_type: Option<String>,
    pub frequency: Option<String>,
    pub owner_id: Option<Uuid>,
    pub status: Option<String>,
    pub implementation_notes: Option<String>,
}

/// Update control request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateControl {
    pub code: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub control_type: Option<String>,
    pub frequency: Option<String>,
    pub owner_id: Option<Uuid>,
    pub status: Option<String>,
    pub implementation_notes: Option<String>,
}

/// Control test definition
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ControlTest {
    pub id: Uuid,
    pub control_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub test_type: String,
    pub automation_config: Option<serde_json::Value>,
    pub frequency: Option<String>,
    pub next_due_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Create control test request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateControlTest {
    pub name: String,
    pub description: Option<String>,
    pub test_type: Option<String>,
    pub automation_config: Option<serde_json::Value>,
    pub frequency: Option<String>,
    pub next_due_at: Option<DateTime<Utc>>,
}

/// Update control test request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateControlTest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub test_type: Option<String>,
    pub automation_config: Option<serde_json::Value>,
    pub frequency: Option<String>,
    pub next_due_at: Option<DateTime<Utc>>,
}

/// Control test result
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ControlTestResult {
    pub id: Uuid,
    pub control_test_id: Uuid,
    pub performed_by: Option<Uuid>,
    pub performed_at: DateTime<Utc>,
    pub status: String,
    pub notes: Option<String>,
    pub evidence_ids: Option<Vec<Uuid>>,
    pub created_at: DateTime<Utc>,
}

/// Create test result request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTestResult {
    pub status: String,
    pub notes: Option<String>,
    pub evidence_ids: Option<Vec<Uuid>>,
}

/// List controls query params
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListControlsQuery {
    pub status: Option<String>,
    pub control_type: Option<String>,
    pub owner_id: Option<Uuid>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Control statistics for dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlStats {
    pub total: i64,
    pub implemented: i64,
    pub in_progress: i64,
    pub not_implemented: i64,
    pub not_applicable: i64,
    pub implementation_percentage: f64,
}

impl Control {
    pub fn validate_create(input: &CreateControl) -> Result<(), String> {
        if input.code.trim().is_empty() {
            return Err("Control code is required".to_string());
        }
        if input.code.len() > 50 {
            return Err("Control code must be 50 characters or less".to_string());
        }
        if input.name.trim().is_empty() {
            return Err("Control name is required".to_string());
        }
        if input.name.len() > 255 {
            return Err("Control name must be 255 characters or less".to_string());
        }

        // Validate control_type if provided
        if let Some(ref ct) = input.control_type {
            if !["preventive", "detective", "corrective"].contains(&ct.as_str()) {
                return Err("Invalid control type. Must be: preventive, detective, or corrective".to_string());
            }
        }

        // Validate frequency if provided
        if let Some(ref freq) = input.frequency {
            if !["continuous", "daily", "weekly", "monthly", "quarterly", "annual"].contains(&freq.as_str()) {
                return Err("Invalid frequency. Must be: continuous, daily, weekly, monthly, quarterly, or annual".to_string());
            }
        }

        // Validate status if provided
        if let Some(ref status) = input.status {
            if !["not_implemented", "in_progress", "implemented", "not_applicable"].contains(&status.as_str()) {
                return Err("Invalid status. Must be: not_implemented, in_progress, implemented, or not_applicable".to_string());
            }
        }

        Ok(())
    }
}

impl ControlTest {
    pub fn validate_create(input: &CreateControlTest) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return Err("Test name is required".to_string());
        }
        if input.name.len() > 255 {
            return Err("Test name must be 255 characters or less".to_string());
        }

        // Validate test_type if provided
        if let Some(ref tt) = input.test_type {
            if !["manual", "automated"].contains(&tt.as_str()) {
                return Err("Invalid test type. Must be: manual or automated".to_string());
            }
        }

        Ok(())
    }
}
