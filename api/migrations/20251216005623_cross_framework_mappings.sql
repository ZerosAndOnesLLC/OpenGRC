-- Cross-Framework Requirement Mappings
-- Enables "test once, satisfy many" functionality
-- Maps equivalent requirements across different compliance frameworks

-- Cross-framework mapping table
CREATE TABLE cross_framework_mappings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_requirement_id UUID NOT NULL REFERENCES framework_requirements(id) ON DELETE CASCADE,
    target_requirement_id UUID NOT NULL REFERENCES framework_requirements(id) ON DELETE CASCADE,
    mapping_type VARCHAR(50) NOT NULL DEFAULT 'equivalent',  -- equivalent, partial, related
    confidence_score DECIMAL(3,2) DEFAULT 1.00,  -- 0.00 to 1.00
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(source_requirement_id, target_requirement_id),
    CHECK (source_requirement_id != target_requirement_id)
);

-- Index for efficient lookups in both directions
CREATE INDEX idx_cross_framework_source ON cross_framework_mappings(source_requirement_id);
CREATE INDEX idx_cross_framework_target ON cross_framework_mappings(target_requirement_id);
CREATE INDEX idx_cross_framework_type ON cross_framework_mappings(mapping_type);

-- View to get all related requirements for a given requirement (bidirectional)
CREATE OR REPLACE VIEW requirement_cross_mappings AS
SELECT
    cfm.id,
    cfm.source_requirement_id,
    cfm.target_requirement_id,
    cfm.mapping_type,
    cfm.confidence_score,
    sr.code AS source_code,
    sr.name AS source_name,
    sf.name AS source_framework,
    tr.code AS target_code,
    tr.name AS target_name,
    tf.name AS target_framework
FROM cross_framework_mappings cfm
JOIN framework_requirements sr ON cfm.source_requirement_id = sr.id
JOIN framework_requirements tr ON cfm.target_requirement_id = tr.id
JOIN frameworks sf ON sr.framework_id = sf.id
JOIN frameworks tf ON tr.framework_id = tf.id;

