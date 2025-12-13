# Access Control Policy

**Policy Code:** SEC-002
**Version:** 1.0
**Effective Date:** [To be set]
**Review Date:** [Annual review recommended]
**Owner:** [To be assigned - typically IT Security Manager]
**Category:** Security

## 1. Purpose

This Access Control Policy establishes requirements for controlling access to [Organization Name]'s information systems, applications, and data. This policy ensures that access is granted based on business need and the principle of least privilege, protecting organizational assets from unauthorized access.

## 2. Scope

This policy applies to:
- All information systems, applications, and databases
- All users including employees, contractors, and third parties
- All access methods including local, remote, and API access
- Physical access to facilities containing information systems
- Administrative, privileged, and standard user accounts

## 3. Policy Statements

### 3.1 Access Control Principles

- Access shall be granted based on the principle of least privilege
- Access rights shall be based on job function and business need
- Segregation of duties shall be implemented for critical functions
- Default access shall be "deny all" unless explicitly permitted
- Access shall be time-limited where appropriate

### 3.2 User Account Management

- Unique user accounts shall be assigned to each individual
- Shared accounts are prohibited except where technically required and documented
- Account provisioning shall require documented management approval
- User accounts shall be disabled within 24 hours of employment termination
- Accounts inactive for 90 days shall be automatically disabled
- Temporary accounts shall have defined expiration dates

### 3.3 Privileged Access

- Privileged access shall be limited to personnel with documented business need
- Administrative accounts shall be separate from standard user accounts
- Privileged account usage shall be logged and monitored
- Privileged access shall require enhanced authentication (MFA)
- Emergency access procedures shall be documented for break-glass scenarios
- Privileged access shall be reviewed quarterly

### 3.4 Access Reviews

- User access reviews shall be conducted at least quarterly
- Access reviews shall verify that access is still required and appropriate
- Access review findings shall be remediated within 30 days
- Access reviews shall be documented and retained for audit purposes
- Managers shall certify access for their direct reports

### 3.5 Remote Access

- Remote access shall require VPN or equivalent secure connection
- Multi-factor authentication is required for all remote access
- Remote access sessions shall time out after 15 minutes of inactivity
- Split tunneling is prohibited on corporate-managed devices
- Remote access rights shall be reviewed quarterly

### 3.6 Third-Party Access

- Third-party access requires documented business justification
- Third-party access shall be limited to specific systems and time periods
- Third-party access shall be monitored and logged
- Non-disclosure agreements are required before granting access
- Third-party accounts shall be disabled upon contract completion

### 3.7 Service Accounts

- Service accounts shall be inventoried and have documented owners
- Service account passwords shall be managed securely (vault or secrets manager)
- Service accounts shall have minimum required privileges
- Service accounts shall not be used for interactive logins
- Service account access shall be reviewed annually

## 4. Roles & Responsibilities

| Role | Responsibilities |
|------|-----------------|
| IT Security | Define access control standards, manage privileged accounts, conduct access reviews |
| IT Operations | Implement access controls, provision/deprovision accounts, maintain access logs |
| Managers | Approve access requests, certify access during reviews, report terminations |
| HR | Notify IT of onboarding/terminations, maintain employee status |
| Users | Request only necessary access, protect credentials, report suspicious activity |
| Internal Audit | Verify access control compliance, audit access review process |

## 5. Compliance

### 5.1 Monitoring
- Access logs shall be retained for a minimum of one year
- Failed login attempts shall be monitored and alerted
- Privileged account usage shall be reviewed weekly
- Access provisioning shall be tracked against approved requests

### 5.2 Enforcement
- Unauthorized access attempts shall be investigated
- Policy violations may result in disciplinary action
- Access shall be immediately revoked upon policy violation if warranted

## 6. Exceptions

- Exceptions must be approved by IT Security leadership
- Exceptions require documented business justification
- Compensating controls must be identified for approved exceptions
- Exceptions shall be reviewed quarterly and renewed annually

## 7. Related Documents

- Information Security Policy (SEC-001)
- Password & Authentication Policy (SEC-003)
- Remote Work Policy (IT-006)
- Acceptable Use Policy (IT-001)
- Vendor Management Policy (COMP-002)

## 8. Definitions

| Term | Definition |
|------|-----------|
| Least Privilege | Granting only the minimum access necessary to perform job functions |
| Privileged Access | Administrative or elevated access that bypasses normal security controls |
| Service Account | Non-interactive account used by applications or services |
| Access Review | Periodic verification that user access is appropriate |
| Break-Glass | Emergency access procedure for critical situations |

## 9. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | [Date] | [Author] | Initial release |
