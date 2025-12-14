use crate::integrations::jira::client::{JiraClient, JiraProject};
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

/// Issue Collector for Jira
pub struct IssueCollector;

impl IssueCollector {
    /// Collect issue data from Jira
    pub async fn sync(
        client: &JiraClient,
        projects: &[JiraProject],
        _context: &SyncContext,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Build JQL to get all open issues for the projects
        let project_keys: Vec<String> = projects
            .iter()
            .filter(|p| !p.archived.unwrap_or(false))
            .map(|p| p.key.clone())
            .collect();

        if project_keys.is_empty() {
            return Ok(result);
        }

        // Search for open issues
        let jql = format!(
            "project IN ({}) AND resolution IS EMPTY ORDER BY created DESC",
            project_keys.join(",")
        );

        let issues = client.search_issues(&jql, 500).await?;
        result.records_processed = issues.len() as i32;

        if issues.is_empty() {
            return Ok(result);
        }

        // Analyze issues
        let mut issues_by_type: HashMap<String, i32> = HashMap::new();
        let mut issues_by_priority: HashMap<String, i32> = HashMap::new();
        let mut issues_by_status: HashMap<String, i32> = HashMap::new();
        let mut issues_by_project: HashMap<String, i32> = HashMap::new();
        let mut security_issues = Vec::new();

        for issue in &issues {
            *issues_by_type.entry(issue.fields.issue_type.name.clone()).or_insert(0) += 1;

            let priority = issue
                .fields
                .priority
                .as_ref()
                .map(|p| p.name.clone())
                .unwrap_or_else(|| "None".to_string());
            *issues_by_priority.entry(priority).or_insert(0) += 1;

            *issues_by_status.entry(issue.fields.status.name.clone()).or_insert(0) += 1;
            *issues_by_project.entry(issue.fields.project.key.clone()).or_insert(0) += 1;

            // Track issues with security labels or security levels set
            let has_security_label = issue
                .fields
                .labels
                .as_ref()
                .map(|labels| {
                    labels.iter().any(|l| {
                        let lower = l.to_lowercase();
                        lower.contains("security")
                            || lower.contains("vulnerability")
                            || lower.contains("cve")
                    })
                })
                .unwrap_or(false);

            if has_security_label || issue.fields.security_level.is_some() {
                security_issues.push(json!({
                    "key": issue.key,
                    "summary": issue.fields.summary,
                    "type": issue.fields.issue_type.name,
                    "status": issue.fields.status.name,
                    "priority": issue.fields.priority.as_ref().map(|p| &p.name),
                    "security_level": issue.fields.security_level.as_ref().map(|s| &s.name),
                    "labels": issue.fields.labels,
                    "created": issue.fields.created,
                }));
            }
        }

        // Generate issues overview evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Jira Issues Overview".to_string(),
            description: Some(format!(
                "{} open issues across {} projects",
                issues.len(),
                issues_by_project.len()
            )),
            evidence_type: "automated".to_string(),
            source: "jira".to_string(),
            source_reference: Some("jira:issues".to_string()),
            data: json!({
                "total_open_issues": issues.len(),
                "projects_with_issues": issues_by_project.len(),
                "by_type": issues_by_type,
                "by_priority": issues_by_priority,
                "by_status": issues_by_status,
                "by_project": issues_by_project,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec!["CC7.1".to_string(), "CC3.2".to_string()],
        });

        // Generate security-related issues report if any found
        if !security_issues.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Security-Related Issues Report".to_string(),
                description: Some(format!(
                    "{} open security-related issues found",
                    security_issues.len()
                )),
                evidence_type: "automated".to_string(),
                source: "jira".to_string(),
                source_reference: Some("jira:security-issues".to_string()),
                data: json!({
                    "total_security_issues": security_issues.len(),
                    "issues": security_issues,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC7.1".to_string(),
                    "CC7.2".to_string(),
                    "CC3.2".to_string(),
                ],
            });
        }

        result.records_created = result.records_processed;
        Ok(result)
    }
}
