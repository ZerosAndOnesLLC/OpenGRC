use crate::integrations::okta::client::OktaClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde_json::json;
use std::collections::HashMap;

/// MFA Status Collector for Okta
pub struct MfaCollector;

impl MfaCollector {
    /// Collect MFA status for all users
    pub async fn sync(client: &OktaClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        // Get all users
        let users = client.list_users().await?;
        let active_users: Vec<_> = users.iter().filter(|u| u.status == "ACTIVE").collect();

        if active_users.is_empty() {
            return Ok(result);
        }

        result.records_processed = active_users.len() as i32;

        // Collect MFA factors for each user
        let mut users_with_mfa = Vec::new();
        let mut users_without_mfa = Vec::new();
        let mut factor_type_counts: HashMap<String, i32> = HashMap::new();
        let mut user_mfa_details = Vec::new();

        for user in &active_users {
            match client.list_user_factors(&user.id).await {
                Ok(factors) => {
                    let active_factors: Vec<_> = factors
                        .iter()
                        .filter(|f| f.status == "ACTIVE")
                        .collect();

                    // Count factor types
                    for factor in &active_factors {
                        *factor_type_counts.entry(factor.factor_type.clone()).or_insert(0) += 1;
                    }

                    let factor_details: Vec<_> = active_factors
                        .iter()
                        .map(|f| {
                            json!({
                                "type": f.factor_type,
                                "provider": f.provider,
                                "vendor_name": f.vendor_name,
                                "status": f.status,
                                "created": f.created,
                            })
                        })
                        .collect();

                    let mfa_detail = json!({
                        "user_id": user.id,
                        "login": user.profile.login,
                        "email": user.profile.email,
                        "display_name": user.profile.display_name,
                        "factor_count": active_factors.len(),
                        "factors": factor_details,
                        "has_mfa": !active_factors.is_empty(),
                    });

                    user_mfa_details.push(mfa_detail);

                    if active_factors.is_empty() {
                        users_without_mfa.push(json!({
                            "user_id": user.id,
                            "login": user.profile.login,
                            "email": user.profile.email,
                            "display_name": user.profile.display_name,
                            "department": user.profile.department,
                            "title": user.profile.title,
                        }));
                    } else {
                        users_with_mfa.push(json!({
                            "user_id": user.id,
                            "login": user.profile.login,
                            "email": user.profile.email,
                            "display_name": user.profile.display_name,
                            "factor_count": active_factors.len(),
                            "factor_types": active_factors.iter().map(|f| &f.factor_type).collect::<Vec<_>>(),
                        }));
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        user_id = %user.id,
                        login = %user.profile.login,
                        error = %e,
                        "Failed to get MFA factors for user"
                    );
                    // Treat as unknown/no MFA
                    users_without_mfa.push(json!({
                        "user_id": user.id,
                        "login": user.profile.login,
                        "email": user.profile.email,
                        "display_name": user.profile.display_name,
                        "error": "Failed to retrieve MFA status",
                    }));
                }
            }
        }

        let mfa_coverage_percent = if !active_users.is_empty() {
            (users_with_mfa.len() as f64 / active_users.len() as f64 * 100.0).round() as i32
        } else {
            0
        };

        // Generate MFA Coverage Report
        result.evidence_collected.push(CollectedEvidence {
            title: "Okta MFA Coverage Report".to_string(),
            description: Some(format!(
                "{}% of active users have MFA enabled ({} of {} users)",
                mfa_coverage_percent,
                users_with_mfa.len(),
                active_users.len()
            )),
            evidence_type: "automated".to_string(),
            source: "okta".to_string(),
            source_reference: Some("okta:mfa-coverage".to_string()),
            data: json!({
                "total_active_users": active_users.len(),
                "users_with_mfa": users_with_mfa.len(),
                "users_without_mfa": users_without_mfa.len(),
                "mfa_coverage_percent": mfa_coverage_percent,
                "factor_type_distribution": factor_type_counts,
                "user_mfa_details": user_mfa_details,
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec![
                "CC6.1".to_string(),
                "CC6.6".to_string(),
            ],
        });

        // Generate Users Without MFA Report if any
        if !users_without_mfa.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "Okta Users Without MFA Report".to_string(),
                description: Some(format!(
                    "{} active users do not have MFA enabled",
                    users_without_mfa.len()
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:users-without-mfa".to_string()),
                data: json!({
                    "users_without_mfa_count": users_without_mfa.len(),
                    "users_without_mfa": users_without_mfa,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.6".to_string(),
                ],
            });
        }

        // Generate Factor Type Summary
        if !factor_type_counts.is_empty() {
            let factor_summary: Vec<_> = factor_type_counts
                .iter()
                .map(|(factor_type, count)| {
                    json!({
                        "factor_type": factor_type,
                        "count": count,
                        "description": describe_factor_type(factor_type),
                    })
                })
                .collect();

            result.evidence_collected.push(CollectedEvidence {
                title: "Okta MFA Factor Types Report".to_string(),
                description: Some(format!(
                    "{} different MFA factor types in use",
                    factor_type_counts.len()
                )),
                evidence_type: "automated".to_string(),
                source: "okta".to_string(),
                source_reference: Some("okta:mfa-factor-types".to_string()),
                data: json!({
                    "factor_type_count": factor_type_counts.len(),
                    "factor_types": factor_summary,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec![
                    "CC6.1".to_string(),
                    "CC6.6".to_string(),
                ],
            });
        }

        result.records_created = active_users.len() as i32;
        Ok(result)
    }
}

/// Get a human-readable description for a factor type
fn describe_factor_type(factor_type: &str) -> &'static str {
    match factor_type {
        "push" | "okta_push" => "Push notification (Okta Verify)",
        "sms" => "SMS text message",
        "call" => "Voice call",
        "email" => "Email verification",
        "question" => "Security question",
        "token" | "token:software:totp" => "Software TOTP token",
        "token:hardware" => "Hardware token",
        "webauthn" => "WebAuthn/FIDO2 (biometric or security key)",
        "u2f" => "U2F security key",
        "hotp" => "HOTP one-time password",
        "signed_nonce" => "Okta Verify with signed nonce",
        _ => "Other factor type",
    }
}
