-- NIST Cybersecurity Framework (CSF) 2.0
-- National Institute of Standards and Technology

-- Insert NIST CSF Framework
INSERT INTO frameworks (id, name, version, description, category, is_system, created_at)
VALUES (
    'f0000000-0000-0000-0000-000000000001',
    'NIST CSF',
    '2.0',
    'The NIST Cybersecurity Framework provides a policy framework of computer security guidance for organizations to assess and improve their ability to prevent, detect, and respond to cyber attacks. Version 2.0 adds Govern as a sixth function.',
    'compliance',
    true,
    NOW()
);

-- ===========================================
-- GOVERN (GV) - New in CSF 2.0
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f1000000-0000-0000-0000-000000000001', 'f0000000-0000-0000-0000-000000000001', 'GV', 'Govern', 'Establish and monitor the organization''s cybersecurity risk management strategy, expectations, and policy. The Govern function provides outcomes to inform what an organization may do to achieve the other five Functions.', 'Govern', NULL, 1);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f1000000-0000-0000-0001-000000000001', 'f0000000-0000-0000-0000-000000000001', 'GV.OC', 'Organizational Context', 'The circumstances surrounding the organization''s cybersecurity risk management decisions are understood.', 'Govern', 'f1000000-0000-0000-0000-000000000001', 1),
('f1000000-0000-0000-0001-000000000002', 'f0000000-0000-0000-0000-000000000001', 'GV.RM', 'Risk Management Strategy', 'The organization''s priorities, constraints, risk tolerance statements, and assumptions are established and used to support operational risk decisions.', 'Govern', 'f1000000-0000-0000-0000-000000000001', 2),
('f1000000-0000-0000-0001-000000000003', 'f0000000-0000-0000-0000-000000000001', 'GV.RR', 'Roles, Responsibilities, and Authorities', 'Cybersecurity roles, responsibilities, and authorities to foster accountability are established and communicated.', 'Govern', 'f1000000-0000-0000-0000-000000000001', 3),
('f1000000-0000-0000-0001-000000000004', 'f0000000-0000-0000-0000-000000000001', 'GV.PO', 'Policy', 'Organizational cybersecurity policy is established, communicated, and enforced.', 'Govern', 'f1000000-0000-0000-0000-000000000001', 4),
('f1000000-0000-0000-0001-000000000005', 'f0000000-0000-0000-0000-000000000001', 'GV.OV', 'Oversight', 'Results of organization-wide cybersecurity risk management activities and performance are used to inform, improve, and adjust the risk management strategy.', 'Govern', 'f1000000-0000-0000-0000-000000000001', 5),
('f1000000-0000-0000-0001-000000000006', 'f0000000-0000-0000-0000-000000000001', 'GV.SC', 'Cybersecurity Supply Chain Risk Management', 'Cyber supply chain risk management processes are identified, established, managed, monitored, and improved by organizational stakeholders.', 'Govern', 'f1000000-0000-0000-0000-000000000001', 6);

-- ===========================================
-- IDENTIFY (ID)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f2000000-0000-0000-0000-000000000001', 'f0000000-0000-0000-0000-000000000001', 'ID', 'Identify', 'Help determine the current cybersecurity risk to the organization. Understanding assets, suppliers, and related cybersecurity risks enables focus and prioritization of efforts.', 'Identify', NULL, 2);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f2000000-0000-0000-0001-000000000001', 'f0000000-0000-0000-0000-000000000001', 'ID.AM', 'Asset Management', 'The data, personnel, devices, systems, and facilities that enable the organization to achieve business purposes are identified and managed consistent with their relative importance to organizational objectives and risk strategy.', 'Identify', 'f2000000-0000-0000-0000-000000000001', 1),
('f2000000-0000-0000-0001-000000000002', 'f0000000-0000-0000-0000-000000000001', 'ID.RA', 'Risk Assessment', 'The organization understands the cybersecurity risk to organizational operations, assets, and individuals.', 'Identify', 'f2000000-0000-0000-0000-000000000001', 2),
('f2000000-0000-0000-0001-000000000003', 'f0000000-0000-0000-0000-000000000001', 'ID.IM', 'Improvement', 'Improvements to organizational cybersecurity risk management processes, procedures and activities are identified across all CSF Functions.', 'Identify', 'f2000000-0000-0000-0000-000000000001', 3);

-- ===========================================
-- PROTECT (PR)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f3000000-0000-0000-0000-000000000001', 'f0000000-0000-0000-0000-000000000001', 'PR', 'Protect', 'Use safeguards to prevent or reduce cybersecurity risk. Outcomes covered by Protect support the ability to secure assets to prevent or lower likelihood and impact of adverse events.', 'Protect', NULL, 3);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f3000000-0000-0000-0001-000000000001', 'f0000000-0000-0000-0000-000000000001', 'PR.AA', 'Identity Management, Authentication, and Access Control', 'Access to physical and logical assets is limited to authorized users, services, and hardware and managed commensurate with the assessed risk of unauthorized access.', 'Protect', 'f3000000-0000-0000-0000-000000000001', 1),
('f3000000-0000-0000-0001-000000000002', 'f0000000-0000-0000-0000-000000000001', 'PR.AT', 'Awareness and Training', 'The organization''s personnel are provided cybersecurity awareness and training so that they can perform their cybersecurity-related tasks.', 'Protect', 'f3000000-0000-0000-0000-000000000001', 2),
('f3000000-0000-0000-0001-000000000003', 'f0000000-0000-0000-0000-000000000001', 'PR.DS', 'Data Security', 'Data are managed consistent with the organization''s risk strategy to protect the confidentiality, integrity, and availability of information.', 'Protect', 'f3000000-0000-0000-0000-000000000001', 3),
('f3000000-0000-0000-0001-000000000004', 'f0000000-0000-0000-0000-000000000001', 'PR.PS', 'Platform Security', 'The hardware, software, and services of physical and virtual platforms are managed consistent with the organization''s risk strategy.', 'Protect', 'f3000000-0000-0000-0000-000000000001', 4),
('f3000000-0000-0000-0001-000000000005', 'f0000000-0000-0000-0000-000000000001', 'PR.IR', 'Technology Infrastructure Resilience', 'Security architectures are managed with the organization''s risk strategy to protect asset confidentiality, integrity, and availability.', 'Protect', 'f3000000-0000-0000-0000-000000000001', 5);

-- ===========================================
-- DETECT (DE)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f4000000-0000-0000-0000-000000000001', 'f0000000-0000-0000-0000-000000000001', 'DE', 'Detect', 'Find and analyze possible cybersecurity attacks and compromises. Enables timely discovery and analysis of anomalies, indicators of compromise, and other potentially adverse events.', 'Detect', NULL, 4);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f4000000-0000-0000-0001-000000000001', 'f0000000-0000-0000-0000-000000000001', 'DE.CM', 'Continuous Monitoring', 'Assets are monitored to find anomalies, indicators of compromise, and other potentially adverse events.', 'Detect', 'f4000000-0000-0000-0000-000000000001', 1),
('f4000000-0000-0000-0001-000000000002', 'f0000000-0000-0000-0000-000000000001', 'DE.AE', 'Adverse Event Analysis', 'Anomalies, indicators of compromise, and other potentially adverse events are analyzed to characterize the events and detect cybersecurity incidents.', 'Detect', 'f4000000-0000-0000-0000-000000000001', 2);

-- ===========================================
-- RESPOND (RS)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f5000000-0000-0000-0000-000000000001', 'f0000000-0000-0000-0000-000000000001', 'RS', 'Respond', 'Take action regarding a detected cybersecurity incident. Supports the ability to contain the effects of cybersecurity incidents.', 'Respond', NULL, 5);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f5000000-0000-0000-0001-000000000001', 'f0000000-0000-0000-0000-000000000001', 'RS.MA', 'Incident Management', 'Responses to detected cybersecurity incidents are managed.', 'Respond', 'f5000000-0000-0000-0000-000000000001', 1),
('f5000000-0000-0000-0001-000000000002', 'f0000000-0000-0000-0000-000000000001', 'RS.AN', 'Incident Analysis', 'Investigations are conducted to ensure effective response and support forensics and recovery activities.', 'Respond', 'f5000000-0000-0000-0000-000000000001', 2),
('f5000000-0000-0000-0001-000000000003', 'f0000000-0000-0000-0000-000000000001', 'RS.CO', 'Incident Response Reporting and Communication', 'Response activities are coordinated with internal and external stakeholders as required by laws, regulations, or policies.', 'Respond', 'f5000000-0000-0000-0000-000000000001', 3),
('f5000000-0000-0000-0001-000000000004', 'f0000000-0000-0000-0000-000000000001', 'RS.MI', 'Incident Mitigation', 'Activities are performed to prevent expansion of an event and mitigate its effects.', 'Respond', 'f5000000-0000-0000-0000-000000000001', 4);

-- ===========================================
-- RECOVER (RC)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f6000000-0000-0000-0000-000000000001', 'f0000000-0000-0000-0000-000000000001', 'RC', 'Recover', 'Restore assets and operations that were impacted by a cybersecurity incident. Supports timely restoration of normal operations to reduce impact.', 'Recover', NULL, 6);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('f6000000-0000-0000-0001-000000000001', 'f0000000-0000-0000-0000-000000000001', 'RC.RP', 'Incident Recovery Plan Execution', 'Restoration activities are performed to ensure operational availability of systems and services affected by cybersecurity incidents.', 'Recover', 'f6000000-0000-0000-0000-000000000001', 1),
('f6000000-0000-0000-0001-000000000002', 'f0000000-0000-0000-0000-000000000001', 'RC.CO', 'Incident Recovery Communication', 'Restoration activities are coordinated with internal and external parties.', 'Recover', 'f6000000-0000-0000-0000-000000000001', 2);
