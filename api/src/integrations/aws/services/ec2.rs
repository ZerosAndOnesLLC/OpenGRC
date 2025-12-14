use crate::integrations::aws::client::AwsClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;

/// EC2 instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsEc2Instance {
    pub instance_id: String,
    pub instance_type: String,
    pub state: String,
    pub launch_time: Option<DateTime<Utc>>,
    pub availability_zone: String,
    pub vpc_id: Option<String>,
    pub subnet_id: Option<String>,
    pub private_ip: Option<String>,
    pub public_ip: Option<String>,
    pub iam_profile: Option<String>,
    pub security_groups: Vec<String>,
    pub tags: HashMap<String, String>,
    pub platform: Option<String>,
    pub monitoring_enabled: bool,
}

/// Security group information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsSecurityGroup {
    pub group_id: String,
    pub group_name: String,
    pub description: String,
    pub vpc_id: Option<String>,
    pub inbound_rules: Vec<SecurityGroupRule>,
    pub outbound_rules: Vec<SecurityGroupRule>,
    pub allows_unrestricted_ssh: bool,
    pub allows_unrestricted_rdp: bool,
    pub allows_all_inbound: bool,
}

/// Security group rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityGroupRule {
    pub protocol: String,
    pub from_port: Option<i32>,
    pub to_port: Option<i32>,
    pub cidr_blocks: Vec<String>,
    pub description: Option<String>,
}

/// EC2 collector
pub struct Ec2Collector;

impl Ec2Collector {
    /// Sync EC2 resources for a region
    pub async fn sync(
        client: &AwsClient,
        _context: &SyncContext,
        region: &str,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        let ec2_client = client.ec2_client(region).await?;

        // Collect instances
        let instances = Self::collect_instances(&ec2_client).await?;
        result.records_processed += instances.len() as i32;

        // Collect security groups
        let security_groups = Self::collect_security_groups(&ec2_client).await?;
        result.records_processed += security_groups.len() as i32;

        // Analyze
        let public_instances: Vec<_> = instances.iter().filter(|i| i.public_ip.is_some()).collect();
        let risky_sgs: Vec<_> = security_groups
            .iter()
            .filter(|sg| sg.allows_unrestricted_ssh || sg.allows_unrestricted_rdp || sg.allows_all_inbound)
            .collect();

        // Generate instance inventory evidence
        result.evidence_collected.push(CollectedEvidence {
            title: format!("EC2 Instance Inventory - {}", region),
            description: Some(format!(
                "{} instances ({} with public IPs)",
                instances.len(),
                public_instances.len()
            )),
            evidence_type: "automated".to_string(),
            source: "aws".to_string(),
            source_reference: Some(format!("ec2:{}:instances", region)),
            data: json!({
                "region": region,
                "total_instances": instances.len(),
                "running_instances": instances.iter().filter(|i| i.state == "running").count(),
                "public_instances": public_instances.len(),
                "instances": instances.iter().map(|i| json!({
                    "instance_id": i.instance_id,
                    "instance_type": i.instance_type,
                    "state": i.state,
                    "public_ip": i.public_ip,
                    "vpc_id": i.vpc_id,
                    "security_groups": i.security_groups,
                })).collect::<Vec<_>>(),
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec!["A1.1".to_string(), "CC6.6".to_string()],
        });

        // Generate security group evidence
        result.evidence_collected.push(CollectedEvidence {
            title: format!("Security Groups - {}", region),
            description: Some(format!(
                "{} security groups ({} with risky rules)",
                security_groups.len(),
                risky_sgs.len()
            )),
            evidence_type: "automated".to_string(),
            source: "aws".to_string(),
            source_reference: Some(format!("ec2:{}:security-groups", region)),
            data: json!({
                "region": region,
                "total_security_groups": security_groups.len(),
                "risky_security_groups": risky_sgs.len(),
                "security_groups": security_groups.iter().map(|sg| json!({
                    "group_id": sg.group_id,
                    "group_name": sg.group_name,
                    "vpc_id": sg.vpc_id,
                    "unrestricted_ssh": sg.allows_unrestricted_ssh,
                    "unrestricted_rdp": sg.allows_unrestricted_rdp,
                    "all_inbound": sg.allows_all_inbound,
                })).collect::<Vec<_>>(),
                "collected_at": Utc::now().to_rfc3339(),
            }),
            control_codes: vec!["CC6.6".to_string(), "CC6.7".to_string()],
        });

        // Risky security groups evidence
        if !risky_sgs.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: format!("Risky Security Groups - {}", region),
                description: Some(format!(
                    "{} security groups with overly permissive rules",
                    risky_sgs.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some(format!("ec2:{}:risky-sgs", region)),
                data: json!({
                    "risky_groups": risky_sgs.iter().map(|sg| json!({
                        "group_id": sg.group_id,
                        "group_name": sg.group_name,
                        "issues": {
                            "unrestricted_ssh": sg.allows_unrestricted_ssh,
                            "unrestricted_rdp": sg.allows_unrestricted_rdp,
                            "all_inbound": sg.allows_all_inbound,
                        },
                        "inbound_rules": sg.inbound_rules,
                    })).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC6.6".to_string(), "CC6.7".to_string()],
            });
        }

        result.records_created = result.records_processed;
        Ok(result)
    }

    async fn collect_instances(
        ec2_client: &aws_sdk_ec2::Client,
    ) -> Result<Vec<AwsEc2Instance>, String> {
        let mut instances = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = ec2_client.describe_instances();
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format!("Failed to describe instances: {}", e))?;

            for reservation in response.reservations() {
                for instance in reservation.instances() {
                    let tags: HashMap<String, String> = instance
                        .tags()
                        .iter()
                        .filter_map(|t| {
                            Some((t.key()?.to_string(), t.value()?.to_string()))
                        })
                        .collect();

                    instances.push(AwsEc2Instance {
                        instance_id: instance.instance_id().unwrap_or_default().to_string(),
                        instance_type: instance
                            .instance_type()
                            .map(|t| t.as_str().to_string())
                            .unwrap_or_default(),
                        state: instance
                            .state()
                            .and_then(|s| s.name())
                            .map(|n| n.as_str().to_string())
                            .unwrap_or_else(|| "unknown".to_string()),
                        launch_time: instance.launch_time().map(|t| {
                            DateTime::from_timestamp(t.secs(), t.subsec_nanos())
                                .unwrap_or_else(Utc::now)
                        }),
                        availability_zone: instance
                            .placement()
                            .and_then(|p| p.availability_zone())
                            .unwrap_or_default()
                            .to_string(),
                        vpc_id: instance.vpc_id().map(|s| s.to_string()),
                        subnet_id: instance.subnet_id().map(|s| s.to_string()),
                        private_ip: instance.private_ip_address().map(|s| s.to_string()),
                        public_ip: instance.public_ip_address().map(|s| s.to_string()),
                        iam_profile: instance
                            .iam_instance_profile()
                            .and_then(|p| p.arn())
                            .map(|s| s.to_string()),
                        security_groups: instance
                            .security_groups()
                            .iter()
                            .filter_map(|sg| sg.group_id().map(|s| s.to_string()))
                            .collect(),
                        tags,
                        platform: instance.platform().map(|p| p.as_str().to_string()),
                        monitoring_enabled: instance
                            .monitoring()
                            .and_then(|m| m.state())
                            .map(|s| s.as_str() == "enabled")
                            .unwrap_or(false),
                    });
                }
            }

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(instances)
    }

    async fn collect_security_groups(
        ec2_client: &aws_sdk_ec2::Client,
    ) -> Result<Vec<AwsSecurityGroup>, String> {
        let mut security_groups = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = ec2_client.describe_security_groups();
            if let Some(token) = &next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format!("Failed to describe security groups: {}", e))?;

            for sg in response.security_groups() {
                let inbound_rules: Vec<SecurityGroupRule> = sg
                    .ip_permissions()
                    .iter()
                    .map(|perm| SecurityGroupRule {
                        protocol: perm.ip_protocol().unwrap_or("-1").to_string(),
                        from_port: perm.from_port(),
                        to_port: perm.to_port(),
                        cidr_blocks: perm
                            .ip_ranges()
                            .iter()
                            .filter_map(|r| r.cidr_ip().map(|s| s.to_string()))
                            .collect(),
                        description: perm
                            .ip_ranges()
                            .first()
                            .and_then(|r| r.description())
                            .map(|s| s.to_string()),
                    })
                    .collect();

                let outbound_rules: Vec<SecurityGroupRule> = sg
                    .ip_permissions_egress()
                    .iter()
                    .map(|perm| SecurityGroupRule {
                        protocol: perm.ip_protocol().unwrap_or("-1").to_string(),
                        from_port: perm.from_port(),
                        to_port: perm.to_port(),
                        cidr_blocks: perm
                            .ip_ranges()
                            .iter()
                            .filter_map(|r| r.cidr_ip().map(|s| s.to_string()))
                            .collect(),
                        description: perm
                            .ip_ranges()
                            .first()
                            .and_then(|r| r.description())
                            .map(|s| s.to_string()),
                    })
                    .collect();

                // Check for risky rules
                let allows_unrestricted_ssh = inbound_rules.iter().any(|r| {
                    r.cidr_blocks.contains(&"0.0.0.0/0".to_string())
                        && (r.from_port == Some(22) || r.to_port == Some(22))
                });

                let allows_unrestricted_rdp = inbound_rules.iter().any(|r| {
                    r.cidr_blocks.contains(&"0.0.0.0/0".to_string())
                        && (r.from_port == Some(3389) || r.to_port == Some(3389))
                });

                let allows_all_inbound = inbound_rules.iter().any(|r| {
                    r.cidr_blocks.contains(&"0.0.0.0/0".to_string())
                        && r.protocol == "-1"
                });

                security_groups.push(AwsSecurityGroup {
                    group_id: sg.group_id().unwrap_or_default().to_string(),
                    group_name: sg.group_name().unwrap_or_default().to_string(),
                    description: sg.description().unwrap_or_default().to_string(),
                    vpc_id: sg.vpc_id().map(|s| s.to_string()),
                    inbound_rules,
                    outbound_rules,
                    allows_unrestricted_ssh,
                    allows_unrestricted_rdp,
                    allows_all_inbound,
                });
            }

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(security_groups)
    }
}
