-- Automated Control Testing Migration
-- Implements: Test rule templates, continuous monitoring, evidence attachment, alerting, remediation

-- =============================================================================
-- Test Rule Templates
-- Pre-built test configurations for common compliance checks
-- =============================================================================
CREATE TABLE control_test_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100) NOT NULL, -- 'aws', 'github', 'http', 'integration', 'manual'
    subcategory VARCHAR(100), -- e.g., 'iam', 'security-hub', 'branch-protection'
    test_type VARCHAR(50) NOT NULL DEFAULT 'automated',
    automation_config JSONB NOT NULL, -- Template configuration
    default_frequency VARCHAR(50) DEFAULT 'daily',
    applicable_frameworks TEXT[], -- ['SOC2', 'ISO27001', 'HIPAA']
    applicable_controls TEXT[], -- Control codes this test is commonly used for
    tags TEXT[],
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_test_templates_category ON control_test_templates(category);
CREATE INDEX idx_test_templates_active ON control_test_templates(is_active);

-- =============================================================================
-- Alert Configuration for Control Tests
-- Defines alerting thresholds and recipients
-- =============================================================================
CREATE TABLE control_test_alert_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    control_test_id UUID NOT NULL REFERENCES control_tests(id) ON DELETE CASCADE,
    alert_on_failure BOOLEAN DEFAULT true,
    consecutive_failures_threshold INTEGER DEFAULT 1, -- Alert after N consecutive failures
    alert_on_recovery BOOLEAN DEFAULT false, -- Alert when test recovers from failure
    alert_recipients UUID[], -- User IDs to notify
    alert_email_enabled BOOLEAN DEFAULT true,
    alert_in_app_enabled BOOLEAN DEFAULT true,
    escalation_after_hours INTEGER, -- Escalate if not resolved after N hours
    escalation_recipients UUID[], -- Escalation user IDs
    is_muted BOOLEAN DEFAULT false,
    muted_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, control_test_id)
);

CREATE INDEX idx_alert_configs_org ON control_test_alert_configs(organization_id);
CREATE INDEX idx_alert_configs_test ON control_test_alert_configs(control_test_id);

-- =============================================================================
-- Remediation Suggestions
-- Suggested actions for test failures
-- =============================================================================
CREATE TABLE control_test_remediations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    control_test_id UUID REFERENCES control_tests(id) ON DELETE CASCADE, -- NULL for global templates
    failure_pattern VARCHAR(500), -- Regex or substring to match in failure notes
    severity VARCHAR(50) DEFAULT 'medium', -- 'low', 'medium', 'high', 'critical'
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    remediation_steps TEXT[] NOT NULL, -- Ordered list of steps
    documentation_url VARCHAR(500),
    estimated_effort VARCHAR(50), -- 'minutes', 'hours', 'days'
    auto_generated BOOLEAN DEFAULT false,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_remediations_test ON control_test_remediations(control_test_id);

-- =============================================================================
-- Control Monitoring Status
-- Tracks continuous monitoring state per control
-- =============================================================================
CREATE TABLE control_monitoring_status (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    control_id UUID NOT NULL REFERENCES controls(id) ON DELETE CASCADE,
    monitoring_enabled BOOLEAN DEFAULT true,
    last_test_at TIMESTAMPTZ,
    last_test_status VARCHAR(50), -- 'passed', 'failed', 'skipped', 'error'
    consecutive_failures INTEGER DEFAULT 0,
    consecutive_passes INTEGER DEFAULT 0,
    current_health VARCHAR(50) DEFAULT 'unknown', -- 'healthy', 'degraded', 'failing', 'unknown'
    health_score INTEGER DEFAULT 100, -- 0-100
    total_tests INTEGER DEFAULT 0,
    passed_tests INTEGER DEFAULT 0,
    failed_tests INTEGER DEFAULT 0,
    last_failure_at TIMESTAMPTZ,
    last_failure_reason TEXT,
    alert_status VARCHAR(50) DEFAULT 'none', -- 'none', 'alerting', 'muted', 'acknowledged'
    acknowledged_by UUID REFERENCES users(id),
    acknowledged_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, control_id)
);

CREATE INDEX idx_monitoring_status_org ON control_monitoring_status(organization_id);
CREATE INDEX idx_monitoring_status_health ON control_monitoring_status(current_health);
CREATE INDEX idx_monitoring_status_alert ON control_monitoring_status(alert_status);

-- =============================================================================
-- Test Execution Runs
-- Detailed tracking of automated test executions
-- =============================================================================
CREATE TABLE control_test_runs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    control_test_id UUID NOT NULL REFERENCES control_tests(id) ON DELETE CASCADE,
    control_id UUID NOT NULL REFERENCES controls(id) ON DELETE CASCADE,
    run_type VARCHAR(50) DEFAULT 'scheduled', -- 'scheduled', 'manual', 'triggered'
    triggered_by UUID REFERENCES users(id),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'running', -- 'running', 'passed', 'failed', 'error', 'timeout', 'skipped'
    execution_time_ms INTEGER,
    notes TEXT,
    raw_output TEXT, -- Full output/response from test
    error_message TEXT,
    evidence_created_id UUID REFERENCES evidence(id), -- Auto-created evidence
    alert_sent BOOLEAN DEFAULT false,
    alert_sent_at TIMESTAMPTZ,
    remediation_suggested_id UUID REFERENCES control_test_remediations(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_test_runs_org ON control_test_runs(organization_id);
CREATE INDEX idx_test_runs_test ON control_test_runs(control_test_id);
CREATE INDEX idx_test_runs_control ON control_test_runs(control_id);
CREATE INDEX idx_test_runs_status ON control_test_runs(status);
CREATE INDEX idx_test_runs_started ON control_test_runs(started_at DESC);

-- =============================================================================
-- Test Alerts History
-- Tracks all alerts sent for test failures
-- =============================================================================
CREATE TABLE control_test_alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    control_test_id UUID NOT NULL REFERENCES control_tests(id) ON DELETE CASCADE,
    test_run_id UUID REFERENCES control_test_runs(id) ON DELETE SET NULL,
    alert_type VARCHAR(50) NOT NULL, -- 'failure', 'recovery', 'escalation'
    severity VARCHAR(50) DEFAULT 'medium',
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    recipients UUID[] NOT NULL,
    email_sent BOOLEAN DEFAULT false,
    in_app_sent BOOLEAN DEFAULT false,
    acknowledged_by UUID REFERENCES users(id),
    acknowledged_at TIMESTAMPTZ,
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMPTZ,
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_test_alerts_org ON control_test_alerts(organization_id);
CREATE INDEX idx_test_alerts_test ON control_test_alerts(control_test_id);
CREATE INDEX idx_test_alerts_type ON control_test_alerts(alert_type);
CREATE INDEX idx_test_alerts_unack ON control_test_alerts(organization_id, acknowledged_at) WHERE acknowledged_at IS NULL;

-- =============================================================================
-- Add new columns to control_tests for enhanced automation
-- =============================================================================
ALTER TABLE control_tests ADD COLUMN IF NOT EXISTS template_id UUID REFERENCES control_test_templates(id);
ALTER TABLE control_tests ADD COLUMN IF NOT EXISTS last_run_at TIMESTAMPTZ;
ALTER TABLE control_tests ADD COLUMN IF NOT EXISTS last_run_status VARCHAR(50);
ALTER TABLE control_tests ADD COLUMN IF NOT EXISTS run_count INTEGER DEFAULT 0;
ALTER TABLE control_tests ADD COLUMN IF NOT EXISTS pass_count INTEGER DEFAULT 0;
ALTER TABLE control_tests ADD COLUMN IF NOT EXISTS fail_count INTEGER DEFAULT 0;
ALTER TABLE control_tests ADD COLUMN IF NOT EXISTS is_enabled BOOLEAN DEFAULT true;
ALTER TABLE control_tests ADD COLUMN IF NOT EXISTS timeout_seconds INTEGER DEFAULT 30;
ALTER TABLE control_tests ADD COLUMN IF NOT EXISTS retry_count INTEGER DEFAULT 0;
ALTER TABLE control_tests ADD COLUMN IF NOT EXISTS retry_delay_seconds INTEGER DEFAULT 5;

-- =============================================================================
-- Add organization_id to control_tests for easier querying
-- =============================================================================
ALTER TABLE control_tests ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE;

-- Backfill organization_id from control
UPDATE control_tests ct
SET organization_id = c.organization_id
FROM controls c
WHERE ct.control_id = c.id AND ct.organization_id IS NULL;

-- =============================================================================
-- Seed Test Rule Templates
-- =============================================================================
INSERT INTO control_test_templates (name, description, category, subcategory, test_type, automation_config, default_frequency, applicable_frameworks, tags) VALUES

-- AWS IAM Tests
('AWS MFA Enabled Check', 'Verify MFA is enabled for all IAM users', 'aws', 'iam', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "iam_mfa_enabled", "expected_result": "all_users_have_mfa"}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001', 'HIPAA', 'PCI-DSS'], ARRAY['iam', 'mfa', 'authentication']),

('AWS Root Account MFA', 'Verify root account has MFA enabled', 'aws', 'iam', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "root_mfa_enabled"}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001', 'PCI-DSS'], ARRAY['iam', 'root', 'mfa']),

('AWS Access Keys Rotation', 'Check IAM access keys are rotated within 90 days', 'aws', 'iam', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "access_key_rotation", "max_age_days": 90}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001', 'PCI-DSS'], ARRAY['iam', 'access-keys', 'rotation']),

('AWS Unused Credentials Check', 'Identify IAM credentials not used in 90 days', 'aws', 'iam', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "unused_credentials", "unused_days": 90}'::jsonb,
 'weekly', ARRAY['SOC2', 'ISO27001'], ARRAY['iam', 'credentials', 'cleanup']),

-- AWS S3 Tests
('AWS S3 Public Access Check', 'Verify no S3 buckets have public access', 'aws', 's3', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "s3_no_public_access"}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001', 'HIPAA', 'PCI-DSS'], ARRAY['s3', 'public-access', 'data-protection']),

('AWS S3 Encryption Check', 'Verify S3 buckets have encryption enabled', 'aws', 's3', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "s3_encryption_enabled"}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001', 'HIPAA', 'PCI-DSS'], ARRAY['s3', 'encryption', 'data-protection']),

('AWS S3 Versioning Check', 'Verify S3 buckets have versioning enabled', 'aws', 's3', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "s3_versioning_enabled"}'::jsonb,
 'weekly', ARRAY['SOC2', 'ISO27001'], ARRAY['s3', 'versioning', 'backup']),

-- AWS Security Hub Tests
('AWS Security Hub Findings', 'Check for critical/high severity findings', 'aws', 'security-hub', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "security_hub_findings", "max_critical": 0, "max_high": 5}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001'], ARRAY['security-hub', 'findings', 'vulnerabilities']),

-- AWS CloudTrail Tests
('AWS CloudTrail Enabled', 'Verify CloudTrail is enabled in all regions', 'aws', 'cloudtrail', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "cloudtrail_enabled"}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001', 'HIPAA', 'PCI-DSS'], ARRAY['cloudtrail', 'logging', 'audit']),

('AWS CloudTrail Log Validation', 'Verify CloudTrail log file validation is enabled', 'aws', 'cloudtrail', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "cloudtrail_log_validation"}'::jsonb,
 'weekly', ARRAY['SOC2', 'ISO27001'], ARRAY['cloudtrail', 'integrity', 'logging']),

-- AWS RDS Tests
('AWS RDS Encryption Check', 'Verify RDS instances have encryption enabled', 'aws', 'rds', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "rds_encryption_enabled"}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001', 'HIPAA', 'PCI-DSS'], ARRAY['rds', 'encryption', 'database']),

('AWS RDS Public Access Check', 'Verify RDS instances are not publicly accessible', 'aws', 'rds', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "rds_no_public_access"}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001', 'HIPAA', 'PCI-DSS'], ARRAY['rds', 'public-access', 'database']),

-- AWS EC2 Tests
('AWS EC2 Security Groups Check', 'Verify no security groups allow unrestricted ingress', 'aws', 'ec2', 'automated',
 '{"automation_type": "integration", "integration_type": "aws", "check_type": "ec2_restricted_security_groups"}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001', 'PCI-DSS'], ARRAY['ec2', 'security-groups', 'network']),

-- GitHub Tests
('GitHub Branch Protection', 'Verify default branch has protection rules', 'github', 'repos', 'automated',
 '{"automation_type": "integration", "integration_type": "github", "check_type": "branch_protection_enabled"}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001'], ARRAY['github', 'branch-protection', 'code-review']),

('GitHub Required Reviews', 'Verify PRs require review before merge', 'github', 'repos', 'automated',
 '{"automation_type": "integration", "integration_type": "github", "check_type": "required_reviews", "min_reviews": 1}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001'], ARRAY['github', 'code-review', 'approvals']),

('GitHub Security Alerts', 'Check for unresolved security vulnerabilities', 'github', 'security', 'automated',
 '{"automation_type": "integration", "integration_type": "github", "check_type": "security_alerts", "max_critical": 0}'::jsonb,
 'daily', ARRAY['SOC2', 'ISO27001'], ARRAY['github', 'vulnerabilities', 'dependencies']),

('GitHub 2FA Enforcement', 'Verify 2FA is required for organization members', 'github', 'org', 'automated',
 '{"automation_type": "integration", "integration_type": "github", "check_type": "two_factor_required"}'::jsonb,
 'weekly', ARRAY['SOC2', 'ISO27001'], ARRAY['github', '2fa', 'authentication']),

-- HTTP Endpoint Tests
('Health Endpoint Check', 'Verify application health endpoint returns 200', 'http', 'availability', 'automated',
 '{"automation_type": "http", "method": "GET", "expected_status_codes": [200], "timeout_seconds": 10}'::jsonb,
 'continuous', ARRAY['SOC2'], ARRAY['http', 'health', 'availability']),

('HTTPS Certificate Valid', 'Verify SSL/TLS certificate is valid and not expiring', 'http', 'security', 'automated',
 '{"automation_type": "http", "method": "GET", "check_type": "ssl_certificate", "min_days_valid": 30}'::jsonb,
 'daily', ARRAY['SOC2', 'PCI-DSS'], ARRAY['http', 'ssl', 'certificate']),

('Security Headers Check', 'Verify security headers are present', 'http', 'security', 'automated',
 '{"automation_type": "http", "method": "GET", "check_headers": ["Strict-Transport-Security", "X-Content-Type-Options", "X-Frame-Options"]}'::jsonb,
 'daily', ARRAY['SOC2', 'OWASP'], ARRAY['http', 'security-headers', 'web']);

-- =============================================================================
-- Seed Default Remediation Suggestions
-- =============================================================================
INSERT INTO control_test_remediations (control_test_id, failure_pattern, severity, title, description, remediation_steps, documentation_url, estimated_effort, auto_generated)
SELECT
    NULL,  -- These are template remediations, not linked to specific tests
    'mfa.*not.*enabled',
    'high',
    'Enable MFA for IAM Users',
    'Multi-factor authentication is not enabled for one or more IAM users, which increases the risk of unauthorized access.',
    ARRAY[
        'Navigate to IAM in the AWS Console',
        'Select Users from the left navigation',
        'Click on the user without MFA',
        'Select the Security credentials tab',
        'Click Assign MFA device',
        'Choose Virtual MFA device and follow the setup wizard',
        'Verify the MFA device is working by signing in'
    ],
    'https://docs.aws.amazon.com/IAM/latest/UserGuide/id_credentials_mfa_enable_virtual.html',
    'minutes',
    true
WHERE NOT EXISTS (SELECT 1 FROM control_test_remediations WHERE failure_pattern = 'mfa.*not.*enabled');

INSERT INTO control_test_remediations (control_test_id, failure_pattern, severity, title, description, remediation_steps, documentation_url, estimated_effort, auto_generated)
SELECT
    NULL,
    'access.*key.*rotation|key.*older.*than',
    'medium',
    'Rotate IAM Access Keys',
    'IAM access keys have not been rotated within the required timeframe, increasing the risk of key compromise.',
    ARRAY[
        'Create a new access key for the IAM user',
        'Update all applications using the old key with the new key',
        'Disable the old access key',
        'Test that applications still work with the new key',
        'Delete the old access key'
    ],
    'https://docs.aws.amazon.com/IAM/latest/UserGuide/id_credentials_access-keys.html#Using_RotateAccessKey',
    'hours',
    true
WHERE NOT EXISTS (SELECT 1 FROM control_test_remediations WHERE failure_pattern = 'access.*key.*rotation|key.*older.*than');

INSERT INTO control_test_remediations (control_test_id, failure_pattern, severity, title, description, remediation_steps, documentation_url, estimated_effort, auto_generated)
SELECT
    NULL,
    's3.*public|bucket.*public',
    'critical',
    'Remove S3 Public Access',
    'One or more S3 buckets have public access enabled, which could expose sensitive data.',
    ARRAY[
        'Navigate to S3 in the AWS Console',
        'Select the bucket with public access',
        'Go to the Permissions tab',
        'Click Block public access and enable all options',
        'Review and remove any public bucket policies',
        'Check Access Control List (ACL) for public grants',
        'Verify the bucket is no longer publicly accessible'
    ],
    'https://docs.aws.amazon.com/AmazonS3/latest/userguide/access-control-block-public-access.html',
    'minutes',
    true
WHERE NOT EXISTS (SELECT 1 FROM control_test_remediations WHERE failure_pattern = 's3.*public|bucket.*public');

INSERT INTO control_test_remediations (control_test_id, failure_pattern, severity, title, description, remediation_steps, documentation_url, estimated_effort, auto_generated)
SELECT
    NULL,
    'branch.*protection.*disabled|no.*branch.*protection',
    'medium',
    'Enable Branch Protection',
    'Default branch does not have protection rules enabled, allowing direct pushes without review.',
    ARRAY[
        'Navigate to the repository Settings on GitHub',
        'Select Branches from the left menu',
        'Click Add branch protection rule',
        'Enter the branch name pattern (e.g., main, master)',
        'Enable Require a pull request before merging',
        'Enable Require approvals and set minimum reviewers',
        'Enable Require status checks to pass before merging',
        'Save the protection rule'
    ],
    'https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/managing-protected-branches/managing-a-branch-protection-rule',
    'minutes',
    true
WHERE NOT EXISTS (SELECT 1 FROM control_test_remediations WHERE failure_pattern = 'branch.*protection.*disabled|no.*branch.*protection');

-- =============================================================================
-- Views for Monitoring Dashboard
-- =============================================================================
CREATE OR REPLACE VIEW v_control_monitoring_summary AS
SELECT
    cms.organization_id,
    COUNT(*) as total_controls,
    COUNT(*) FILTER (WHERE cms.monitoring_enabled = true) as monitored_controls,
    COUNT(*) FILTER (WHERE cms.current_health = 'healthy') as healthy_controls,
    COUNT(*) FILTER (WHERE cms.current_health = 'degraded') as degraded_controls,
    COUNT(*) FILTER (WHERE cms.current_health = 'failing') as failing_controls,
    COUNT(*) FILTER (WHERE cms.current_health = 'unknown') as unknown_controls,
    COUNT(*) FILTER (WHERE cms.alert_status = 'alerting') as alerting_controls,
    ROUND(AVG(cms.health_score)::numeric, 1) as avg_health_score,
    SUM(cms.total_tests) as total_test_runs,
    SUM(cms.passed_tests) as total_passed,
    SUM(cms.failed_tests) as total_failed
FROM control_monitoring_status cms
GROUP BY cms.organization_id;

CREATE OR REPLACE VIEW v_recent_test_failures AS
SELECT
    ctr.organization_id,
    ctr.id as run_id,
    ctr.control_test_id,
    ct.name as test_name,
    c.code as control_code,
    c.name as control_name,
    ctr.status,
    ctr.error_message,
    ctr.started_at,
    ctr.completed_at,
    ctr.execution_time_ms,
    ctr.alert_sent,
    ctr.remediation_suggested_id
FROM control_test_runs ctr
JOIN control_tests ct ON ctr.control_test_id = ct.id
JOIN controls c ON ctr.control_id = c.id
WHERE ctr.status IN ('failed', 'error')
ORDER BY ctr.started_at DESC;

CREATE OR REPLACE VIEW v_unacknowledged_alerts AS
SELECT
    cta.organization_id,
    cta.id as alert_id,
    cta.control_test_id,
    ct.name as test_name,
    c.code as control_code,
    c.name as control_name,
    cta.alert_type,
    cta.severity,
    cta.title,
    cta.message,
    cta.created_at
FROM control_test_alerts cta
JOIN control_tests ct ON cta.control_test_id = ct.id
JOIN controls c ON ct.control_id = c.id
WHERE cta.acknowledged_at IS NULL AND cta.resolved_at IS NULL
ORDER BY
    CASE cta.severity
        WHEN 'critical' THEN 1
        WHEN 'high' THEN 2
        WHEN 'medium' THEN 3
        ELSE 4
    END,
    cta.created_at DESC;

-- =============================================================================
-- Functions
-- =============================================================================

-- Function to update control monitoring status after test run
CREATE OR REPLACE FUNCTION update_control_monitoring_status(
    p_organization_id UUID,
    p_control_id UUID,
    p_test_status VARCHAR(50)
) RETURNS void AS $$
DECLARE
    v_consecutive_failures INTEGER;
    v_consecutive_passes INTEGER;
    v_health VARCHAR(50);
    v_health_score INTEGER;
BEGIN
    -- Ensure monitoring status record exists
    INSERT INTO control_monitoring_status (organization_id, control_id)
    VALUES (p_organization_id, p_control_id)
    ON CONFLICT (organization_id, control_id) DO NOTHING;

    -- Update based on test result
    IF p_test_status = 'passed' THEN
        UPDATE control_monitoring_status
        SET
            last_test_at = NOW(),
            last_test_status = p_test_status,
            consecutive_passes = consecutive_passes + 1,
            consecutive_failures = 0,
            total_tests = total_tests + 1,
            passed_tests = passed_tests + 1,
            updated_at = NOW()
        WHERE organization_id = p_organization_id AND control_id = p_control_id;
    ELSE
        UPDATE control_monitoring_status
        SET
            last_test_at = NOW(),
            last_test_status = p_test_status,
            consecutive_failures = consecutive_failures + 1,
            consecutive_passes = 0,
            total_tests = total_tests + 1,
            failed_tests = failed_tests + 1,
            last_failure_at = NOW(),
            last_failure_reason = CASE WHEN p_test_status = 'error' THEN 'Test execution error' ELSE 'Test failed' END,
            updated_at = NOW()
        WHERE organization_id = p_organization_id AND control_id = p_control_id;
    END IF;

    -- Calculate health status
    SELECT consecutive_failures, consecutive_passes INTO v_consecutive_failures, v_consecutive_passes
    FROM control_monitoring_status
    WHERE organization_id = p_organization_id AND control_id = p_control_id;

    -- Determine health based on consecutive results
    IF v_consecutive_failures >= 3 THEN
        v_health := 'failing';
        v_health_score := GREATEST(0, 100 - (v_consecutive_failures * 20));
    ELSIF v_consecutive_failures >= 1 THEN
        v_health := 'degraded';
        v_health_score := GREATEST(50, 100 - (v_consecutive_failures * 15));
    ELSIF v_consecutive_passes >= 3 THEN
        v_health := 'healthy';
        v_health_score := 100;
    ELSE
        v_health := 'healthy';
        v_health_score := LEAST(100, 80 + (v_consecutive_passes * 5));
    END IF;

    -- Update health
    UPDATE control_monitoring_status
    SET
        current_health = v_health,
        health_score = v_health_score,
        alert_status = CASE
            WHEN v_health = 'failing' AND alert_status = 'none' THEN 'alerting'
            WHEN v_health = 'healthy' AND alert_status = 'alerting' THEN 'none'
            ELSE alert_status
        END
    WHERE organization_id = p_organization_id AND control_id = p_control_id;
END;
$$ LANGUAGE plpgsql;

-- Function to find matching remediation for a failure
CREATE OR REPLACE FUNCTION find_matching_remediation(
    p_control_test_id UUID,
    p_failure_message TEXT
) RETURNS UUID AS $$
DECLARE
    v_remediation_id UUID;
BEGIN
    -- First try test-specific remediations
    SELECT id INTO v_remediation_id
    FROM control_test_remediations
    WHERE control_test_id = p_control_test_id
      AND p_failure_message ~* failure_pattern
    ORDER BY severity DESC, created_at DESC
    LIMIT 1;

    -- If not found, try global template remediations
    IF v_remediation_id IS NULL THEN
        SELECT id INTO v_remediation_id
        FROM control_test_remediations
        WHERE control_test_id IS NULL
          AND p_failure_message ~* failure_pattern
        ORDER BY severity DESC, created_at DESC
        LIMIT 1;
    END IF;

    RETURN v_remediation_id;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- Add email template for control test alerts
-- =============================================================================
INSERT INTO email_templates (organization_id, template_type, subject, body_html, body_text)
VALUES (
    NULL,
    'control_test_failure',
    '[OpenGRC] Control Test Failed: {{test_name}}',
    '<html>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333;">
<div style="max-width: 600px; margin: 0 auto; padding: 20px;">
<h2 style="color: #dc2626;">Control Test Failure Alert</h2>
<p>A control test has failed in your organization.</p>
<table style="width: 100%; border-collapse: collapse; margin: 20px 0;">
<tr><td style="padding: 8px; border-bottom: 1px solid #eee; font-weight: bold;">Control:</td><td style="padding: 8px; border-bottom: 1px solid #eee;">{{control_code}} - {{control_name}}</td></tr>
<tr><td style="padding: 8px; border-bottom: 1px solid #eee; font-weight: bold;">Test:</td><td style="padding: 8px; border-bottom: 1px solid #eee;">{{test_name}}</td></tr>
<tr><td style="padding: 8px; border-bottom: 1px solid #eee; font-weight: bold;">Status:</td><td style="padding: 8px; border-bottom: 1px solid #eee; color: #dc2626;">{{status}}</td></tr>
<tr><td style="padding: 8px; border-bottom: 1px solid #eee; font-weight: bold;">Time:</td><td style="padding: 8px; border-bottom: 1px solid #eee;">{{timestamp}}</td></tr>
<tr><td style="padding: 8px; border-bottom: 1px solid #eee; font-weight: bold;">Consecutive Failures:</td><td style="padding: 8px; border-bottom: 1px solid #eee;">{{consecutive_failures}}</td></tr>
</table>
<h3>Error Details</h3>
<div style="background: #fef2f2; padding: 15px; border-radius: 4px; margin: 10px 0;">
<pre style="margin: 0; white-space: pre-wrap;">{{error_message}}</pre>
</div>
{{#if remediation_title}}
<h3>Suggested Remediation</h3>
<p><strong>{{remediation_title}}</strong></p>
<p>{{remediation_description}}</p>
{{/if}}
<p style="margin-top: 30px;">
<a href="{{dashboard_url}}" style="background: #2563eb; color: white; padding: 12px 24px; text-decoration: none; border-radius: 4px;">View in Dashboard</a>
</p>
</div>
</body>
</html>',
    'Control Test Failure Alert

A control test has failed in your organization.

Control: {{control_code}} - {{control_name}}
Test: {{test_name}}
Status: {{status}}
Time: {{timestamp}}
Consecutive Failures: {{consecutive_failures}}

Error Details:
{{error_message}}

{{#if remediation_title}}
Suggested Remediation: {{remediation_title}}
{{remediation_description}}
{{/if}}

View in Dashboard: {{dashboard_url}}'
) ON CONFLICT DO NOTHING;
