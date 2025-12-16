-- AI Assistant Feature Tables
-- Supports multiple LLM providers (OpenAI, Anthropic, OpenAI-compatible APIs)

-- AI provider configuration per organization
CREATE TABLE ai_configurations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL DEFAULT 'openai',  -- openai, anthropic, openrouter, custom
    api_endpoint VARCHAR(500),  -- Custom endpoint URL (null = use default)
    api_key_encrypted TEXT,  -- Encrypted API key (base64)
    model VARCHAR(100) NOT NULL DEFAULT 'gpt-4o-mini',
    max_tokens INTEGER DEFAULT 4096,
    temperature DECIMAL(3,2) DEFAULT 0.7,
    enabled BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id)
);

-- AI completion history for auditing and caching
CREATE TABLE ai_completions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    feature VARCHAR(50) NOT NULL,  -- policy_draft, evidence_summary, gap_analysis, risk_scoring, nl_search, audit_prep
    prompt_hash VARCHAR(64) NOT NULL,  -- SHA-256 hash for caching similar requests
    input_context JSONB NOT NULL,  -- Structured input data
    prompt_text TEXT NOT NULL,  -- Full prompt sent to LLM
    completion_text TEXT,  -- Response from LLM
    model VARCHAR(100),
    tokens_input INTEGER,
    tokens_output INTEGER,
    latency_ms INTEGER,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',  -- pending, completed, error
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient lookups
CREATE INDEX idx_ai_completions_org ON ai_completions(organization_id);
CREATE INDEX idx_ai_completions_feature ON ai_completions(feature);
CREATE INDEX idx_ai_completions_hash ON ai_completions(prompt_hash);
CREATE INDEX idx_ai_completions_created ON ai_completions(created_at DESC);
CREATE INDEX idx_ai_completions_user ON ai_completions(user_id);

-- AI-generated policy drafts
CREATE TABLE ai_policy_drafts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    completion_id UUID REFERENCES ai_completions(id) ON DELETE SET NULL,
    title VARCHAR(255) NOT NULL,
    category VARCHAR(100),
    framework_codes TEXT[],  -- Array of framework codes this policy addresses
    generated_content TEXT NOT NULL,
    user_prompt TEXT,  -- What the user asked for
    accepted BOOLEAN DEFAULT false,
    accepted_policy_id UUID REFERENCES policies(id) ON DELETE SET NULL,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_policy_drafts_org ON ai_policy_drafts(organization_id);

-- AI evidence summaries
CREATE TABLE ai_evidence_summaries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    evidence_id UUID NOT NULL REFERENCES evidence(id) ON DELETE CASCADE,
    completion_id UUID REFERENCES ai_completions(id) ON DELETE SET NULL,
    summary TEXT NOT NULL,
    key_points JSONB,  -- Array of key points extracted
    compliance_relevance JSONB,  -- Mapping to control codes
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(evidence_id)
);

-- AI gap analysis recommendations
CREATE TABLE ai_gap_recommendations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    framework_id UUID NOT NULL REFERENCES frameworks(id) ON DELETE CASCADE,
    completion_id UUID REFERENCES ai_completions(id) ON DELETE SET NULL,
    requirement_id UUID REFERENCES framework_requirements(id) ON DELETE CASCADE,
    gap_description TEXT NOT NULL,
    recommendation TEXT NOT NULL,
    priority VARCHAR(20),  -- critical, high, medium, low
    estimated_effort VARCHAR(50),  -- hours, days, weeks
    suggested_controls JSONB,  -- Array of suggested control implementations
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_gap_recommendations_org ON ai_gap_recommendations(organization_id);
CREATE INDEX idx_ai_gap_recommendations_framework ON ai_gap_recommendations(framework_id);

-- AI risk scoring suggestions
CREATE TABLE ai_risk_assessments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    risk_id UUID NOT NULL REFERENCES risks(id) ON DELETE CASCADE,
    completion_id UUID REFERENCES ai_completions(id) ON DELETE SET NULL,
    suggested_likelihood INTEGER CHECK (suggested_likelihood BETWEEN 1 AND 5),
    suggested_impact INTEGER CHECK (suggested_impact BETWEEN 1 AND 5),
    likelihood_rationale TEXT,
    impact_rationale TEXT,
    suggested_treatment TEXT,
    control_recommendations JSONB,  -- Array of control suggestions
    accepted BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_risk_assessments_risk ON ai_risk_assessments(risk_id);

-- AI audit preparation checklists
CREATE TABLE ai_audit_preparations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    audit_id UUID NOT NULL REFERENCES audits(id) ON DELETE CASCADE,
    completion_id UUID REFERENCES ai_completions(id) ON DELETE SET NULL,
    preparation_summary TEXT NOT NULL,
    checklist_items JSONB NOT NULL,  -- Array of preparation items
    evidence_gaps JSONB,  -- Missing evidence identified
    risk_areas JSONB,  -- Areas that need attention
    timeline_suggestions JSONB,  -- Suggested preparation timeline
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ai_audit_preparations_audit ON ai_audit_preparations(audit_id);
