use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Policy status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus {
    Draft,
    PendingApproval,
    Published,
    Archived,
}

impl Default for PolicyStatus {
    fn default() -> Self {
        Self::Draft
    }
}

/// Policy categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyCategory {
    Security,
    Privacy,
    Hr,
    It,
    Compliance,
    Operations,
    Business,
    Other,
}

/// Policy entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Policy {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub title: String,
    pub category: Option<String>,
    pub content: Option<String>,
    pub version: i32,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub approver_id: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub effective_date: Option<NaiveDate>,
    pub review_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Policy with acknowledgment stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyWithStats {
    #[serde(flatten)]
    pub policy: Policy,
    pub acknowledgment_count: i64,
    pub pending_acknowledgments: i64,
}

/// Policy version history
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PolicyVersion {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub version: i32,
    pub content: Option<String>,
    pub changed_by: Option<Uuid>,
    pub change_summary: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Policy acknowledgment
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PolicyAcknowledgment {
    pub id: Uuid,
    pub policy_id: Uuid,
    pub policy_version: i32,
    pub user_id: Uuid,
    pub acknowledged_at: DateTime<Utc>,
    pub ip_address: Option<String>,
}

/// Create policy request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePolicy {
    pub code: String,
    pub title: String,
    pub category: Option<String>,
    pub content: Option<String>,
    pub owner_id: Option<Uuid>,
    pub effective_date: Option<NaiveDate>,
    pub review_date: Option<NaiveDate>,
}

/// Update policy request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdatePolicy {
    pub code: Option<String>,
    pub title: Option<String>,
    pub category: Option<String>,
    pub content: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub effective_date: Option<NaiveDate>,
    pub review_date: Option<NaiveDate>,
    pub change_summary: Option<String>,
}

/// List policies query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListPoliciesQuery {
    pub status: Option<String>,
    pub category: Option<String>,
    pub owner_id: Option<Uuid>,
    pub search: Option<String>,
    pub needs_review: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Policy statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyStats {
    pub total: i64,
    pub published: i64,
    pub draft: i64,
    pub pending_approval: i64,
    pub needs_review: i64,
    pub by_category: Vec<CategoryCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CategoryCount {
    pub category: Option<String>,
    pub count: i64,
}

impl Policy {
    pub fn validate_create(input: &CreatePolicy) -> Result<(), String> {
        if input.code.trim().is_empty() {
            return Err("Policy code is required".to_string());
        }
        if input.code.len() > 50 {
            return Err("Policy code must be 50 characters or less".to_string());
        }
        if input.title.trim().is_empty() {
            return Err("Policy title is required".to_string());
        }
        if input.title.len() > 255 {
            return Err("Policy title must be 255 characters or less".to_string());
        }

        if let Some(ref cat) = input.category {
            if !["security", "privacy", "hr", "it", "compliance", "operations", "business", "other"]
                .contains(&cat.as_str())
            {
                return Err("Invalid policy category".to_string());
            }
        }

        Ok(())
    }

    pub fn needs_review(&self) -> bool {
        if let Some(review_date) = self.review_date {
            review_date <= chrono::Utc::now().date_naive()
        } else {
            false
        }
    }
}
