use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Asset types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Hardware,
    Software,
    Data,
    Network,
    Cloud,
    Physical,
    People,
    Other,
}

/// Asset classification levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssetClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

impl Default for AssetClassification {
    fn default() -> Self {
        Self::Internal
    }
}

/// Asset status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AssetStatus {
    Active,
    Inactive,
    Decommissioned,
    UnderReview,
    Retired,
}

impl Default for AssetStatus {
    fn default() -> Self {
        Self::Active
    }
}

/// Asset entity
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Asset {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub asset_type: Option<String>,
    pub category: Option<String>,
    pub classification: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub custodian_id: Option<Uuid>,
    pub location: Option<String>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub warranty_until: Option<NaiveDate>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Asset with control mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetWithControls {
    #[serde(flatten)]
    pub asset: Asset,
    pub linked_control_count: i64,
}

/// Asset to control mapping
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetControlMapping {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub control_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Create asset request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAsset {
    pub name: String,
    pub description: Option<String>,
    pub asset_type: Option<String>,
    pub category: Option<String>,
    pub classification: Option<String>,
    pub owner_id: Option<Uuid>,
    pub custodian_id: Option<Uuid>,
    pub location: Option<String>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub warranty_until: Option<NaiveDate>,
    pub metadata: Option<serde_json::Value>,
}

/// Update asset request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAsset {
    pub name: Option<String>,
    pub description: Option<String>,
    pub asset_type: Option<String>,
    pub category: Option<String>,
    pub classification: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub custodian_id: Option<Uuid>,
    pub location: Option<String>,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub purchase_date: Option<NaiveDate>,
    pub warranty_until: Option<NaiveDate>,
    pub metadata: Option<serde_json::Value>,
}

/// List assets query
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListAssetsQuery {
    pub asset_type: Option<String>,
    pub category: Option<String>,
    pub classification: Option<String>,
    pub status: Option<String>,
    pub owner_id: Option<Uuid>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Asset statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetStats {
    pub total: i64,
    pub by_type: Vec<AssetTypeCount>,
    pub by_classification: Vec<ClassificationCount>,
    pub by_status: Vec<AssetStatusCount>,
    pub warranty_expiring_soon: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetTypeCount {
    pub asset_type: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ClassificationCount {
    pub classification: Option<String>,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AssetStatusCount {
    pub status: Option<String>,
    pub count: i64,
}

impl Asset {
    pub fn validate_create(input: &CreateAsset) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return Err("Asset name is required".to_string());
        }
        if input.name.len() > 255 {
            return Err("Asset name must be 255 characters or less".to_string());
        }

        if let Some(ref asset_type) = input.asset_type {
            if !["hardware", "software", "data", "network", "cloud", "physical", "people", "other"]
                .contains(&asset_type.as_str())
            {
                return Err("Invalid asset type".to_string());
            }
        }

        if let Some(ref classification) = input.classification {
            if !["public", "internal", "confidential", "restricted"].contains(&classification.as_str())
            {
                return Err("Invalid classification level".to_string());
            }
        }

        if let Some(ref ip) = input.ip_address {
            if ip.len() > 45 {
                return Err("IP address must be 45 characters or less".to_string());
            }
        }

        if let Some(ref mac) = input.mac_address {
            if mac.len() > 17 {
                return Err("MAC address must be 17 characters or less".to_string());
            }
        }

        Ok(())
    }

    pub fn warranty_expiring_soon(&self, days: i64) -> bool {
        if let Some(warranty_date) = self.warranty_until {
            let threshold = chrono::Utc::now().date_naive() + chrono::Duration::days(days);
            warranty_date <= threshold && warranty_date >= chrono::Utc::now().date_naive()
        } else {
            false
        }
    }
}
