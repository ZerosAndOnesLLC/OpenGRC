# Data Classification Policy

**Policy Code:** COMP-005
**Version:** 1.0
**Effective Date:** [To be set]
**Review Date:** [Annual review recommended]
**Owner:** [To be assigned - typically CISO or Data Governance Officer]
**Category:** Compliance

## 1. Purpose

This Data Classification Policy establishes a framework for classifying and handling [Organization Name]'s information assets based on sensitivity and value. This policy ensures that data receives appropriate protection throughout its lifecycle and that handling requirements are clearly understood.

## 2. Scope

This policy applies to:
- All data created, processed, stored, or transmitted by the organization
- All data formats including electronic, paper, and verbal
- All employees, contractors, and third parties handling organizational data
- All systems and locations where data is stored or processed

## 3. Policy Statements

### 3.1 Classification Levels

#### 3.1.1 Restricted (Highest Sensitivity)
**Definition**: Data that, if disclosed, could cause severe damage to the organization, individuals, or result in significant legal/regulatory penalties.

**Examples**:
- Encryption keys and security credentials
- Authentication secrets (passwords, API keys)
- Personally identifiable information (PII) with high sensitivity (SSN, financial data)
- Protected health information (PHI)
- Payment card data (PCI)
- Trade secrets and critical intellectual property
- Board and executive strategic documents
- M&A information before public disclosure

**Handling Requirements**:
- Encryption required at rest and in transit
- Access limited to specific named individuals
- No storage on personal devices
- No transmission via email without encryption
- Printing requires approval
- Destruction requires witnessed shredding/secure deletion

#### 3.1.2 Confidential
**Definition**: Data that is intended for internal use only and whose disclosure could cause harm to the organization or individuals.

**Examples**:
- Customer data and contracts
- Employee personal information
- Financial records and reports
- Business strategies and plans
- Vendor contracts and pricing
- Internal communications of sensitive nature
- Security assessments and audit reports
- Source code and technical documentation

**Handling Requirements**:
- Encryption recommended at rest, required in transit
- Access limited to those with business need
- No sharing with unauthorized parties
- External sharing requires NDA
- Secure disposal required
- Clear labeling recommended

#### 3.1.3 Internal
**Definition**: Data intended for general internal use that is not meant for public disclosure.

**Examples**:
- Internal policies and procedures
- Organizational charts
- General business communications
- Training materials
- Internal meeting notes
- Project documentation

**Handling Requirements**:
- Standard access controls
- No posting to public locations
- Reasonable care in handling
- Standard disposal procedures

#### 3.1.4 Public
**Definition**: Data that is intended for public access or whose disclosure would not harm the organization.

**Examples**:
- Marketing materials
- Published press releases
- Public website content
- Published product documentation
- Job postings

**Handling Requirements**:
- No special controls required
- Ensure accuracy before publishing
- Follow brand guidelines

### 3.2 Classification Responsibilities

#### 3.2.1 Data Owners
- Assign classification to data they create or manage
- Review and update classifications periodically
- Approve access to classified data
- Ensure appropriate handling within their scope

#### 3.2.2 Data Custodians
- Implement technical controls appropriate to classification
- Ensure systems meet handling requirements
- Monitor access and usage
- Report classification violations

#### 3.2.3 All Users
- Handle data according to its classification
- Report suspected misclassification
- Not attempt to access data beyond authorization
- Properly dispose of data when no longer needed

### 3.3 Labeling Requirements

| Classification | Digital Documents | Emails | Physical Documents |
|---------------|-------------------|--------|-------------------|
| Restricted | Header/Footer label required | Subject prefix [RESTRICTED] | Stamp/Label required |
| Confidential | Header/Footer recommended | Subject prefix recommended | Label recommended |
| Internal | Optional | Not required | Not required |
| Public | Optional | Not required | Not required |

### 3.4 Handling Matrix

| Requirement | Restricted | Confidential | Internal | Public |
|-------------|------------|--------------|----------|--------|
| Encryption at Rest | Required | Recommended | Optional | N/A |
| Encryption in Transit | Required | Required | Recommended | N/A |
| Access Control | Named individuals | Role-based | Standard | None |
| External Sharing | Prohibited without approval | NDA required | Care required | Allowed |
| Printing | Approval required | Limited | Standard | Allowed |
| Mobile Devices | Prohibited | With MDM only | With MDM | Allowed |
| Cloud Storage | Approved services only | Approved services | Approved services | Any |

### 3.5 Data Lifecycle

#### 3.5.1 Creation
- Classify data at creation
- Apply appropriate controls immediately
- Document classification decisions for Restricted data

#### 3.5.2 Storage
- Store in approved locations for classification level
- Ensure appropriate access controls
- Maintain classification through transfers

#### 3.5.3 Transmission
- Use approved methods for classification level
- Encrypt as required
- Verify recipient authorization

#### 3.5.4 Disposal
- Follow retention requirements
- Dispose according to classification requirements
- Document destruction of Restricted data

### 3.6 Reclassification

- Data classification may change over time
- Annual review of classified data recommended
- Upgrade classification if sensitivity increases
- Downgrade when information becomes less sensitive
- Document reclassification decisions

### 3.7 Aggregation

- Combined data may require higher classification
- Consider cumulative sensitivity
- Apply highest classification when in doubt

## 4. Roles & Responsibilities

| Role | Responsibilities |
|------|-----------------|
| Data Owners | Classify data, approve access, review classifications |
| IT Security | Define controls, audit compliance, investigate incidents |
| Data Custodians | Implement controls, manage systems, monitor usage |
| All Users | Follow handling requirements, report violations |
| Legal/Compliance | Identify regulatory requirements, advise on classification |

## 5. Compliance

### 5.1 Monitoring
- Data access shall be logged for sensitive classifications
- Classification coverage shall be assessed periodically
- Handling compliance shall be audited
- Violations shall be tracked and reported

### 5.2 Enforcement
- Violations may result in access revocation
- Serious violations may result in disciplinary action
- Intentional violations may result in termination

## 6. Exceptions

- Exceptions must be documented and approved
- Compensating controls must be identified
- Exceptions for Restricted data require CISO approval
- Exceptions shall be time-limited and reviewed

## 7. Related Documents

- Information Security Policy (SEC-001)
- Encryption Policy (SEC-004)
- Data Privacy Policy (PRIV-001)
- Data Retention Policy (PRIV-002)
- Acceptable Use Policy (IT-001)

## 8. Definitions

| Term | Definition |
|------|-----------|
| Data Owner | Person accountable for the data and its classification |
| Data Custodian | Person responsible for implementing data protections |
| PII | Personally Identifiable Information |
| PHI | Protected Health Information (HIPAA) |
| PCI | Payment Card Industry data |
| Aggregation | Combining data that may increase sensitivity |

## 9. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | [Date] | [Author] | Initial release |
