use crate::integrations::github::client::{GitHubClient, GitHubRepository};
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

/// Security Alerts Collector for GitHub (Dependabot, Code Scanning, Secret Scanning)
pub struct SecurityAlertsCollector;

impl SecurityAlertsCollector {
    /// Collect all security alerts from GitHub
    pub async fn sync(
        client: &GitHubClient,
        repos: &[GitHubRepository],
        _context: &SyncContext,
        collect_dependabot: bool,
        collect_code_scanning: bool,
        collect_secret_scanning: bool,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Collect Dependabot alerts
        if collect_dependabot {
            match Self::collect_dependabot_alerts(client, repos).await {
                Ok(dependabot_result) => result.merge(dependabot_result),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to collect Dependabot alerts");
                }
            }
        }

        // Collect Code Scanning alerts
        if collect_code_scanning {
            match Self::collect_code_scanning_alerts(client, repos).await {
                Ok(code_scanning_result) => result.merge(code_scanning_result),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to collect Code Scanning alerts");
                }
            }
        }

        // Collect Secret Scanning alerts
        if collect_secret_scanning {
            match Self::collect_secret_scanning_alerts(client, repos).await {
                Ok(secret_scanning_result) => result.merge(secret_scanning_result),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to collect Secret Scanning alerts");
                }
            }
        }

        Ok(result)
    }

    async fn collect_dependabot_alerts(
        client: &GitHubClient,
        repos: &[GitHubRepository],
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();
        let mut all_alerts = Vec::new();
        let mut alerts_by_severity: HashMap<String, i32> = HashMap::new();
        let mut alerts_by_repo: HashMap<String, Vec<serde_json::Value>> = HashMap::new();

        for repo in repos {
            if repo.archived || repo.disabled {
                continue;
            }

            let parts: Vec<&str> = repo.full_name.split('/').collect();
            if parts.len() != 2 {
                continue;
            }
            let owner = parts[0];

            match client.list_dependabot_alerts(owner, &repo.name).await {
                Ok(alerts) => {
                    result.records_processed += alerts.len() as i32;

                    for alert in &alerts {
                        let severity = alert.security_advisory.severity.to_lowercase();
                        *alerts_by_severity.entry(severity.clone()).or_insert(0) += 1;

                        let alert_json = json!({
                            "number": alert.number,
                            "state": alert.state,
                            "severity": severity,
                            "package": {
                                "ecosystem": alert.dependency.package.ecosystem,
                                "name": alert.dependency.package.name,
                            },
                            "manifest_path": alert.dependency.manifest_path,
                            "advisory": {
                                "ghsa_id": alert.security_advisory.ghsa_id,
                                "cve_id": alert.security_advisory.cve_id,
                                "summary": alert.security_advisory.summary,
                            },
                            "vulnerable_version_range": alert.security_vulnerability.vulnerable_version_range,
                            "first_patched_version": alert.security_vulnerability.first_patched_version.as_ref().map(|v| &v.identifier),
                            "created_at": alert.created_at,
                            "html_url": alert.html_url,
                        });

                        alerts_by_repo
                            .entry(repo.full_name.clone())
                            .or_default()
                            .push(alert_json.clone());
                        all_alerts.push(alert_json);
                    }
                }
                Err(e) => {
                    tracing::debug!(repo = %repo.full_name, error = %e, "Failed to get Dependabot alerts");
                }
            }
        }

        if !all_alerts.is_empty() {
            let critical_count = *alerts_by_severity.get("critical").unwrap_or(&0);
            let high_count = *alerts_by_severity.get("high").unwrap_or(&0);

            result.evidence_collected.push(CollectedEvidence {
                title: "Dependabot Vulnerability Report".to_string(),
                description: Some(format!(
                    "{} open vulnerabilities found ({} critical, {} high)",
                    all_alerts.len(),
                    critical_count,
                    high_count
                )),
                evidence_type: "automated".to_string(),
                source: "github".to_string(),
                source_reference: Some("github:dependabot-alerts".to_string()),
                data: json!({
                    "total_alerts": all_alerts.len(),
                    "by_severity": alerts_by_severity,
                    "repositories_affected": alerts_by_repo.len(),
                    "alerts_by_repository": alerts_by_repo,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC3.2".to_string(),
                    "CC7.1".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        result.records_created = result.records_processed;
        Ok(result)
    }

    async fn collect_code_scanning_alerts(
        client: &GitHubClient,
        repos: &[GitHubRepository],
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();
        let mut all_alerts = Vec::new();
        let mut alerts_by_severity: HashMap<String, i32> = HashMap::new();
        let mut alerts_by_tool: HashMap<String, i32> = HashMap::new();

        for repo in repos {
            if repo.archived || repo.disabled {
                continue;
            }

            let parts: Vec<&str> = repo.full_name.split('/').collect();
            if parts.len() != 2 {
                continue;
            }
            let owner = parts[0];

            match client.list_code_scanning_alerts(owner, &repo.name).await {
                Ok(alerts) => {
                    result.records_processed += alerts.len() as i32;

                    for alert in &alerts {
                        let severity = alert
                            .rule
                            .security_severity_level
                            .as_deref()
                            .or(alert.rule.severity.as_deref())
                            .unwrap_or("unknown")
                            .to_lowercase();

                        *alerts_by_severity.entry(severity.clone()).or_insert(0) += 1;
                        *alerts_by_tool.entry(alert.tool.name.clone()).or_insert(0) += 1;

                        all_alerts.push(json!({
                            "repository": repo.full_name,
                            "number": alert.number,
                            "state": alert.state,
                            "severity": severity,
                            "rule": {
                                "id": alert.rule.id,
                                "name": alert.rule.name,
                                "description": alert.rule.description,
                                "tags": alert.rule.tags,
                            },
                            "tool": alert.tool.name,
                            "location": alert.most_recent_instance.as_ref().and_then(|i| {
                                i.location.as_ref().map(|l| json!({
                                    "path": l.path,
                                    "start_line": l.start_line,
                                    "end_line": l.end_line,
                                }))
                            }),
                            "created_at": alert.created_at,
                            "html_url": alert.html_url,
                        }));
                    }
                }
                Err(e) => {
                    tracing::debug!(repo = %repo.full_name, error = %e, "Failed to get Code Scanning alerts");
                }
            }
        }

        if !all_alerts.is_empty() {
            let critical_count = *alerts_by_severity.get("critical").unwrap_or(&0);
            let high_count = *alerts_by_severity.get("high").unwrap_or(&0);

            result.evidence_collected.push(CollectedEvidence {
                title: "Code Scanning Security Report".to_string(),
                description: Some(format!(
                    "{} open code scanning alerts ({} critical, {} high)",
                    all_alerts.len(),
                    critical_count,
                    high_count
                )),
                evidence_type: "automated".to_string(),
                source: "github".to_string(),
                source_reference: Some("github:code-scanning-alerts".to_string()),
                data: json!({
                    "total_alerts": all_alerts.len(),
                    "by_severity": alerts_by_severity,
                    "by_tool": alerts_by_tool,
                    "alerts": all_alerts,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC7.1".to_string(),
                    "CC7.2".to_string(),
                    "CC8.1".to_string(),
                ],
            });
        }

        result.records_created = result.records_processed;
        Ok(result)
    }

    async fn collect_secret_scanning_alerts(
        client: &GitHubClient,
        repos: &[GitHubRepository],
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();
        let mut all_alerts = Vec::new();
        let mut alerts_by_type: HashMap<String, i32> = HashMap::new();

        for repo in repos {
            if repo.archived || repo.disabled {
                continue;
            }

            let parts: Vec<&str> = repo.full_name.split('/').collect();
            if parts.len() != 2 {
                continue;
            }
            let owner = parts[0];

            match client.list_secret_scanning_alerts(owner, &repo.name).await {
                Ok(alerts) => {
                    result.records_processed += alerts.len() as i32;

                    for alert in &alerts {
                        *alerts_by_type
                            .entry(alert.secret_type_display_name.clone())
                            .or_insert(0) += 1;

                        all_alerts.push(json!({
                            "repository": repo.full_name,
                            "number": alert.number,
                            "state": alert.state,
                            "secret_type": alert.secret_type,
                            "secret_type_display_name": alert.secret_type_display_name,
                            "push_protection_bypassed": alert.push_protection_bypassed,
                            "created_at": alert.created_at,
                            "html_url": alert.html_url,
                        }));
                    }
                }
                Err(e) => {
                    tracing::debug!(repo = %repo.full_name, error = %e, "Failed to get Secret Scanning alerts");
                }
            }
        }

        if !all_alerts.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Secret Scanning Alert Report".to_string(),
                description: Some(format!(
                    "{} secrets detected in code repositories",
                    all_alerts.len()
                )),
                evidence_type: "automated".to_string(),
                source: "github".to_string(),
                source_reference: Some("github:secret-scanning-alerts".to_string()),
                data: json!({
                    "total_alerts": all_alerts.len(),
                    "by_secret_type": alerts_by_type,
                    "alerts": all_alerts,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.7".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        result.records_created = result.records_processed;
        Ok(result)
    }
}
