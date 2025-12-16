-- SOX ITGC - Sarbanes-Oxley IT General Controls
-- Based on COSO Framework and PCAOB Standards

-- Insert SOX ITGC Framework
INSERT INTO frameworks (id, name, version, description, category, is_system, created_at)
VALUES (
    'g0000000-0000-0000-0000-000000000001',
    'SOX ITGC',
    '2024',
    'IT General Controls (ITGCs) for Sarbanes-Oxley compliance based on COSO Internal Control Framework and PCAOB standards. Covers access controls, change management, computer operations, and program development.',
    'compliance',
    true,
    NOW()
);

-- ===========================================
-- ACCESS TO PROGRAMS AND DATA (APD)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('g1000000-0000-0000-0000-000000000001', 'g0000000-0000-0000-0000-000000000001', 'APD', 'Access to Programs and Data', 'Controls that restrict access to programs and data to authorized personnel.', 'Access Controls', NULL, 1);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('g1000000-0000-0000-0001-000000000001', 'g0000000-0000-0000-0000-000000000001', 'APD.1', 'User Access Management', 'Access to systems, applications, and data is restricted to authorized users based on job responsibilities and business need.', 'Access Controls', 'g1000000-0000-0000-0000-000000000001', 1),
('g1000000-0000-0000-0001-000000000002', 'g0000000-0000-0000-0000-000000000001', 'APD.2', 'User Provisioning and De-provisioning', 'Formal procedures exist for granting, modifying, and removing access to systems and applications.', 'Access Controls', 'g1000000-0000-0000-0000-000000000001', 2),
('g1000000-0000-0000-0001-000000000003', 'g0000000-0000-0000-0000-000000000001', 'APD.3', 'Privileged Access Management', 'Administrative and privileged access is restricted, monitored, and periodically reviewed.', 'Access Controls', 'g1000000-0000-0000-0000-000000000001', 3),
('g1000000-0000-0000-0001-000000000004', 'g0000000-0000-0000-0000-000000000001', 'APD.4', 'Authentication Controls', 'Strong authentication mechanisms (passwords, MFA) are implemented and enforced.', 'Access Controls', 'g1000000-0000-0000-0000-000000000001', 4),
('g1000000-0000-0000-0001-000000000005', 'g0000000-0000-0000-0000-000000000001', 'APD.5', 'User Access Reviews', 'Periodic reviews of user access are performed to ensure access remains appropriate.', 'Access Controls', 'g1000000-0000-0000-0000-000000000001', 5),
('g1000000-0000-0000-0001-000000000006', 'g0000000-0000-0000-0000-000000000001', 'APD.6', 'Segregation of Duties', 'Access is configured to enforce appropriate segregation of duties and prevent conflicts.', 'Access Controls', 'g1000000-0000-0000-0000-000000000001', 6),
('g1000000-0000-0000-0001-000000000007', 'g0000000-0000-0000-0000-000000000001', 'APD.7', 'Physical Access Controls', 'Physical access to computing facilities and equipment is restricted to authorized personnel.', 'Access Controls', 'g1000000-0000-0000-0000-000000000001', 7),
('g1000000-0000-0000-0001-000000000008', 'g0000000-0000-0000-0000-000000000001', 'APD.8', 'Network Security', 'Network access is controlled through firewalls, segmentation, and monitoring.', 'Access Controls', 'g1000000-0000-0000-0000-000000000001', 8);

-- ===========================================
-- PROGRAM CHANGE MANAGEMENT (PCM)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('g2000000-0000-0000-0000-000000000001', 'g0000000-0000-0000-0000-000000000001', 'PCM', 'Program Change Management', 'Controls that ensure changes to programs are authorized, tested, approved, and properly implemented.', 'Change Management', NULL, 2);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('g2000000-0000-0000-0001-000000000001', 'g0000000-0000-0000-0000-000000000001', 'PCM.1', 'Change Request and Authorization', 'All changes to production systems are requested through a formal process and authorized before development.', 'Change Management', 'g2000000-0000-0000-0000-000000000001', 1),
('g2000000-0000-0000-0001-000000000002', 'g0000000-0000-0000-0000-000000000001', 'PCM.2', 'Development Standards', 'Changes are developed according to established coding standards and development practices.', 'Change Management', 'g2000000-0000-0000-0000-000000000001', 2),
('g2000000-0000-0000-0001-000000000003', 'g0000000-0000-0000-0000-000000000001', 'PCM.3', 'Testing and Quality Assurance', 'Changes are tested in a non-production environment before implementation, including user acceptance testing.', 'Change Management', 'g2000000-0000-0000-0000-000000000001', 3),
('g2000000-0000-0000-0001-000000000004', 'g0000000-0000-0000-0000-000000000001', 'PCM.4', 'Change Approval', 'Changes are reviewed and approved by appropriate personnel before migration to production.', 'Change Management', 'g2000000-0000-0000-0000-000000000001', 4),
('g2000000-0000-0000-0001-000000000005', 'g0000000-0000-0000-0000-000000000001', 'PCM.5', 'Implementation Controls', 'Migration of changes to production follows defined procedures with appropriate controls.', 'Change Management', 'g2000000-0000-0000-0000-000000000001', 5),
('g2000000-0000-0000-0001-000000000006', 'g0000000-0000-0000-0000-000000000001', 'PCM.6', 'Segregation of Development and Production', 'Development and production environments are logically or physically separated.', 'Change Management', 'g2000000-0000-0000-0000-000000000001', 6),
('g2000000-0000-0000-0001-000000000007', 'g0000000-0000-0000-0000-000000000001', 'PCM.7', 'Emergency Change Procedures', 'Emergency changes follow an expedited but controlled process with retroactive documentation.', 'Change Management', 'g2000000-0000-0000-0000-000000000001', 7),
('g2000000-0000-0000-0001-000000000008', 'g0000000-0000-0000-0000-000000000001', 'PCM.8', 'Rollback Procedures', 'Procedures exist to rollback changes that fail or cause issues in production.', 'Change Management', 'g2000000-0000-0000-0000-000000000001', 8);

-- ===========================================
-- PROGRAM DEVELOPMENT (PD)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('g3000000-0000-0000-0000-000000000001', 'g0000000-0000-0000-0000-000000000001', 'PD', 'Program Development', 'Controls that ensure new systems and applications are developed with appropriate controls.', 'Program Development', NULL, 3);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('g3000000-0000-0000-0001-000000000001', 'g0000000-0000-0000-0000-000000000001', 'PD.1', 'Project Governance', 'New system development projects follow a formal methodology with appropriate oversight and approvals.', 'Program Development', 'g3000000-0000-0000-0000-000000000001', 1),
('g3000000-0000-0000-0001-000000000002', 'g0000000-0000-0000-0000-000000000001', 'PD.2', 'Requirements Definition', 'Business and technical requirements are documented, reviewed, and approved before development.', 'Program Development', 'g3000000-0000-0000-0000-000000000001', 2),
('g3000000-0000-0000-0001-000000000003', 'g0000000-0000-0000-0000-000000000001', 'PD.3', 'Design and Development', 'System design is documented and development follows established standards and practices.', 'Program Development', 'g3000000-0000-0000-0000-000000000001', 3),
('g3000000-0000-0000-0001-000000000004', 'g0000000-0000-0000-0000-000000000001', 'PD.4', 'Security Requirements', 'Security requirements are identified and incorporated into the system design and development.', 'Program Development', 'g3000000-0000-0000-0000-000000000001', 4),
('g3000000-0000-0000-0001-000000000005', 'g0000000-0000-0000-0000-000000000001', 'PD.5', 'Testing Strategy', 'Comprehensive testing including unit, integration, system, and user acceptance testing is performed.', 'Program Development', 'g3000000-0000-0000-0000-000000000001', 5),
('g3000000-0000-0000-0001-000000000006', 'g0000000-0000-0000-0000-000000000001', 'PD.6', 'Data Migration Controls', 'Data migration is planned, tested, and validated to ensure completeness and accuracy.', 'Program Development', 'g3000000-0000-0000-0000-000000000001', 6),
('g3000000-0000-0000-0001-000000000007', 'g0000000-0000-0000-0000-000000000001', 'PD.7', 'Implementation and Go-Live', 'Go-live decisions are formally approved and implementation follows defined procedures.', 'Program Development', 'g3000000-0000-0000-0000-000000000001', 7),
('g3000000-0000-0000-0001-000000000008', 'g0000000-0000-0000-0000-000000000001', 'PD.8', 'Post-Implementation Review', 'Post-implementation reviews are conducted to ensure the system meets requirements.', 'Program Development', 'g3000000-0000-0000-0000-000000000001', 8);

-- ===========================================
-- COMPUTER OPERATIONS (CO)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('g4000000-0000-0000-0000-000000000001', 'g0000000-0000-0000-0000-000000000001', 'CO', 'Computer Operations', 'Controls that ensure programs are executed as authorized and deviations are investigated.', 'Computer Operations', NULL, 4);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('g4000000-0000-0000-0001-000000000001', 'g0000000-0000-0000-0000-000000000001', 'CO.1', 'Job Scheduling', 'Batch jobs and scheduled processes are authorized, scheduled, and monitored.', 'Computer Operations', 'g4000000-0000-0000-0000-000000000001', 1),
('g4000000-0000-0000-0001-000000000002', 'g0000000-0000-0000-0000-000000000001', 'CO.2', 'Job Execution Monitoring', 'Job execution is monitored and failures are investigated and resolved timely.', 'Computer Operations', 'g4000000-0000-0000-0000-000000000001', 2),
('g4000000-0000-0000-0001-000000000003', 'g0000000-0000-0000-0000-000000000001', 'CO.3', 'Data Backup and Recovery', 'Data is backed up regularly and backup media is stored securely offsite.', 'Computer Operations', 'g4000000-0000-0000-0000-000000000001', 3),
('g4000000-0000-0000-0001-000000000004', 'g0000000-0000-0000-0000-000000000001', 'CO.4', 'Recovery Testing', 'Backup and recovery procedures are tested periodically to ensure data can be restored.', 'Computer Operations', 'g4000000-0000-0000-0000-000000000001', 4),
('g4000000-0000-0000-0001-000000000005', 'g0000000-0000-0000-0000-000000000001', 'CO.5', 'Problem and Incident Management', 'IT problems and incidents are logged, tracked, and resolved through a formal process.', 'Computer Operations', 'g4000000-0000-0000-0000-000000000001', 5),
('g4000000-0000-0000-0001-000000000006', 'g0000000-0000-0000-0000-000000000001', 'CO.6', 'Disaster Recovery Planning', 'Disaster recovery plans exist and are tested to ensure business continuity.', 'Computer Operations', 'g4000000-0000-0000-0000-000000000001', 6),
('g4000000-0000-0000-0001-000000000007', 'g0000000-0000-0000-0000-000000000001', 'CO.7', 'System Availability Monitoring', 'System availability and performance are monitored with defined thresholds and responses.', 'Computer Operations', 'g4000000-0000-0000-0000-000000000001', 7),
('g4000000-0000-0000-0001-000000000008', 'g0000000-0000-0000-0000-000000000001', 'CO.8', 'Environmental Controls', 'Physical and environmental controls protect computing equipment from damage.', 'Computer Operations', 'g4000000-0000-0000-0000-000000000001', 8);

-- ===========================================
-- IT ENTITY LEVEL CONTROLS (ELC)
-- ===========================================

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('g5000000-0000-0000-0000-000000000001', 'g0000000-0000-0000-0000-000000000001', 'ELC', 'IT Entity Level Controls', 'Overarching controls that support the IT control environment.', 'Entity Level Controls', NULL, 5);

INSERT INTO framework_requirements (id, framework_id, code, name, description, category, parent_id, sort_order) VALUES
('g5000000-0000-0000-0001-000000000001', 'g0000000-0000-0000-0000-000000000001', 'ELC.1', 'IT Governance', 'IT governance structure and processes are established to align IT with business objectives.', 'Entity Level Controls', 'g5000000-0000-0000-0000-000000000001', 1),
('g5000000-0000-0000-0001-000000000002', 'g0000000-0000-0000-0000-000000000001', 'ELC.2', 'IT Policies and Procedures', 'IT policies and procedures are documented, approved, and communicated.', 'Entity Level Controls', 'g5000000-0000-0000-0000-000000000001', 2),
('g5000000-0000-0000-0001-000000000003', 'g0000000-0000-0000-0000-000000000001', 'ELC.3', 'IT Risk Management', 'IT risks are identified, assessed, and managed through a formal process.', 'Entity Level Controls', 'g5000000-0000-0000-0000-000000000001', 3),
('g5000000-0000-0000-0001-000000000004', 'g0000000-0000-0000-0000-000000000001', 'ELC.4', 'IT Organization and Staffing', 'Appropriate IT organizational structure and staffing supports control objectives.', 'Entity Level Controls', 'g5000000-0000-0000-0000-000000000001', 4),
('g5000000-0000-0000-0001-000000000005', 'g0000000-0000-0000-0000-000000000001', 'ELC.5', 'Vendor Management', 'Third-party IT service providers are subject to appropriate oversight and controls.', 'Entity Level Controls', 'g5000000-0000-0000-0000-000000000001', 5),
('g5000000-0000-0000-0001-000000000006', 'g0000000-0000-0000-0000-000000000001', 'ELC.6', 'Security Awareness', 'IT security awareness training is provided to employees and contractors.', 'Entity Level Controls', 'g5000000-0000-0000-0000-000000000001', 6),
('g5000000-0000-0000-0001-000000000007', 'g0000000-0000-0000-0000-000000000001', 'ELC.7', 'Audit Trail and Logging', 'Audit trails and logs are maintained for security and compliance purposes.', 'Entity Level Controls', 'g5000000-0000-0000-0000-000000000001', 7),
('g5000000-0000-0000-0001-000000000008', 'g0000000-0000-0000-0000-000000000001', 'ELC.8', 'IT Monitoring and Reporting', 'IT control effectiveness is monitored and reported to management.', 'Entity Level Controls', 'g5000000-0000-0000-0000-000000000001', 8);
