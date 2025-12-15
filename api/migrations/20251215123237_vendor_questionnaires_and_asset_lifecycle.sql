-- Vendor Security Questionnaires and Asset Lifecycle Tracking
-- Migration for Phase 3.1 and 3.2

-- ==================== VENDOR QUESTIONNAIRES ====================

-- Questionnaire templates (reusable questionnaire definitions)
CREATE TABLE questionnaire_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    category VARCHAR(100), -- security, privacy, compliance, general
    is_default BOOLEAN DEFAULT false,
    version INTEGER DEFAULT 1,
    status VARCHAR(50) DEFAULT 'draft', -- draft, published, archived
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_questionnaire_templates_org ON questionnaire_templates(organization_id);
CREATE INDEX idx_questionnaire_templates_status ON questionnaire_templates(status);

-- Questionnaire sections (group questions logically)
CREATE TABLE questionnaire_sections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    template_id UUID NOT NULL REFERENCES questionnaire_templates(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_questionnaire_sections_template ON questionnaire_sections(template_id);

-- Questionnaire questions
CREATE TABLE questionnaire_questions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    template_id UUID NOT NULL REFERENCES questionnaire_templates(id) ON DELETE CASCADE,
    section_id UUID REFERENCES questionnaire_sections(id) ON DELETE SET NULL,
    question_text TEXT NOT NULL,
    help_text TEXT,
    question_type VARCHAR(50) NOT NULL, -- text, textarea, single_choice, multiple_choice, yes_no, file_upload, date, number
    options JSONB, -- for choice questions: [{"value": "yes", "label": "Yes"}, ...]
    is_required BOOLEAN DEFAULT true,
    weight INTEGER DEFAULT 1, -- for scoring
    risk_mapping VARCHAR(50), -- maps to risk level: critical, high, medium, low
    control_codes TEXT[], -- maps to control codes like CC6.1, CC6.2
    sort_order INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_questionnaire_questions_template ON questionnaire_questions(template_id);
CREATE INDEX idx_questionnaire_questions_section ON questionnaire_questions(section_id);

-- Questionnaire assignments (sending questionnaires to vendors)
CREATE TABLE questionnaire_assignments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    template_id UUID NOT NULL REFERENCES questionnaire_templates(id) ON DELETE CASCADE,
    vendor_id UUID NOT NULL REFERENCES vendors(id) ON DELETE CASCADE,
    access_token VARCHAR(255) NOT NULL UNIQUE, -- token for vendor portal access
    status VARCHAR(50) DEFAULT 'pending', -- pending, in_progress, submitted, reviewed, approved, rejected
    assigned_by UUID REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    due_date DATE,
    submitted_at TIMESTAMPTZ,
    reviewed_by UUID REFERENCES users(id),
    reviewed_at TIMESTAMPTZ,
    review_notes TEXT,
    score DECIMAL(5,2), -- calculated score 0-100
    risk_rating VARCHAR(50), -- calculated: critical, high, medium, low
    reminder_sent_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ, -- token expiration
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_questionnaire_assignments_org ON questionnaire_assignments(organization_id);
CREATE INDEX idx_questionnaire_assignments_vendor ON questionnaire_assignments(vendor_id);
CREATE INDEX idx_questionnaire_assignments_status ON questionnaire_assignments(status);
CREATE INDEX idx_questionnaire_assignments_token ON questionnaire_assignments(access_token);
CREATE INDEX idx_questionnaire_assignments_due ON questionnaire_assignments(due_date) WHERE status IN ('pending', 'in_progress');

-- Questionnaire responses (vendor answers)
CREATE TABLE questionnaire_responses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    assignment_id UUID NOT NULL REFERENCES questionnaire_assignments(id) ON DELETE CASCADE,
    question_id UUID NOT NULL REFERENCES questionnaire_questions(id) ON DELETE CASCADE,
    response_text TEXT,
    response_value JSONB, -- for structured responses (choices, multiple values)
    file_path VARCHAR(500), -- for file upload questions
    file_name VARCHAR(255),
    answered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(assignment_id, question_id)
);

CREATE INDEX idx_questionnaire_responses_assignment ON questionnaire_responses(assignment_id);
CREATE INDEX idx_questionnaire_responses_question ON questionnaire_responses(question_id);

-- Questionnaire response comments (for reviewer feedback on individual questions)
CREATE TABLE questionnaire_response_comments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    response_id UUID NOT NULL REFERENCES questionnaire_responses(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id),
    comment TEXT NOT NULL,
    is_internal BOOLEAN DEFAULT true, -- internal comments not visible to vendor
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_questionnaire_response_comments_response ON questionnaire_response_comments(response_id);

-- ==================== ASSET LIFECYCLE TRACKING ====================

-- Add lifecycle tracking columns to assets table
ALTER TABLE assets
ADD COLUMN IF NOT EXISTS lifecycle_stage VARCHAR(50) DEFAULT 'active', -- procurement, deployment, active, maintenance, decommissioning, decommissioned
ADD COLUMN IF NOT EXISTS commissioned_date DATE,
ADD COLUMN IF NOT EXISTS decommission_date DATE,
ADD COLUMN IF NOT EXISTS last_maintenance_date DATE,
ADD COLUMN IF NOT EXISTS next_maintenance_due DATE,
ADD COLUMN IF NOT EXISTS maintenance_frequency VARCHAR(50), -- monthly, quarterly, semi-annual, annual
ADD COLUMN IF NOT EXISTS end_of_life_date DATE,
ADD COLUMN IF NOT EXISTS end_of_support_date DATE,
ADD COLUMN IF NOT EXISTS integration_source VARCHAR(100), -- aws, azure, gcp, manual
ADD COLUMN IF NOT EXISTS integration_id UUID REFERENCES integrations(id) ON DELETE SET NULL,
ADD COLUMN IF NOT EXISTS external_id VARCHAR(255), -- ID from external system (EC2 instance ID, etc.)
ADD COLUMN IF NOT EXISTS last_synced_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_assets_lifecycle_stage ON assets(lifecycle_stage);
CREATE INDEX IF NOT EXISTS idx_assets_integration_source ON assets(integration_source);
CREATE INDEX IF NOT EXISTS idx_assets_external_id ON assets(external_id);
CREATE INDEX IF NOT EXISTS idx_assets_next_maintenance ON assets(next_maintenance_due) WHERE next_maintenance_due IS NOT NULL;

-- Asset lifecycle events (audit trail of lifecycle changes)
CREATE TABLE asset_lifecycle_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    asset_id UUID NOT NULL REFERENCES assets(id) ON DELETE CASCADE,
    event_type VARCHAR(50) NOT NULL, -- created, deployed, maintained, decommissioned, restored, updated, synced
    previous_stage VARCHAR(50),
    new_stage VARCHAR(50),
    notes TEXT,
    performed_by UUID REFERENCES users(id),
    performed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX idx_asset_lifecycle_events_asset ON asset_lifecycle_events(asset_id);
CREATE INDEX idx_asset_lifecycle_events_type ON asset_lifecycle_events(event_type);
CREATE INDEX idx_asset_lifecycle_events_performed ON asset_lifecycle_events(performed_at);

-- ==================== TRIGGERS ====================

-- Update timestamps trigger for new tables
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_questionnaire_templates_updated_at
    BEFORE UPDATE ON questionnaire_templates
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_questionnaire_questions_updated_at
    BEFORE UPDATE ON questionnaire_questions
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_questionnaire_assignments_updated_at
    BEFORE UPDATE ON questionnaire_assignments
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_questionnaire_responses_updated_at
    BEFORE UPDATE ON questionnaire_responses
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();
