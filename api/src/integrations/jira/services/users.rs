use crate::integrations::jira::client::{JiraClient, JiraProject};
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;
use std::collections::{HashMap, HashSet};

/// User Collector for Jira
pub struct UserCollector;

impl UserCollector {
    /// Collect user data from Jira
    pub async fn sync(
        client: &JiraClient,
        projects: &[JiraProject],
        _context: &SyncContext,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Collect unique users from all projects
        let mut all_users: HashMap<String, serde_json::Value> = HashMap::new();
        let mut user_projects: HashMap<String, HashSet<String>> = HashMap::new();
        let mut inactive_users: HashSet<String> = HashSet::new();

        for project in projects {
            if project.archived.unwrap_or(false) {
                continue;
            }

            match client.list_project_users(&project.key).await {
                Ok(users) => {
                    for user in users {
                        result.records_processed += 1;

                        // Track which projects this user has access to
                        user_projects
                            .entry(user.account_id.clone())
                            .or_default()
                            .insert(project.key.clone());

                        // Track inactive users
                        if !user.active {
                            inactive_users.insert(user.account_id.clone());
                        }

                        // Store user details (deduped by account_id)
                        all_users.entry(user.account_id.clone()).or_insert_with(|| {
                            json!({
                                "account_id": user.account_id,
                                "display_name": user.display_name,
                                "email": user.email_address,
                                "account_type": user.account_type,
                                "active": user.active,
                            })
                        });
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        project = %project.key,
                        error = %e,
                        "Failed to get users for project"
                    );
                }
            }
        }

        if all_users.is_empty() {
            return Ok(result);
        }

        // Enrich user data with project access info
        let users_with_access: Vec<serde_json::Value> = all_users
            .iter()
            .map(|(account_id, user)| {
                let mut user = user.clone();
                if let Some(obj) = user.as_object_mut() {
                    let projects = user_projects
                        .get(account_id)
                        .map(|p| p.iter().cloned().collect::<Vec<_>>())
                        .unwrap_or_default();
                    obj.insert("project_access".to_string(), json!(projects));
                    obj.insert("project_count".to_string(), json!(projects.len()));
                }
                user
            })
            .collect();

        // Generate user inventory evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Jira User Access Report".to_string(),
            description: Some(format!(
                "{} users with access to {} projects ({} inactive)",
                all_users.len(),
                projects.len(),
                inactive_users.len()
            )),
            evidence_type: "automated".to_string(),
            source: "jira".to_string(),
            source_reference: Some("jira:users".to_string()),
            data: json!({
                "total_users": all_users.len(),
                "active_users": all_users.len() - inactive_users.len(),
                "inactive_users": inactive_users.len(),
                "projects_analyzed": projects.len(),
                "users": users_with_access,
                "inactive_user_ids": inactive_users.into_iter().collect::<Vec<_>>(),
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.2".to_string(),
                "CC6.3".to_string(),
            ],
        });

        result.records_created = all_users.len() as i32;
        Ok(result)
    }
}
