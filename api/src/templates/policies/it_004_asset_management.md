# Asset Management Policy

**Policy Code:** IT-004
**Version:** 1.0
**Effective Date:** [To be set]
**Review Date:** [Annual review recommended]
**Owner:** [To be assigned - typically IT Asset Manager]
**Category:** IT

## 1. Purpose

This Asset Management Policy establishes requirements for identifying, tracking, and managing [Organization Name]'s IT assets throughout their lifecycle. This policy ensures that assets are properly inventoried, secured, maintained, and disposed of to protect organizational information and optimize resource utilization.

## 2. Scope

This policy applies to:
- Hardware assets (laptops, desktops, servers, network equipment, mobile devices)
- Software assets (licenses, subscriptions, cloud services)
- Data assets (databases, file repositories, backups)
- Virtual assets (virtual machines, containers, cloud resources)
- All personnel responsible for procuring, using, or managing IT assets

## 3. Policy Statements

### 3.1 Asset Inventory

#### 3.1.1 Inventory Requirements
- All IT assets shall be inventoried in an asset management system
- Asset inventory shall be maintained and kept current
- Inventory shall be reconciled at least annually
- New assets shall be added to inventory before deployment

#### 3.1.2 Inventory Data
Each asset record shall include:
- Unique asset identifier/tag
- Asset type and description
- Serial number (hardware)
- Location (physical or cloud)
- Owner and custodian
- Acquisition date and cost
- Warranty/support information
- Data classification of data stored/processed
- End-of-life date

### 3.2 Asset Lifecycle

#### 3.2.1 Procurement
- Assets shall be procured through approved channels
- Security requirements shall be considered during procurement
- Asset records shall be created at procurement
- Licenses shall be validated before purchase

#### 3.2.2 Deployment
- Assets shall be configured according to security baselines
- Endpoint protection shall be installed before deployment
- Asset tags shall be applied to hardware
- Asset assignment shall be recorded

#### 3.2.3 Maintenance
- Assets shall be maintained according to vendor recommendations
- Security updates shall be applied per vulnerability management policy
- Hardware shall be serviced by authorized technicians
- Maintenance records shall be kept

#### 3.2.4 Retirement/Disposal
- Data shall be securely wiped before disposal
- Hardware shall be disposed of per environmental regulations
- Asset records shall be updated upon disposal
- Certificates of destruction shall be obtained for sensitive assets

### 3.3 Asset Ownership

- All assets shall have an assigned owner
- Owners are responsible for security and appropriate use of assets
- Owner responsibilities transfer when assets are reassigned
- Owners shall approve access to their assets
- Orphaned assets (no owner) shall be identified and assigned

### 3.4 Hardware Assets

#### 3.4.1 Laptops and Workstations
- Full disk encryption shall be enabled
- Endpoint protection shall be installed
- Automatic updates shall be enabled
- Device shall be registered in MDM (if applicable)
- Users shall report loss or theft immediately

#### 3.4.2 Servers
- Servers shall be placed in secure locations
- Access shall be limited to authorized personnel
- Server inventory shall include configuration details
- Virtual servers shall be tracked with host information

#### 3.4.3 Network Equipment
- Network devices shall be inventoried with location
- Default credentials shall be changed
- Firmware shall be kept current
- Configuration backups shall be maintained

#### 3.4.4 Mobile Devices
- Company-issued devices shall be tracked
- BYOD devices accessing company data shall be registered
- MDM enrollment shall be required
- Remote wipe capability shall be enabled

### 3.5 Software Assets

#### 3.5.1 License Management
- Software licenses shall be tracked in asset management system
- License counts shall not be exceeded
- License renewals shall be tracked
- Unused licenses shall be identified for optimization

#### 3.5.2 Software Inventory
- Installed software shall be inventoried
- Unauthorized software shall be detected and removed
- Software versions shall be tracked
- End-of-life software shall be identified

#### 3.5.3 Cloud Services
- Cloud subscriptions shall be inventoried
- Service agreements shall be documented
- Data location and processing shall be known
- Offboarding procedures shall be documented

### 3.6 Data Assets

- Data repositories shall be inventoried
- Data classification shall be assigned
- Data owners shall be designated
- Data flows shall be documented for sensitive data

### 3.7 Asset Classification

| Classification | Description | Security Requirements |
|----------------|-------------|----------------------|
| Critical | Essential to business operations | Enhanced security, priority support |
| Important | Supports key business functions | Standard security, defined SLA |
| Standard | General business use | Baseline security |
| Test/Development | Non-production use | Isolation from production |

## 4. Roles & Responsibilities

| Role | Responsibilities |
|------|-----------------|
| IT Asset Management | Maintain inventory, track lifecycle, generate reports |
| Procurement | Purchase assets through approved processes |
| IT Operations | Deploy, maintain, and retire assets |
| IT Security | Define security requirements, audit compliance |
| Asset Owners | Ensure appropriate use and security of assigned assets |
| All Users | Report asset issues, protect assigned assets |

## 5. Compliance

### 5.1 Monitoring
- Asset inventory accuracy shall be audited annually
- License compliance shall be reviewed quarterly
- Asset age and replacement needs shall be tracked
- Lost/stolen assets shall be tracked

### 5.2 Enforcement
- Unapproved assets may be removed from the network
- License violations shall be remediated immediately
- Asset policy violations may result in disciplinary action

## 6. Exceptions

- Exceptions to tracking requirements must be documented
- Short-term assets (less than 30 days) may have simplified tracking
- Exceptions shall be approved by IT Asset Management

## 7. Related Documents

- Information Security Policy (SEC-001)
- Acceptable Use Policy (IT-001)
- Data Classification Policy (COMP-005)
- Physical Security Policy (SEC-008)

## 8. Definitions

| Term | Definition |
|------|-----------|
| Asset | Any resource with value to the organization |
| Asset Tag | Unique identifier assigned to physical assets |
| Custodian | Person responsible for day-to-day care of an asset |
| MDM | Mobile Device Management - software for managing mobile devices |
| End-of-Life | Date when vendor support ends for an asset |

## 9. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | [Date] | [Author] | Initial release |
