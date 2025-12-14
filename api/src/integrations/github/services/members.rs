use crate::integrations::github::client::GitHubClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;

/// Organization Members Collector for GitHub
pub struct MembersCollector;

impl MembersCollector {
    /// Collect organization member data from GitHub
    pub async fn sync(
        client: &GitHubClient,
        org: &str,
        _context: &SyncContext,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get all organization members
        let members = client.list_org_members(org).await?;
        result.records_processed = members.len() as i32;

        if members.is_empty() {
            return Ok(result);
        }

        // Get membership details (role) for each member
        let mut admins = Vec::new();
        let mut regular_members = Vec::new();
        let mut membership_details = Vec::new();

        for member in &members {
            match client.get_org_membership(org, &member.login).await {
                Ok(membership) => {
                    let is_admin = membership.role == "admin";
                    if is_admin {
                        admins.push(member.login.clone());
                    } else {
                        regular_members.push(member.login.clone());
                    }

                    membership_details.push(json!({
                        "login": member.login,
                        "id": member.id,
                        "avatar_url": member.avatar_url,
                        "html_url": member.html_url,
                        "type": member.user_type,
                        "site_admin": member.site_admin,
                        "role": membership.role,
                        "state": membership.state,
                    }));
                }
                Err(e) => {
                    tracing::warn!(
                        member = %member.login,
                        error = %e,
                        "Failed to get membership details"
                    );
                    // Still include basic member info
                    membership_details.push(json!({
                        "login": member.login,
                        "id": member.id,
                        "avatar_url": member.avatar_url,
                        "html_url": member.html_url,
                        "type": member.user_type,
                        "site_admin": member.site_admin,
                        "role": "unknown",
                        "state": "unknown",
                    }));
                }
            }
        }

        // Generate organization members inventory evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "GitHub Organization Members Report".to_string(),
            description: Some(format!(
                "Organization has {} members ({} admins, {} regular members)",
                members.len(),
                admins.len(),
                regular_members.len()
            )),
            evidence_type: "automated".to_string(),
            source: "github".to_string(),
            source_reference: Some("github:org-members".to_string()),
            data: json!({
                "organization": org,
                "total_members": members.len(),
                "admin_count": admins.len(),
                "member_count": regular_members.len(),
                "admins": admins,
                "members": membership_details,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.2".to_string(),
                "CC6.3".to_string(),
            ],
        });

        // Generate admin access report if there are admins
        if !admins.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "GitHub Organization Admin Access Report".to_string(),
                description: Some(format!(
                    "{} users have admin access to the organization",
                    admins.len()
                )),
                evidence_type: "automated".to_string(),
                source: "github".to_string(),
                source_reference: Some("github:org-admins".to_string()),
                data: json!({
                    "organization": org,
                    "admin_count": admins.len(),
                    "admin_users": admins,
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
