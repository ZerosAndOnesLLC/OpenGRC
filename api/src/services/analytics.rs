use crate::cache::CacheClient;
use crate::utils::{AppError, AppResult};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Row};
use std::time::Duration;
use uuid::Uuid;

// ============================================================================
// Compliance Snapshots
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceSnapshot {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub snapshot_date: NaiveDate,
    pub snapshot_type: String,
    pub total_frameworks: i32,
    pub total_requirements: i32,
    pub covered_requirements: i32,
    pub framework_coverage_pct: Decimal,
    pub total_controls: i32,
    pub implemented_controls: i32,
    pub partially_implemented_controls: i32,
    pub not_implemented_controls: i32,
    pub control_implementation_pct: Decimal,
    pub controls_tested: i32,
    pub controls_passed: i32,
    pub controls_failed: i32,
    pub control_pass_rate: Decimal,
    pub total_risks: i32,
    pub high_risks: i32,
    pub medium_risks: i32,
    pub low_risks: i32,
    pub average_risk_score: Decimal,
    pub average_residual_score: Decimal,
    pub risks_with_controls: i32,
    pub total_evidence: i32,
    pub valid_evidence: i32,
    pub expiring_evidence: i32,
    pub expired_evidence: i32,
    pub evidence_freshness_score: Decimal,
    pub total_policies: i32,
    pub published_policies: i32,
    pub policies_needing_review: i32,
    pub policy_acknowledgment_rate: Decimal,
    pub total_vendors: i32,
    pub high_risk_vendors: i32,
    pub vendors_assessed_last_year: i32,
    pub total_assets: i32,
    pub assets_with_controls: i32,
    pub total_open_tasks: i32,
    pub overdue_tasks: i32,
    pub active_audits: i32,
    pub open_findings: i32,
    pub overall_compliance_score: Decimal,
    pub framework_details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceTrend {
    pub date: NaiveDate,
    pub compliance_score: Decimal,
    pub framework_coverage: Decimal,
    pub control_implementation: Decimal,
    pub control_pass_rate: Decimal,
    pub evidence_freshness: Decimal,
    pub risk_score: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceTrendResponse {
    pub snapshots: Vec<ComplianceTrend>,
    pub current: Option<ComplianceSnapshot>,
    pub previous_period: Option<ComplianceSnapshot>,
    pub change: ComplianceChange,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComplianceChange {
    pub compliance_score: Decimal,
    pub framework_coverage: Decimal,
    pub control_implementation: Decimal,
    pub control_pass_rate: Decimal,
    pub evidence_freshness: Decimal,
    pub risk_score: Decimal,
}

// ============================================================================
// Risk Predictions
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskPrediction {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub risk_id: Uuid,
    pub current_likelihood: i32,
    pub current_impact: i32,
    pub current_score: i32,
    pub predicted_likelihood: i32,
    pub predicted_impact: i32,
    pub predicted_score: i32,
    pub predicted_90d_score: Option<i32>,
    pub confidence_level: Decimal,
    pub trend: String,
    pub trend_velocity: Option<Decimal>,
    pub factor_scores: serde_json::Value,
    pub explanation: Option<String>,
    pub is_current: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskPredictionWithDetails {
    // From risk_predictions
    pub id: Uuid,
    pub organization_id: Uuid,
    pub risk_id: Uuid,
    pub current_likelihood: i32,
    pub current_impact: i32,
    pub current_score: i32,
    pub predicted_likelihood: i32,
    pub predicted_impact: i32,
    pub predicted_score: i32,
    pub predicted_90d_score: Option<i32>,
    pub confidence_level: Decimal,
    pub trend: String,
    pub trend_velocity: Option<Decimal>,
    pub factor_scores: serde_json::Value,
    pub explanation: Option<String>,
    pub is_current: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    // From risks
    pub risk_title: String,
    pub risk_code: String,
    pub risk_category: String,
    pub risk_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskPredictionSummary {
    pub total_risks: i64,
    pub risks_with_predictions: i64,
    pub increasing_risks: i64,
    pub decreasing_risks: i64,
    pub stable_risks: i64,
    pub avg_confidence: Decimal,
    pub high_confidence_predictions: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskPredictionFactor {
    pub id: Uuid,
    pub organization_id: Option<Uuid>,
    pub name: String,
    pub code: String,
    pub category: String,
    pub weight: Decimal,
    pub description: Option<String>,
    pub is_system: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================================================
// Benchmarks
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IndustryBenchmark {
    pub id: Uuid,
    pub benchmark_name: String,
    pub benchmark_code: String,
    pub industry: Option<String>,
    pub company_size: Option<String>,
    pub framework_id: Option<Uuid>,
    pub avg_framework_coverage: Option<Decimal>,
    pub avg_control_implementation: Option<Decimal>,
    pub avg_control_pass_rate: Option<Decimal>,
    pub avg_risk_score: Option<Decimal>,
    pub avg_evidence_freshness: Option<Decimal>,
    pub avg_policy_acknowledgment: Option<Decimal>,
    pub avg_vendor_assessment_rate: Option<Decimal>,
    pub avg_compliance_score: Option<Decimal>,
    pub p25_compliance_score: Option<Decimal>,
    pub p50_compliance_score: Option<Decimal>,
    pub p75_compliance_score: Option<Decimal>,
    pub p90_compliance_score: Option<Decimal>,
    pub sample_size: i32,
    pub data_as_of: NaiveDate,
    pub valid_until: NaiveDate,
    pub detailed_metrics: serde_json::Value,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct OrganizationBenchmarkComparison {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub benchmark_id: Uuid,
    pub org_framework_coverage: Option<Decimal>,
    pub org_control_implementation: Option<Decimal>,
    pub org_control_pass_rate: Option<Decimal>,
    pub org_risk_score: Option<Decimal>,
    pub org_evidence_freshness: Option<Decimal>,
    pub org_policy_acknowledgment: Option<Decimal>,
    pub org_vendor_assessment_rate: Option<Decimal>,
    pub org_compliance_score: Option<Decimal>,
    pub diff_framework_coverage: Option<Decimal>,
    pub diff_control_implementation: Option<Decimal>,
    pub diff_control_pass_rate: Option<Decimal>,
    pub diff_risk_score: Option<Decimal>,
    pub diff_evidence_freshness: Option<Decimal>,
    pub diff_policy_acknowledgment: Option<Decimal>,
    pub diff_vendor_assessment_rate: Option<Decimal>,
    pub diff_compliance_score: Option<Decimal>,
    pub percentile_rank: Option<Decimal>,
    pub overall_status: String,
    pub detailed_comparison: serde_json::Value,
    pub recommendations: serde_json::Value,
    pub comparison_date: NaiveDate,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BenchmarkComparisonWithDetails {
    // From organization_benchmark_comparisons
    pub id: Uuid,
    pub organization_id: Uuid,
    pub benchmark_id: Uuid,
    pub org_framework_coverage: Option<Decimal>,
    pub org_control_implementation: Option<Decimal>,
    pub org_control_pass_rate: Option<Decimal>,
    pub org_risk_score: Option<Decimal>,
    pub org_evidence_freshness: Option<Decimal>,
    pub org_policy_acknowledgment: Option<Decimal>,
    pub org_vendor_assessment_rate: Option<Decimal>,
    pub org_compliance_score: Option<Decimal>,
    pub diff_framework_coverage: Option<Decimal>,
    pub diff_control_implementation: Option<Decimal>,
    pub diff_control_pass_rate: Option<Decimal>,
    pub diff_risk_score: Option<Decimal>,
    pub diff_evidence_freshness: Option<Decimal>,
    pub diff_policy_acknowledgment: Option<Decimal>,
    pub diff_vendor_assessment_rate: Option<Decimal>,
    pub diff_compliance_score: Option<Decimal>,
    pub percentile_rank: Option<Decimal>,
    pub overall_status: String,
    pub detailed_comparison: serde_json::Value,
    pub recommendations: serde_json::Value,
    pub comparison_date: NaiveDate,
    pub created_at: DateTime<Utc>,
    // From industry_benchmarks
    pub benchmark_name: String,
    pub benchmark_industry: Option<String>,
    pub benchmark_company_size: Option<String>,
}

// ============================================================================
// Custom Reports
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SavedReport {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: serde_json::Value,
    pub layout: String,
    pub chart_config: serde_json::Value,
    pub is_public: bool,
    pub shared_with: serde_json::Value,
    pub schedule_enabled: bool,
    pub schedule_cron: Option<String>,
    pub schedule_recipients: serde_json::Value,
    pub last_scheduled_run: Option<DateTime<Utc>>,
    pub next_scheduled_run: Option<DateTime<Utc>>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: i32,
    pub created_by: Uuid,
    pub updated_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSavedReport {
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub config: serde_json::Value,
    pub layout: Option<String>,
    pub chart_config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub shared_with: Option<Vec<Uuid>>,
    pub schedule_enabled: Option<bool>,
    pub schedule_cron: Option<String>,
    pub schedule_recipients: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSavedReport {
    pub name: Option<String>,
    pub description: Option<String>,
    pub config: Option<serde_json::Value>,
    pub layout: Option<String>,
    pub chart_config: Option<serde_json::Value>,
    pub is_public: Option<bool>,
    pub shared_with: Option<Vec<Uuid>>,
    pub schedule_enabled: Option<bool>,
    pub schedule_cron: Option<String>,
    pub schedule_recipients: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportExecution {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub report_id: Option<Uuid>,
    pub report_name: String,
    pub report_type: String,
    pub report_config: serde_json::Value,
    pub executed_by: Option<Uuid>,
    pub execution_type: String,
    pub applied_filters: serde_json::Value,
    pub date_range_start: Option<NaiveDate>,
    pub date_range_end: Option<NaiveDate>,
    pub rows_returned: i32,
    pub execution_time_ms: i32,
    pub output_format: String,
    pub output_file_path: Option<String>,
    pub status: String,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReportTemplate {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub report_type: String,
    pub category: String,
    pub config: serde_json::Value,
    pub layout: String,
    pub chart_config: serde_json::Value,
    pub preview_image: Option<String>,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// Executive Dashboard
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ExecutiveMetric {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub metric_name: String,
    pub metric_code: String,
    pub category: String,
    pub current_value: Decimal,
    pub previous_value: Option<Decimal>,
    pub change_value: Option<Decimal>,
    pub change_pct: Option<Decimal>,
    pub trend: Option<String>,
    pub target_value: Option<Decimal>,
    pub threshold_warning: Option<Decimal>,
    pub threshold_critical: Option<Decimal>,
    pub status: String,
    pub sparkline_data: serde_json::Value,
    pub display_format: String,
    pub display_order: i32,
    pub is_visible: bool,
    pub computed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DashboardWidget {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub dashboard_type: String,
    pub widget_type: String,
    pub widget_title: String,
    pub config: serde_json::Value,
    pub grid_x: i32,
    pub grid_y: i32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDashboardWidget {
    pub dashboard_type: Option<String>,
    pub widget_type: String,
    pub widget_title: String,
    pub config: serde_json::Value,
    pub grid_x: Option<i32>,
    pub grid_y: Option<i32>,
    pub grid_width: Option<i32>,
    pub grid_height: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDashboardWidget {
    pub widget_title: Option<String>,
    pub config: Option<serde_json::Value>,
    pub grid_x: Option<i32>,
    pub grid_y: Option<i32>,
    pub grid_width: Option<i32>,
    pub grid_height: Option<i32>,
    pub is_visible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutiveDashboard {
    pub metrics: Vec<ExecutiveMetric>,
    pub widgets: Vec<DashboardWidget>,
    pub snapshot: Option<ComplianceSnapshot>,
    pub trend_data: Vec<ComplianceTrend>,
    pub risk_predictions: RiskPredictionSummary,
    pub benchmark: Option<BenchmarkComparisonWithDetails>,
}

// ============================================================================
// Analytics Service
// ============================================================================

#[derive(Clone)]
pub struct AnalyticsService {
    db: PgPool,
    cache: CacheClient,
}

impl AnalyticsService {
    pub fn new(db: PgPool, cache: CacheClient) -> Self {
        Self { db, cache }
    }

    // ========================================================================
    // Compliance Snapshots & Trends
    // ========================================================================

    /// Capture a compliance snapshot for an organization
    pub async fn capture_snapshot(&self, org_id: Uuid, snapshot_type: &str) -> AppResult<Uuid> {
        let result: (Uuid,) = sqlx::query_as(
            "SELECT capture_compliance_snapshot($1, $2)"
        )
        .bind(org_id)
        .bind(snapshot_type)
        .fetch_one(&self.db)
        .await?;

        // Invalidate cache
        self.cache.delete(&format!("analytics:snapshot:current:{}", org_id)).await.ok();
        self.cache.delete(&format!("analytics:trends:{}", org_id)).await.ok();

        Ok(result.0)
    }

    /// Get current snapshot
    pub async fn get_current_snapshot(&self, org_id: Uuid) -> AppResult<Option<ComplianceSnapshot>> {
        let cache_key = format!("analytics:snapshot:current:{}", org_id);
        if let Some(cached) = self.cache.get::<ComplianceSnapshot>(&cache_key).await? {
            return Ok(Some(cached));
        }

        let snapshot: Option<ComplianceSnapshot> = sqlx::query_as(
            r#"
            SELECT * FROM compliance_snapshots
            WHERE organization_id = $1
            ORDER BY snapshot_date DESC
            LIMIT 1
            "#
        )
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(ref s) = snapshot {
            self.cache.set(&cache_key, s, Some(Duration::from_secs(300))).await.ok();
        }

        Ok(snapshot)
    }

    /// Get compliance trends for a date range
    pub async fn get_compliance_trends(
        &self,
        org_id: Uuid,
        days: i32,
    ) -> AppResult<ComplianceTrendResponse> {
        let cache_key = format!("analytics:trends:{}:{}", org_id, days);
        if let Some(cached) = self.cache.get::<ComplianceTrendResponse>(&cache_key).await? {
            return Ok(cached);
        }

        // Get snapshots for the period
        let snapshots: Vec<ComplianceSnapshot> = sqlx::query_as(
            r#"
            SELECT * FROM compliance_snapshots
            WHERE organization_id = $1
              AND snapshot_date >= CURRENT_DATE - $2::INTEGER
            ORDER BY snapshot_date ASC
            "#
        )
        .bind(org_id)
        .bind(days)
        .fetch_all(&self.db)
        .await?;

        let trends: Vec<ComplianceTrend> = snapshots.iter().map(|s| ComplianceTrend {
            date: s.snapshot_date,
            compliance_score: s.overall_compliance_score,
            framework_coverage: s.framework_coverage_pct,
            control_implementation: s.control_implementation_pct,
            control_pass_rate: s.control_pass_rate,
            evidence_freshness: s.evidence_freshness_score,
            risk_score: s.average_risk_score,
        }).collect();

        let current = snapshots.last().cloned();
        let previous = if snapshots.len() > 1 {
            snapshots.get(snapshots.len().saturating_sub(days as usize / 2)).cloned()
        } else {
            None
        };

        let change = if let (Some(ref curr), Some(ref prev)) = (&current, &previous) {
            ComplianceChange {
                compliance_score: curr.overall_compliance_score - prev.overall_compliance_score,
                framework_coverage: curr.framework_coverage_pct - prev.framework_coverage_pct,
                control_implementation: curr.control_implementation_pct - prev.control_implementation_pct,
                control_pass_rate: curr.control_pass_rate - prev.control_pass_rate,
                evidence_freshness: curr.evidence_freshness_score - prev.evidence_freshness_score,
                risk_score: curr.average_risk_score - prev.average_risk_score,
            }
        } else {
            ComplianceChange::default()
        };

        let response = ComplianceTrendResponse {
            snapshots: trends,
            current,
            previous_period: previous,
            change,
        };

        self.cache.set(&cache_key, &response, Some(Duration::from_secs(300))).await.ok();

        Ok(response)
    }

    // ========================================================================
    // Risk Predictions
    // ========================================================================

    /// Get all current risk predictions for an organization
    pub async fn get_risk_predictions(&self, org_id: Uuid) -> AppResult<Vec<RiskPredictionWithDetails>> {
        let predictions: Vec<RiskPredictionWithDetails> = sqlx::query_as(
            r#"
            SELECT
                rp.*,
                r.title AS risk_title,
                r.code AS risk_code,
                r.category AS risk_category,
                r.status AS risk_status
            FROM risk_predictions rp
            JOIN risks r ON rp.risk_id = r.id
            WHERE rp.organization_id = $1
              AND rp.is_current = TRUE
            ORDER BY rp.predicted_score DESC
            "#
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(predictions)
    }

    /// Get prediction for a specific risk
    pub async fn get_risk_prediction(&self, org_id: Uuid, risk_id: Uuid) -> AppResult<Option<RiskPrediction>> {
        let prediction: Option<RiskPrediction> = sqlx::query_as(
            r#"
            SELECT * FROM risk_predictions
            WHERE organization_id = $1
              AND risk_id = $2
              AND is_current = TRUE
            "#
        )
        .bind(org_id)
        .bind(risk_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(prediction)
    }

    /// Recompute prediction for a risk
    pub async fn recompute_risk_prediction(&self, risk_id: Uuid) -> AppResult<Option<Uuid>> {
        let result: Option<(Uuid,)> = sqlx::query_as(
            "SELECT compute_risk_prediction($1)"
        )
        .bind(risk_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(result.map(|r| r.0))
    }

    /// Get risk prediction summary
    pub async fn get_risk_prediction_summary(&self, org_id: Uuid) -> AppResult<RiskPredictionSummary> {
        let summary: RiskPredictionSummary = sqlx::query_as(
            r#"
            SELECT
                (SELECT COUNT(*) FROM risks WHERE organization_id = $1) AS total_risks,
                COUNT(*) AS risks_with_predictions,
                COUNT(*) FILTER (WHERE trend = 'increasing') AS increasing_risks,
                COUNT(*) FILTER (WHERE trend = 'decreasing') AS decreasing_risks,
                COUNT(*) FILTER (WHERE trend = 'stable') AS stable_risks,
                COALESCE(AVG(confidence_level), 0) AS avg_confidence,
                COUNT(*) FILTER (WHERE confidence_level >= 70) AS high_confidence_predictions
            FROM risk_predictions
            WHERE organization_id = $1
              AND is_current = TRUE
            "#
        )
        .bind(org_id)
        .fetch_one(&self.db)
        .await?;

        Ok(summary)
    }

    /// Get risk prediction factors
    pub async fn get_prediction_factors(&self, org_id: Uuid) -> AppResult<Vec<RiskPredictionFactor>> {
        let factors: Vec<RiskPredictionFactor> = sqlx::query_as(
            r#"
            SELECT * FROM risk_prediction_factors
            WHERE (organization_id = $1 OR organization_id IS NULL)
              AND is_active = TRUE
            ORDER BY is_system DESC, weight DESC
            "#
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        Ok(factors)
    }

    // ========================================================================
    // Benchmarks
    // ========================================================================

    /// Get all available benchmarks
    pub async fn get_available_benchmarks(&self) -> AppResult<Vec<IndustryBenchmark>> {
        let benchmarks: Vec<IndustryBenchmark> = sqlx::query_as(
            r#"
            SELECT * FROM industry_benchmarks
            WHERE is_active = TRUE
              AND valid_until >= CURRENT_DATE
            ORDER BY industry NULLS LAST, company_size NULLS LAST
            "#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(benchmarks)
    }

    /// Compare organization against a benchmark
    pub async fn compare_to_benchmark(
        &self,
        org_id: Uuid,
        benchmark_id: Uuid,
    ) -> AppResult<BenchmarkComparisonWithDetails> {
        // Get current snapshot
        let snapshot = self.get_current_snapshot(org_id).await?
            .ok_or_else(|| AppError::NotFound("No compliance snapshot found. Run a snapshot first.".to_string()))?;

        // Get benchmark
        let benchmark: IndustryBenchmark = sqlx::query_as(
            "SELECT * FROM industry_benchmarks WHERE id = $1"
        )
        .bind(benchmark_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Benchmark not found".to_string()))?;

        // Calculate differences
        let org_framework_coverage = snapshot.framework_coverage_pct;
        let org_control_implementation = snapshot.control_implementation_pct;
        let org_control_pass_rate = snapshot.control_pass_rate;
        let org_risk_score = snapshot.average_risk_score;
        let org_evidence_freshness = snapshot.evidence_freshness_score;
        let org_policy_acknowledgment = snapshot.policy_acknowledgment_rate;
        let org_compliance_score = snapshot.overall_compliance_score;

        let diff_framework_coverage = benchmark.avg_framework_coverage.map(|b| org_framework_coverage - b);
        let diff_control_implementation = benchmark.avg_control_implementation.map(|b| org_control_implementation - b);
        let diff_control_pass_rate = benchmark.avg_control_pass_rate.map(|b| org_control_pass_rate - b);
        let diff_risk_score = benchmark.avg_risk_score.map(|b| org_risk_score - b);
        let diff_evidence_freshness = benchmark.avg_evidence_freshness.map(|b| org_evidence_freshness - b);
        let diff_policy_acknowledgment = benchmark.avg_policy_acknowledgment.map(|b| org_policy_acknowledgment - b);
        let diff_compliance_score = benchmark.avg_compliance_score.map(|b| org_compliance_score - b);

        // Calculate percentile rank
        let percentile_rank = if let (Some(p25), Some(p50), Some(p75), Some(p90)) = (
            benchmark.p25_compliance_score,
            benchmark.p50_compliance_score,
            benchmark.p75_compliance_score,
            benchmark.p90_compliance_score,
        ) {
            let score = org_compliance_score;
            if score >= p90 { Some(Decimal::from(95)) }
            else if score >= p75 { Some(Decimal::from(87)) }
            else if score >= p50 { Some(Decimal::from(62)) }
            else if score >= p25 { Some(Decimal::from(37)) }
            else { Some(Decimal::from(12)) }
        } else {
            None
        };

        // Determine status
        let overall_status = match diff_compliance_score {
            Some(d) if d >= Decimal::from(10) => "above_average",
            Some(d) if d >= Decimal::ZERO => "average",
            Some(d) if d >= Decimal::from(-10) => "below_average",
            _ => "needs_attention",
        }.to_string();

        // Generate recommendations
        let mut recommendations: Vec<serde_json::Value> = Vec::new();
        if diff_framework_coverage.map(|d| d < Decimal::ZERO).unwrap_or(false) {
            recommendations.push(serde_json::json!({
                "area": "Framework Coverage",
                "priority": "high",
                "recommendation": "Increase control mapping to framework requirements to improve coverage"
            }));
        }
        if diff_control_implementation.map(|d| d < Decimal::ZERO).unwrap_or(false) {
            recommendations.push(serde_json::json!({
                "area": "Control Implementation",
                "priority": "high",
                "recommendation": "Prioritize implementation of controls marked as 'not implemented'"
            }));
        }
        if diff_evidence_freshness.map(|d| d < Decimal::ZERO).unwrap_or(false) {
            recommendations.push(serde_json::json!({
                "area": "Evidence Freshness",
                "priority": "medium",
                "recommendation": "Review and update evidence that is expiring or expired"
            }));
        }

        // Insert comparison record and join with benchmark for full details
        let result: BenchmarkComparisonWithDetails = sqlx::query_as(
            r#"
            WITH inserted AS (
                INSERT INTO organization_benchmark_comparisons (
                    organization_id, benchmark_id,
                    org_framework_coverage, org_control_implementation, org_control_pass_rate,
                    org_risk_score, org_evidence_freshness, org_policy_acknowledgment, org_compliance_score,
                    diff_framework_coverage, diff_control_implementation, diff_control_pass_rate,
                    diff_risk_score, diff_evidence_freshness, diff_policy_acknowledgment, diff_compliance_score,
                    percentile_rank, overall_status, recommendations
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19)
                RETURNING *
            )
            SELECT
                i.*,
                ib.benchmark_name,
                ib.industry AS benchmark_industry,
                ib.company_size AS benchmark_company_size
            FROM inserted i
            JOIN industry_benchmarks ib ON i.benchmark_id = ib.id
            "#
        )
        .bind(org_id)
        .bind(benchmark_id)
        .bind(org_framework_coverage)
        .bind(org_control_implementation)
        .bind(org_control_pass_rate)
        .bind(org_risk_score)
        .bind(org_evidence_freshness)
        .bind(org_policy_acknowledgment)
        .bind(org_compliance_score)
        .bind(diff_framework_coverage)
        .bind(diff_control_implementation)
        .bind(diff_control_pass_rate)
        .bind(diff_risk_score)
        .bind(diff_evidence_freshness)
        .bind(diff_policy_acknowledgment)
        .bind(diff_compliance_score)
        .bind(percentile_rank)
        .bind(&overall_status)
        .bind(serde_json::Value::Array(recommendations))
        .fetch_one(&self.db)
        .await?;

        Ok(result)
    }

    /// Get latest benchmark comparison for org
    pub async fn get_latest_benchmark_comparison(&self, org_id: Uuid) -> AppResult<Option<BenchmarkComparisonWithDetails>> {
        let comparison: Option<BenchmarkComparisonWithDetails> = sqlx::query_as(
            r#"
            SELECT
                obc.*,
                ib.benchmark_name,
                ib.industry AS benchmark_industry,
                ib.company_size AS benchmark_company_size
            FROM organization_benchmark_comparisons obc
            JOIN industry_benchmarks ib ON obc.benchmark_id = ib.id
            WHERE obc.organization_id = $1
            ORDER BY obc.comparison_date DESC
            LIMIT 1
            "#
        )
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?;

        Ok(comparison)
    }

    // ========================================================================
    // Custom Reports
    // ========================================================================

    /// List saved reports for an organization
    pub async fn list_saved_reports(&self, org_id: Uuid, user_id: Uuid) -> AppResult<Vec<SavedReport>> {
        let reports: Vec<SavedReport> = sqlx::query_as(
            r#"
            SELECT * FROM saved_reports
            WHERE organization_id = $1
              AND (created_by = $2 OR is_public = TRUE OR shared_with @> to_jsonb($2::TEXT))
            ORDER BY updated_at DESC
            "#
        )
        .bind(org_id)
        .bind(user_id)
        .fetch_all(&self.db)
        .await?;

        Ok(reports)
    }

    /// Get a saved report
    pub async fn get_saved_report(&self, org_id: Uuid, report_id: Uuid) -> AppResult<SavedReport> {
        let report: SavedReport = sqlx::query_as(
            "SELECT * FROM saved_reports WHERE id = $1 AND organization_id = $2"
        )
        .bind(report_id)
        .bind(org_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Report not found".to_string()))?;

        Ok(report)
    }

    /// Create a saved report
    pub async fn create_saved_report(
        &self,
        org_id: Uuid,
        user_id: Uuid,
        input: CreateSavedReport,
    ) -> AppResult<SavedReport> {
        let report: SavedReport = sqlx::query_as(
            r#"
            INSERT INTO saved_reports (
                organization_id, name, description, report_type, config, layout,
                chart_config, is_public, shared_with, schedule_enabled, schedule_cron,
                schedule_recipients, created_by
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.report_type)
        .bind(&input.config)
        .bind(input.layout.as_deref().unwrap_or("table"))
        .bind(input.chart_config.clone().unwrap_or(serde_json::json!({})))
        .bind(input.is_public.unwrap_or(false))
        .bind(serde_json::to_value(&input.shared_with.unwrap_or_default()).unwrap_or(serde_json::json!([])))
        .bind(input.schedule_enabled.unwrap_or(false))
        .bind(&input.schedule_cron)
        .bind(serde_json::to_value(&input.schedule_recipients.unwrap_or_default()).unwrap_or(serde_json::json!([])))
        .bind(user_id)
        .fetch_one(&self.db)
        .await?;

        Ok(report)
    }

    /// Update a saved report
    pub async fn update_saved_report(
        &self,
        org_id: Uuid,
        report_id: Uuid,
        user_id: Uuid,
        input: UpdateSavedReport,
    ) -> AppResult<SavedReport> {
        let report: SavedReport = sqlx::query_as(
            r#"
            UPDATE saved_reports SET
                name = COALESCE($3, name),
                description = COALESCE($4, description),
                config = COALESCE($5, config),
                layout = COALESCE($6, layout),
                chart_config = COALESCE($7, chart_config),
                is_public = COALESCE($8, is_public),
                shared_with = COALESCE($9, shared_with),
                schedule_enabled = COALESCE($10, schedule_enabled),
                schedule_cron = COALESCE($11, schedule_cron),
                schedule_recipients = COALESCE($12, schedule_recipients),
                updated_by = $13
            WHERE id = $1 AND organization_id = $2
            RETURNING *
            "#
        )
        .bind(report_id)
        .bind(org_id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.config)
        .bind(&input.layout)
        .bind(&input.chart_config)
        .bind(input.is_public)
        .bind(input.shared_with.as_ref().map(|v| serde_json::to_value(v).unwrap_or(serde_json::json!([]))))
        .bind(input.schedule_enabled)
        .bind(&input.schedule_cron)
        .bind(input.schedule_recipients.as_ref().map(|v| serde_json::to_value(v).unwrap_or(serde_json::json!([]))))
        .bind(user_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Report not found".to_string()))?;

        Ok(report)
    }

    /// Delete a saved report
    pub async fn delete_saved_report(&self, org_id: Uuid, report_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            "DELETE FROM saved_reports WHERE id = $1 AND organization_id = $2"
        )
        .bind(report_id)
        .bind(org_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Report not found".to_string()));
        }

        Ok(())
    }

    /// Get report templates
    pub async fn get_report_templates(&self) -> AppResult<Vec<ReportTemplate>> {
        let templates: Vec<ReportTemplate> = sqlx::query_as(
            r#"
            SELECT * FROM report_templates
            WHERE is_active = TRUE
            ORDER BY sort_order, name
            "#
        )
        .fetch_all(&self.db)
        .await?;

        Ok(templates)
    }

    /// Record report execution
    pub async fn record_report_execution(
        &self,
        org_id: Uuid,
        report_id: Option<Uuid>,
        report_name: &str,
        report_type: &str,
        config: &serde_json::Value,
        user_id: Option<Uuid>,
        execution_type: &str,
        rows_returned: i32,
        execution_time_ms: i32,
        output_format: &str,
    ) -> AppResult<ReportExecution> {
        let execution: ReportExecution = sqlx::query_as(
            r#"
            INSERT INTO report_executions (
                organization_id, report_id, report_name, report_type, report_config,
                executed_by, execution_type, rows_returned, execution_time_ms, output_format
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(report_id)
        .bind(report_name)
        .bind(report_type)
        .bind(config)
        .bind(user_id)
        .bind(execution_type)
        .bind(rows_returned)
        .bind(execution_time_ms)
        .bind(output_format)
        .fetch_one(&self.db)
        .await?;

        // Update run count on saved report if applicable
        if let Some(rid) = report_id {
            sqlx::query(
                "UPDATE saved_reports SET run_count = run_count + 1, last_run_at = NOW() WHERE id = $1"
            )
            .bind(rid)
            .execute(&self.db)
            .await
            .ok();
        }

        Ok(execution)
    }

    // ========================================================================
    // Executive Dashboard
    // ========================================================================

    /// Get or compute executive metrics
    pub async fn get_executive_metrics(&self, org_id: Uuid) -> AppResult<Vec<ExecutiveMetric>> {
        // Check if we have recent metrics (computed within last hour)
        let metrics: Vec<ExecutiveMetric> = sqlx::query_as(
            r#"
            SELECT * FROM executive_metrics
            WHERE organization_id = $1
              AND is_visible = TRUE
              AND computed_at > NOW() - INTERVAL '1 hour'
            ORDER BY display_order
            "#
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        if !metrics.is_empty() {
            return Ok(metrics);
        }

        // Compute and store new metrics
        self.compute_executive_metrics(org_id).await
    }

    /// Compute executive metrics from current data
    async fn compute_executive_metrics(&self, org_id: Uuid) -> AppResult<Vec<ExecutiveMetric>> {
        // Get current snapshot or create one
        let snapshot = match self.get_current_snapshot(org_id).await? {
            Some(s) => s,
            None => {
                self.capture_snapshot(org_id, "daily").await?;
                self.get_current_snapshot(org_id).await?.unwrap()
            }
        };

        // Get historical data for sparklines (last 30 days)
        let historical: Vec<ComplianceSnapshot> = sqlx::query_as(
            r#"
            SELECT * FROM compliance_snapshots
            WHERE organization_id = $1
              AND snapshot_date >= CURRENT_DATE - 30
            ORDER BY snapshot_date
            "#
        )
        .bind(org_id)
        .fetch_all(&self.db)
        .await?;

        // Define metrics to compute
        let metric_definitions = vec![
            ("Overall Compliance Score", "overall_compliance", "compliance", snapshot.overall_compliance_score, "percentage", 80, 70, 50),
            ("Framework Coverage", "framework_coverage", "compliance", snapshot.framework_coverage_pct, "percentage", 80, 65, 40),
            ("Control Implementation", "control_implementation", "operations", snapshot.control_implementation_pct, "percentage", 85, 70, 50),
            ("Control Pass Rate", "control_pass_rate", "operations", snapshot.control_pass_rate, "percentage", 90, 75, 60),
            ("Evidence Freshness", "evidence_freshness", "operations", snapshot.evidence_freshness_score, "percentage", 80, 60, 40),
            ("Average Risk Score", "avg_risk_score", "risk", snapshot.average_risk_score, "score", 8, 12, 18),
            ("High Risks", "high_risks", "risk", Decimal::from(snapshot.high_risks), "number", 0, 3, 8),
            ("Open Findings", "open_findings", "compliance", Decimal::from(snapshot.open_findings), "number", 0, 5, 15),
            ("Overdue Tasks", "overdue_tasks", "operations", Decimal::from(snapshot.overdue_tasks), "number", 0, 5, 15),
        ];

        let mut metrics = Vec::new();
        let mut order = 0;

        for (name, code, category, value, format, target, warn, crit) in metric_definitions {
            // Get previous value
            let previous_value = historical.first().map(|s| match code {
                "overall_compliance" => s.overall_compliance_score,
                "framework_coverage" => s.framework_coverage_pct,
                "control_implementation" => s.control_implementation_pct,
                "control_pass_rate" => s.control_pass_rate,
                "evidence_freshness" => s.evidence_freshness_score,
                "avg_risk_score" => s.average_risk_score,
                "high_risks" => Decimal::from(s.high_risks),
                "open_findings" => Decimal::from(s.open_findings),
                "overdue_tasks" => Decimal::from(s.overdue_tasks),
                _ => Decimal::ZERO,
            });

            let change_value = previous_value.map(|p| value - p);
            let change_pct = previous_value.and_then(|p| {
                if p != Decimal::ZERO {
                    Some((value - p) / p * Decimal::from(100))
                } else {
                    None
                }
            });

            let trend = change_value.map(|c| {
                if c > Decimal::ZERO { "up" }
                else if c < Decimal::ZERO { "down" }
                else { "stable" }
            }.to_string());

            // Determine status based on thresholds
            let is_lower_better = matches!(code, "avg_risk_score" | "high_risks" | "open_findings" | "overdue_tasks");
            let status = if is_lower_better {
                if value <= Decimal::from(target) { "excellent" }
                else if value <= Decimal::from(warn) { "normal" }
                else if value <= Decimal::from(crit) { "warning" }
                else { "critical" }
            } else {
                if value >= Decimal::from(target) { "excellent" }
                else if value >= Decimal::from(warn) { "normal" }
                else if value >= Decimal::from(crit) { "warning" }
                else { "critical" }
            }.to_string();

            // Build sparkline data
            let sparkline: Vec<serde_json::Value> = historical.iter().map(|s| {
                let v = match code {
                    "overall_compliance" => s.overall_compliance_score,
                    "framework_coverage" => s.framework_coverage_pct,
                    "control_implementation" => s.control_implementation_pct,
                    "control_pass_rate" => s.control_pass_rate,
                    "evidence_freshness" => s.evidence_freshness_score,
                    "avg_risk_score" => s.average_risk_score,
                    "high_risks" => Decimal::from(s.high_risks),
                    "open_findings" => Decimal::from(s.open_findings),
                    "overdue_tasks" => Decimal::from(s.overdue_tasks),
                    _ => Decimal::ZERO,
                };
                serde_json::json!({"date": s.snapshot_date.to_string(), "value": v})
            }).collect();

            // Upsert metric
            let metric: ExecutiveMetric = sqlx::query_as(
                r#"
                INSERT INTO executive_metrics (
                    organization_id, metric_name, metric_code, category,
                    current_value, previous_value, change_value, change_pct, trend,
                    target_value, threshold_warning, threshold_critical,
                    status, sparkline_data, display_format, display_order
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                ON CONFLICT (organization_id, metric_code)
                DO UPDATE SET
                    current_value = EXCLUDED.current_value,
                    previous_value = EXCLUDED.previous_value,
                    change_value = EXCLUDED.change_value,
                    change_pct = EXCLUDED.change_pct,
                    trend = EXCLUDED.trend,
                    status = EXCLUDED.status,
                    sparkline_data = EXCLUDED.sparkline_data,
                    computed_at = NOW()
                RETURNING *
                "#
            )
            .bind(org_id)
            .bind(name)
            .bind(code)
            .bind(category)
            .bind(value)
            .bind(previous_value)
            .bind(change_value)
            .bind(change_pct)
            .bind(&trend)
            .bind(Decimal::from(target))
            .bind(Decimal::from(warn))
            .bind(Decimal::from(crit))
            .bind(&status)
            .bind(serde_json::Value::Array(sparkline))
            .bind(format)
            .bind(order)
            .fetch_one(&self.db)
            .await?;

            metrics.push(metric);
            order += 1;
        }

        Ok(metrics)
    }

    /// Get dashboard widgets
    pub async fn get_dashboard_widgets(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        dashboard_type: &str,
    ) -> AppResult<Vec<DashboardWidget>> {
        let widgets: Vec<DashboardWidget> = sqlx::query_as(
            r#"
            SELECT * FROM dashboard_widgets
            WHERE organization_id = $1
              AND (user_id = $2 OR user_id IS NULL)
              AND dashboard_type = $3
              AND is_visible = TRUE
            ORDER BY grid_y, grid_x
            "#
        )
        .bind(org_id)
        .bind(user_id)
        .bind(dashboard_type)
        .fetch_all(&self.db)
        .await?;

        Ok(widgets)
    }

    /// Create dashboard widget
    pub async fn create_dashboard_widget(
        &self,
        org_id: Uuid,
        user_id: Option<Uuid>,
        input: CreateDashboardWidget,
    ) -> AppResult<DashboardWidget> {
        let widget: DashboardWidget = sqlx::query_as(
            r#"
            INSERT INTO dashboard_widgets (
                organization_id, user_id, dashboard_type, widget_type, widget_title,
                config, grid_x, grid_y, grid_width, grid_height
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#
        )
        .bind(org_id)
        .bind(user_id)
        .bind(input.dashboard_type.as_deref().unwrap_or("executive"))
        .bind(&input.widget_type)
        .bind(&input.widget_title)
        .bind(&input.config)
        .bind(input.grid_x.unwrap_or(0))
        .bind(input.grid_y.unwrap_or(0))
        .bind(input.grid_width.unwrap_or(1))
        .bind(input.grid_height.unwrap_or(1))
        .fetch_one(&self.db)
        .await?;

        Ok(widget)
    }

    /// Update dashboard widget
    pub async fn update_dashboard_widget(
        &self,
        org_id: Uuid,
        widget_id: Uuid,
        input: UpdateDashboardWidget,
    ) -> AppResult<DashboardWidget> {
        let widget: DashboardWidget = sqlx::query_as(
            r#"
            UPDATE dashboard_widgets SET
                widget_title = COALESCE($3, widget_title),
                config = COALESCE($4, config),
                grid_x = COALESCE($5, grid_x),
                grid_y = COALESCE($6, grid_y),
                grid_width = COALESCE($7, grid_width),
                grid_height = COALESCE($8, grid_height),
                is_visible = COALESCE($9, is_visible)
            WHERE id = $1 AND organization_id = $2
            RETURNING *
            "#
        )
        .bind(widget_id)
        .bind(org_id)
        .bind(&input.widget_title)
        .bind(&input.config)
        .bind(input.grid_x)
        .bind(input.grid_y)
        .bind(input.grid_width)
        .bind(input.grid_height)
        .bind(input.is_visible)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Widget not found".to_string()))?;

        Ok(widget)
    }

    /// Delete dashboard widget
    pub async fn delete_dashboard_widget(&self, org_id: Uuid, widget_id: Uuid) -> AppResult<()> {
        let result = sqlx::query(
            "DELETE FROM dashboard_widgets WHERE id = $1 AND organization_id = $2"
        )
        .bind(widget_id)
        .bind(org_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Widget not found".to_string()));
        }

        Ok(())
    }

    /// Get full executive dashboard data
    pub async fn get_executive_dashboard(&self, org_id: Uuid, user_id: Option<Uuid>) -> AppResult<ExecutiveDashboard> {
        // Get all data in parallel-ish manner
        let metrics = self.get_executive_metrics(org_id).await?;
        let widgets = self.get_dashboard_widgets(org_id, user_id, "executive").await?;
        let snapshot = self.get_current_snapshot(org_id).await?;
        let trends = self.get_compliance_trends(org_id, 30).await?;
        let risk_predictions = self.get_risk_prediction_summary(org_id).await?;
        let benchmark = self.get_latest_benchmark_comparison(org_id).await?;

        Ok(ExecutiveDashboard {
            metrics,
            widgets,
            snapshot,
            trend_data: trends.snapshots,
            risk_predictions,
            benchmark,
        })
    }
}
