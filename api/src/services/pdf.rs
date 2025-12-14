use crate::utils::AppResult;
use chrono::{DateTime, NaiveDate, Utc};
use printpdf::*;
use sqlx::PgPool;
use std::io::BufWriter;
use uuid::Uuid;

const PAGE_WIDTH_MM: f32 = 210.0;
const PAGE_HEIGHT_MM: f32 = 297.0;
const MARGIN_MM: f32 = 20.0;
const LINE_HEIGHT_MM: f32 = 5.0;
const HEADER_HEIGHT_MM: f32 = 25.0;
const FOOTER_HEIGHT_MM: f32 = 15.0;

#[derive(Clone)]
pub struct PdfService {
    db: PgPool,
}

// Report data structures (shared with CSV reports)
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

#[derive(Debug, sqlx::FromRow)]
struct OrgInfo {
    name: String,
}

impl PdfService {
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }

    /// Get organization name for branding
    async fn get_org_name(&self, org_id: Uuid) -> String {
        sqlx::query_as::<_, OrgInfo>("SELECT name FROM organizations WHERE id = $1")
            .bind(org_id)
            .fetch_optional(&self.db)
            .await
            .ok()
            .flatten()
            .map(|o| o.name)
            .unwrap_or_else(|| "Organization".to_string())
    }

    /// Create PDF document with header and footer
    fn create_pdf_doc(title: &str, org_name: &str) -> (PdfDocumentReference, PdfPageIndex, PdfLayerIndex) {
        let (doc, page1, layer1) = PdfDocument::new(
            title,
            Mm(PAGE_WIDTH_MM),
            Mm(PAGE_HEIGHT_MM),
            "Layer 1",
        );

        // We'll add header/footer content when rendering pages
        let _ = org_name; // Used in render functions

        (doc, page1, layer1)
    }

    /// Add header to a page
    fn add_header(layer: &PdfLayerReference, title: &str, org_name: &str, font: &IndirectFontRef) {
        // Title
        layer.use_text(title, 16.0, Mm(MARGIN_MM), Mm(PAGE_HEIGHT_MM - MARGIN_MM), font);

        // Organization name (right aligned approximation)
        layer.use_text(
            org_name,
            10.0,
            Mm(PAGE_WIDTH_MM - MARGIN_MM - 50.0),
            Mm(PAGE_HEIGHT_MM - MARGIN_MM),
            font,
        );

        // Separator line
        let line = Line {
            points: vec![
                (Point::new(Mm(MARGIN_MM), Mm(PAGE_HEIGHT_MM - MARGIN_MM - 8.0)), false),
                (Point::new(Mm(PAGE_WIDTH_MM - MARGIN_MM), Mm(PAGE_HEIGHT_MM - MARGIN_MM - 8.0)), false),
            ],
            is_closed: false,
        };
        layer.add_line(line);
    }

    /// Add footer to a page
    fn add_footer(layer: &PdfLayerReference, page_num: u32, total_pages: u32, font: &IndirectFontRef) {
        // Page number
        let page_text = format!("Page {} of {}", page_num, total_pages);
        layer.use_text(&page_text, 9.0, Mm(PAGE_WIDTH_MM / 2.0 - 10.0), Mm(MARGIN_MM / 2.0), font);

        // Generated date
        let date_text = format!("Generated: {}", Utc::now().format("%Y-%m-%d %H:%M UTC"));
        layer.use_text(&date_text, 8.0, Mm(MARGIN_MM), Mm(MARGIN_MM / 2.0), font);

        // OpenGRC branding
        layer.use_text("OpenGRC", 8.0, Mm(PAGE_WIDTH_MM - MARGIN_MM - 20.0), Mm(MARGIN_MM / 2.0), font);
    }

    /// Calculate content area Y position
    fn content_start_y() -> f32 {
        PAGE_HEIGHT_MM - MARGIN_MM - HEADER_HEIGHT_MM
    }

    fn content_end_y() -> f32 {
        MARGIN_MM + FOOTER_HEIGHT_MM
    }

    /// Generate controls report as PDF
    pub async fn generate_controls_pdf(&self, org_id: Uuid) -> AppResult<Vec<u8>> {
        let org_name = self.get_org_name(org_id).await;

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

        // Calculate summary stats
        let total = rows.len() as i64;
        let mut status_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for row in &rows {
            let status = row.status.as_deref().unwrap_or("Unknown").to_string();
            *status_counts.entry(status).or_insert(0) += 1;
        }
        let status_breakdown: Vec<(String, i64)> = status_counts.into_iter().collect();

        let (doc, page1, layer1) = Self::create_pdf_doc("Control Health Report", &org_name);
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

        let current_layer = doc.get_page(page1).get_layer(layer1);
        Self::add_header(&current_layer, "Control Health Report", &org_name, &font);

        let mut y_pos = Self::content_start_y();

        // Summary section
        current_layer.use_text("Summary", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        current_layer.use_text(&format!("Total Controls: {}", total), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;

        for (status, count) in &status_breakdown {
            let pct = if total > 0 { (*count as f64 / total as f64) * 100.0 } else { 0.0 };
            current_layer.use_text(
                &format!("{}: {} ({:.1}%)", status, count, pct),
                10.0,
                Mm(MARGIN_MM + 5.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= LINE_HEIGHT_MM;
        }

        y_pos -= LINE_HEIGHT_MM;

        // Table header
        current_layer.use_text("Control Details", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        // Column headers
        current_layer.use_text("Code", 9.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        current_layer.use_text("Name", 9.0, Mm(MARGIN_MM + 25.0), Mm(y_pos), &font);
        current_layer.use_text("Type", 9.0, Mm(MARGIN_MM + 85.0), Mm(y_pos), &font);
        current_layer.use_text("Status", 9.0, Mm(MARGIN_MM + 115.0), Mm(y_pos), &font);
        current_layer.use_text("Reqs", 9.0, Mm(MARGIN_MM + 145.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;

        // Data rows
        for row in rows.iter().take(40) {
            if y_pos < Self::content_end_y() {
                break; // Would need pagination for more rows
            }

            let name_truncated = if row.name.len() > 35 {
                format!("{}...", &row.name[..32])
            } else {
                row.name.clone()
            };

            current_layer.use_text(&row.code, 8.0, Mm(MARGIN_MM), Mm(y_pos), &font);
            current_layer.use_text(&name_truncated, 8.0, Mm(MARGIN_MM + 25.0), Mm(y_pos), &font);
            current_layer.use_text(
                row.control_type.as_deref().unwrap_or("-"),
                8.0,
                Mm(MARGIN_MM + 85.0),
                Mm(y_pos),
                &font,
            );
            current_layer.use_text(
                row.status.as_deref().unwrap_or("-"),
                8.0,
                Mm(MARGIN_MM + 115.0),
                Mm(y_pos),
                &font,
            );
            current_layer.use_text(
                &row.requirement_count.to_string(),
                8.0,
                Mm(MARGIN_MM + 145.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= LINE_HEIGHT_MM;
        }

        if rows.len() > 40 {
            y_pos -= LINE_HEIGHT_MM;
            current_layer.use_text(
                &format!("... and {} more controls (see CSV export for full list)", rows.len() - 40),
                8.0,
                Mm(MARGIN_MM),
                Mm(y_pos),
                &font,
            );
        }

        Self::add_footer(&current_layer, 1, 1, &font);

        let mut buf = BufWriter::new(Vec::new());
        doc.save(&mut buf)?;

        Ok(buf.into_inner()?)
    }

    /// Generate risks report as PDF
    pub async fn generate_risks_pdf(&self, org_id: Uuid) -> AppResult<Vec<u8>> {
        let org_name = self.get_org_name(org_id).await;

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

        // Summary: count by risk level
        let (critical, high, medium, low) = rows.iter().fold((0i64, 0i64, 0i64, 0i64), |acc, r| {
            match r.inherent_score {
                s if s >= 20 => (acc.0 + 1, acc.1, acc.2, acc.3),
                s if s >= 12 => (acc.0, acc.1 + 1, acc.2, acc.3),
                s if s >= 6 => (acc.0, acc.1, acc.2 + 1, acc.3),
                _ => (acc.0, acc.1, acc.2, acc.3 + 1),
            }
        });

        let (doc, page1, layer1) = Self::create_pdf_doc("Risk Register Report", &org_name);
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

        let current_layer = doc.get_page(page1).get_layer(layer1);
        Self::add_header(&current_layer, "Risk Register Report", &org_name, &font);

        let mut y_pos = Self::content_start_y();

        // Summary
        current_layer.use_text("Risk Summary", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        current_layer.use_text(&format!("Total Risks: {}", rows.len()), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;
        current_layer.use_text(&format!("Critical (20-25): {}", critical), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;
        current_layer.use_text(&format!("High (12-19): {}", high), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;
        current_layer.use_text(&format!("Medium (6-11): {}", medium), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;
        current_layer.use_text(&format!("Low (1-5): {}", low), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 2.0;

        // Table
        current_layer.use_text("Risk Details", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        current_layer.use_text("Code", 9.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        current_layer.use_text("Title", 9.0, Mm(MARGIN_MM + 20.0), Mm(y_pos), &font);
        current_layer.use_text("Category", 9.0, Mm(MARGIN_MM + 80.0), Mm(y_pos), &font);
        current_layer.use_text("Score", 9.0, Mm(MARGIN_MM + 115.0), Mm(y_pos), &font);
        current_layer.use_text("Status", 9.0, Mm(MARGIN_MM + 135.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;

        for row in rows.iter().take(35) {
            if y_pos < Self::content_end_y() {
                break;
            }

            let title_truncated = if row.title.len() > 30 {
                format!("{}...", &row.title[..27])
            } else {
                row.title.clone()
            };

            current_layer.use_text(&row.code, 8.0, Mm(MARGIN_MM), Mm(y_pos), &font);
            current_layer.use_text(&title_truncated, 8.0, Mm(MARGIN_MM + 20.0), Mm(y_pos), &font);
            current_layer.use_text(
                row.category.as_deref().unwrap_or("-"),
                8.0,
                Mm(MARGIN_MM + 80.0),
                Mm(y_pos),
                &font,
            );
            current_layer.use_text(
                &row.inherent_score.to_string(),
                8.0,
                Mm(MARGIN_MM + 115.0),
                Mm(y_pos),
                &font,
            );
            current_layer.use_text(
                row.status.as_deref().unwrap_or("-"),
                8.0,
                Mm(MARGIN_MM + 135.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= LINE_HEIGHT_MM;
        }

        Self::add_footer(&current_layer, 1, 1, &font);

        let mut buf = BufWriter::new(Vec::new());
        doc.save(&mut buf)?;

        Ok(buf.into_inner()?)
    }

    /// Generate evidence report as PDF
    pub async fn generate_evidence_pdf(&self, org_id: Uuid) -> AppResult<Vec<u8>> {
        let org_name = self.get_org_name(org_id).await;

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

        // Summary by source
        let mut source_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for row in &rows {
            let source = row.source.as_deref().unwrap_or("Manual").to_string();
            *source_counts.entry(source).or_insert(0) += 1;
        }

        let (doc, page1, layer1) = Self::create_pdf_doc("Evidence Summary Report", &org_name);
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

        let current_layer = doc.get_page(page1).get_layer(layer1);
        Self::add_header(&current_layer, "Evidence Summary Report", &org_name, &font);

        let mut y_pos = Self::content_start_y();

        current_layer.use_text("Evidence Summary", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        current_layer.use_text(&format!("Total Evidence Items: {}", rows.len()), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;

        for (source, count) in &source_counts {
            current_layer.use_text(&format!("{}: {}", source, count), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
            y_pos -= LINE_HEIGHT_MM;
        }
        y_pos -= LINE_HEIGHT_MM;

        current_layer.use_text("Evidence Details", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        current_layer.use_text("Title", 9.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        current_layer.use_text("Type", 9.0, Mm(MARGIN_MM + 70.0), Mm(y_pos), &font);
        current_layer.use_text("Source", 9.0, Mm(MARGIN_MM + 100.0), Mm(y_pos), &font);
        current_layer.use_text("Valid Until", 9.0, Mm(MARGIN_MM + 130.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;

        for row in rows.iter().take(35) {
            if y_pos < Self::content_end_y() {
                break;
            }

            let title_truncated = if row.title.len() > 40 {
                format!("{}...", &row.title[..37])
            } else {
                row.title.clone()
            };

            current_layer.use_text(&title_truncated, 8.0, Mm(MARGIN_MM), Mm(y_pos), &font);
            current_layer.use_text(
                row.evidence_type.as_deref().unwrap_or("-"),
                8.0,
                Mm(MARGIN_MM + 70.0),
                Mm(y_pos),
                &font,
            );
            current_layer.use_text(
                row.source.as_deref().unwrap_or("-"),
                8.0,
                Mm(MARGIN_MM + 100.0),
                Mm(y_pos),
                &font,
            );
            current_layer.use_text(
                &row.valid_until.map_or("-".to_string(), |d| d.format("%Y-%m-%d").to_string()),
                8.0,
                Mm(MARGIN_MM + 130.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= LINE_HEIGHT_MM;
        }

        Self::add_footer(&current_layer, 1, 1, &font);

        let mut buf = BufWriter::new(Vec::new());
        doc.save(&mut buf)?;

        Ok(buf.into_inner()?)
    }

    /// Generate policies report as PDF
    pub async fn generate_policies_pdf(&self, org_id: Uuid) -> AppResult<Vec<u8>> {
        let org_name = self.get_org_name(org_id).await;

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

        // Summary by status
        let mut status_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for row in &rows {
            let status = row.status.as_deref().unwrap_or("Draft").to_string();
            *status_counts.entry(status).or_insert(0) += 1;
        }

        let (doc, page1, layer1) = Self::create_pdf_doc("Policy Acknowledgments Report", &org_name);
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

        let current_layer = doc.get_page(page1).get_layer(layer1);
        Self::add_header(&current_layer, "Policy Acknowledgments Report", &org_name, &font);

        let mut y_pos = Self::content_start_y();

        current_layer.use_text("Policy Summary", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        current_layer.use_text(&format!("Total Policies: {}", rows.len()), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;

        for (status, count) in &status_counts {
            current_layer.use_text(&format!("{}: {}", status, count), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
            y_pos -= LINE_HEIGHT_MM;
        }
        y_pos -= LINE_HEIGHT_MM;

        current_layer.use_text("Policy Details", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        current_layer.use_text("Code", 9.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        current_layer.use_text("Title", 9.0, Mm(MARGIN_MM + 25.0), Mm(y_pos), &font);
        current_layer.use_text("Status", 9.0, Mm(MARGIN_MM + 95.0), Mm(y_pos), &font);
        current_layer.use_text("Ver", 9.0, Mm(MARGIN_MM + 125.0), Mm(y_pos), &font);
        current_layer.use_text("Acks", 9.0, Mm(MARGIN_MM + 145.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;

        for row in rows.iter().take(35) {
            if y_pos < Self::content_end_y() {
                break;
            }

            let title_truncated = if row.title.len() > 40 {
                format!("{}...", &row.title[..37])
            } else {
                row.title.clone()
            };

            current_layer.use_text(&row.code, 8.0, Mm(MARGIN_MM), Mm(y_pos), &font);
            current_layer.use_text(&title_truncated, 8.0, Mm(MARGIN_MM + 25.0), Mm(y_pos), &font);
            current_layer.use_text(
                row.status.as_deref().unwrap_or("-"),
                8.0,
                Mm(MARGIN_MM + 95.0),
                Mm(y_pos),
                &font,
            );
            current_layer.use_text(
                &row.version.unwrap_or(1).to_string(),
                8.0,
                Mm(MARGIN_MM + 125.0),
                Mm(y_pos),
                &font,
            );
            current_layer.use_text(
                &row.acknowledgment_count.to_string(),
                8.0,
                Mm(MARGIN_MM + 145.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= LINE_HEIGHT_MM;
        }

        Self::add_footer(&current_layer, 1, 1, &font);

        let mut buf = BufWriter::new(Vec::new());
        doc.save(&mut buf)?;

        Ok(buf.into_inner()?)
    }

    /// Generate vendors report as PDF
    pub async fn generate_vendors_pdf(&self, org_id: Uuid) -> AppResult<Vec<u8>> {
        let org_name = self.get_org_name(org_id).await;

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

        // Summary by criticality
        let mut criticality_counts: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
        for row in &rows {
            let crit = row.criticality.as_deref().unwrap_or("Unknown").to_string();
            *criticality_counts.entry(crit).or_insert(0) += 1;
        }

        let (doc, page1, layer1) = Self::create_pdf_doc("Vendor Risk Report", &org_name);
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

        let current_layer = doc.get_page(page1).get_layer(layer1);
        Self::add_header(&current_layer, "Vendor Risk Report", &org_name, &font);

        let mut y_pos = Self::content_start_y();

        current_layer.use_text("Vendor Summary", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        current_layer.use_text(&format!("Total Vendors: {}", rows.len()), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;

        for (crit, count) in &criticality_counts {
            current_layer.use_text(&format!("{}: {}", crit, count), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
            y_pos -= LINE_HEIGHT_MM;
        }
        y_pos -= LINE_HEIGHT_MM;

        current_layer.use_text("Vendor Details", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        current_layer.use_text("Name", 9.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        current_layer.use_text("Category", 9.0, Mm(MARGIN_MM + 50.0), Mm(y_pos), &font);
        current_layer.use_text("Criticality", 9.0, Mm(MARGIN_MM + 90.0), Mm(y_pos), &font);
        current_layer.use_text("Status", 9.0, Mm(MARGIN_MM + 125.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;

        for row in rows.iter().take(35) {
            if y_pos < Self::content_end_y() {
                break;
            }

            let name_truncated = if row.name.len() > 25 {
                format!("{}...", &row.name[..22])
            } else {
                row.name.clone()
            };

            current_layer.use_text(&name_truncated, 8.0, Mm(MARGIN_MM), Mm(y_pos), &font);
            current_layer.use_text(
                row.category.as_deref().unwrap_or("-"),
                8.0,
                Mm(MARGIN_MM + 50.0),
                Mm(y_pos),
                &font,
            );
            current_layer.use_text(
                row.criticality.as_deref().unwrap_or("-"),
                8.0,
                Mm(MARGIN_MM + 90.0),
                Mm(y_pos),
                &font,
            );
            current_layer.use_text(
                row.status.as_deref().unwrap_or("-"),
                8.0,
                Mm(MARGIN_MM + 125.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= LINE_HEIGHT_MM;
        }

        Self::add_footer(&current_layer, 1, 1, &font);

        let mut buf = BufWriter::new(Vec::new());
        doc.save(&mut buf)?;

        Ok(buf.into_inner()?)
    }

    /// Generate compliance posture report as PDF
    pub async fn generate_compliance_posture_pdf(&self, org_id: Uuid) -> AppResult<Vec<u8>> {
        let org_name = self.get_org_name(org_id).await;

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

        let (doc, page1, layer1) = Self::create_pdf_doc("Compliance Posture Report", &org_name);
        let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;

        let current_layer = doc.get_page(page1).get_layer(layer1);
        Self::add_header(&current_layer, "Compliance Posture Report", &org_name, &font);

        let mut y_pos = Self::content_start_y();

        // Overall summary
        let total_reqs: i64 = rows.iter().map(|r| r.total_requirements).sum();
        let covered_reqs: i64 = rows.iter().map(|r| r.covered_requirements).sum();
        let overall_pct = if total_reqs > 0 { (covered_reqs as f64 / total_reqs as f64) * 100.0 } else { 0.0 };

        current_layer.use_text("Overall Compliance Summary", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        current_layer.use_text(&format!("Total Requirements: {}", total_reqs), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;
        current_layer.use_text(&format!("Requirements Covered: {}", covered_reqs), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;
        current_layer.use_text(&format!("Overall Coverage: {:.1}%", overall_pct), 10.0, Mm(MARGIN_MM + 5.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 2.0;

        // Framework breakdown
        current_layer.use_text("Framework Coverage", 12.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM * 1.5;

        current_layer.use_text("Framework", 9.0, Mm(MARGIN_MM), Mm(y_pos), &font);
        current_layer.use_text("Total Reqs", 9.0, Mm(MARGIN_MM + 70.0), Mm(y_pos), &font);
        current_layer.use_text("Covered", 9.0, Mm(MARGIN_MM + 100.0), Mm(y_pos), &font);
        current_layer.use_text("Coverage %", 9.0, Mm(MARGIN_MM + 130.0), Mm(y_pos), &font);
        y_pos -= LINE_HEIGHT_MM;

        for row in &rows {
            if y_pos < Self::content_end_y() {
                break;
            }

            let name_truncated = if row.framework_name.len() > 35 {
                format!("{}...", &row.framework_name[..32])
            } else {
                row.framework_name.clone()
            };

            current_layer.use_text(&name_truncated, 8.0, Mm(MARGIN_MM), Mm(y_pos), &font);
            current_layer.use_text(&row.total_requirements.to_string(), 8.0, Mm(MARGIN_MM + 70.0), Mm(y_pos), &font);
            current_layer.use_text(&row.covered_requirements.to_string(), 8.0, Mm(MARGIN_MM + 100.0), Mm(y_pos), &font);
            current_layer.use_text(&format!("{:.1}%", row.coverage_percentage), 8.0, Mm(MARGIN_MM + 130.0), Mm(y_pos), &font);
            y_pos -= LINE_HEIGHT_MM;

            // Draw simple progress bar
            let bar_width: f32 = 50.0;
            let filled_width: f32 = ((row.coverage_percentage / 100.0) * bar_width as f64) as f32;

            // Background bar
            let bg_line = Line {
                points: vec![
                    (Point::new(Mm(MARGIN_MM), Mm(y_pos + 2.0)), false),
                    (Point::new(Mm(MARGIN_MM + bar_width), Mm(y_pos + 2.0)), false),
                ],
                is_closed: false,
            };
            current_layer.set_outline_color(Color::Rgb(Rgb::new(0.8, 0.8, 0.8, None)));
            current_layer.set_outline_thickness(3.0);
            current_layer.add_line(bg_line);

            // Filled portion
            if filled_width > 0.0 {
                let fill_line = Line {
                    points: vec![
                        (Point::new(Mm(MARGIN_MM), Mm(y_pos + 2.0)), false),
                        (Point::new(Mm(MARGIN_MM + filled_width), Mm(y_pos + 2.0)), false),
                    ],
                    is_closed: false,
                };
                let color = if row.coverage_percentage >= 80.0 {
                    Color::Rgb(Rgb::new(0.2, 0.7, 0.2, None)) // Green
                } else if row.coverage_percentage >= 50.0 {
                    Color::Rgb(Rgb::new(0.9, 0.7, 0.1, None)) // Yellow
                } else {
                    Color::Rgb(Rgb::new(0.8, 0.2, 0.2, None)) // Red
                };
                current_layer.set_outline_color(color);
                current_layer.add_line(fill_line);
            }

            // Reset line style
            current_layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
            current_layer.set_outline_thickness(1.0);

            y_pos -= LINE_HEIGHT_MM * 1.5;
        }

        Self::add_footer(&current_layer, 1, 1, &font);

        let mut buf = BufWriter::new(Vec::new());
        doc.save(&mut buf)?;

        Ok(buf.into_inner()?)
    }
}
