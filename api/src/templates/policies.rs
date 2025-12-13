use once_cell::sync::Lazy;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct PolicyTemplate {
    pub id: &'static str,
    pub code: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub category: &'static str,
    pub frameworks: &'static [&'static str],
    pub review_frequency: &'static str,
    pub content: &'static str,
    pub related_templates: &'static [&'static str],
    pub suggested_controls: &'static [&'static str],
}

pub static POLICY_TEMPLATES: Lazy<Vec<PolicyTemplate>> = Lazy::new(|| {
    vec![
        // ==================== SECURITY POLICIES ====================
        PolicyTemplate {
            id: "sec-001",
            code: "SEC-001",
            title: "Information Security Policy",
            description: "Establishes the organization's approach to managing information security and protecting assets",
            category: "security",
            frameworks: &["soc2", "iso27001", "hipaa", "pci-dss"],
            review_frequency: "annual",
            related_templates: &["sec-002", "sec-003", "comp-001"],
            suggested_controls: &["AC-001", "RA-001"],
            content: include_str!("policies/sec_001_information_security.md"),
        },
        PolicyTemplate {
            id: "sec-002",
            code: "SEC-002",
            title: "Access Control Policy",
            description: "Defines requirements for controlling access to information systems and data",
            category: "security",
            frameworks: &["soc2", "iso27001", "hipaa", "pci-dss"],
            review_frequency: "annual",
            related_templates: &["sec-001", "sec-003"],
            suggested_controls: &["AC-002", "AC-003", "AC-004"],
            content: include_str!("policies/sec_002_access_control.md"),
        },
        PolicyTemplate {
            id: "sec-003",
            code: "SEC-003",
            title: "Password & Authentication Policy",
            description: "Establishes requirements for passwords, MFA, and authentication mechanisms",
            category: "security",
            frameworks: &["soc2", "iso27001", "hipaa", "pci-dss"],
            review_frequency: "annual",
            related_templates: &["sec-001", "sec-002"],
            suggested_controls: &["IA-001", "IA-002", "IA-003"],
            content: include_str!("policies/sec_003_password_authentication.md"),
        },
        PolicyTemplate {
            id: "sec-004",
            code: "SEC-004",
            title: "Encryption Policy",
            description: "Defines requirements for encryption of data at rest and in transit",
            category: "security",
            frameworks: &["soc2", "iso27001", "hipaa", "pci-dss"],
            review_frequency: "annual",
            related_templates: &["sec-001", "comp-005"],
            suggested_controls: &["SC-001", "SC-002"],
            content: include_str!("policies/sec_004_encryption.md"),
        },
        PolicyTemplate {
            id: "sec-005",
            code: "SEC-005",
            title: "Network Security Policy",
            description: "Establishes requirements for securing network infrastructure and communications",
            category: "security",
            frameworks: &["soc2", "iso27001", "pci-dss"],
            review_frequency: "annual",
            related_templates: &["sec-001", "sec-004"],
            suggested_controls: &["SC-003", "SC-004", "SC-005"],
            content: include_str!("policies/sec_005_network_security.md"),
        },
        PolicyTemplate {
            id: "sec-006",
            code: "SEC-006",
            title: "Vulnerability Management Policy",
            description: "Defines processes for identifying, assessing, and remediating security vulnerabilities",
            category: "security",
            frameworks: &["soc2", "iso27001", "pci-dss"],
            review_frequency: "annual",
            related_templates: &["sec-001", "it-002", "comp-003"],
            suggested_controls: &["RA-002", "RA-003", "SI-001"],
            content: include_str!("policies/sec_006_vulnerability_management.md"),
        },
        PolicyTemplate {
            id: "sec-007",
            code: "SEC-007",
            title: "Security Awareness Training Policy",
            description: "Establishes requirements for security awareness and training programs",
            category: "security",
            frameworks: &["soc2", "iso27001", "hipaa", "pci-dss"],
            review_frequency: "annual",
            related_templates: &["sec-001", "hr-001"],
            suggested_controls: &["AT-001", "AT-002"],
            content: include_str!("policies/sec_007_security_awareness.md"),
        },
        PolicyTemplate {
            id: "sec-008",
            code: "SEC-008",
            title: "Physical Security Policy",
            description: "Defines requirements for physical access controls and facility security",
            category: "security",
            frameworks: &["soc2", "iso27001", "hipaa", "pci-dss"],
            review_frequency: "annual",
            related_templates: &["sec-001", "it-004"],
            suggested_controls: &["PE-001", "PE-002", "PE-003"],
            content: include_str!("policies/sec_008_physical_security.md"),
        },

        // ==================== IT OPERATIONS POLICIES ====================
        PolicyTemplate {
            id: "it-001",
            code: "IT-001",
            title: "Acceptable Use Policy",
            description: "Defines acceptable use of organizational IT resources and systems",
            category: "it",
            frameworks: &["soc2", "iso27001", "hipaa"],
            review_frequency: "annual",
            related_templates: &["sec-001", "hr-001"],
            suggested_controls: &["PL-001", "AC-005"],
            content: include_str!("policies/it_001_acceptable_use.md"),
        },
        PolicyTemplate {
            id: "it-002",
            code: "IT-002",
            title: "Change Management Policy",
            description: "Establishes processes for managing changes to IT systems and infrastructure",
            category: "it",
            frameworks: &["soc2", "iso27001", "pci-dss"],
            review_frequency: "annual",
            related_templates: &["it-007", "comp-003"],
            suggested_controls: &["CM-001", "CM-002", "CM-003"],
            content: include_str!("policies/it_002_change_management.md"),
        },
        PolicyTemplate {
            id: "it-003",
            code: "IT-003",
            title: "Backup & Recovery Policy",
            description: "Defines requirements for data backup, retention, and recovery procedures",
            category: "it",
            frameworks: &["soc2", "iso27001", "hipaa"],
            review_frequency: "annual",
            related_templates: &["comp-004", "priv-002"],
            suggested_controls: &["CP-001", "CP-002", "CP-003"],
            content: include_str!("policies/it_003_backup_recovery.md"),
        },
        PolicyTemplate {
            id: "it-004",
            code: "IT-004",
            title: "Asset Management Policy",
            description: "Establishes requirements for managing IT assets throughout their lifecycle",
            category: "it",
            frameworks: &["soc2", "iso27001"],
            review_frequency: "annual",
            related_templates: &["sec-001", "comp-005"],
            suggested_controls: &["AM-001", "AM-002", "AM-003"],
            content: include_str!("policies/it_004_asset_management.md"),
        },
        PolicyTemplate {
            id: "it-005",
            code: "IT-005",
            title: "Mobile Device & BYOD Policy",
            description: "Defines security requirements for mobile devices and bring-your-own-device programs",
            category: "it",
            frameworks: &["soc2", "iso27001", "hipaa"],
            review_frequency: "annual",
            related_templates: &["sec-001", "sec-004", "it-001"],
            suggested_controls: &["AC-006", "MP-001", "MP-002"],
            content: include_str!("policies/it_005_mobile_byod.md"),
        },
        PolicyTemplate {
            id: "it-006",
            code: "IT-006",
            title: "Remote Work Policy",
            description: "Establishes security requirements for remote and distributed work arrangements",
            category: "it",
            frameworks: &["soc2", "iso27001"],
            review_frequency: "annual",
            related_templates: &["sec-001", "it-005", "sec-005"],
            suggested_controls: &["AC-007", "SC-006"],
            content: include_str!("policies/it_006_remote_work.md"),
        },
        PolicyTemplate {
            id: "it-007",
            code: "IT-007",
            title: "Software Development Lifecycle Policy",
            description: "Defines secure software development practices and requirements",
            category: "it",
            frameworks: &["soc2", "iso27001", "pci-dss"],
            review_frequency: "annual",
            related_templates: &["it-002", "sec-006"],
            suggested_controls: &["SA-001", "SA-002", "SA-003"],
            content: include_str!("policies/it_007_sdlc.md"),
        },

        // ==================== COMPLIANCE & RISK POLICIES ====================
        PolicyTemplate {
            id: "comp-001",
            code: "COMP-001",
            title: "Risk Management Policy",
            description: "Establishes the framework for identifying, assessing, and managing organizational risks",
            category: "compliance",
            frameworks: &["soc2", "iso27001", "hipaa"],
            review_frequency: "annual",
            related_templates: &["sec-001", "comp-002"],
            suggested_controls: &["RA-001", "RA-004", "PM-001"],
            content: include_str!("policies/comp_001_risk_management.md"),
        },
        PolicyTemplate {
            id: "comp-002",
            code: "COMP-002",
            title: "Vendor Management Policy",
            description: "Defines requirements for assessing and managing third-party vendors",
            category: "compliance",
            frameworks: &["soc2", "iso27001", "hipaa"],
            review_frequency: "annual",
            related_templates: &["comp-001", "priv-001"],
            suggested_controls: &["SA-004", "SA-005", "PM-002"],
            content: include_str!("policies/comp_002_vendor_management.md"),
        },
        PolicyTemplate {
            id: "comp-003",
            code: "COMP-003",
            title: "Incident Response Policy",
            description: "Establishes procedures for detecting, responding to, and recovering from security incidents",
            category: "compliance",
            frameworks: &["soc2", "iso27001", "hipaa", "pci-dss"],
            review_frequency: "annual",
            related_templates: &["comp-004", "priv-003"],
            suggested_controls: &["IR-001", "IR-002", "IR-003", "IR-004"],
            content: include_str!("policies/comp_003_incident_response.md"),
        },
        PolicyTemplate {
            id: "comp-004",
            code: "COMP-004",
            title: "Business Continuity Policy",
            description: "Defines requirements for maintaining business operations during disruptions",
            category: "compliance",
            frameworks: &["soc2", "iso27001", "hipaa"],
            review_frequency: "annual",
            related_templates: &["it-003", "comp-003"],
            suggested_controls: &["CP-004", "CP-005", "CP-006"],
            content: include_str!("policies/comp_004_business_continuity.md"),
        },
        PolicyTemplate {
            id: "comp-005",
            code: "COMP-005",
            title: "Data Classification Policy",
            description: "Establishes data classification levels and handling requirements",
            category: "compliance",
            frameworks: &["soc2", "iso27001", "hipaa"],
            review_frequency: "annual",
            related_templates: &["sec-004", "priv-001"],
            suggested_controls: &["RA-005", "MP-003", "MP-004"],
            content: include_str!("policies/comp_005_data_classification.md"),
        },

        // ==================== PRIVACY POLICIES ====================
        PolicyTemplate {
            id: "priv-001",
            code: "PRIV-001",
            title: "Data Privacy Policy",
            description: "Defines requirements for protecting personal and sensitive data",
            category: "privacy",
            frameworks: &["gdpr", "ccpa", "hipaa"],
            review_frequency: "annual",
            related_templates: &["comp-005", "priv-002", "priv-003"],
            suggested_controls: &["PR-001", "PR-002", "PR-003"],
            content: include_str!("policies/priv_001_data_privacy.md"),
        },
        PolicyTemplate {
            id: "priv-002",
            code: "PRIV-002",
            title: "Data Retention Policy",
            description: "Establishes requirements for retaining and disposing of data",
            category: "privacy",
            frameworks: &["soc2", "gdpr", "hipaa"],
            review_frequency: "annual",
            related_templates: &["it-003", "priv-001"],
            suggested_controls: &["SI-002", "MP-005", "MP-006"],
            content: include_str!("policies/priv_002_data_retention.md"),
        },
        PolicyTemplate {
            id: "priv-003",
            code: "PRIV-003",
            title: "Data Breach Notification Policy",
            description: "Defines procedures for notifying affected parties and authorities of data breaches",
            category: "privacy",
            frameworks: &["gdpr", "hipaa", "ccpa"],
            review_frequency: "annual",
            related_templates: &["comp-003", "priv-001"],
            suggested_controls: &["IR-005", "IR-006"],
            content: include_str!("policies/priv_003_breach_notification.md"),
        },

        // ==================== HUMAN RESOURCES POLICIES ====================
        PolicyTemplate {
            id: "hr-001",
            code: "HR-001",
            title: "Code of Conduct",
            description: "Establishes expected behavior and ethical standards for all employees",
            category: "hr",
            frameworks: &["soc2", "iso27001"],
            review_frequency: "annual",
            related_templates: &["it-001", "sec-007"],
            suggested_controls: &["PS-001", "PS-002"],
            content: include_str!("policies/hr_001_code_of_conduct.md"),
        },
        PolicyTemplate {
            id: "hr-002",
            code: "HR-002",
            title: "Background Check Policy",
            description: "Defines requirements for pre-employment and periodic background checks",
            category: "hr",
            frameworks: &["soc2", "iso27001", "hipaa"],
            review_frequency: "annual",
            related_templates: &["hr-001", "sec-001"],
            suggested_controls: &["PS-003", "PS-004"],
            content: include_str!("policies/hr_002_background_check.md"),
        },
    ]
});

pub fn get_template(id: &str) -> Option<&'static PolicyTemplate> {
    POLICY_TEMPLATES.iter().find(|t| t.id == id)
}

pub fn list_templates() -> &'static [PolicyTemplate] {
    &POLICY_TEMPLATES
}

pub fn search_templates(
    category: Option<&str>,
    framework: Option<&str>,
    query: Option<&str>,
) -> Vec<&'static PolicyTemplate> {
    POLICY_TEMPLATES
        .iter()
        .filter(|t| {
            let category_match = category.map_or(true, |c| t.category == c);
            let framework_match = framework.map_or(true, |f| t.frameworks.contains(&f));
            let query_match = query.map_or(true, |q| {
                let q_lower = q.to_lowercase();
                t.title.to_lowercase().contains(&q_lower)
                    || t.description.to_lowercase().contains(&q_lower)
                    || t.code.to_lowercase().contains(&q_lower)
            });
            category_match && framework_match && query_match
        })
        .collect()
}
