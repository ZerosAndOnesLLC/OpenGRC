-- SOC 2 Trust Service Criteria (2017)
-- This is the official AICPA Trust Service Criteria framework

-- Insert SOC 2 Framework
INSERT INTO frameworks (id, name, version, description, category, is_system, created_at)
VALUES (
    'a0000000-0000-0000-0000-000000000001',
    'SOC 2',
    '2017',
    'SOC 2 Trust Service Criteria established by the AICPA. Includes Security (required), Availability, Processing Integrity, Confidentiality, and Privacy.',
    'compliance',
    true,
    NOW()
);

-- ===========================================
-- SECURITY (CC) - Common Criteria (Required)
-- ===========================================

-- CC1: Control Environment
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000001', 'CC1', 'Control Environment', 'The criteria relevant to how the entity demonstrates commitment to integrity and ethical values.', 'Security', NULL, 1);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0001-000000000001', 'a0000000-0000-0000-0000-000000000001', 'CC1.1', 'Commitment to Integrity and Ethical Values', 'The entity demonstrates a commitment to integrity and ethical values.', 'Security', 'a1000000-0000-0000-0000-000000000001', 1),
('a1000000-0000-0000-0001-000000000002', 'a0000000-0000-0000-0000-000000000001', 'CC1.2', 'Board Independence and Oversight', 'The board of directors demonstrates independence from management and exercises oversight of the development and performance of internal control.', 'Security', 'a1000000-0000-0000-0000-000000000001', 2),
('a1000000-0000-0000-0001-000000000003', 'a0000000-0000-0000-0000-000000000001', 'CC1.3', 'Management Structure and Authority', 'Management establishes, with board oversight, structures, reporting lines, and appropriate authorities and responsibilities in the pursuit of objectives.', 'Security', 'a1000000-0000-0000-0000-000000000001', 3),
('a1000000-0000-0000-0001-000000000004', 'a0000000-0000-0000-0000-000000000001', 'CC1.4', 'Commitment to Competence', 'The entity demonstrates a commitment to attract, develop, and retain competent individuals in alignment with objectives.', 'Security', 'a1000000-0000-0000-0000-000000000001', 4),
('a1000000-0000-0000-0001-000000000005', 'a0000000-0000-0000-0000-000000000001', 'CC1.5', 'Accountability for Internal Control', 'The entity holds individuals accountable for their internal control responsibilities in the pursuit of objectives.', 'Security', 'a1000000-0000-0000-0000-000000000001', 5);

-- CC2: Communication and Information
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0000-000000000002', 'a0000000-0000-0000-0000-000000000001', 'CC2', 'Communication and Information', 'The criteria relevant to how the entity uses information to support internal control functioning.', 'Security', NULL, 2);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0002-000000000001', 'a0000000-0000-0000-0000-000000000001', 'CC2.1', 'Quality Information', 'The entity obtains or generates and uses relevant, quality information to support the functioning of internal control.', 'Security', 'a1000000-0000-0000-0000-000000000002', 1),
('a1000000-0000-0000-0002-000000000002', 'a0000000-0000-0000-0000-000000000001', 'CC2.2', 'Internal Communication', 'The entity internally communicates information, including objectives and responsibilities for internal control, necessary to support the functioning of internal control.', 'Security', 'a1000000-0000-0000-0000-000000000002', 2),
('a1000000-0000-0000-0002-000000000003', 'a0000000-0000-0000-0000-000000000001', 'CC2.3', 'External Communication', 'The entity communicates with external parties regarding matters affecting the functioning of internal control.', 'Security', 'a1000000-0000-0000-0000-000000000002', 3);

-- CC3: Risk Assessment
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0000-000000000003', 'a0000000-0000-0000-0000-000000000001', 'CC3', 'Risk Assessment', 'The criteria relevant to how the entity identifies risks and assesses risks affecting achievement of objectives.', 'Security', NULL, 3);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0003-000000000001', 'a0000000-0000-0000-0000-000000000001', 'CC3.1', 'Specified Objectives', 'The entity specifies objectives with sufficient clarity to enable the identification and assessment of risks relating to objectives.', 'Security', 'a1000000-0000-0000-0000-000000000003', 1),
('a1000000-0000-0000-0003-000000000002', 'a0000000-0000-0000-0000-000000000001', 'CC3.2', 'Risk Identification and Analysis', 'The entity identifies risks to the achievement of its objectives across the entity and analyzes risks as a basis for determining how the risks should be managed.', 'Security', 'a1000000-0000-0000-0000-000000000003', 2),
('a1000000-0000-0000-0003-000000000003', 'a0000000-0000-0000-0000-000000000001', 'CC3.3', 'Fraud Risk Assessment', 'The entity considers the potential for fraud in assessing risks to the achievement of objectives.', 'Security', 'a1000000-0000-0000-0000-000000000003', 3),
('a1000000-0000-0000-0003-000000000004', 'a0000000-0000-0000-0000-000000000001', 'CC3.4', 'Significant Change Identification', 'The entity identifies and assesses changes that could significantly impact the system of internal control.', 'Security', 'a1000000-0000-0000-0000-000000000003', 4);

-- CC4: Monitoring Activities
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0000-000000000004', 'a0000000-0000-0000-0000-000000000001', 'CC4', 'Monitoring Activities', 'The criteria relevant to how the entity monitors components of internal control.', 'Security', NULL, 4);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0004-000000000001', 'a0000000-0000-0000-0000-000000000001', 'CC4.1', 'Ongoing and Separate Evaluations', 'The entity selects, develops, and performs ongoing and/or separate evaluations to ascertain whether the components of internal control are present and functioning.', 'Security', 'a1000000-0000-0000-0000-000000000004', 1),
('a1000000-0000-0000-0004-000000000002', 'a0000000-0000-0000-0000-000000000001', 'CC4.2', 'Internal Control Deficiencies', 'The entity evaluates and communicates internal control deficiencies in a timely manner to those parties responsible for taking corrective action, including senior management and the board of directors, as appropriate.', 'Security', 'a1000000-0000-0000-0000-000000000004', 2);

-- CC5: Control Activities
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0000-000000000005', 'a0000000-0000-0000-0000-000000000001', 'CC5', 'Control Activities', 'The criteria relevant to how the entity selects and develops control activities.', 'Security', NULL, 5);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0005-000000000001', 'a0000000-0000-0000-0000-000000000001', 'CC5.1', 'Selection and Development of Control Activities', 'The entity selects and develops control activities that contribute to the mitigation of risks to the achievement of objectives to acceptable levels.', 'Security', 'a1000000-0000-0000-0000-000000000005', 1),
('a1000000-0000-0000-0005-000000000002', 'a0000000-0000-0000-0000-000000000001', 'CC5.2', 'Technology General Controls', 'The entity also selects and develops general control activities over technology to support the achievement of objectives.', 'Security', 'a1000000-0000-0000-0000-000000000005', 2),
('a1000000-0000-0000-0005-000000000003', 'a0000000-0000-0000-0000-000000000001', 'CC5.3', 'Policies and Procedures', 'The entity deploys control activities through policies that establish what is expected and procedures that put policies into action.', 'Security', 'a1000000-0000-0000-0000-000000000005', 3);

-- CC6: Logical and Physical Access Controls
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0000-000000000006', 'a0000000-0000-0000-0000-000000000001', 'CC6', 'Logical and Physical Access Controls', 'The criteria relevant to how the entity implements logical and physical access controls.', 'Security', NULL, 6);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0006-000000000001', 'a0000000-0000-0000-0000-000000000001', 'CC6.1', 'Logical Access Security Software', 'The entity implements logical access security software, infrastructure, and architectures over protected information assets to protect them from security events to meet the entity''s objectives.', 'Security', 'a1000000-0000-0000-0000-000000000006', 1),
('a1000000-0000-0000-0006-000000000002', 'a0000000-0000-0000-0000-000000000001', 'CC6.2', 'New User Registration and Authorization', 'Prior to issuing system credentials and granting system access, the entity registers and authorizes new internal and external users whose access is administered by the entity.', 'Security', 'a1000000-0000-0000-0000-000000000006', 2),
('a1000000-0000-0000-0006-000000000003', 'a0000000-0000-0000-0000-000000000001', 'CC6.3', 'Access Removal', 'The entity removes access to protected information assets when an entity''s system account is no longer needed or upon termination.', 'Security', 'a1000000-0000-0000-0000-000000000006', 3),
('a1000000-0000-0000-0006-000000000004', 'a0000000-0000-0000-0000-000000000001', 'CC6.4', 'Access Review', 'The entity restricts physical access to facilities and protected information assets to authorized personnel to meet the entity''s objectives.', 'Security', 'a1000000-0000-0000-0000-000000000006', 4),
('a1000000-0000-0000-0006-000000000005', 'a0000000-0000-0000-0000-000000000001', 'CC6.5', 'Logical Access Security Measures', 'The entity discontinues logical and physical protections over physical assets only after the ability to read or recover data and software from those assets has been diminished and is no longer required to meet the entity''s objectives.', 'Security', 'a1000000-0000-0000-0000-000000000006', 5),
('a1000000-0000-0000-0006-000000000006', 'a0000000-0000-0000-0000-000000000001', 'CC6.6', 'External Threats', 'The entity implements logical access security measures to protect against threats from sources outside its system boundaries.', 'Security', 'a1000000-0000-0000-0000-000000000006', 6),
('a1000000-0000-0000-0006-000000000007', 'a0000000-0000-0000-0000-000000000001', 'CC6.7', 'Data Transmission', 'The entity restricts the transmission, movement, and removal of information to authorized internal and external users and processes, and protects it during transmission, movement, or removal to meet the entity''s objectives.', 'Security', 'a1000000-0000-0000-0000-000000000006', 7),
('a1000000-0000-0000-0006-000000000008', 'a0000000-0000-0000-0000-000000000001', 'CC6.8', 'Malware Prevention', 'The entity implements controls to prevent or detect and act upon the introduction of unauthorized or malicious software to meet the entity''s objectives.', 'Security', 'a1000000-0000-0000-0000-000000000006', 8);

-- CC7: System Operations
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0000-000000000007', 'a0000000-0000-0000-0000-000000000001', 'CC7', 'System Operations', 'The criteria relevant to how the entity manages system operations.', 'Security', NULL, 7);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0007-000000000001', 'a0000000-0000-0000-0000-000000000001', 'CC7.1', 'Security Event Detection', 'To meet its objectives, the entity uses detection and monitoring procedures to identify (1) changes to configurations that result in the introduction of new vulnerabilities, and (2) susceptibilities to newly discovered vulnerabilities.', 'Security', 'a1000000-0000-0000-0000-000000000007', 1),
('a1000000-0000-0000-0007-000000000002', 'a0000000-0000-0000-0000-000000000001', 'CC7.2', 'Anomaly Monitoring', 'The entity monitors system components and the operation of those components for anomalies that are indicative of malicious acts, natural disasters, and errors affecting the entity''s ability to meet its objectives; anomalies are analyzed to determine whether they represent security events.', 'Security', 'a1000000-0000-0000-0000-000000000007', 2),
('a1000000-0000-0000-0007-000000000003', 'a0000000-0000-0000-0000-000000000001', 'CC7.3', 'Security Event Evaluation', 'The entity evaluates security events to determine whether they could or have resulted in a failure of the entity to meet its objectives (security incidents) and, if so, takes actions to prevent or address such failures.', 'Security', 'a1000000-0000-0000-0000-000000000007', 3),
('a1000000-0000-0000-0007-000000000004', 'a0000000-0000-0000-0000-000000000001', 'CC7.4', 'Security Incident Response', 'The entity responds to identified security incidents by executing a defined incident response program to understand, contain, remediate, and communicate security incidents, as appropriate.', 'Security', 'a1000000-0000-0000-0000-000000000007', 4),
('a1000000-0000-0000-0007-000000000005', 'a0000000-0000-0000-0000-000000000001', 'CC7.5', 'Recovery from Security Incidents', 'The entity identifies, develops, and implements activities to recover from identified security incidents.', 'Security', 'a1000000-0000-0000-0000-000000000007', 5);

-- CC8: Change Management
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0000-000000000008', 'a0000000-0000-0000-0000-000000000001', 'CC8', 'Change Management', 'The criteria relevant to how the entity manages changes to system components.', 'Security', NULL, 8);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0008-000000000001', 'a0000000-0000-0000-0000-000000000001', 'CC8.1', 'Change Authorization and Approval', 'The entity authorizes, designs, develops or acquires, configures, documents, tests, approves, and implements changes to infrastructure, data, software, and procedures to meet its objectives.', 'Security', 'a1000000-0000-0000-0000-000000000008', 1);

-- CC9: Risk Mitigation
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0000-000000000009', 'a0000000-0000-0000-0000-000000000001', 'CC9', 'Risk Mitigation', 'The criteria relevant to how the entity identifies and mitigates risks.', 'Security', NULL, 9);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a1000000-0000-0000-0009-000000000001', 'a0000000-0000-0000-0000-000000000001', 'CC9.1', 'Business Disruption Mitigation', 'The entity identifies, selects, and develops risk mitigation activities for risks arising from potential business disruptions.', 'Security', 'a1000000-0000-0000-0000-000000000009', 1),
('a1000000-0000-0000-0009-000000000002', 'a0000000-0000-0000-0000-000000000001', 'CC9.2', 'Vendor Risk Management', 'The entity assesses and manages risks associated with vendors and business partners.', 'Security', 'a1000000-0000-0000-0000-000000000009', 2);

-- ===========================================
-- AVAILABILITY (A)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a2000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000001', 'A', 'Availability', 'The criteria relevant to system availability for operation and use as committed or agreed.', 'Availability', NULL, 10);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a2000000-0000-0000-0001-000000000001', 'a0000000-0000-0000-0000-000000000001', 'A1.1', 'Availability Commitment and Requirements', 'The entity maintains, monitors, and evaluates current processing capacity and use of system components (infrastructure, data, and software) to manage capacity demand and to enable the implementation of additional capacity to help meet its objectives.', 'Availability', 'a2000000-0000-0000-0000-000000000001', 1),
('a2000000-0000-0000-0001-000000000002', 'a0000000-0000-0000-0000-000000000001', 'A1.2', 'Environmental Protections', 'The entity authorizes, designs, develops or acquires, implements, operates, approves, maintains, and monitors environmental protections, software, data backup processes, and recovery infrastructure to meet its objectives.', 'Availability', 'a2000000-0000-0000-0000-000000000001', 2),
('a2000000-0000-0000-0001-000000000003', 'a0000000-0000-0000-0000-000000000001', 'A1.3', 'Recovery Testing', 'The entity tests recovery plan procedures supporting system recovery to meet its objectives.', 'Availability', 'a2000000-0000-0000-0000-000000000001', 3);

-- ===========================================
-- PROCESSING INTEGRITY (PI)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a3000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000001', 'PI', 'Processing Integrity', 'The criteria relevant to system processing being complete, valid, accurate, timely, and authorized.', 'Processing Integrity', NULL, 11);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a3000000-0000-0000-0001-000000000001', 'a0000000-0000-0000-0000-000000000001', 'PI1.1', 'Processing Integrity Policies', 'The entity obtains or generates, uses, and communicates relevant, quality information regarding the objectives related to processing, including definitions of data processed and product and service specifications, to support the use of products and services.', 'Processing Integrity', 'a3000000-0000-0000-0000-000000000001', 1),
('a3000000-0000-0000-0001-000000000002', 'a0000000-0000-0000-0000-000000000001', 'PI1.2', 'System Input Authorization', 'The entity implements policies and procedures over system inputs, including controls over completeness and accuracy, to result in products, services, and reporting to meet the entity''s objectives.', 'Processing Integrity', 'a3000000-0000-0000-0000-000000000001', 2),
('a3000000-0000-0000-0001-000000000003', 'a0000000-0000-0000-0000-000000000001', 'PI1.3', 'System Processing', 'The entity implements policies and procedures over system processing to result in products, services, and reporting to meet the entity''s objectives.', 'Processing Integrity', 'a3000000-0000-0000-0000-000000000001', 3),
('a3000000-0000-0000-0001-000000000004', 'a0000000-0000-0000-0000-000000000001', 'PI1.4', 'System Output', 'The entity implements policies and procedures to make available or deliver output completely, accurately, and timely in accordance with specifications to meet the entity''s objectives.', 'Processing Integrity', 'a3000000-0000-0000-0000-000000000001', 4),
('a3000000-0000-0000-0001-000000000005', 'a0000000-0000-0000-0000-000000000001', 'PI1.5', 'Storage Inputs and Outputs', 'The entity implements policies and procedures to store inputs, items in processing, and outputs completely, accurately, and timely in accordance with system specifications to meet the entity''s objectives.', 'Processing Integrity', 'a3000000-0000-0000-0000-000000000001', 5);

-- ===========================================
-- CONFIDENTIALITY (C)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a4000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000001', 'C', 'Confidentiality', 'The criteria relevant to information designated as confidential being protected as committed or agreed.', 'Confidentiality', NULL, 12);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a4000000-0000-0000-0001-000000000001', 'a0000000-0000-0000-0000-000000000001', 'C1.1', 'Confidential Information Identification', 'The entity identifies and maintains confidential information to meet the entity''s objectives related to confidentiality.', 'Confidentiality', 'a4000000-0000-0000-0000-000000000001', 1),
('a4000000-0000-0000-0001-000000000002', 'a0000000-0000-0000-0000-000000000001', 'C1.2', 'Confidential Information Disposal', 'The entity disposes of confidential information to meet the entity''s objectives related to confidentiality.', 'Confidentiality', 'a4000000-0000-0000-0000-000000000001', 2);

-- ===========================================
-- PRIVACY (P)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a5000000-0000-0000-0000-000000000001', 'a0000000-0000-0000-0000-000000000001', 'P', 'Privacy', 'The criteria relevant to personal information collected, used, retained, disclosed, and disposed of.', 'Privacy', NULL, 13);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('a5000000-0000-0000-0001-000000000001', 'a0000000-0000-0000-0000-000000000001', 'P1.1', 'Privacy Notice', 'The entity provides notice to data subjects about its privacy practices to meet the entity''s objectives related to privacy. The notice is updated and communicated to data subjects in a timely manner for changes to the entity''s privacy practices, including changes in the use of personal information, to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 1),
('a5000000-0000-0000-0001-000000000002', 'a0000000-0000-0000-0000-000000000001', 'P2.1', 'Choice and Consent', 'The entity communicates choices available regarding the collection, use, retention, disclosure, and disposal of personal information to data subjects and the consequences, if any, of each choice. Explicit consent for the collection, use, retention, disclosure, and disposal of personal information is obtained from data subjects or other authorized persons, if required.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 2),
('a5000000-0000-0000-0001-000000000003', 'a0000000-0000-0000-0000-000000000001', 'P3.1', 'Personal Information Collection', 'Personal information is collected consistent with the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 3),
('a5000000-0000-0000-0001-000000000004', 'a0000000-0000-0000-0000-000000000001', 'P3.2', 'Collection Limitation', 'For information requiring explicit consent, the entity communicates the need for such consent, as well as the consequences of a failure to provide consent for the request for personal information, and obtains the consent prior to the collection of the information to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 4),
('a5000000-0000-0000-0001-000000000005', 'a0000000-0000-0000-0000-000000000001', 'P4.1', 'Use of Personal Information', 'The entity limits the use of personal information to the purposes identified in the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 5),
('a5000000-0000-0000-0001-000000000006', 'a0000000-0000-0000-0000-000000000001', 'P4.2', 'Data Retention', 'The entity retains personal information consistent with the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 6),
('a5000000-0000-0000-0001-000000000007', 'a0000000-0000-0000-0000-000000000001', 'P4.3', 'Personal Information Disposal', 'The entity securely disposes of personal information to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 7),
('a5000000-0000-0000-0001-000000000008', 'a0000000-0000-0000-0000-000000000001', 'P5.1', 'Data Subject Access', 'The entity grants identified and authenticated data subjects the ability to access their stored personal information for review and, upon request, provides physical or electronic copies of that information to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 8),
('a5000000-0000-0000-0001-000000000009', 'a0000000-0000-0000-0000-000000000001', 'P5.2', 'Data Correction', 'The entity corrects, amends, or appends personal information based on information provided by data subjects and communicates such information to third parties, as committed or required, to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 9),
('a5000000-0000-0000-0001-000000000010', 'a0000000-0000-0000-0000-000000000001', 'P6.1', 'Disclosure to Third Parties', 'The entity discloses personal information to third parties with the explicit consent of data subjects, and such disclosure is consistent with the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 10),
('a5000000-0000-0000-0001-000000000011', 'a0000000-0000-0000-0000-000000000001', 'P6.2', 'Authorized Disclosure', 'The entity creates and retains a complete, accurate, and timely record of authorized disclosures of personal information to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 11),
('a5000000-0000-0000-0001-000000000012', 'a0000000-0000-0000-0000-000000000001', 'P6.3', 'Unauthorized Disclosure', 'The entity creates and retains a complete, accurate, and timely record of detected or reported unauthorized disclosures of personal information to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 12),
('a5000000-0000-0000-0001-000000000013', 'a0000000-0000-0000-0000-000000000001', 'P6.4', 'Third-Party Provider Security', 'The entity obtains privacy commitments from vendors and other third parties who have access to personal information to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 13),
('a5000000-0000-0000-0001-000000000014', 'a0000000-0000-0000-0000-000000000001', 'P6.5', 'Third-Party Compliance', 'The entity obtains commitments from vendors and other third parties with access to personal information to notify the entity in the event of actual or suspected unauthorized disclosures of personal information.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 14),
('a5000000-0000-0000-0001-000000000015', 'a0000000-0000-0000-0000-000000000001', 'P6.6', 'Third-Party Notification', 'The entity provides notification of breaches and incidents to affected data subjects, regulators, and others to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 15),
('a5000000-0000-0000-0001-000000000016', 'a0000000-0000-0000-0000-000000000001', 'P6.7', 'Data Subject Inquiries', 'The entity provides data subjects with an account of the personal information held and, upon request, information about the disclosure of their personal information to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 16),
('a5000000-0000-0000-0001-000000000017', 'a0000000-0000-0000-0000-000000000001', 'P7.1', 'Data Quality', 'The entity collects and maintains accurate, up-to-date, complete, and relevant personal information to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 17),
('a5000000-0000-0000-0001-000000000018', 'a0000000-0000-0000-0000-000000000001', 'P8.1', 'Privacy Monitoring', 'The entity implements a process for receiving, addressing, resolving, and communicating the resolution of inquiries, complaints, and disputes from data subjects and others and periodically monitors compliance to meet the entity''s objectives related to privacy.', 'Privacy', 'a5000000-0000-0000-0000-000000000001', 18);

-- Create index on framework_id for faster queries (if not already exists)
CREATE INDEX IF NOT EXISTS idx_framework_requirements_framework_id ON framework_requirements(framework_id);
CREATE INDEX IF NOT EXISTS idx_framework_requirements_parent_id ON framework_requirements(parent_id);
CREATE INDEX IF NOT EXISTS idx_framework_requirements_code ON framework_requirements(framework_id, code);
