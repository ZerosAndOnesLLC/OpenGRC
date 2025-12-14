use crate::integrations::google_workspace::client::GoogleWorkspaceClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

/// Audit Logs Collector for Google Workspace
pub struct AuditCollector;

impl AuditCollector {
    /// Collect login audit data from Google Workspace
    pub async fn sync_login_audit(
        client: &GoogleWorkspaceClient,
        _context: &SyncContext,
        days: u32,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get login activities
        let activities = client.list_login_activities(days).await?;
        result.records_processed = activities.len() as i32;

        if activities.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Google Workspace Login Audit Report".to_string(),
                description: Some(format!(
                    "No login activities found in the last {} days",
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "google_workspace".to_string(),
                source_reference: Some("google_workspace:login-audit".to_string()),
                data: json!({
                    "log_days": days,
                    "total_events": 0,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC7.2".to_string(),
                    "CC7.3".to_string(),
                ],
            });
            return Ok(result);
        }

        // Categorize login events
        let mut login_success = Vec::new();
        let mut login_failure = Vec::new();
        let mut suspicious_logins = Vec::new();
        let mut event_type_counts: HashMap<String, i32> = HashMap::new();
        let mut unique_users: HashMap<String, i32> = HashMap::new();

        for activity in &activities {
            let actor_email = activity.actor.as_ref().and_then(|a| a.email.clone()).unwrap_or_default();

            // Count unique users
            *unique_users.entry(actor_email.clone()).or_insert(0) += 1;

            if let Some(ref events) = activity.events {
                for event in events {
                    let event_name = event.name.clone().unwrap_or_default();

                    // Count event types
                    *event_type_counts.entry(event_name.clone()).or_insert(0) += 1;

                    let event_detail = json!({
                        "time": activity.id.as_ref().and_then(|id| id.time.clone()),
                        "actor_email": actor_email,
                        "ip_address": activity.ip_address,
                        "event_type": event.event_type,
                        "event_name": event_name.clone(),
                        "parameters": event.parameters,
                    });

                    // Categorize
                    match event_name.as_str() {
                        "login_success" => login_success.push(event_detail),
                        "login_failure" | "login_challenge" | "login_verification" => {
                            login_failure.push(event_detail.clone());

                            // Check for suspicious patterns
                            if let Some(ref params) = event.parameters {
                                for param in params {
                                    if param.name.as_deref() == Some("is_suspicious") && param.bool_value.unwrap_or(false) {
                                        suspicious_logins.push(event_detail.clone());
                                    }
                                }
                            }
                        }
                        "logout" => {} // Not tracking logouts
                        _ => {}
                    }
                }
            }
        }

        // Generate Login Audit Summary
        result.evidence_collected.push(CollectedEvidence {
            title: "Google Workspace Login Audit Summary".to_string(),
            description: Some(format!(
                "{} login events in the last {} days ({} successful, {} failed)",
                activities.len(),
                days,
                login_success.len(),
                login_failure.len()
            )),
            evidence_type: "automated".to_string(),
            source: "google_workspace".to_string(),
            source_reference: Some("google_workspace:login-audit-summary".to_string()),
            data: json!({
                "log_days": days,
                "total_events": activities.len(),
                "successful_logins": login_success.len(),
                "failed_logins": login_failure.len(),
                "unique_users": unique_users.len(),
                "event_type_distribution": event_type_counts,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC7.2".to_string(),
                "CC7.3".to_string(),
            ],
        });

        // Generate Failed Logins Report
        if !login_failure.is_empty() {
            // Group failures by user
            let mut failures_by_user: HashMap<String, Vec<&serde_json::Value>> = HashMap::new();
            for event in &login_failure {
                let user = event.get("actor_email").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
                failures_by_user.entry(user).or_default().push(event);
            }

            result.evidence_collected.push(CollectedEvidence {
                title: "Google Workspace Failed Login Report".to_string(),
                description: Some(format!(
                    "{} failed login attempts across {} users in the last {} days",
                    login_failure.len(),
                    failures_by_user.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "google_workspace".to_string(),
                source_reference: Some("google_workspace:failed-logins".to_string()),
                data: json!({
                    "failed_login_count": login_failure.len(),
                    "unique_users_affected": failures_by_user.len(),
                    "failed_logins": login_failure,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        // Generate Suspicious Login Report
        if !suspicious_logins.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Google Workspace Suspicious Login Report".to_string(),
                description: Some(format!(
                    "{} suspicious login attempts detected in the last {} days",
                    suspicious_logins.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "google_workspace".to_string(),
                source_reference: Some("google_workspace:suspicious-logins".to_string()),
                data: json!({
                    "suspicious_login_count": suspicious_logins.len(),
                    "suspicious_logins": suspicious_logins,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC7.1".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        result.records_created = activities.len() as i32;
        Ok(result)
    }

    /// Collect admin audit data from Google Workspace
    pub async fn sync_admin_audit(
        client: &GoogleWorkspaceClient,
        _context: &SyncContext,
        days: u32,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get admin activities
        let activities = client.list_admin_activities(days).await?;
        result.records_processed = activities.len() as i32;

        if activities.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Google Workspace Admin Audit Report".to_string(),
                description: Some(format!(
                    "No admin activities found in the last {} days",
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "google_workspace".to_string(),
                source_reference: Some("google_workspace:admin-audit".to_string()),
                data: json!({
                    "log_days": days,
                    "total_events": 0,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC7.2".to_string(),
                    "CC7.3".to_string(),
                ],
            });
            return Ok(result);
        }

        // Categorize admin events
        let mut user_changes = Vec::new();
        let mut group_changes = Vec::new();
        let mut security_changes = Vec::new();
        let mut application_changes = Vec::new();
        let mut domain_changes = Vec::new();
        let mut event_type_counts: HashMap<String, i32> = HashMap::new();

        for activity in &activities {
            let actor_email = activity.actor.as_ref().and_then(|a| a.email.clone()).unwrap_or_default();

            if let Some(ref events) = activity.events {
                for event in events {
                    let event_name = event.name.clone().unwrap_or_default();
                    let event_type = event.event_type.clone().unwrap_or_default();

                    // Count event types
                    *event_type_counts.entry(event_name.clone()).or_insert(0) += 1;

                    let event_detail = json!({
                        "time": activity.id.as_ref().and_then(|id| id.time.clone()),
                        "actor_email": actor_email,
                        "ip_address": activity.ip_address,
                        "event_type": event_type.clone(),
                        "event_name": event_name.clone(),
                        "parameters": event.parameters,
                    });

                    // Categorize by event type
                    match event_type.as_str() {
                        "USER_SETTINGS" | "CREATE_USER" | "DELETE_USER" | "SUSPEND_USER" | "UNSUSPEND_USER" => {
                            user_changes.push(event_detail);
                        }
                        "GROUP_SETTINGS" | "CREATE_GROUP" | "DELETE_GROUP" | "ADD_GROUP_MEMBER" | "REMOVE_GROUP_MEMBER" => {
                            group_changes.push(event_detail);
                        }
                        "SECURITY_SETTINGS" | "2SV_SETTING_CHANGE" | "SSO_SETTINGS_CHANGE" => {
                            security_changes.push(event_detail);
                        }
                        "APPLICATION_SETTINGS" | "APP_INSTALL" | "APP_UNINSTALL" => {
                            application_changes.push(event_detail);
                        }
                        "DOMAIN_SETTINGS" | "DNS_SETTINGS" => {
                            domain_changes.push(event_detail);
                        }
                        _ => {
                            // Check event name for categorization
                            if event_name.contains("USER") || event_name.contains("user") {
                                user_changes.push(event_detail);
                            } else if event_name.contains("GROUP") || event_name.contains("group") {
                                group_changes.push(event_detail);
                            } else if event_name.contains("SECURITY") || event_name.contains("2SV") {
                                security_changes.push(event_detail);
                            }
                        }
                    }
                }
            }
        }

        // Generate Admin Audit Summary
        result.evidence_collected.push(CollectedEvidence {
            title: "Google Workspace Admin Audit Summary".to_string(),
            description: Some(format!(
                "{} administrative events in the last {} days",
                activities.len(),
                days
            )),
            evidence_type: "automated".to_string(),
            source: "google_workspace".to_string(),
            source_reference: Some("google_workspace:admin-audit-summary".to_string()),
            data: json!({
                "log_days": days,
                "total_events": activities.len(),
                "user_changes": user_changes.len(),
                "group_changes": group_changes.len(),
                "security_changes": security_changes.len(),
                "application_changes": application_changes.len(),
                "domain_changes": domain_changes.len(),
                "event_type_distribution": event_type_counts,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC7.2".to_string(),
                "CC7.3".to_string(),
            ],
        });

        // Generate User Changes Report
        if !user_changes.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Google Workspace User Changes Report".to_string(),
                description: Some(format!(
                    "{} user-related administrative changes in the last {} days",
                    user_changes.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "google_workspace".to_string(),
                source_reference: Some("google_workspace:user-changes".to_string()),
                data: json!({
                    "user_change_count": user_changes.len(),
                    "user_changes": user_changes,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.2".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        // Generate Security Changes Report
        if !security_changes.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Google Workspace Security Settings Changes Report".to_string(),
                description: Some(format!(
                    "{} security-related administrative changes in the last {} days",
                    security_changes.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "google_workspace".to_string(),
                source_reference: Some("google_workspace:security-changes".to_string()),
                data: json!({
                    "security_change_count": security_changes.len(),
                    "security_changes": security_changes,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.6".to_string(),
                    "CC7.1".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        // Generate Group Changes Report
        if !group_changes.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Google Workspace Group Changes Report".to_string(),
                description: Some(format!(
                    "{} group-related administrative changes in the last {} days",
                    group_changes.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "google_workspace".to_string(),
                source_reference: Some("google_workspace:group-changes".to_string()),
                data: json!({
                    "group_change_count": group_changes.len(),
                    "group_changes": group_changes,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.2".to_string(),
                    "CC6.3".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        result.records_created = activities.len() as i32;
        Ok(result)
    }
}
