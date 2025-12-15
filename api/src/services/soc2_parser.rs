use crate::cache::CacheClient;
use crate::utils::{AppError, AppResult};
use chrono::NaiveDate;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

// ==================== Models ====================

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Soc2ParsedReport {
    pub id: Uuid,
    pub vendor_document_id: Uuid,
    pub vendor_id: Uuid,
    pub organization_id: Uuid,
    pub report_type: Option<String>,
    pub audit_period_start: Option<NaiveDate>,
    pub audit_period_end: Option<NaiveDate>,
    pub auditor_firm: Option<String>,
    pub opinion_type: Option<String>,
    pub trust_services_criteria: serde_json::Value,
    pub total_exceptions: Option<i32>,
    pub critical_exceptions: Option<i32>,
    pub high_exceptions: Option<i32>,
    pub medium_exceptions: Option<i32>,
    pub low_exceptions: Option<i32>,
    pub raw_findings: serde_json::Value,
    pub subservice_organizations: serde_json::Value,
    pub complementary_user_entity_controls: serde_json::Value,
    pub parsed_at: chrono::DateTime<chrono::Utc>,
    pub parsing_version: Option<String>,
    pub confidence_score: Option<rust_decimal::Decimal>,
    pub raw_text_hash: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Soc2Finding {
    pub id: Uuid,
    pub parsed_report_id: Uuid,
    pub finding_type: String,
    pub severity: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub criteria_codes: Option<Vec<String>>,
    pub management_response: Option<String>,
    pub remediation_status: Option<String>,
    pub remediation_date: Option<NaiveDate>,
    pub potential_impact: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedSoc2Data {
    pub report_type: Option<String>,
    pub audit_period_start: Option<NaiveDate>,
    pub audit_period_end: Option<NaiveDate>,
    pub auditor_firm: Option<String>,
    pub opinion_type: Option<String>,
    pub trust_services_criteria: Vec<String>,
    pub findings: Vec<ParsedFinding>,
    pub subservice_organizations: Vec<String>,
    pub cuecs: Vec<String>,
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFinding {
    pub finding_type: String,
    pub severity: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub criteria_codes: Vec<String>,
    pub management_response: Option<String>,
    pub remediation_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2ParseResult {
    pub parsed_report: Soc2ParsedReport,
    pub findings: Vec<Soc2Finding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2ReportSummary {
    pub id: Uuid,
    pub vendor_name: String,
    pub document_title: String,
    pub report_type: Option<String>,
    pub opinion_type: Option<String>,
    pub auditor_firm: Option<String>,
    pub audit_period_start: Option<NaiveDate>,
    pub audit_period_end: Option<NaiveDate>,
    pub total_exceptions: i32,
    pub critical_exceptions: i32,
    pub high_exceptions: i32,
    pub trust_services: Vec<String>,
    pub parsed_at: chrono::DateTime<chrono::Utc>,
}

// ==================== Service ====================

#[derive(Clone)]
pub struct Soc2ParserService {
    db: PgPool,
    #[allow(dead_code)]
    cache: CacheClient,
}

impl Soc2ParserService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    /// Parse a PDF file and extract SOC 2 report data
    pub fn parse_pdf(&self, pdf_data: &[u8]) -> AppResult<ParsedSoc2Data> {
        // Extract text from PDF
        let text = pdf_extract::extract_text_from_mem(pdf_data)
            .map_err(|e| AppError::BadRequest(format!("Failed to extract text from PDF: {}", e)))?;

        self.parse_text(&text)
    }

    /// Parse extracted text to find SOC 2 report data
    pub fn parse_text(&self, text: &str) -> AppResult<ParsedSoc2Data> {
        let mut data = ParsedSoc2Data {
            report_type: None,
            audit_period_start: None,
            audit_period_end: None,
            auditor_firm: None,
            opinion_type: None,
            trust_services_criteria: vec![],
            findings: vec![],
            subservice_organizations: vec![],
            cuecs: vec![],
            confidence_score: 0.0,
        };

        let mut confidence_factors = 0;
        let mut confidence_hits = 0;

        // Detect report type (Type 1 vs Type 2)
        confidence_factors += 1;
        if let Some(report_type) = self.detect_report_type(text) {
            data.report_type = Some(report_type);
            confidence_hits += 1;
        }

        // Extract audit period
        confidence_factors += 1;
        if let Some((start, end)) = self.extract_audit_period(text) {
            data.audit_period_start = Some(start);
            data.audit_period_end = Some(end);
            confidence_hits += 1;
        }

        // Extract auditor firm
        confidence_factors += 1;
        if let Some(auditor) = self.extract_auditor_firm(text) {
            data.auditor_firm = Some(auditor);
            confidence_hits += 1;
        }

        // Detect opinion type
        confidence_factors += 1;
        if let Some(opinion) = self.detect_opinion_type(text) {
            data.opinion_type = Some(opinion);
            confidence_hits += 1;
        }

        // Extract trust services criteria
        confidence_factors += 1;
        let criteria = self.extract_trust_services_criteria(text);
        if !criteria.is_empty() {
            data.trust_services_criteria = criteria;
            confidence_hits += 1;
        }

        // Extract findings/exceptions
        confidence_factors += 1;
        let findings = self.extract_findings(text);
        if !findings.is_empty() {
            data.findings = findings;
            confidence_hits += 1;
        }

        // Extract subservice organizations
        let subservice = self.extract_subservice_organizations(text);
        if !subservice.is_empty() {
            data.subservice_organizations = subservice;
        }

        // Extract CUECs
        let cuecs = self.extract_cuecs(text);
        if !cuecs.is_empty() {
            data.cuecs = cuecs;
        }

        // Calculate confidence score
        data.confidence_score = if confidence_factors > 0 {
            (confidence_hits as f64 / confidence_factors as f64) * 100.0
        } else {
            0.0
        };

        Ok(data)
    }

    /// Detect if this is a Type 1 or Type 2 report
    fn detect_report_type(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        // Type 2 patterns (check first as they're more specific)
        let type2_patterns = [
            "type 2",
            "type ii",
            "type two",
            "soc 2 type 2",
            "soc 2 type ii",
            "operating effectiveness",
            "period of time",
        ];

        for pattern in type2_patterns {
            if text_lower.contains(pattern) {
                return Some("type2".to_string());
            }
        }

        // Type 1 patterns
        let type1_patterns = [
            "type 1",
            "type i",
            "type one",
            "soc 2 type 1",
            "soc 2 type i",
            "point in time",
            "as of",
        ];

        for pattern in type1_patterns {
            if text_lower.contains(pattern) {
                return Some("type1".to_string());
            }
        }

        None
    }

    /// Extract audit period dates
    fn extract_audit_period(&self, text: &str) -> Option<(NaiveDate, NaiveDate)> {
        // Common patterns for audit periods
        let patterns = [
            // "January 1, 2024 through December 31, 2024"
            r"(?i)(January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{1,2},?\s+\d{4}\s+(?:through|to|and)\s+(January|February|March|April|May|June|July|August|September|October|November|December)\s+\d{1,2},?\s+\d{4}",
            // "01/01/2024 to 12/31/2024"
            r"(\d{1,2}/\d{1,2}/\d{4})\s+(?:through|to|and)\s+(\d{1,2}/\d{1,2}/\d{4})",
            // "2024-01-01 to 2024-12-31"
            r"(\d{4}-\d{2}-\d{2})\s+(?:through|to|and)\s+(\d{4}-\d{2}-\d{2})",
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(text) {
                    // Try to parse the dates
                    if let Some(dates) = self.parse_date_range_from_capture(&caps) {
                        return Some(dates);
                    }
                }
            }
        }

        // Fallback: Look for "period" mentions
        let period_pattern = r"(?i)(?:examination|audit|review)\s+period[:\s]+([^\n]+)";
        if let Ok(re) = Regex::new(period_pattern) {
            if let Some(caps) = re.captures(text) {
                if let Some(period_text) = caps.get(1) {
                    // Try to extract dates from the period text
                    return self.extract_dates_from_text(period_text.as_str());
                }
            }
        }

        None
    }

    fn parse_date_range_from_capture(&self, caps: &regex::Captures) -> Option<(NaiveDate, NaiveDate)> {
        let full_match = caps.get(0)?.as_str();
        self.extract_dates_from_text(full_match)
    }

    fn extract_dates_from_text(&self, text: &str) -> Option<(NaiveDate, NaiveDate)> {
        // Try to find two dates in the text
        let date_patterns = [
            r"(January|February|March|April|May|June|July|August|September|October|November|December)\s+(\d{1,2}),?\s+(\d{4})",
            r"(\d{1,2})/(\d{1,2})/(\d{4})",
            r"(\d{4})-(\d{2})-(\d{2})",
        ];

        let mut dates: Vec<NaiveDate> = vec![];

        for pattern in date_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for caps in re.captures_iter(text) {
                    if let Some(date) = self.parse_captured_date(&caps) {
                        dates.push(date);
                        if dates.len() >= 2 {
                            break;
                        }
                    }
                }
                if dates.len() >= 2 {
                    break;
                }
            }
        }

        if dates.len() >= 2 {
            dates.sort();
            Some((dates[0], dates[1]))
        } else {
            None
        }
    }

    fn parse_captured_date(&self, caps: &regex::Captures) -> Option<NaiveDate> {
        let full = caps.get(0)?.as_str();

        // Try ISO format first
        if let Ok(date) = NaiveDate::parse_from_str(full, "%Y-%m-%d") {
            return Some(date);
        }

        // Try US format
        if let Ok(date) = NaiveDate::parse_from_str(full, "%m/%d/%Y") {
            return Some(date);
        }

        // Try month name format
        let months = [
            ("january", 1), ("february", 2), ("march", 3), ("april", 4),
            ("may", 5), ("june", 6), ("july", 7), ("august", 8),
            ("september", 9), ("october", 10), ("november", 11), ("december", 12),
        ];

        let lower = full.to_lowercase();
        for (name, num) in months {
            if lower.contains(name) {
                // Extract day and year
                let day_year_re = Regex::new(r"(\d{1,2}),?\s+(\d{4})").ok()?;
                if let Some(dy_caps) = day_year_re.captures(&lower) {
                    let day: u32 = dy_caps.get(1)?.as_str().parse().ok()?;
                    let year: i32 = dy_caps.get(2)?.as_str().parse().ok()?;
                    return NaiveDate::from_ymd_opt(year, num, day);
                }
            }
        }

        None
    }

    /// Extract auditor firm name
    fn extract_auditor_firm(&self, text: &str) -> Option<String> {
        // Common Big 4 and well-known auditing firms
        let known_firms = [
            "Deloitte",
            "PricewaterhouseCoopers", "PwC",
            "Ernst & Young", "EY",
            "KPMG",
            "Grant Thornton",
            "BDO",
            "RSM",
            "Crowe",
            "Baker Tilly",
            "Moss Adams",
            "Plante Moran",
            "CliftonLarsonAllen", "CLA",
            "Marcum",
            "Wipfli",
            "Schellman",
            "A-LIGN",
            "Coalfire",
            "Secureframe",
        ];

        for firm in known_firms {
            if text.contains(firm) {
                return Some(firm.to_string());
            }
        }

        // Try to find "Independent Service Auditor's Report" section and extract firm
        let patterns = [
            r"(?i)(?:prepared by|issued by|audited by)[:\s]+([A-Z][A-Za-z\s&]+(?:LLP|LLC|Inc\.?|PC|P\.C\.))",
            r"(?i)independent\s+(?:service\s+)?auditor[s']?\s+report\s+([A-Z][A-Za-z\s&]+)",
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(text) {
                    if let Some(firm) = caps.get(1) {
                        let firm_name = firm.as_str().trim();
                        if firm_name.len() > 3 && firm_name.len() < 100 {
                            return Some(firm_name.to_string());
                        }
                    }
                }
            }
        }

        None
    }

    /// Detect auditor opinion type
    fn detect_opinion_type(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        // Adverse opinion (most severe, check first)
        let adverse_patterns = [
            "adverse opinion",
            "do not present fairly",
            "material misstatement",
            "controls were not suitably designed",
        ];
        for pattern in adverse_patterns {
            if text_lower.contains(pattern) {
                return Some("adverse".to_string());
            }
        }

        // Disclaimer of opinion
        let disclaimer_patterns = [
            "disclaimer of opinion",
            "disclaim an opinion",
            "unable to obtain sufficient",
            "scope limitation",
        ];
        for pattern in disclaimer_patterns {
            if text_lower.contains(pattern) {
                return Some("disclaimer".to_string());
            }
        }

        // Qualified opinion
        let qualified_patterns = [
            "qualified opinion",
            "except for",
            "with the exception of",
            "except as noted",
        ];
        for pattern in qualified_patterns {
            if text_lower.contains(pattern) {
                // Make sure it's not "unqualified"
                if !text_lower.contains("unqualified") {
                    return Some("qualified".to_string());
                }
            }
        }

        // Unqualified (clean) opinion
        let unqualified_patterns = [
            "unqualified opinion",
            "present fairly",
            "in all material respects",
            "suitably designed and operating effectively",
            "fairly stated",
            "clean opinion",
        ];
        for pattern in unqualified_patterns {
            if text_lower.contains(pattern) {
                return Some("unqualified".to_string());
            }
        }

        None
    }

    /// Extract trust services criteria covered
    fn extract_trust_services_criteria(&self, text: &str) -> Vec<String> {
        let text_lower = text.to_lowercase();
        let mut criteria = vec![];

        let criteria_map = [
            (vec!["security", "common criteria", "cc criteria"], "security"),
            (vec!["availability", "availability criteria"], "availability"),
            (vec!["processing integrity", "processing criteria"], "processing_integrity"),
            (vec!["confidentiality", "confidentiality criteria"], "confidentiality"),
            (vec!["privacy", "privacy criteria"], "privacy"),
        ];

        for (patterns, name) in criteria_map {
            for pattern in patterns {
                if text_lower.contains(pattern) {
                    if !criteria.contains(&name.to_string()) {
                        criteria.push(name.to_string());
                    }
                    break;
                }
            }
        }

        criteria
    }

    /// Extract findings and exceptions
    fn extract_findings(&self, text: &str) -> Vec<ParsedFinding> {
        let mut findings = vec![];

        // Look for exception sections
        let exception_patterns = [
            r"(?i)(?:exception|finding|deviation|deficiency)[\s#:]+(\d+)[:\s]+([^\n]+)",
            r"(?i)(?:test|control)\s+(?:exception|failure)[:\s]+([^\n]+)",
            r"(?i)(?:CC\d+\.\d+|A\d+\.\d+|PI\d+\.\d+|C\d+\.\d+|P\d+\.\d+)[:\s]+([^\n]{10,200})",
        ];

        for pattern in exception_patterns {
            if let Ok(re) = Regex::new(pattern) {
                for caps in re.captures_iter(text) {
                    let title = caps.get(caps.len() - 1)
                        .map(|m| m.as_str().trim().to_string());

                    if let Some(ref t) = title {
                        if t.len() > 10 {
                            // Extract criteria codes if present
                            let criteria_codes = self.extract_criteria_codes(caps.get(0).map(|m| m.as_str()).unwrap_or(""));

                            let finding = ParsedFinding {
                                finding_type: "exception".to_string(),
                                severity: self.infer_severity(t),
                                title: Some(t.clone()),
                                description: None,
                                criteria_codes,
                                management_response: None,
                                remediation_status: None,
                            };
                            findings.push(finding);
                        }
                    }
                }
            }
        }

        // Look for "no exceptions noted" patterns
        let no_exceptions_patterns = [
            r"(?i)no\s+(?:exceptions?|deviations?|deficienc(?:y|ies))\s+(?:were\s+)?(?:noted|identified|found)",
            r"(?i)tests?\s+(?:were\s+)?(?:performed|completed)\s+without\s+exception",
        ];

        for pattern in no_exceptions_patterns {
            if let Ok(re) = Regex::new(pattern) {
                if re.is_match(text) && findings.is_empty() {
                    // Report has clean findings
                    return vec![];
                }
            }
        }

        // Deduplicate by title
        let mut seen_titles: std::collections::HashSet<String> = std::collections::HashSet::new();
        findings.retain(|f| {
            if let Some(ref title) = f.title {
                let key = title.to_lowercase();
                if seen_titles.contains(&key) {
                    false
                } else {
                    seen_titles.insert(key);
                    true
                }
            } else {
                true
            }
        });

        findings
    }

    fn extract_criteria_codes(&self, text: &str) -> Vec<String> {
        let mut codes = vec![];

        // SOC 2 Trust Services Criteria patterns
        let pattern = r"(CC\d+\.\d+|A\d+\.\d+|PI\d+\.\d+|C\d+\.\d+|P\d+\.\d+)";
        if let Ok(re) = Regex::new(pattern) {
            for caps in re.captures_iter(text) {
                if let Some(code) = caps.get(1) {
                    let code_str = code.as_str().to_uppercase();
                    if !codes.contains(&code_str) {
                        codes.push(code_str);
                    }
                }
            }
        }

        codes
    }

    fn infer_severity(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        if text_lower.contains("critical") || text_lower.contains("severe") || text_lower.contains("significant deficiency") {
            return Some("critical".to_string());
        }
        if text_lower.contains("high") || text_lower.contains("major") || text_lower.contains("material") {
            return Some("high".to_string());
        }
        if text_lower.contains("medium") || text_lower.contains("moderate") {
            return Some("medium".to_string());
        }
        if text_lower.contains("low") || text_lower.contains("minor") || text_lower.contains("observation") {
            return Some("low".to_string());
        }

        // Default to medium if no severity indicator
        Some("medium".to_string())
    }

    /// Extract subservice organizations (carve-outs)
    fn extract_subservice_organizations(&self, text: &str) -> Vec<String> {
        let mut orgs = vec![];

        let patterns = [
            r"(?i)subservice\s+organization[s]?[:\s]+([^\n]+)",
            r"(?i)carved[\s-]out\s+(?:from|organizations?)[:\s]+([^\n]+)",
            r"(?i)excludes?\s+(?:the\s+)?(?:services?\s+)?(?:provided\s+)?by[:\s]+([^\n]+)",
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                for caps in re.captures_iter(text) {
                    if let Some(org) = caps.get(1) {
                        let org_text = org.as_str().trim();
                        if org_text.len() > 3 && org_text.len() < 200 {
                            orgs.push(org_text.to_string());
                        }
                    }
                }
            }
        }

        orgs
    }

    /// Extract Complementary User Entity Controls (CUECs)
    fn extract_cuecs(&self, text: &str) -> Vec<String> {
        let mut cuecs = vec![];

        let patterns = [
            r"(?i)(?:CUEC|complementary\s+user\s+entity\s+control)[s]?[:\s]+([^\n]+)",
            r"(?i)user\s+(?:entity\s+)?responsibilities?[:\s]+([^\n]+)",
        ];

        for pattern in patterns {
            if let Ok(re) = Regex::new(pattern) {
                for caps in re.captures_iter(text) {
                    if let Some(cuec) = caps.get(1) {
                        let cuec_text = cuec.as_str().trim();
                        if cuec_text.len() > 10 && cuec_text.len() < 500 {
                            cuecs.push(cuec_text.to_string());
                        }
                    }
                }
            }
        }

        cuecs
    }

    // ==================== Database Operations ====================

    /// Parse a vendor document and store the results
    pub async fn parse_vendor_document(
        &self,
        org_id: Uuid,
        vendor_id: Uuid,
        document_id: Uuid,
        pdf_data: &[u8],
    ) -> AppResult<Soc2ParseResult> {
        // Parse the PDF
        let parsed_data = self.parse_pdf(pdf_data)?;

        // Calculate hash for deduplication
        let mut hasher = Sha256::new();
        hasher.update(pdf_data);
        let text_hash = format!("{:x}", hasher.finalize());

        // Check if we already parsed this exact document
        let existing = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM soc2_parsed_reports WHERE raw_text_hash = $1 AND vendor_document_id = $2"
        )
        .bind(&text_hash)
        .bind(document_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(existing_id) = existing {
            // Return existing parsed report
            return self.get_parsed_report(org_id, existing_id).await;
        }

        // Count findings by severity
        let (critical, high, medium, low) = parsed_data.findings.iter().fold(
            (0i32, 0i32, 0i32, 0i32),
            |acc, f| match f.severity.as_deref() {
                Some("critical") => (acc.0 + 1, acc.1, acc.2, acc.3),
                Some("high") => (acc.0, acc.1 + 1, acc.2, acc.3),
                Some("medium") => (acc.0, acc.1, acc.2 + 1, acc.3),
                Some("low") | Some("informational") => (acc.0, acc.1, acc.2, acc.3 + 1),
                _ => (acc.0, acc.1, acc.2 + 1, acc.3),
            },
        );

        let total = parsed_data.findings.len() as i32;

        // Insert parsed report
        let report = sqlx::query_as::<_, Soc2ParsedReport>(
            r#"
            INSERT INTO soc2_parsed_reports (
                vendor_document_id, vendor_id, organization_id,
                report_type, audit_period_start, audit_period_end,
                auditor_firm, opinion_type, trust_services_criteria,
                total_exceptions, critical_exceptions, high_exceptions,
                medium_exceptions, low_exceptions, raw_findings,
                subservice_organizations, complementary_user_entity_controls,
                confidence_score, raw_text_hash
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19
            )
            RETURNING *
            "#,
        )
        .bind(document_id)
        .bind(vendor_id)
        .bind(org_id)
        .bind(&parsed_data.report_type)
        .bind(parsed_data.audit_period_start)
        .bind(parsed_data.audit_period_end)
        .bind(&parsed_data.auditor_firm)
        .bind(&parsed_data.opinion_type)
        .bind(serde_json::to_value(&parsed_data.trust_services_criteria).unwrap_or_default())
        .bind(total)
        .bind(critical)
        .bind(high)
        .bind(medium)
        .bind(low)
        .bind(serde_json::to_value(&parsed_data.findings).unwrap_or_default())
        .bind(serde_json::to_value(&parsed_data.subservice_organizations).unwrap_or_default())
        .bind(serde_json::to_value(&parsed_data.cuecs).unwrap_or_default())
        .bind(rust_decimal::Decimal::from_f64_retain(parsed_data.confidence_score))
        .bind(&text_hash)
        .fetch_one(&self.db)
        .await?;

        // Insert individual findings
        let mut findings = vec![];
        for finding in &parsed_data.findings {
            let f = sqlx::query_as::<_, Soc2Finding>(
                r#"
                INSERT INTO soc2_findings (
                    parsed_report_id, finding_type, severity,
                    title, description, criteria_codes,
                    management_response, remediation_status
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING *
                "#,
            )
            .bind(report.id)
            .bind(&finding.finding_type)
            .bind(&finding.severity)
            .bind(&finding.title)
            .bind(&finding.description)
            .bind(&finding.criteria_codes)
            .bind(&finding.management_response)
            .bind(&finding.remediation_status)
            .fetch_one(&self.db)
            .await?;

            findings.push(f);
        }

        tracing::info!(
            "Parsed SOC 2 report for vendor {}: {} findings, confidence {}%",
            vendor_id, total, parsed_data.confidence_score
        );

        Ok(Soc2ParseResult {
            parsed_report: report,
            findings,
        })
    }

    /// Get a parsed report by ID
    pub async fn get_parsed_report(&self, org_id: Uuid, id: Uuid) -> AppResult<Soc2ParseResult> {
        let report = sqlx::query_as::<_, Soc2ParsedReport>(
            "SELECT * FROM soc2_parsed_reports WHERE id = $1 AND organization_id = $2"
        )
        .bind(id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Parsed report not found".to_string()))?;

        let findings = sqlx::query_as::<_, Soc2Finding>(
            "SELECT * FROM soc2_findings WHERE parsed_report_id = $1 ORDER BY severity, created_at"
        )
        .bind(id)
        .fetch_all(&self.db)
        .await?;

        Ok(Soc2ParseResult {
            parsed_report: report,
            findings,
        })
    }

    /// Get parsed report for a vendor document
    pub async fn get_parsed_report_by_document(
        &self,
        org_id: Uuid,
        document_id: Uuid,
    ) -> AppResult<Option<Soc2ParseResult>> {
        let report = sqlx::query_as::<_, Soc2ParsedReport>(
            "SELECT * FROM soc2_parsed_reports WHERE vendor_document_id = $1 AND organization_id = $2"
        )
        .bind(document_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(report) = report {
            let findings = sqlx::query_as::<_, Soc2Finding>(
                "SELECT * FROM soc2_findings WHERE parsed_report_id = $1 ORDER BY severity, created_at"
            )
            .bind(report.id)
            .fetch_all(&self.db)
            .await?;

            Ok(Some(Soc2ParseResult {
                parsed_report: report,
                findings,
            }))
        } else {
            Ok(None)
        }
    }

    /// List all parsed reports for a vendor
    pub async fn list_vendor_reports(&self, org_id: Uuid, vendor_id: Uuid) -> AppResult<Vec<Soc2ParsedReport>> {
        let reports = sqlx::query_as::<_, Soc2ParsedReport>(
            "SELECT * FROM soc2_parsed_reports WHERE vendor_id = $1 AND organization_id = $2 ORDER BY parsed_at DESC"
        )
        .bind(vendor_id)
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(reports)
    }

    /// Get summary of all parsed reports for an organization
    pub async fn get_report_summaries(&self, org_id: Uuid) -> AppResult<Vec<Soc2ReportSummary>> {
        let summaries = sqlx::query_as::<_, Soc2ReportSummary>(
            r#"
            SELECT
                spr.id,
                v.name as vendor_name,
                vd.title as document_title,
                spr.report_type,
                spr.opinion_type,
                spr.auditor_firm,
                spr.audit_period_start,
                spr.audit_period_end,
                COALESCE(spr.total_exceptions, 0) as total_exceptions,
                COALESCE(spr.critical_exceptions, 0) as critical_exceptions,
                COALESCE(spr.high_exceptions, 0) as high_exceptions,
                COALESCE(spr.trust_services_criteria, '[]'::jsonb) as trust_services,
                spr.parsed_at
            FROM soc2_parsed_reports spr
            JOIN vendors v ON spr.vendor_id = v.id
            JOIN vendor_documents vd ON spr.vendor_document_id = vd.id
            WHERE spr.organization_id = $1
            ORDER BY spr.parsed_at DESC
            "#
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(summaries)
    }

    /// Delete a parsed report
    pub async fn delete_parsed_report(&self, org_id: Uuid, id: Uuid) -> AppResult<()> {
        sqlx::query("DELETE FROM soc2_parsed_reports WHERE id = $1 AND organization_id = $2")
            .bind(id)
            .bind(org_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}

// Custom FromRow implementation for Soc2ReportSummary to handle JSON conversion
impl<'r> sqlx::FromRow<'r, sqlx::postgres::PgRow> for Soc2ReportSummary {
    fn from_row(row: &'r sqlx::postgres::PgRow) -> Result<Self, sqlx::Error> {
        use sqlx::Row;

        let trust_services_json: serde_json::Value = row.try_get("trust_services")?;
        let trust_services: Vec<String> = serde_json::from_value(trust_services_json)
            .unwrap_or_default();

        Ok(Self {
            id: row.try_get("id")?,
            vendor_name: row.try_get("vendor_name")?,
            document_title: row.try_get("document_title")?,
            report_type: row.try_get("report_type")?,
            opinion_type: row.try_get("opinion_type")?,
            auditor_firm: row.try_get("auditor_firm")?,
            audit_period_start: row.try_get("audit_period_start")?,
            audit_period_end: row.try_get("audit_period_end")?,
            total_exceptions: row.try_get("total_exceptions")?,
            critical_exceptions: row.try_get("critical_exceptions")?,
            high_exceptions: row.try_get("high_exceptions")?,
            trust_services,
            parsed_at: row.try_get("parsed_at")?,
        })
    }
}
