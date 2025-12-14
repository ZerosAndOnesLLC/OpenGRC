use crate::integrations::aws::client::AwsClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;

/// AWS Config rule with compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfigRule {
    pub config_rule_name: String,
    pub config_rule_arn: String,
    pub config_rule_id: String,
    pub description: Option<String>,
    pub source_owner: String,
    pub source_identifier: String,
    pub compliance_type: String,
    pub compliant_count: i32,
    pub non_compliant_count: i32,
}

/// AWS Config collector
pub struct ConfigCollector;

impl ConfigCollector {
    /// Sync AWS Config rules and compliance for a region
    pub async fn sync(
        client: &AwsClient,
        _context: &SyncContext,
        region: &str,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        let config_client = client.config_client(region).await?;

        // Get config rules
        let rules = Self::collect_rules(&config_client).await?;
        result.records_processed = rules.len() as i32;

        // Calculate compliance stats
        let compliant_rules: Vec<_> = rules
            .iter()
            .filter(|r| r.compliance_type == "COMPLIANT")
            .collect();
        let non_compliant_rules: Vec<_> = rules
            .iter()
            .filter(|r| r.compliance_type == "NON_COMPLIANT")
            .collect();

        if !rules.is_empty() {
            let compliance_rate = compliant_rules.len() as f64 / rules.len() as f64 * 100.0;

            result.evidence_collected.push(CollectedEvidence {
                title: format!("AWS Config Compliance Summary - {}", region),
                description: Some(format!(
                    "{:.1}% compliance rate ({} of {} rules passing)",
                    compliance_rate,
                    compliant_rules.len(),
                    rules.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some(format!("config:{}:rules", region)),
                data: json!({
                    "region": region,
                    "total_rules": rules.len(),
                    "compliant_rules": compliant_rules.len(),
                    "non_compliant_rules": non_compliant_rules.len(),
                    "compliance_rate": compliance_rate,
                    "rules": rules.iter().map(|r| json!({
                        "name": r.config_rule_name,
                        "compliance": r.compliance_type,
                        "source": r.source_identifier,
                        "compliant_resources": r.compliant_count,
                        "non_compliant_resources": r.non_compliant_count,
                    })).collect::<Vec<_>>(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC7.1".to_string(), "CC3.2".to_string()],
            });

            // Non-compliant rules evidence
            if !non_compliant_rules.is_empty() {
                result.evidence_collected.push(CollectedEvidence {
                    title: format!("Non-Compliant Config Rules - {}", region),
                    description: Some(format!(
                        "{} Config rules with non-compliant resources",
                        non_compliant_rules.len()
                    )),
                    evidence_type: "automated".to_string(),
                    source: "aws".to_string(),
                    source_reference: Some(format!("config:{}:non-compliant", region)),
                    data: json!({
                        "non_compliant_rules": non_compliant_rules.iter().map(|r| json!({
                            "name": r.config_rule_name,
                            "description": r.description,
                            "source": r.source_identifier,
                            "non_compliant_count": r.non_compliant_count,
                        })).collect::<Vec<_>>(),
                        "collected_at": Utc::now().to_rfc3339(),
                    }),
                    control_codes: vec!["CC7.1".to_string(), "CC7.2".to_string()],
                });
            }
        }

        result.records_created = result.records_processed;
        Ok(result)
    }

    async fn collect_rules(
        config_client: &aws_sdk_config::Client,
    ) -> Result<Vec<AwsConfigRule>, String> {
        let mut rules = Vec::new();

        // Get all config rules
        let rules_response = config_client
            .describe_config_rules()
            .send()
            .await
            .map_err(|e| format!("Failed to describe config rules: {}", e))?;

        // Get compliance for all rules
        let compliance_response = config_client
            .describe_compliance_by_config_rule()
            .send()
            .await
            .map_err(|e| format!("Failed to get compliance: {}", e))?;

        let compliance_map: std::collections::HashMap<_, _> = compliance_response
            .compliance_by_config_rules()
            .iter()
            .filter_map(|c| {
                c.config_rule_name().map(|name| {
                    (
                        name.to_string(),
                        c.compliance()
                            .and_then(|comp| comp.compliance_type())
                            .map(|t| t.as_str().to_string())
                            .unwrap_or_else(|| "INSUFFICIENT_DATA".to_string()),
                    )
                })
            })
            .collect();

        for rule in rules_response.config_rules() {
            let rule_name = rule.config_rule_name().unwrap_or_default().to_string();
            let compliance_type = compliance_map
                .get(&rule_name)
                .cloned()
                .unwrap_or_else(|| "INSUFFICIENT_DATA".to_string());

            let source = rule.source();

            rules.push(AwsConfigRule {
                config_rule_name: rule_name.clone(),
                config_rule_arn: rule.config_rule_arn().unwrap_or_default().to_string(),
                config_rule_id: rule.config_rule_id().unwrap_or_default().to_string(),
                description: rule.description().map(|s| s.to_string()),
                source_owner: source
                    .map(|s| s.owner().as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                source_identifier: source
                    .and_then(|s| s.source_identifier())
                    .unwrap_or("Unknown")
                    .to_string(),
                compliance_type,
                compliant_count: 0,     // Would need additional API call
                non_compliant_count: 0, // Would need additional API call
            });
        }

        Ok(rules)
    }
}
