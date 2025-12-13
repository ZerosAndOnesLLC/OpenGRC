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

**Phase 1: Foundation (MVP) - In Progress**

- [x] Project scaffolding (Rust API + Next.js UI)
- [x] Database schema with migrations
- [x] SOC 2 Trust Service Criteria pre-loaded
- [x] TitaniumVault SSO authentication
- [x] Multi-tenant architecture
- [x] Controls management (CRUD + stats)
- [x] Frameworks & requirements management
- [x] Evidence management
- [x] Policy management with versioning
- [x] Risk register with scoring
- [x] Vendor management
- [x] Asset inventory
- [x] Audit tracking
- [x] Dashboard with real-time stats
- [x] Redis caching layer
- [x] Evidence file uploads to S3 (presigned URLs)
- [ ] Automated control testing
- [ ] Report generation

## Features

- **Multi-Framework Support** - SOC 2, ISO 27001, HIPAA, PCI DSS, GDPR, NIST CSF
- **Control Management** - Define, test, and monitor security controls
- **Evidence Collection** - Manual uploads + automated collection from 50+ integrations
- **Policy Management** - Version-controlled policies with employee acknowledgments
- **Risk Register** - Track and treat risks with scoring matrices
- **Vendor Management** - Assess and monitor third-party risk
- **Access Reviews** - Periodic user access certification campaigns
- **Audit Portal** - Collaborate with external auditors seamlessly

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

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](docs/CONTRIBUTING.md) for guidelines.

## License

MIT - See [LICENSE](LICENSE) for details.

## Support

- [Documentation](docs/)
- [GitHub Issues](https://github.com/your-org/opengrc/issues)
- [Discord Community](https://discord.gg/opengrc)
