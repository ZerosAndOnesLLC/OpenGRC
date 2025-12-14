use crate::integrations::jira::client::JiraClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;

/// Project Collector for Jira
pub struct ProjectCollector;

impl ProjectCollector {
    /// Collect project data from Jira
    pub async fn sync(client: &JiraClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get all projects
        let projects = client.list_projects().await?;
        result.records_processed = projects.len() as i32;

        if projects.is_empty() {
            return Ok(result);
        }

        // Analyze projects
        let archived_projects: Vec<_> = projects.iter().filter(|p| p.archived.unwrap_or(false)).collect();
        let private_projects: Vec<_> = projects.iter().filter(|p| p.is_private.unwrap_or(false)).collect();

        // Group by project type
        let mut project_types: std::collections::HashMap<String, i32> = std::collections::HashMap::new();
        for project in &projects {
            *project_types.entry(project.project_type_key.clone()).or_insert(0) += 1;
        }

        // Generate project inventory evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Jira Project Inventory".to_string(),
            description: Some(format!(
                "Inventory of {} Jira projects ({} active, {} archived)",
                projects.len(),
                projects.len() - archived_projects.len(),
                archived_projects.len()
            )),
            evidence_type: "automated".to_string(),
            source: "jira".to_string(),
            source_reference: Some("jira:projects".to_string()),
            data: json!({
                "total_projects": projects.len(),
                "active_count": projects.len() - archived_projects.len(),
                "archived_count": archived_projects.len(),
                "private_count": private_projects.len(),
                "by_type": project_types,
                "projects": projects.iter().map(|p| json!({
                    "id": p.id,
                    "key": p.key,
                    "name": p.name,
                    "description": p.description,
                    "project_type": p.project_type_key,
                    "lead": p.lead.as_ref().map(|l| &l.display_name),
                    "is_private": p.is_private,
                    "archived": p.archived,
                })).collect::<Vec<_>>(),
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec!["CC6.1".to_string(), "CC6.7".to_string(), "A1.1".to_string()],
        });

        result.records_created = result.records_processed;
        Ok(result)
    }
}
