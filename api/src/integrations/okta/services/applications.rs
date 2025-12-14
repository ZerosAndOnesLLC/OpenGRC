use crate::integrations::okta::client::OktaClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

/// Application Assignments Collector for Okta
pub struct ApplicationsCollector;

impl ApplicationsCollector {
    /// Collect application assignment data from Okta
    pub async fn sync(client: &OktaClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get all applications
        let apps = client.list_applications().await?;
        result.records_processed = apps.len() as i32;

        if apps.is_empty() {
            return Ok(result);
        }

        // Categorize apps by status
        let active_apps: Vec<_> = apps.iter().filter(|a| a.status == "ACTIVE").collect();
        let inactive_apps: Vec<_> = apps.iter().filter(|a| a.status == "INACTIVE").collect();

        // Categorize by sign-on mode
        let mut sign_on_mode_counts: HashMap<String, i32> = HashMap::new();
        for app in &apps {
            if let Some(ref mode) = app.sign_on_mode {
                *sign_on_mode_counts.entry(mode.clone()).or_insert(0) += 1;
            }
        }

        // Build app details with user counts
        let mut app_details = Vec::new();
        let mut saml_apps = Vec::new();
        let mut oidc_apps = Vec::new();
        let mut bookmark_apps = Vec::new();

        for app in &apps {
            // Get user count for each app
            let app_users = client.list_app_users(&app.id).await.unwrap_or_default();
            let active_user_count = app_users.iter().filter(|u| u.status == "ACTIVE").count();

            let app_detail = json!({
                "id": app.id,
                "name": app.name,
                "label": app.label,
                "status": app.status,
                "sign_on_mode": app.sign_on_mode,
                "created": app.created,
                "last_updated": app.last_updated,
                "total_users": app_users.len(),
                "active_users": active_user_count,
                "features": app.features,
            });

            app_details.push(app_detail.clone());

            // Categorize by sign-on mode
            match app.sign_on_mode.as_deref() {
                Some("SAML_2_0") | Some("SAML_1_1") => saml_apps.push(app_detail),
                Some("OPENID_CONNECT") => oidc_apps.push(app_detail),
                Some("BOOKMARK") => bookmark_apps.push(app_detail),
                _ => {}
            }
        }

        // Generate Application Inventory Evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "Okta Application Inventory Report".to_string(),
            description: Some(format!(
                "Okta has {} applications ({} active, {} inactive)",
                apps.len(),
                active_apps.len(),
                inactive_apps.len()
            )),
            evidence_type: "automated".to_string(),
            source: "okta".to_string(),
            source_reference: Some("okta:applications".to_string()),
            data: json!({
                "total_applications": apps.len(),
                "active_applications": active_apps.len(),
                "inactive_applications": inactive_apps.len(),
                "sign_on_mode_distribution": sign_on_mode_counts,
                "applications": app_details,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.2".to_string(),
                "CC6.3".to_string(),
            ],
        });

        // Generate SAML Applications Report (SSO apps)
        if !saml_apps.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Okta SAML SSO Applications Report".to_string(),
                description: Some(format!(
                    "{} applications using SAML single sign-on",
                    saml_apps.len()
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:saml-applications".to_string()),
                data: json!({
                    "saml_application_count": saml_apps.len(),
                    "saml_applications": saml_apps,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.6".to_string(),
                ],
            });
        }

        // Generate OIDC Applications Report
        if !oidc_apps.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Okta OpenID Connect Applications Report".to_string(),
                description: Some(format!(
                    "{} applications using OpenID Connect",
                    oidc_apps.len()
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:oidc-applications".to_string()),
                data: json!({
                    "oidc_application_count": oidc_apps.len(),
                    "oidc_applications": oidc_apps,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.6".to_string(),
                ],
            });
        }

        // Generate apps with high user counts (potential high-risk)
        let high_user_apps: Vec<_> = app_details
            .iter()
            .filter(|a| {
                a.get("active_users")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0)
                    > 10
            })
            .cloned()
            .collect();

        if !high_user_apps.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Okta High-Usage Applications Report".to_string(),
                description: Some(format!(
                    "{} applications have more than 10 active users",
                    high_user_apps.len()
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:high-usage-applications".to_string()),
                data: json!({
                    "high_usage_application_count": high_user_apps.len(),
                    "high_usage_applications": high_user_apps,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.2".to_string(),
                ],
            });
        }

        result.records_created = apps.len() as i32;
        Ok(result)
    }
}
