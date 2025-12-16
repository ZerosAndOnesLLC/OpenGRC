-- PCI DSS 4.0 - Payment Card Industry Data Security Standard
-- Published March 2022, mandatory March 2024

-- Insert PCI DSS 4.0 Framework
INSERT INTO frameworks (id, name, version, description, category, is_system, created_at)
VALUES (
    'd0000000-0000-0000-0000-000000000001',
    'PCI DSS',
    '4.0',
    'Payment Card Industry Data Security Standard (PCI DSS) v4.0 provides a baseline of technical and operational requirements designed to protect cardholder data. Applies to all entities that store, process, or transmit cardholder data.',
    'compliance',
    true,
    NOW()
);

-- ===========================================
-- BUILD AND MAINTAIN A SECURE NETWORK AND SYSTEMS
-- ===========================================

-- Requirement 1: Install and Maintain Network Security Controls
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d1000000-0000-0000-0000-000000000001', 'd0000000-0000-0000-0000-000000000001', 'REQ-1', 'Install and Maintain Network Security Controls', 'Network security controls (NSCs), such as firewalls and other network security technologies, are network policy enforcement points that typically control network traffic between two or more logical or physical network segments.', 'Build and Maintain a Secure Network', NULL, 1);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d1000000-0000-0000-0001-000000000001', 'd0000000-0000-0000-0000-000000000001', '1.1', 'Processes and mechanisms for network security controls are defined and understood', 'All security policies and operational procedures identified in Requirement 1 are documented, kept up to date, in use, and known to all affected parties.', 'Build and Maintain a Secure Network', 'd1000000-0000-0000-0000-000000000001', 1),
('d1000000-0000-0000-0001-000000000002', 'd0000000-0000-0000-0000-000000000001', '1.2', 'Network security controls are configured and maintained', 'NSC rulesets are configured to restrict traffic from untrusted networks to system components in the CDE.', 'Build and Maintain a Secure Network', 'd1000000-0000-0000-0000-000000000001', 2),
('d1000000-0000-0000-0001-000000000003', 'd0000000-0000-0000-0000-000000000001', '1.3', 'Network access to and from the CDE is restricted', 'Network access to and from the cardholder data environment is appropriately restricted.', 'Build and Maintain a Secure Network', 'd1000000-0000-0000-0000-000000000001', 3),
('d1000000-0000-0000-0001-000000000004', 'd0000000-0000-0000-0000-000000000001', '1.4', 'Network connections between trusted and untrusted networks are controlled', 'Network traffic between trusted and untrusted networks is controlled.', 'Build and Maintain a Secure Network', 'd1000000-0000-0000-0000-000000000001', 4),
('d1000000-0000-0000-0001-000000000005', 'd0000000-0000-0000-0000-000000000001', '1.5', 'Risks to the CDE from computing devices that connect to both untrusted networks and the CDE are mitigated', 'Risks from devices that can connect to both untrusted networks and the CDE are mitigated.', 'Build and Maintain a Secure Network', 'd1000000-0000-0000-0000-000000000001', 5);

-- Requirement 2: Apply Secure Configurations to All System Components
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d1000000-0000-0000-0000-000000000002', 'd0000000-0000-0000-0000-000000000001', 'REQ-2', 'Apply Secure Configurations to All System Components', 'Malicious individuals often use vendor default passwords and other vendor default settings to compromise systems. These passwords and settings are well known and easily determined.', 'Build and Maintain a Secure Network', NULL, 2);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d1000000-0000-0000-0002-000000000001', 'd0000000-0000-0000-0000-000000000001', '2.1', 'Processes and mechanisms for secure configurations are defined and understood', 'All security policies and operational procedures that are identified in Requirement 2 are documented, kept up to date, in use, and known to all affected parties.', 'Build and Maintain a Secure Network', 'd1000000-0000-0000-0000-000000000002', 1),
('d1000000-0000-0000-0002-000000000002', 'd0000000-0000-0000-0000-000000000001', '2.2', 'System components are configured and managed securely', 'Vendor default accounts are managed and removed if not needed, and default passwords are changed.', 'Build and Maintain a Secure Network', 'd1000000-0000-0000-0000-000000000002', 2),
('d1000000-0000-0000-0002-000000000003', 'd0000000-0000-0000-0000-000000000001', '2.3', 'Wireless environments are configured and managed securely', 'Wireless access points are managed securely with strong authentication and encryption.', 'Build and Maintain a Secure Network', 'd1000000-0000-0000-0000-000000000002', 3);

-- ===========================================
-- PROTECT ACCOUNT DATA
-- ===========================================

-- Requirement 3: Protect Stored Account Data
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d2000000-0000-0000-0000-000000000001', 'd0000000-0000-0000-0000-000000000001', 'REQ-3', 'Protect Stored Account Data', 'Protection methods such as encryption, truncation, masking, and hashing are critical components of account data protection.', 'Protect Account Data', NULL, 3);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d2000000-0000-0000-0001-000000000001', 'd0000000-0000-0000-0000-000000000001', '3.1', 'Processes and mechanisms for protecting stored account data are defined and understood', 'All security policies and operational procedures identified in Requirement 3 are documented, kept up to date, in use, and known to all affected parties.', 'Protect Account Data', 'd2000000-0000-0000-0000-000000000001', 1),
('d2000000-0000-0000-0001-000000000002', 'd0000000-0000-0000-0000-000000000001', '3.2', 'Storage of account data is kept to a minimum', 'Account data storage amount and retention time are limited to that which is required for business, legal, and/or regulatory purposes.', 'Protect Account Data', 'd2000000-0000-0000-0000-000000000001', 2),
('d2000000-0000-0000-0001-000000000003', 'd0000000-0000-0000-0000-000000000001', '3.3', 'Sensitive authentication data (SAD) is not stored after authorization', 'SAD is not retained after authorization, even if encrypted.', 'Protect Account Data', 'd2000000-0000-0000-0000-000000000001', 3),
('d2000000-0000-0000-0001-000000000004', 'd0000000-0000-0000-0000-000000000001', '3.4', 'Access to displays of full PAN and ability to copy cardholder data are restricted', 'The full PAN is not readable anywhere it is displayed or accessible without proper authorization.', 'Protect Account Data', 'd2000000-0000-0000-0000-000000000001', 4),
('d2000000-0000-0000-0001-000000000005', 'd0000000-0000-0000-0000-000000000001', '3.5', 'PAN is secured wherever it is stored', 'The primary account number is rendered unreadable anywhere it is stored using strong cryptography.', 'Protect Account Data', 'd2000000-0000-0000-0000-000000000001', 5),
('d2000000-0000-0000-0001-000000000006', 'd0000000-0000-0000-0000-000000000001', '3.6', 'Cryptographic keys used to protect stored account data are secured', 'Cryptographic key material used to protect stored account data is managed in accordance with industry standards.', 'Protect Account Data', 'd2000000-0000-0000-0000-000000000001', 6),
('d2000000-0000-0000-0001-000000000007', 'd0000000-0000-0000-0000-000000000001', '3.7', 'Where cryptography is used to protect stored account data, key management processes and procedures are defined and implemented', 'Full documentation of key management processes covering all aspects of the cryptographic key lifecycle.', 'Protect Account Data', 'd2000000-0000-0000-0000-000000000001', 7);

-- Requirement 4: Protect Cardholder Data with Strong Cryptography During Transmission
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d2000000-0000-0000-0000-000000000002', 'd0000000-0000-0000-0000-000000000001', 'REQ-4', 'Protect Cardholder Data with Strong Cryptography During Transmission Over Open, Public Networks', 'Sensitive information must be encrypted during transmission over networks that are easily accessed by malicious individuals.', 'Protect Account Data', NULL, 4);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d2000000-0000-0000-0002-000000000001', 'd0000000-0000-0000-0000-000000000001', '4.1', 'Processes and mechanisms for protecting cardholder data with strong cryptography during transmission are defined and understood', 'All security policies and operational procedures identified in Requirement 4 are documented, kept up to date, in use, and known to all affected parties.', 'Protect Account Data', 'd2000000-0000-0000-0000-000000000002', 1),
('d2000000-0000-0000-0002-000000000002', 'd0000000-0000-0000-0000-000000000001', '4.2', 'PAN is protected with strong cryptography during transmission', 'Strong cryptography and security protocols are used to safeguard PAN during transmission over open, public networks.', 'Protect Account Data', 'd2000000-0000-0000-0000-000000000002', 2);

-- ===========================================
-- MAINTAIN A VULNERABILITY MANAGEMENT PROGRAM
-- ===========================================

-- Requirement 5: Protect All Systems and Networks from Malicious Software
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d3000000-0000-0000-0000-000000000001', 'd0000000-0000-0000-0000-000000000001', 'REQ-5', 'Protect All Systems and Networks from Malicious Software', 'Malicious software (malware) is software or firmware designed to infiltrate or damage a computer system without the owner''s knowledge or consent.', 'Vulnerability Management', NULL, 5);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d3000000-0000-0000-0001-000000000001', 'd0000000-0000-0000-0000-000000000001', '5.1', 'Processes and mechanisms for protecting systems and networks from malicious software are defined and understood', 'All security policies and operational procedures identified in Requirement 5 are documented, kept up to date, in use, and known to all affected parties.', 'Vulnerability Management', 'd3000000-0000-0000-0000-000000000001', 1),
('d3000000-0000-0000-0001-000000000002', 'd0000000-0000-0000-0000-000000000001', '5.2', 'Malicious software is prevented or detected and addressed', 'An anti-malware solution is deployed on all system components, except for those identified as not at risk from malware.', 'Vulnerability Management', 'd3000000-0000-0000-0000-000000000001', 2),
('d3000000-0000-0000-0001-000000000003', 'd0000000-0000-0000-0000-000000000001', '5.3', 'Anti-malware mechanisms and processes are active, maintained, and monitored', 'Anti-malware solution(s) are kept current via automatic updates and perform periodic and real-time scans.', 'Vulnerability Management', 'd3000000-0000-0000-0000-000000000001', 3),
('d3000000-0000-0000-0001-000000000004', 'd0000000-0000-0000-0000-000000000001', '5.4', 'Anti-phishing mechanisms protect users against phishing attacks', 'Technical controls are in place to detect and protect users from phishing attacks.', 'Vulnerability Management', 'd3000000-0000-0000-0000-000000000001', 4);

-- Requirement 6: Develop and Maintain Secure Systems and Software
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d3000000-0000-0000-0000-000000000002', 'd0000000-0000-0000-0000-000000000001', 'REQ-6', 'Develop and Maintain Secure Systems and Software', 'Security vulnerabilities in systems and software may allow criminals to access PAN and other account data.', 'Vulnerability Management', NULL, 6);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d3000000-0000-0000-0002-000000000001', 'd0000000-0000-0000-0000-000000000001', '6.1', 'Processes and mechanisms for developing and maintaining secure systems and software are defined and understood', 'All security policies and operational procedures identified in Requirement 6 are documented, kept up to date, in use, and known to all affected parties.', 'Vulnerability Management', 'd3000000-0000-0000-0000-000000000002', 1),
('d3000000-0000-0000-0002-000000000002', 'd0000000-0000-0000-0000-000000000001', '6.2', 'Bespoke and custom software are developed securely', 'Software development processes and practices address security throughout the development lifecycle.', 'Vulnerability Management', 'd3000000-0000-0000-0000-000000000002', 2),
('d3000000-0000-0000-0002-000000000003', 'd0000000-0000-0000-0000-000000000001', '6.3', 'Security vulnerabilities are identified and addressed', 'Security vulnerabilities in systems and software are identified and managed using a risk-based approach.', 'Vulnerability Management', 'd3000000-0000-0000-0000-000000000002', 3),
('d3000000-0000-0000-0002-000000000004', 'd0000000-0000-0000-0000-000000000001', '6.4', 'Public-facing web applications are protected against attacks', 'Public-facing web applications are protected against known attacks.', 'Vulnerability Management', 'd3000000-0000-0000-0000-000000000002', 4),
('d3000000-0000-0000-0002-000000000005', 'd0000000-0000-0000-0000-000000000001', '6.5', 'Changes to all system components are managed securely', 'Change management processes ensure that security is maintained when changes are implemented.', 'Vulnerability Management', 'd3000000-0000-0000-0000-000000000002', 5);

-- ===========================================
-- IMPLEMENT STRONG ACCESS CONTROL MEASURES
-- ===========================================

-- Requirement 7: Restrict Access to System Components and Cardholder Data by Business Need to Know
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d4000000-0000-0000-0000-000000000001', 'd0000000-0000-0000-0000-000000000001', 'REQ-7', 'Restrict Access to System Components and Cardholder Data by Business Need to Know', 'To ensure critical data can only be accessed by authorized personnel, systems and processes must be in place to limit access based on need to know and according to job responsibilities.', 'Access Control', NULL, 7);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d4000000-0000-0000-0001-000000000001', 'd0000000-0000-0000-0000-000000000001', '7.1', 'Processes and mechanisms for restricting access to cardholder data by business need to know are defined and understood', 'All security policies and operational procedures identified in Requirement 7 are documented, kept up to date, in use, and known to all affected parties.', 'Access Control', 'd4000000-0000-0000-0000-000000000001', 1),
('d4000000-0000-0000-0001-000000000002', 'd0000000-0000-0000-0000-000000000001', '7.2', 'Access to system components and data is appropriately defined and assigned', 'Access rights are granted only to those with a business need and are approved by authorized personnel.', 'Access Control', 'd4000000-0000-0000-0000-000000000001', 2),
('d4000000-0000-0000-0001-000000000003', 'd0000000-0000-0000-0000-000000000001', '7.3', 'Access to system components and data is managed via an access control system(s)', 'An access control system is used to manage access to system components and cardholder data.', 'Access Control', 'd4000000-0000-0000-0000-000000000001', 3);

-- Requirement 8: Identify Users and Authenticate Access to System Components
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d4000000-0000-0000-0000-000000000002', 'd0000000-0000-0000-0000-000000000001', 'REQ-8', 'Identify Users and Authenticate Access to System Components', 'Assigning a unique identification (ID) to each person with access ensures that actions taken on critical data and systems are performed by, and can be traced to, known and authorized users.', 'Access Control', NULL, 8);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d4000000-0000-0000-0002-000000000001', 'd0000000-0000-0000-0000-000000000001', '8.1', 'Processes and mechanisms for identification and authentication are defined and understood', 'All security policies and operational procedures identified in Requirement 8 are documented, kept up to date, in use, and known to all affected parties.', 'Access Control', 'd4000000-0000-0000-0000-000000000002', 1),
('d4000000-0000-0000-0002-000000000002', 'd0000000-0000-0000-0000-000000000001', '8.2', 'User identification and related accounts are strictly managed throughout an account''s lifecycle', 'User accounts are managed throughout the full account lifecycle from creation to termination.', 'Access Control', 'd4000000-0000-0000-0000-000000000002', 2),
('d4000000-0000-0000-0002-000000000003', 'd0000000-0000-0000-0000-000000000001', '8.3', 'Strong authentication for users and administrators is established and managed', 'Strong authentication methods are implemented including MFA for all access to the CDE.', 'Access Control', 'd4000000-0000-0000-0000-000000000002', 3),
('d4000000-0000-0000-0002-000000000004', 'd0000000-0000-0000-0000-000000000001', '8.4', 'Multi-factor authentication (MFA) is implemented to secure access into the CDE', 'MFA is required for all personnel with administrative access and for all remote network access.', 'Access Control', 'd4000000-0000-0000-0000-000000000002', 4),
('d4000000-0000-0000-0002-000000000005', 'd0000000-0000-0000-0000-000000000001', '8.5', 'Multi-factor authentication (MFA) systems are configured to prevent misuse', 'MFA systems are implemented to prevent bypass and misuse.', 'Access Control', 'd4000000-0000-0000-0000-000000000002', 5),
('d4000000-0000-0000-0002-000000000006', 'd0000000-0000-0000-0000-000000000001', '8.6', 'Use of application and system accounts and associated authentication factors is strictly managed', 'Application and system accounts are managed with strong authentication.', 'Access Control', 'd4000000-0000-0000-0000-000000000002', 6);

-- Requirement 9: Restrict Physical Access to Cardholder Data
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d4000000-0000-0000-0000-000000000003', 'd0000000-0000-0000-0000-000000000001', 'REQ-9', 'Restrict Physical Access to Cardholder Data', 'Any physical access to cardholder data or systems that store, process, or transmit cardholder data provides the opportunity for individuals to access and/or remove devices, data, systems, or hardcopies.', 'Access Control', NULL, 9);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d4000000-0000-0000-0003-000000000001', 'd0000000-0000-0000-0000-000000000001', '9.1', 'Processes and mechanisms for restricting physical access to cardholder data are defined and understood', 'All security policies and operational procedures identified in Requirement 9 are documented, kept up to date, in use, and known to all affected parties.', 'Access Control', 'd4000000-0000-0000-0000-000000000003', 1),
('d4000000-0000-0000-0003-000000000002', 'd0000000-0000-0000-0000-000000000001', '9.2', 'Physical access controls manage entry into facilities and systems containing cardholder data', 'Appropriate physical access controls are in place for facilities and sensitive areas.', 'Access Control', 'd4000000-0000-0000-0000-000000000003', 2),
('d4000000-0000-0000-0003-000000000003', 'd0000000-0000-0000-0000-000000000001', '9.3', 'Physical access for personnel and visitors is authorized and managed', 'Physical access for onsite personnel and visitors is authorized and managed appropriately.', 'Access Control', 'd4000000-0000-0000-0000-000000000003', 3),
('d4000000-0000-0000-0003-000000000004', 'd0000000-0000-0000-0000-000000000001', '9.4', 'Media with cardholder data is securely stored, accessed, distributed, and destroyed', 'Media containing cardholder data is physically secured and controlled.', 'Access Control', 'd4000000-0000-0000-0000-000000000003', 4),
('d4000000-0000-0000-0003-000000000005', 'd0000000-0000-0000-0000-000000000001', '9.5', 'Point-of-interaction (POI) devices are protected from tampering and unauthorized substitution', 'POI devices at point of sale are protected from tampering and substitution.', 'Access Control', 'd4000000-0000-0000-0000-000000000003', 5);

-- ===========================================
-- REGULARLY MONITOR AND TEST NETWORKS
-- ===========================================

-- Requirement 10: Log and Monitor All Access to System Components and Cardholder Data
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d5000000-0000-0000-0000-000000000001', 'd0000000-0000-0000-0000-000000000001', 'REQ-10', 'Log and Monitor All Access to System Components and Cardholder Data', 'Logging mechanisms and the ability to track user activities are critical in preventing, detecting, or minimizing the impact of a data compromise.', 'Monitor and Test Networks', NULL, 10);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d5000000-0000-0000-0001-000000000001', 'd0000000-0000-0000-0000-000000000001', '10.1', 'Processes and mechanisms for logging and monitoring all access are defined and understood', 'All security policies and operational procedures identified in Requirement 10 are documented, kept up to date, in use, and known to all affected parties.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000001', 1),
('d5000000-0000-0000-0001-000000000002', 'd0000000-0000-0000-0000-000000000001', '10.2', 'Audit logs are implemented to support the detection of anomalies and suspicious activity', 'Audit logs are enabled and active for all system components that store, process, or transmit CHD and/or SAD.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000001', 2),
('d5000000-0000-0000-0001-000000000003', 'd0000000-0000-0000-0000-000000000001', '10.3', 'Audit logs are protected from destruction and unauthorized modifications', 'Audit trails are protected so they cannot be altered.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000001', 3),
('d5000000-0000-0000-0001-000000000004', 'd0000000-0000-0000-0000-000000000001', '10.4', 'Audit logs are reviewed to identify anomalies or suspicious activity', 'Audit logs of all system components are reviewed at least daily.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000001', 4),
('d5000000-0000-0000-0001-000000000005', 'd0000000-0000-0000-0000-000000000001', '10.5', 'Audit log history is retained and available for analysis', 'Audit log history is retained for at least 12 months, with at least three months immediately available for analysis.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000001', 5),
('d5000000-0000-0000-0001-000000000006', 'd0000000-0000-0000-0000-000000000001', '10.6', 'Time-synchronization mechanisms support consistent time across all systems', 'Time-synchronization technology is deployed and kept current.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000001', 6),
('d5000000-0000-0000-0001-000000000007', 'd0000000-0000-0000-0000-000000000001', '10.7', 'Failures of critical security control systems are detected, reported, and responded to promptly', 'Failures of critical security control systems are detected, alerted, and addressed promptly.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000001', 7);

-- Requirement 11: Test Security of Systems and Networks Regularly
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d5000000-0000-0000-0000-000000000002', 'd0000000-0000-0000-0000-000000000001', 'REQ-11', 'Test Security of Systems and Networks Regularly', 'Vulnerabilities are being discovered continually by malicious individuals and researchers, and being introduced by new software.', 'Monitor and Test Networks', NULL, 11);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d5000000-0000-0000-0002-000000000001', 'd0000000-0000-0000-0000-000000000001', '11.1', 'Processes and mechanisms for regularly testing security of systems and networks are defined and understood', 'All security policies and operational procedures identified in Requirement 11 are documented, kept up to date, in use, and known to all affected parties.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000002', 1),
('d5000000-0000-0000-0002-000000000002', 'd0000000-0000-0000-0000-000000000001', '11.2', 'Wireless access points are identified and monitored, and unauthorized wireless access points are addressed', 'Authorized and unauthorized wireless access points are managed.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000002', 2),
('d5000000-0000-0000-0002-000000000003', 'd0000000-0000-0000-0000-000000000001', '11.3', 'External and internal vulnerabilities are regularly identified, prioritized, and addressed', 'Vulnerability scans are performed regularly and high-risk vulnerabilities are addressed.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000002', 3),
('d5000000-0000-0000-0002-000000000004', 'd0000000-0000-0000-0000-000000000001', '11.4', 'External and internal penetration testing is regularly performed', 'Penetration testing is performed at least annually and after significant changes.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000002', 4),
('d5000000-0000-0000-0002-000000000005', 'd0000000-0000-0000-0000-000000000001', '11.5', 'Network intrusions and unexpected file changes are detected and responded to', 'Change-detection mechanisms and intrusion-detection and/or intrusion-prevention techniques are used.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000002', 5),
('d5000000-0000-0000-0002-000000000006', 'd0000000-0000-0000-0000-000000000001', '11.6', 'Unauthorized changes on payment pages are detected and responded to', 'A change- and tamper-detection mechanism is deployed on payment pages.', 'Monitor and Test Networks', 'd5000000-0000-0000-0000-000000000002', 6);

-- ===========================================
-- MAINTAIN AN INFORMATION SECURITY POLICY
-- ===========================================

-- Requirement 12: Support Information Security with Organizational Policies and Programs
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d6000000-0000-0000-0000-000000000001', 'd0000000-0000-0000-0000-000000000001', 'REQ-12', 'Support Information Security with Organizational Policies and Programs', 'A strong security policy sets the security tone for the whole entity and informs personnel what is expected of them.', 'Information Security Policy', NULL, 12);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('d6000000-0000-0000-0001-000000000001', 'd0000000-0000-0000-0000-000000000001', '12.1', 'A comprehensive information security policy is established, published, maintained, and disseminated', 'An information security policy is established, published, maintained, and disseminated to all relevant personnel.', 'Information Security Policy', 'd6000000-0000-0000-0000-000000000001', 1),
('d6000000-0000-0000-0001-000000000002', 'd0000000-0000-0000-0000-000000000001', '12.2', 'Acceptable use policies for end-user technologies are defined and implemented', 'Acceptable use policies for end-user technologies are documented and implemented.', 'Information Security Policy', 'd6000000-0000-0000-0000-000000000001', 2),
('d6000000-0000-0000-0001-000000000003', 'd0000000-0000-0000-0000-000000000001', '12.3', 'Risks to the cardholder data environment are formally identified, evaluated, and managed', 'A risk assessment process is implemented to identify threats, vulnerabilities, and resulting risk.', 'Information Security Policy', 'd6000000-0000-0000-0000-000000000001', 3),
('d6000000-0000-0000-0001-000000000004', 'd0000000-0000-0000-0000-000000000001', '12.4', 'PCI DSS compliance is managed', 'Responsibility for PCI DSS compliance is formally assigned and program is established.', 'Information Security Policy', 'd6000000-0000-0000-0000-000000000001', 4),
('d6000000-0000-0000-0001-000000000005', 'd0000000-0000-0000-0000-000000000001', '12.5', 'PCI DSS scope is documented and validated', 'The PCI DSS scope is documented and confirmed by the entity at least annually.', 'Information Security Policy', 'd6000000-0000-0000-0000-000000000001', 5),
('d6000000-0000-0000-0001-000000000006', 'd0000000-0000-0000-0000-000000000001', '12.6', 'Security awareness education is an ongoing activity', 'A formal security awareness program is implemented to make all personnel aware of security policies.', 'Information Security Policy', 'd6000000-0000-0000-0000-000000000001', 6),
('d6000000-0000-0000-0001-000000000007', 'd0000000-0000-0000-0000-000000000001', '12.7', 'Personnel are screened to reduce risks from insider threats', 'Potential personnel are screened prior to hire to minimize risk of insider threats.', 'Information Security Policy', 'd6000000-0000-0000-0000-000000000001', 7),
('d6000000-0000-0000-0001-000000000008', 'd0000000-0000-0000-0000-000000000001', '12.8', 'Risk to information assets associated with third-party service provider relationships is managed', 'Third-party service providers with access to account data are managed with policies and agreements.', 'Information Security Policy', 'd6000000-0000-0000-0000-000000000001', 8),
('d6000000-0000-0000-0001-000000000009', 'd0000000-0000-0000-0000-000000000001', '12.9', 'Third-party service providers support their customers'' PCI DSS compliance', 'TPSPs acknowledge and support their customers'' PCI DSS compliance.', 'Information Security Policy', 'd6000000-0000-0000-0000-000000000001', 9),
('d6000000-0000-0000-0001-000000000010', 'd0000000-0000-0000-0000-000000000001', '12.10', 'Suspected and confirmed security incidents that could impact the CDE are responded to immediately', 'An incident response plan is implemented and ready to be activated in the event of a cardholder data breach.', 'Information Security Policy', 'd6000000-0000-0000-0000-000000000001', 10);
