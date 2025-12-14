-- 2.6 Automated Evidence Collection
-- Adds support for scheduled snapshots, auto-linking, change detection, and freshness scoring

-- ==================== Evidence Freshness Fields ====================

-- Add freshness-related fields to evidence table
ALTER TABLE evidence
ADD COLUMN IF NOT EXISTS freshness_score INTEGER DEFAULT 100,
ADD COLUMN IF NOT EXISTS days_stale INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS freshness_sla_days INTEGER,
ADD COLUMN IF NOT EXISTS content_hash VARCHAR(64),
ADD COLUMN IF NOT EXISTS last_verified_at TIMESTAMPTZ;

-- Create index for freshness queries
CREATE INDEX IF NOT EXISTS idx_evidence_freshness ON evidence(organization_id, freshness_score);
CREATE INDEX IF NOT EXISTS idx_evidence_stale ON evidence(organization_id, days_stale) WHERE days_stale > 0;
CREATE INDEX IF NOT EXISTS idx_evidence_valid_until ON evidence(organization_id, valid_until) WHERE valid_until IS NOT NULL;

-- ==================== Evidence Collection Tasks (Schedule) ====================

-- Update evidence_collection_tasks for scheduled collection
ALTER TABLE evidence_collection_tasks
ADD COLUMN IF NOT EXISTS enabled BOOLEAN DEFAULT true,
ADD COLUMN IF NOT EXISTS cron_schedule VARCHAR(100),
ADD COLUMN IF NOT EXISTS timezone VARCHAR(50) DEFAULT 'UTC',
ADD COLUMN IF NOT EXISTS evidence_type VARCHAR(50),
ADD COLUMN IF NOT EXISTS auto_link_controls BOOLEAN DEFAULT true,
ADD COLUMN IF NOT EXISTS retention_days INTEGER DEFAULT 365,
ADD COLUMN IF NOT EXISTS last_error VARCHAR(1000),
ADD COLUMN IF NOT EXISTS error_count INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS success_count INTEGER DEFAULT 0;

-- Create indexes for scheduler polling
CREATE INDEX IF NOT EXISTS idx_evidence_tasks_schedule
ON evidence_collection_tasks(next_run_at, status)
WHERE enabled = true;

CREATE INDEX IF NOT EXISTS idx_evidence_tasks_org_integration
ON evidence_collection_tasks(organization_id, integration_id);

-- ==================== Evidence Changes Tracking ====================

-- Track evidence changes for change detection
CREATE TABLE IF NOT EXISTS evidence_changes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    evidence_id UUID NOT NULL REFERENCES evidence(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    change_type VARCHAR(20) NOT NULL, -- 'created', 'updated', 'content_changed', 'expired', 'renewed'
    previous_hash VARCHAR(64),
    new_hash VARCHAR(64),
    previous_data JSONB,
    new_data JSONB,
    change_summary TEXT,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_evidence_changes_evidence ON evidence_changes(evidence_id);
CREATE INDEX IF NOT EXISTS idx_evidence_changes_org ON evidence_changes(organization_id, detected_at DESC);
CREATE INDEX IF NOT EXISTS idx_evidence_changes_unacknowledged
ON evidence_changes(organization_id, detected_at)
WHERE acknowledged_at IS NULL;

-- ==================== Evidence Collection Runs ====================

-- Track each collection run
CREATE TABLE IF NOT EXISTS evidence_collection_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES evidence_collection_tasks(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL DEFAULT 'running', -- 'running', 'completed', 'failed', 'partial'
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    evidence_created INTEGER DEFAULT 0,
    evidence_updated INTEGER DEFAULT 0,
    evidence_unchanged INTEGER DEFAULT 0,
    changes_detected INTEGER DEFAULT 0,
    controls_linked INTEGER DEFAULT 0,
    error_message TEXT,
    duration_ms INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_collection_runs_task ON evidence_collection_runs(task_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_collection_runs_org ON evidence_collection_runs(organization_id, started_at DESC);

-- ==================== Evidence Control Auto-Mapping Rules ====================

-- Define rules for auto-linking evidence to controls based on patterns
CREATE TABLE IF NOT EXISTS evidence_control_mapping_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    enabled BOOLEAN DEFAULT true,
    priority INTEGER DEFAULT 0, -- Higher = applied first
    -- Matching criteria (all conditions must match)
    source_pattern VARCHAR(255), -- Regex pattern for evidence source (e.g., 'aws|github')
    type_pattern VARCHAR(255), -- Regex pattern for evidence_type
    title_pattern VARCHAR(255), -- Regex pattern for title
    -- Control codes to link to
    control_codes TEXT[] NOT NULL, -- Framework requirement codes to auto-link
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mapping_rules_org ON evidence_control_mapping_rules(organization_id, enabled, priority DESC);

-- ==================== Evidence SLA Definitions ====================

-- Define freshness SLAs by evidence type/source
CREATE TABLE IF NOT EXISTS evidence_freshness_slas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE, -- NULL = system default
    evidence_type VARCHAR(50),
    source VARCHAR(50),
    max_age_days INTEGER NOT NULL, -- Maximum allowed age before considered stale
    warning_days INTEGER NOT NULL, -- Days before max_age to start warning
    critical_days INTEGER NOT NULL, -- Days past max_age to be critical
    auto_expire BOOLEAN DEFAULT false, -- Automatically mark as expired
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, evidence_type, source)
);

-- Insert default SLAs
INSERT INTO evidence_freshness_slas (organization_id, evidence_type, source, max_age_days, warning_days, critical_days) VALUES
(NULL, 'security_finding', NULL, 7, 5, 14),
(NULL, 'config_snapshot', NULL, 1, 1, 3),
(NULL, 'audit_log', NULL, 1, 1, 7),
(NULL, 'user_inventory', NULL, 7, 5, 14),
(NULL, 'access_review', NULL, 90, 80, 120),
(NULL, 'policy_acknowledgment', NULL, 365, 335, 395),
(NULL, 'vendor_assessment', NULL, 365, 335, 395),
(NULL, 'penetration_test', NULL, 365, 335, 395),
(NULL, 'document', NULL, 90, 60, 120),
(NULL, NULL, NULL, 90, 60, 120) -- Default fallback
ON CONFLICT DO NOTHING;

-- ==================== Functions ====================

-- Function to calculate evidence freshness score
CREATE OR REPLACE FUNCTION calculate_evidence_freshness(
    p_collected_at TIMESTAMPTZ,
    p_valid_until TIMESTAMPTZ,
    p_max_age_days INTEGER DEFAULT 90
) RETURNS TABLE(freshness_score INTEGER, days_stale INTEGER) AS $$
DECLARE
    v_age_days INTEGER;
    v_days_until_expiry INTEGER;
    v_score INTEGER;
    v_stale INTEGER;
BEGIN
    -- Calculate age in days
    v_age_days := EXTRACT(DAY FROM NOW() - p_collected_at)::INTEGER;

    -- If we have a valid_until date, use it
    IF p_valid_until IS NOT NULL THEN
        v_days_until_expiry := EXTRACT(DAY FROM p_valid_until - NOW())::INTEGER;

        IF v_days_until_expiry < 0 THEN
            -- Expired
            v_score := 0;
            v_stale := ABS(v_days_until_expiry);
        ELSIF v_days_until_expiry <= 7 THEN
            -- Expiring very soon
            v_score := 25 + (v_days_until_expiry * 5);
        ELSIF v_days_until_expiry <= 30 THEN
            -- Expiring soon
            v_score := 50 + ((v_days_until_expiry - 7) * 2);
        ELSE
            -- Not expiring soon
            v_score := 100 - LEAST(v_age_days, p_max_age_days) * 100 / p_max_age_days;
        END IF;
        v_stale := GREATEST(0, -v_days_until_expiry);
    ELSE
        -- No valid_until, score based on age
        IF v_age_days > p_max_age_days THEN
            v_score := GREATEST(0, 100 - (v_age_days - p_max_age_days) * 10);
            v_stale := v_age_days - p_max_age_days;
        ELSE
            v_score := 100 - (v_age_days * 100 / GREATEST(p_max_age_days, 1));
            v_stale := 0;
        END IF;
    END IF;

    freshness_score := GREATEST(0, LEAST(100, v_score));
    days_stale := GREATEST(0, v_stale);
    RETURN NEXT;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Function to update evidence freshness (call periodically)
CREATE OR REPLACE FUNCTION update_evidence_freshness(p_organization_id UUID DEFAULT NULL)
RETURNS INTEGER AS $$
DECLARE
    v_updated INTEGER;
BEGIN
    WITH sla_lookup AS (
        SELECT DISTINCT ON (e.id)
            e.id,
            COALESCE(s.max_age_days, 90) as max_age_days
        FROM evidence e
        LEFT JOIN evidence_freshness_slas s ON
            (s.organization_id = e.organization_id OR s.organization_id IS NULL)
            AND (s.evidence_type = e.evidence_type OR s.evidence_type IS NULL)
            AND (s.source = e.source OR s.source IS NULL)
        WHERE p_organization_id IS NULL OR e.organization_id = p_organization_id
        ORDER BY e.id, s.organization_id DESC NULLS LAST, s.evidence_type DESC NULLS LAST, s.source DESC NULLS LAST
    ),
    freshness_calc AS (
        SELECT
            sl.id,
            (calculate_evidence_freshness(e.collected_at, e.valid_until, sl.max_age_days)).*
        FROM sla_lookup sl
        JOIN evidence e ON e.id = sl.id
    )
    UPDATE evidence e
    SET
        freshness_score = fc.freshness_score,
        days_stale = fc.days_stale
    FROM freshness_calc fc
    WHERE e.id = fc.id
      AND (e.freshness_score IS DISTINCT FROM fc.freshness_score
           OR e.days_stale IS DISTINCT FROM fc.days_stale);

    GET DIAGNOSTICS v_updated = ROW_COUNT;
    RETURN v_updated;
END;
$$ LANGUAGE plpgsql;

-- Function to get evidence needing refresh by source
CREATE OR REPLACE FUNCTION get_stale_evidence_by_source(
    p_organization_id UUID,
    p_min_staleness INTEGER DEFAULT 0
) RETURNS TABLE(
    source VARCHAR,
    integration_id UUID,
    stale_count BIGINT,
    oldest_days INTEGER,
    avg_freshness NUMERIC
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        e.source::VARCHAR,
        ect.integration_id,
        COUNT(*)::BIGINT as stale_count,
        MAX(e.days_stale)::INTEGER as oldest_days,
        AVG(e.freshness_score)::NUMERIC as avg_freshness
    FROM evidence e
    LEFT JOIN evidence_collection_tasks ect ON
        ect.organization_id = e.organization_id
        AND ect.collection_config->>'source' = e.source
    WHERE e.organization_id = p_organization_id
      AND e.days_stale >= p_min_staleness
    GROUP BY e.source, ect.integration_id
    ORDER BY stale_count DESC;
END;
$$ LANGUAGE plpgsql;

-- ==================== Triggers ====================

-- Trigger to update freshness_sla_days when evidence is created/updated
CREATE OR REPLACE FUNCTION set_evidence_freshness_sla()
RETURNS TRIGGER AS $$
BEGIN
    SELECT max_age_days INTO NEW.freshness_sla_days
    FROM evidence_freshness_slas
    WHERE (organization_id = NEW.organization_id OR organization_id IS NULL)
      AND (evidence_type = NEW.evidence_type OR evidence_type IS NULL)
      AND (source = NEW.source OR source IS NULL)
    ORDER BY organization_id DESC NULLS LAST, evidence_type DESC NULLS LAST, source DESC NULLS LAST
    LIMIT 1;

    IF NEW.freshness_sla_days IS NULL THEN
        NEW.freshness_sla_days := 90; -- Default
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER trg_evidence_set_sla
BEFORE INSERT OR UPDATE OF evidence_type, source ON evidence
FOR EACH ROW
EXECUTE FUNCTION set_evidence_freshness_sla();

-- Trigger to track evidence changes
CREATE OR REPLACE FUNCTION track_evidence_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'UPDATE' THEN
        -- Check if content changed (using hash or comparing key fields)
        IF OLD.content_hash IS DISTINCT FROM NEW.content_hash
           OR OLD.title IS DISTINCT FROM NEW.title
           OR OLD.description IS DISTINCT FROM NEW.description THEN
            INSERT INTO evidence_changes (
                evidence_id, organization_id, change_type,
                previous_hash, new_hash,
                previous_data, new_data,
                change_summary
            ) VALUES (
                NEW.id, NEW.organization_id, 'content_changed',
                OLD.content_hash, NEW.content_hash,
                jsonb_build_object('title', OLD.title, 'description', OLD.description),
                jsonb_build_object('title', NEW.title, 'description', NEW.description),
                CASE
                    WHEN OLD.title IS DISTINCT FROM NEW.title THEN 'Title updated'
                    WHEN OLD.description IS DISTINCT FROM NEW.description THEN 'Description updated'
                    ELSE 'Content hash changed'
                END
            );
        END IF;

        -- Check if expired
        IF OLD.valid_until > NOW() AND NEW.valid_until <= NOW() THEN
            INSERT INTO evidence_changes (
                evidence_id, organization_id, change_type, change_summary
            ) VALUES (
                NEW.id, NEW.organization_id, 'expired', 'Evidence has expired'
            );
        END IF;

        -- Check if renewed
        IF OLD.valid_until <= NOW() AND NEW.valid_until > NOW() THEN
            INSERT INTO evidence_changes (
                evidence_id, organization_id, change_type, change_summary
            ) VALUES (
                NEW.id, NEW.organization_id, 'renewed', 'Evidence validity extended'
            );
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE TRIGGER trg_evidence_track_changes
AFTER UPDATE ON evidence
FOR EACH ROW
EXECUTE FUNCTION track_evidence_changes();

-- ==================== Views ====================

-- View for evidence freshness dashboard
CREATE OR REPLACE VIEW v_evidence_freshness_summary AS
SELECT
    e.organization_id,
    COUNT(*) as total_evidence,
    COUNT(*) FILTER (WHERE e.freshness_score >= 80) as fresh_count,
    COUNT(*) FILTER (WHERE e.freshness_score >= 50 AND e.freshness_score < 80) as aging_count,
    COUNT(*) FILTER (WHERE e.freshness_score >= 20 AND e.freshness_score < 50) as stale_count,
    COUNT(*) FILTER (WHERE e.freshness_score < 20) as critical_count,
    COUNT(*) FILTER (WHERE e.valid_until < NOW()) as expired_count,
    COUNT(*) FILTER (WHERE e.valid_until <= NOW() + INTERVAL '30 days' AND e.valid_until > NOW()) as expiring_soon_count,
    AVG(e.freshness_score)::INTEGER as avg_freshness_score,
    MAX(e.days_stale) as max_days_stale
FROM evidence e
GROUP BY e.organization_id;

-- View for evidence collection task status
CREATE OR REPLACE VIEW v_evidence_collection_status AS
SELECT
    t.id as task_id,
    t.organization_id,
    t.integration_id,
    i.name as integration_name,
    i.integration_type,
    t.name as task_name,
    t.enabled,
    t.cron_schedule,
    t.last_run_at,
    t.next_run_at,
    t.status,
    t.success_count,
    t.error_count,
    t.last_error,
    r.evidence_created as last_run_created,
    r.evidence_updated as last_run_updated,
    r.changes_detected as last_run_changes,
    r.duration_ms as last_run_duration_ms
FROM evidence_collection_tasks t
JOIN integrations i ON t.integration_id = i.id
LEFT JOIN LATERAL (
    SELECT * FROM evidence_collection_runs
    WHERE task_id = t.id
    ORDER BY started_at DESC
    LIMIT 1
) r ON true;

-- View for pending evidence changes (unacknowledged)
CREATE OR REPLACE VIEW v_pending_evidence_changes AS
SELECT
    c.id as change_id,
    c.organization_id,
    c.evidence_id,
    e.title as evidence_title,
    e.source,
    e.evidence_type,
    c.change_type,
    c.change_summary,
    c.detected_at,
    EXTRACT(EPOCH FROM (NOW() - c.detected_at)) / 3600 as hours_pending
FROM evidence_changes c
JOIN evidence e ON c.evidence_id = e.id
WHERE c.acknowledged_at IS NULL
ORDER BY c.detected_at DESC;
