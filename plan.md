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
- **Storage**: S3-compatible - Evidence file storage
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
- [ ] Project scaffolding (Rust API + Next.js UI)
- [ ] Database setup with migrations
- [ ] TitaniumVault authentication integration
- [ ] Multi-tenant architecture
- [ ] Role-based access control (Admin, Compliance Manager, Contributor, Viewer, Auditor)
- [ ] Activity logging

#### 1.2 Framework Management
- [ ] Pre-loaded SOC 2 Trust Service Criteria
- [ ] Framework requirement browser
- [ ] Custom framework creation
- [ ] Control library with templates
- [ ] Control-to-requirement mapping
- [ ] Gap analysis dashboard

#### 1.3 Evidence Management
- [ ] Evidence upload (drag & drop, bulk)
- [ ] Evidence metadata and tagging
- [ ] Link evidence to controls
- [ ] Evidence expiration tracking
- [ ] Full-text search (Meilisearch)
- [ ] Version history

#### 1.4 Policy Management
- [ ] Policy editor (Markdown with preview)
- [ ] Policy templates (20+ common policies)
- [ ] Version control with diff view
- [ ] Approval workflow
- [ ] Employee acknowledgment portal
- [ ] Acknowledgment tracking & reminders

#### 1.5 Risk Register
- [ ] Risk CRUD with scoring matrix
- [ ] Risk categories and templates
- [ ] Risk-to-control mapping
- [ ] Treatment plan tracking
- [ ] Risk heatmap visualization
- [ ] Risk assessment history

#### 1.6 Dashboard & Reporting
- [ ] Compliance posture dashboard
- [ ] Control health overview
- [ ] Upcoming tasks/deadlines
- [ ] Framework coverage reports
- [ ] Export to PDF/CSV

### Phase 2: Automation & Integrations
**Goal: Reduce manual work by 80%**

#### 2.1 Integration Framework
- [ ] Pluggable integration architecture
- [ ] OAuth2 connection flow
- [ ] Credential vault (encrypted storage)
- [ ] Sync scheduling (cron-based)
- [ ] Error handling & retry logic
- [ ] Integration health monitoring

#### 2.2 Cloud Provider Integrations
- [ ] **AWS**
  - IAM users, roles, policies
  - CloudTrail logs
  - Security Hub findings
  - Config compliance
  - S3 bucket policies
  - EC2/RDS inventory
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
- [ ] **GitHub**
  - Repository inventory
  - Branch protection rules
  - Dependabot alerts
  - Code scanning alerts
  - Access permissions
- [ ] **GitLab**
  - Similar to GitHub
- [ ] **Jira**
  - Security ticket tracking
  - Change management

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
- [ ] Scheduled evidence snapshots
- [ ] Evidence auto-linking to controls
- [ ] Change detection & alerting
- [ ] Evidence freshness scoring

#### 2.7 Automated Control Testing
- [ ] Define automated test rules
- [ ] Continuous control monitoring
- [ ] Pass/fail with evidence attachment
- [ ] Alerting on failures
- [ ] Remediation suggestions

### Phase 3: Advanced Features
**Goal: Enterprise-grade capabilities**

#### 3.1 Vendor Management
- [ ] Vendor inventory
- [ ] Risk tiering
- [ ] Security questionnaire builder
- [ ] Questionnaire portal for vendors
- [ ] Document collection & expiry tracking
- [ ] SOC 2 report parser (extract key findings)

#### 3.2 Asset Management
- [ ] Manual asset inventory
- [ ] Auto-discovery from integrations
- [ ] Asset classification
- [ ] Asset-to-control mapping
- [ ] Lifecycle tracking

#### 3.3 User Access Reviews
- [ ] Campaign creation wizard
- [ ] Pull users from integrations
- [ ] Reviewer assignment (by department/manager)
- [ ] Bulk approve/revoke interface
- [ ] Certification reports
- [ ] Access removal tracking

#### 3.4 Audit Management
- [ ] Audit workspace
- [ ] Auditor portal (external access)
- [ ] Request list management
- [ ] Evidence packaging
- [ ] Finding tracking
- [ ] Remediation workflows

#### 3.5 Task Management
- [ ] Task assignment & tracking
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
├── LICENSE                 # Apache 2.0 or MIT
└── plan.md                 # This file
```

## Competitive Advantages

### 1. **Truly Open Source**
- Apache 2.0 or MIT license
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

1. [ ] Initialize Rust API project with Axum
2. [ ] Initialize Next.js UI project
3. [ ] Set up database migrations
4. [ ] Implement TitaniumVault auth integration
5. [ ] Build core CRUD for controls, evidence, policies
6. [ ] Load SOC 2 framework data
7. [ ] Create dashboard UI
8. [ ] Write comprehensive README
9. [ ] Set up CI/CD
10. [ ] Create Docker Compose for easy deployment

---

*This plan is a living document. Update as the project evolves.*
