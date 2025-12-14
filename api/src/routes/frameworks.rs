use axum::{
    extract::{Multipart, Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::models::{
    CreateFramework, CreateFrameworkRequirement, Framework, FrameworkRequirement,
    FrameworkWithRequirements, UpdateFramework, UpdateFrameworkRequirement,
    FrameworkGapAnalysis,
};
use crate::models::framework::build_requirement_tree;
use crate::services::AppServices;
use crate::utils::{AppError, AppResult};

#[derive(Debug, Deserialize)]
pub struct ListFrameworksQuery {
    pub category: Option<String>,
    pub is_system: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ListRequirementsQuery {
    pub tree: Option<bool>,
}

// ==================== Framework Routes ====================

/// GET /api/v1/frameworks
pub async fn list_frameworks(
    State(services): State<Arc<AppServices>>,
    Query(query): Query<ListFrameworksQuery>,
) -> AppResult<Json<Vec<Framework>>> {
    let frameworks = services
        .framework
        .list_frameworks(query.category.as_deref(), query.is_system)
        .await?;

    Ok(Json(frameworks))
}

/// GET /api/v1/frameworks/:id
pub async fn get_framework(
    State(services): State<Arc<AppServices>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<FrameworkWithRequirements>> {
    let framework = services.framework.get_framework_with_requirements(id).await?;
    Ok(Json(framework))
}

/// POST /api/v1/frameworks
pub async fn create_framework(
    State(services): State<Arc<AppServices>>,
    Json(input): Json<CreateFramework>,
) -> AppResult<Json<Framework>> {
    let framework = services.framework.create_framework(input).await?;
    Ok(Json(framework))
}

/// PUT /api/v1/frameworks/:id
pub async fn update_framework(
    State(services): State<Arc<AppServices>>,
    Path(id): Path<Uuid>,
    Json(input): Json<UpdateFramework>,
) -> AppResult<Json<Framework>> {
    let framework = services.framework.update_framework(id, input).await?;
    Ok(Json(framework))
}

/// DELETE /api/v1/frameworks/:id
pub async fn delete_framework(
    State(services): State<Arc<AppServices>>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    services.framework.delete_framework(id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ==================== Requirement Routes ====================

/// GET /api/v1/frameworks/:framework_id/requirements
pub async fn list_requirements(
    State(services): State<Arc<AppServices>>,
    Path(framework_id): Path<Uuid>,
    Query(query): Query<ListRequirementsQuery>,
) -> AppResult<Json<serde_json::Value>> {
    let requirements = services.framework.list_requirements(framework_id).await?;

    // Return tree structure if requested
    if query.tree.unwrap_or(false) {
        let tree = build_requirement_tree(requirements);
        Ok(Json(serde_json::json!({ "tree": tree })))
    } else {
        Ok(Json(serde_json::json!({ "requirements": requirements })))
    }
}

/// GET /api/v1/frameworks/:framework_id/requirements/:id
pub async fn get_requirement(
    State(services): State<Arc<AppServices>>,
    Path((_framework_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<FrameworkRequirement>> {
    let requirement = services.framework.get_requirement(id).await?;
    Ok(Json(requirement))
}

/// POST /api/v1/frameworks/:framework_id/requirements
pub async fn create_requirement(
    State(services): State<Arc<AppServices>>,
    Path(framework_id): Path<Uuid>,
    Json(input): Json<CreateFrameworkRequirement>,
) -> AppResult<Json<FrameworkRequirement>> {
    let requirement = services
        .framework
        .create_requirement(framework_id, input)
        .await?;
    Ok(Json(requirement))
}

/// POST /api/v1/frameworks/:framework_id/requirements/batch
pub async fn batch_create_requirements(
    State(services): State<Arc<AppServices>>,
    Path(framework_id): Path<Uuid>,
    Json(input): Json<Vec<CreateFrameworkRequirement>>,
) -> AppResult<Json<Vec<FrameworkRequirement>>> {
    let requirements = services
        .framework
        .batch_create_requirements(framework_id, input)
        .await?;
    Ok(Json(requirements))
}

/// POST /api/v1/frameworks/:framework_id/requirements/import
/// Import requirements from CSV or JSON file
pub async fn import_requirements(
    State(services): State<Arc<AppServices>>,
    Path(framework_id): Path<Uuid>,
    mut multipart: Multipart,
) -> AppResult<Json<ImportResult>> {
    let mut file_content: Option<String> = None;
    let mut file_format: Option<String> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        AppError::BadRequest(format!("Failed to read multipart field: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();

        if name == "file" {
            let content_type = field.content_type().map(|s| s.to_string());
            let filename = field.file_name().map(|s| s.to_lowercase());

            // Determine format from content type or filename
            file_format = if content_type.as_deref() == Some("application/json") {
                Some("json".to_string())
            } else if content_type.as_deref() == Some("text/csv") {
                Some("csv".to_string())
            } else if let Some(ref f) = filename {
                if f.ends_with(".json") {
                    Some("json".to_string())
                } else if f.ends_with(".csv") {
                    Some("csv".to_string())
                } else {
                    None
                }
            } else {
                None
            };

            let bytes = field.bytes().await.map_err(|e| {
                AppError::BadRequest(format!("Failed to read file: {}", e))
            })?;
            file_content = Some(String::from_utf8_lossy(&bytes).to_string());
        }
    }

    let content = file_content.ok_or_else(|| {
        AppError::BadRequest("No file uploaded".to_string())
    })?;
    let format = file_format.ok_or_else(|| {
        AppError::BadRequest("Could not determine file format. Use .csv or .json extension".to_string())
    })?;

    let requirements = match format.as_str() {
        "json" => parse_json_requirements(&content)?,
        "csv" => parse_csv_requirements(&content)?,
        _ => return Err(AppError::BadRequest("Unsupported format".to_string())),
    };

    if requirements.is_empty() {
        return Err(AppError::BadRequest("No requirements found in file".to_string()));
    }

    let created = services
        .framework
        .batch_create_requirements(framework_id, requirements)
        .await?;

    Ok(Json(ImportResult {
        imported: created.len(),
        requirements: created,
    }))
}

#[derive(Debug, serde::Serialize)]
pub struct ImportResult {
    pub imported: usize,
    pub requirements: Vec<FrameworkRequirement>,
}

fn parse_json_requirements(content: &str) -> AppResult<Vec<CreateFrameworkRequirement>> {
    // Support both array format and object with requirements key
    if let Ok(reqs) = serde_json::from_str::<Vec<CreateFrameworkRequirement>>(content) {
        return Ok(reqs);
    }

    #[derive(Deserialize)]
    struct WrappedReqs {
        requirements: Vec<CreateFrameworkRequirement>,
    }

    if let Ok(wrapped) = serde_json::from_str::<WrappedReqs>(content) {
        return Ok(wrapped.requirements);
    }

    Err(AppError::BadRequest(
        "Invalid JSON format. Expected array of requirements or { \"requirements\": [...] }".to_string()
    ))
}

fn parse_csv_requirements(content: &str) -> AppResult<Vec<CreateFrameworkRequirement>> {
    let mut requirements = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Ok(requirements);
    }

    // Parse header row to find column indices
    let header = lines[0].to_lowercase();
    let headers: Vec<&str> = header.split(',').map(|s| s.trim()).collect();

    let code_idx = headers.iter().position(|&h| h == "code");
    let name_idx = headers.iter().position(|&h| h == "name");
    let desc_idx = headers.iter().position(|&h| h == "description" || h == "desc");
    let cat_idx = headers.iter().position(|&h| h == "category" || h == "cat");
    let parent_idx = headers.iter().position(|&h| h == "parent_code" || h == "parent");
    let sort_idx = headers.iter().position(|&h| h == "sort_order" || h == "sort" || h == "order");

    let code_idx = code_idx.ok_or_else(|| {
        AppError::BadRequest("CSV must have a 'code' column".to_string())
    })?;
    let name_idx = name_idx.ok_or_else(|| {
        AppError::BadRequest("CSV must have a 'name' column".to_string())
    })?;

    // Track codes for parent reference resolution (will be done in a second pass or by service)
    for (line_num, line) in lines.iter().enumerate().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        let fields = parse_csv_line(line);
        if fields.len() <= code_idx || fields.len() <= name_idx {
            continue;
        }

        let code = fields.get(code_idx).map(|s| s.trim().to_string()).unwrap_or_default();
        let name = fields.get(name_idx).map(|s| s.trim().to_string()).unwrap_or_default();

        if code.is_empty() || name.is_empty() {
            continue;
        }

        let description = desc_idx.and_then(|i| fields.get(i).map(|s| {
            let s = s.trim();
            if s.is_empty() { None } else { Some(s.to_string()) }
        })).flatten();

        let category = cat_idx.and_then(|i| fields.get(i).map(|s| {
            let s = s.trim();
            if s.is_empty() { None } else { Some(s.to_string()) }
        })).flatten();

        let sort_order = sort_idx
            .and_then(|i| fields.get(i))
            .and_then(|s| s.trim().parse::<i32>().ok())
            .or_else(|| Some((line_num - 1) as i32));

        // Note: parent_id is not set here since we don't have UUIDs yet
        // The service layer would need to resolve parent codes to IDs
        let _ = parent_idx; // TODO: Implement parent code resolution

        requirements.push(CreateFrameworkRequirement {
            code,
            name,
            description,
            category,
            parent_id: None,
            sort_order,
        });
    }

    Ok(requirements)
}

fn parse_csv_line(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in line.chars() {
        match ch {
            '"' => in_quotes = !in_quotes,
            ',' if !in_quotes => {
                fields.push(current.clone());
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    fields.push(current);
    fields
}

/// PUT /api/v1/frameworks/:framework_id/requirements/:id
pub async fn update_requirement(
    State(services): State<Arc<AppServices>>,
    Path((_framework_id, id)): Path<(Uuid, Uuid)>,
    Json(input): Json<UpdateFrameworkRequirement>,
) -> AppResult<Json<FrameworkRequirement>> {
    let requirement = services.framework.update_requirement(id, input).await?;
    Ok(Json(requirement))
}

/// DELETE /api/v1/frameworks/:framework_id/requirements/:id
pub async fn delete_requirement(
    State(services): State<Arc<AppServices>>,
    Path((_framework_id, id)): Path<(Uuid, Uuid)>,
) -> AppResult<Json<serde_json::Value>> {
    services.framework.delete_requirement(id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

// ==================== Gap Analysis ====================

/// GET /api/v1/frameworks/:framework_id/gap-analysis
pub async fn get_gap_analysis(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Path(framework_id): Path<Uuid>,
) -> AppResult<Json<FrameworkGapAnalysis>> {
    let org_id = user
        .organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| AppError::BadRequest("User not associated with an organization".to_string()))?;

    let analysis = services.framework.get_gap_analysis(org_id, framework_id).await?;
    Ok(Json(analysis))
}
