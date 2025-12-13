# Mobile Device & BYOD Policy

**Policy Code:** IT-005
**Version:** 1.0
**Effective Date:** [To be set]
**Review Date:** [Annual review recommended]
**Owner:** [To be assigned - typically IT Security Manager]
**Category:** IT

## 1. Purpose

This Mobile Device & BYOD (Bring Your Own Device) Policy establishes requirements for securing mobile devices that access [Organization Name]'s data and systems. This policy ensures that both company-issued and personal devices meet security standards to protect organizational information.

## 2. Scope

This policy applies to:
- Company-issued mobile devices (smartphones, tablets)
- Personal devices used to access organizational data (BYOD)
- All employees, contractors, and third parties using mobile devices
- All applications and data accessed via mobile devices
- Mobile device management systems and configurations

## 3. Policy Statements

### 3.1 Device Eligibility

#### 3.1.1 Company-Issued Devices
- Company devices shall be approved models from the procurement list
- Devices shall be enrolled in Mobile Device Management (MDM)
- Users shall acknowledge device usage agreement
- Devices remain organization property

#### 3.1.2 BYOD Requirements
- Personal devices must meet minimum security requirements
- Supported operating systems: iOS [version] or later, Android [version] or later
- Devices must be capable of MDM enrollment
- Jailbroken or rooted devices are prohibited
- Users must agree to BYOD agreement

### 3.2 Security Requirements

#### 3.2.1 Device Security
- Screen lock is required (PIN, password, or biometric)
- Minimum PIN length: 6 digits
- Minimum password length: 8 characters
- Auto-lock timeout: Maximum 2 minutes
- Encryption shall be enabled (typically default on modern devices)
- Remote wipe capability must be enabled

#### 3.2.2 Authentication
- Multi-factor authentication is required for accessing organizational applications
- Biometric authentication is encouraged but must be backed by PIN/password
- Stored passwords must use the device keychain/credential manager
- Auto-fill from untrusted sources is prohibited

#### 3.2.3 Software Requirements
- Operating system must be within two major versions of current
- Security updates must be installed within 14 days of release
- Only applications from official app stores are permitted
- Organizational applications must be approved versions

### 3.3 Data Protection

#### 3.3.1 Data Access
- Confidential data may only be accessed through approved applications
- Data shall not be copied to unauthorized cloud services
- Screenshots/screen recording of confidential data should be avoided
- Data synchronization to unauthorized accounts is prohibited

#### 3.3.2 Data Storage
- Confidential data should be stored in containerized applications
- Local storage of confidential data should be minimized
- Backup to personal cloud accounts (for work data) is prohibited
- USB/external storage transfer may be restricted

### 3.4 Mobile Device Management (MDM)

#### 3.4.1 MDM Enrollment
- All devices accessing organizational data must be enrolled in MDM
- MDM profile must not be removed while accessing organizational data
- MDM enables security policy enforcement and remote management
- Removal of MDM profile may result in data wipe

#### 3.4.2 MDM Capabilities
MDM may enforce the following:
- Password/PIN requirements
- Encryption status verification
- App installation restrictions
- Network configuration
- Remote lock and wipe
- Location tracking (company devices only, with notice)

### 3.5 Lost or Stolen Devices

- Lost or stolen devices must be reported to IT immediately (within 1 hour)
- Report via: [helpdesk contact] or [emergency contact]
- IT will initiate remote lock and potential wipe
- Users must change passwords for any accounts accessed from the device
- Incident report must be filed
- Police report may be required for company devices

### 3.6 Acceptable Use

#### 3.6.1 Permitted Use
- Business communication and collaboration
- Access to approved business applications
- Remote work functions
- Limited personal use (company devices) per Acceptable Use Policy

#### 3.6.2 Prohibited Use
- Storage of inappropriate content
- Installation of unauthorized applications
- Connection to unauthorized networks for data access
- Circumventing security controls
- Using device while driving (except hands-free calling where legal)

### 3.7 Privacy

#### 3.7.1 Company Devices
- Company devices may be monitored for security compliance
- Location may be tracked for company devices
- All data on company devices is considered organizational property

#### 3.7.2 BYOD Devices
- MDM can see: Device type, OS version, installed apps, compliance status
- MDM cannot see: Personal email, messages, photos, browsing history
- Personal data is segregated from work data in containerized apps
- Work data may be selectively wiped without affecting personal data

### 3.8 Offboarding

- Upon termination, company devices must be returned immediately
- BYOD devices will have organizational data and profiles removed
- Access to organizational applications will be revoked
- Users acknowledge that work data will be wiped

## 4. Roles & Responsibilities

| Role | Responsibilities |
|------|-----------------|
| IT Security | Define security requirements, manage MDM policies |
| IT Support | Enroll devices, support users, respond to incidents |
| Users | Comply with policy, report issues, protect devices |
| Managers | Ensure team compliance, approve BYOD participation |
| HR | Include policy in onboarding, manage offboarding |

## 5. Compliance

### 5.1 Monitoring
- Device compliance is monitored via MDM
- Non-compliant devices may be blocked from accessing resources
- Compliance reports are generated monthly
- Security incidents are tracked and analyzed

### 5.2 Enforcement
- Non-compliant devices will be quarantined
- Repeated non-compliance may result in access revocation
- Policy violations may result in disciplinary action
- Severe violations may result in device wipe

## 6. Exceptions

- Exceptions must be approved by IT Security
- Alternative controls must be documented
- Exceptions are reviewed quarterly
- Medical device exceptions may be accommodated

## 7. Related Documents

- Information Security Policy (SEC-001)
- Acceptable Use Policy (IT-001)
- Remote Work Policy (IT-006)
- Data Classification Policy (COMP-005)

## 8. Definitions

| Term | Definition |
|------|-----------|
| BYOD | Bring Your Own Device - using personal devices for work |
| MDM | Mobile Device Management - software for securing and managing mobile devices |
| Containerization | Separating work data from personal data on devices |
| Jailbroken/Rooted | Devices with removed manufacturer security restrictions |
| Remote Wipe | Ability to erase device data remotely |

## 9. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | [Date] | [Author] | Initial release |
