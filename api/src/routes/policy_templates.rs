use axum::{
    extract::{Path, Query},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::templates::policies::{get_template, list_templates, search_templates, PolicyTemplate};
use crate::utils::{AppError, AppResult};

#[derive(Debug, Serialize)]
pub struct PolicyTemplateResponse {
    pub id: String,
    pub code: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub frameworks: Vec<String>,
    pub review_frequency: String,
    pub related_templates: Vec<String>,
    pub suggested_controls: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PolicyTemplateDetailResponse {
    #[serde(flatten)]
    pub template: PolicyTemplateResponse,
    pub content: String,
}

impl From<&PolicyTemplate> for PolicyTemplateResponse {
    fn from(t: &PolicyTemplate) -> Self {
        Self {
            id: t.id.to_string(),
            code: t.code.to_string(),
            title: t.title.to_string(),
            description: t.description.to_string(),
            category: t.category.to_string(),
            frameworks: t.frameworks.iter().map(|f| f.to_string()).collect(),
            review_frequency: t.review_frequency.to_string(),
            related_templates: t.related_templates.iter().map(|r| r.to_string()).collect(),
            suggested_controls: t.suggested_controls.iter().map(|c| c.to_string()).collect(),
        }
    }
}

impl From<&PolicyTemplate> for PolicyTemplateDetailResponse {
    fn from(t: &PolicyTemplate) -> Self {
        Self {
            template: PolicyTemplateResponse::from(t),
            content: t.content.to_string(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub category: Option<String>,
    pub framework: Option<String>,
    pub q: Option<String>,
}

/// List all policy templates (without content for efficiency)
pub async fn list_policy_templates() -> AppResult<Json<Vec<PolicyTemplateResponse>>> {
    let templates: Vec<PolicyTemplateResponse> = list_templates()
        .iter()
        .map(PolicyTemplateResponse::from)
        .collect();
    Ok(Json(templates))
}

/// Get a specific policy template by ID (includes full content)
pub async fn get_policy_template(
    Path(id): Path<String>,
) -> AppResult<Json<PolicyTemplateDetailResponse>> {
    let template = get_template(&id)
        .ok_or_else(|| AppError::NotFound(format!("Policy template '{}' not found", id)))?;
    Ok(Json(PolicyTemplateDetailResponse::from(template)))
}

/// Search policy templates by category, framework, or text query
pub async fn search_policy_templates(
    Query(query): Query<SearchQuery>,
) -> AppResult<Json<Vec<PolicyTemplateResponse>>> {
    let templates = search_templates(
        query.category.as_deref(),
        query.framework.as_deref(),
        query.q.as_deref(),
    );
    let results: Vec<PolicyTemplateResponse> = templates
        .iter()
        .map(|t| PolicyTemplateResponse::from(*t))
        .collect();
    Ok(Json(results))
}

/// Get available categories
pub async fn get_template_categories() -> AppResult<Json<Vec<String>>> {
    let categories = vec![
        "security".to_string(),
        "it".to_string(),
        "compliance".to_string(),
        "privacy".to_string(),
        "hr".to_string(),
    ];
    Ok(Json(categories))
}

/// Get available frameworks
pub async fn get_template_frameworks() -> AppResult<Json<Vec<FrameworkInfo>>> {
    let frameworks = vec![
        FrameworkInfo { id: "soc2".to_string(), name: "SOC 2".to_string() },
        FrameworkInfo { id: "iso27001".to_string(), name: "ISO 27001".to_string() },
        FrameworkInfo { id: "hipaa".to_string(), name: "HIPAA".to_string() },
        FrameworkInfo { id: "pci-dss".to_string(), name: "PCI DSS".to_string() },
        FrameworkInfo { id: "gdpr".to_string(), name: "GDPR".to_string() },
        FrameworkInfo { id: "ccpa".to_string(), name: "CCPA".to_string() },
    ];
    Ok(Json(frameworks))
}

#[derive(Debug, Serialize)]
pub struct FrameworkInfo {
    pub id: String,
    pub name: String,
}
