use crate::integrations::aws::client::AwsClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// IAM User data collected from AWS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsIamUser {
    pub user_id: String,
    pub user_name: String,
    pub arn: String,
    pub path: String,
    pub create_date: Option<DateTime<Utc>>,
    pub password_last_used: Option<DateTime<Utc>>,
    pub mfa_enabled: bool,
    pub mfa_devices: Vec<AwsMfaDevice>,
    pub access_keys: Vec<AwsAccessKey>,
    pub groups: Vec<String>,
    pub attached_policies: Vec<AwsAttachedPolicy>,
    pub inline_policy_names: Vec<String>,
    pub tags: HashMap<String, String>,
}

/// MFA device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsMfaDevice {
    pub serial_number: String,
    pub enable_date: Option<DateTime<Utc>>,
}

/// Access key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsAccessKey {
    pub access_key_id: String,
    pub status: String, // Active, Inactive
    pub create_date: Option<DateTime<Utc>>,
    pub last_used_date: Option<DateTime<Utc>>,
    pub last_used_service: Option<String>,
    pub last_used_region: Option<String>,
}

/// Attached policy information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsAttachedPolicy {
    pub policy_arn: String,
    pub policy_name: String,
}

/// IAM Role data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsIamRole {
    pub role_id: String,
    pub role_name: String,
    pub arn: String,
    pub path: String,
    pub assume_role_policy_document: Option<serde_json::Value>,
    pub description: Option<String>,
    pub max_session_duration: i32,
    pub create_date: Option<DateTime<Utc>>,
    pub attached_policies: Vec<AwsAttachedPolicy>,
    pub inline_policy_names: Vec<String>,
    pub last_used_date: Option<DateTime<Utc>>,
    pub last_used_region: Option<String>,
    pub tags: HashMap<String, String>,
}

/// IAM Policy data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsIamPolicy {
    pub policy_id: String,
    pub policy_name: String,
    pub arn: String,
    pub path: String,
    pub default_version_id: String,
    pub attachment_count: i32,
    pub is_attachable: bool,
    pub create_date: Option<DateTime<Utc>>,
    pub update_date: Option<DateTime<Utc>>,
    pub policy_document: Option<serde_json::Value>,
    pub description: Option<String>,
    // Risk analysis
    pub allows_admin_access: bool,
    pub uses_wildcard_actions: bool,
    pub uses_wildcard_resources: bool,
    pub risk_score: i32,
    pub tags: HashMap<String, String>,
}

/// IAM Collector for syncing IAM data
pub struct IamCollector;

impl IamCollector {
    /// Sync IAM data from AWS
    pub async fn sync(client: &AwsClient, _context: &SyncContext) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();
        let iam_client = client.iam_client();

        // Collect users
        let users = Self::collect_users(&iam_client).await?;
        result.records_processed += users.len() as i32;

        // Analyze users for compliance evidence
        let users_without_mfa: Vec<_> = users.iter().filter(|u| !u.mfa_enabled).collect();
        let users_with_old_keys: Vec<_> = users
            .iter()
            .filter(|u| {
                u.access_keys.iter().any(|k| {
                    k.status == "Active"
                        && k.create_date.map_or(false, |d| {
                            (Utc::now() - d).num_days() > 90
                        })
                })
            })
            .collect();

        // Generate MFA compliance evidence
        if !users.is_empty() {
            let mfa_compliance_rate =
                (users.len() - users_without_mfa.len()) as f64 / users.len() as f64 * 100.0;

            result.evidence_collected.push(CollectedEvidence {
                title: "IAM User MFA Compliance Report".to_string(),
                description: Some(format!(
                    "MFA is enabled for {:.1}% of IAM users ({} of {})",
                    mfa_compliance_rate,
                    users.len() - users_without_mfa.len(),
                    users.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some("iam:users".to_string()),
                data: json!({
                    "total_users": users.len(),
                    "users_with_mfa": users.len() - users_without_mfa.len(),
                    "users_without_mfa": users_without_mfa.iter().map(|u| &u.user_name).collect::<Vec<_>>(),
                    "compliance_rate": mfa_compliance_rate,
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC6.1".to_string(), "CC6.6".to_string()],
            });
        }

        // Generate access key rotation evidence
        if !users_with_old_keys.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "IAM Access Key Rotation Report".to_string(),
                description: Some(format!(
                    "{} users have access keys older than 90 days",
                    users_with_old_keys.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some("iam:access-keys".to_string()),
                data: json!({
                    "users_with_old_keys": users_with_old_keys.iter().map(|u| {
                        json!({
                            "user_name": u.user_name,
                            "access_keys": u.access_keys.iter().filter(|k| k.status == "Active").map(|k| {
                                json!({
                                    "access_key_id": k.access_key_id,
                                    "create_date": k.create_date,
                                    "age_days": k.create_date.map(|d| (Utc::now() - d).num_days())
                                })
                            }).collect::<Vec<_>>()
                        })
                    }).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC6.2".to_string()],
            });
        }

        // Collect roles
        let roles = Self::collect_roles(&iam_client).await?;
        result.records_processed += roles.len() as i32;

        // Generate roles inventory evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "IAM Roles Inventory".to_string(),
            description: Some(format!("Inventory of {} IAM roles", roles.len())),
            evidence_type: "automated".to_string(),
            source: "aws".to_string(),
            source_reference: Some("iam:roles".to_string()),
            data: json!({
                "total_roles": roles.len(),
                "roles": roles.iter().map(|r| json!({
                    "role_name": r.role_name,
                    "arn": r.arn,
                    "create_date": r.create_date,
                    "last_used": r.last_used_date,
                    "attached_policies": r.attached_policies.len(),
                })).collect::<Vec<_>>(),
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec!["CC6.1".to_string(), "CC6.3".to_string()],
        });

        // Collect policies
        let policies = Self::collect_policies(&iam_client).await?;
        result.records_processed += policies.len() as i32;

        // Identify risky policies
        let risky_policies: Vec<_> = policies.iter().filter(|p| p.risk_score > 50).collect();

        if !risky_policies.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: "IAM Policies Risk Analysis".to_string(),
                description: Some(format!(
                    "{} policies with elevated risk scores identified",
                    risky_policies.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some("iam:policies".to_string()),
                data: json!({
                    "risky_policies": risky_policies.iter().map(|p| json!({
                        "policy_name": p.policy_name,
                        "arn": p.arn,
                        "risk_score": p.risk_score,
                        "allows_admin": p.allows_admin_access,
                        "wildcard_actions": p.uses_wildcard_actions,
                        "wildcard_resources": p.uses_wildcard_resources,
                    })).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC6.1".to_string(), "CC6.3".to_string()],
            });
        }

        // Generate user inventory evidence
        result.evidence_collected.push(CollectedEvidence {
            title: "IAM Users Inventory".to_string(),
            description: Some(format!("Complete inventory of {} IAM users", users.len())),
            evidence_type: "automated".to_string(),
            source: "aws".to_string(),
            source_reference: Some("iam:users".to_string()),
            data: json!({
                "total_users": users.len(),
                "users": users.iter().map(|u| json!({
                    "user_name": u.user_name,
                    "arn": u.arn,
                    "create_date": u.create_date,
                    "mfa_enabled": u.mfa_enabled,
                    "access_keys_count": u.access_keys.len(),
                    "groups": u.groups,
                    "attached_policies": u.attached_policies.len(),
                })).collect::<Vec<_>>(),
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec!["CC6.1".to_string(), "CC6.2".to_string()],
        });

        result.records_created = result.records_processed;
        Ok(result)
    }

    /// Collect all IAM users with their details
    async fn collect_users(
        iam_client: &aws_sdk_iam::Client,
    ) -> Result<Vec<AwsIamUser>, String> {
        let mut users = Vec::new();
        let mut marker = None;

        loop {
            let mut request = iam_client.list_users();
            if let Some(m) = &marker {
                request = request.marker(m);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format!("Failed to list IAM users: {}", e))?;

            for user in response.users() {
                let user_name = user.user_name().to_string();

                // Get MFA devices
                let mfa_devices = Self::get_user_mfa_devices(iam_client, &user_name).await?;

                // Get access keys
                let access_keys = Self::get_user_access_keys(iam_client, &user_name).await?;

                // Get groups
                let groups = Self::get_user_groups(iam_client, &user_name).await?;

                // Get attached policies
                let attached_policies =
                    Self::get_user_attached_policies(iam_client, &user_name).await?;

                // Get inline policy names
                let inline_policy_names =
                    Self::get_user_inline_policies(iam_client, &user_name).await?;

                // Get tags
                let tags = Self::get_user_tags(iam_client, &user_name).await?;

                let create_date = {
                    let d = user.create_date();
                    DateTime::from_timestamp(d.secs(), d.subsec_nanos())
                };
                let password_last_used = user.password_last_used().and_then(|d| {
                    DateTime::from_timestamp(d.secs(), d.subsec_nanos())
                });

                users.push(AwsIamUser {
                    user_id: user.user_id().to_string(),
                    user_name: user_name.clone(),
                    arn: user.arn().to_string(),
                    path: user.path().to_string(),
                    create_date,
                    password_last_used,
                    mfa_enabled: !mfa_devices.is_empty(),
                    mfa_devices,
                    access_keys,
                    groups,
                    attached_policies,
                    inline_policy_names,
                    tags,
                });
            }

            if response.is_truncated() {
                marker = response.marker().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(users)
    }

    async fn get_user_mfa_devices(
        iam_client: &aws_sdk_iam::Client,
        user_name: &str,
    ) -> Result<Vec<AwsMfaDevice>, String> {
        let response = iam_client
            .list_mfa_devices()
            .user_name(user_name)
            .send()
            .await
            .map_err(|e| format!("Failed to list MFA devices for {}: {}", user_name, e))?;

        Ok(response
            .mfa_devices()
            .iter()
            .map(|d| {
                let enable_date = {
                    let dt = d.enable_date();
                    DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
                };
                AwsMfaDevice {
                    serial_number: d.serial_number().to_string(),
                    enable_date,
                }
            })
            .collect())
    }

    async fn get_user_access_keys(
        iam_client: &aws_sdk_iam::Client,
        user_name: &str,
    ) -> Result<Vec<AwsAccessKey>, String> {
        let response = iam_client
            .list_access_keys()
            .user_name(user_name)
            .send()
            .await
            .map_err(|e| format!("Failed to list access keys for {}: {}", user_name, e))?;

        let mut access_keys = Vec::new();

        for key_meta in response.access_key_metadata() {
            let access_key_id = key_meta.access_key_id().unwrap_or_default().to_string();

            // Get last used info
            let last_used = iam_client
                .get_access_key_last_used()
                .access_key_id(&access_key_id)
                .send()
                .await
                .ok();

            let (last_used_date, last_used_service, last_used_region) =
                if let Some(lu) = last_used.and_then(|r| r.access_key_last_used) {
                    let date = lu.last_used_date().and_then(|dt| {
                        DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
                    });
                    let service = Some(lu.service_name().to_string());
                    let region = Some(lu.region().to_string());
                    (date, service, region)
                } else {
                    (None, None, None)
                };

            let create_date = key_meta.create_date().and_then(|dt| {
                DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
            });

            access_keys.push(AwsAccessKey {
                access_key_id,
                status: key_meta.status().map(|s| s.as_str()).unwrap_or("Unknown").to_string(),
                create_date,
                last_used_date,
                last_used_service,
                last_used_region,
            });
        }

        Ok(access_keys)
    }

    async fn get_user_groups(
        iam_client: &aws_sdk_iam::Client,
        user_name: &str,
    ) -> Result<Vec<String>, String> {
        let response = iam_client
            .list_groups_for_user()
            .user_name(user_name)
            .send()
            .await
            .map_err(|e| format!("Failed to list groups for {}: {}", user_name, e))?;

        Ok(response
            .groups()
            .iter()
            .map(|g| g.group_name().to_string())
            .collect())
    }

    async fn get_user_attached_policies(
        iam_client: &aws_sdk_iam::Client,
        user_name: &str,
    ) -> Result<Vec<AwsAttachedPolicy>, String> {
        let response = iam_client
            .list_attached_user_policies()
            .user_name(user_name)
            .send()
            .await
            .map_err(|e| format!("Failed to list attached policies for {}: {}", user_name, e))?;

        Ok(response
            .attached_policies()
            .iter()
            .map(|p| AwsAttachedPolicy {
                policy_arn: p.policy_arn().unwrap_or_default().to_string(),
                policy_name: p.policy_name().unwrap_or_default().to_string(),
            })
            .collect())
    }

    async fn get_user_inline_policies(
        iam_client: &aws_sdk_iam::Client,
        user_name: &str,
    ) -> Result<Vec<String>, String> {
        let response = iam_client
            .list_user_policies()
            .user_name(user_name)
            .send()
            .await
            .map_err(|e| format!("Failed to list inline policies for {}: {}", user_name, e))?;

        Ok(response.policy_names().to_vec())
    }

    async fn get_user_tags(
        iam_client: &aws_sdk_iam::Client,
        user_name: &str,
    ) -> Result<HashMap<String, String>, String> {
        let response = iam_client
            .list_user_tags()
            .user_name(user_name)
            .send()
            .await
            .map_err(|e| format!("Failed to list tags for {}: {}", user_name, e))?;

        Ok(response
            .tags()
            .iter()
            .map(|t| (t.key().to_string(), t.value().to_string()))
            .collect())
    }

    /// Collect all IAM roles
    async fn collect_roles(
        iam_client: &aws_sdk_iam::Client,
    ) -> Result<Vec<AwsIamRole>, String> {
        let mut roles = Vec::new();
        let mut marker = None;

        loop {
            let mut request = iam_client.list_roles();
            if let Some(m) = &marker {
                request = request.marker(m);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format!("Failed to list IAM roles: {}", e))?;

            for role in response.roles() {
                let role_name = role.role_name().to_string();

                // Get attached policies
                let attached_policies =
                    Self::get_role_attached_policies(iam_client, &role_name).await?;

                // Get inline policy names
                let inline_policy_names =
                    Self::get_role_inline_policies(iam_client, &role_name).await?;

                // Get role last used
                let (last_used_date, last_used_region) = role
                    .role_last_used()
                    .map(|lu| {
                        (
                            lu.last_used_date().map(|dt| {
                                DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
                                    .unwrap_or_else(Utc::now)
                            }),
                            lu.region().map(|s| s.to_string()),
                        )
                    })
                    .unwrap_or((None, None));

                // Get tags
                let tags = Self::get_role_tags(iam_client, &role_name).await?;

                // Parse assume role policy
                let assume_role_policy_document = role
                    .assume_role_policy_document()
                    .and_then(|doc| {
                        urlencoding::decode(doc)
                            .ok()
                            .and_then(|decoded| serde_json::from_str(&decoded).ok())
                    });

                let create_date = {
                    let d = role.create_date();
                    DateTime::from_timestamp(d.secs(), d.subsec_nanos())
                };

                roles.push(AwsIamRole {
                    role_id: role.role_id().to_string(),
                    role_name: role_name.clone(),
                    arn: role.arn().to_string(),
                    path: role.path().to_string(),
                    assume_role_policy_document,
                    description: role.description().map(|s| s.to_string()),
                    max_session_duration: role.max_session_duration().unwrap_or(3600),
                    create_date,
                    attached_policies,
                    inline_policy_names,
                    last_used_date,
                    last_used_region,
                    tags,
                });
            }

            if response.is_truncated() {
                marker = response.marker().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(roles)
    }

    async fn get_role_attached_policies(
        iam_client: &aws_sdk_iam::Client,
        role_name: &str,
    ) -> Result<Vec<AwsAttachedPolicy>, String> {
        let response = iam_client
            .list_attached_role_policies()
            .role_name(role_name)
            .send()
            .await
            .map_err(|e| format!("Failed to list attached policies for role {}: {}", role_name, e))?;

        Ok(response
            .attached_policies()
            .iter()
            .map(|p| AwsAttachedPolicy {
                policy_arn: p.policy_arn().unwrap_or_default().to_string(),
                policy_name: p.policy_name().unwrap_or_default().to_string(),
            })
            .collect())
    }

    async fn get_role_inline_policies(
        iam_client: &aws_sdk_iam::Client,
        role_name: &str,
    ) -> Result<Vec<String>, String> {
        let response = iam_client
            .list_role_policies()
            .role_name(role_name)
            .send()
            .await
            .map_err(|e| format!("Failed to list inline policies for role {}: {}", role_name, e))?;

        Ok(response.policy_names().to_vec())
    }

    async fn get_role_tags(
        iam_client: &aws_sdk_iam::Client,
        role_name: &str,
    ) -> Result<HashMap<String, String>, String> {
        let response = iam_client
            .list_role_tags()
            .role_name(role_name)
            .send()
            .await
            .map_err(|e| format!("Failed to list tags for role {}: {}", role_name, e))?;

        Ok(response
            .tags()
            .iter()
            .map(|t| (t.key().to_string(), t.value().to_string()))
            .collect())
    }

    /// Collect all customer-managed IAM policies
    async fn collect_policies(
        iam_client: &aws_sdk_iam::Client,
    ) -> Result<Vec<AwsIamPolicy>, String> {
        let mut policies = Vec::new();
        let mut marker = None;

        loop {
            let mut request = iam_client
                .list_policies()
                .scope(aws_sdk_iam::types::PolicyScopeType::Local); // Only customer-managed

            if let Some(m) = &marker {
                request = request.marker(m);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format!("Failed to list IAM policies: {}", e))?;

            for policy in response.policies() {
                let policy_arn = policy.arn().unwrap_or_default().to_string();
                let default_version_id = policy
                    .default_version_id()
                    .unwrap_or("v1")
                    .to_string();

                // Get policy document
                let policy_doc = Self::get_policy_document(
                    iam_client,
                    &policy_arn,
                    &default_version_id,
                )
                .await?;

                // Analyze policy for risks
                let (allows_admin, wildcard_actions, wildcard_resources, risk_score) =
                    Self::analyze_policy_risk(&policy_doc);

                // Get tags
                let tags = Self::get_policy_tags(iam_client, &policy_arn).await?;

                policies.push(AwsIamPolicy {
                    policy_id: policy.policy_id().unwrap_or_default().to_string(),
                    policy_name: policy.policy_name().unwrap_or_default().to_string(),
                    arn: policy_arn,
                    path: policy.path().unwrap_or("/").to_string(),
                    default_version_id,
                    attachment_count: policy.attachment_count().unwrap_or(0),
                    is_attachable: policy.is_attachable(),
                    create_date: policy.create_date().map(|d| {
                        DateTime::from_timestamp(d.secs(), d.subsec_nanos())
                            .unwrap_or_else(Utc::now)
                    }),
                    update_date: policy.update_date().map(|d| {
                        DateTime::from_timestamp(d.secs(), d.subsec_nanos())
                            .unwrap_or_else(Utc::now)
                    }),
                    policy_document: policy_doc,
                    description: policy.description().map(|s| s.to_string()),
                    allows_admin_access: allows_admin,
                    uses_wildcard_actions: wildcard_actions,
                    uses_wildcard_resources: wildcard_resources,
                    risk_score,
                    tags,
                });
            }

            if response.is_truncated() {
                marker = response.marker().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(policies)
    }

    async fn get_policy_document(
        iam_client: &aws_sdk_iam::Client,
        policy_arn: &str,
        version_id: &str,
    ) -> Result<Option<serde_json::Value>, String> {
        let response = iam_client
            .get_policy_version()
            .policy_arn(policy_arn)
            .version_id(version_id)
            .send()
            .await
            .map_err(|e| format!("Failed to get policy version {}: {}", policy_arn, e))?;

        Ok(response.policy_version().and_then(|pv| {
            pv.document().and_then(|doc| {
                urlencoding::decode(doc)
                    .ok()
                    .and_then(|decoded| serde_json::from_str(&decoded).ok())
            })
        }))
    }

    async fn get_policy_tags(
        iam_client: &aws_sdk_iam::Client,
        policy_arn: &str,
    ) -> Result<HashMap<String, String>, String> {
        let response = iam_client
            .list_policy_tags()
            .policy_arn(policy_arn)
            .send()
            .await
            .map_err(|e| format!("Failed to list tags for policy {}: {}", policy_arn, e))?;

        Ok(response
            .tags()
            .iter()
            .map(|t| (t.key().to_string(), t.value().to_string()))
            .collect())
    }

    /// Analyze a policy document for risk indicators
    fn analyze_policy_risk(
        policy_doc: &Option<serde_json::Value>,
    ) -> (bool, bool, bool, i32) {
        let Some(doc) = policy_doc else {
            return (false, false, false, 0);
        };

        let mut allows_admin = false;
        let mut wildcard_actions = false;
        let mut wildcard_resources = false;
        let mut risk_score = 0;

        if let Some(statements) = doc.get("Statement").and_then(|s| s.as_array()) {
            for statement in statements {
                let effect = statement
                    .get("Effect")
                    .and_then(|e| e.as_str())
                    .unwrap_or("");

                if effect != "Allow" {
                    continue;
                }

                // Check actions
                let actions = match statement.get("Action") {
                    Some(serde_json::Value::String(s)) => vec![s.as_str()],
                    Some(serde_json::Value::Array(arr)) => {
                        arr.iter().filter_map(|v| v.as_str()).collect()
                    }
                    _ => vec![],
                };

                for action in &actions {
                    if *action == "*" {
                        allows_admin = true;
                        wildcard_actions = true;
                        risk_score += 50;
                    } else if action.ends_with(":*") {
                        wildcard_actions = true;
                        risk_score += 20;
                    } else if action.contains('*') {
                        wildcard_actions = true;
                        risk_score += 10;
                    }

                    // High-risk actions
                    let high_risk = [
                        "iam:*",
                        "iam:CreateUser",
                        "iam:AttachUserPolicy",
                        "iam:AttachRolePolicy",
                        "iam:PutUserPolicy",
                        "iam:CreateAccessKey",
                        "sts:AssumeRole",
                        "organizations:*",
                    ];
                    if high_risk.iter().any(|hr| action.eq_ignore_ascii_case(hr)) {
                        risk_score += 15;
                    }
                }

                // Check resources
                let resources = match statement.get("Resource") {
                    Some(serde_json::Value::String(s)) => vec![s.as_str()],
                    Some(serde_json::Value::Array(arr)) => {
                        arr.iter().filter_map(|v| v.as_str()).collect()
                    }
                    _ => vec![],
                };

                for resource in &resources {
                    if *resource == "*" {
                        wildcard_resources = true;
                        risk_score += 25;
                    }
                }
            }
        }

        // Cap risk score at 100
        risk_score = risk_score.min(100);

        (allows_admin, wildcard_actions, wildcard_resources, risk_score)
    }
}
