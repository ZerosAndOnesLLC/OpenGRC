-- SOC 2 Parsed Reports and Access Reviews Enhancement
-- ====================================================

-- Store parsed SOC 2 report findings from vendor documents
CREATE TABLE soc2_parsed_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    vendor_document_id UUID NOT NULL REFERENCES vendor_documents(id) ON DELETE CASCADE,
    vendor_id UUID NOT NULL REFERENCES vendors(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Report metadata
    report_type VARCHAR(50),  -- 'type1', 'type2'
    audit_period_start DATE,
    audit_period_end DATE,
    auditor_firm VARCHAR(255),
    opinion_type VARCHAR(50),  -- 'unqualified', 'qualified', 'adverse', 'disclaimer'

    -- Trust Service Categories covered
    trust_services_criteria JSONB DEFAULT '[]',  -- ['security', 'availability', 'processing_integrity', 'confidentiality', 'privacy']

    -- Findings summary
    total_exceptions INTEGER DEFAULT 0,
    critical_exceptions INTEGER DEFAULT 0,
    high_exceptions INTEGER DEFAULT 0,
    medium_exceptions INTEGER DEFAULT 0,
    low_exceptions INTEGER DEFAULT 0,

    -- Raw parsed data
    raw_findings JSONB DEFAULT '[]',  -- Array of finding objects
    subservice_organizations JSONB DEFAULT '[]',  -- Carve-outs
    complementary_user_entity_controls JSONB DEFAULT '[]',  -- CUECs

    -- Processing info
    parsed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    parsing_version VARCHAR(20) DEFAULT '1.0',
    confidence_score DECIMAL(5,2),  -- How confident we are in the parsing (0-100)
    raw_text_hash VARCHAR(64),  -- SHA256 of raw text for deduplication

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Individual findings/exceptions from SOC 2 reports
CREATE TABLE soc2_findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parsed_report_id UUID NOT NULL REFERENCES soc2_parsed_reports(id) ON DELETE CASCADE,

    -- Finding details
    finding_type VARCHAR(50) NOT NULL,  -- 'exception', 'observation', 'deficiency'
    severity VARCHAR(20),  -- 'critical', 'high', 'medium', 'low', 'informational'
    title VARCHAR(500),
    description TEXT,

    -- Trust Service Criteria affected
    criteria_codes TEXT[],  -- ['CC6.1', 'CC6.2', etc.]

    -- Management response
    management_response TEXT,
    remediation_status VARCHAR(50),  -- 'remediated', 'in_progress', 'accepted', 'pending'
    remediation_date DATE,

    -- Impact assessment (from parser)
    potential_impact TEXT,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- ====================================================
-- Access Reviews Enhancement
-- ====================================================

-- Add integration source tracking to access review campaigns
ALTER TABLE access_review_campaigns
    ADD COLUMN IF NOT EXISTS integration_type VARCHAR(50),
    ADD COLUMN IF NOT EXISTS integration_id UUID REFERENCES integrations(id) ON DELETE SET NULL,
    ADD COLUMN IF NOT EXISTS scope JSONB DEFAULT '{}',  -- What to review: specific apps, roles, etc.
    ADD COLUMN IF NOT EXISTS review_type VARCHAR(50) DEFAULT 'periodic',  -- 'periodic', 'termination', 'role_change'
    ADD COLUMN IF NOT EXISTS reminder_sent_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN IF NOT EXISTS last_sync_at TIMESTAMP WITH TIME ZONE;

-- Add more fields to access review items
ALTER TABLE access_review_items
    ADD COLUMN IF NOT EXISTS integration_user_id VARCHAR(255),  -- External user ID from integration
    ADD COLUMN IF NOT EXISTS department VARCHAR(255),
    ADD COLUMN IF NOT EXISTS manager VARCHAR(255),
    ADD COLUMN IF NOT EXISTS last_login_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN IF NOT EXISTS risk_level VARCHAR(20),  -- 'high', 'medium', 'low'
    ADD COLUMN IF NOT EXISTS mfa_enabled BOOLEAN,
    ADD COLUMN IF NOT EXISTS is_admin BOOLEAN DEFAULT false,
    ADD COLUMN IF NOT EXISTS applications JSONB DEFAULT '[]',  -- Apps/roles user has access to
    ADD COLUMN IF NOT EXISTS removal_requested_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN IF NOT EXISTS removal_completed_at TIMESTAMP WITH TIME ZONE,
    ADD COLUMN IF NOT EXISTS removal_ticket_id VARCHAR(255);  -- External ticket for tracking

-- Track access removal actions
CREATE TABLE access_removal_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    access_review_item_id UUID NOT NULL REFERENCES access_review_items(id) ON DELETE CASCADE,
    campaign_id UUID NOT NULL REFERENCES access_review_campaigns(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- What was removed
    user_identifier VARCHAR(255) NOT NULL,
    user_name VARCHAR(255),
    access_type VARCHAR(100),  -- 'account', 'role', 'application', 'permission'
    access_description TEXT,

    -- Action details
    action VARCHAR(50) NOT NULL,  -- 'disabled', 'deleted', 'role_removed', 'downgraded'
    action_reason TEXT,

    -- Who/when
    requested_by UUID REFERENCES users(id),
    requested_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    executed_by UUID REFERENCES users(id),
    executed_at TIMESTAMP WITH TIME ZONE,

    -- Status tracking
    status VARCHAR(50) DEFAULT 'pending',  -- 'pending', 'in_progress', 'completed', 'failed', 'cancelled'
    error_message TEXT,

    -- External tracking
    external_ticket_id VARCHAR(255),
    external_ticket_url TEXT,

    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_soc2_parsed_reports_vendor ON soc2_parsed_reports(vendor_id);
CREATE INDEX idx_soc2_parsed_reports_org ON soc2_parsed_reports(organization_id);
CREATE INDEX idx_soc2_parsed_reports_document ON soc2_parsed_reports(vendor_document_id);
CREATE INDEX idx_soc2_findings_report ON soc2_findings(parsed_report_id);
CREATE INDEX idx_soc2_findings_severity ON soc2_findings(severity);
CREATE INDEX idx_access_removal_logs_item ON access_removal_logs(access_review_item_id);
CREATE INDEX idx_access_removal_logs_campaign ON access_removal_logs(campaign_id);
CREATE INDEX idx_access_removal_logs_status ON access_removal_logs(status);
CREATE INDEX idx_access_review_items_risk ON access_review_items(risk_level);
CREATE INDEX idx_access_review_campaigns_integration ON access_review_campaigns(integration_id);

-- Trigger for updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_soc2_parsed_reports_updated_at
    BEFORE UPDATE ON soc2_parsed_reports
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_access_removal_logs_updated_at
    BEFORE UPDATE ON access_removal_logs
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
