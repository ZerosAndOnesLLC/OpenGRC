-- Advanced Analytics Tables for OpenGRC
-- Phase 4.2: Compliance Trend Analysis, Predictive Risk Scoring, Benchmark Comparisons,
-- Custom Report Builder, and Executive Dashboards

-- ============================================================================
-- 1. COMPLIANCE TREND ANALYSIS
-- ============================================================================

-- Daily/weekly snapshots of compliance metrics for trend analysis
CREATE TABLE compliance_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    snapshot_date DATE NOT NULL,
    snapshot_type VARCHAR(20) NOT NULL DEFAULT 'daily', -- daily, weekly, monthly

    -- Framework coverage metrics
    total_frameworks INTEGER NOT NULL DEFAULT 0,
    total_requirements INTEGER NOT NULL DEFAULT 0,
    covered_requirements INTEGER NOT NULL DEFAULT 0,
    framework_coverage_pct DECIMAL(5,2) NOT NULL DEFAULT 0,

    -- Control metrics
    total_controls INTEGER NOT NULL DEFAULT 0,
    implemented_controls INTEGER NOT NULL DEFAULT 0,
    partially_implemented_controls INTEGER NOT NULL DEFAULT 0,
    not_implemented_controls INTEGER NOT NULL DEFAULT 0,
    control_implementation_pct DECIMAL(5,2) NOT NULL DEFAULT 0,

    -- Control testing metrics
    controls_tested INTEGER NOT NULL DEFAULT 0,
    controls_passed INTEGER NOT NULL DEFAULT 0,
    controls_failed INTEGER NOT NULL DEFAULT 0,
    control_pass_rate DECIMAL(5,2) NOT NULL DEFAULT 0,

    -- Risk metrics
    total_risks INTEGER NOT NULL DEFAULT 0,
    high_risks INTEGER NOT NULL DEFAULT 0,
    medium_risks INTEGER NOT NULL DEFAULT 0,
    low_risks INTEGER NOT NULL DEFAULT 0,
    average_risk_score DECIMAL(5,2) NOT NULL DEFAULT 0,
    average_residual_score DECIMAL(5,2) NOT NULL DEFAULT 0,
    risks_with_controls INTEGER NOT NULL DEFAULT 0,

    -- Evidence metrics
    total_evidence INTEGER NOT NULL DEFAULT 0,
    valid_evidence INTEGER NOT NULL DEFAULT 0,
    expiring_evidence INTEGER NOT NULL DEFAULT 0,
    expired_evidence INTEGER NOT NULL DEFAULT 0,
    evidence_freshness_score DECIMAL(5,2) NOT NULL DEFAULT 0,

    -- Policy metrics
    total_policies INTEGER NOT NULL DEFAULT 0,
    published_policies INTEGER NOT NULL DEFAULT 0,
    policies_needing_review INTEGER NOT NULL DEFAULT 0,
    policy_acknowledgment_rate DECIMAL(5,2) NOT NULL DEFAULT 0,

    -- Vendor metrics
    total_vendors INTEGER NOT NULL DEFAULT 0,
    high_risk_vendors INTEGER NOT NULL DEFAULT 0,
    vendors_assessed_last_year INTEGER NOT NULL DEFAULT 0,

    -- Asset metrics
    total_assets INTEGER NOT NULL DEFAULT 0,
    assets_with_controls INTEGER NOT NULL DEFAULT 0,

    -- Task metrics
    total_open_tasks INTEGER NOT NULL DEFAULT 0,
    overdue_tasks INTEGER NOT NULL DEFAULT 0,

    -- Audit metrics
    active_audits INTEGER NOT NULL DEFAULT 0,
    open_findings INTEGER NOT NULL DEFAULT 0,

    -- Overall compliance score (0-100)
    overall_compliance_score DECIMAL(5,2) NOT NULL DEFAULT 0,

    -- Framework-specific breakdown (JSON for flexibility)
    framework_details JSONB DEFAULT '[]',

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(organization_id, snapshot_date, snapshot_type)
);

CREATE INDEX idx_compliance_snapshots_org_date ON compliance_snapshots(organization_id, snapshot_date DESC);
CREATE INDEX idx_compliance_snapshots_type ON compliance_snapshots(organization_id, snapshot_type, snapshot_date DESC);

-- ============================================================================
-- 2. PREDICTIVE RISK SCORING
-- ============================================================================

-- Store risk predictions with confidence levels
CREATE TABLE risk_predictions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    risk_id UUID NOT NULL REFERENCES risks(id) ON DELETE CASCADE,

    -- Current scores
    current_likelihood INTEGER NOT NULL,
    current_impact INTEGER NOT NULL,
    current_score INTEGER NOT NULL,

    -- Predicted scores (30-day projection)
    predicted_likelihood INTEGER NOT NULL,
    predicted_impact INTEGER NOT NULL,
    predicted_score INTEGER NOT NULL,

    -- 90-day prediction
    predicted_90d_score INTEGER,

    -- Confidence and trend
    confidence_level DECIMAL(5,2) NOT NULL, -- 0-100
    trend VARCHAR(20) NOT NULL, -- increasing, decreasing, stable
    trend_velocity DECIMAL(5,2), -- rate of change

    -- Factors contributing to prediction
    factor_scores JSONB NOT NULL DEFAULT '{}',

    -- Explanation for the prediction
    explanation TEXT,

    -- Status
    is_current BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days')
);

CREATE INDEX idx_risk_predictions_org ON risk_predictions(organization_id);
CREATE INDEX idx_risk_predictions_risk ON risk_predictions(risk_id);
CREATE INDEX idx_risk_predictions_current ON risk_predictions(organization_id, is_current) WHERE is_current = TRUE;
CREATE INDEX idx_risk_predictions_trend ON risk_predictions(organization_id, trend) WHERE is_current = TRUE;

-- Risk prediction factors (what influences the prediction)
-- System-level factors have NULL organization_id, org-specific overrides have the org_id
CREATE TABLE risk_prediction_factors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE, -- NULL for system-level

    -- Factor definition
    name VARCHAR(100) NOT NULL,
    code VARCHAR(50) NOT NULL,
    category VARCHAR(50) NOT NULL, -- control_effectiveness, historical_trend, similar_risks, external, environmental

    -- Weight for scoring
    weight DECIMAL(5,2) NOT NULL DEFAULT 1.0,

    -- Description
    description TEXT,

    -- Whether this is a system-defined or custom factor
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Unique constraint for org-specific factors
CREATE UNIQUE INDEX idx_risk_prediction_factors_org_code ON risk_prediction_factors(organization_id, code) WHERE organization_id IS NOT NULL;
-- Unique constraint for system-level factors
CREATE UNIQUE INDEX idx_risk_prediction_factors_system_code ON risk_prediction_factors(code) WHERE organization_id IS NULL;
CREATE INDEX idx_risk_prediction_factors_org ON risk_prediction_factors(organization_id) WHERE organization_id IS NOT NULL;
CREATE INDEX idx_risk_prediction_factors_system ON risk_prediction_factors(is_system) WHERE is_system = TRUE;

-- Seed default prediction factors (system-level with NULL org_id)
INSERT INTO risk_prediction_factors (organization_id, name, code, category, weight, description, is_system) VALUES
    (NULL, 'Control Effectiveness', 'control_effectiveness', 'control_effectiveness', 2.0, 'Effectiveness of controls mapped to this risk', TRUE),
    (NULL, 'Historical Trend', 'historical_trend', 'historical_trend', 1.5, 'Risk score changes over the past 90 days', TRUE),
    (NULL, 'Similar Risk Outcomes', 'similar_risk_outcomes', 'similar_risks', 1.0, 'Outcomes of risks with similar characteristics', TRUE),
    (NULL, 'Treatment Progress', 'treatment_progress', 'control_effectiveness', 1.5, 'Progress on treatment plan implementation', TRUE),
    (NULL, 'Time Since Last Review', 'time_since_review', 'environmental', 0.5, 'Time elapsed since last risk assessment', TRUE),
    (NULL, 'Control Test Results', 'control_test_results', 'control_effectiveness', 2.0, 'Recent test results for mapped controls', TRUE),
    (NULL, 'Evidence Freshness', 'evidence_freshness', 'environmental', 0.75, 'Freshness of evidence supporting controls', TRUE),
    (NULL, 'Industry Incidents', 'industry_incidents', 'external', 0.5, 'Recent industry security incidents for similar risks', TRUE);

-- ============================================================================
-- 3. BENCHMARK COMPARISONS
-- ============================================================================

-- Industry benchmark data (aggregated, anonymized)
CREATE TABLE industry_benchmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Benchmark identification
    benchmark_name VARCHAR(200) NOT NULL,
    benchmark_code VARCHAR(50) NOT NULL UNIQUE,
    industry VARCHAR(100), -- NULL means all industries
    company_size VARCHAR(50), -- small, medium, large, enterprise, NULL for all
    framework_id UUID REFERENCES frameworks(id) ON DELETE SET NULL,

    -- Benchmark metrics
    avg_framework_coverage DECIMAL(5,2),
    avg_control_implementation DECIMAL(5,2),
    avg_control_pass_rate DECIMAL(5,2),
    avg_risk_score DECIMAL(5,2),
    avg_evidence_freshness DECIMAL(5,2),
    avg_policy_acknowledgment DECIMAL(5,2),
    avg_vendor_assessment_rate DECIMAL(5,2),
    avg_compliance_score DECIMAL(5,2),

    -- Percentile distribution
    p25_compliance_score DECIMAL(5,2),
    p50_compliance_score DECIMAL(5,2),
    p75_compliance_score DECIMAL(5,2),
    p90_compliance_score DECIMAL(5,2),

    -- Sample size and validity
    sample_size INTEGER NOT NULL DEFAULT 0,
    data_as_of DATE NOT NULL,
    valid_until DATE NOT NULL,

    -- Additional metrics in JSON
    detailed_metrics JSONB DEFAULT '{}',

    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_industry_benchmarks_active ON industry_benchmarks(is_active) WHERE is_active = TRUE;
CREATE INDEX idx_industry_benchmarks_industry ON industry_benchmarks(industry, company_size);
CREATE INDEX idx_industry_benchmarks_framework ON industry_benchmarks(framework_id);

-- Organization benchmark comparison results
CREATE TABLE organization_benchmark_comparisons (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    benchmark_id UUID NOT NULL REFERENCES industry_benchmarks(id) ON DELETE CASCADE,

    -- Organization's metrics at comparison time
    org_framework_coverage DECIMAL(5,2),
    org_control_implementation DECIMAL(5,2),
    org_control_pass_rate DECIMAL(5,2),
    org_risk_score DECIMAL(5,2),
    org_evidence_freshness DECIMAL(5,2),
    org_policy_acknowledgment DECIMAL(5,2),
    org_vendor_assessment_rate DECIMAL(5,2),
    org_compliance_score DECIMAL(5,2),

    -- Comparison results (org value - benchmark value)
    diff_framework_coverage DECIMAL(5,2),
    diff_control_implementation DECIMAL(5,2),
    diff_control_pass_rate DECIMAL(5,2),
    diff_risk_score DECIMAL(5,2),
    diff_evidence_freshness DECIMAL(5,2),
    diff_policy_acknowledgment DECIMAL(5,2),
    diff_vendor_assessment_rate DECIMAL(5,2),
    diff_compliance_score DECIMAL(5,2),

    -- Percentile rank (where does org fall)
    percentile_rank DECIMAL(5,2),

    -- Status assessment
    overall_status VARCHAR(20) NOT NULL, -- above_average, average, below_average, needs_attention

    -- Detailed comparison data
    detailed_comparison JSONB DEFAULT '{}',

    -- Recommendations based on comparison
    recommendations JSONB DEFAULT '[]',

    comparison_date DATE NOT NULL DEFAULT CURRENT_DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_org_benchmarks_org ON organization_benchmark_comparisons(organization_id, comparison_date DESC);
CREATE INDEX idx_org_benchmarks_benchmark ON organization_benchmark_comparisons(benchmark_id);

-- Seed default industry benchmarks
INSERT INTO industry_benchmarks (benchmark_name, benchmark_code, industry, company_size,
    avg_framework_coverage, avg_control_implementation, avg_control_pass_rate, avg_risk_score,
    avg_evidence_freshness, avg_policy_acknowledgment, avg_vendor_assessment_rate, avg_compliance_score,
    p25_compliance_score, p50_compliance_score, p75_compliance_score, p90_compliance_score,
    sample_size, data_as_of, valid_until, detailed_metrics) VALUES
    ('Technology Industry - SMB', 'tech_smb', 'Technology', 'small',
     65.0, 72.0, 78.0, 12.5, 70.0, 85.0, 60.0, 68.0,
     55.0, 68.0, 78.0, 88.0, 250, '2024-12-01', '2025-06-01',
     '{"soc2_coverage": 70, "avg_time_to_remediation_days": 14, "controls_per_100_employees": 25}'),
    ('Technology Industry - Enterprise', 'tech_enterprise', 'Technology', 'enterprise',
     82.0, 88.0, 91.0, 8.5, 85.0, 94.0, 80.0, 85.0,
     72.0, 85.0, 92.0, 96.0, 180, '2024-12-01', '2025-06-01',
     '{"soc2_coverage": 92, "avg_time_to_remediation_days": 7, "controls_per_100_employees": 45}'),
    ('Healthcare Industry - All', 'healthcare_all', 'Healthcare', NULL,
     78.0, 82.0, 85.0, 10.0, 80.0, 92.0, 75.0, 80.0,
     68.0, 80.0, 88.0, 94.0, 320, '2024-12-01', '2025-06-01',
     '{"hipaa_coverage": 88, "avg_time_to_remediation_days": 10, "controls_per_100_employees": 35}'),
    ('Financial Services - All', 'finserv_all', 'Financial Services', NULL,
     85.0, 90.0, 92.0, 7.5, 88.0, 96.0, 85.0, 88.0,
     78.0, 88.0, 94.0, 98.0, 420, '2024-12-01', '2025-06-01',
     '{"pci_coverage": 94, "soc2_coverage": 92, "avg_time_to_remediation_days": 5}'),
    ('All Industries - Baseline', 'all_baseline', NULL, NULL,
     70.0, 75.0, 80.0, 11.0, 72.0, 88.0, 65.0, 72.0,
     60.0, 72.0, 82.0, 90.0, 1500, '2024-12-01', '2025-06-01',
     '{"avg_time_to_remediation_days": 12, "controls_per_100_employees": 30}');

-- ============================================================================
-- 4. CUSTOM REPORT BUILDER
-- ============================================================================

-- Saved report configurations
CREATE TABLE saved_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Report metadata
    name VARCHAR(200) NOT NULL,
    description TEXT,
    report_type VARCHAR(50) NOT NULL, -- compliance, risk, controls, evidence, executive, custom

    -- Report configuration
    config JSONB NOT NULL DEFAULT '{}',
    -- Config structure:
    -- {
    --   "data_sources": ["controls", "risks", "evidence"],
    --   "columns": [{"field": "name", "label": "Control Name"}, ...],
    --   "filters": [{"field": "status", "operator": "equals", "value": "implemented"}],
    --   "grouping": {"field": "category", "aggregations": ["count", "avg_score"]},
    --   "sorting": [{"field": "created_at", "direction": "desc"}],
    --   "date_range": {"type": "relative", "value": "last_30_days"},
    --   "visualizations": [{"type": "bar_chart", "x": "category", "y": "count"}],
    --   "include_summary": true,
    --   "schedule": {"enabled": false, "cron": "0 9 * * 1", "recipients": []}
    -- }

    -- Display settings
    layout VARCHAR(20) NOT NULL DEFAULT 'table', -- table, chart, combined
    chart_config JSONB DEFAULT '{}',

    -- Sharing settings
    is_public BOOLEAN NOT NULL DEFAULT FALSE, -- Visible to all org members
    shared_with JSONB DEFAULT '[]', -- List of user IDs

    -- Scheduling
    schedule_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    schedule_cron VARCHAR(100),
    schedule_recipients JSONB DEFAULT '[]', -- Email addresses
    last_scheduled_run TIMESTAMPTZ,
    next_scheduled_run TIMESTAMPTZ,

    -- Usage tracking
    last_run_at TIMESTAMPTZ,
    run_count INTEGER NOT NULL DEFAULT 0,

    -- Ownership
    created_by UUID NOT NULL,
    updated_by UUID,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_saved_reports_org ON saved_reports(organization_id);
CREATE INDEX idx_saved_reports_creator ON saved_reports(organization_id, created_by);
CREATE INDEX idx_saved_reports_type ON saved_reports(organization_id, report_type);
CREATE INDEX idx_saved_reports_public ON saved_reports(organization_id, is_public) WHERE is_public = TRUE;
CREATE INDEX idx_saved_reports_scheduled ON saved_reports(schedule_enabled, next_scheduled_run) WHERE schedule_enabled = TRUE;

-- Report execution history
CREATE TABLE report_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    report_id UUID REFERENCES saved_reports(id) ON DELETE SET NULL,

    -- Report details (stored for historical reference even if report is deleted)
    report_name VARCHAR(200) NOT NULL,
    report_type VARCHAR(50) NOT NULL,
    report_config JSONB NOT NULL,

    -- Execution details
    executed_by UUID,
    execution_type VARCHAR(20) NOT NULL, -- manual, scheduled

    -- Filters applied at execution time
    applied_filters JSONB DEFAULT '{}',
    date_range_start DATE,
    date_range_end DATE,

    -- Results summary
    rows_returned INTEGER NOT NULL DEFAULT 0,
    execution_time_ms INTEGER NOT NULL DEFAULT 0,

    -- Output
    output_format VARCHAR(20) NOT NULL, -- view, csv, pdf, json
    output_file_path VARCHAR(500), -- For stored exports

    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'completed', -- running, completed, failed
    error_message TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_report_executions_org ON report_executions(organization_id, created_at DESC);
CREATE INDEX idx_report_executions_report ON report_executions(report_id, created_at DESC);
CREATE INDEX idx_report_executions_user ON report_executions(executed_by, created_at DESC);

-- Report templates (system-provided starting points)
CREATE TABLE report_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Template metadata
    name VARCHAR(200) NOT NULL,
    description TEXT,
    report_type VARCHAR(50) NOT NULL,
    category VARCHAR(50) NOT NULL, -- compliance, risk, audit, operations

    -- Template configuration
    config JSONB NOT NULL,
    layout VARCHAR(20) NOT NULL DEFAULT 'table',
    chart_config JSONB DEFAULT '{}',

    -- Preview image path
    preview_image VARCHAR(500),

    -- Sorting
    sort_order INTEGER NOT NULL DEFAULT 0,

    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Seed report templates
INSERT INTO report_templates (name, description, report_type, category, config, layout, sort_order) VALUES
    ('Compliance Posture Summary', 'High-level overview of compliance status across all frameworks', 'compliance', 'compliance',
     '{"data_sources": ["frameworks", "controls"], "columns": [{"field": "framework_name", "label": "Framework"}, {"field": "coverage_pct", "label": "Coverage %"}, {"field": "controls_implemented", "label": "Controls Implemented"}, {"field": "gaps", "label": "Gaps"}], "grouping": {"field": "framework_name"}, "include_summary": true, "visualizations": [{"type": "progress_bars", "field": "coverage_pct"}]}',
     'combined', 1),
    ('Risk Register Report', 'Complete risk register with scores and controls', 'risk', 'risk',
     '{"data_sources": ["risks", "controls"], "columns": [{"field": "code", "label": "ID"}, {"field": "title", "label": "Risk"}, {"field": "category", "label": "Category"}, {"field": "inherent_score", "label": "Inherent"}, {"field": "residual_score", "label": "Residual"}, {"field": "status", "label": "Status"}, {"field": "control_count", "label": "Controls"}], "sorting": [{"field": "inherent_score", "direction": "desc"}], "include_summary": true}',
     'table', 2),
    ('Control Effectiveness', 'Control testing results and effectiveness metrics', 'controls', 'operations',
     '{"data_sources": ["controls", "control_tests"], "columns": [{"field": "code", "label": "Control"}, {"field": "name", "label": "Name"}, {"field": "frequency", "label": "Frequency"}, {"field": "last_test_date", "label": "Last Tested"}, {"field": "last_test_result", "label": "Result"}, {"field": "pass_rate", "label": "Pass Rate"}], "filters": [{"field": "status", "operator": "equals", "value": "implemented"}], "grouping": {"field": "control_type"}}',
     'combined', 3),
    ('Evidence Inventory', 'All evidence with freshness and control mapping', 'evidence', 'audit',
     '{"data_sources": ["evidence"], "columns": [{"field": "title", "label": "Evidence"}, {"field": "evidence_type", "label": "Type"}, {"field": "source", "label": "Source"}, {"field": "collected_at", "label": "Collected"}, {"field": "valid_until", "label": "Valid Until"}, {"field": "freshness_score", "label": "Freshness"}, {"field": "linked_controls", "label": "Controls"}], "sorting": [{"field": "valid_until", "direction": "asc"}]}',
     'table', 4),
    ('Vendor Risk Assessment', 'Vendor inventory with risk ratings and assessment status', 'vendors', 'risk',
     '{"data_sources": ["vendors", "vendor_assessments"], "columns": [{"field": "name", "label": "Vendor"}, {"field": "category", "label": "Category"}, {"field": "criticality", "label": "Criticality"}, {"field": "risk_rating", "label": "Risk Rating"}, {"field": "last_assessment", "label": "Last Assessment"}, {"field": "contract_end", "label": "Contract End"}], "sorting": [{"field": "criticality", "direction": "desc"}], "grouping": {"field": "criticality"}}',
     'table', 5),
    ('Audit Readiness', 'Audit preparation status with evidence gaps', 'audit', 'audit',
     '{"data_sources": ["audits", "controls", "evidence"], "columns": [{"field": "control_code", "label": "Control"}, {"field": "control_name", "label": "Control Name"}, {"field": "requirements_met", "label": "Requirements"}, {"field": "evidence_count", "label": "Evidence"}, {"field": "evidence_fresh", "label": "Fresh Evidence"}, {"field": "last_test", "label": "Last Test"}, {"field": "status", "label": "Status"}]}',
     'table', 6),
    ('Policy Acknowledgment Status', 'Policy compliance and acknowledgment tracking', 'policies', 'compliance',
     '{"data_sources": ["policies", "policy_acknowledgments"], "columns": [{"field": "code", "label": "Policy"}, {"field": "title", "label": "Title"}, {"field": "status", "label": "Status"}, {"field": "version", "label": "Version"}, {"field": "acknowledgment_rate", "label": "Ack Rate"}, {"field": "pending_acks", "label": "Pending"}, {"field": "review_date", "label": "Next Review"}], "grouping": {"field": "category"}}',
     'combined', 7),
    ('Executive Summary', 'High-level KPIs and trends for leadership', 'executive', 'compliance',
     '{"data_sources": ["compliance_snapshots"], "columns": [{"field": "overall_compliance_score", "label": "Compliance Score"}, {"field": "framework_coverage_pct", "label": "Framework Coverage"}, {"field": "control_pass_rate", "label": "Control Pass Rate"}, {"field": "high_risks", "label": "High Risks"}, {"field": "open_findings", "label": "Open Findings"}, {"field": "overdue_tasks", "label": "Overdue Tasks"}], "date_range": {"type": "relative", "value": "last_30_days"}, "visualizations": [{"type": "trend_line", "field": "overall_compliance_score"}, {"type": "gauge", "field": "compliance_score"}], "include_summary": true}',
     'combined', 8);

-- ============================================================================
-- 5. EXECUTIVE DASHBOARD
-- ============================================================================

-- Executive metrics (computed periodically)
CREATE TABLE executive_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Metric identification
    metric_name VARCHAR(100) NOT NULL,
    metric_code VARCHAR(50) NOT NULL,
    category VARCHAR(50) NOT NULL, -- compliance, risk, operations, security

    -- Current value
    current_value DECIMAL(10,2) NOT NULL,
    previous_value DECIMAL(10,2),

    -- Change tracking
    change_value DECIMAL(10,2),
    change_pct DECIMAL(5,2),
    trend VARCHAR(20), -- up, down, stable

    -- Target/threshold
    target_value DECIMAL(10,2),
    threshold_warning DECIMAL(10,2),
    threshold_critical DECIMAL(10,2),

    -- Status based on thresholds
    status VARCHAR(20) NOT NULL DEFAULT 'normal', -- normal, warning, critical, excellent

    -- Sparkline data (last 30 data points)
    sparkline_data JSONB DEFAULT '[]',

    -- Display settings
    display_format VARCHAR(20) NOT NULL DEFAULT 'percentage', -- percentage, number, score, currency
    display_order INTEGER NOT NULL DEFAULT 0,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,

    -- Refresh tracking
    computed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(organization_id, metric_code)
);

CREATE INDEX idx_executive_metrics_org ON executive_metrics(organization_id);
CREATE INDEX idx_executive_metrics_category ON executive_metrics(organization_id, category);
CREATE INDEX idx_executive_metrics_visible ON executive_metrics(organization_id, is_visible, display_order) WHERE is_visible = TRUE;

-- Dashboard widget configurations
CREATE TABLE dashboard_widgets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID, -- NULL means org-wide default, otherwise user-specific
    dashboard_type VARCHAR(50) NOT NULL DEFAULT 'executive', -- executive, compliance, operations

    -- Widget definition
    widget_type VARCHAR(50) NOT NULL, -- kpi_card, chart, table, heatmap, trend_line, gauge, progress
    widget_title VARCHAR(200) NOT NULL,

    -- Widget configuration
    config JSONB NOT NULL DEFAULT '{}',
    -- Config examples:
    -- KPI Card: {"metric_code": "overall_compliance", "show_trend": true, "show_sparkline": true}
    -- Chart: {"chart_type": "bar", "data_source": "risks_by_category", "colors": {...}}
    -- Table: {"data_source": "top_risks", "columns": [...], "limit": 5}

    -- Layout (grid position)
    grid_x INTEGER NOT NULL DEFAULT 0,
    grid_y INTEGER NOT NULL DEFAULT 0,
    grid_width INTEGER NOT NULL DEFAULT 1, -- 1-4 columns
    grid_height INTEGER NOT NULL DEFAULT 1, -- 1-4 rows

    -- Visibility
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_dashboard_widgets_org ON dashboard_widgets(organization_id, dashboard_type);
CREATE INDEX idx_dashboard_widgets_user ON dashboard_widgets(organization_id, user_id, dashboard_type);

-- ============================================================================
-- 6. TRIGGERS AND FUNCTIONS
-- ============================================================================

-- Function to compute overall compliance score
CREATE OR REPLACE FUNCTION compute_compliance_score(
    framework_coverage DECIMAL,
    control_implementation DECIMAL,
    control_pass_rate DECIMAL,
    risk_score DECIMAL,
    evidence_freshness DECIMAL,
    policy_acknowledgment DECIMAL
) RETURNS DECIMAL AS $$
DECLARE
    -- Weights for each component
    w_framework DECIMAL := 0.20;
    w_controls DECIMAL := 0.25;
    w_testing DECIMAL := 0.20;
    w_risk DECIMAL := 0.15;
    w_evidence DECIMAL := 0.10;
    w_policy DECIMAL := 0.10;

    -- Normalize risk score (lower is better, so invert)
    normalized_risk DECIMAL;
    score DECIMAL;
BEGIN
    -- Risk score of 1-25, invert so 25=0%, 1=100%
    normalized_risk := GREATEST(0, (25 - COALESCE(risk_score, 12.5)) / 25 * 100);

    score := (
        COALESCE(framework_coverage, 0) * w_framework +
        COALESCE(control_implementation, 0) * w_controls +
        COALESCE(control_pass_rate, 0) * w_testing +
        normalized_risk * w_risk +
        COALESCE(evidence_freshness, 0) * w_evidence +
        COALESCE(policy_acknowledgment, 0) * w_policy
    );

    RETURN ROUND(score, 2);
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Function to capture compliance snapshot
CREATE OR REPLACE FUNCTION capture_compliance_snapshot(
    p_org_id UUID,
    p_snapshot_type VARCHAR DEFAULT 'daily'
) RETURNS UUID AS $$
DECLARE
    snapshot_id UUID;
    v_total_frameworks INTEGER;
    v_total_requirements INTEGER;
    v_covered_requirements INTEGER;
    v_total_controls INTEGER;
    v_implemented_controls INTEGER;
    v_partially_implemented INTEGER;
    v_not_implemented INTEGER;
    v_controls_tested INTEGER;
    v_controls_passed INTEGER;
    v_controls_failed INTEGER;
    v_total_risks INTEGER;
    v_high_risks INTEGER;
    v_medium_risks INTEGER;
    v_low_risks INTEGER;
    v_avg_risk_score DECIMAL;
    v_avg_residual_score DECIMAL;
    v_risks_with_controls INTEGER;
    v_total_evidence INTEGER;
    v_valid_evidence INTEGER;
    v_expiring_evidence INTEGER;
    v_expired_evidence INTEGER;
    v_total_policies INTEGER;
    v_published_policies INTEGER;
    v_policies_review INTEGER;
    v_policy_ack_rate DECIMAL;
    v_total_vendors INTEGER;
    v_high_risk_vendors INTEGER;
    v_vendors_assessed INTEGER;
    v_total_assets INTEGER;
    v_assets_with_controls INTEGER;
    v_open_tasks INTEGER;
    v_overdue_tasks INTEGER;
    v_active_audits INTEGER;
    v_open_findings INTEGER;
    v_framework_details JSONB;
    v_overall_score DECIMAL;
BEGIN
    -- Frameworks
    SELECT COUNT(*) INTO v_total_frameworks FROM frameworks WHERE is_system = FALSE OR id IN (SELECT DISTINCT framework_id FROM control_requirement_mappings crm JOIN controls c ON crm.control_id = c.id WHERE c.organization_id = p_org_id);

    SELECT COUNT(DISTINCT fr.id), COUNT(DISTINCT CASE WHEN crm.id IS NOT NULL THEN fr.id END)
    INTO v_total_requirements, v_covered_requirements
    FROM framework_requirements fr
    LEFT JOIN control_requirement_mappings crm ON fr.id = crm.framework_requirement_id
    LEFT JOIN controls c ON crm.control_id = c.id AND c.organization_id = p_org_id;

    -- Controls
    SELECT
        COUNT(*),
        COUNT(*) FILTER (WHERE status = 'implemented'),
        COUNT(*) FILTER (WHERE status = 'partially_implemented'),
        COUNT(*) FILTER (WHERE status = 'not_implemented')
    INTO v_total_controls, v_implemented_controls, v_partially_implemented, v_not_implemented
    FROM controls WHERE organization_id = p_org_id;

    -- Control tests (last 90 days)
    SELECT
        COUNT(DISTINCT ct.control_id),
        COUNT(DISTINCT ct.control_id) FILTER (WHERE ctr.status = 'passed'),
        COUNT(DISTINCT ct.control_id) FILTER (WHERE ctr.status = 'failed')
    INTO v_controls_tested, v_controls_passed, v_controls_failed
    FROM control_tests ct
    JOIN controls c ON ct.control_id = c.id
    LEFT JOIN control_test_results ctr ON ct.id = ctr.control_test_id AND ctr.performed_at > NOW() - INTERVAL '90 days'
    WHERE c.organization_id = p_org_id;

    -- Risks
    SELECT
        COUNT(*),
        COUNT(*) FILTER (WHERE inherent_score >= 15),
        COUNT(*) FILTER (WHERE inherent_score >= 5 AND inherent_score < 15),
        COUNT(*) FILTER (WHERE inherent_score < 5),
        COALESCE(AVG(inherent_score), 0),
        COALESCE(AVG(residual_score), 0),
        COUNT(DISTINCT r.id) FILTER (WHERE rcm.id IS NOT NULL)
    INTO v_total_risks, v_high_risks, v_medium_risks, v_low_risks, v_avg_risk_score, v_avg_residual_score, v_risks_with_controls
    FROM risks r
    LEFT JOIN risk_control_mappings rcm ON r.id = rcm.risk_id
    WHERE r.organization_id = p_org_id;

    -- Evidence
    SELECT
        COUNT(*),
        COUNT(*) FILTER (WHERE valid_until IS NULL OR valid_until > NOW()),
        COUNT(*) FILTER (WHERE valid_until > NOW() AND valid_until <= NOW() + INTERVAL '30 days'),
        COUNT(*) FILTER (WHERE valid_until <= NOW())
    INTO v_total_evidence, v_valid_evidence, v_expiring_evidence, v_expired_evidence
    FROM evidence WHERE organization_id = p_org_id;

    -- Policies
    SELECT
        COUNT(*),
        COUNT(*) FILTER (WHERE status = 'published'),
        COUNT(*) FILTER (WHERE review_date <= CURRENT_DATE)
    INTO v_total_policies, v_published_policies, v_policies_review
    FROM policies WHERE organization_id = p_org_id;

    -- Policy acknowledgment rate (simplified)
    v_policy_ack_rate := 85.0; -- TODO: Calculate from actual acknowledgments

    -- Vendors
    SELECT
        COUNT(*),
        COUNT(*) FILTER (WHERE criticality = 'critical'),
        COUNT(DISTINCT v.id) FILTER (WHERE va.assessed_at > NOW() - INTERVAL '1 year')
    INTO v_total_vendors, v_high_risk_vendors, v_vendors_assessed
    FROM vendors v
    LEFT JOIN vendor_assessments va ON v.id = va.vendor_id
    WHERE v.organization_id = p_org_id;

    -- Assets
    SELECT
        COUNT(*),
        COUNT(DISTINCT a.id) FILTER (WHERE acm.id IS NOT NULL)
    INTO v_total_assets, v_assets_with_controls
    FROM assets a
    LEFT JOIN asset_control_mappings acm ON a.id = acm.asset_id
    WHERE a.organization_id = p_org_id;

    -- Tasks
    SELECT
        COUNT(*) FILTER (WHERE status IN ('open', 'in_progress')),
        COUNT(*) FILTER (WHERE status IN ('open', 'in_progress') AND due_at < NOW())
    INTO v_open_tasks, v_overdue_tasks
    FROM tasks WHERE organization_id = p_org_id;

    -- Audits
    SELECT
        COUNT(*) FILTER (WHERE status IN ('planning', 'fieldwork', 'review')),
        COUNT(af.id) FILTER (WHERE af.status = 'open')
    INTO v_active_audits, v_open_findings
    FROM audits a
    LEFT JOIN audit_findings af ON a.id = af.audit_id
    WHERE a.organization_id = p_org_id;

    -- Framework details
    SELECT COALESCE(jsonb_agg(jsonb_build_object(
        'framework_id', f.id,
        'framework_name', f.name,
        'total_requirements', COUNT(fr.id),
        'covered_requirements', COUNT(DISTINCT CASE WHEN crm.id IS NOT NULL THEN fr.id END),
        'coverage_pct', ROUND(COUNT(DISTINCT CASE WHEN crm.id IS NOT NULL THEN fr.id END)::DECIMAL / NULLIF(COUNT(fr.id), 0) * 100, 2)
    )), '[]'::jsonb)
    INTO v_framework_details
    FROM frameworks f
    JOIN framework_requirements fr ON f.id = fr.framework_id
    LEFT JOIN control_requirement_mappings crm ON fr.id = crm.framework_requirement_id
    LEFT JOIN controls c ON crm.control_id = c.id AND c.organization_id = p_org_id
    WHERE f.is_system = TRUE OR f.id IN (SELECT DISTINCT framework_id FROM control_requirement_mappings crm2 JOIN controls c2 ON crm2.control_id = c2.id WHERE c2.organization_id = p_org_id)
    GROUP BY f.id, f.name;

    -- Calculate overall score
    v_overall_score := compute_compliance_score(
        CASE WHEN v_total_requirements > 0 THEN v_covered_requirements::DECIMAL / v_total_requirements * 100 ELSE 0 END,
        CASE WHEN v_total_controls > 0 THEN v_implemented_controls::DECIMAL / v_total_controls * 100 ELSE 0 END,
        CASE WHEN v_controls_tested > 0 THEN v_controls_passed::DECIMAL / v_controls_tested * 100 ELSE 0 END,
        v_avg_risk_score,
        CASE WHEN v_total_evidence > 0 THEN v_valid_evidence::DECIMAL / v_total_evidence * 100 ELSE 0 END,
        v_policy_ack_rate
    );

    -- Insert snapshot
    INSERT INTO compliance_snapshots (
        organization_id, snapshot_date, snapshot_type,
        total_frameworks, total_requirements, covered_requirements, framework_coverage_pct,
        total_controls, implemented_controls, partially_implemented_controls, not_implemented_controls, control_implementation_pct,
        controls_tested, controls_passed, controls_failed, control_pass_rate,
        total_risks, high_risks, medium_risks, low_risks, average_risk_score, average_residual_score, risks_with_controls,
        total_evidence, valid_evidence, expiring_evidence, expired_evidence, evidence_freshness_score,
        total_policies, published_policies, policies_needing_review, policy_acknowledgment_rate,
        total_vendors, high_risk_vendors, vendors_assessed_last_year,
        total_assets, assets_with_controls,
        total_open_tasks, overdue_tasks,
        active_audits, open_findings,
        overall_compliance_score, framework_details
    ) VALUES (
        p_org_id, CURRENT_DATE, p_snapshot_type,
        v_total_frameworks, v_total_requirements, v_covered_requirements,
        CASE WHEN v_total_requirements > 0 THEN ROUND(v_covered_requirements::DECIMAL / v_total_requirements * 100, 2) ELSE 0 END,
        v_total_controls, v_implemented_controls, v_partially_implemented, v_not_implemented,
        CASE WHEN v_total_controls > 0 THEN ROUND(v_implemented_controls::DECIMAL / v_total_controls * 100, 2) ELSE 0 END,
        v_controls_tested, v_controls_passed, v_controls_failed,
        CASE WHEN v_controls_tested > 0 THEN ROUND(v_controls_passed::DECIMAL / v_controls_tested * 100, 2) ELSE 0 END,
        v_total_risks, v_high_risks, v_medium_risks, v_low_risks, v_avg_risk_score, v_avg_residual_score, v_risks_with_controls,
        v_total_evidence, v_valid_evidence, v_expiring_evidence, v_expired_evidence,
        CASE WHEN v_total_evidence > 0 THEN ROUND(v_valid_evidence::DECIMAL / v_total_evidence * 100, 2) ELSE 0 END,
        v_total_policies, v_published_policies, v_policies_review, v_policy_ack_rate,
        v_total_vendors, v_high_risk_vendors, v_vendors_assessed,
        v_total_assets, v_assets_with_controls,
        v_open_tasks, v_overdue_tasks,
        v_active_audits, v_open_findings,
        v_overall_score, v_framework_details
    )
    ON CONFLICT (organization_id, snapshot_date, snapshot_type)
    DO UPDATE SET
        total_frameworks = EXCLUDED.total_frameworks,
        total_requirements = EXCLUDED.total_requirements,
        covered_requirements = EXCLUDED.covered_requirements,
        framework_coverage_pct = EXCLUDED.framework_coverage_pct,
        total_controls = EXCLUDED.total_controls,
        implemented_controls = EXCLUDED.implemented_controls,
        partially_implemented_controls = EXCLUDED.partially_implemented_controls,
        not_implemented_controls = EXCLUDED.not_implemented_controls,
        control_implementation_pct = EXCLUDED.control_implementation_pct,
        controls_tested = EXCLUDED.controls_tested,
        controls_passed = EXCLUDED.controls_passed,
        controls_failed = EXCLUDED.controls_failed,
        control_pass_rate = EXCLUDED.control_pass_rate,
        total_risks = EXCLUDED.total_risks,
        high_risks = EXCLUDED.high_risks,
        medium_risks = EXCLUDED.medium_risks,
        low_risks = EXCLUDED.low_risks,
        average_risk_score = EXCLUDED.average_risk_score,
        average_residual_score = EXCLUDED.average_residual_score,
        risks_with_controls = EXCLUDED.risks_with_controls,
        total_evidence = EXCLUDED.total_evidence,
        valid_evidence = EXCLUDED.valid_evidence,
        expiring_evidence = EXCLUDED.expiring_evidence,
        expired_evidence = EXCLUDED.expired_evidence,
        evidence_freshness_score = EXCLUDED.evidence_freshness_score,
        total_policies = EXCLUDED.total_policies,
        published_policies = EXCLUDED.published_policies,
        policies_needing_review = EXCLUDED.policies_needing_review,
        policy_acknowledgment_rate = EXCLUDED.policy_acknowledgment_rate,
        total_vendors = EXCLUDED.total_vendors,
        high_risk_vendors = EXCLUDED.high_risk_vendors,
        vendors_assessed_last_year = EXCLUDED.vendors_assessed_last_year,
        total_assets = EXCLUDED.total_assets,
        assets_with_controls = EXCLUDED.assets_with_controls,
        total_open_tasks = EXCLUDED.total_open_tasks,
        overdue_tasks = EXCLUDED.overdue_tasks,
        active_audits = EXCLUDED.active_audits,
        open_findings = EXCLUDED.open_findings,
        overall_compliance_score = EXCLUDED.overall_compliance_score,
        framework_details = EXCLUDED.framework_details,
        created_at = NOW()
    RETURNING id INTO snapshot_id;

    RETURN snapshot_id;
END;
$$ LANGUAGE plpgsql;

-- Function to compute risk prediction
CREATE OR REPLACE FUNCTION compute_risk_prediction(p_risk_id UUID) RETURNS UUID AS $$
DECLARE
    v_org_id UUID;
    v_current_likelihood INTEGER;
    v_current_impact INTEGER;
    v_current_score INTEGER;
    v_predicted_score INTEGER;
    v_predicted_likelihood INTEGER;
    v_predicted_impact INTEGER;
    v_confidence DECIMAL;
    v_trend VARCHAR(20);
    v_trend_velocity DECIMAL;
    v_factor_scores JSONB;
    v_explanation TEXT;
    v_control_effectiveness DECIMAL;
    v_historical_change DECIMAL;
    v_treatment_progress DECIMAL;
    v_test_pass_rate DECIMAL;
    v_evidence_freshness DECIMAL;
    prediction_id UUID;
BEGIN
    -- Get risk details
    SELECT organization_id, likelihood, impact, inherent_score
    INTO v_org_id, v_current_likelihood, v_current_impact, v_current_score
    FROM risks WHERE id = p_risk_id;

    IF v_org_id IS NULL THEN
        RETURN NULL;
    END IF;

    -- Calculate control effectiveness (0-100)
    SELECT COALESCE(AVG(
        CASE rcm.effectiveness
            WHEN 'full' THEN 100
            WHEN 'partial' THEN 60
            WHEN 'minimal' THEN 20
            ELSE 0
        END
    ), 0)
    INTO v_control_effectiveness
    FROM risk_control_mappings rcm
    WHERE rcm.risk_id = p_risk_id;

    -- Calculate historical trend (change over last 90 days from risk_assessments)
    SELECT COALESCE(
        (SELECT score FROM risk_assessments WHERE risk_id = p_risk_id ORDER BY created_at DESC LIMIT 1) -
        (SELECT score FROM risk_assessments WHERE risk_id = p_risk_id AND created_at < NOW() - INTERVAL '90 days' ORDER BY created_at DESC LIMIT 1),
        0
    ) INTO v_historical_change;

    -- Calculate treatment progress (simple heuristic based on status)
    SELECT CASE status
        WHEN 'closed' THEN 100
        WHEN 'accepted' THEN 80
        WHEN 'treating' THEN 50
        WHEN 'assessed' THEN 25
        ELSE 0
    END INTO v_treatment_progress
    FROM risks WHERE id = p_risk_id;

    -- Control test pass rate
    SELECT COALESCE(AVG(CASE WHEN ctr.status = 'passed' THEN 100 ELSE 0 END), 50)
    INTO v_test_pass_rate
    FROM risk_control_mappings rcm
    JOIN control_tests ct ON rcm.control_id = ct.control_id
    JOIN control_test_results ctr ON ct.id = ctr.control_test_id AND ctr.performed_at > NOW() - INTERVAL '90 days'
    WHERE rcm.risk_id = p_risk_id;

    -- Evidence freshness for mapped controls
    SELECT COALESCE(AVG(
        CASE WHEN e.valid_until IS NULL OR e.valid_until > NOW() THEN 100
             WHEN e.valid_until > NOW() - INTERVAL '30 days' THEN 50
             ELSE 0
        END
    ), 50)
    INTO v_evidence_freshness
    FROM risk_control_mappings rcm
    JOIN evidence_control_links ecl ON rcm.control_id = ecl.control_id
    JOIN evidence e ON ecl.evidence_id = e.id
    WHERE rcm.risk_id = p_risk_id;

    -- Build factor scores
    v_factor_scores := jsonb_build_object(
        'control_effectiveness', v_control_effectiveness,
        'historical_trend', CASE WHEN v_historical_change < 0 THEN 100 WHEN v_historical_change > 0 THEN 0 ELSE 50 END,
        'treatment_progress', v_treatment_progress,
        'control_test_results', v_test_pass_rate,
        'evidence_freshness', v_evidence_freshness
    );

    -- Compute predicted score
    -- Positive factors reduce risk, negative factors increase risk
    v_predicted_score := v_current_score - ROUND(
        (v_control_effectiveness * 0.03 +
         v_treatment_progress * 0.02 +
         v_test_pass_rate * 0.02 +
         v_evidence_freshness * 0.01 -
         CASE WHEN v_historical_change > 0 THEN v_historical_change * 0.1 ELSE 0 END)
    )::INTEGER;

    -- Clamp to valid range
    v_predicted_score := GREATEST(1, LEAST(25, v_predicted_score));

    -- Derive predicted likelihood and impact
    IF v_predicted_score < v_current_score THEN
        -- Risk is decreasing, reduce likelihood first
        v_predicted_likelihood := GREATEST(1, v_current_likelihood - CEIL((v_current_score - v_predicted_score) / 3.0)::INTEGER);
        v_predicted_impact := v_current_impact;
    ELSE
        v_predicted_likelihood := v_current_likelihood;
        v_predicted_impact := v_current_impact;
    END IF;

    -- Calculate confidence (based on data availability)
    v_confidence := 50 +
        CASE WHEN v_control_effectiveness > 0 THEN 15 ELSE 0 END +
        CASE WHEN v_historical_change IS NOT NULL THEN 15 ELSE 0 END +
        CASE WHEN v_test_pass_rate != 50 THEN 10 ELSE 0 END +
        CASE WHEN v_evidence_freshness != 50 THEN 10 ELSE 0 END;

    -- Determine trend
    IF v_predicted_score < v_current_score THEN
        v_trend := 'decreasing';
        v_trend_velocity := (v_current_score - v_predicted_score)::DECIMAL / 30; -- per day
    ELSIF v_predicted_score > v_current_score THEN
        v_trend := 'increasing';
        v_trend_velocity := (v_predicted_score - v_current_score)::DECIMAL / 30;
    ELSE
        v_trend := 'stable';
        v_trend_velocity := 0;
    END IF;

    -- Generate explanation
    v_explanation := 'Risk prediction based on: ';
    IF v_control_effectiveness > 70 THEN
        v_explanation := v_explanation || 'Strong control effectiveness. ';
    ELSIF v_control_effectiveness < 30 THEN
        v_explanation := v_explanation || 'Weak control coverage. ';
    END IF;
    IF v_treatment_progress > 50 THEN
        v_explanation := v_explanation || 'Good treatment progress. ';
    END IF;
    IF v_test_pass_rate > 80 THEN
        v_explanation := v_explanation || 'Controls passing tests. ';
    ELSIF v_test_pass_rate < 50 THEN
        v_explanation := v_explanation || 'Control test failures detected. ';
    END IF;

    -- Mark old predictions as not current
    UPDATE risk_predictions SET is_current = FALSE WHERE risk_id = p_risk_id AND is_current = TRUE;

    -- Insert new prediction
    INSERT INTO risk_predictions (
        organization_id, risk_id,
        current_likelihood, current_impact, current_score,
        predicted_likelihood, predicted_impact, predicted_score,
        confidence_level, trend, trend_velocity,
        factor_scores, explanation
    ) VALUES (
        v_org_id, p_risk_id,
        v_current_likelihood, v_current_impact, v_current_score,
        v_predicted_likelihood, v_predicted_impact, v_predicted_score,
        v_confidence, v_trend, v_trend_velocity,
        v_factor_scores, v_explanation
    ) RETURNING id INTO prediction_id;

    RETURN prediction_id;
END;
$$ LANGUAGE plpgsql;

-- Trigger to auto-compute prediction when risk is updated
CREATE OR REPLACE FUNCTION trigger_risk_prediction() RETURNS TRIGGER AS $$
BEGIN
    -- Compute prediction asynchronously would be better, but for now inline
    PERFORM compute_risk_prediction(NEW.id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_risk_prediction
    AFTER INSERT OR UPDATE ON risks
    FOR EACH ROW
    EXECUTE FUNCTION trigger_risk_prediction();

-- ============================================================================
-- 7. UPDATED_AT TRIGGERS
-- ============================================================================

CREATE TRIGGER update_risk_prediction_factors_updated_at
    BEFORE UPDATE ON risk_prediction_factors
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_industry_benchmarks_updated_at
    BEFORE UPDATE ON industry_benchmarks
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_saved_reports_updated_at
    BEFORE UPDATE ON saved_reports
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_dashboard_widgets_updated_at
    BEFORE UPDATE ON dashboard_widgets
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
