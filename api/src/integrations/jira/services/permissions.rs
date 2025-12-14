use crate::integrations::jira::client::{JiraClient, JiraProject};
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

/// Permissions Collector for Jira
pub struct PermissionsCollector;

impl PermissionsCollector {
    /// Collect project permissions data from Jira
    pub async fn sync(
        client: &JiraClient,
        projects: &[JiraProject],
        _context: &SyncContext,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        let mut all_role_assignments = Vec::new();
        let mut admin_users: HashMap<String, Vec<String>> = HashMap::new();

        for project in projects {
            if project.archived.unwrap_or(false) {
                continue;
            }

            result.records_processed += 1;

            match client.get_project_roles(&project.key).await {
                Ok(roles) => {
                    for role in roles {
                        let mut role_actors = Vec::new();

                        if let Some(actors) = &role.actors {
                            for actor in actors {
                                let actor_info = json!({
                                    "type": actor.actor_type,
                                    "display_name": actor.display_name,
                                    "name": actor.name,
                                    "account_id": actor.actor_user.as_ref().map(|u| &u.account_id),
                                });
                                role_actors.push(actor_info);

                                // Track admin role assignments
                                if role.name.to_lowercase().contains("admin") {
                                    if let Some(user) = &actor.actor_user {
                                        admin_users
                                            .entry(project.key.clone())
                                            .or_default()
                                            .push(user.account_id.clone());
                                    }
                                }
                            }
                        }

                        if !role_actors.is_empty() {
                            all_role_assignments.push(json!({
                                "project": project.key,
                                "project_name": project.name,
                                "role_id": role.id,
                                "role_name": role.name,
                                "description": role.description,
                                "actors": role_actors,
                                "actor_count": role_actors.len(),
                            }));
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        project = %project.key,
                        error = %e,
                        "Failed to get roles for project"
                    );
                }
            }
        }

        if all_role_assignments.is_empty() {
            return Ok(result);
        }

        // Count total admin assignments
        let total_admin_assignments: usize = admin_users.values().map(|v| v.len()).sum();

        // Generate project roles evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Jira Project Role Assignments".to_string(),
            description: Some(format!(
                "Role assignments across {} projects ({} admin assignments)",
                projects.len(),
                total_admin_assignments
            )),
            evidence_type: "automated".to_string(),
            source: "jira".to_string(),
            source_reference: Some("jira:project-roles".to_string()),
            data: json!({
                "projects_analyzed": projects.len(),
                "total_role_assignments": all_role_assignments.len(),
                "admin_assignments": total_admin_assignments,
                "role_assignments": all_role_assignments,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.2".to_string(),
                "CC6.3".to_string(),
            ],
        });

        // Generate admin access report if there are admin assignments
        if !admin_users.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Jira Project Admin Access Report".to_string(),
                description: Some(format!(
                    "{} admin role assignments across {} projects",
                    total_admin_assignments,
                    admin_users.len()
                )),
                evidence_type: "automated".to_string(),
                source: "jira".to_string(),
                source_reference: Some("jira:admin-access".to_string()),
                data: json!({
                    "projects_with_admins": admin_users.len(),
                    "total_admin_assignments": total_admin_assignments,
                    "admin_by_project": admin_users,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.2".to_string(),
                    "CC6.3".to_string(),
                ],
            });
        }

        result.records_created = result.records_processed;
        Ok(result)
    }
}
