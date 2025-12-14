use crate::integrations::okta::client::OktaClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;

/// User Directory Collector for Okta
pub struct UsersCollector;

impl UsersCollector {
    /// Collect user directory data from Okta
    pub async fn sync(client: &OktaClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get all users
        let users = client.list_users().await?;
        result.records_processed = users.len() as i32;

        if users.is_empty() {
            return Ok(result);
        }

        // Categorize users by status
        let active_users: Vec<_> = users.iter().filter(|u| u.status == "ACTIVE").collect();
        let suspended_users: Vec<_> = users.iter().filter(|u| u.status == "SUSPENDED").collect();
        let deprovisioned_users: Vec<_> = users.iter().filter(|u| u.status == "DEPROVISIONED").collect();
        let staged_users: Vec<_> = users.iter().filter(|u| u.status == "STAGED").collect();
        let provisioned_users: Vec<_> = users.iter().filter(|u| u.status == "PROVISIONED").collect();
        let recovery_users: Vec<_> = users.iter().filter(|u| u.status == "RECOVERY").collect();
        let locked_users: Vec<_> = users.iter().filter(|u| u.status == "LOCKED_OUT").collect();

        // Build user details
        let user_details: Vec<_> = users
            .iter()
            .map(|u| {
                json!({
                    "id": u.id,
                    "login": u.profile.login,
                    "email": u.profile.email,
                    "first_name": u.profile.first_name,
                    "last_name": u.profile.last_name,
                    "display_name": u.profile.display_name,
                    "status": u.status,
                    "created": u.created,
                    "last_login": u.last_login,
                    "last_updated": u.last_updated,
                    "password_changed": u.password_changed,
                    "department": u.profile.department,
                    "title": u.profile.title,
                    "manager": u.profile.manager,
                    "employee_number": u.profile.employee_number,
                })
            })
            .collect();

        // Generate User Inventory Evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Okta User Directory Report".to_string(),
            description: Some(format!(
                "Okta directory contains {} total users ({} active, {} suspended, {} locked out)",
                users.len(),
                active_users.len(),
                suspended_users.len(),
                locked_users.len()
            )),
            evidence_type: "automated".to_string(),
            source: "okta".to_string(),
            source_reference: Some("okta:user-directory".to_string()),
            data: json!({
                "total_users": users.len(),
                "active_users": active_users.len(),
                "suspended_users": suspended_users.len(),
                "deprovisioned_users": deprovisioned_users.len(),
                "staged_users": staged_users.len(),
                "provisioned_users": provisioned_users.len(),
                "recovery_users": recovery_users.len(),
                "locked_users": locked_users.len(),
                "users": user_details,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.2".to_string(),
                "CC6.3".to_string(),
            ],
        });

        // Generate locked out users alert if any
        if !locked_users.is_empty() {
            let locked_details: Vec<_> = locked_users
                .iter()
                .map(|u| {
                    json!({
                        "id": u.id,
                        "login": u.profile.login,
                        "email": u.profile.email,
                        "display_name": u.profile.display_name,
                        "status_changed": u.status_changed,
                    })
                })
                .collect();

            result.evidence_collected.push(CollectedEvidence {
                title: "Okta Locked Out Users Report".to_string(),
                description: Some(format!(
                    "{} users are currently locked out",
                    locked_users.len()
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:locked-users".to_string()),
                data: json!({
                    "locked_user_count": locked_users.len(),
                    "locked_users": locked_details,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC7.2".to_string(),
                ],
            });
        }

        // Generate stale user report (users who haven't logged in recently)
        let stale_threshold = Utc::now() - chrono::Duration::days(90);
        let stale_users: Vec<_> = users
            .iter()
            .filter(|u| {
                u.status == "ACTIVE"
                    && u.last_login
                        .as_ref()
                        .map(|l| {
                            chrono::DateTime::parse_from_rfc3339(l)
                                .map(|dt| dt < stale_threshold)
                                .unwrap_or(true)
                        })
                        .unwrap_or(true) // No login = stale
            })
            .collect();

        if !stale_users.is_empty() {
            let stale_details: Vec<_> = stale_users
                .iter()
                .map(|u| {
                    json!({
                        "id": u.id,
                        "login": u.profile.login,
                        "email": u.profile.email,
                        "display_name": u.profile.display_name,
                        "last_login": u.last_login,
                        "created": u.created,
                    })
                })
                .collect();

            result.evidence_collected.push(CollectedEvidence {
                title: "Okta Stale User Accounts Report".to_string(),
                description: Some(format!(
                    "{} active users have not logged in within 90 days",
                    stale_users.len()
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:stale-users".to_string()),
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

/// Groups Collector for Okta
pub struct GroupsCollector;

impl GroupsCollector {
    /// Collect group data from Okta
    pub async fn sync(client: &OktaClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get all groups
        let groups = client.list_groups().await?;
        result.records_processed = groups.len() as i32;

        if groups.is_empty() {
            return Ok(result);
        }

        // Categorize groups by type
        let okta_groups: Vec<_> = groups.iter().filter(|g| g.group_type == "OKTA_GROUP").collect();
        let built_in_groups: Vec<_> = groups.iter().filter(|g| g.group_type == "BUILT_IN").collect();
        let app_groups: Vec<_> = groups.iter().filter(|g| g.group_type == "APP_GROUP").collect();

        // Build group details with member counts
        let mut group_details = Vec::new();
        for group in &groups {
            let members = client.list_group_members(&group.id).await.unwrap_or_default();
            group_details.push(json!({
                "id": group.id,
                "name": group.profile.name,
                "description": group.profile.description,
                "type": group.group_type,
                "member_count": members.len(),
                "created": group.created,
                "last_updated": group.last_updated,
                "last_membership_updated": group.last_membership_updated,
            }));
        }

        // Generate Groups Inventory Evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Okta Groups Report".to_string(),
            description: Some(format!(
                "Okta directory contains {} groups ({} custom, {} built-in, {} app groups)",
                groups.len(),
                okta_groups.len(),
                built_in_groups.len(),
                app_groups.len()
            )),
            evidence_type: "automated".to_string(),
            source: "okta".to_string(),
            source_reference: Some("okta:groups".to_string()),
            data: json!({
                "total_groups": groups.len(),
                "okta_groups": okta_groups.len(),
                "built_in_groups": built_in_groups.len(),
                "app_groups": app_groups.len(),
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
