use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::middleware::AuthUser;
use crate::services::AppServices;
use crate::utils::AppResult;

fn get_org_id(user: &AuthUser) -> AppResult<Uuid> {
    user.organization_id
        .as_ref()
        .and_then(|id| Uuid::parse_str(id).ok())
        .ok_or_else(|| crate::utils::AppError::BadRequest("User not associated with an organization".to_string()))
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    /// Search query string
    pub q: String,
    /// Filter by document types (comma-separated: control,risk,policy,evidence,vendor,framework,asset)
    pub types: Option<String>,
    /// Maximum number of results (default 20, max 100)
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SearchApiResponse {
    pub results: Vec<SearchResultItem>,
    pub total: usize,
    pub query: String,
    pub processing_time_ms: usize,
}

#[derive(Debug, Serialize)]
pub struct SearchResultItem {
    pub id: String,
    pub entity_id: String,
    #[serde(rename = "type")]
    pub doc_type: String,
    pub code: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
    /// URL path to navigate to
    pub path: String,
}

impl From<crate::search::SearchResult> for SearchResultItem {
    fn from(result: crate::search::SearchResult) -> Self {
        let path = match result.doc_type.as_str() {
            "control" => format!("/controls?id={}", result.entity_id),
            "risk" => format!("/risks?id={}", result.entity_id),
            "policy" => format!("/policies?id={}", result.entity_id),
            "evidence" => format!("/evidence?id={}", result.entity_id),
            "vendor" => format!("/vendors?id={}", result.entity_id),
            "framework" => format!("/frameworks?id={}", result.entity_id),
            "asset" => format!("/assets?id={}", result.entity_id),
            _ => "/".to_string(),
        };

        SearchResultItem {
            id: result.id,
            entity_id: result.entity_id,
            doc_type: result.doc_type,
            code: result.code,
            title: result.title,
            description: result.description,
            category: result.category,
            status: result.status,
            path,
        }
    }
}

/// GET /api/v1/search
/// Search across all entity types
pub async fn search(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    let org_id = match get_org_id(&user) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    if query.q.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Search query cannot be empty" })),
        )
            .into_response();
    }

    let limit = query.limit.unwrap_or(20).min(100);
    let doc_types = query.types.map(|t| {
        t.split(',')
            .map(|s| s.trim().to_lowercase())
            .filter(|s| !s.is_empty())
            .collect()
    });

    match services
        .search
        .search(&query.q, Some(org_id), doc_types, limit)
        .await
    {
        Ok(results) => {
            let response = SearchApiResponse {
                results: results.hits.into_iter().map(SearchResultItem::from).collect(),
                total: results.total_hits,
                query: query.q,
                processing_time_ms: results.processing_time_ms,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            tracing::error!("Search error: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Search failed" })),
            )
                .into_response()
        }
    }
}

/// GET /api/v1/search/status
/// Check if search is enabled
pub async fn search_status(State(services): State<Arc<AppServices>>) -> impl IntoResponse {
    let enabled = services.search.is_enabled();
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "enabled": enabled,
            "engine": "meilisearch"
        })),
    )
}

/// POST /api/v1/search/reindex
/// Trigger a full reindex of all documents (admin only)
pub async fn reindex_all(
    State(services): State<Arc<AppServices>>,
    Extension(user): Extension<AuthUser>,
) -> impl IntoResponse {
    // Check if search is enabled
    if !services.search.is_enabled() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Search is not enabled" })),
        )
            .into_response();
    }

    let org_id = match get_org_id(&user) {
        Ok(id) => id,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    };

    // Reindex in background
    let services_clone = services.clone();
    tokio::spawn(async move {
        if let Err(e) = reindex_organization(&services_clone, org_id).await {
            tracing::error!("Reindex failed for org {}: {}", org_id, e);
        }
    });

    (
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "message": "Reindex started",
            "organization_id": org_id.to_string()
        })),
    )
        .into_response()
}

async fn reindex_organization(services: &AppServices, org_id: Uuid) -> AppResult<()> {
    use crate::search::SearchDocument;

    tracing::info!("Starting reindex for organization {}", org_id);

    let mut documents = vec![];

    // Index controls
    let controls: Vec<(Uuid, String, String, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT id, code, name, description, control_type, status FROM controls WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_all(&services.db)
        .await?;

    for (id, code, name, desc, ctrl_type, status) in controls {
        documents.push(SearchDocument::new_control(id, org_id, code, name, desc, ctrl_type, status));
    }

    // Index risks
    let risks: Vec<(Uuid, String, String, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT id, code, title, description, category, status FROM risks WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_all(&services.db)
        .await?;

    for (id, code, title, desc, category, status) in risks {
        documents.push(SearchDocument::new_risk(id, org_id, code, title, desc, category, status));
    }

    // Index policies
    let policies: Vec<(Uuid, String, String, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT id, code, title, content, category, status FROM policies WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_all(&services.db)
        .await?;

    for (id, code, title, content, category, status) in policies {
        documents.push(SearchDocument::new_policy(id, org_id, code, title, content, category, status));
    }

    // Index evidence
    let evidence: Vec<(Uuid, String, Option<String>, Option<String>)> = sqlx::query_as(
        "SELECT id, title, description, evidence_type FROM evidence WHERE organization_id = $1",
    )
    .bind(org_id)
    .fetch_all(&services.db)
    .await?;

    for (id, title, desc, evidence_type) in evidence {
        documents.push(SearchDocument::new_evidence(id, org_id, title, desc, evidence_type));
    }

    // Index vendors
    let vendors: Vec<(Uuid, String, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT id, name, description, category, status FROM vendors WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_all(&services.db)
        .await?;

    for (id, name, desc, category, status) in vendors {
        documents.push(SearchDocument::new_vendor(id, org_id, name, desc, category, status));
    }

    // Index assets
    let assets: Vec<(Uuid, String, Option<String>, Option<String>, Option<String>)> =
        sqlx::query_as(
            "SELECT id, name, description, asset_type, status FROM assets WHERE organization_id = $1",
        )
        .bind(org_id)
        .fetch_all(&services.db)
        .await?;

    for (id, name, desc, asset_type, status) in assets {
        documents.push(SearchDocument::new_asset(id, org_id, name, desc, asset_type, status));
    }

    // Batch index all documents
    services.search.index_documents(documents).await?;

    tracing::info!("Reindex completed for organization {}", org_id);
    Ok(())
}
