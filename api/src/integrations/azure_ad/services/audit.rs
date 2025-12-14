use crate::integrations::azure_ad::client::AzureAdClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

/// Audit Logs Collector for Azure AD
pub struct AuditCollector;

impl AuditCollector {
    /// Collect sign-in logs from Azure AD
    pub async fn sync_sign_in_logs(
        client: &AzureAdClient,
        _context: &SyncContext,
        days: u32,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get sign-in logs
        let logs = client.list_sign_in_logs(days).await?;
        result.records_processed = logs.len() as i32;

        if logs.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Azure AD Sign-In Logs Report".to_string(),
                description: Some(format!(
                    "No sign-in logs found in the last {} days",
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "azure_ad".to_string(),
                source_reference: Some("azure_ad:sign-in-logs".to_string()),
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

        // Categorize sign-in events
        let mut successful_logins = Vec::new();
        let mut failed_logins = Vec::new();
        let mut risky_logins = Vec::new();
        let mut mfa_blocked = Vec::new();
        let mut unique_users: HashMap<String, i32> = HashMap::new();
        let mut unique_apps: HashMap<String, i32> = HashMap::new();

        for log in &logs {
            let upn = log.user_principal_name.clone().unwrap_or_default();
            let app = log.app_display_name.clone().unwrap_or_default();

            *unique_users.entry(upn.clone()).or_insert(0) += 1;
            *unique_apps.entry(app.clone()).or_insert(0) += 1;

            let is_success = log.status.as_ref()
                .map(|s| s.error_code.unwrap_or(0) == 0)
                .unwrap_or(true);

            let log_detail = json!({
                "id": log.id,
                "created_date_time": log.created_date_time,
                "user_display_name": log.user_display_name,
                "user_principal_name": log.user_principal_name,
                "app_display_name": log.app_display_name,
                "ip_address": log.ip_address,
                "status": log.status,
                "is_interactive": log.is_interactive,
                "conditional_access_status": log.conditional_access_status,
                "risk_level_aggregated": log.risk_level_aggregated,
                "risk_level_during_sign_in": log.risk_level_during_sign_in,
                "location": log.location,
                "device_detail": log.device_detail,
            });

            if is_success {
                successful_logins.push(log_detail.clone());
            } else {
                failed_logins.push(log_detail.clone());

                // Check if blocked by conditional access
                if log.conditional_access_status.as_deref() == Some("failure") {
                    mfa_blocked.push(log_detail.clone());
                }
            }

            // Check for risky logins
            let risk_level = log.risk_level_aggregated.as_deref().or(log.risk_level_during_sign_in.as_deref());
            if risk_level.map(|r| r != "none" && r != "hidden").unwrap_or(false) {
                risky_logins.push(log_detail);
            }
        }

        // Generate Sign-In Logs Summary
        result.evidence_collected.push(CollectedEvidence {
            title: "Azure AD Sign-In Logs Summary".to_string(),
            description: Some(format!(
                "{} sign-in events in the last {} days ({} successful, {} failed)",
                logs.len(),
                days,
                successful_logins.len(),
                failed_logins.len()
            )),
            evidence_type: "automated".to_string(),
            source: "azure_ad".to_string(),
            source_reference: Some("azure_ad:sign-in-summary".to_string()),
            data: json!({
                "log_days": days,
                "total_events": logs.len(),
                "successful_logins": successful_logins.len(),
                "failed_logins": failed_logins.len(),
                "risky_logins": risky_logins.len(),
                "mfa_blocked_logins": mfa_blocked.len(),
                "unique_users": unique_users.len(),
                "unique_apps": unique_apps.len(),
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC7.2".to_string(),
                "CC7.3".to_string(),
            ],
        });

        // Generate Failed Logins Report
        if !failed_logins.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Azure AD Failed Sign-Ins Report".to_string(),
                description: Some(format!(
                    "{} failed sign-in attempts in the last {} days",
                    failed_logins.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "azure_ad".to_string(),
                source_reference: Some("azure_ad:failed-sign-ins".to_string()),
                data: json!({
                    "failed_login_count": failed_logins.len(),
                    "failed_logins": failed_logins,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        // Generate Risky Sign-Ins Report
        if !risky_logins.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Azure AD Risky Sign-Ins Report".to_string(),
                description: Some(format!(
                    "{} risky sign-in attempts detected in the last {} days",
                    risky_logins.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "azure_ad".to_string(),
                source_reference: Some("azure_ad:risky-sign-ins".to_string()),
                data: json!({
                    "risky_login_count": risky_logins.len(),
                    "risky_logins": risky_logins,
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

    /// Collect audit logs from Azure AD
    pub async fn sync_audit_logs(
        client: &AzureAdClient,
        _context: &SyncContext,
        days: u32,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get audit logs
        let logs = client.list_audit_logs(days).await?;
        result.records_processed = logs.len() as i32;

        if logs.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Azure AD Audit Logs Report".to_string(),
                description: Some(format!(
                    "No audit logs found in the last {} days",
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "azure_ad".to_string(),
                source_reference: Some("azure_ad:audit-logs".to_string()),
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

        // Categorize audit events
        let mut user_management_events = Vec::new();
        let mut group_management_events = Vec::new();
        let mut policy_events = Vec::new();
        let mut app_events = Vec::new();
        let mut category_counts: HashMap<String, i32> = HashMap::new();
        let mut result_counts: HashMap<String, i32> = HashMap::new();

        for log in &logs {
            let category = log.category.clone().unwrap_or_else(|| "Unknown".to_string());
            let activity_result = log.result.clone().unwrap_or_else(|| "Unknown".to_string());

            *category_counts.entry(category.clone()).or_insert(0) += 1;
            *result_counts.entry(activity_result.clone()).or_insert(0) += 1;

            let log_detail = json!({
                "id": log.id,
                "activity_date_time": log.activity_date_time,
                "activity_display_name": log.activity_display_name,
                "operation_type": log.operation_type,
                "category": log.category,
                "result": log.result,
                "result_reason": log.result_reason,
                "initiated_by": log.initiated_by,
                "target_resources": log.target_resources,
            });

            // Categorize by activity type
            match category.as_str() {
                "UserManagement" => user_management_events.push(log_detail),
                "GroupManagement" => group_management_events.push(log_detail),
                "Policy" => policy_events.push(log_detail),
                "ApplicationManagement" => app_events.push(log_detail),
                _ => {}
            }
        }

        // Generate Audit Logs Summary
        result.evidence_collected.push(CollectedEvidence {
            title: "Azure AD Audit Logs Summary".to_string(),
            description: Some(format!(
                "{} audit events in the last {} days",
                logs.len(),
                days
            )),
            evidence_type: "automated".to_string(),
            source: "azure_ad".to_string(),
            source_reference: Some("azure_ad:audit-summary".to_string()),
            data: json!({
                "log_days": days,
                "total_events": logs.len(),
                "user_management_events": user_management_events.len(),
                "group_management_events": group_management_events.len(),
                "policy_events": policy_events.len(),
                "app_events": app_events.len(),
                "category_distribution": category_counts,
                "result_distribution": result_counts,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC7.2".to_string(),
                "CC7.3".to_string(),
            ],
        });

        // Generate User Management Events Report
        if !user_management_events.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Azure AD User Management Events Report".to_string(),
                description: Some(format!(
                    "{} user management events in the last {} days",
                    user_management_events.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "azure_ad".to_string(),
                source_reference: Some("azure_ad:user-management-events".to_string()),
                data: json!({
                    "event_count": user_management_events.len(),
                    "events": user_management_events,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.2".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        // Generate Group Management Events Report
        if !group_management_events.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Azure AD Group Management Events Report".to_string(),
                description: Some(format!(
                    "{} group management events in the last {} days",
                    group_management_events.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "azure_ad".to_string(),
                source_reference: Some("azure_ad:group-management-events".to_string()),
                data: json!({
                    "event_count": group_management_events.len(),
                    "events": group_management_events,
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
                title: "Azure AD Policy Events Report".to_string(),
                description: Some(format!(
                    "{} policy-related events in the last {} days",
                    policy_events.len(),
                    days
                )),
                evidence_type: "automated".to_string(),
                source: "azure_ad".to_string(),
                source_reference: Some("azure_ad:policy-events".to_string()),
                data: json!({
                    "event_count": policy_events.len(),
                    "events": policy_events,
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
