# OpenGRC - Open Source Compliance Platform

## Vision

Build the most comprehensive open-source GRC (Governance, Risk, and Compliance) platform that makes SOC 2, ISO 27001, HIPAA, and other framework compliance accessible to every organization. Eliminate the $50k+/year barrier to entry that paid platforms create.

## Why We Win

| Paid Platforms | OpenGRC |
|----------------|---------|
| $10k-80k/year | Free & self-hosted |
| Vendor lock-in | Own your data |
| Limited customization | Fully extensible |
| Closed integrations | Open integration framework |
| Per-seat pricing | Unlimited users |
| Black-box scoring | Transparent algorithms |

## Tech Stack

- **API**: Rust (Axum) - Performance, safety, async-first
- **UI**: Next.js 14 (App Router) - Modern React, SSR, great DX
- **Database**: PostgreSQL - Reliable, scalable, rich features
- **Cache**: Redis - Session management, job queues, caching
- **Auth**: TitaniumVault - Dogfooding our own auth
- **Search**: Meilisearch - Fast full-text search for evidence/policies
- **Storage**: Local filesystem or S3-compatible - Evidence file storage
- **Queue**: Redis + background workers - Async job processing

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        OpenGRC Platform                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Next.js    │  │   Rust API   │  │   Workers    │          │
│  │     UI       │  │    (Axum)    │  │   (Rust)     │          │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘          │
│         │                 │                 │                    │
│         └────────────────┼─────────────────┘                    │
│                          │                                       │
│  ┌───────────────────────┼───────────────────────┐              │
│  │                       ▼                       │              │
│  │  ┌─────────┐  ┌─────────────┐  ┌─────────┐   │              │
│  │  │ Postgres│  │    Redis    │  │   S3    │   │              │
│  │  │         │  │ Cache/Queue │  │ Storage │   │              │
│  │  └─────────┘  └─────────────┘  └─────────┘   │              │
│  │                    Data Layer                 │              │
│  └───────────────────────────────────────────────┘              │
│                                                                  │
│  ┌───────────────────────────────────────────────┐              │
│  │              Integration Layer                │              │
│  │  ┌─────┐ ┌──────┐ ┌────┐ ┌────┐ ┌─────────┐  │              │
│  │  │ AWS │ │GitHub│ │Okta│ │Jira│ │ 50+ more│  │              │
│  │  └─────┘ └──────┘ └────┘ └────┘ └─────────┘  │              │
│  └───────────────────────────────────────────────┘              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Database Schema (Core Entities)

### Organizations & Users
```sql
-- Multi-tenant foundation
organizations (
    id, name, slug, settings, subscription_tier,
    created_at, updated_at
)

-- Users managed via TitaniumVault, local cache for app data
users (
    id, organization_id, tv_user_id, email, name, role,
    last_login_at, created_at
)
```

### Frameworks & Controls
```sql
-- Compliance frameworks (SOC 2, ISO 27001, HIPAA, etc.)
frameworks (
    id, name, version, description, category,
    is_system, -- true for built-in frameworks
    created_at
)

-- Framework requirements/criteria
framework_requirements (
    id, framework_id, code, name, description,
    category, parent_id, sort_order
)

-- Organization's controls
controls (
    id, organization_id, code, name, description,
    control_type, -- preventive, detective, corrective
    frequency, -- continuous, daily, weekly, monthly, quarterly, annual
    owner_id, status, implementation_notes,
    created_at, updated_at
)

-- Map controls to framework requirements (many-to-many)
control_requirement_mappings (
    id, control_id, framework_requirement_id,
    created_at
)

-- Control test definitions
control_tests (
    id, control_id, name, description,
    test_type, -- manual, automated
    automation_config, -- JSON for automated test config
    frequency, next_due_at,
    created_at
)

-- Control test results
control_test_results (
    id, control_test_id, performed_by, performed_at,
    status, -- passed, failed, not_applicable
    notes, evidence_ids,
    created_at
)
```

### Evidence Management
```sql
-- Evidence records
evidence (
    id, organization_id, title, description,
    evidence_type, -- document, screenshot, log, automated
    source, -- manual, aws, github, okta, etc.
    source_reference, -- external ID/URL
    file_path, file_size, mime_type,
    collected_at, valid_from, valid_until,
    uploaded_by, created_at
)

-- Link evidence to controls
evidence_control_links (
    id, evidence_id, control_id, control_test_result_id,
    linked_by, linked_at
)

-- Evidence collection tasks (for automated collection)
evidence_collection_tasks (
    id, organization_id, integration_id,
    name, description, schedule, -- cron expression
    collection_config, -- JSON defining what to collect
    last_run_at, next_run_at, status,
    created_at
)
```

### Policy Management
```sql
-- Policies
policies (
    id, organization_id, code, title,
    category, -- security, hr, it, privacy, etc.
    content, -- markdown
    version, status, -- draft, published, archived
    owner_id, approver_id, approved_at,
    effective_date, review_date,
    created_at, updated_at
)

-- Policy versions (full history)
policy_versions (
    id, policy_id, version, content,
    changed_by, change_summary,
    created_at
)

-- Policy acknowledgments
policy_acknowledgments (
    id, policy_id, policy_version, user_id,
    acknowledged_at, ip_address
)

-- Link policies to controls
policy_control_links (
    id, policy_id, control_id,
    created_at
)
```

### Risk Management
```sql
-- Risk register
risks (
    id, organization_id, code, title, description,
    category, -- operational, security, compliance, financial, strategic
    source, -- internal, external, third_party
    likelihood, -- 1-5
    impact, -- 1-5
    inherent_score, -- likelihood * impact
    residual_likelihood, residual_impact, residual_score,
    status, -- identified, assessed, treating, accepted, closed
    owner_id, treatment_plan,
    identified_at, review_date,
    created_at, updated_at
)

-- Risk to control mapping
risk_control_mappings (
    id, risk_id, control_id,
    effectiveness, -- full, partial, minimal
    created_at
)

-- Risk assessments history
risk_assessments (
    id, risk_id, assessed_by,
    likelihood, impact, score,
    notes, created_at
)
```

### Vendor Management
```sql
-- Vendors
vendors (
    id, organization_id, name, description,
    category, -- saas, infrastructure, consulting, etc.
    criticality, -- critical, high, medium, low
    data_classification, -- what data do they access
    status, -- active, inactive, under_review
    contract_start, contract_end,
    owner_id, website,
    created_at, updated_at
)

-- Vendor assessments
vendor_assessments (
    id, vendor_id, assessment_type, -- initial, annual, incident
    assessed_by, assessed_at,
    risk_rating, -- high, medium, low
    findings, recommendations,
    next_assessment_date,
    created_at
)

-- Vendor documents (SOC 2 reports, questionnaires, etc.)
vendor_documents (
    id, vendor_id, document_type,
    title, file_path, valid_from, valid_until,
    uploaded_by, created_at
)
```

### Asset Management
```sql
-- Assets
assets (
    id, organization_id, name, description,
    asset_type, -- hardware, software, data, service
    category, -- server, workstation, database, application, etc.
    classification, -- public, internal, confidential, restricted
    status, -- active, inactive, decommissioned
    owner_id, custodian_id,
    location, ip_address, mac_address,
    purchase_date, warranty_until,
    metadata, -- JSON for type-specific fields
    created_at, updated_at
)

-- Asset to control mapping
asset_control_mappings (
    id, asset_id, control_id,
    created_at
)
```

### User Access Reviews
```sql
-- Access review campaigns
access_review_campaigns (
    id, organization_id, name, description,
    integration_id, -- which system to review
    status, -- draft, active, completed, cancelled
    started_at, due_at, completed_at,
    created_by, created_at
)

-- Individual access review items
access_review_items (
    id, campaign_id, user_identifier,
    user_name, user_email,
    access_details, -- JSON describing current access
    reviewer_id, review_status, -- pending, approved, revoked, flagged
    reviewed_at, review_notes,
    created_at
)
```

### Integrations
```sql
-- Integration configurations
integrations (
    id, organization_id, integration_type,
    name, config, -- encrypted JSON
    status, -- active, inactive, error
    last_sync_at, last_error,
    created_at, updated_at
)

-- Integration sync logs
integration_sync_logs (
    id, integration_id, sync_type,
    started_at, completed_at,
    status, records_processed, errors,
    created_at
)
```

### Audit & Reporting
```sql
-- Audit preparation (for working with external auditors)
audits (
    id, organization_id, name,
    framework_id, audit_type, -- type1, type2, certification
    auditor_firm, auditor_contact,
    period_start, period_end,
    status, -- planning, fieldwork, review, completed
    created_at, updated_at
)

-- Auditor requests
audit_requests (
    id, audit_id, request_type,
    title, description,
    status, -- open, in_progress, submitted, accepted, rejected
    assigned_to, due_at,
    created_at, updated_at
)

-- Audit request responses
audit_request_responses (
    id, audit_request_id, response_text,
    evidence_ids, responded_by, responded_at,
    created_at
)

-- Audit findings
audit_findings (
    id, audit_id, finding_type, -- observation, exception, deficiency
    title, description, recommendation,
    control_ids, status, -- open, remediation, closed
    remediation_plan, remediation_due,
    created_at, updated_at
)

-- Activity/audit log
activity_logs (
    id, organization_id, user_id,
    action, entity_type, entity_id,
    old_values, new_values,
    ip_address, user_agent,
    created_at
)
```

### Tasks & Workflows
```sql
-- Tasks
tasks (
    id, organization_id, title, description,
    task_type, -- control_test, evidence_collection, review, remediation
    related_entity_type, related_entity_id,
    assignee_id, due_at, completed_at,
    status, -- open, in_progress, completed, overdue
    priority, -- low, medium, high, critical
    created_by, created_at, updated_at
)

-- Task comments
task_comments (
    id, task_id, user_id, content,
    created_at
)

-- Notifications
notifications (
    id, organization_id, user_id,
    notification_type, title, message,
    related_entity_type, related_entity_id,
    read_at, created_at
)
```

## Module Breakdown

### Phase 1: Foundation (MVP)
**Goal: Basic compliance management that's immediately useful**

#### 1.1 Core Platform
- [x] Project scaffolding (Rust API + Next.js UI)
- [x] Database setup with migrations
- [x] TitaniumVault authentication integration
- [x] Multi-tenant architecture
- [x] Role-based access control (Admin, Compliance Manager, Contributor, Viewer, Auditor)
- [x] Activity logging

#### 1.2 Framework Management
- [x] Pre-loaded SOC 2 Trust Service Criteria
- [x] Framework requirement browser
- [x] Custom framework creation with bulk import (CSV/JSON)
- [x] Control library with templates
- [x] Control-to-requirement mapping
- [x] Gap analysis dashboard

#### 1.3 Evidence Management
- [x] Evidence upload (drag & drop, bulk)
- [x] Evidence metadata and tagging
- [x] Link evidence to controls
- [x] Evidence expiration tracking
- [x] Full-text search (Meilisearch)
- [x] Version history

#### 1.4 Policy Management
- [x] Policy editor (Markdown with preview)
- [x] Policy templates (25 common policies)
- [x] Version control with diff view
- [x] Approval workflow
- [x] Employee acknowledgment portal (/policies/pending)
- [x] Acknowledgment tracking & reminders (email via SES + in-app notifications)

#### 1.5 Risk Register
- [x] Risk CRUD with scoring matrix
- [x] Risk categories and templates
- [x] Risk-to-control mapping
- [x] Treatment plan tracking
- [x] Risk heatmap visualization
- [x] Risk assessment history

#### 1.6 Dashboard & Reporting
- [x] Compliance posture dashboard
- [x] Control health overview
- [x] Upcoming tasks/deadlines
- [x] Framework coverage reports (CSV export)
- [x] PDF export with branding, headers/footers, and summary charts

### Phase 2: Automation & Integrations
**Goal: Reduce manual work by 80%**

#### 2.1 Integration Framework (Complete - v1.7.0)
- [x] Pluggable integration architecture (IntegrationProvider trait)
- [x] Integration CRUD with caching and cache invalidation
- [x] Available integrations catalog (13 providers defined)
- [x] Integration management UI with stats cards
- [x] Sync triggering and sync log tracking
- [x] Sensitive credential masking in API responses
- [x] Credential vault (AES-256-GCM encrypted storage)
- [x] OAuth2 connection flow
- [ ] Sync scheduling (cron-based worker) - DEFERRED
- [x] Error handling & retry logic
- [x] Integration health monitoring dashboard

#### 2.2 Cloud Provider Integrations

##### AWS Integration (Detailed Plan)

**Authentication Strategy**
```
┌─────────────────────────────────────────────────────────────────┐
│                    AWS Authentication Options                    │
├─────────────────────────────────────────────────────────────────┤
│  Option 1: Cross-Account IAM Role (Recommended)                 │
│  ┌──────────────┐     ┌──────────────┐     ┌──────────────┐    │
│  │   OpenGRC    │────▶│  STS Assume  │────▶│  Customer    │    │
│  │   AWS Acct   │     │     Role     │     │   AWS Acct   │    │
│  └──────────────┘     └──────────────┘     └──────────────┘    │
│                                                                  │
│  Option 2: IAM Access Keys (For self-hosted)                    │
│  ┌──────────────┐     ┌──────────────┐                         │
│  │   OpenGRC    │────▶│  Customer    │                         │
│  │   Instance   │     │   AWS Acct   │                         │
│  └──────────────┘     └──────────────┘                         │
└─────────────────────────────────────────────────────────────────┘
```

**Configuration Schema**
```json
{
  "auth_method": "assume_role | access_keys",
  "role_arn": "arn:aws:iam::123456789012:role/OpenGRCReadOnly",
  "external_id": "optional-external-id-for-security",
  "access_key_id": "AKIA...",
  "secret_access_key": "encrypted...",
  "regions": ["us-east-1", "us-west-2"],
  "services": {
    "iam": { "enabled": true },
    "cloudtrail": { "enabled": true, "trail_name": "main-trail" },
    "securityhub": { "enabled": true },
    "config": { "enabled": true },
    "s3": { "enabled": true, "buckets": [] },
    "ec2": { "enabled": true },
    "rds": { "enabled": true }
  },
  "sync_options": {
    "full_sync_schedule": "0 2 * * *",
    "incremental_hours": 24
  }
}
```

**Required AWS SDK Crates** (all free, Apache 2.0)
```toml
aws-sdk-iam = "1.x"
aws-sdk-cloudtrail = "1.x"
aws-sdk-securityhub = "1.x"
aws-sdk-config = "1.x"
aws-sdk-ec2 = "1.x"
aws-sdk-rds = "1.x"
aws-sdk-sts = "1.x"
aws-sdk-organizations = "1.x"
```

**Service Implementations**

| Service | Capability | Data Collected | Control Mappings |
|---------|------------|----------------|------------------|
| IAM | UserSync, AccessSync | Users, roles, policies, MFA status, access keys | CC6.1, CC6.2, CC6.3 |
| CloudTrail | AuditLogs | Management events, data events | CC4.1, CC7.2, CC7.3 |
| Security Hub | SecurityFindings | Findings, compliance standards | CC3.2, CC7.1, CC7.2 |
| Config | ComplianceStatus | Rule compliance, config snapshots | CC6.1, CC7.1, CC8.1 |
| S3 | ConfigurationState | Bucket policies, encryption, versioning | CC6.1, CC6.6, CC6.7 |
| EC2 | AssetInventory | Instances, security groups, VPCs | A1.1, CC6.1, CC6.6 |
| RDS | AssetInventory | Databases, encryption, backups | CC6.1, CC6.6, CC6.7 |

---

**1. IAM Integration** (`aws/iam.rs`)

*Capabilities*: `UserSync`, `AccessSync`, `ConfigurationState`

*API Calls*:
```rust
// Users
iam.list_users() -> User[]
iam.get_user(username) -> User
iam.list_mfa_devices(username) -> MFADevice[]
iam.list_access_keys(username) -> AccessKeyMetadata[]
iam.get_access_key_last_used(access_key_id) -> AccessKeyLastUsed

// Roles
iam.list_roles() -> Role[]
iam.get_role(role_name) -> Role
iam.list_role_policies(role_name) -> PolicyName[]
iam.list_attached_role_policies(role_name) -> AttachedPolicy[]

// Policies
iam.list_policies(scope: LOCAL) -> Policy[]
iam.get_policy(policy_arn) -> Policy
iam.get_policy_version(policy_arn, version) -> PolicyDocument

// Groups
iam.list_groups() -> Group[]
iam.get_group(group_name) -> Group (with users)
iam.list_group_policies(group_name) -> PolicyName[]
```

*Evidence Collected*:
| Evidence Type | Description | Control Codes |
|---------------|-------------|---------------|
| IAM User Inventory | List of all IAM users with MFA status | CC6.1, CC6.2 |
| IAM Roles Inventory | Roles and their trust policies | CC6.1, CC6.3 |
| IAM Policy Analysis | Custom policies with permissions | CC6.1, CC6.3 |
| Access Key Age Report | Keys older than 90 days | CC6.1, CC6.2 |
| MFA Compliance Report | Users without MFA enabled | CC6.1, CC6.6 |
| Unused Credentials Report | Users with no recent activity | CC6.2, CC6.3 |
| Admin Privileges Report | Users/roles with admin access | CC6.1, CC6.3 |

*Data Model*:
```rust
pub struct AwsIamUser {
    pub user_id: String,
    pub user_name: String,
    pub arn: String,
    pub create_date: DateTime<Utc>,
    pub password_last_used: Option<DateTime<Utc>>,
    pub mfa_enabled: bool,
    pub mfa_devices: Vec<AwsMfaDevice>,
    pub access_keys: Vec<AwsAccessKey>,
    pub groups: Vec<String>,
    pub inline_policies: Vec<String>,
    pub attached_policies: Vec<AwsAttachedPolicy>,
    pub tags: HashMap<String, String>,
}

pub struct AwsIamRole {
    pub role_id: String,
    pub role_name: String,
    pub arn: String,
    pub path: String,
    pub assume_role_policy: Value,  // Trust policy
    pub description: Option<String>,
    pub max_session_duration: i32,
    pub create_date: DateTime<Utc>,
    pub inline_policies: Vec<String>,
    pub attached_policies: Vec<AwsAttachedPolicy>,
    pub last_used: Option<RoleLastUsed>,
    pub tags: HashMap<String, String>,
}

pub struct AwsAccessKey {
    pub access_key_id: String,
    pub status: AccessKeyStatus,  // Active | Inactive
    pub create_date: DateTime<Utc>,
    pub last_used_date: Option<DateTime<Utc>>,
    pub last_used_service: Option<String>,
    pub last_used_region: Option<String>,
}
```

*Compliance Checks*:
```rust
pub struct IamComplianceChecks;

impl IamComplianceChecks {
    /// CC6.1 - Root account should have MFA
    fn check_root_mfa(summary: &CredentialReport) -> ComplianceResult;

    /// CC6.2 - Access keys should be rotated within 90 days
    fn check_access_key_rotation(keys: &[AwsAccessKey]) -> ComplianceResult;

    /// CC6.1 - All human users should have MFA
    fn check_user_mfa(users: &[AwsIamUser]) -> ComplianceResult;

    /// CC6.3 - No inline policies on users (use groups)
    fn check_no_user_inline_policies(users: &[AwsIamUser]) -> ComplianceResult;

    /// CC6.1 - No wildcard (*) in IAM policies
    fn check_no_wildcard_permissions(policies: &[PolicyDocument]) -> ComplianceResult;

    /// CC6.2 - Unused credentials should be disabled
    fn check_unused_credentials(users: &[AwsIamUser], days: u32) -> ComplianceResult;
}
```

---

**2. CloudTrail Integration** (`aws/cloudtrail.rs`)

*Capabilities*: `AuditLogs`, `ComplianceStatus`

*API Calls*:
```rust
// Trail configuration
cloudtrail.describe_trails() -> TrailInfo[]
cloudtrail.get_trail_status(trail_name) -> TrailStatus

// Events (for evidence collection)
cloudtrail.lookup_events(
    start_time, end_time,
    lookup_attributes: [EventCategory, EventName, ResourceType],
    max_results
) -> Event[]

// For real-time: CloudTrail Lake or S3 event notifications
cloudtrail.list_event_data_stores() -> EventDataStore[]
cloudtrail.start_query(query_statement) -> QueryId
```

*Evidence Collected*:
| Evidence Type | Description | Control Codes |
|---------------|-------------|---------------|
| CloudTrail Configuration | Trail settings and status | CC4.1, CC7.2 |
| Management Events | API activity for period | CC7.2, CC7.3 |
| Console Sign-in Events | User authentication events | CC6.1, CC7.2 |
| IAM Changes | User/role/policy modifications | CC6.1, CC6.2 |
| Security Group Changes | Network security modifications | CC6.6, CC6.7 |
| S3 Access Logs | Data access events (if enabled) | CC6.6, CC7.3 |

*Data Model*:
```rust
pub struct AwsCloudTrailEvent {
    pub event_id: String,
    pub event_name: String,
    pub event_time: DateTime<Utc>,
    pub event_source: String,
    pub user_identity: UserIdentity,
    pub source_ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub aws_region: String,
    pub resources: Vec<Resource>,
    pub request_parameters: Option<Value>,
    pub response_elements: Option<Value>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub read_only: bool,
    pub management_event: bool,
}

pub struct UserIdentity {
    pub identity_type: String,  // Root, IAMUser, AssumedRole, etc.
    pub principal_id: String,
    pub arn: Option<String>,
    pub account_id: String,
    pub user_name: Option<String>,
    pub session_context: Option<SessionContext>,
}
```

*Event Categories for Compliance*:
```rust
pub enum CloudTrailEventCategory {
    // Authentication & Access
    ConsoleLogin,
    AssumeRole,
    GetSessionToken,

    // IAM Changes (CC6.1, CC6.2)
    CreateUser,
    DeleteUser,
    CreateRole,
    AttachUserPolicy,
    CreateAccessKey,
    UpdateLoginProfile,

    // Security Configuration (CC6.6, CC7.1)
    AuthorizeSecurityGroupIngress,
    CreateSecurityGroup,
    ModifyVpcAttribute,
    CreateNetworkAcl,

    // Data Access (CC6.6, CC7.3)
    GetObject,
    PutObject,
    DeleteObject,

    // Resource Changes (CC8.1)
    RunInstances,
    TerminateInstances,
    CreateDBInstance,
    ModifyDBInstance,
}
```

---

**3. Security Hub Integration** (`aws/securityhub.rs`)

*Capabilities*: `SecurityFindings`, `ComplianceStatus`

*API Calls*:
```rust
// Standards
securityhub.get_enabled_standards() -> StandardsSubscription[]
securityhub.describe_standards_controls(subscription_arn) -> StandardsControl[]

// Findings
securityhub.get_findings(
    filters: {
        severity_label, workflow_status, compliance_status,
        product_name, record_state, resource_type
    },
    sort_criteria,
    max_results
) -> AwsSecurityFinding[]

// Aggregation
securityhub.get_insight_results(insight_arn) -> InsightResults
```

*Evidence Collected*:
| Evidence Type | Description | Control Codes |
|---------------|-------------|---------------|
| Security Hub Enabled Standards | CIS, AWS Best Practices, PCI DSS | CC3.2, CC7.1 |
| Critical/High Findings | Active security issues | CC7.1, CC7.2 |
| Compliance Summary | Pass/fail by standard | CC3.2, CC7.1 |
| Finding Trends | New vs resolved over time | CC7.2, CC4.1 |
| Resource Findings Map | Findings by resource | CC7.1, A1.1 |

*Data Model*:
```rust
pub struct AwsSecurityHubFinding {
    pub id: String,
    pub schema_version: String,
    pub product_arn: String,
    pub generator_id: String,
    pub aws_account_id: String,
    pub types: Vec<String>,
    pub title: String,
    pub description: String,
    pub severity: Severity,
    pub confidence: Option<i32>,
    pub criticality: Option<i32>,
    pub remediation: Option<Remediation>,
    pub resources: Vec<Resource>,
    pub compliance: Option<Compliance>,
    pub workflow: WorkflowState,
    pub record_state: RecordState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Severity {
    pub label: SeverityLabel,  // INFORMATIONAL, LOW, MEDIUM, HIGH, CRITICAL
    pub normalized: i32,       // 0-100
}

pub struct Compliance {
    pub status: ComplianceStatus,  // PASSED, WARNING, FAILED, NOT_AVAILABLE
    pub related_requirements: Vec<String>,
    pub status_reasons: Vec<StatusReason>,
}
```

*Standard Mappings*:
```rust
pub struct SecurityHubStandardMappings;

impl SecurityHubStandardMappings {
    // AWS Foundational Security Best Practices → SOC 2
    const AWS_FSBP_TO_SOC2: &[(&str, &str)] = &[
        ("IAM.1", "CC6.1"),   // MFA for root
        ("IAM.4", "CC6.2"),   // Root access key
        ("IAM.6", "CC6.1"),   // Hardware MFA for root
        ("EC2.2", "CC6.6"),   // Default SG no traffic
        ("S3.1", "CC6.7"),    // Block public access
        ("S3.4", "CC6.6"),    // Encryption at rest
        ("RDS.3", "CC6.6"),   // Encryption at rest
        ("CloudTrail.1", "CC7.2"),  // Enabled
        ("CloudTrail.2", "CC7.2"),  // Encryption
        // ... more mappings
    ];

    // CIS AWS Foundations Benchmark → SOC 2
    const CIS_TO_SOC2: &[(&str, &str)] = &[
        ("1.1", "CC6.1"),     // Root MFA
        ("1.2", "CC6.2"),     // MFA enabled
        ("1.4", "CC6.2"),     // Access key rotation
        ("2.1", "CC7.2"),     // CloudTrail enabled
        ("2.3", "CC6.6"),     // S3 bucket for logs
        ("3.1", "CC7.2"),     // Log metric: unauthorized API
        // ... more mappings
    ];
}
```

---

**4. AWS Config Integration** (`aws/config.rs`)

*Capabilities*: `ComplianceStatus`, `ConfigurationState`

*API Calls*:
```rust
// Rules
config.describe_config_rules() -> ConfigRule[]
config.describe_compliance_by_config_rule() -> ComplianceByConfigRule[]

// Resources
config.list_discovered_resources(resource_type) -> ResourceIdentifier[]
config.get_resource_config_history(resource_type, resource_id) -> ConfigurationItem[]

// Compliance
config.get_compliance_details_by_config_rule(rule_name) -> EvaluationResult[]
config.get_compliance_summary_by_resource_type() -> ComplianceSummary[]

// Conformance Packs
config.describe_conformance_packs() -> ConformancePack[]
config.get_conformance_pack_compliance_summary() -> ConformancePackComplianceSummary[]
```

*Evidence Collected*:
| Evidence Type | Description | Control Codes |
|---------------|-------------|---------------|
| Config Rules Compliance | Rule evaluations | CC3.2, CC7.1 |
| Resource Configuration | Current state snapshots | CC6.1, CC8.1 |
| Configuration Changes | Change history | CC8.1, CC7.2 |
| Conformance Pack Status | Pack-level compliance | CC3.2, CC7.1 |
| Non-Compliant Resources | Failing resources | CC7.1, CC7.2 |

*Data Model*:
```rust
pub struct AwsConfigRuleCompliance {
    pub config_rule_name: String,
    pub config_rule_arn: String,
    pub compliance_type: ComplianceType,  // COMPLIANT, NON_COMPLIANT, etc.
    pub annotation: Option<String>,
    pub evaluation_results: Vec<EvaluationResult>,
}

pub struct EvaluationResult {
    pub resource_type: String,
    pub resource_id: String,
    pub compliance_type: ComplianceType,
    pub result_recorded_time: DateTime<Utc>,
    pub config_rule_invoked_time: DateTime<Utc>,
    pub annotation: Option<String>,
}

pub struct AwsConfigurationItem {
    pub resource_type: String,
    pub resource_id: String,
    pub resource_name: Option<String>,
    pub arn: Option<String>,
    pub aws_region: String,
    pub configuration_state_id: String,
    pub configuration: Value,
    pub configuration_item_capture_time: DateTime<Utc>,
    pub configuration_item_status: ConfigurationItemStatus,
    pub related_events: Vec<String>,
    pub relationships: Vec<Relationship>,
    pub tags: HashMap<String, String>,
}
```

*Config Rule to Control Mappings*:
```rust
pub struct AwsConfigRuleMappings;

impl AwsConfigRuleMappings {
    const MANAGED_RULES_TO_SOC2: &[(&str, &str)] = &[
        // Identity & Access
        ("iam-user-mfa-enabled", "CC6.1"),
        ("iam-root-access-key-check", "CC6.2"),
        ("iam-password-policy", "CC6.1"),
        ("access-keys-rotated", "CC6.2"),

        // Encryption
        ("encrypted-volumes", "CC6.6"),
        ("rds-storage-encrypted", "CC6.6"),
        ("s3-bucket-server-side-encryption-enabled", "CC6.6"),
        ("s3-bucket-ssl-requests-only", "CC6.6"),

        // Logging & Monitoring
        ("cloudtrail-enabled", "CC7.2"),
        ("cloud-trail-encryption-enabled", "CC7.2"),
        ("cloudwatch-log-group-encrypted", "CC7.2"),
        ("vpc-flow-logs-enabled", "CC7.2"),

        // Network Security
        ("restricted-ssh", "CC6.6"),
        ("vpc-default-security-group-closed", "CC6.6"),
        ("ec2-instance-no-public-ip", "CC6.6"),

        // Backup & Recovery
        ("rds-instance-deletion-protection-enabled", "A1.2"),
        ("db-instance-backup-enabled", "A1.2"),
        ("dynamodb-pitr-enabled", "A1.2"),

        // More...
    ];
}
```

---

**5. S3 Integration** (`aws/s3.rs`)

*Capabilities*: `ConfigurationState`, `ComplianceStatus`

*API Calls*:
```rust
// Bucket listing
s3.list_buckets() -> Bucket[]

// Bucket configuration (per bucket)
s3.get_bucket_encryption(bucket) -> ServerSideEncryptionConfiguration
s3.get_bucket_versioning(bucket) -> VersioningConfiguration
s3.get_bucket_logging(bucket) -> LoggingEnabled
s3.get_bucket_policy(bucket) -> Policy
s3.get_bucket_acl(bucket) -> AccessControlPolicy
s3.get_public_access_block(bucket) -> PublicAccessBlockConfiguration
s3.get_bucket_lifecycle_configuration(bucket) -> LifecycleConfiguration
s3.get_bucket_replication(bucket) -> ReplicationConfiguration
s3.get_bucket_tagging(bucket) -> Tagging
```

*Evidence Collected*:
| Evidence Type | Description | Control Codes |
|---------------|-------------|---------------|
| Bucket Inventory | All buckets with regions | A1.1, CC6.6 |
| Encryption Status | SSE-S3, SSE-KMS, or none | CC6.6, CC6.7 |
| Public Access Status | Block public access settings | CC6.6, CC6.7 |
| Versioning Status | Enabled/disabled | A1.2, CC6.1 |
| Logging Status | Access logging config | CC7.2, CC7.3 |
| Bucket Policies | IAM policies on buckets | CC6.1, CC6.6 |
| Lifecycle Rules | Data retention policies | PI1.3, C1.1 |

*Data Model*:
```rust
pub struct AwsS3Bucket {
    pub name: String,
    pub region: String,
    pub creation_date: DateTime<Utc>,
    pub encryption: Option<BucketEncryption>,
    pub versioning: VersioningStatus,
    pub logging: Option<LoggingConfig>,
    pub public_access_block: Option<PublicAccessBlockConfig>,
    pub policy: Option<Value>,
    pub acl: BucketAcl,
    pub lifecycle_rules: Vec<LifecycleRule>,
    pub replication: Option<ReplicationConfig>,
    pub tags: HashMap<String, String>,
    pub compliance_checks: S3ComplianceResults,
}

pub struct S3ComplianceResults {
    pub is_encrypted: bool,
    pub is_public: bool,
    pub versioning_enabled: bool,
    pub logging_enabled: bool,
    pub public_access_blocked: bool,
    pub ssl_enforced: bool,
    pub has_lifecycle_policy: bool,
}
```

---

**6. EC2 Integration** (`aws/ec2.rs`)

*Capabilities*: `AssetInventory`, `ConfigurationState`

*API Calls*:
```rust
// Instances
ec2.describe_instances() -> Reservation[]
ec2.describe_instance_status() -> InstanceStatus[]

// Security Groups
ec2.describe_security_groups() -> SecurityGroup[]

// VPCs & Networking
ec2.describe_vpcs() -> Vpc[]
ec2.describe_subnets() -> Subnet[]
ec2.describe_network_acls() -> NetworkAcl[]
ec2.describe_route_tables() -> RouteTable[]
ec2.describe_internet_gateways() -> InternetGateway[]
ec2.describe_nat_gateways() -> NatGateway[]

// Volumes
ec2.describe_volumes() -> Volume[]
ec2.describe_snapshots(owner: self) -> Snapshot[]

// AMIs
ec2.describe_images(owner: self) -> Image[]
```

*Evidence Collected*:
| Evidence Type | Description | Control Codes |
|---------------|-------------|---------------|
| EC2 Instance Inventory | All instances with details | A1.1, CC6.1 |
| Security Group Rules | Inbound/outbound rules | CC6.6, CC6.7 |
| VPC Configuration | Network architecture | CC6.6, CC6.7 |
| EBS Encryption Status | Volume encryption | CC6.6 |
| Public IP Inventory | Instances with public IPs | CC6.6, CC6.7 |
| AMI Inventory | Custom machine images | CC8.1 |
| Snapshot Inventory | Backup snapshots | A1.2 |

*Data Model*:
```rust
pub struct AwsEc2Instance {
    pub instance_id: String,
    pub instance_type: String,
    pub state: InstanceState,
    pub launch_time: DateTime<Utc>,
    pub availability_zone: String,
    pub vpc_id: Option<String>,
    pub subnet_id: Option<String>,
    pub private_ip_address: Option<String>,
    pub public_ip_address: Option<String>,
    pub iam_instance_profile: Option<IamInstanceProfile>,
    pub security_groups: Vec<GroupIdentifier>,
    pub root_device_type: String,
    pub root_device_name: String,
    pub block_device_mappings: Vec<InstanceBlockDeviceMapping>,
    pub platform: Option<String>,  // Windows or None (Linux)
    pub tags: HashMap<String, String>,
    pub metadata_options: InstanceMetadataOptions,
}

pub struct AwsSecurityGroup {
    pub group_id: String,
    pub group_name: String,
    pub description: String,
    pub vpc_id: Option<String>,
    pub inbound_rules: Vec<IpPermission>,
    pub outbound_rules: Vec<IpPermission>,
    pub tags: HashMap<String, String>,
    pub compliance_checks: SecurityGroupComplianceResults,
}

pub struct SecurityGroupComplianceResults {
    pub allows_unrestricted_ssh: bool,      // 0.0.0.0/0 to port 22
    pub allows_unrestricted_rdp: bool,      // 0.0.0.0/0 to port 3389
    pub allows_all_inbound: bool,           // 0.0.0.0/0 all ports
    pub allows_all_outbound: bool,
    pub risky_rules: Vec<RiskyRuleDetail>,
}
```

---

**7. RDS Integration** (`aws/rds.rs`)

*Capabilities*: `AssetInventory`, `ConfigurationState`

*API Calls*:
```rust
// Instances
rds.describe_db_instances() -> DBInstance[]
rds.describe_db_clusters() -> DBCluster[]

// Security
rds.describe_db_security_groups() -> DBSecurityGroup[]
rds.describe_db_subnet_groups() -> DBSubnetGroup[]

// Snapshots & Backups
rds.describe_db_snapshots() -> DBSnapshot[]
rds.describe_db_cluster_snapshots() -> DBClusterSnapshot[]

// Parameters
rds.describe_db_parameter_groups() -> DBParameterGroup[]
rds.describe_db_parameters(group_name) -> Parameter[]
```

*Evidence Collected*:
| Evidence Type | Description | Control Codes |
|---------------|-------------|---------------|
| RDS Instance Inventory | All databases | A1.1, CC6.1 |
| Encryption Status | At-rest encryption | CC6.6 |
| Public Accessibility | Publicly accessible flag | CC6.6, CC6.7 |
| Backup Configuration | Retention, window | A1.2 |
| Security Group Config | Network access controls | CC6.6 |
| SSL/TLS Status | In-transit encryption | CC6.6 |
| Multi-AZ Status | High availability | A1.2 |

*Data Model*:
```rust
pub struct AwsRdsInstance {
    pub db_instance_identifier: String,
    pub db_instance_class: String,
    pub engine: String,
    pub engine_version: String,
    pub db_instance_status: String,
    pub master_username: String,
    pub endpoint: Option<Endpoint>,
    pub allocated_storage: i32,
    pub instance_create_time: Option<DateTime<Utc>>,
    pub availability_zone: Option<String>,
    pub multi_az: bool,
    pub publicly_accessible: bool,
    pub storage_encrypted: bool,
    pub kms_key_id: Option<String>,
    pub vpc_security_groups: Vec<VpcSecurityGroupMembership>,
    pub db_subnet_group: Option<DBSubnetGroup>,
    pub backup_retention_period: i32,
    pub preferred_backup_window: Option<String>,
    pub deletion_protection: bool,
    pub iam_database_authentication_enabled: bool,
    pub performance_insights_enabled: bool,
    pub auto_minor_version_upgrade: bool,
    pub tags: HashMap<String, String>,
    pub compliance_checks: RdsComplianceResults,
}

pub struct RdsComplianceResults {
    pub is_encrypted: bool,
    pub is_public: bool,
    pub has_backups: bool,
    pub is_multi_az: bool,
    pub deletion_protection: bool,
    pub iam_auth_enabled: bool,
    pub auto_upgrade_enabled: bool,
}
```

---

**AWS Integration API Endpoints**

```
POST   /api/v1/integrations/aws/test         # Test AWS connection
GET    /api/v1/integrations/:id/aws/iam      # Get IAM sync results
GET    /api/v1/integrations/:id/aws/cloudtrail/events  # Get recent events
GET    /api/v1/integrations/:id/aws/securityhub/findings  # Get findings
GET    /api/v1/integrations/:id/aws/config/compliance  # Get Config compliance
GET    /api/v1/integrations/:id/aws/s3/buckets  # Get S3 inventory
GET    /api/v1/integrations/:id/aws/ec2/instances  # Get EC2 inventory
GET    /api/v1/integrations/:id/aws/rds/instances  # Get RDS inventory
POST   /api/v1/integrations/:id/sync         # Trigger full sync (existing)
POST   /api/v1/integrations/:id/sync/partial # Trigger partial sync
```

---

**UI Components**

1. **AWS Setup Wizard** (`/integrations/new/aws`)
   - Choose auth method (IAM Role vs Access Keys)
   - CloudFormation template download for role creation
   - Region selection (multi-select)
   - Service enablement toggles
   - Test connection with detailed results

2. **AWS Dashboard** (`/integrations/:id/aws`)
   - Overview cards: IAM users, EC2 instances, RDS databases, S3 buckets
   - Security Hub findings summary (critical/high/medium/low)
   - Config compliance summary by category
   - Recent CloudTrail events timeline
   - Compliance score trend chart

3. **AWS IAM View** (`/integrations/:id/aws/iam`)
   - User table with MFA status, last activity, access keys
   - Role table with trust policies, attached policies
   - Policy analysis with risky permissions highlighted
   - Access key age warnings

4. **AWS Security Findings** (`/integrations/:id/aws/findings`)
   - Filterable findings list (severity, status, standard)
   - Finding detail drawer with remediation steps
   - Link findings to controls and risks
   - Bulk acknowledge/suppress

5. **AWS Asset Inventory** (`/integrations/:id/aws/inventory`)
   - Tabbed view: EC2 | RDS | S3 | VPCs
   - Compliance status badges per resource
   - Link assets to OpenGRC asset inventory
   - Export to CSV

---

**Implementation Order**

1. **Phase 2.2.1**: Core AWS Provider ✅
   - [x] AWS credential handling (role assumption, access keys)
   - [x] Multi-region iteration
   - [x] Base `AwsIntegrationProvider` struct
   - [x] Test connection implementation
   - [x] Add aws-sdk-* crates to Cargo.toml

2. **Phase 2.2.2**: IAM Integration ✅
   - [x] `AwsIamCollector` implementation
   - [x] User, role, policy collection
   - [x] MFA and access key analysis
   - [x] Compliance checks
   - [ ] Evidence generation (aws/evidence/ folder empty)

3. **Phase 2.2.3**: Security Hub & Config ✅
   - [x] `AwsSecurityHubCollector` implementation
   - [x] `AwsConfigCollector` implementation
   - [x] Finding normalization
   - [x] Control mapping
   - [x] Compliance aggregation

4. **Phase 2.2.4**: Asset Collection (S3, EC2, RDS) ✅
   - [x] `AwsS3Collector` implementation
   - [x] `AwsEc2Collector` implementation
   - [x] `AwsRdsCollector` implementation
   - [ ] Asset linking to OpenGRC inventory

5. **Phase 2.2.5**: CloudTrail Integration (Partial)
   - [x] `AwsCloudTrailCollector` implementation
   - [x] Event categorization
   - [ ] Security event alerting
   - [ ] Audit log evidence generation

6. **Phase 2.2.6**: UI Implementation (Complete)
   - [x] AWS setup wizard with sample IAM policy
   - [x] AWS dashboard (Overview, IAM, Security, Resources, CloudTrail tabs)
   - [x] Collect Evidence button
   - [x] Sample IAM policy in setup flow (instead of CloudFormation)

---

- [x] **AWS** (complete - collectors, evidence generation, UI views, sample IAM policy)
- [ ] **GCP**
  - IAM & Admin
  - Cloud Audit Logs
  - Security Command Center
  - Asset inventory
- [ ] **Azure**
  - Azure AD users/groups
  - Activity logs
  - Security Center
  - Resource inventory

#### 2.3 Identity Provider Integrations
- [ ] **Okta**
  - User inventory
  - MFA status
  - Application assignments
  - System logs
- [ ] **Google Workspace**
  - User directory
  - Security settings
  - Login audit
- [ ] **Azure AD / Entra ID**
  - Users and groups
  - Conditional access
  - Sign-in logs

#### 2.4 DevOps Integrations
- [x] **GitHub** (v1.8.0)
  - [x] Repository inventory
  - [x] Branch protection rules
  - [x] Dependabot alerts
  - [x] Code scanning alerts
  - [x] Secret scanning alerts
  - [x] Organization members & access permissions
  - [x] Evidence collection with control code mapping
  - [x] Dashboard UI
- [ ] **GitLab** (deferred)
  - Similar to GitHub
- [x] **Jira** (v1.8.0)
  - [x] Project inventory
  - [x] Issue tracking & security issues
  - [x] User access report
  - [x] Project role assignments
  - [x] Evidence collection with control code mapping
  - [x] Dashboard UI

#### 2.5 Infrastructure Integrations
- [ ] **Cloudflare**
  - WAF rules
  - DDoS protection status
- [ ] **Datadog / New Relic**
  - Monitoring configuration
  - Alert policies
- [ ] **PagerDuty**
  - Incident response
  - On-call schedules

#### 2.6 Automated Evidence Collection
- [x] Scheduled evidence snapshots (cron-based collection tasks)
- [x] Evidence auto-linking to controls (mapping rules with regex patterns)
- [x] Change detection & alerting (triggers track changes, acknowledgment workflow)
- [x] Evidence freshness scoring (SLA-based scoring, staleness tracking)

#### 2.7 Automated Control Testing
- [ ] Define automated test rules
- [ ] Continuous control monitoring
- [ ] Pass/fail with evidence attachment
- [ ] Alerting on failures
- [ ] Remediation suggestions

### Phase 3: Advanced Features
**Goal: Enterprise-grade capabilities**

#### 3.1 Vendor Management (Partial)
- [x] Vendor inventory
- [x] Risk tiering (criticality field)
- [ ] Security questionnaire builder
- [ ] Questionnaire portal for vendors
- [x] Document collection & expiry tracking (vendor_documents table)
- [ ] SOC 2 report parser (extract key findings)

#### 3.2 Asset Management (Partial)
- [x] Manual asset inventory
- [ ] Auto-discovery from integrations
- [x] Asset classification
- [x] Asset-to-control mapping (asset_control_mappings table)
- [ ] Lifecycle tracking

#### 3.3 User Access Reviews (Partial)
- [x] Campaign creation wizard (access_review_campaigns table)
- [ ] Pull users from integrations
- [x] Reviewer assignment (reviewer_id field)
- [x] Bulk approve/revoke interface (access_review_items table)
- [ ] Certification reports
- [ ] Access removal tracking

#### 3.4 Audit Management (Partial)
- [x] Audit workspace (audits table)
- [ ] Auditor portal (external access)
- [x] Request list management (audit_requests table)
- [ ] Evidence packaging
- [x] Finding tracking (audit_findings table)
- [ ] Remediation workflows

#### 3.5 Task Management (Partial)
- [x] Task assignment & tracking (tasks table)
- [ ] Due date reminders
- [ ] Recurring tasks
- [ ] Workload dashboard
- [ ] Email/Slack notifications

#### 3.6 Multi-Framework Support
- [ ] ISO 27001:2022
- [ ] HIPAA
- [ ] PCI DSS 4.0
- [ ] GDPR
- [ ] NIST CSF
- [ ] SOX ITGC
- [ ] Cross-framework mapping (test once, satisfy many)

### Phase 4: Intelligence & Scale
**Goal: AI-powered insights and enterprise scale**

#### 4.1 AI Features
- [ ] Policy drafting assistant
- [ ] Evidence summarization
- [ ] Gap analysis recommendations
- [ ] Risk scoring suggestions
- [ ] Natural language search
- [ ] Audit preparation assistant

#### 4.2 Advanced Analytics
- [ ] Compliance trend analysis
- [ ] Predictive risk scoring
- [ ] Benchmark comparisons
- [ ] Custom report builder
- [ ] Executive dashboards

#### 4.3 Enterprise Features
- [ ] SSO/SAML configuration
- [ ] SCIM user provisioning
- [ ] Custom roles & permissions
- [ ] Audit log exports (SIEM integration)
- [ ] API rate limiting & quotas
- [ ] White-labeling

#### 4.4 Collaboration
- [ ] Comments & mentions
- [ ] Real-time collaboration
- [ ] Slack/Teams integration
- [ ] Email digests
- [ ] Mobile app (React Native)

## API Design

### RESTful Endpoints Structure
```
/api/v1/
├── auth/
│   ├── login
│   ├── logout
│   └── refresh
├── organizations/
│   ├── {org_id}/
│   │   ├── settings
│   │   ├── users
│   │   └── invitations
├── frameworks/
│   ├── {framework_id}/
│   │   └── requirements
├── controls/
│   ├── {control_id}/
│   │   ├── tests
│   │   ├── evidence
│   │   └── mappings
├── evidence/
│   ├── upload
│   ├── {evidence_id}
│   └── search
├── policies/
│   ├── {policy_id}/
│   │   ├── versions
│   │   └── acknowledgments
├── risks/
│   ├── {risk_id}/
│   │   ├── assessments
│   │   └── controls
├── vendors/
│   ├── {vendor_id}/
│   │   ├── assessments
│   │   └── documents
├── assets/
├── access-reviews/
│   ├── campaigns/
│   │   └── {campaign_id}/items
├── audits/
│   ├── {audit_id}/
│   │   ├── requests
│   │   └── findings
├── integrations/
│   ├── available
│   ├── {integration_id}/
│   │   ├── sync
│   │   └── logs
├── tasks/
├── reports/
│   ├── compliance-posture
│   ├── control-health
│   └── export
└── search/
```

## UI/UX Design Principles

### Navigation Structure
```
┌─────────────────────────────────────────────────────────────┐
│  OpenGRC    [Organization ▼]    [Search...]    [? ] [User] │
├─────────────────────────────────────────────────────────────┤
│ Dashboard        │                                          │
│ ─────────────    │                                          │
│ Frameworks       │         Main Content Area                │
│ Controls         │                                          │
│ Evidence         │                                          │
│ Policies         │                                          │
│ Risks            │                                          │
│ ─────────────    │                                          │
│ Vendors          │                                          │
│ Assets           │                                          │
│ Access Reviews   │                                          │
│ ─────────────    │                                          │
│ Audits           │                                          │
│ Tasks            │                                          │
│ Reports          │                                          │
│ ─────────────    │                                          │
│ Integrations     │                                          │
│ Settings         │                                          │
└─────────────────────────────────────────────────────────────┘
```

### Design System
- Clean, professional aesthetic (think Linear, Notion)
- Dark mode support from day one
- Consistent component library (shadcn/ui base)
- Data-dense but not cluttered
- Fast, responsive interactions
- Keyboard navigation throughout

### Key UX Patterns
- **Global search**: Find anything instantly (Cmd+K)
- **Inline editing**: Click to edit, auto-save
- **Bulk actions**: Select multiple, act once
- **Contextual help**: Tooltips explaining compliance concepts
- **Progress indicators**: Always show compliance progress
- **Smart defaults**: Pre-fill based on common patterns

## Project Structure

```
opengrc/
├── api/                    # Rust API
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── config/
│   │   ├── routes/
│   │   │   ├── mod.rs
│   │   │   ├── auth.rs
│   │   │   ├── controls.rs
│   │   │   ├── evidence.rs
│   │   │   ├── policies.rs
│   │   │   ├── risks.rs
│   │   │   ├── vendors.rs
│   │   │   ├── integrations.rs
│   │   │   └── ...
│   │   ├── models/
│   │   ├── services/
│   │   ├── integrations/
│   │   │   ├── mod.rs
│   │   │   ├── aws/
│   │   │   ├── github/
│   │   │   ├── okta/
│   │   │   └── ...
│   │   ├── middleware/
│   │   ├── cache/
│   │   └── utils/
│   └── migrations/
├── worker/                 # Rust background worker
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── jobs/
│       │   ├── evidence_collection.rs
│       │   ├── control_testing.rs
│       │   ├── notifications.rs
│       │   └── ...
│       └── ...
├── ui/                     # Next.js UI
│   ├── package.json
│   ├── src/
│   │   ├── app/
│   │   │   ├── layout.tsx
│   │   │   ├── page.tsx
│   │   │   ├── dashboard/
│   │   │   ├── frameworks/
│   │   │   ├── controls/
│   │   │   ├── evidence/
│   │   │   ├── policies/
│   │   │   ├── risks/
│   │   │   ├── vendors/
│   │   │   ├── integrations/
│   │   │   └── ...
│   │   ├── components/
│   │   ├── lib/
│   │   ├── hooks/
│   │   └── styles/
│   └── public/
├── shared/                 # Shared types/schemas
│   └── schemas/
├── docker/
│   ├── docker-compose.yml
│   ├── Dockerfile.api
│   ├── Dockerfile.worker
│   └── Dockerfile.ui
├── terraform/              # Infrastructure as code
├── docs/                   # Documentation
│   ├── api.md
│   ├── deployment.md
│   └── contributing.md
├── scripts/
├── .github/
│   └── workflows/
├── README.md
├── LICENSE                 # MIT
└── plan.md                 # This file
```

## Competitive Advantages

### 1. **Truly Open Source**
- MIT license
- No "open core" gotchas
- Community-driven development
- Self-host anywhere

### 2. **Performance First**
- Rust API = blazing fast
- Efficient database queries with proper indexing
- Redis caching throughout
- Optimistic UI updates

### 3. **Developer Experience**
- Beautiful, modern UI
- Comprehensive API
- Excellent documentation
- Easy local development (Docker Compose)
- Terraform modules for deployment

### 4. **Integration Architecture**
- Pluggable integration system
- Easy to add new integrations
- Community-contributed integrations
- Webhook support for custom integrations

### 5. **Multi-Framework by Design**
- Not just SOC 2
- Control once, map to many frameworks
- Cross-framework gap analysis
- Framework update tracking

### 6. **Enterprise Ready**
- Multi-tenant from day one
- Audit logging everywhere
- Role-based access control
- API-first design

## Success Metrics

### Adoption
- GitHub stars
- Docker pulls
- Active installations (opt-in telemetry)
- Community contributors

### Quality
- Test coverage > 80%
- API response times < 100ms (p95)
- Zero critical security vulnerabilities
- Documentation completeness

### Community
- Discord/Slack community size
- Issues resolved per month
- PRs merged from community
- Integration contributions

## Monetization (Optional - for sustainability)

While fully open source, potential revenue streams:
1. **Managed Cloud Offering** - Hosted version for those who don't want to self-host
2. **Enterprise Support** - SLAs, dedicated support, custom development
3. **Consulting Services** - Implementation, compliance guidance
4. **Training & Certification** - OpenGRC certified practitioner

## Roadmap Timeline

### Phase 1: Foundation (MVP)
- Core platform, manual workflows
- SOC 2 framework
- Basic reporting
- **Target: Usable for SOC 2 compliance**

### Phase 2: Automation
- Integration framework
- Major cloud/SaaS integrations
- Automated evidence collection
- **Target: 80% reduction in manual work**

### Phase 3: Advanced
- All major frameworks
- Vendor management
- Access reviews
- Audit portal
- **Target: Feature parity with paid tools**

### Phase 4: Intelligence
- AI features
- Advanced analytics
- Mobile app
- **Target: Beyond paid competitors**

## Security Considerations

A GRC platform must lead by example. Security is not optional.

### Authentication & Authorization
- **TitaniumVault Integration**: All auth delegated to TV, no password storage
- **Session Management**: Short-lived JWTs (15min) with secure refresh tokens
- **MFA Enforcement**: Require MFA for all admin/compliance manager roles
- **API Keys**: Scoped, rotatable, with audit logging
- **RBAC**: Principle of least privilege, deny by default

### Data Protection
```
┌─────────────────────────────────────────────────────────────┐
│                    Data Protection Layers                    │
├─────────────────────────────────────────────────────────────┤
│  Transit          │  TLS 1.3 everywhere, no exceptions      │
│  Rest (DB)        │  AES-256 encryption at storage layer    │
│  Rest (Files)     │  S3 SSE-KMS with customer-managed keys  │
│  Sensitive Fields │  Application-level encryption (secrets) │
│  Backups          │  Encrypted with separate key rotation   │
└─────────────────────────────────────────────────────────────┘
```

### Multi-Tenant Isolation
- **Database**: Row-level security with `organization_id` on every query
- **Caching**: Namespace all Redis keys with org prefix
- **File Storage**: Separate S3 prefixes per organization
- **Search**: Meilisearch tenant isolation via index-per-org or filtered queries
- **Logs**: Never log sensitive data, always include org context

### Input Validation & Sanitization
- Validate all inputs at API boundary using strong typing
- Parameterized queries only - no string concatenation for SQL
- Sanitize all user content before storage and display
- File upload validation: type checking, size limits, virus scanning
- Rate limiting on all endpoints (stricter on auth endpoints)

### Security Headers & CORS
```rust
// Required security headers
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
Content-Security-Policy: default-src 'self'; script-src 'self'
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
X-XSS-Protection: 1; mode=block
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: geolocation=(), microphone=(), camera=()
```

### Audit Logging Requirements
Every security-relevant action must be logged:
- Authentication events (login, logout, failures, MFA)
- Authorization failures (access denied)
- Data access (who viewed what, when)
- Data modifications (create, update, delete with before/after)
- Admin actions (user management, settings changes)
- Integration activities (syncs, API calls)
- Export/download events

Log format:
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "organization_id": "uuid",
  "user_id": "uuid",
  "action": "control.update",
  "entity_type": "control",
  "entity_id": "uuid",
  "ip_address": "x.x.x.x",
  "user_agent": "...",
  "old_values": {},
  "new_values": {},
  "request_id": "uuid"
}
```

### Secrets Management
- Never commit secrets to git
- Use environment variables or secrets manager (AWS Secrets Manager, Vault)
- Rotate credentials regularly (automated where possible)
- Integration credentials encrypted at rest with org-specific keys
- No secrets in logs, error messages, or API responses

### Vulnerability Management
- Automated dependency scanning (Dependabot, cargo-audit)
- SAST in CI pipeline
- Container image scanning
- Regular penetration testing (annual minimum)
- Responsible disclosure program
- Security patch SLA: Critical (24h), High (72h), Medium (7d)

### Incident Response
- Security incident runbook documented
- Contact points defined
- Breach notification procedures (GDPR, state laws)
- Post-incident review process
- Customer communication templates

### Compliance Self-Assessment
OpenGRC should pass its own compliance checks:
- [ ] SOC 2 Type II ready architecture
- [ ] GDPR compliant (data subject rights, DPA ready)
- [ ] CCPA compliant
- [ ] Accessibility (WCAG 2.1 AA)

### Secure Development Practices
- Code review required for all changes
- Security-focused PR checklist
- No `unsafe` Rust without explicit justification and review
- Dependency minimization (fewer deps = smaller attack surface)
- Regular security training for contributors

## Testing Strategy

### Testing Pyramid
```
                    ┌───────────┐
                    │   E2E     │  Few, critical paths
                    │  Tests    │
                   ─┴───────────┴─
                 ┌─────────────────┐
                 │  Integration    │  API contracts, DB queries
                 │     Tests       │
                ─┴─────────────────┴─
              ┌───────────────────────┐
              │      Unit Tests       │  Business logic, utilities
              │                       │
             ─┴───────────────────────┴─
```

### Unit Tests (Rust API)
- **Coverage Target**: 80%+ on business logic
- **What to Test**:
  - Service layer functions
  - Validation logic
  - Permission checks
  - Risk scoring algorithms
  - Data transformations
- **Tools**: Built-in Rust test framework, `mockall` for mocking

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_score_calculation() {
        let risk = Risk { likelihood: 4, impact: 5 };
        assert_eq!(risk.inherent_score(), 20);
    }

    #[tokio::test]
    async fn test_control_service_create() {
        let mock_repo = MockControlRepository::new();
        let service = ControlService::new(mock_repo);
        // ...
    }
}
```

### Integration Tests (API)
- **Coverage**: All API endpoints
- **What to Test**:
  - Request/response contracts
  - Authentication/authorization flows
  - Database operations (use test containers)
  - Cache invalidation
  - Multi-tenant isolation
- **Tools**: `reqwest` for HTTP, `testcontainers` for Postgres/Redis

```rust
#[tokio::test]
async fn test_create_control_requires_auth() {
    let app = spawn_test_app().await;
    let response = app.post("/api/v1/controls").json(&control).send().await;
    assert_eq!(response.status(), 401);
}

#[tokio::test]
async fn test_org_isolation() {
    let app = spawn_test_app().await;
    let control = create_control_for_org(&app, org_a).await;

    // User from org_b should not see org_a's control
    let response = app.get_as_user(org_b_user, &format!("/api/v1/controls/{}", control.id)).await;
    assert_eq!(response.status(), 404);
}
```

### UI Tests (Next.js)
- **Unit Tests**: Component logic with Jest + React Testing Library
- **Integration**: API mocking with MSW (Mock Service Worker)
- **Visual Regression**: Chromatic or Percy for UI consistency

```typescript
// Component test
describe('ControlCard', () => {
  it('displays control status badge correctly', () => {
    render(<ControlCard control={mockControl} />);
    expect(screen.getByText('Implemented')).toHaveClass('badge-success');
  });
});

// Integration test
describe('Controls Page', () => {
  it('fetches and displays controls', async () => {
    server.use(
      rest.get('/api/v1/controls', (req, res, ctx) => {
        return res(ctx.json({ data: mockControls }));
      })
    );
    render(<ControlsPage />);
    await waitFor(() => {
      expect(screen.getByText('Access Control Policy')).toBeInTheDocument();
    });
  });
});
```

### End-to-End Tests
- **Tool**: Playwright (fast, reliable, cross-browser)
- **Coverage**: Critical user journeys only
- **What to Test**:
  - User login flow
  - Create/edit control with evidence
  - Policy approval workflow
  - Risk assessment flow
  - Report generation

```typescript
test('complete control creation flow', async ({ page }) => {
  await page.goto('/controls');
  await page.click('button:has-text("New Control")');
  await page.fill('[name="name"]', 'Access Control Policy');
  await page.fill('[name="description"]', 'Ensures proper access...');
  await page.selectOption('[name="frequency"]', 'quarterly');
  await page.click('button:has-text("Save")');
  await expect(page.locator('.toast-success')).toBeVisible();
  await expect(page).toHaveURL(/\/controls\/[\w-]+/);
});
```

### Performance Tests
- **Tool**: k6 for load testing
- **Benchmarks**:
  - API response time < 100ms (p95)
  - Dashboard load < 2 seconds
  - Search results < 500ms
  - Support 100 concurrent users per instance

```javascript
// k6 load test
export default function() {
  const controls = http.get(`${BASE_URL}/api/v1/controls`, { headers });
  check(controls, {
    'status is 200': (r) => r.status === 200,
    'response time < 100ms': (r) => r.timings.duration < 100,
  });
}
```

### Security Tests
- **SAST**: `cargo-audit`, `npm audit` in CI
- **DAST**: OWASP ZAP automated scans
- **Dependency Scanning**: Dependabot alerts
- **Specific Tests**:
  - SQL injection attempts
  - XSS payload injection
  - CSRF token validation
  - Authorization bypass attempts
  - Rate limit enforcement

### CI Pipeline
```yaml
# .github/workflows/ci.yml
jobs:
  test-api:
    steps:
      - cargo fmt --check
      - cargo clippy -- -D warnings
      - cargo audit
      - cargo test --all-features

  test-ui:
    steps:
      - npm run lint
      - npm run typecheck
      - npm run test
      - npm run build

  e2e:
    needs: [test-api, test-ui]
    steps:
      - docker-compose up -d
      - npx playwright test

  security-scan:
    steps:
      - run: cargo audit
      - run: npm audit
      - uses: zaproxy/action-baseline@v0.7.0
```

### Test Data Management
- Seed scripts for development and testing
- Factory functions for generating test entities
- Anonymized production data snapshots for performance testing
- Framework data (SOC 2 requirements) as fixtures

## Getting Started (Development)

```bash
# Clone the repo
git clone https://github.com/your-org/opengrc.git
cd opengrc

# Start infrastructure
docker-compose up -d postgres redis meilisearch

# Run API
cd api
cargo run

# Run UI
cd ../ui
npm install
npm run dev

# Run worker
cd ../worker
cargo run
```

## Next Steps

1. [x] Initialize Rust API project with Axum
2. [x] Initialize Next.js UI project
3. [x] Set up database migrations
4. [x] Implement TitaniumVault auth integration
5. [x] Build core CRUD for controls, evidence, policies
6. [x] Load SOC 2 framework data
7. [x] Create dashboard UI
8. [x] Write comprehensive README
9. [x] Set up CI/CD
10. [x] Create Docker Compose for easy deployment

### Completed in Phase 1 Foundation
- [x] Project scaffolding complete (api/, ui/, worker/)
- [x] Full database schema (51 tables with indexes and triggers)
- [x] TitaniumVault SSO integration
- [x] Multi-tenant architecture with organization_id on all tables
- [x] RBAC with role field on users
- [x] Activity logging table and infrastructure
- [x] Controls CRUD with stats endpoint
- [x] Evidence management with S3 presigned URLs
- [x] Policy management with version history
- [x] Risk register with scoring (1-5 likelihood/impact matrix)
- [x] Vendor management with assessments
- [x] Asset inventory
- [x] Audit tracking with requests/findings
- [x] Frameworks & requirements management
- [x] SOC 2 Trust Service Criteria seeded
- [x] Redis caching layer
- [x] Dashboard with real-time stats
- [x] Dark mode support
- [x] Responsive sidebar navigation
- [x] Gap analysis dashboard visualization
- [x] Full-text search with Meilisearch
- [x] Policy templates (25 common policies)
- [x] Risk heatmap visualization
- [x] Control-to-requirement mapping UI

### Remaining Phase 1 Work
All Phase 1 items complete!

**Phase 1 100% Complete** - All core features implemented.

### Completed in Phase 2 Integration Framework
- [x] Pluggable integration architecture with IntegrationProvider trait
- [x] Integration CRUD with caching and cache invalidation
- [x] Available integrations catalog (13 providers)
- [x] Integration management UI with health dashboard
- [x] Credential vault with AES-256-GCM encryption
- [x] OAuth2 connection flow with PKCE
- [x] Error handling & retry logic
- [x] AWS Integration (complete)
  - [x] All 7 AWS service collectors (IAM, CloudTrail, Security Hub, Config, S3, EC2, RDS)
  - [x] 13 AWS-specific database tables
  - [x] Compliance checks and control mapping
  - [x] Evidence generation from AWS data (manual trigger via button)
  - [x] AWS sample IAM policy (in UI setup flow)

### Remaining Phase 2 Work
- [ ] Identity provider integrations (Okta, Google Workspace, Azure AD)
- [ ] Sync scheduling worker (deferred)

**Completed in v1.8.0:**
- [x] GitHub integration (repositories, security alerts, branch protection, members)
- [x] Jira integration (projects, issues, users, permissions)

**Completed in v1.9.0:**
- [x] Storage abstraction - local filesystem or S3 backend
- [x] Local storage default for easier development (no cloud credentials needed)
- [x] S3 storage with presigned URLs for production
- [x] Support for MinIO/LocalStack via custom endpoints

**Deferred/Out of Scope:**
- CloudFormation template generator (providing sample policy instead)
- GCP integration
- Azure integration
- GitLab integration

**Phase 2 ~90% Complete** - Integration framework, AWS, GitHub, Jira integrations, and storage abstraction complete.

## Policy Templates

### Overview
Policy templates provide organizations with professionally-written, compliance-ready policy documents that can be customized to their needs. Each template includes:
- **Full policy content** in Markdown format
- **Framework mappings** showing which compliance frameworks the policy supports
- **Suggested controls** that should reference the policy
- **Review frequency** recommendations

### Template Categories

#### Security Policies (SEC)
| Code | Policy Name | Frameworks |
|------|-------------|------------|
| SEC-001 | Information Security Policy | SOC 2, ISO 27001, HIPAA, PCI DSS |
| SEC-002 | Access Control Policy | SOC 2, ISO 27001, HIPAA, PCI DSS |
| SEC-003 | Password & Authentication Policy | SOC 2, ISO 27001, HIPAA, PCI DSS |
| SEC-004 | Encryption Policy | SOC 2, ISO 27001, HIPAA, PCI DSS |
| SEC-005 | Network Security Policy | SOC 2, ISO 27001, PCI DSS |
| SEC-006 | Vulnerability Management Policy | SOC 2, ISO 27001, PCI DSS |
| SEC-007 | Security Awareness Training Policy | SOC 2, ISO 27001, HIPAA, PCI DSS |
| SEC-008 | Physical Security Policy | SOC 2, ISO 27001, HIPAA, PCI DSS |

#### IT Operations Policies (IT)
| Code | Policy Name | Frameworks |
|------|-------------|------------|
| IT-001 | Acceptable Use Policy | SOC 2, ISO 27001, HIPAA |
| IT-002 | Change Management Policy | SOC 2, ISO 27001, PCI DSS |
| IT-003 | Backup & Recovery Policy | SOC 2, ISO 27001, HIPAA |
| IT-004 | Asset Management Policy | SOC 2, ISO 27001 |
| IT-005 | Mobile Device & BYOD Policy | SOC 2, ISO 27001, HIPAA |
| IT-006 | Remote Work Policy | SOC 2, ISO 27001 |
| IT-007 | Software Development Lifecycle Policy | SOC 2, ISO 27001, PCI DSS |

#### Compliance & Risk Policies (COMP)
| Code | Policy Name | Frameworks |
|------|-------------|------------|
| COMP-001 | Risk Management Policy | SOC 2, ISO 27001, HIPAA |
| COMP-002 | Vendor Management Policy | SOC 2, ISO 27001, HIPAA |
| COMP-003 | Incident Response Policy | SOC 2, ISO 27001, HIPAA, PCI DSS |
| COMP-004 | Business Continuity Policy | SOC 2, ISO 27001, HIPAA |
| COMP-005 | Data Classification Policy | SOC 2, ISO 27001, HIPAA |

#### Privacy Policies (PRIV)
| Code | Policy Name | Frameworks |
|------|-------------|------------|
| PRIV-001 | Data Privacy Policy | GDPR, CCPA, HIPAA |
| PRIV-002 | Data Retention Policy | SOC 2, GDPR, HIPAA |
| PRIV-003 | Data Breach Notification Policy | GDPR, HIPAA, State Laws |

#### Human Resources Policies (HR)
| Code | Policy Name | Frameworks |
|------|-------------|------------|
| HR-001 | Code of Conduct | SOC 2, ISO 27001 |
| HR-002 | Background Check Policy | SOC 2, ISO 27001, HIPAA |

### Template Structure

Each template follows a consistent structure:

```markdown
# [Policy Name]

**Policy Code:** [CODE]
**Version:** 1.0
**Effective Date:** [To be set]
**Review Date:** [Annual review recommended]
**Owner:** [To be assigned]
**Category:** [security|privacy|hr|it|compliance]

## 1. Purpose

[Why this policy exists and what it aims to achieve]

## 2. Scope

[Who and what this policy applies to]

## 3. Policy Statements

[Core policy requirements organized by topic]

### 3.1 [Topic Area]
- Requirement 1
- Requirement 2

### 3.2 [Topic Area]
- Requirement 1
- Requirement 2

## 4. Roles & Responsibilities

| Role | Responsibilities |
|------|-----------------|
| [Role 1] | [Responsibilities] |
| [Role 2] | [Responsibilities] |

## 5. Compliance

[How compliance with this policy is measured and enforced]

## 6. Exceptions

[Process for requesting policy exceptions]

## 7. Related Documents

- [Related Policy 1]
- [Related Procedure 1]

## 8. Definitions

| Term | Definition |
|------|-----------|
| [Term 1] | [Definition] |

## 9. Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | [Date] | [Author] | Initial release |
```

### API Endpoints

```
GET  /api/v1/policy-templates           # List all templates
GET  /api/v1/policy-templates/:id       # Get template details
GET  /api/v1/policy-templates/search    # Search templates by framework/category
```

### Template Response Schema

```json
{
  "id": "sec-001",
  "code": "SEC-001",
  "title": "Information Security Policy",
  "description": "Establishes the organization's approach to managing information security",
  "category": "security",
  "frameworks": ["soc2", "iso27001", "hipaa", "pci-dss"],
  "review_frequency": "annual",
  "content": "# Information Security Policy\n\n...",
  "related_templates": ["sec-002", "sec-003"],
  "suggested_controls": ["AC-001", "AC-002"]
}
```

---

*This plan is a living document. Update as the project evolves.*
