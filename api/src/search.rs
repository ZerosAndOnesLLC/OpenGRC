use meilisearch_sdk::client::Client;
use meilisearch_sdk::indexes::Index;
use meilisearch_sdk::search::SearchResults;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::utils::AppResult;

const INDEX_NAME: &str = "opengrc";

/// Searchable document types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Control,
    Risk,
    Policy,
    Evidence,
    Vendor,
    Framework,
    Asset,
}

/// A searchable document stored in Meilisearch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    /// Unique ID combining type and entity ID (e.g., "control:uuid")
    pub id: String,
    /// The entity's database ID
    pub entity_id: String,
    /// Organization ID for filtering
    pub organization_id: String,
    /// Document type
    pub doc_type: DocumentType,
    /// Primary identifier (code for controls/risks/policies, name for vendors/assets)
    pub code: Option<String>,
    /// Title or name
    pub title: String,
    /// Description or content
    pub description: Option<String>,
    /// Category for faceted search
    pub category: Option<String>,
    /// Status for filtering
    pub status: Option<String>,
    /// Additional searchable tags
    pub tags: Vec<String>,
}

impl SearchDocument {
    pub fn new_control(
        entity_id: Uuid,
        org_id: Uuid,
        code: String,
        name: String,
        description: Option<String>,
        control_type: Option<String>,
        status: Option<String>,
    ) -> Self {
        Self {
            id: format!("control:{}", entity_id),
            entity_id: entity_id.to_string(),
            organization_id: org_id.to_string(),
            doc_type: DocumentType::Control,
            code: Some(code),
            title: name,
            description,
            category: control_type,
            status,
            tags: vec!["control".to_string()],
        }
    }

    pub fn new_risk(
        entity_id: Uuid,
        org_id: Uuid,
        code: String,
        title: String,
        description: Option<String>,
        category: Option<String>,
        status: Option<String>,
    ) -> Self {
        Self {
            id: format!("risk:{}", entity_id),
            entity_id: entity_id.to_string(),
            organization_id: org_id.to_string(),
            doc_type: DocumentType::Risk,
            code: Some(code),
            title,
            description,
            category,
            status,
            tags: vec!["risk".to_string()],
        }
    }

    pub fn new_policy(
        entity_id: Uuid,
        org_id: Uuid,
        code: String,
        title: String,
        content: Option<String>,
        category: Option<String>,
        status: Option<String>,
    ) -> Self {
        Self {
            id: format!("policy:{}", entity_id),
            entity_id: entity_id.to_string(),
            organization_id: org_id.to_string(),
            doc_type: DocumentType::Policy,
            code: Some(code),
            title,
            description: content,
            category,
            status,
            tags: vec!["policy".to_string()],
        }
    }

    pub fn new_evidence(
        entity_id: Uuid,
        org_id: Uuid,
        title: String,
        description: Option<String>,
        evidence_type: Option<String>,
    ) -> Self {
        Self {
            id: format!("evidence:{}", entity_id),
            entity_id: entity_id.to_string(),
            organization_id: org_id.to_string(),
            doc_type: DocumentType::Evidence,
            code: None,
            title,
            description,
            category: evidence_type,
            status: None,
            tags: vec!["evidence".to_string()],
        }
    }

    pub fn new_vendor(
        entity_id: Uuid,
        org_id: Uuid,
        name: String,
        description: Option<String>,
        category: Option<String>,
        status: Option<String>,
    ) -> Self {
        Self {
            id: format!("vendor:{}", entity_id),
            entity_id: entity_id.to_string(),
            organization_id: org_id.to_string(),
            doc_type: DocumentType::Vendor,
            code: None,
            title: name,
            description,
            category,
            status,
            tags: vec!["vendor".to_string()],
        }
    }

    pub fn new_framework(
        entity_id: Uuid,
        name: String,
        description: Option<String>,
        category: Option<String>,
    ) -> Self {
        Self {
            id: format!("framework:{}", entity_id),
            entity_id: entity_id.to_string(),
            organization_id: "global".to_string(), // Frameworks are global
            doc_type: DocumentType::Framework,
            code: None,
            title: name,
            description,
            category,
            status: None,
            tags: vec!["framework".to_string()],
        }
    }

    pub fn new_asset(
        entity_id: Uuid,
        org_id: Uuid,
        name: String,
        description: Option<String>,
        asset_type: Option<String>,
        status: Option<String>,
    ) -> Self {
        Self {
            id: format!("asset:{}", entity_id),
            entity_id: entity_id.to_string(),
            organization_id: org_id.to_string(),
            doc_type: DocumentType::Asset,
            code: None,
            title: name,
            description,
            category: asset_type,
            status,
            tags: vec!["asset".to_string()],
        }
    }
}

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub entity_id: String,
    pub doc_type: String,
    pub code: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub status: Option<String>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub hits: Vec<SearchResult>,
    pub total_hits: usize,
    pub processing_time_ms: usize,
}

/// Search client wrapper
#[derive(Clone)]
pub struct SearchClient {
    client: Arc<Client>,
    enabled: bool,
}

impl SearchClient {
    pub fn new(host: &str, api_key: Option<&str>) -> Self {
        let client = if let Some(key) = api_key {
            Client::new(host, Some(key)).expect("Failed to create Meilisearch client")
        } else {
            Client::new(host, None::<&str>).expect("Failed to create Meilisearch client")
        };

        Self {
            client: Arc::new(client),
            enabled: true,
        }
    }

    pub fn disabled() -> Self {
        // Create a dummy client that won't be used
        let client = Client::new("http://localhost:7700", None::<&str>)
            .expect("Failed to create Meilisearch client");
        Self {
            client: Arc::new(client),
            enabled: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn index(&self) -> Index {
        self.client.index(INDEX_NAME)
    }

    /// Initialize the search index with proper settings
    pub async fn init_index(&self) -> AppResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let index = self.index();

        // Set searchable attributes
        let searchable = ["title", "code", "description", "tags", "category"];
        index.set_searchable_attributes(&searchable).await?;

        // Set filterable attributes
        let filterable = ["organization_id", "doc_type", "category", "status"];
        index.set_filterable_attributes(&filterable).await?;

        // Set sortable attributes
        let sortable = ["title", "code"];
        index.set_sortable_attributes(&sortable).await?;

        // Set displayed attributes
        let displayed = [
            "id",
            "entity_id",
            "doc_type",
            "code",
            "title",
            "description",
            "category",
            "status",
        ];
        index.set_displayed_attributes(&displayed).await?;

        Ok(())
    }

    /// Index a single document
    pub async fn index_document(&self, doc: SearchDocument) -> AppResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let index = self.index();
        index.add_documents(&[doc], Some("id")).await?;
        Ok(())
    }

    /// Index multiple documents
    pub async fn index_documents(&self, docs: Vec<SearchDocument>) -> AppResult<()> {
        if !self.enabled || docs.is_empty() {
            return Ok(());
        }

        let index = self.index();
        index.add_documents(&docs, Some("id")).await?;
        Ok(())
    }

    /// Delete a document by its compound ID (e.g., "control:uuid")
    pub async fn delete_document(&self, doc_id: &str) -> AppResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let index = self.index();
        index.delete_document(doc_id).await?;
        Ok(())
    }

    /// Delete all documents for an organization
    pub async fn delete_org_documents(&self, org_id: Uuid) -> AppResult<()> {
        if !self.enabled {
            return Ok(());
        }

        let index = self.index();
        let filter = format!("organization_id = \"{}\"", org_id);
        let mut query = meilisearch_sdk::documents::DocumentDeletionQuery::new(&index);
        query.with_filter(&filter);
        index.delete_documents_with(&query).await?;
        Ok(())
    }

    /// Search documents
    pub async fn search(
        &self,
        query: &str,
        org_id: Option<Uuid>,
        doc_types: Option<Vec<String>>,
        limit: usize,
    ) -> AppResult<SearchResponse> {
        if !self.enabled {
            return Ok(SearchResponse {
                hits: vec![],
                total_hits: 0,
                processing_time_ms: 0,
            });
        }

        let index = self.index();

        // Build filter
        let mut filters = vec![];

        if let Some(org) = org_id {
            // Include org-specific and global (frameworks) documents
            filters.push(format!(
                "(organization_id = \"{}\" OR organization_id = \"global\")",
                org
            ));
        }

        if let Some(types) = doc_types {
            if !types.is_empty() {
                let type_filter: Vec<String> =
                    types.iter().map(|t| format!("doc_type = \"{}\"", t)).collect();
                filters.push(format!("({})", type_filter.join(" OR ")));
            }
        }

        let filter = if filters.is_empty() {
            None
        } else {
            Some(filters.join(" AND "))
        };

        let mut search_query = index.search();
        search_query.with_query(query);
        search_query.with_limit(limit);

        if let Some(f) = &filter {
            search_query.with_filter(f);
        }

        let results: SearchResults<SearchDocument> = search_query.execute().await?;

        let hits: Vec<SearchResult> = results
            .hits
            .into_iter()
            .map(|hit| {
                let doc = hit.result;
                SearchResult {
                    id: doc.id,
                    entity_id: doc.entity_id,
                    doc_type: format!("{:?}", doc.doc_type).to_lowercase(),
                    code: doc.code,
                    title: doc.title,
                    description: doc.description,
                    category: doc.category,
                    status: doc.status,
                }
            })
            .collect();

        Ok(SearchResponse {
            total_hits: results.estimated_total_hits.unwrap_or(hits.len()),
            hits,
            processing_time_ms: results.processing_time_ms,
        })
    }
}
