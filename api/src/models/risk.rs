use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Risk status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskStatus {
    Identified,
    Assessed,
    Treating,
    Monitoring,
    Accepted,
    Closed,
}

impl Default for RiskStatus {
    fn default() -> Self {
        Self::Identified
    }
}

/// Risk categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskCategory {
    Strategic,
    Operational,
    Financial,
    Compliance,
    Technology,
    Security,
    Reputational,
    Other,
}

/// Risk source types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RiskSource {
    Internal,
    External,
    Regulatory,
    ThirdParty,
    Technology,
    Other,
}

/// Risk entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Risk {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub code: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    pub likelihood: Option<i32>,
    pub impact: Option<i32>,
    pub inherent_score: Option<i32>,
    pub residual_likelihood: Option<i32>,
    pub residual_impact: Option<i32>,
    pub residual_score: Option<i32>,
    pub status: String,
    pub owner_id: Option<Uuid>,
    pub treatment_plan: Option<String>,
    pub identified_at: DateTime<Utc>,
    pub review_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Risk with linked controls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskWithControls {
    #[serde(flatten)]
    pub risk: Risk,
    pub linked_control_count: i64,
    pub linked_controls: Option<Vec<LinkedControlSummary>>,
}

/// Linked control summary
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LinkedControlSummary {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub effectiveness: Option<String>,
}

/// Risk to control mapping
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskControlMapping {
    pub id: Uuid,
    pub risk_id: Uuid,
    pub control_id: Uuid,
    pub effectiveness: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Create risk request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRisk {
    pub code: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    pub likelihood: Option<i32>,
    pub impact: Option<i32>,
    pub owner_id: Option<Uuid>,
    pub treatment_plan: Option<String>,
    pub review_date: Option<NaiveDate>,
}

/// Update risk request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRisk {
    pub code: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    pub likelihood: Option<i32>,
    pub impact: Option<i32>,
    pub residual_likelihood: Option<i32>,
    pub residual_impact: Option<i32>,
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub treatment_plan: Option<String>,
    pub review_date: Option<NaiveDate>,
}

/// List risks query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListRisksQuery {
    pub status: Option<String>,
    pub category: Option<String>,
    pub source: Option<String>,
    pub owner_id: Option<Uuid>,
    pub min_score: Option<i32>,
    pub max_score: Option<i32>,
    pub search: Option<String>,
    pub needs_review: Option<bool>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Risk statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskStats {
    pub total: i64,
    pub by_status: Vec<StatusCount>,
    pub by_category: Vec<RiskCategoryCount>,
    pub high_risks: i64,
    pub medium_risks: i64,
    pub low_risks: i64,
    pub needs_review: i64,
    pub average_inherent_score: f64,
    pub average_residual_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StatusCount {
    pub status: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskCategoryCount {
    pub category: Option<String>,
    pub count: i64,
}

/// Link controls request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkControlsRequest {
    pub control_ids: Vec<Uuid>,
    pub effectiveness: Option<String>,
}

/// Risk heatmap cell data
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct HeatmapCell {
    pub likelihood: i32,
    pub impact: i32,
    pub count: i64,
}

/// Risk heatmap data for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskHeatmapData {
    pub cells: Vec<HeatmapCell>,
    pub total_risks: i64,
    pub risks_with_scores: i64,
}

impl Risk {
    pub fn validate_create(input: &CreateRisk) -> Result<(), String> {
        if input.code.trim().is_empty() {
            return Err("Risk code is required".to_string());
        }
        if input.code.len() > 50 {
            return Err("Risk code must be 50 characters or less".to_string());
        }
        if input.title.trim().is_empty() {
            return Err("Risk title is required".to_string());
        }
        if input.title.len() > 255 {
            return Err("Risk title must be 255 characters or less".to_string());
        }

        // Validate likelihood and impact (1-5 scale)
        if let Some(likelihood) = input.likelihood {
            if !(1..=5).contains(&likelihood) {
                return Err("Likelihood must be between 1 and 5".to_string());
            }
        }
        if let Some(impact) = input.impact {
            if !(1..=5).contains(&impact) {
                return Err("Impact must be between 1 and 5".to_string());
            }
        }

        if let Some(ref cat) = input.category {
            if !["strategic", "operational", "financial", "compliance", "technology", "security", "reputational", "other"]
                .contains(&cat.as_str())
            {
                return Err("Invalid risk category".to_string());
            }
        }

        if let Some(ref src) = input.source {
            if !["internal", "external", "regulatory", "third_party", "technology", "other"]
                .contains(&src.as_str())
            {
                return Err("Invalid risk source".to_string());
            }
        }

        Ok(())
    }

    /// Calculate inherent risk score (likelihood * impact)
    pub fn calculate_inherent_score(likelihood: Option<i32>, impact: Option<i32>) -> Option<i32> {
        match (likelihood, impact) {
            (Some(l), Some(i)) => Some(l * i),
            _ => None,
        }
    }

    /// Calculate residual risk score
    pub fn calculate_residual_score(
        residual_likelihood: Option<i32>,
        residual_impact: Option<i32>,
    ) -> Option<i32> {
        match (residual_likelihood, residual_impact) {
            (Some(l), Some(i)) => Some(l * i),
            _ => None,
        }
    }

    /// Get risk level based on score
    pub fn get_risk_level(score: Option<i32>) -> &'static str {
        match score {
            Some(s) if s >= 15 => "critical",
            Some(s) if s >= 10 => "high",
            Some(s) if s >= 5 => "medium",
            Some(_) => "low",
            None => "unknown",
        }
    }

    pub fn needs_review(&self) -> bool {
        if let Some(review_date) = self.review_date {
            review_date <= chrono::Utc::now().date_naive()
        } else {
            false
        }
    }
}
