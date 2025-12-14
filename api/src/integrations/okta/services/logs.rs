use crate::integrations::okta::client::OktaClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

/// System Logs Collector for Okta
pub struct LogsCollector;

impl LogsCollector {
    /// Collect system log data from Okta
    pub async fn sync(
        client: &OktaClient,
        _context: &SyncContext,
        days: u32,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get security-related logs
        let logs = client.list_security_logs(days).await?;
        result.records_processed = logs.len() as i32;

        if logs.is_empty() {
            // Generate empty report
            result.evidence_collected.push(CollectedEvidence {
                title: "Okta System Logs Report".to_string(),
                description: Some(format!(
                    "No security-related events found in the last {} days",
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:system-logs".to_string()),
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

        // Categorize events
        let mut event_type_counts: HashMap<String, i32> = HashMap::new();
        let mut severity_counts: HashMap<String, i32> = HashMap::new();
        let mut outcome_counts: HashMap<String, i32> = HashMap::new();
        let mut failed_logins = Vec::new();
        let mut successful_logins = Vec::new();
        let mut mfa_events = Vec::new();
        let mut user_lifecycle_events = Vec::new();
        let mut admin_events = Vec::new();
        let mut policy_events = Vec::new();

        for log in &logs {
            // Count by event type
            *event_type_counts.entry(log.event_type.clone()).or_insert(0) += 1;

            // Count by severity
            *severity_counts.entry(log.severity.clone()).or_insert(0) += 1;

            // Count by outcome
            if let Some(ref outcome) = log.outcome {
                if let Some(ref result_str) = outcome.result {
                    *outcome_counts.entry(result_str.clone()).or_insert(0) += 1;
                }
            }

            // Categorize security events
            let event_detail = json!({
                "uuid": log.uuid,
                "published": log.published,
                "event_type": log.event_type,
                "severity": log.severity,
                "display_message": log.display_message,
                "actor": log.actor.as_ref().map(|a| json!({
                    "id": a.id,
                    "type": a.actor_type,
                    "alternate_id": a.alternate_id,
                    "display_name": a.display_name,
                })),
                "client": log.client.as_ref().map(|c| json!({
                    "ip_address": c.ip_address,
                    "device": c.device,
                    "zone": c.zone,
                    "geographical_context": c.geographical_context.as_ref().map(|g| json!({
                        "city": g.city,
                        "state": g.state,
                        "country": g.country,
                    })),
                })),
                "outcome": log.outcome.as_ref().map(|o| json!({
                    "result": o.result,
                    "reason": o.reason,
                })),
                "target": log.target,
            });

            // Categorize by event type
            if log.event_type.starts_with("user.session.start") {
                if log.outcome.as_ref().and_then(|o| o.result.as_ref()).map(|r| r == "FAILURE").unwrap_or(false) {
                    failed_logins.push(event_detail.clone());
                } else {
                    successful_logins.push(event_detail.clone());
                }
            } else if log.event_type.contains("mfa") || log.event_type.contains("factor") {
                mfa_events.push(event_detail.clone());
            } else if log.event_type.starts_with("user.lifecycle") {
                user_lifecycle_events.push(event_detail.clone());
            } else if log.event_type.contains("admin") || log.event_type.starts_with("group.") || log.event_type.starts_with("application.") {
                admin_events.push(event_detail.clone());
            } else if log.event_type.starts_with("policy.") {
                policy_events.push(event_detail.clone());
            }
        }

        // Generate System Logs Summary Evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Okta System Logs Summary".to_string(),
            description: Some(format!(
                "{} security-related events in the last {} days",
                logs.len(),
                days
            )),
            evidence_type: "automated".to_string(),
            source: "okta".to_string(),
            source_reference: Some("okta:system-logs-summary".to_string()),
            data: json!({
                "log_days": days,
                "total_events": logs.len(),
                "event_type_distribution": event_type_counts,
                "severity_distribution": severity_counts,
                "outcome_distribution": outcome_counts,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC7.2".to_string(),
                "CC7.3".to_string(),
            ],
        });

        // Generate Failed Login Attempts Report
        if !failed_logins.is_empty() {
            // Group by actor
            let mut failed_by_user: HashMap<String, Vec<&serde_json::Value>> = HashMap::new();
            for event in &failed_logins {
                let user = event
                    .get("actor")
                    .and_then(|a| a.get("alternate_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                failed_by_user.entry(user).or_default().push(event);
            }

            result.evidence_collected.push(CollectedEvidence {
                title: "Okta Failed Login Attempts Report".to_string(),
                description: Some(format!(
                    "{} failed login attempts in the last {} days",
                    failed_logins.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:failed-logins".to_string()),
                data: json!({
                    "failed_login_count": failed_logins.len(),
                    "unique_users_affected": failed_by_user.len(),
                    "failed_logins": failed_logins,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        // Generate MFA Events Report
        if !mfa_events.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Okta MFA Activity Report".to_string(),
                description: Some(format!(
                    "{} MFA-related events in the last {} days",
                    mfa_events.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:mfa-activity".to_string()),
                data: json!({
                    "mfa_event_count": mfa_events.len(),
                    "mfa_events": mfa_events,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.6".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        // Generate User Lifecycle Events Report
        if !user_lifecycle_events.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Okta User Lifecycle Events Report".to_string(),
                description: Some(format!(
                    "{} user lifecycle events in the last {} days",
                    user_lifecycle_events.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:user-lifecycle".to_string()),
                data: json!({
                    "lifecycle_event_count": user_lifecycle_events.len(),
                    "lifecycle_events": user_lifecycle_events,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.2".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        // Generate Admin Activity Report
        if !admin_events.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Okta Administrative Activity Report".to_string(),
                description: Some(format!(
                    "{} administrative events in the last {} days",
                    admin_events.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:admin-activity".to_string()),
                data: json!({
                    "admin_event_count": admin_events.len(),
                    "admin_events": admin_events,
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

        // Generate Policy Events Report
        if !policy_events.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Okta Policy Events Report".to_string(),
                description: Some(format!(
                    "{} policy-related events in the last {} days",
                    policy_events.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:policy-events".to_string()),
                data: json!({
                    "policy_event_count": policy_events.len(),
                    "policy_events": policy_events,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        // Check for suspicious activity patterns
        let suspicious_events = detect_suspicious_activity(&logs);
        if !suspicious_events.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Okta Suspicious Activity Report".to_string(),
                description: Some(format!(
                    "{} potentially suspicious events detected in the last {} days",
                    suspicious_events.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:suspicious-activity".to_string()),
                data: json!({
                    "suspicious_event_count": suspicious_events.len(),
                    "suspicious_events": suspicious_events,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC7.1".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        result.records_created = logs.len() as i32;
        Ok(result)
    }
}

/// Detect potentially suspicious activity in logs
fn detect_suspicious_activity(logs: &[crate::integrations::okta::client::OktaLogEvent]) -> Vec<serde_json::Value> {
    let mut suspicious = Vec::new();

    // Group failed logins by user
    let mut failed_by_user: HashMap<String, i32> = HashMap::new();

    for log in logs {
        // Check for multiple failed logins (potential brute force)
        if log.event_type.starts_with("user.session.start") {
            if let Some(outcome) = &log.outcome {
                if outcome.result.as_ref().map(|r| r == "FAILURE").unwrap_or(false) {
                    let user = log.actor.as_ref()
                        .and_then(|a| a.alternate_id.clone())
                        .unwrap_or_else(|| "unknown".to_string());
                    *failed_by_user.entry(user).or_insert(0) += 1;
                }
            }
        }

        // Check for admin privilege escalation
        if log.event_type.contains("privilege") || log.event_type.contains("admin") {
            if log.event_type.contains("grant") || log.event_type.contains("assign") {
                suspicious.push(json!({
                    "type": "privilege_change",
                    "event": log.event_type,
                    "message": log.display_message,
                    "actor": log.actor.as_ref().and_then(|a| a.alternate_id.clone()),
                    "timestamp": log.published,
                }));
            }
        }

        // Check for suspicious IP or unusual location
        if let Some(client) = &log.client {
            if let Some(security_ctx) = &log.security_context {
                if security_ctx.is_proxy.unwrap_or(false) {
                    suspicious.push(json!({
                        "type": "proxy_detected",
                        "event": log.event_type,
                        "ip_address": client.ip_address,
                        "actor": log.actor.as_ref().and_then(|a| a.alternate_id.clone()),
                        "timestamp": log.published,
                    }));
                }
            }
        }

        // Check for password changes (potential account takeover)
        if log.event_type.contains("password") && log.event_type.contains("reset") {
            suspicious.push(json!({
                "type": "password_reset",
                "event": log.event_type,
                "message": log.display_message,
                "actor": log.actor.as_ref().and_then(|a| a.alternate_id.clone()),
                "timestamp": log.published,
            }));
        }
    }

    // Add brute force candidates (5+ failures)
    for (user, count) in failed_by_user {
        if count >= 5 {
            suspicious.push(json!({
                "type": "potential_brute_force",
                "user": user,
                "failed_attempts": count,
            }));
        }
    }

    suspicious
}
