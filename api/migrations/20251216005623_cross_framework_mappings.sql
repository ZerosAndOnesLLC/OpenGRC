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

-- ===========================================
-- SEED CROSS-FRAMEWORK MAPPINGS
-- ===========================================
-- Framework IDs:
-- SOC 2:     a0000000-0000-0000-0000-000000000001
-- ISO 27001: b0000000-0000-0000-0000-000000000001
-- HIPAA:     c0000000-0000-0000-0000-000000000001
-- PCI DSS:   d0000000-0000-0000-0000-000000000001
-- GDPR:      e0000000-0000-0000-0000-000000000001
-- NIST CSF:  f0000000-0000-0000-0000-000000000001
-- SOX ITGC:  g0000000-0000-0000-0000-000000000001

-- ===========================================
-- ACCESS CONTROL MAPPINGS
-- ===========================================

-- SOC 2 CC6.1 (Logical Access Security) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- SOC 2 CC6.1 -> ISO 27001 A.9 (Access Control)
('a1060000-0000-0000-0001-000000000001', 'b1000000-0009-0000-0000-000000000001', 'equivalent', 0.95, 'Both address logical access security controls'),
-- SOC 2 CC6.1 -> HIPAA 164.312(a)(1) (Access Control)
('a1060000-0000-0000-0001-000000000001', 'c1000000-0003-0001-0000-000000000001', 'equivalent', 0.90, 'Both require unique user identification and access controls'),
-- SOC 2 CC6.1 -> PCI DSS Req 7 (Restrict Access)
('a1060000-0000-0000-0001-000000000001', 'd1000000-0000-0007-0000-000000000001', 'equivalent', 0.95, 'Both restrict access to cardholder data/sensitive information'),
-- SOC 2 CC6.1 -> NIST CSF PR.AA (Identity Management and Access Control)
('a1060000-0000-0000-0001-000000000001', 'f3000000-0000-0000-0001-000000000001', 'equivalent', 0.95, 'Both address access control to physical and logical assets'),
-- SOC 2 CC6.1 -> SOX ITGC AC (Access Controls)
('a1060000-0000-0000-0001-000000000001', 'g1000000-0000-0000-0001-000000000001', 'equivalent', 0.90, 'Both address logical access controls');

-- ===========================================
-- RISK MANAGEMENT MAPPINGS
-- ===========================================

-- SOC 2 CC3.1 (Risk Assessment) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- SOC 2 CC3.1 -> ISO 27001 Clause 6 (Planning/Risk)
('a1030000-0000-0000-0001-000000000001', 'b1000000-0000-0006-0000-000000000001', 'equivalent', 0.90, 'Both address risk assessment and treatment'),
-- SOC 2 CC3.1 -> HIPAA 164.308(a)(1)(ii)(A) (Risk Analysis)
('a1030000-0000-0000-0001-000000000001', 'c1000000-0001-0001-0001-000000000001', 'equivalent', 0.90, 'Both require risk analysis'),
-- SOC 2 CC3.1 -> NIST CSF ID.RA (Risk Assessment)
('a1030000-0000-0000-0001-000000000001', 'f2000000-0000-0000-0001-000000000002', 'equivalent', 0.95, 'Both address cybersecurity risk assessment'),
-- SOC 2 CC3.1 -> GDPR Art 32 (Security of Processing)
('a1030000-0000-0000-0001-000000000001', 'e1000000-0000-0000-0000-000000000032', 'partial', 0.75, 'GDPR requires risk-based security measures');

-- ===========================================
-- INCIDENT RESPONSE MAPPINGS
-- ===========================================

-- SOC 2 CC7.4 (Incident Response) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- SOC 2 CC7.4 -> ISO 27001 A.16 (Incident Management)
('a1070000-0000-0000-0001-000000000004', 'b1000000-0016-0000-0000-000000000001', 'equivalent', 0.95, 'Both address incident management'),
-- SOC 2 CC7.4 -> HIPAA 164.308(a)(6) (Security Incident Procedures)
('a1070000-0000-0000-0001-000000000004', 'c1000000-0001-0006-0000-000000000001', 'equivalent', 0.90, 'Both require incident response procedures'),
-- SOC 2 CC7.4 -> PCI DSS Req 12.10 (Incident Response Plan)
('a1070000-0000-0000-0001-000000000004', 'd1000000-0000-0012-0010-000000000001', 'equivalent', 0.90, 'Both require incident response plans'),
-- SOC 2 CC7.4 -> NIST CSF RS (Respond)
('a1070000-0000-0000-0001-000000000004', 'f5000000-0000-0000-0000-000000000001', 'equivalent', 0.95, 'Both address incident response'),
-- SOC 2 CC7.4 -> GDPR Art 33 (Breach Notification)
('a1070000-0000-0000-0001-000000000004', 'e1000000-0000-0000-0000-000000000033', 'partial', 0.80, 'GDPR focuses on breach notification; SOC 2 broader incident response');

-- ===========================================
-- CHANGE MANAGEMENT MAPPINGS
-- ===========================================

-- SOC 2 CC8.1 (Change Management) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- SOC 2 CC8.1 -> ISO 27001 A.12.1.2 (Change Management)
('a1080000-0000-0000-0001-000000000001', 'b1000000-0012-0001-0002-000000000001', 'equivalent', 0.95, 'Both address change management controls'),
-- SOC 2 CC8.1 -> PCI DSS Req 6.4 (Change Control)
('a1080000-0000-0000-0001-000000000001', 'd1000000-0000-0006-0004-000000000001', 'equivalent', 0.90, 'Both require change control procedures'),
-- SOC 2 CC8.1 -> SOX ITGC CM (Change Management)
('a1080000-0000-0000-0001-000000000001', 'g1000000-0000-0000-0002-000000000001', 'equivalent', 0.95, 'Both address change management for IT systems');

-- ===========================================
-- DATA PROTECTION/ENCRYPTION MAPPINGS
-- ===========================================

-- SOC 2 CC6.7 (Encryption) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- SOC 2 CC6.7 -> ISO 27001 A.10 (Cryptography)
('a1060000-0000-0000-0001-000000000007', 'b1000000-0010-0000-0000-000000000001', 'equivalent', 0.95, 'Both address cryptographic controls'),
-- SOC 2 CC6.7 -> HIPAA 164.312(a)(2)(iv) (Encryption)
('a1060000-0000-0000-0001-000000000007', 'c1000000-0003-0001-0002-000000000004', 'equivalent', 0.90, 'Both address encryption of data'),
-- SOC 2 CC6.7 -> PCI DSS Req 3 (Protect Stored Data)
('a1060000-0000-0000-0001-000000000007', 'd1000000-0000-0003-0000-000000000001', 'equivalent', 0.90, 'Both address protection of stored data'),
-- SOC 2 CC6.7 -> PCI DSS Req 4 (Encrypt Transmission)
('a1060000-0000-0000-0001-000000000007', 'd1000000-0000-0004-0000-000000000001', 'equivalent', 0.90, 'Both address encryption in transit'),
-- SOC 2 CC6.7 -> NIST CSF PR.DS (Data Security)
('a1060000-0000-0000-0001-000000000007', 'f3000000-0000-0000-0001-000000000003', 'equivalent', 0.90, 'Both address data security and encryption');

-- ===========================================
-- AWARENESS AND TRAINING MAPPINGS
-- ===========================================

-- SOC 2 CC1.4 (Security Awareness) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- SOC 2 CC1.4 -> ISO 27001 A.7.2.2 (Security Awareness)
('a1010000-0000-0000-0001-000000000004', 'b1000000-0007-0002-0002-000000000001', 'equivalent', 0.95, 'Both address security awareness training'),
-- SOC 2 CC1.4 -> HIPAA 164.308(a)(5) (Security Awareness Training)
('a1010000-0000-0000-0001-000000000004', 'c1000000-0001-0005-0000-000000000001', 'equivalent', 0.90, 'Both require security awareness training'),
-- SOC 2 CC1.4 -> PCI DSS Req 12.6 (Security Awareness Program)
('a1010000-0000-0000-0001-000000000004', 'd1000000-0000-0012-0006-000000000001', 'equivalent', 0.90, 'Both require security awareness programs'),
-- SOC 2 CC1.4 -> NIST CSF PR.AT (Awareness and Training)
('a1010000-0000-0000-0001-000000000004', 'f3000000-0000-0000-0001-000000000002', 'equivalent', 0.95, 'Both address awareness and training');

-- ===========================================
-- MONITORING AND LOGGING MAPPINGS
-- ===========================================

-- SOC 2 CC7.2 (Monitoring) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- SOC 2 CC7.2 -> ISO 27001 A.12.4 (Logging and Monitoring)
('a1070000-0000-0000-0001-000000000002', 'b1000000-0012-0004-0000-000000000001', 'equivalent', 0.95, 'Both address logging and monitoring'),
-- SOC 2 CC7.2 -> HIPAA 164.312(b) (Audit Controls)
('a1070000-0000-0000-0001-000000000002', 'c1000000-0003-0002-0000-000000000001', 'equivalent', 0.90, 'Both require audit controls and logging'),
-- SOC 2 CC7.2 -> PCI DSS Req 10 (Track and Monitor Access)
('a1070000-0000-0000-0001-000000000002', 'd1000000-0000-0010-0000-000000000001', 'equivalent', 0.95, 'Both require tracking and monitoring access'),
-- SOC 2 CC7.2 -> NIST CSF DE.CM (Continuous Monitoring)
('a1070000-0000-0000-0001-000000000002', 'f4000000-0000-0000-0001-000000000001', 'equivalent', 0.95, 'Both address continuous monitoring'),
-- SOC 2 CC7.2 -> SOX ITGC CO.2 (System and Security Monitoring)
('a1070000-0000-0000-0001-000000000002', 'g1000000-0000-0004-0002-000000000001', 'equivalent', 0.90, 'Both address system monitoring');

-- ===========================================
-- VENDOR MANAGEMENT MAPPINGS
-- ===========================================

-- SOC 2 CC9.2 (Vendor Management) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- SOC 2 CC9.2 -> ISO 27001 A.15 (Supplier Relationships)
('a1090000-0000-0000-0001-000000000002', 'b1000000-0015-0000-0000-000000000001', 'equivalent', 0.95, 'Both address supplier/vendor management'),
-- SOC 2 CC9.2 -> HIPAA 164.308(b) (Business Associate Contracts)
('a1090000-0000-0000-0001-000000000002', 'c1000000-0001-0009-0000-000000000001', 'equivalent', 0.85, 'Both address third-party oversight'),
-- SOC 2 CC9.2 -> PCI DSS Req 12.8 (Service Provider Management)
('a1090000-0000-0000-0001-000000000002', 'd1000000-0000-0012-0008-000000000001', 'equivalent', 0.90, 'Both require service provider management'),
-- SOC 2 CC9.2 -> NIST CSF GV.SC (Supply Chain Risk Management)
('a1090000-0000-0000-0001-000000000002', 'f1000000-0000-0000-0001-000000000006', 'equivalent', 0.90, 'Both address supply chain risk management');

-- ===========================================
-- BUSINESS CONTINUITY MAPPINGS
-- ===========================================

-- SOC 2 A1.2 (Recovery) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- SOC 2 A1.2 -> ISO 27001 A.17 (Business Continuity)
('a2010000-0000-0000-0001-000000000002', 'b1000000-0017-0000-0000-000000000001', 'equivalent', 0.95, 'Both address business continuity'),
-- SOC 2 A1.2 -> HIPAA 164.308(a)(7) (Contingency Plan)
('a2010000-0000-0000-0001-000000000002', 'c1000000-0001-0007-0000-000000000001', 'equivalent', 0.90, 'Both require contingency planning'),
-- SOC 2 A1.2 -> NIST CSF RC (Recover)
('a2010000-0000-0000-0001-000000000002', 'f6000000-0000-0000-0000-000000000001', 'equivalent', 0.95, 'Both address recovery capabilities');

-- ===========================================
-- PHYSICAL SECURITY MAPPINGS
-- ===========================================

-- SOC 2 CC6.4 (Physical Access) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- SOC 2 CC6.4 -> ISO 27001 A.11 (Physical Security)
('a1060000-0000-0000-0001-000000000004', 'b1000000-0011-0000-0000-000000000001', 'equivalent', 0.95, 'Both address physical security'),
-- SOC 2 CC6.4 -> HIPAA 164.310 (Physical Safeguards)
('a1060000-0000-0000-0001-000000000004', 'c1000000-0002-0000-0000-000000000001', 'equivalent', 0.90, 'Both address physical access controls'),
-- SOC 2 CC6.4 -> PCI DSS Req 9 (Physical Access)
('a1060000-0000-0000-0001-000000000004', 'd1000000-0000-0009-0000-000000000001', 'equivalent', 0.95, 'Both restrict physical access');

-- ===========================================
-- DATA PRIVACY MAPPINGS (GDPR-centric)
-- ===========================================

-- GDPR Art 5 (Principles) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- GDPR Art 5 -> ISO 27001 A.18.1.4 (Privacy)
('e1000000-0000-0000-0000-000000000005', 'b1000000-0018-0001-0004-000000000001', 'partial', 0.80, 'ISO covers privacy but not as comprehensively as GDPR'),
-- GDPR Art 5 -> HIPAA Privacy Rule concepts
('e1000000-0000-0000-0000-000000000005', 'c1000000-0004-0000-0000-000000000001', 'partial', 0.75, 'Both address privacy principles but for different data types');

-- GDPR Art 17 (Right to Erasure) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- GDPR Art 17 -> PCI DSS Req 3.1 (Data Retention)
('e1000000-0000-0000-0000-000000000017', 'd1000000-0000-0003-0001-000000000001', 'partial', 0.70, 'Both address data deletion but with different triggers');

-- ===========================================
-- ISO 27001 to other frameworks
-- ===========================================

-- ISO 27001 A.8 (Asset Management) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- ISO A.8 -> NIST CSF ID.AM (Asset Management)
('b1000000-0008-0000-0000-000000000001', 'f2000000-0000-0000-0001-000000000001', 'equivalent', 0.95, 'Both address asset management'),
-- ISO A.8 -> HIPAA 164.310(d) (Device and Media Controls)
('b1000000-0008-0000-0000-000000000001', 'c1000000-0002-0004-0000-000000000001', 'partial', 0.80, 'HIPAA focuses on media controls, ISO broader');

-- ===========================================
-- NIST CSF Govern mappings
-- ===========================================

-- NIST CSF GV.PO (Policy) mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- NIST GV.PO -> ISO 27001 A.5 (Policies)
('f1000000-0000-0000-0001-000000000004', 'b1000000-0005-0000-0000-000000000001', 'equivalent', 0.95, 'Both address security policy'),
-- NIST GV.PO -> HIPAA 164.316 (Policies and Documentation)
('f1000000-0000-0000-0001-000000000004', 'c1000000-0005-0000-0000-000000000001', 'equivalent', 0.90, 'Both require documented policies'),
-- NIST GV.PO -> PCI DSS Req 12 (Information Security Policy)
('f1000000-0000-0000-0001-000000000004', 'd1000000-0000-0012-0000-000000000001', 'equivalent', 0.90, 'Both require security policies'),
-- NIST GV.PO -> SOX ITGC ELC (Entity Level Controls)
('f1000000-0000-0000-0001-000000000004', 'g1000000-0000-0000-0005-000000000001', 'partial', 0.80, 'SOX focuses on IT governance policies');

-- ===========================================
-- SOX ITGC specific mappings
-- ===========================================

-- SOX ITGC Program Development mappings
INSERT INTO cross_framework_mappings (source_requirement_id, target_requirement_id, mapping_type, confidence_score, notes) VALUES
-- SOX PD -> ISO 27001 A.14 (System Acquisition)
('g1000000-0000-0000-0003-000000000001', 'b1000000-0014-0000-0000-000000000001', 'equivalent', 0.90, 'Both address system development lifecycle'),
-- SOX PD -> PCI DSS Req 6 (Develop Secure Systems)
('g1000000-0000-0000-0003-000000000001', 'd1000000-0000-0006-0000-000000000001', 'equivalent', 0.85, 'Both address secure development');
