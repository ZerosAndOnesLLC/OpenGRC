use crate::integrations::azure_ad::client::AzureAdClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;

/// User Directory Collector for Azure AD
pub struct UsersCollector;

impl UsersCollector {
    /// Collect user directory data from Azure AD
    pub async fn sync(client: &AzureAdClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get all users
        let users = client.list_users().await?;
        result.records_processed = users.len() as i32;

        if users.is_empty() {
            return Ok(result);
        }

        // Categorize users
        let enabled_users: Vec<_> = users.iter().filter(|u| u.account_enabled.unwrap_or(false)).collect();
        let disabled_users: Vec<_> = users.iter().filter(|u| !u.account_enabled.unwrap_or(true)).collect();
        let member_users: Vec<_> = users.iter().filter(|u| u.user_type.as_deref() == Some("Member")).collect();
        let guest_users: Vec<_> = users.iter().filter(|u| u.user_type.as_deref() == Some("Guest")).collect();
        let licensed_users: Vec<_> = users.iter().filter(|u| {
            u.assigned_licenses.as_ref().map(|l| !l.is_empty()).unwrap_or(false)
        }).collect();

        // Build user details
        let user_details: Vec<_> = users
            .iter()
            .map(|u| {
                json!({
                    "id": u.id,
                    "display_name": u.display_name,
                    "user_principal_name": u.user_principal_name,
                    "mail": u.mail,
                    "given_name": u.given_name,
                    "surname": u.surname,
                    "job_title": u.job_title,
                    "department": u.department,
                    "account_enabled": u.account_enabled,
                    "user_type": u.user_type,
                    "created_date_time": u.created_date_time,
                    "last_sign_in": u.sign_in_activity.as_ref().and_then(|s| s.last_sign_in_date_time.clone()),
                    "has_license": u.assigned_licenses.as_ref().map(|l| !l.is_empty()).unwrap_or(false),
                })
            })
            .collect();

        // Generate User Inventory Evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Azure AD User Directory Report".to_string(),
            description: Some(format!(
                "Azure AD directory contains {} total users ({} enabled, {} disabled, {} members, {} guests)",
                users.len(),
                enabled_users.len(),
                disabled_users.len(),
                member_users.len(),
                guest_users.len()
            )),
            evidence_type: "automated".to_string(),
            source: "azure_ad".to_string(),
            source_reference: Some("azure_ad:user-directory".to_string()),
            data: json!({
                "total_users": users.len(),
                "enabled_users": enabled_users.len(),
                "disabled_users": disabled_users.len(),
                "member_users": member_users.len(),
                "guest_users": guest_users.len(),
                "licensed_users": licensed_users.len(),
                "users": user_details,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.2".to_string(),
                "CC6.3".to_string(),
            ],
        });

        // Generate Guest Users Report
        if !guest_users.is_empty() {
            let guest_details: Vec<_> = guest_users
                .iter()
                .map(|u| {
                    json!({
                        "id": u.id,
                        "display_name": u.display_name,
                        "user_principal_name": u.user_principal_name,
                        "mail": u.mail,
                        "account_enabled": u.account_enabled,
                        "created_date_time": u.created_date_time,
                        "last_sign_in": u.sign_in_activity.as_ref().and_then(|s| s.last_sign_in_date_time.clone()),
                    })
                })
                .collect();

            result.evidence_collected.push(CollectedEvidence {
                title: "Azure AD Guest Users Report".to_string(),
                description: Some(format!(
                    "{} guest users in the directory",
                    guest_users.len()
                )),
                evidence_type: "automated".to_string(),
                source: "azure_ad".to_string(),
                source_reference: Some("azure_ad:guest-users".to_string()),
                data: json!({
                    "guest_user_count": guest_users.len(),
                    "guest_users": guest_details,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.2".to_string(),
                    "CC6.3".to_string(),
                ],
            });
        }

        // Generate stale users report (no sign-in in 90 days)
        let stale_threshold = Utc::now() - chrono::Duration::days(90);
        let stale_users: Vec<_> = enabled_users
            .iter()
            .filter(|u| {
                u.sign_in_activity
                    .as_ref()
                    .and_then(|s| s.last_sign_in_date_time.as_ref())
                    .map(|l| {
                        chrono::DateTime::parse_from_rfc3339(l)
                            .map(|dt| dt < stale_threshold)
                            .unwrap_or(true)
                    })
                    .unwrap_or(true)
            })
            .collect();

        if !stale_users.is_empty() {
            let stale_details: Vec<_> = stale_users
                .iter()
                .map(|u| {
                    json!({
                        "id": u.id,
                        "display_name": u.display_name,
                        "user_principal_name": u.user_principal_name,
                        "mail": u.mail,
                        "department": u.department,
                        "created_date_time": u.created_date_time,
                        "last_sign_in": u.sign_in_activity.as_ref().and_then(|s| s.last_sign_in_date_time.clone()),
                    })
                })
                .collect();

            result.evidence_collected.push(CollectedEvidence {
                title: "Azure AD Stale User Accounts Report".to_string(),
                description: Some(format!(
                    "{} enabled users have not signed in within 90 days",
                    stale_users.len()
                )),
                evidence_type: "automated".to_string(),
                source: "azure_ad".to_string(),
                source_reference: Some("azure_ad:stale-users".to_string()),
                data: json!({
                    "stale_user_count": stale_users.len(),
                    "threshold_days": 90,
                    "stale_users": stale_details,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.2".to_string(),
                    "CC6.3".to_string(),
                ],
            });
        }

        result.records_created = result.records_processed;
        Ok(result)
    }
}

/// Groups Collector for Azure AD
pub struct GroupsCollector;

impl GroupsCollector {
    /// Collect group data from Azure AD
    pub async fn sync(client: &AzureAdClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get all groups
        let groups = client.list_groups().await?;
        result.records_processed = groups.len() as i32;

        if groups.is_empty() {
            return Ok(result);
        }

        // Categorize groups
        let security_groups: Vec<_> = groups.iter().filter(|g| g.security_enabled.unwrap_or(false)).collect();
        let mail_groups: Vec<_> = groups.iter().filter(|g| g.mail_enabled.unwrap_or(false)).collect();
        let dynamic_groups: Vec<_> = groups.iter().filter(|g| {
            g.group_types.as_ref().map(|t| t.contains(&"DynamicMembership".to_string())).unwrap_or(false)
        }).collect();

        // Build group details with member counts
        let mut group_details = Vec::new();
        for group in &groups {
            let members = client.list_group_members(&group.id).await.unwrap_or_default();

            group_details.push(json!({
                "id": group.id,
                "display_name": group.display_name,
                "description": group.description,
                "mail": group.mail,
                "security_enabled": group.security_enabled,
                "mail_enabled": group.mail_enabled,
                "group_types": group.group_types,
                "member_count": members.len(),
                "membership_rule": group.membership_rule,
                "created_date_time": group.created_date_time,
            }));
        }

        // Generate Groups Inventory Evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Azure AD Groups Report".to_string(),
            description: Some(format!(
                "Azure AD has {} groups ({} security, {} mail-enabled, {} dynamic)",
                groups.len(),
                security_groups.len(),
                mail_groups.len(),
                dynamic_groups.len()
            )),
            evidence_type: "automated".to_string(),
            source: "azure_ad".to_string(),
            source_reference: Some("azure_ad:groups".to_string()),
            data: json!({
                "total_groups": groups.len(),
                "security_groups": security_groups.len(),
                "mail_groups": mail_groups.len(),
                "dynamic_groups": dynamic_groups.len(),
                "groups": group_details,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.2".to_string(),
                "CC6.3".to_string(),
            ],
        });

        result.records_created = groups.len() as i32;
        Ok(result)
    }
}

/// Conditional Access Collector for Azure AD
pub struct ConditionalAccessCollector;

impl ConditionalAccessCollector {
    /// Collect conditional access policies from Azure AD
    pub async fn sync(client: &AzureAdClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get conditional access policies
        let policies = client.list_conditional_access_policies().await?;
        result.records_processed = policies.len() as i32;

        if policies.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Azure AD Conditional Access Report".to_string(),
                description: Some("No conditional access policies found or insufficient permissions".to_string()),
                evidence_type: "automated".to_string(),
                source: "azure_ad".to_string(),
                source_reference: Some("azure_ad:conditional-access".to_string()),
                data: json!({
                    "policy_count": 0,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.6".to_string(),
                ],
            });
            return Ok(result);
        }

        // Categorize policies
        let enabled_policies: Vec<_> = policies.iter().filter(|p| p.state.as_deref() == Some("enabled")).collect();
        let disabled_policies: Vec<_> = policies.iter().filter(|p| p.state.as_deref() == Some("disabled")).collect();
        let report_only_policies: Vec<_> = policies.iter().filter(|p| p.state.as_deref() == Some("enabledForReportingButNotEnforced")).collect();

        // Identify MFA-related policies
        let mfa_policies: Vec<_> = policies.iter().filter(|p| {
            p.grant_controls.as_ref()
                .and_then(|g| g.built_in_controls.as_ref())
                .map(|c| c.contains(&"mfa".to_string()))
                .unwrap_or(false)
        }).collect();

        // Build policy details
        let policy_details: Vec<_> = policies
            .iter()
            .map(|p| {
                json!({
                    "id": p.id,
                    "display_name": p.display_name,
                    "state": p.state,
                    "created_date_time": p.created_date_time,
                    "modified_date_time": p.modified_date_time,
                    "conditions": p.conditions,
                    "grant_controls": p.grant_controls,
                })
            })
            .collect();

        // Generate Conditional Access Report
        result.evidence_collected.push(CollectedEvidence {
            title: "Azure AD Conditional Access Policies Report".to_string(),
            description: Some(format!(
                "{} conditional access policies ({} enabled, {} report-only, {} disabled)",
                policies.len(),
                enabled_policies.len(),
                report_only_policies.len(),
                disabled_policies.len()
            )),
            evidence_type: "automated".to_string(),
            source: "azure_ad".to_string(),
            source_reference: Some("azure_ad:conditional-access".to_string()),
            data: json!({
                "total_policies": policies.len(),
                "enabled_policies": enabled_policies.len(),
                "disabled_policies": disabled_policies.len(),
                "report_only_policies": report_only_policies.len(),
                "mfa_policies": mfa_policies.len(),
                "policies": policy_details,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.6".to_string(),
            ],
        });

        result.records_created = policies.len() as i32;
        Ok(result)
    }
}
