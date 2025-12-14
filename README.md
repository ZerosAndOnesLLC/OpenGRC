# OpenGRC

Open-source Governance, Risk, and Compliance (GRC) platform. Get SOC 2, ISO 27001, HIPAA, and other framework compliance without the $50k/year price tag.

## Why OpenGRC?

| Paid Platforms | OpenGRC |
|----------------|---------|
| $10k-80k/year | Free & self-hosted |
| Vendor lock-in | Own your data |
| Limited customization | Fully extensible |
| Per-seat pricing | Unlimited users |

## Current Status

**Phase 1: Foundation (MVP) - 100% Complete**

- [x] Project scaffolding (Rust API + Next.js UI)
- [x] Database schema with migrations (51+ tables)
- [x] SOC 2 Trust Service Criteria pre-loaded
- [x] TitaniumVault SSO authentication
- [x] Multi-tenant architecture
- [x] Controls management (CRUD + stats)
- [x] Frameworks & requirements management
- [x] Custom framework creation with bulk import (CSV/JSON)
- [x] Evidence management with full-text search (Meilisearch)
- [x] Policy management with versioning
- [x] Employee acknowledgment portal (/policies/pending)
- [x] Policy reminders (email via SES + in-app notifications)
- [x] Risk register with scoring
- [x] Vendor management with assessments
- [x] Asset inventory with classification
- [x] Audit tracking with requests/findings
- [x] Dashboard with real-time stats
- [x] Redis caching layer
- [x] Evidence file uploads to S3 (presigned URLs)
- [x] Risk heatmap visualization (5x5 matrix)
- [x] Gap analysis dashboard (framework coverage)
- [x] Policy templates (25 pre-built compliance policies)
- [x] Control-to-requirement mapping UI
- [x] Report generation (CSV + PDF export with branding)

**Phase 2: Automation & Integrations - 95% Complete**

- [x] Integration framework with pluggable architecture
- [x] Credential vault (AES-256-GCM encryption)
- [x] OAuth2 connection flow with PKCE
- [x] AWS integration (IAM, CloudTrail, Security Hub, Config, S3, EC2, RDS)
- [x] AWS evidence generation (manual trigger)
- [x] AWS sample IAM policy for setup
- [x] GitHub integration (repos, security alerts, branch protection, members)
- [x] Jira integration (projects, issues, users, permissions)
- [x] Automated evidence collection with cron-based scheduling
- [x] Evidence freshness scoring with SLA tracking
- [x] Change detection and alerting
- [x] Auto-linking evidence to controls via mapping rules
- [ ] Identity provider integrations (Okta, Google Workspace, Azure AD)

## Features

- **Multi-Framework Support** - SOC 2, ISO 27001, HIPAA, PCI DSS, GDPR, NIST CSF
- **Control Management** - Define, test, and monitor security controls
- **Evidence Collection** - Manual uploads + scheduled automated collection with freshness scoring
- **Policy Management** - Version-controlled policies with employee acknowledgment portal
- **Policy Reminders** - Email (AWS SES) and in-app notification reminders
- **Policy Templates** - 25 pre-written policy templates covering SOC 2, ISO 27001, HIPAA, and more
- **Risk Register** - Track and treat risks with scoring matrices
- **Risk Heatmap** - Interactive 5x5 likelihood/impact matrix visualization
- **Gap Analysis** - Framework coverage dashboard with per-category breakdown
- **Vendor Management** - Assess and monitor third-party risk
- **Access Reviews** - Periodic user access certification campaigns
- **Audit Portal** - Collaborate with external auditors seamlessly
- **PDF Reports** - Branded PDF exports with headers, footers, and summary charts
- **In-App Notifications** - Real-time notification center for policy reminders and alerts

## Tech Stack

- **API**: Rust (Axum) - Fast, safe, async
- **UI**: Next.js 14 - Modern React with App Router
- **Database**: PostgreSQL - Reliable and scalable
- **Cache/Queue**: Redis - Fast caching and job queues
- **Search**: Meilisearch - Instant full-text search
- **Storage**: S3-compatible - Evidence file storage

## Quick Start

### Prerequisites

- Docker & Docker Compose
- Rust 1.75+ (for development)
- Node.js 20+ (for development)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/your-org/opengrc.git
cd opengrc

# Start infrastructure services
docker-compose up -d postgres redis meilisearch minio

# Run database migrations
cd api
cargo sqlx migrate run

# Start the API (in one terminal)
cargo run

# Start the UI (in another terminal)
cd ../ui
npm install
npm run dev
```

### Access the Application

- **UI**: http://localhost:3000
- **API**: http://localhost:8080
- **API Health**: http://localhost:8080/health
- **Meilisearch**: http://localhost:7700
- **MinIO Console**: http://localhost:9001 (opengrc / opengrc_dev)

## Project Structure

```
opengrc/
├── api/                # Rust API (Axum)
├── worker/             # Rust background worker
├── ui/                 # Next.js frontend
├── docker/             # Dockerfiles
├── terraform/          # Infrastructure as code
├── docs/               # Documentation
└── docker-compose.yml  # Local development
```

## Configuration

### Environment Variables

#### API
```env
DATABASE_URL=postgres://opengrc:password@localhost:5432/opengrc
REDIS_URL=redis://localhost:6379
MEILISEARCH_URL=http://localhost:7700
MEILISEARCH_KEY=your_master_key
S3_ENDPOINT=http://localhost:9000
S3_ACCESS_KEY=opengrc
S3_SECRET_KEY=your_secret
S3_BUCKET=opengrc-evidence
TV_API_URL=https://api.titanium-vault.com
TV_CLIENT_ID=your_client_id
TV_CLIENT_SECRET=your_client_secret
```

#### UI
```env
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_TV_URL=https://titanium-vault.com
```

## Integrations

OpenGRC supports automated evidence collection from:

### Cloud Providers
- AWS (IAM, CloudTrail, Security Hub, Config)
- GCP (IAM, Audit Logs, Security Command Center)
- Azure (Entra ID, Activity Logs, Security Center)

### Identity Providers
- Okta
- Google Workspace
- Azure AD / Entra ID

### DevOps
- GitHub
- GitLab
- Jira

### Infrastructure
- Cloudflare
- Datadog
- PagerDuty

## Policy Templates

OpenGRC includes 25 professionally-written policy templates to jumpstart your compliance program:

### Security Policies
- Information Security Policy (SEC-001)
- Access Control Policy (SEC-002)
- Password & Authentication Policy (SEC-003)
- Encryption Policy (SEC-004)
- Network Security Policy (SEC-005)
- Vulnerability Management Policy (SEC-006)
- Security Awareness Training Policy (SEC-007)
- Physical Security Policy (SEC-008)

### IT Operations Policies
- Acceptable Use Policy (IT-001)
- Change Management Policy (IT-002)
- Backup & Recovery Policy (IT-003)
- Asset Management Policy (IT-004)
- Mobile Device & BYOD Policy (IT-005)
- Remote Work Policy (IT-006)
- Software Development Lifecycle Policy (IT-007)

### Compliance & Risk Policies
- Risk Management Policy (COMP-001)
- Vendor Management Policy (COMP-002)
- Incident Response Policy (COMP-003)
- Business Continuity Policy (COMP-004)
- Data Classification Policy (COMP-005)

### Privacy Policies
- Data Privacy Policy (PRIV-001)
- Data Retention Policy (PRIV-002)
- Data Breach Notification Policy (PRIV-003)

### Human Resources Policies
- Code of Conduct (HR-001)
- Background Check Policy (HR-002)

All templates include framework mappings (SOC 2, ISO 27001, HIPAA, PCI DSS, GDPR) and can be customized for your organization.

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

## License

MIT - See [LICENSE](LICENSE) for details.

## Support

- [Documentation](docs/)
- [GitHub Issues](https://github.com/your-org/opengrc/issues)
- [Discord Community](https://discord.gg/opengrc)
