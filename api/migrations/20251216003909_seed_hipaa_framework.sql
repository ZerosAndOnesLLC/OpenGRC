-- HIPAA Security Rule (45 CFR Part 164)
-- Health Insurance Portability and Accountability Act

-- Insert HIPAA Framework
INSERT INTO frameworks (id, name, version, description, category, is_system, created_at)
VALUES (
    'c0000000-0000-0000-0000-000000000001',
    'HIPAA',
    '2013',
    'HIPAA Security Rule establishes national standards to protect electronic personal health information (ePHI). Includes Administrative, Physical, and Technical Safeguards.',
    'compliance',
    true,
    NOW()
);

-- ===========================================
-- ADMINISTRATIVE SAFEGUARDS (164.308)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.308', 'Administrative Safeguards', 'Administrative actions, policies, and procedures to manage the selection, development, implementation, and maintenance of security measures to protect ePHI.', 'Administrative Safeguards', NULL, 1);

-- 164.308(a)(1) Security Management Process
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(1)', 'Security Management Process', 'Implement policies and procedures to prevent, detect, contain, and correct security violations.', 'Administrative Safeguards', 'c1000000-0000-0000-0000-000000000001', 1);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000101', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(1)(ii)(A)', 'Risk Analysis (Required)', 'Conduct an accurate and thorough assessment of the potential risks and vulnerabilities to the confidentiality, integrity, and availability of ePHI held by the covered entity or business associate.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000001', 1),
('c1000000-0000-0000-0001-000000000102', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(1)(ii)(B)', 'Risk Management (Required)', 'Implement security measures sufficient to reduce risks and vulnerabilities to a reasonable and appropriate level.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000001', 2),
('c1000000-0000-0000-0001-000000000103', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(1)(ii)(C)', 'Sanction Policy (Required)', 'Apply appropriate sanctions against workforce members who fail to comply with security policies and procedures.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000001', 3),
('c1000000-0000-0000-0001-000000000104', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(1)(ii)(D)', 'Information System Activity Review (Required)', 'Implement procedures to regularly review records of information system activity, such as audit logs, access reports, and security incident tracking reports.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000001', 4);

-- 164.308(a)(2) Assigned Security Responsibility
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000002', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(2)', 'Assigned Security Responsibility (Required)', 'Identify the security official who is responsible for the development and implementation of the policies and procedures required by the Security Rule.', 'Administrative Safeguards', 'c1000000-0000-0000-0000-000000000001', 2);

-- 164.308(a)(3) Workforce Security
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000003', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(3)', 'Workforce Security', 'Implement policies and procedures to ensure that all members of its workforce have appropriate access to ePHI and to prevent unauthorized access.', 'Administrative Safeguards', 'c1000000-0000-0000-0000-000000000001', 3);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000301', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(3)(ii)(A)', 'Authorization and/or Supervision (Addressable)', 'Implement procedures for the authorization and/or supervision of workforce members who work with ePHI or in locations where it might be accessed.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000003', 1),
('c1000000-0000-0000-0001-000000000302', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(3)(ii)(B)', 'Workforce Clearance Procedure (Addressable)', 'Implement procedures to determine that the access of a workforce member to ePHI is appropriate.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000003', 2),
('c1000000-0000-0000-0001-000000000303', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(3)(ii)(C)', 'Termination Procedures (Addressable)', 'Implement procedures for terminating access to ePHI when the employment of, or other arrangement with, a workforce member ends.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000003', 3);

-- 164.308(a)(4) Information Access Management
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000004', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(4)', 'Information Access Management', 'Implement policies and procedures for authorizing access to ePHI.', 'Administrative Safeguards', 'c1000000-0000-0000-0000-000000000001', 4);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000401', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(4)(ii)(A)', 'Isolating Health Care Clearinghouse Functions (Required)', 'If a health care clearinghouse is part of a larger organization, the clearinghouse must implement policies and procedures that protect ePHI from unauthorized access by the larger organization.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000004', 1),
('c1000000-0000-0000-0001-000000000402', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(4)(ii)(B)', 'Access Authorization (Addressable)', 'Implement policies and procedures for granting access to ePHI, for example, through access to a workstation, transaction, program, process, or other mechanism.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000004', 2),
('c1000000-0000-0000-0001-000000000403', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(4)(ii)(C)', 'Access Establishment and Modification (Addressable)', 'Implement policies and procedures that, based upon the covered entity''s or business associate''s access authorization policies, establish, document, review, and modify a user''s right of access to a workstation, transaction, program, or process.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000004', 3);

-- 164.308(a)(5) Security Awareness and Training
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000005', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(5)', 'Security Awareness and Training', 'Implement a security awareness and training program for all members of its workforce (including management).', 'Administrative Safeguards', 'c1000000-0000-0000-0000-000000000001', 5);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000501', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(5)(ii)(A)', 'Security Reminders (Addressable)', 'Periodic security updates.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000005', 1),
('c1000000-0000-0000-0001-000000000502', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(5)(ii)(B)', 'Protection from Malicious Software (Addressable)', 'Procedures for guarding against, detecting, and reporting malicious software.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000005', 2),
('c1000000-0000-0000-0001-000000000503', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(5)(ii)(C)', 'Log-in Monitoring (Addressable)', 'Procedures for monitoring log-in attempts and reporting discrepancies.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000005', 3),
('c1000000-0000-0000-0001-000000000504', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(5)(ii)(D)', 'Password Management (Addressable)', 'Procedures for creating, changing, and safeguarding passwords.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000005', 4);

-- 164.308(a)(6) Security Incident Procedures
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000006', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(6)', 'Security Incident Procedures', 'Implement policies and procedures to address security incidents.', 'Administrative Safeguards', 'c1000000-0000-0000-0000-000000000001', 6);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000601', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(6)(ii)', 'Response and Reporting (Required)', 'Identify and respond to suspected or known security incidents; mitigate, to the extent practicable, harmful effects of security incidents that are known to the covered entity or business associate; and document security incidents and their outcomes.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000006', 1);

-- 164.308(a)(7) Contingency Plan
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000007', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(7)', 'Contingency Plan', 'Establish (and implement as needed) policies and procedures for responding to an emergency or other occurrence that damages systems that contain ePHI.', 'Administrative Safeguards', 'c1000000-0000-0000-0000-000000000001', 7);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000701', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(7)(ii)(A)', 'Data Backup Plan (Required)', 'Establish and implement procedures to create and maintain retrievable exact copies of ePHI.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000007', 1),
('c1000000-0000-0000-0001-000000000702', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(7)(ii)(B)', 'Disaster Recovery Plan (Required)', 'Establish (and implement as needed) procedures to restore any loss of data.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000007', 2),
('c1000000-0000-0000-0001-000000000703', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(7)(ii)(C)', 'Emergency Mode Operation Plan (Required)', 'Establish (and implement as needed) procedures to enable continuation of critical business processes for protection of the security of ePHI while operating in emergency mode.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000007', 3),
('c1000000-0000-0000-0001-000000000704', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(7)(ii)(D)', 'Testing and Revision Procedures (Addressable)', 'Implement procedures for periodic testing and revision of contingency plans.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000007', 4),
('c1000000-0000-0000-0001-000000000705', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(7)(ii)(E)', 'Applications and Data Criticality Analysis (Addressable)', 'Assess the relative criticality of specific applications and data in support of other contingency plan components.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000007', 5);

-- 164.308(a)(8) Evaluation
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000008', 'c0000000-0000-0000-0000-000000000001', '164.308(a)(8)', 'Evaluation (Required)', 'Perform a periodic technical and nontechnical evaluation, based initially upon the standards implemented under this rule and, subsequently, in response to environmental or operational changes affecting the security of ePHI.', 'Administrative Safeguards', 'c1000000-0000-0000-0000-000000000001', 8);

-- 164.308(b)(1) Business Associate Contracts
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000009', 'c0000000-0000-0000-0000-000000000001', '164.308(b)(1)', 'Business Associate Contracts and Other Arrangements', 'A covered entity may permit a business associate to create, receive, maintain, or transmit ePHI on the covered entity''s behalf only if the covered entity obtains satisfactory assurances that the business associate will appropriately safeguard the information.', 'Administrative Safeguards', 'c1000000-0000-0000-0000-000000000001', 9);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c1000000-0000-0000-0001-000000000901', 'c0000000-0000-0000-0000-000000000001', '164.308(b)(4)', 'Written Contract or Other Arrangement (Required)', 'Document the satisfactory assurances required through a written contract or other arrangement with the business associate.', 'Administrative Safeguards', 'c1000000-0000-0000-0001-000000000009', 1);

-- ===========================================
-- PHYSICAL SAFEGUARDS (164.310)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c2000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.310', 'Physical Safeguards', 'Physical measures, policies, and procedures to protect electronic information systems and related buildings and equipment from natural and environmental hazards, and unauthorized intrusion.', 'Physical Safeguards', NULL, 2);

-- 164.310(a)(1) Facility Access Controls
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c2000000-0000-0000-0001-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.310(a)(1)', 'Facility Access Controls', 'Implement policies and procedures to limit physical access to its electronic information systems and the facility or facilities in which they are housed, while ensuring that properly authorized access is allowed.', 'Physical Safeguards', 'c2000000-0000-0000-0000-000000000001', 1);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c2000000-0000-0000-0001-000000000101', 'c0000000-0000-0000-0000-000000000001', '164.310(a)(2)(i)', 'Contingency Operations (Addressable)', 'Establish (and implement as needed) procedures that allow facility access in support of restoration of lost data under the disaster recovery plan and emergency mode operations plan.', 'Physical Safeguards', 'c2000000-0000-0000-0001-000000000001', 1),
('c2000000-0000-0000-0001-000000000102', 'c0000000-0000-0000-0000-000000000001', '164.310(a)(2)(ii)', 'Facility Security Plan (Addressable)', 'Implement policies and procedures to safeguard the facility and the equipment therein from unauthorized physical access, tampering, and theft.', 'Physical Safeguards', 'c2000000-0000-0000-0001-000000000001', 2),
('c2000000-0000-0000-0001-000000000103', 'c0000000-0000-0000-0000-000000000001', '164.310(a)(2)(iii)', 'Access Control and Validation Procedures (Addressable)', 'Implement procedures to control and validate a person''s access to facilities based on their role or function, including visitor control, and control of access to software programs for testing and revision.', 'Physical Safeguards', 'c2000000-0000-0000-0001-000000000001', 3),
('c2000000-0000-0000-0001-000000000104', 'c0000000-0000-0000-0000-000000000001', '164.310(a)(2)(iv)', 'Maintenance Records (Addressable)', 'Implement policies and procedures to document repairs and modifications to the physical components of a facility which are related to security.', 'Physical Safeguards', 'c2000000-0000-0000-0001-000000000001', 4);

-- 164.310(b) Workstation Use
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c2000000-0000-0000-0001-000000000002', 'c0000000-0000-0000-0000-000000000001', '164.310(b)', 'Workstation Use (Required)', 'Implement policies and procedures that specify the proper functions to be performed, the manner in which those functions are to be performed, and the physical attributes of the surroundings of a specific workstation or class of workstation that can access ePHI.', 'Physical Safeguards', 'c2000000-0000-0000-0000-000000000001', 2);

-- 164.310(c) Workstation Security
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c2000000-0000-0000-0001-000000000003', 'c0000000-0000-0000-0000-000000000001', '164.310(c)', 'Workstation Security (Required)', 'Implement physical safeguards for all workstations that access ePHI, to restrict access to authorized users.', 'Physical Safeguards', 'c2000000-0000-0000-0000-000000000001', 3);

-- 164.310(d)(1) Device and Media Controls
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c2000000-0000-0000-0001-000000000004', 'c0000000-0000-0000-0000-000000000001', '164.310(d)(1)', 'Device and Media Controls', 'Implement policies and procedures that govern the receipt and removal of hardware and electronic media that contain ePHI into and out of a facility, and the movement of these items within the facility.', 'Physical Safeguards', 'c2000000-0000-0000-0000-000000000001', 4);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c2000000-0000-0000-0001-000000000401', 'c0000000-0000-0000-0000-000000000001', '164.310(d)(2)(i)', 'Disposal (Required)', 'Implement policies and procedures to address the final disposition of ePHI, and/or the hardware or electronic media on which it is stored.', 'Physical Safeguards', 'c2000000-0000-0000-0001-000000000004', 1),
('c2000000-0000-0000-0001-000000000402', 'c0000000-0000-0000-0000-000000000001', '164.310(d)(2)(ii)', 'Media Re-use (Required)', 'Implement procedures for removal of ePHI from electronic media before the media are made available for re-use.', 'Physical Safeguards', 'c2000000-0000-0000-0001-000000000004', 2),
('c2000000-0000-0000-0001-000000000403', 'c0000000-0000-0000-0000-000000000001', '164.310(d)(2)(iii)', 'Accountability (Addressable)', 'Maintain a record of the movements of hardware and electronic media and any person responsible therefore.', 'Physical Safeguards', 'c2000000-0000-0000-0001-000000000004', 3),
('c2000000-0000-0000-0001-000000000404', 'c0000000-0000-0000-0000-000000000001', '164.310(d)(2)(iv)', 'Data Backup and Storage (Addressable)', 'Create a retrievable, exact copy of ePHI, when needed, before movement of equipment.', 'Physical Safeguards', 'c2000000-0000-0000-0001-000000000004', 4);

-- ===========================================
-- TECHNICAL SAFEGUARDS (164.312)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c3000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.312', 'Technical Safeguards', 'The technology and the policy and procedures for its use that protect ePHI and control access to it.', 'Technical Safeguards', NULL, 3);

-- 164.312(a)(1) Access Control
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c3000000-0000-0000-0001-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.312(a)(1)', 'Access Control', 'Implement technical policies and procedures for electronic information systems that maintain ePHI to allow access only to those persons or software programs that have been granted access rights.', 'Technical Safeguards', 'c3000000-0000-0000-0000-000000000001', 1);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c3000000-0000-0000-0001-000000000101', 'c0000000-0000-0000-0000-000000000001', '164.312(a)(2)(i)', 'Unique User Identification (Required)', 'Assign a unique name and/or number for identifying and tracking user identity.', 'Technical Safeguards', 'c3000000-0000-0000-0001-000000000001', 1),
('c3000000-0000-0000-0001-000000000102', 'c0000000-0000-0000-0000-000000000001', '164.312(a)(2)(ii)', 'Emergency Access Procedure (Required)', 'Establish (and implement as needed) procedures for obtaining necessary ePHI during an emergency.', 'Technical Safeguards', 'c3000000-0000-0000-0001-000000000001', 2),
('c3000000-0000-0000-0001-000000000103', 'c0000000-0000-0000-0000-000000000001', '164.312(a)(2)(iii)', 'Automatic Logoff (Addressable)', 'Implement electronic procedures that terminate an electronic session after a predetermined time of inactivity.', 'Technical Safeguards', 'c3000000-0000-0000-0001-000000000001', 3),
('c3000000-0000-0000-0001-000000000104', 'c0000000-0000-0000-0000-000000000001', '164.312(a)(2)(iv)', 'Encryption and Decryption (Addressable)', 'Implement a mechanism to encrypt and decrypt ePHI.', 'Technical Safeguards', 'c3000000-0000-0000-0001-000000000001', 4);

-- 164.312(b) Audit Controls
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c3000000-0000-0000-0001-000000000002', 'c0000000-0000-0000-0000-000000000001', '164.312(b)', 'Audit Controls (Required)', 'Implement hardware, software, and/or procedural mechanisms that record and examine activity in information systems that contain or use ePHI.', 'Technical Safeguards', 'c3000000-0000-0000-0000-000000000001', 2);

-- 164.312(c)(1) Integrity
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c3000000-0000-0000-0001-000000000003', 'c0000000-0000-0000-0000-000000000001', '164.312(c)(1)', 'Integrity', 'Implement policies and procedures to protect ePHI from improper alteration or destruction.', 'Technical Safeguards', 'c3000000-0000-0000-0000-000000000001', 3);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c3000000-0000-0000-0001-000000000301', 'c0000000-0000-0000-0000-000000000001', '164.312(c)(2)', 'Mechanism to Authenticate ePHI (Addressable)', 'Implement electronic mechanisms to corroborate that ePHI has not been altered or destroyed in an unauthorized manner.', 'Technical Safeguards', 'c3000000-0000-0000-0001-000000000003', 1);

-- 164.312(d) Person or Entity Authentication
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c3000000-0000-0000-0001-000000000004', 'c0000000-0000-0000-0000-000000000001', '164.312(d)', 'Person or Entity Authentication (Required)', 'Implement procedures to verify that a person or entity seeking access to ePHI is the one claimed.', 'Technical Safeguards', 'c3000000-0000-0000-0000-000000000001', 4);

-- 164.312(e)(1) Transmission Security
INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c3000000-0000-0000-0001-000000000005', 'c0000000-0000-0000-0000-000000000001', '164.312(e)(1)', 'Transmission Security', 'Implement technical security measures to guard against unauthorized access to ePHI that is being transmitted over an electronic communications network.', 'Technical Safeguards', 'c3000000-0000-0000-0000-000000000001', 5);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c3000000-0000-0000-0001-000000000501', 'c0000000-0000-0000-0000-000000000001', '164.312(e)(2)(i)', 'Integrity Controls (Addressable)', 'Implement security measures to ensure that electronically transmitted ePHI is not improperly modified without detection until disposed of.', 'Technical Safeguards', 'c3000000-0000-0000-0001-000000000005', 1),
('c3000000-0000-0000-0001-000000000502', 'c0000000-0000-0000-0000-000000000001', '164.312(e)(2)(ii)', 'Encryption (Addressable)', 'Implement a mechanism to encrypt ePHI whenever deemed appropriate.', 'Technical Safeguards', 'c3000000-0000-0000-0001-000000000005', 2);

-- ===========================================
-- ORGANIZATIONAL REQUIREMENTS (164.314)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c4000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.314', 'Organizational Requirements', 'Requirements for business associate contracts and other arrangements.', 'Organizational Requirements', NULL, 4);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c4000000-0000-0000-0001-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.314(a)(1)', 'Business Associate Contracts (Required)', 'The contract must provide that the business associate will comply with the applicable requirements of the Security Rule, report security incidents, and ensure subcontractors agree to the same restrictions.', 'Organizational Requirements', 'c4000000-0000-0000-0000-000000000001', 1),
('c4000000-0000-0000-0001-000000000002', 'c0000000-0000-0000-0000-000000000001', '164.314(b)(1)', 'Requirements for Group Health Plans', 'A group health plan must ensure its plan sponsor will reasonably and appropriately safeguard ePHI created, received, maintained, or transmitted to or by the plan sponsor on behalf of the group health plan.', 'Organizational Requirements', 'c4000000-0000-0000-0000-000000000001', 2);

-- ===========================================
-- POLICIES AND PROCEDURES (164.316)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c5000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.316', 'Policies and Procedures and Documentation Requirements', 'Requirements for implementing reasonable and appropriate policies and procedures.', 'Documentation', NULL, 5);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c5000000-0000-0000-0001-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.316(a)', 'Policies and Procedures (Required)', 'Implement reasonable and appropriate policies and procedures to comply with the standards, implementation specifications, or other requirements of the Security Rule.', 'Documentation', 'c5000000-0000-0000-0000-000000000001', 1),
('c5000000-0000-0000-0001-000000000002', 'c0000000-0000-0000-0000-000000000001', '164.316(b)(1)', 'Documentation (Required)', 'Maintain the policies and procedures implemented to comply with the Security Rule in written (which may be electronic) form.', 'Documentation', 'c5000000-0000-0000-0000-000000000001', 2),
('c5000000-0000-0000-0001-000000000003', 'c0000000-0000-0000-0000-000000000001', '164.316(b)(2)(i)', 'Time Limit (Required)', 'Retain the documentation required for 6 years from the date of its creation or the date when it last was in effect, whichever is later.', 'Documentation', 'c5000000-0000-0000-0000-000000000001', 3),
('c5000000-0000-0000-0001-000000000004', 'c0000000-0000-0000-0000-000000000001', '164.316(b)(2)(ii)', 'Availability (Required)', 'Make documentation available to those persons responsible for implementing the procedures to which the documentation pertains.', 'Documentation', 'c5000000-0000-0000-0000-000000000001', 4),
('c5000000-0000-0000-0001-000000000005', 'c0000000-0000-0000-0000-000000000001', '164.316(b)(2)(iii)', 'Updates (Required)', 'Review documentation periodically, and update as needed, in response to environmental or operational changes affecting the security of ePHI.', 'Documentation', 'c5000000-0000-0000-0000-000000000001', 5);

-- ===========================================
-- BREACH NOTIFICATION RULE (164.400-414)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c6000000-0000-0000-0000-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.400', 'Breach Notification Rule', 'Requirements for notification in the case of breach of unsecured PHI.', 'Breach Notification', NULL, 6);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('c6000000-0000-0000-0001-000000000001', 'c0000000-0000-0000-0000-000000000001', '164.404', 'Notification to Individuals', 'A covered entity shall notify each individual whose unsecured PHI has been, or is reasonably believed to have been, accessed, acquired, used, or disclosed as a result of a breach.', 'Breach Notification', 'c6000000-0000-0000-0000-000000000001', 1),
('c6000000-0000-0000-0001-000000000002', 'c0000000-0000-0000-0000-000000000001', '164.406', 'Notification to Media', 'For breaches involving more than 500 residents of a State or jurisdiction, covered entities must notify prominent media outlets serving the State or jurisdiction.', 'Breach Notification', 'c6000000-0000-0000-0000-000000000001', 2),
('c6000000-0000-0000-0001-000000000003', 'c0000000-0000-0000-0000-000000000001', '164.408', 'Notification to Secretary', 'Covered entities must notify the Secretary of HHS of breaches of unsecured PHI.', 'Breach Notification', 'c6000000-0000-0000-0000-000000000001', 3),
('c6000000-0000-0000-0001-000000000004', 'c0000000-0000-0000-0000-000000000001', '164.410', 'Notification by Business Associate', 'A business associate shall notify the covered entity of a breach of unsecured PHI.', 'Breach Notification', 'c6000000-0000-0000-0000-000000000001', 4),
('c6000000-0000-0000-0001-000000000005', 'c0000000-0000-0000-0000-000000000001', '164.412', 'Law Enforcement Delay', 'If a law enforcement official determines that a notification would impede a criminal investigation, a covered entity or business associate shall delay the notification.', 'Breach Notification', 'c6000000-0000-0000-0000-000000000001', 5),
('c6000000-0000-0000-0001-000000000006', 'c0000000-0000-0000-0000-000000000001', '164.414', 'Administrative Requirements', 'A covered entity is required to comply with the administrative requirements of 164.530 with respect to the Breach Notification Rule.', 'Breach Notification', 'c6000000-0000-0000-0000-000000000001', 6);
