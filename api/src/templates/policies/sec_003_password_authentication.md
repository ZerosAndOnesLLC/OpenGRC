# Password & Authentication Policy

**Policy Code:** SEC-003
**Version:** 1.0
**Effective Date:** [To be set]
**Review Date:** [Annual review recommended]
**Owner:** [To be assigned - typically IT Security Manager]
**Category:** Security

## 1. Purpose

This Password & Authentication Policy establishes requirements for user authentication to protect [Organization Name]'s information systems from unauthorized access. This policy defines password complexity requirements, multi-factor authentication standards, and authentication management procedures.

## 2. Scope

This policy applies to:
- All systems requiring user authentication
- All users including employees, contractors, and third parties
- All authentication methods including passwords, MFA, and biometrics
- Service accounts and API authentication
- Local and remote authentication

## 3. Policy Statements

### 3.1 Password Requirements

#### 3.1.1 Password Complexity
- Minimum length: 14 characters for standard users, 16 characters for privileged accounts
- Must contain at least three of the following: uppercase letters, lowercase letters, numbers, special characters
- Shall not contain the username, real name, or company name
- Shall not be a common dictionary word or easily guessable pattern
- Shall not match any of the previous 12 passwords

#### 3.1.2 Password Management
- Passwords shall be changed at least every 90 days
- Privileged account passwords shall be changed at least every 60 days
- Passwords must be changed immediately if compromise is suspected
- Password reuse is prohibited for 12 password generations
- Initial/temporary passwords must be changed upon first use

#### 3.1.3 Password Protection
- Passwords shall not be shared with any other person
- Passwords shall not be written down or stored in plain text
- Passwords shall not be transmitted via unencrypted channels
- Password managers are encouraged for storing complex passwords
- Passwords shall not be embedded in scripts or code

### 3.2 Multi-Factor Authentication (MFA)

#### 3.2.1 MFA Requirements
- MFA is required for all remote access
- MFA is required for all privileged access
- MFA is required for access to systems containing sensitive data
- MFA is required for cloud service administrative access
- MFA is strongly recommended for all user accounts

#### 3.2.2 Acceptable MFA Methods
- Hardware security keys (FIDO2/WebAuthn) - Preferred
- Authenticator applications (TOTP) - Acceptable
- Push notifications from approved applications - Acceptable
- SMS-based verification - Acceptable only as fallback, not for privileged access
- Biometric authentication - Acceptable when combined with another factor

#### 3.2.3 MFA Management
- Backup MFA methods shall be configured for account recovery
- Lost or compromised MFA devices shall be reported immediately
- MFA enrollment shall be completed within 48 hours of account creation
- MFA bypass codes shall be stored securely and used only for emergencies

### 3.3 Account Lockout

- Accounts shall lock after 5 consecutive failed authentication attempts
- Account lockout duration shall be at least 30 minutes
- Locked accounts may be manually unlocked by IT after identity verification
- Account lockout events shall be logged and monitored
- Repeated lockouts shall trigger security investigation

### 3.4 Session Management

- Sessions shall timeout after 15 minutes of inactivity for sensitive systems
- Users shall log out when leaving workstations unattended
- Concurrent sessions shall be limited based on system risk
- Session tokens shall be invalidated upon logout
- Re-authentication is required for sensitive transactions

### 3.5 Service Account Authentication

- Service accounts shall use strong, unique passwords or certificates
- Service account credentials shall be stored in approved secrets management systems
- Service account credentials shall be rotated at least annually
- API keys shall be unique per integration and rotatable
- Service accounts shall not use MFA but shall have enhanced monitoring

### 3.6 Single Sign-On (SSO)

- SSO shall be implemented where technically feasible
- SSO systems shall integrate with the corporate identity provider
- Applications not supporting SSO shall be documented
- SSO sessions shall be subject to session management requirements
- Break-glass procedures shall exist for SSO system failures

## 4. Roles & Responsibilities

| Role | Responsibilities |
|------|-----------------|
| IT Security | Define authentication standards, manage identity systems, monitor compliance |
| IT Operations | Implement authentication controls, manage account lifecycle, respond to lockouts |
| Users | Create strong passwords, protect credentials, enroll in MFA, report compromises |
| Managers | Ensure team compliance, approve access requiring enhanced authentication |
| Help Desk | Verify identity before unlocking accounts, assist with MFA enrollment |

## 5. Compliance

### 5.1 Monitoring
- Password policy compliance shall be enforced technically where possible
- MFA enrollment status shall be reported monthly
- Authentication failures shall be monitored and alerted
- Password age compliance shall be tracked and reported

### 5.2 Enforcement
- Systems shall enforce password complexity requirements
- Access shall be denied without required MFA
- Policy violations may result in disciplinary action
- Repeated violations shall trigger mandatory security training

## 6. Exceptions

- Exceptions for legacy systems must include remediation timeline
- Systems unable to support MFA must have compensating controls
- Exception requests require security risk assessment
- Exceptions expire annually and must be renewed

## 7. Related Documents

- Information Security Policy (SEC-001)
- Access Control Policy (SEC-002)
- Remote Work Policy (IT-006)
- Acceptable Use Policy (IT-001)

## 8. Definitions

| Term | Definition |
|------|-----------|
| Multi-Factor Authentication | Authentication requiring two or more independent credentials |
| TOTP | Time-based One-Time Password, generated by authenticator apps |
| FIDO2 | Fast Identity Online standard for passwordless authentication |
| SSO | Single Sign-On, allowing one login for multiple applications |
| Secrets Management | Secure storage and rotation of credentials and keys |

## 9. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | [Date] | [Author] | Initial release |
