use crate::integrations::aws::client::AwsClient;
use crate::integrations::provider::{CollectedEvidence, SyncContext, SyncResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Security Hub finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsSecurityHubFinding {
    pub id: String,
    pub product_arn: String,
    pub product_name: String,
    pub generator_id: String,
    pub aws_account_id: String,
    pub region: String,
    pub types: Vec<String>,
    pub title: String,
    pub description: String,
    pub severity_label: String,
    pub severity_normalized: i32,
    pub workflow_status: String,
    pub record_state: String,
    pub compliance_status: Option<String>,
    pub compliance_standards: Vec<String>,
    pub related_resources: Vec<String>,
    pub remediation_text: Option<String>,
    pub remediation_url: Option<String>,
    pub first_observed_at: Option<DateTime<Utc>>,
    pub last_observed_at: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Security Hub collector
pub struct SecurityHubCollector;

impl SecurityHubCollector {
    /// Sync Security Hub findings for a region
    pub async fn sync(
        client: &AwsClient,
        _context: &SyncContext,
        region: &str,
    ) -> Result<SyncResult, String> {
        let mut result = SyncResult::default();

        let sh_client = client.securityhub_client(region).await?;

        // Get findings
        let findings = Self::collect_findings(&sh_client, region).await?;
        result.records_processed = findings.len() as i32;

        // Group findings by severity
        let critical: Vec<_> = findings
            .iter()
            .filter(|f| f.severity_label == "CRITICAL")
            .collect();
        let high: Vec<_> = findings
            .iter()
            .filter(|f| f.severity_label == "HIGH")
            .collect();
        let medium: Vec<_> = findings
            .iter()
            .filter(|f| f.severity_label == "MEDIUM")
            .collect();
        let low: Vec<_> = findings
            .iter()
            .filter(|f| f.severity_label == "LOW")
            .collect();

        // Generate evidence
        if !findings.is_empty() {
            result.evidence_collected.push(CollectedEvidence {
                title: format!("Security Hub Findings Summary - {}", region),
                description: Some(format!(
                    "{} total findings: {} critical, {} high, {} medium, {} low",
                    findings.len(),
                    critical.len(),
                    high.len(),
                    medium.len(),
                    low.len()
                )),
                evidence_type: "automated".to_string(),
                source: "aws".to_string(),
                source_reference: Some(format!("securityhub:{}:findings", region)),
                data: json!({
                    "region": region,
                    "total_findings": findings.len(),
                    "by_severity": {
                        "critical": critical.len(),
                        "high": high.len(),
                        "medium": medium.len(),
                        "low": low.len(),
                    },
                    "active_findings": findings.iter()
                        .filter(|f| f.workflow_status != "RESOLVED" && f.workflow_status != "SUPPRESSED")
                        .count(),
                    "collected_at": Utc::now().to_rfc3339(),
                }),
                control_codes: vec!["CC7.1".to_string(), "CC7.2".to_string()],
            });

            // Add critical findings evidence
            if !critical.is_empty() {
                result.evidence_collected.push(CollectedEvidence {
                    title: format!("Critical Security Findings - {}", region),
                    description: Some(format!("{} critical severity findings", critical.len())),
                    evidence_type: "automated".to_string(),
                    source: "aws".to_string(),
                    source_reference: Some(format!("securityhub:{}:critical", region)),
                    data: json!({
                        "findings": critical.iter().map(|f| json!({
                            "id": f.id,
                            "title": f.title,
                            "generator_id": f.generator_id,
                            "workflow_status": f.workflow_status,
                            "resources": f.related_resources,
                            "first_observed": f.first_observed_at,
                        })).collect::<Vec<_>>(),
                        "collected_at": Utc::now().to_rfc3339(),
                    }),
                    control_codes: vec!["CC7.1".to_string(), "CC3.2".to_string()],
                });
            }
        }

        result.records_created = result.records_processed;
        Ok(result)
    }

    async fn collect_findings(
        sh_client: &aws_sdk_securityhub::Client,
        region: &str,
    ) -> Result<Vec<AwsSecurityHubFinding>, String> {
        let mut findings = Vec::new();
        let mut next_token: Option<String> = None;

        loop {
            let mut request = sh_client
                .get_findings()
                .filters(
                    aws_sdk_securityhub::types::AwsSecurityFindingFilters::builder()
                        .record_state(
                            aws_sdk_securityhub::types::StringFilter::builder()
                                .comparison(aws_sdk_securityhub::types::StringFilterComparison::Equals)
                                .value("ACTIVE")
                                .build(),
                        )
                        .build(),
                )
                .max_results(100);

            if let Some(token) = &next_token {
                request = request.next_token(token);
            }

            let response = request
                .send()
                .await
                .map_err(|e| format!("Failed to get Security Hub findings: {}", e))?;

            for finding in response.findings() {
                let severity = finding.severity();
                let compliance = finding.compliance();
                let remediation = finding.remediation();

                findings.push(AwsSecurityHubFinding {
                    id: finding.id().unwrap_or_default().to_string(),
                    product_arn: finding.product_arn().unwrap_or_default().to_string(),
                    product_name: finding
                        .product_name()
                        .unwrap_or("Unknown")
                        .to_string(),
                    generator_id: finding.generator_id().unwrap_or_default().to_string(),
                    aws_account_id: finding.aws_account_id().unwrap_or_default().to_string(),
                    region: region.to_string(),
                    types: finding.types().iter().map(|s| s.to_string()).collect(),
                    title: finding.title().unwrap_or_default().to_string(),
                    description: finding.description().unwrap_or("").to_string(),
                    severity_label: severity
                        .and_then(|s| s.label())
                        .map(|l| l.as_str().to_string())
                        .unwrap_or_else(|| "INFORMATIONAL".to_string()),
                    severity_normalized: severity
                        .map(|s| s.normalized().unwrap_or(0) as i32)
                        .unwrap_or(0),
                    workflow_status: finding
                        .workflow()
                        .and_then(|w| w.status())
                        .map(|s| s.as_str().to_string())
                        .unwrap_or_else(|| "NEW".to_string()),
                    record_state: finding
                        .record_state()
                        .map(|s| s.as_str().to_string())
                        .unwrap_or_else(|| "ACTIVE".to_string()),
                    compliance_status: compliance
                        .and_then(|c| c.status())
                        .map(|s| s.as_str().to_string()),
                    compliance_standards: compliance
                        .map(|c| {
                            c.related_requirements()
                                .iter()
                                .map(|s| s.to_string())
                                .collect()
                        })
                        .unwrap_or_default(),
                    related_resources: finding
                        .resources()
                        .iter()
                        .filter_map(|r| r.id().map(|s| s.to_string()))
                        .collect(),
                    remediation_text: remediation
                        .and_then(|r| r.recommendation())
                        .and_then(|rec| rec.text())
                        .map(|s| s.to_string()),
                    remediation_url: remediation
                        .and_then(|r| r.recommendation())
                        .and_then(|rec| rec.url())
                        .map(|s| s.to_string()),
                    first_observed_at: finding.first_observed_at().and_then(|s| {
                        DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
                    }),
                    last_observed_at: finding.last_observed_at().and_then(|s| {
                        DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
                    }),
                    created_at: finding.created_at().and_then(|s| {
                        DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
                    }),
                    updated_at: finding.updated_at().and_then(|s| {
                        DateTime::parse_from_rfc3339(s).ok().map(|d| d.with_timezone(&Utc))
                    }),
                });
            }

            next_token = response.next_token().map(|s| s.to_string());
            if next_token.is_none() {
                break;
            }
        }

        Ok(findings)
    }
}
