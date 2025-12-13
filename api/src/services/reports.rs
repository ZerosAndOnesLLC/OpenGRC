use crate::utils::AppResult;
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct ReportsService {
    db: PgPool,
}

// Report data structures
#[derive(Debug, sqlx::FromRow)]
pub struct ControlReportRow {
    pub code: String,
    pub name: String,
    pub control_type: Option<String>,
    pub frequency: Option<String>,
    pub status: Option<String>,
    pub requirement_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct RiskReportRow {
    pub code: String,
    pub title: String,
    pub category: Option<String>,
    pub likelihood: i32,
    pub impact: i32,
    pub inherent_score: i32,
    pub residual_likelihood: Option<i32>,
    pub residual_impact: Option<i32>,
    pub residual_score: Option<i32>,
    pub status: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct EvidenceReportRow {
    pub title: String,
    pub evidence_type: Option<String>,
    pub source: Option<String>,
    pub valid_from: Option<DateTime<Utc>>,
    pub valid_until: Option<DateTime<Utc>>,
    pub control_count: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct PolicyReportRow {
    pub code: String,
    pub title: String,
    pub category: Option<String>,
    pub version: Option<i32>,
    pub status: Option<String>,
    pub effective_date: Option<NaiveDate>,
    pub review_date: Option<NaiveDate>,
    pub acknowledgment_count: i64,
}

#[derive(Debug, sqlx::FromRow)]
pub struct VendorReportRow {
    pub name: String,
    pub category: Option<String>,
    pub criticality: Option<String>,
    pub status: Option<String>,
    pub contract_start: Option<NaiveDate>,
    pub contract_end: Option<NaiveDate>,
    pub last_assessment: Option<DateTime<Utc>>,
}

#[derive(Debug, sqlx::FromRow)]
pub struct CompliancePostureRow {
    pub framework_name: String,
    pub total_requirements: i64,
    pub covered_requirements: i64,
    pub coverage_percentage: f64,
}

impl ReportsService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Generate controls report as CSV
    pub async fn generate_controls_csv(&self, org_id: Uuid) -> AppResult<String> {
        let rows = sqlx::query_as!(
            ControlReportRow,
            r#"
            SELECT
                c.code,
                c.name,
                c.control_type,
                c.frequency,
                c.status,
                COALESCE(COUNT(crm.id), 0) as "requirement_count!",
                c.created_at
            FROM controls c
            LEFT JOIN control_requirement_mappings crm ON c.id = crm.control_id
            WHERE c.organization_id = $1
            GROUP BY c.id
            ORDER BY c.code
            "#,
            org_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut csv = String::from("Code,Name,Type,Frequency,Status,Requirements Mapped,Created At\n");
        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                escape_csv(&row.code),
                escape_csv(&row.name),
                row.control_type.as_deref().unwrap_or("-"),
                row.frequency.as_deref().unwrap_or("-"),
                row.status.as_deref().unwrap_or("-"),
                row.requirement_count,
                row.created_at.format("%Y-%m-%d")
            ));
        }

        Ok(csv)
    }

    /// Generate risks report as CSV
    pub async fn generate_risks_csv(&self, org_id: Uuid) -> AppResult<String> {
        let rows = sqlx::query_as!(
            RiskReportRow,
            r#"
            SELECT
                code,
                title,
                category,
                COALESCE(likelihood, 0) as "likelihood!",
                COALESCE(impact, 0) as "impact!",
                COALESCE(likelihood * impact, 0) as "inherent_score!",
                residual_likelihood,
                residual_impact,
                CASE
                    WHEN residual_likelihood IS NOT NULL AND residual_impact IS NOT NULL
                    THEN residual_likelihood * residual_impact
                    ELSE NULL
                END as residual_score,
                status,
                source
            FROM risks
            WHERE organization_id = $1
            ORDER BY COALESCE(likelihood * impact, 0) DESC, code
            "#,
            org_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut csv = String::from("Code,Title,Category,Likelihood,Impact,Inherent Score,Residual Likelihood,Residual Impact,Residual Score,Status,Source\n");
        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{},{}\n",
                escape_csv(&row.code),
                escape_csv(&row.title),
                row.category.as_deref().unwrap_or("-"),
                row.likelihood,
                row.impact,
                row.inherent_score,
                row.residual_likelihood.map_or("-".to_string(), |v| v.to_string()),
                row.residual_impact.map_or("-".to_string(), |v| v.to_string()),
                row.residual_score.map_or("-".to_string(), |v| v.to_string()),
                row.status.as_deref().unwrap_or("-"),
                row.source.as_deref().unwrap_or("-")
            ));
        }

        Ok(csv)
    }

    /// Generate evidence report as CSV
    pub async fn generate_evidence_csv(&self, org_id: Uuid) -> AppResult<String> {
        let rows = sqlx::query_as!(
            EvidenceReportRow,
            r#"
            SELECT
                e.title,
                e.evidence_type,
                e.source,
                e.valid_from,
                e.valid_until,
                COALESCE(COUNT(ecl.id), 0) as "control_count!"
            FROM evidence e
            LEFT JOIN evidence_control_links ecl ON e.id = ecl.evidence_id
            WHERE e.organization_id = $1
            GROUP BY e.id
            ORDER BY e.title
            "#,
            org_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut csv = String::from("Title,Type,Source,Valid From,Valid Until,Linked Controls\n");
        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{}\n",
                escape_csv(&row.title),
                row.evidence_type.as_deref().unwrap_or("-"),
                row.source.as_deref().unwrap_or("-"),
                row.valid_from.map_or("-".to_string(), |d| d.format("%Y-%m-%d").to_string()),
                row.valid_until.map_or("-".to_string(), |d| d.format("%Y-%m-%d").to_string()),
                row.control_count
            ));
        }

        Ok(csv)
    }

    /// Generate policies report as CSV
    pub async fn generate_policies_csv(&self, org_id: Uuid) -> AppResult<String> {
        let rows = sqlx::query_as!(
            PolicyReportRow,
            r#"
            SELECT
                p.code,
                p.title,
                p.category,
                p.version,
                p.status,
                p.effective_date,
                p.review_date,
                COALESCE(COUNT(pa.id), 0) as "acknowledgment_count!"
            FROM policies p
            LEFT JOIN policy_acknowledgments pa ON p.id = pa.policy_id
            WHERE p.organization_id = $1
            GROUP BY p.id
            ORDER BY p.code
            "#,
            org_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut csv = String::from("Code,Title,Category,Version,Status,Effective Date,Review Date,Acknowledgments\n");
        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                escape_csv(&row.code),
                escape_csv(&row.title),
                row.category.as_deref().unwrap_or("-"),
                row.version.unwrap_or(1),
                row.status.as_deref().unwrap_or("-"),
                row.effective_date.map_or("-".to_string(), |d| d.format("%Y-%m-%d").to_string()),
                row.review_date.map_or("-".to_string(), |d| d.format("%Y-%m-%d").to_string()),
                row.acknowledgment_count
            ));
        }

        Ok(csv)
    }

    /// Generate vendors report as CSV
    pub async fn generate_vendors_csv(&self, org_id: Uuid) -> AppResult<String> {
        let rows = sqlx::query_as!(
            VendorReportRow,
            r#"
            SELECT
                v.name,
                v.category,
                v.criticality,
                v.status,
                v.contract_start,
                v.contract_end,
                (SELECT MAX(va.assessed_at) FROM vendor_assessments va WHERE va.vendor_id = v.id) as last_assessment
            FROM vendors v
            WHERE v.organization_id = $1
            ORDER BY v.name
            "#,
            org_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut csv = String::from("Name,Category,Criticality,Status,Contract Start,Contract End,Last Assessment\n");
        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{}\n",
                escape_csv(&row.name),
                row.category.as_deref().unwrap_or("-"),
                row.criticality.as_deref().unwrap_or("-"),
                row.status.as_deref().unwrap_or("-"),
                row.contract_start.map_or("-".to_string(), |d| d.format("%Y-%m-%d").to_string()),
                row.contract_end.map_or("-".to_string(), |d| d.format("%Y-%m-%d").to_string()),
                row.last_assessment.map_or("-".to_string(), |d| d.format("%Y-%m-%d").to_string())
            ));
        }

        Ok(csv)
    }

    /// Generate compliance posture report as CSV
    pub async fn generate_compliance_posture_csv(&self, org_id: Uuid) -> AppResult<String> {
        let rows = sqlx::query_as!(
            CompliancePostureRow,
            r#"
            SELECT
                f.name as framework_name,
                COUNT(DISTINCT fr.id) as "total_requirements!",
                COUNT(DISTINCT CASE WHEN crm.id IS NOT NULL THEN fr.id END) as "covered_requirements!",
                CASE
                    WHEN COUNT(DISTINCT fr.id) > 0
                    THEN (COUNT(DISTINCT CASE WHEN crm.id IS NOT NULL THEN fr.id END)::float / COUNT(DISTINCT fr.id)::float) * 100
                    ELSE 0
                END as "coverage_percentage!"
            FROM frameworks f
            JOIN framework_requirements fr ON f.id = fr.framework_id
            LEFT JOIN control_requirement_mappings crm ON fr.id = crm.framework_requirement_id
            LEFT JOIN controls c ON crm.control_id = c.id AND c.organization_id = $1
            GROUP BY f.id
            ORDER BY f.name
            "#,
            org_id
        )
        .fetch_all(&self.db)
        .await?;

        let mut csv = String::from("Framework,Total Requirements,Covered Requirements,Coverage %\n");
        for row in rows {
            csv.push_str(&format!(
                "{},{},{},{:.1}%\n",
                escape_csv(&row.framework_name),
                row.total_requirements,
                row.covered_requirements,
                row.coverage_percentage
            ));
        }

        Ok(csv)
    }
}

/// Escape a string for CSV (wrap in quotes if contains comma, quote, or newline)
fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}
