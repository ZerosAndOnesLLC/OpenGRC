use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Framework {
    pub id: Uuid,
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FrameworkRequirement {
    pub id: Uuid,
    pub framework_id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkWithRequirements {
    #[serde(flatten)]
    pub framework: Framework,
    pub requirements: Vec<FrameworkRequirement>,
    pub requirement_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFramework {
    pub name: String,
    pub version: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub is_system: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFramework {
    pub name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFrameworkRequirement {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFrameworkRequirement {
    pub code: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub category: Option<String>,
    pub parent_id: Option<Uuid>,
    pub sort_order: Option<i32>,
}

/// Nested requirement tree structure for hierarchical display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementTree {
    #[serde(flatten)]
    pub requirement: FrameworkRequirement,
    pub children: Vec<RequirementTree>,
}

/// Gap analysis result for a single requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequirementGapAnalysis {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub category: Option<String>,
    pub control_count: i64,
    pub is_covered: bool,
}

/// Gap analysis summary for a framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkGapAnalysis {
    pub framework_id: Uuid,
    pub framework_name: String,
    pub total_requirements: i64,
    pub covered_requirements: i64,
    pub uncovered_requirements: i64,
    pub coverage_percentage: f64,
    pub by_category: Vec<CategoryGapAnalysis>,
    pub requirements: Vec<RequirementGapAnalysis>,
}

/// Gap analysis breakdown by category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryGapAnalysis {
    pub category: Option<String>,
    pub total: i64,
    pub covered: i64,
    pub coverage_percentage: f64,
}

impl Framework {
    pub fn validate_create(input: &CreateFramework) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return Err("Framework name is required".to_string());
        }
        if input.name.len() > 255 {
            return Err("Framework name must be 255 characters or less".to_string());
        }
        Ok(())
    }
}

impl FrameworkRequirement {
    pub fn validate_create(input: &CreateFrameworkRequirement) -> Result<(), String> {
        if input.code.trim().is_empty() {
            return Err("Requirement code is required".to_string());
        }
        if input.code.len() > 50 {
            return Err("Requirement code must be 50 characters or less".to_string());
        }
        if input.name.trim().is_empty() {
            return Err("Requirement name is required".to_string());
        }
        if input.name.len() > 255 {
            return Err("Requirement name must be 255 characters or less".to_string());
        }
        Ok(())
    }
}

/// Build a tree structure from flat list of requirements
pub fn build_requirement_tree(requirements: Vec<FrameworkRequirement>) -> Vec<RequirementTree> {
    use std::collections::HashMap;

    // Index by id for fast lookups
    let mut by_id: HashMap<Uuid, FrameworkRequirement> = HashMap::new();
    let mut children_map: HashMap<Option<Uuid>, Vec<Uuid>> = HashMap::new();

    for req in requirements {
        children_map.entry(req.parent_id).or_default().push(req.id);
        by_id.insert(req.id, req);
    }

    fn build_tree(
        parent_id: Option<Uuid>,
        children_map: &HashMap<Option<Uuid>, Vec<Uuid>>,
        by_id: &HashMap<Uuid, FrameworkRequirement>,
    ) -> Vec<RequirementTree> {
        children_map
            .get(&parent_id)
            .map(|child_ids| {
                let mut nodes: Vec<RequirementTree> = child_ids
                    .iter()
                    .filter_map(|id| by_id.get(id))
                    .map(|req| RequirementTree {
                        requirement: req.clone(),
                        children: build_tree(Some(req.id), children_map, by_id),
                    })
                    .collect();
                nodes.sort_by_key(|n| n.requirement.sort_order);
                nodes
            })
            .unwrap_or_default()
    }

    build_tree(None, &children_map, &by_id)
}
