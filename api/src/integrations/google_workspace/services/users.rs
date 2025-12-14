use crate::integrations::google_workspace::client::GoogleWorkspaceClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;

/// User Directory Collector for Google Workspace
pub struct UsersCollector;

impl UsersCollector {
    /// Collect user directory data from Google Workspace
    pub async fn sync(client: &GoogleWorkspaceClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get all users
        let users = client.list_users().await?;
        result.records_processed = users.len() as i32;

        if users.is_empty() {
            return Ok(result);
        }

        // Categorize users
        let active_users: Vec<_> = users.iter().filter(|u| !u.suspended.unwrap_or(false) && !u.archived.unwrap_or(false)).collect();
        let suspended_users: Vec<_> = users.iter().filter(|u| u.suspended.unwrap_or(false)).collect();
        let archived_users: Vec<_> = users.iter().filter(|u| u.archived.unwrap_or(false)).collect();
        let admin_users: Vec<_> = users.iter().filter(|u| u.is_admin.unwrap_or(false)).collect();
        let delegated_admins: Vec<_> = users.iter().filter(|u| u.is_delegated_admin.unwrap_or(false)).collect();

        // 2-Step Verification stats
        let enrolled_2sv: Vec<_> = users.iter().filter(|u| u.is_enrolled_in_2sv.unwrap_or(false)).collect();
        let enforced_2sv: Vec<_> = users.iter().filter(|u| u.is_enforced_in_2sv.unwrap_or(false)).collect();
        let not_enrolled_2sv: Vec<_> = active_users.iter().filter(|u| !u.is_enrolled_in_2sv.unwrap_or(false)).collect();

        // Build user details
        let user_details: Vec<_> = users
            .iter()
            .map(|u| {
                json!({
                    "id": u.id,
                    "primary_email": u.primary_email,
                    "full_name": u.name.full_name,
                    "given_name": u.name.given_name,
                    "family_name": u.name.family_name,
                    "is_admin": u.is_admin,
                    "is_delegated_admin": u.is_delegated_admin,
                    "is_enrolled_in_2sv": u.is_enrolled_in_2sv,
                    "is_enforced_in_2sv": u.is_enforced_in_2sv,
                    "suspended": u.suspended,
                    "archived": u.archived,
                    "creation_time": u.creation_time,
                    "last_login_time": u.last_login_time,
                    "org_unit_path": u.org_unit_path,
                    "agreed_to_terms": u.agreed_to_terms,
                })
            })
            .collect();

        // Generate User Inventory Evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Google Workspace User Directory Report".to_string(),
            description: Some(format!(
                "Google Workspace directory contains {} total users ({} active, {} suspended, {} archived)",
                users.len(),
                active_users.len(),
                suspended_users.len(),
                archived_users.len()
            )),
            evidence_type: "automated".to_string(),
            source: "google_workspace".to_string(),
            source_reference: Some("google_workspace:user-directory".to_string()),
            data: json!({
                "total_users": users.len(),
                "active_users": active_users.len(),
                "suspended_users": suspended_users.len(),
                "archived_users": archived_users.len(),
                "admin_users": admin_users.len(),
                "delegated_admins": delegated_admins.len(),
                "users": user_details,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.2".to_string(),
                "CC6.3".to_string(),
            ],
        });

        // Generate Admin Users Report
        if !admin_users.is_empty() {
            let admin_details: Vec<_> = admin_users
                .iter()
                .map(|u| {
                    json!({
                        "id": u.id,
                        "primary_email": u.primary_email,
                        "full_name": u.name.full_name,
                        "is_admin": u.is_admin,
                        "is_delegated_admin": u.is_delegated_admin,
                        "is_enrolled_in_2sv": u.is_enrolled_in_2sv,
                        "last_login_time": u.last_login_time,
                    })
                })
                .collect();

            result.evidence_collected.push(CollectedEvidence {
                title: "Google Workspace Admin Users Report".to_string(),
                description: Some(format!(
                    "{} users have admin privileges ({} super admins, {} delegated admins)",
                    admin_users.len() + delegated_admins.len(),
                    admin_users.len(),
                    delegated_admins.len()
                )),
                evidence_type: "automated".to_string(),
                source: "google_workspace".to_string(),
                source_reference: Some("google_workspace:admin-users".to_string()),
                data: json!({
                    "admin_count": admin_users.len(),
                    "delegated_admin_count": delegated_admins.len(),
                    "admin_users": admin_details,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.2".to_string(),
                    "CC6.3".to_string(),
                ],
            });
        }

        // Generate 2-Step Verification Report
        let total_active = active_users.len();
        let enrolled_count = enrolled_2sv.len();
        let coverage_percent = if total_active > 0 {
            (enrolled_count as f64 / total_active as f64 * 100.0).round() as i32
        } else {
            0
        };

        result.evidence_collected.push(CollectedEvidence {
            title: "Google Workspace 2-Step Verification Report".to_string(),
            description: Some(format!(
                "{}% of active users have 2-Step Verification enrolled ({} of {})",
                coverage_percent,
                enrolled_count,
                total_active
            )),
            evidence_type: "automated".to_string(),
            source: "google_workspace".to_string(),
            source_reference: Some("google_workspace:2sv-status".to_string()),
            data: json!({
                "total_active_users": total_active,
                "enrolled_in_2sv": enrolled_count,
                "enforced_2sv": enforced_2sv.len(),
                "not_enrolled_2sv": not_enrolled_2sv.len(),
                "coverage_percent": coverage_percent,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.6".to_string(),
            ],
        });

        // Generate Users Without 2SV Report if any
        if !not_enrolled_2sv.is_empty() {
            let not_enrolled_details: Vec<_> = not_enrolled_2sv
                .iter()
                .map(|u| {
                    json!({
                        "id": u.id,
                        "primary_email": u.primary_email,
                        "full_name": u.name.full_name,
                        "is_admin": u.is_admin,
                        "last_login_time": u.last_login_time,
                        "org_unit_path": u.org_unit_path,
                    })
                })
                .collect();

            result.evidence_collected.push(CollectedEvidence {
                title: "Google Workspace Users Without 2-Step Verification".to_string(),
                description: Some(format!(
                    "{} active users do not have 2-Step Verification enrolled",
                    not_enrolled_2sv.len()
                )),
                evidence_type: "automated".to_string(),
                source: "google_workspace".to_string(),
                source_reference: Some("google_workspace:users-without-2sv".to_string()),
                data: json!({
                    "users_without_2sv_count": not_enrolled_2sv.len(),
                    "users_without_2sv": not_enrolled_details,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.6".to_string(),
                ],
            });
        }

        // Generate stale users report (no login in 90 days)
        let stale_threshold = Utc::now() - chrono::Duration::days(90);
        let stale_users: Vec<_> = active_users
            .iter()
            .filter(|u| {
                u.last_login_time
                    .as_ref()
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
                        "primary_email": u.primary_email,
                        "full_name": u.name.full_name,
                        "last_login_time": u.last_login_time,
                        "creation_time": u.creation_time,
                        "org_unit_path": u.org_unit_path,
                    })
                })
                .collect();

            result.evidence_collected.push(CollectedEvidence {
                title: "Google Workspace Stale User Accounts Report".to_string(),
                description: Some(format!(
                    "{} active users have not logged in within 90 days",
                    stale_users.len()
                )),
                evidence_type: "automated".to_string(),
                source: "google_workspace".to_string(),
                source_reference: Some("google_workspace:stale-users".to_string()),
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

/// Groups Collector for Google Workspace
pub struct GroupsCollector;

impl GroupsCollector {
    /// Collect group data from Google Workspace
    pub async fn sync(client: &GoogleWorkspaceClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get all groups
        let groups = client.list_groups().await?;
        result.records_processed = groups.len() as i32;

        if groups.is_empty() {
            return Ok(result);
        }

        // Build group details with member counts
        let mut group_details = Vec::new();
        for group in &groups {
            let members = client.list_group_members(&group.email).await.unwrap_or_default();

            // Count by role
            let owners = members.iter().filter(|m| m.role.as_deref() == Some("OWNER")).count();
            let managers = members.iter().filter(|m| m.role.as_deref() == Some("MANAGER")).count();
            let regular_members = members.iter().filter(|m| m.role.as_deref() == Some("MEMBER")).count();

            group_details.push(json!({
                "id": group.id,
                "email": group.email,
                "name": group.name,
                "description": group.description,
                "direct_members_count": group.direct_members_count,
                "admin_created": group.admin_created,
                "member_breakdown": {
                    "owners": owners,
                    "managers": managers,
                    "members": regular_members,
                    "total": members.len(),
                }
            }));
        }

        // Generate Groups Inventory Evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Google Workspace Groups Report".to_string(),
            description: Some(format!(
                "Google Workspace has {} groups",
                groups.len()
            )),
            evidence_type: "automated".to_string(),
            source: "google_workspace".to_string(),
            source_reference: Some("google_workspace:groups".to_string()),
            data: json!({
                "total_groups": groups.len(),
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
