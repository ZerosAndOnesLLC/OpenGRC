use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Vendor criticality levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VendorCriticality {
    Critical,
    High,
    Medium,
    Low,
}

impl Default for VendorCriticality {
    fn default() -> Self {
        Self::Medium
    }
}

/// Vendor status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VendorStatus {
    Active,
    Inactive,
    UnderReview,
    Terminated,
}

impl Default for VendorStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Vendor entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Vendor {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub criticality: Option<String>,
    pub data_classification: Option<String>,
    pub status: Option<String>,
    pub contract_start: Option<NaiveDate>,
    pub contract_end: Option<NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub website: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Vendor with assessment info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorWithAssessment {
    #[serde(flatten)]
    pub vendor: Vendor,
    pub latest_assessment: Option<VendorAssessment>,
    pub assessment_count: i64,
}

/// Vendor assessment
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VendorAssessment {
    pub id: Uuid,
    pub vendor_id: Uuid,
    pub assessment_type: Option<String>,
    pub assessed_by: Option<Uuid>,
    pub assessed_at: DateTime<Utc>,
    pub risk_rating: Option<String>,
    pub findings: Option<String>,
    pub recommendations: Option<String>,
    pub next_assessment_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
}

/// Create vendor request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVendor {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub criticality: Option<String>,
    pub data_classification: Option<String>,
    pub contract_start: Option<NaiveDate>,
    pub contract_end: Option<NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub website: Option<String>,
}

/// Update vendor request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVendor {
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub criticality: Option<String>,
    pub data_classification: Option<String>,
    pub status: Option<String>,
    pub contract_start: Option<NaiveDate>,
    pub contract_end: Option<NaiveDate>,
    pub owner_id: Option<Uuid>,
    pub website: Option<String>,
}

/// Create assessment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVendorAssessment {
    pub assessment_type: Option<String>,
    pub risk_rating: Option<String>,
    pub findings: Option<String>,
    pub recommendations: Option<String>,
    pub next_assessment_date: Option<NaiveDate>,
}

/// List vendors query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListVendorsQuery {
    pub status: Option<String>,
    pub category: Option<String>,
    pub criticality: Option<String>,
    pub owner_id: Option<Uuid>,
    pub search: Option<String>,
    pub contract_expiring: Option<bool>,
    pub needs_assessment: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Vendor statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VendorStats {
    pub total: i64,
    pub active: i64,
    pub inactive: i64,
    pub by_criticality: Vec<CriticalityCount>,
    pub by_category: Vec<VendorCategoryCount>,
    pub contracts_expiring_soon: i64,
    pub needs_assessment: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CriticalityCount {
    pub criticality: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct VendorCategoryCount {
    pub category: Option<String>,
    pub count: i64,
}

impl Vendor {
    pub fn validate_create(input: &CreateVendor) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return Err("Vendor name is required".to_string());
        }
        if input.name.len() > 255 {
            return Err("Vendor name must be 255 characters or less".to_string());
        }

        if let Some(ref criticality) = input.criticality {
            if !["critical", "high", "medium", "low"].contains(&criticality.as_str()) {
                return Err("Invalid criticality level".to_string());
            }
        }

        if let Some(ref website) = input.website {
            if website.len() > 500 {
                return Err("Website URL must be 500 characters or less".to_string());
            }
        }

        Ok(())
    }

    pub fn contract_expiring_soon(&self, days: i64) -> bool {
        if let Some(end_date) = self.contract_end {
            let threshold = chrono::Utc::now().date_naive() + chrono::Duration::days(days);
            end_date <= threshold && end_date >= chrono::Utc::now().date_naive()
        } else {
            false
        }
    }
}
