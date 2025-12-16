use crate::integrations::aws::client::AwsClient;
use crate::integrations::provider::{CollectedEvidence, SecurityAlertInfo, SyncContext, SyncResult};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// CloudTrail event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsCloudTrailEvent {
    pub event_id: String,
    pub event_name: String,
    pub event_source: String,
    pub event_time: DateTime<Utc>,
    pub username: Option<String>,
    pub user_type: Option<String>,
    pub source_ip: Option<String>,
    pub user_agent: Option<String>,
    pub aws_region: String,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub read_only: bool,
    pub is_root_action: bool,
    pub is_sensitive_action: bool,
    pub resources: Vec<String>,
}

/// Sensitive IAM actions to flag
const SENSITIVE_ACTIONS: &[&str] = &[
    "CreateUser",
    "DeleteUser",
    "CreateRole",
    "DeleteRole",
    "CreatePolicy",
    "DeletePolicy",
    "AttachUserPolicy",
    "AttachRolePolicy",
    "DetachUserPolicy",
    "DetachRolePolicy",
    "PutUserPolicy",
    "PutRolePolicy",
    "CreateAccessKey",
    "DeleteAccessKey",
    "UpdateAccessKey",
    "CreateLoginProfile",
    "UpdateLoginProfile",
    "DeleteLoginProfile",
    "DeactivateMFADevice",
    "EnableMFADevice",
    "CreateVirtualMFADevice",
    "DeleteVirtualMFADevice",
    "UpdateAssumeRolePolicy",
    "CreateSAMLProvider",
    "DeleteSAMLProvider",
    "CreateOpenIDConnectProvider",
    "DeleteOpenIDConnectProvider",
    "ConsoleLogin",
    "StopLogging",
    "DeleteTrail",
    "UpdateTrail",
    "PutEventSelectors",
];

/// CloudTrail collector
pub struct CloudTrailCollector;

impl CloudTrailCollector {
    /// Sync CloudTrail events for a region
    pub async fn sync(
        client: &AwsClient,
        _context: &SyncContext,
        region: &str,
        hours: u32,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        let ct_client = client.cloudtrail_client(region).await?;

        // Collect events from the last N hours
        let start_time = Utc::now() - Duration::hours(hours as i64);
        let events = Self::collect_events(&ct_client, region, start_time).await?;
        result.records_processed = events.len() as i32;

        // Filter for security-relevant events
        let root_actions: Vec<_> = events.iter().filter(|e| e.is_root_action).collect();
        let sensitive_actions: Vec<_> = events.iter().filter(|e| e.is_sensitive_action).collect();
        let failed_actions: Vec<_> = events.iter().filter(|e| e.error_code.is_some()).collect();

        // Generate evidence
        result.evidence_collected.push(CollectedEvidence {
            title: format!("CloudTrail Activity Summary - {} ({}h)", region, hours),
            description: Some(format!(
                "{} events in last {} hours: {} root actions, {} sensitive actions, {} failures",
                events.len(),
                hours,
                root_actions.len(),
                sensitive_actions.len(),
                failed_actions.len()
            )),
            evidence_type: "automated".to_string(),
            source: "aws".to_string(),
            source_reference: Some(format!("cloudtrail:{}:events", region)),
            data: json!({
                "region": region,
                "hours": hours,
                "total_events": events.len(),
                "root_actions": root_actions.len(),
                "sensitive_actions": sensitive_actions.len(),
                "failed_actions": failed_actions.len(),
                "event_sources": Self::count_by_source(&events),
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec!["CC7.2".to_string(), "CC7.3".to_string()],
        });

        // Root account activity evidence
        if !root_actions.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: format!("Root Account Activity - {}", region),
                description: Some(format!(
                    "{} actions performed by root account",
                    root_actions.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some(format!("cloudtrail:{}:root", region)),
                data: json!({
                    "root_actions": root_actions.iter().map(|e| json!({
                        "event_name": e.event_name,
                        "event_time": e.event_time,
                        "source_ip": e.source_ip,
                        "event_source": e.event_source,
                    })).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC6.1".to_string(), "CC7.2".to_string()],
            });
        }

        // Sensitive actions evidence
        if !sensitive_actions.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: format!("Sensitive IAM Actions - {}", region),
                description: Some(format!(
                    "{} sensitive IAM actions detected",
                    sensitive_actions.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some(format!("cloudtrail:{}:sensitive", region)),
                data: json!({
                    "sensitive_actions": sensitive_actions.iter().take(100).map(|e| json!({
                        "event_name": e.event_name,
                        "event_time": e.event_time,
                        "username": e.username,
                        "source_ip": e.source_ip,
                        "error_code": e.error_code,
                    })).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC6.1".to_string(), "CC6.2".to_string(), "CC7.2".to_string()],
            });
        }

        // Populate security alerts for notification system
        result.security_alerts = Some(SecurityAlertInfo {
            root_actions: root_actions.iter().map(|e| json!({
                "event_name": e.event_name,
                "event_time": e.event_time.to_rfc3339(),
                "source_ip": e.source_ip,
                "event_source": e.event_source,
            })).collect(),
            sensitive_actions: sensitive_actions.iter().take(50).map(|e| json!({
                "event_name": e.event_name,
                "event_time": e.event_time.to_rfc3339(),
                "username": e.username,
                "source_ip": e.source_ip,
            })).collect(),
            failed_actions: failed_actions.iter().take(50).map(|e| json!({
                "event_name": e.event_name,
                "event_time": e.event_time.to_rfc3339(),
                "username": e.username,
                "error_code": e.error_code,
            })).collect(),
            ..Default::default()
        });

        result.records_created = result.records_processed;
        Ok(result)
    }

    async fn collect_events(
        ct_client: &aws_sdk_cloudtrail::Client,
        region: &str,
        start_time: DateTime<Utc>,
    ) -> Result<Vec<AwsCloudTrailEvent>, String> {
        let mut events = Vec::new();
        let mut next_token: Option<String> = None;

        let start_aws = aws_sdk_cloudtrail::primitives::DateTime::from_secs(start_time.timestamp());
        let end_aws = aws_sdk_cloudtrail::primitives::DateTime::from_secs(Utc::now().timestamp());

        loop {
            let mut request = ct_client
                .lookup_events()
                .start_time(start_aws)
                .end_time(end_aws)
                .max_results(50);

            if let Some(token) = &next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format!("Failed to lookup CloudTrail events: {}", e))?;

            for event in response.events() {
                let event_name = event.event_name().unwrap_or_default().to_string();
                let username = event.username().map(|s| s.to_string());

                // Determine if root action
                let is_root = username.as_ref().map_or(false, |u| u == "root")
                    || event
                        .cloud_trail_event()
                        .and_then(|e| serde_json::from_str::<serde_json::Value>(e).ok())
                        .and_then(|v| v.get("userIdentity")?.get("type")?.as_str().map(|s| s == "Root"))
                        .unwrap_or(false);

                // Determine if sensitive action
                let is_sensitive = SENSITIVE_ACTIONS.iter().any(|a| event_name.contains(a));

                // Parse event details from JSON
                let event_details: Option<serde_json::Value> = event
                    .cloud_trail_event()
                    .and_then(|e| serde_json::from_str(e).ok());

                let (error_code, error_message) = event_details
                    .as_ref()
                    .map(|v| {
                        (
                            v.get("errorCode").and_then(|e| e.as_str()).map(String::from),
                            v.get("errorMessage").and_then(|e| e.as_str()).map(String::from),
                        )
                    })
                    .unwrap_or((None, None));

                let source_ip = event_details
                    .as_ref()
                    .and_then(|v| v.get("sourceIPAddress"))
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let user_agent = event_details
                    .as_ref()
                    .and_then(|v| v.get("userAgent"))
                    .and_then(|v| v.as_str())
                    .map(String::from);

                let read_only = event_details
                    .as_ref()
                    .and_then(|v| v.get("readOnly"))
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let event_time = event
                    .event_time()
                    .map(|t| {
                        DateTime::from_timestamp(t.secs(), t.subsec_nanos())
                            .unwrap_or_else(Utc::now)
                    })
                    .unwrap_or_else(Utc::now);

                events.push(AwsCloudTrailEvent {
                    event_id: event.event_id().unwrap_or_default().to_string(),
                    event_name,
                    event_source: event.event_source().unwrap_or_default().to_string(),
                    event_time,
                    username,
                    user_type: event_details
                        .as_ref()
                        .and_then(|v| v.get("userIdentity")?.get("type")?.as_str())
                        .map(String::from),
                    source_ip,
                    user_agent,
                    aws_region: region.to_string(),
                    error_code,
                    error_message,
                    read_only,
                    is_root_action: is_root,
                    is_sensitive_action: is_sensitive,
                    resources: event
                        .resources()
                        .iter()
                        .filter_map(|r| r.resource_name().map(String::from))
                        .collect(),
                });
            }

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() || events.len() >= 1000 {
                break;
            }
        }

        Ok(events)
    }

    fn count_by_source(events: &[AwsCloudTrailEvent]) -> serde_json::Value {
        let mut counts: std::collections::HashMap<&str, i32> = std::collections::HashMap::new();
        for event in events {
            *counts.entry(event.event_source.as_str()).or_insert(0) += 1;
        }
        json!(counts)
    }
}
