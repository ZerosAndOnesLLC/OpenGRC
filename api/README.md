# OpenGRC API

The Rust-based API server for OpenGRC - an open-source GRC (Governance, Risk, and Compliance) platform.

## Overview

This API provides the backend services for the OpenGRC platform, handling:
- Authentication and authorization via TitaniumVault
- Multi-tenant organization management
- Controls, evidence, policies, and risk management
- Integration framework for automated compliance
- Real-time caching with Redis
- PostgreSQL database with SQLx migrations

## Tech Stack

- **Framework**: Axum 0.7 (async web framework)
- **Runtime**: Tokio (async runtime)
- **Database**: PostgreSQL with SQLx
- **Cache**: Redis
- **Authentication**: TitaniumVault integration
- **Logging**: Tracing with JSON output
- **Error Handling**: Custom error types with proper HTTP status codes

## Prerequisites

- Rust 1.75+ (2021 edition)
- PostgreSQL 14+
- Redis 6+
- Access to TitaniumVault API (for authentication)

## Getting Started

### 1. Clone and Setup

```bash
cd /home/mack/dev/opengrc/api
```

### 2. Configure Environment

Copy the example environment file and configure it:

```bash
cp .env.example .env
```

Edit `.env` and set the required values:

```bash
# Database (required)
DATABASE_URL=postgresql://postgres:password@localhost:5432/opengrc

# Redis (required)
REDIS_URL=redis://localhost:6379

# TitaniumVault (required for auth)
TV_API_URL=https://api.titanium-vault.com
TV_API_KEY=your_api_key_here

# Server (optional, has defaults)
HOST=0.0.0.0
PORT=8080

# CORS (optional)
CORS_ORIGINS=http://localhost:3000,http://localhost:3001

# Encryption (required in production)
# Generate with: openssl rand -hex 32
ENCRYPTION_KEY=your_64_character_hex_key_here

# Storage (optional, defaults to local)
STORAGE_TYPE=local                    # "local" or "s3"
STORAGE_LOCAL_PATH=./storage          # Path for local storage (default: ./storage)

# S3 Storage (required if STORAGE_TYPE=s3)
S3_BUCKET=opengrc-evidence
S3_REGION=us-east-1
S3_ENDPOINT=                          # Optional: for MinIO/LocalStack
AWS_ACCESS_KEY_ID=                    # Optional: uses IAM role if not set
AWS_SECRET_ACCESS_KEY=

# Logging (optional)
RUST_LOG=info,opengrc_api=debug
ENVIRONMENT=development
```

### 3. Set Up Database

Ensure PostgreSQL is running and the database exists:

```bash
# If using local PostgreSQL
createdb opengrc

# The migrations will run automatically on startup
# Or run them manually with:
sqlx migrate run
```

### 4. Run the API

```bash
cargo run
```

The server will start on `http://0.0.0.0:8080` (or your configured HOST:PORT).

### 5. Verify It's Running

Check the health endpoint:

```bash
curl http://localhost:8080/health
```

Expected response:
```json
{
  "status": "healthy",
  "database": "connected",
  "cache": "connected",
  "version": "1.9.0"
}
```

## Development

### Running in Development Mode

```bash
# Run with auto-reload (requires cargo-watch)
cargo install cargo-watch
cargo watch -x run

# Run with detailed logging
RUST_LOG=debug cargo run
```

### Database Migrations

```bash
# Create a new migration
sqlx migrate add <migration_name>

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Testing

```bash
# Run tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

### Code Quality

```bash
# Check for errors and warnings
cargo check

# Run clippy for linting
cargo clippy

# Format code
cargo fmt

# Fix auto-fixable issues
cargo fix
```

## Project Structure

```
api/
├── Cargo.toml                 # Dependencies and project metadata
├── .env.example               # Environment variable template
├── README.md                  # This file
├── migrations/                # SQLx database migrations
│   ├── 20251212184023_initial_schema.sql
│   └── 20251212204426_seed_soc2_framework.sql
└── src/
    ├── main.rs                # Server startup and graceful shutdown
    ├── lib.rs                 # Module exports
    ├── config/
    │   └── mod.rs            # Configuration management from env vars
    ├── routes/
    │   ├── mod.rs            # Router setup and route registration
    │   ├── health.rs         # Health check endpoint
    │   ├── sso.rs            # SSO authentication routes
    │   ├── auth.rs           # TitaniumVault authentication
    │   ├── frameworks.rs     # Framework & requirements (IMPLEMENTED)
    │   ├── controls.rs       # Controls CRUD (placeholder)
    │   ├── evidence.rs       # Evidence management (placeholder)
    │   ├── policies.rs       # Policy management (placeholder)
    │   ├── risks.rs          # Risk register (placeholder)
    │   ├── vendors.rs        # Vendor management (placeholder)
    │   ├── assets.rs         # Asset management (placeholder)
    │   ├── audits.rs         # Audit management (placeholder)
    │   └── integrations.rs   # Integration management (placeholder)
    ├── models/
    │   ├── mod.rs            # Database models (Organization, User, etc.)
    │   ├── framework.rs      # Framework & requirement models
    │   └── control.rs        # Control, test & mapping models
    ├── services/
    │   ├── mod.rs            # Business logic layer with AppServices
    │   ├── framework.rs      # Framework service with caching
    │   └── control.rs        # Control service with caching
    ├── middleware/
    │   ├── mod.rs            # Middleware exports
    │   ├── auth.rs           # JWT validation with TitaniumVault
    │   └── logging.rs        # Request/response logging
    ├── cache/
    │   └── mod.rs            # Redis cache utilities
    └── utils/
        ├── mod.rs            # Utility exports
        └── error.rs          # Custom error types and handlers
```

## API Endpoints

### Public Endpoints

- `GET /health` - Health check (database and cache status)

### Protected Endpoints (require TitaniumVault authentication)

All protected endpoints require an `Authorization: Bearer <token>` header.

#### Authentication
- `GET /api/v1/auth/me` - Get current authenticated user

#### Controls (Implemented)
- `GET /api/v1/controls` - List controls (supports `?status=`, `?control_type=`, `?owner_id=`, `?search=`, `?limit=`, `?offset=`)
- `GET /api/v1/controls/stats` - Get control statistics (total, implemented, in_progress, etc.)
- `GET /api/v1/controls/:id` - Get control with mapped requirements
- `POST /api/v1/controls` - Create control
- `PUT /api/v1/controls/:id` - Update control
- `DELETE /api/v1/controls/:id` - Delete control (fails if evidence linked)

#### Control Requirement Mappings (Implemented)
- `POST /api/v1/controls/:id/requirements` - Map control to framework requirements
- `DELETE /api/v1/controls/:id/requirements` - Unmap requirements from control

#### Control Tests (Implemented)
- `GET /api/v1/controls/:id/tests` - List tests for a control
- `POST /api/v1/controls/:id/tests` - Create a control test
- `POST /api/v1/controls/:control_id/tests/:test_id/results` - Record test result

#### Evidence
- `GET /api/v1/evidence` - List evidence
- `GET /api/v1/evidence/:id` - Get evidence by ID
- `POST /api/v1/evidence` - Create evidence
- `PUT /api/v1/evidence/:id` - Update evidence
- `DELETE /api/v1/evidence/:id` - Delete evidence

#### Policies
- `GET /api/v1/policies` - List policies
- `GET /api/v1/policies/:id` - Get policy by ID
- `POST /api/v1/policies` - Create policy
- `PUT /api/v1/policies/:id` - Update policy
- `DELETE /api/v1/policies/:id` - Delete policy

#### Risks
- `GET /api/v1/risks` - List risks
- `GET /api/v1/risks/:id` - Get risk by ID
- `POST /api/v1/risks` - Create risk
- `PUT /api/v1/risks/:id` - Update risk
- `DELETE /api/v1/risks/:id` - Delete risk

#### Vendors
- `GET /api/v1/vendors` - List vendors
- `GET /api/v1/vendors/:id` - Get vendor by ID
- `POST /api/v1/vendors` - Create vendor
- `PUT /api/v1/vendors/:id` - Update vendor
- `DELETE /api/v1/vendors/:id` - Delete vendor

#### Assets
- `GET /api/v1/assets` - List assets
- `GET /api/v1/assets/:id` - Get asset by ID
- `POST /api/v1/assets` - Create asset
- `PUT /api/v1/assets/:id` - Update asset
- `DELETE /api/v1/assets/:id` - Delete asset

#### Audits
- `GET /api/v1/audits` - List audits
- `GET /api/v1/audits/:id` - Get audit by ID
- `POST /api/v1/audits` - Create audit
- `PUT /api/v1/audits/:id` - Update audit
- `DELETE /api/v1/audits/:id` - Delete audit

#### Integrations (Implemented)
- `GET /api/v1/integrations` - List integrations (with sync stats)
- `GET /api/v1/integrations/available` - List available integration types (includes auth_methods and oauth_config)
- `GET /api/v1/integrations/stats` - Get integration statistics
- `GET /api/v1/integrations/:id` - Get integration by ID (masks sensitive config)
- `POST /api/v1/integrations` - Create integration
- `PUT /api/v1/integrations/:id` - Update integration
- `DELETE /api/v1/integrations/:id` - Delete integration
- `POST /api/v1/integrations/test` - Test connection with config (before creating)
- `POST /api/v1/integrations/:id/test` - Test connection for existing integration
- `POST /api/v1/integrations/:id/sync` - Trigger sync
- `GET /api/v1/integrations/:id/logs` - Get sync logs

##### OAuth2 Connection Flow (Implemented)
- `POST /api/v1/integrations/oauth/:type/authorize` - Start OAuth authorization flow
- `GET /api/v1/integrations/oauth/:type/check` - Check OAuth configuration status
- `GET /api/v1/integrations/oauth/callback` - OAuth callback handler (public endpoint)
- `POST /api/v1/integrations/:id/oauth/refresh` - Refresh OAuth tokens

**OAuth Environment Variables**:
```bash
# GitHub OAuth
GITHUB_OAUTH_CLIENT_ID=your_client_id
GITHUB_OAUTH_CLIENT_SECRET=your_client_secret

# GitLab OAuth
GITLAB_OAUTH_CLIENT_ID=your_client_id
GITLAB_OAUTH_CLIENT_SECRET=your_client_secret

# Google (GCP & Workspace) OAuth
GOOGLE_OAUTH_CLIENT_ID=your_client_id
GOOGLE_OAUTH_CLIENT_SECRET=your_client_secret

# Azure (Azure AD & Entra ID) OAuth
AZURE_OAUTH_CLIENT_ID=your_client_id
AZURE_OAUTH_CLIENT_SECRET=your_client_secret
AZURE_OAUTH_TENANT_ID=common  # or specific tenant

# Okta OAuth
OKTA_OAUTH_CLIENT_ID=your_client_id
OKTA_OAUTH_CLIENT_SECRET=your_client_secret

# Atlassian (Jira) OAuth
ATLASSIAN_OAUTH_CLIENT_ID=your_client_id
ATLASSIAN_OAUTH_CLIENT_SECRET=your_client_secret

# API Base URL (required for OAuth callbacks)
API_BASE_URL=https://api.your-domain.com
```

**OAuth Flow**:
1. Client calls `POST /api/v1/integrations/oauth/:type/authorize` with optional scopes and integration name
2. Server returns `authorization_url` - client redirects user to this URL
3. User authenticates with provider and grants access
4. Provider redirects to `/api/v1/integrations/oauth/callback` with code
5. Server exchanges code for tokens and creates integration
6. Server redirects to UI with success or error

**Supported OAuth Providers**: GitHub, GitLab, Google (GCP, Workspace), Azure (AD, Cloud), Okta, Atlassian (Jira)

##### Integration Health Monitoring (Implemented)
- `GET /api/v1/integrations/health` - Get health for all integrations (sorted by severity)
- `GET /api/v1/integrations/health/stats` - Get aggregated health statistics
- `GET /api/v1/integrations/health/failures` - Get recent failures (supports `?limit=`)
- `GET /api/v1/integrations/health/trend` - Get health trend data for charts (supports `?hours=`)
- `GET /api/v1/integrations/:id/health` - Get health for specific integration

**Health Status Types**:
- `healthy` - Last sync successful, error rate < 5%
- `degraded` - Last sync succeeded but error rate 5-20% or sync overdue
- `unhealthy` - 3+ consecutive failures or error rate > 20%
- `unknown` - Never synced

**Health Metrics**:
- Success rate (24h and 7d rolling windows)
- Consecutive failures count
- Average sync duration
- Last successful sync timestamp
- Last error message and timestamp

**Supported Integration Types**: AWS, GCP, Azure, Okta, Google Workspace, Azure AD, GitHub, GitLab, Jira, Cloudflare, Datadog, PagerDuty, Webhook

##### AWS Integration (Implemented v1.6.0)

Full AWS integration with automated evidence collection from 7 AWS services:

**AWS Services Supported**:
- IAM - Users, Roles, Policies, MFA status, Access Keys
- Security Hub - Security findings with severity tracking
- AWS Config - Compliance rules and resource compliance
- CloudTrail - Audit events with sensitive action detection
- S3 - Bucket inventory with encryption/versioning status
- EC2 - Instance inventory with security groups
- RDS - Database instances with encryption status

**AWS API Endpoints**:
- `GET /api/v1/integrations/:id/aws/overview` - Account overview with compliance stats
- `GET /api/v1/integrations/:id/aws/iam/users` - IAM users with MFA/access keys
- `GET /api/v1/integrations/:id/aws/iam/roles` - IAM roles
- `GET /api/v1/integrations/:id/aws/iam/policies` - IAM policies with risk analysis
- `GET /api/v1/integrations/:id/aws/findings` - Security Hub findings
- `GET /api/v1/integrations/:id/aws/findings/summary` - Findings summary by severity
- `GET /api/v1/integrations/:id/aws/config-rules` - AWS Config rule compliance
- `GET /api/v1/integrations/:id/aws/s3/buckets` - S3 bucket inventory
- `GET /api/v1/integrations/:id/aws/ec2/instances` - EC2 instance inventory
- `GET /api/v1/integrations/:id/aws/ec2/security-groups` - Security groups
- `GET /api/v1/integrations/:id/aws/rds/instances` - RDS database inventory
- `GET /api/v1/integrations/:id/aws/cloudtrail` - CloudTrail events
- `GET /api/v1/integrations/:id/aws/cloudtrail/stats` - CloudTrail statistics

**AWS Configuration**:
```json
{
  "auth_type": "access_key",  // or "assume_role"
  "access_key_id": "AKIA...",
  "secret_access_key": "...",
  "regions": ["us-east-1", "us-west-2"],  // optional, defaults to all regions
  // For assume_role auth:
  "role_arn": "arn:aws:iam::123456789012:role/OpenGRCRole"
}
```

**Required IAM Policy**:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "iam:Get*", "iam:List*",
        "securityhub:GetFindings", "securityhub:GetEnabledStandards",
        "config:Describe*", "config:Get*",
        "cloudtrail:LookupEvents",
        "s3:ListAllMyBuckets", "s3:GetBucket*",
        "ec2:Describe*",
        "rds:Describe*",
        "sts:GetCallerIdentity"
      ],
      "Resource": "*"
    }
  ]
}
```

**SOC 2 Control Mappings**: AWS findings are automatically mapped to SOC 2 controls (CC6.1, CC6.2, CC6.3, CC7.2, etc.)

##### GitHub Integration (Implemented v1.8.0)

Full GitHub integration with automated evidence collection for repository security and compliance:

**GitHub Services Supported**:
- Repositories - Repository inventory with visibility, settings, and metadata
- Branch Protection - Branch protection rules with required reviews, status checks
- Dependabot Alerts - Dependency vulnerability alerts with severity tracking
- Code Scanning - Code scanning alerts from GitHub Advanced Security
- Secret Scanning - Secret scanning alerts for exposed credentials
- Organization Members - Member list with role assignments and 2FA status

**GitHub API Endpoints**:
- `GET /api/v1/integrations/:id/github/overview` - Organization overview with stats
- `GET /api/v1/integrations/:id/github/repositories` - Repository inventory
- `GET /api/v1/integrations/:id/github/branch-protection` - Branch protection status
- `GET /api/v1/integrations/:id/github/dependabot` - Dependabot vulnerability alerts
- `GET /api/v1/integrations/:id/github/code-scanning` - Code scanning alerts
- `GET /api/v1/integrations/:id/github/secret-scanning` - Secret scanning alerts
- `GET /api/v1/integrations/:id/github/members` - Organization members

**GitHub Configuration**:
```json
{
  "access_token": "ghp_...",  // Personal access token or OAuth token
  "organization": "my-org",   // Optional: specific organization
  "repositories": ["repo1", "repo2"],  // Optional: specific repos (empty = all)
  "services": {
    "repositories": true,
    "branch_protection": true,
    "dependabot_alerts": true,
    "code_scanning": true,
    "secret_scanning": true,
    "members": true
  }
}
```

**Required GitHub Permissions** (for Personal Access Token):
- `repo` - Full control of private repositories
- `read:org` - Read organization membership
- `security_events` - Read security events (for code scanning)
- `admin:org` (optional) - For organization-level settings

**SOC 2 Control Mappings**:
- Repository settings → CC6.1 (Logical Access)
- Branch protection → CC8.1 (Change Management)
- Dependabot alerts → CC7.1 (Security Events)
- Code scanning → CC7.2 (System Monitoring)
- Secret scanning → CC6.7 (Security Incidents)
- Organization members → CC6.2 (Access Management)

##### Jira Integration (Implemented v1.8.0)

Full Jira integration for project tracking and compliance workflows:

**Jira Services Supported**:
- Projects - Project inventory with metadata and lead assignment
- Issues - Issue tracking with security-related filtering
- Users - User access and license management
- Permissions - Project role assignments and admin access

**Jira API Endpoints**:
- `GET /api/v1/integrations/:id/jira/overview` - Instance overview with stats
- `GET /api/v1/integrations/:id/jira/projects` - Project inventory
- `GET /api/v1/integrations/:id/jira/issues` - Issues (supports filtering)
- `GET /api/v1/integrations/:id/jira/users` - User access report
- `GET /api/v1/integrations/:id/jira/permissions` - Project permissions and roles

**Jira Configuration**:
```json
{
  "instance_url": "https://your-org.atlassian.net",
  "email": "user@example.com",
  "access_token": "ATATT3...",  // API token
  "auth_method": "api_token",   // or "oauth"
  "projects": ["PROJ1", "PROJ2"],  // Optional: specific projects (empty = all)
  "services": {
    "projects": true,
    "issues": true,
    "users": true,
    "permissions": true
  }
}
```

**Jira API Token**:
1. Go to https://id.atlassian.com/manage-profile/security/api-tokens
2. Create API token
3. Use your email and token for authentication

**SOC 2 Control Mappings**:
- Project settings → CC6.1 (Logical Access)
- Security issues → CC7.1 (Security Events)
- User access → CC6.2 (Access Management)
- Permissions → CC6.3 (Access Authorization)

##### Error Handling & Retry Logic (Implemented)

**Error Categories**:
- `transient` - Temporary errors (network, timeout) - automatic retry with exponential backoff
- `rate_limited` - Rate limit hit - retry with longer backoff
- `auth_failure` - Authentication error - may require re-authentication
- `config_error` - Configuration problem - no retry, user needs to fix
- `permanent` - Permanent error - no retry
- `unknown` - Unclassified error - retry allowed

**Retry Configuration** (per integration):
- `retry_enabled` - Enable/disable automatic retries (default: true)
- `max_retry_attempts` - Maximum retry attempts (default: 3)
- `retry_backoff_base_ms` - Base backoff delay (default: 1000ms)
- `retry_backoff_max_ms` - Maximum backoff delay (default: 300000ms / 5 min)

**Circuit Breaker**:
- Automatically opens after consecutive failures (threshold: 5)
- When open, sync requests are blocked
- Transitions to half-open after reset period (default: 10 min)
- Closes on successful sync

**Circuit Breaker States**:
- `closed` - Normal operation, requests allowed
- `open` - Failures exceeded threshold, requests blocked
- `half_open` - Testing recovery, limited requests allowed

#### Frameworks (Implemented)
- `GET /api/v1/frameworks` - List frameworks (supports `?category=` and `?is_system=` filters)
- `GET /api/v1/frameworks/:id` - Get framework with all requirements
- `POST /api/v1/frameworks` - Create framework
- `PUT /api/v1/frameworks/:id` - Update framework (cannot modify system frameworks)
- `DELETE /api/v1/frameworks/:id` - Delete framework (cannot delete system frameworks)

#### Framework Requirements (Implemented)
- `GET /api/v1/frameworks/:framework_id/requirements` - List requirements (supports `?tree=true` for hierarchical view)
- `GET /api/v1/frameworks/:framework_id/requirements/:id` - Get requirement by ID
- `POST /api/v1/frameworks/:framework_id/requirements` - Create requirement
- `POST /api/v1/frameworks/:framework_id/requirements/batch` - Batch create requirements
- `PUT /api/v1/frameworks/:framework_id/requirements/:id` - Update requirement
- `DELETE /api/v1/frameworks/:framework_id/requirements/:id` - Delete requirement

**Included Frameworks**: SOC 2 Trust Service Criteria (2017) with all 64 requirements across Security, Availability, Processing Integrity, Confidentiality, and Privacy categories.

**Note**: Most other endpoint implementations are currently placeholders. They return stub responses and need to be implemented with actual business logic.

## Architecture

### Request Flow

1. **Incoming Request** → CORS Layer
2. **Logging Middleware** → Logs request details (method, URI, IP)
3. **Tracing Layer** → Adds distributed tracing
4. **Compression Layer** → Gzip compression for responses
5. **Auth Middleware** → Validates JWT with TitaniumVault (protected routes only)
6. **Route Handler** → Processes request
7. **Response** → Returns JSON with appropriate status code

### Error Handling

The API uses a custom `AppError` enum that maps to appropriate HTTP status codes:

- `BadRequest(400)` - Invalid request data
- `Unauthorized(401)` - Missing or invalid authentication
- `Forbidden(403)` - Insufficient permissions
- `NotFound(404)` - Resource not found
- `Conflict(409)` - Resource conflict
- `InternalServerError(500)` - Server errors, database errors, cache errors

All errors return JSON:
```json
{
  "error": "Error message here"
}
```

### Database

- Uses SQLx for compile-time verified queries
- Connection pooling for performance
- Automatic migrations on startup
- Prepared statements for security

### File Storage

The API supports two storage backends for evidence files:

**Local Storage** (default, recommended for development):
```bash
STORAGE_TYPE=local
STORAGE_LOCAL_PATH=./storage    # Files stored at ./storage/orgs/{org_id}/evidence/{evidence_id}/
```

- Files stored on local filesystem
- No cloud credentials needed
- Download URLs route through API (`/api/v1/storage/download/...`)
- Great for development and testing

**S3 Storage** (recommended for production):
```bash
STORAGE_TYPE=s3
S3_BUCKET=opengrc-evidence
S3_REGION=us-east-1
AWS_ACCESS_KEY_ID=...           # Optional: uses IAM role if not set
AWS_SECRET_ACCESS_KEY=...
S3_ENDPOINT=http://localhost:9000  # Optional: for MinIO/LocalStack
```

- Files stored in S3 bucket
- Presigned URLs for direct upload/download (faster, no API bottleneck)
- Supports custom endpoints for MinIO/LocalStack in dev
- Uses IAM roles in production (no credentials needed in ECS)

### Caching Strategy

The `CacheClient` provides:
- `get<T>()` - Retrieve cached value
- `set<T>()` - Store value with optional TTL
- `delete()` - Remove single key
- `delete_pattern()` - Remove keys matching pattern
- `exists()` - Check if key exists
- `increment()` - Atomic increment
- `expire()` - Set TTL on existing key

Helper functions:
- `cache_key(prefix, id)` - Generate cache key
- `org_cache_key(org_id, entity, id)` - Generate multi-tenant cache key

### Authentication

Protected routes require a valid TitaniumVault JWT token:

```bash
curl -H "Authorization: Bearer <token>" http://localhost:8080/api/v1/auth/me
```

The auth middleware:
1. Extracts the Bearer token from the Authorization header
2. Validates the token with TitaniumVault API
3. Injects the authenticated user into the request extensions
4. Routes can access the user via `get_auth_user(&request)`

## Production Deployment

### Environment Variables

Ensure all required environment variables are set:
- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `TV_API_URL` - TitaniumVault API URL
- `TV_API_KEY` - TitaniumVault API key
- `CORS_ORIGINS` - Comma-separated list of allowed origins
- `ENCRYPTION_KEY` - 256-bit key for encrypting integration credentials (generate with `openssl rand -hex 32`)
- `ENVIRONMENT=production`

### Building for Production

```bash
# Build optimized binary
cargo build --release

# Binary will be at target/release/opengrc-api
./target/release/opengrc-api
```

### Docker

```dockerfile
# Build stage
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/opengrc-api /usr/local/bin/
CMD ["opengrc-api"]
```

### Health Checks

The `/health` endpoint is designed for load balancer health checks:
- Returns 200 OK when healthy
- Returns JSON with status of database and cache connections
- Use this for AWS ALB target group health checks

### Graceful Shutdown

The server handles SIGTERM and SIGINT signals for graceful shutdown:
- Stops accepting new connections
- Completes in-flight requests
- Closes database and cache connections
- Exits cleanly

Perfect for ECS container deployments.

### Logging

Structured JSON logging is enabled by default:
- CloudWatch-friendly format
- Request ID tracing
- Client IP from X-Forwarded-For header (ALB support)
- Configurable via RUST_LOG environment variable

## Performance Considerations

- **Async throughout** - All I/O operations are async for maximum throughput
- **Connection pooling** - Database and Redis connections are pooled
- **Compression** - Gzip compression for responses
- **Prepared statements** - SQLx uses prepared statements for security and performance
- **Cache-first patterns** - Check cache before hitting database
- **Multi-tenant isolation** - Organization-scoped queries and cache keys

## Next Steps

1. ~~Implement database migrations for all entities~~ ✓
2. ~~Implement Framework routes with caching~~ ✓
3. ~~Implement Controls routes with tests and mappings~~ ✓
4. ~~Implement Evidence, Policies, Risks routes~~ ✓
5. ~~Build Dashboard API endpoints~~ ✓
6. ~~Implement integration framework architecture~~ ✓ (v1.1.0)
7. ~~Add credential encryption for integration configs~~ ✓ (v1.2.0)
8. ~~Add integration health monitoring dashboard~~ ✓ (v1.3.0)
9. ~~Implement OAuth2 connection flow for integrations~~ ✓ (v1.4.0)
10. ~~Implement error handling & retry logic with circuit breaker~~ ✓ (v1.4.0)
11. ~~Implement AWS integration provider~~ ✓ (v1.6.0)
12. ~~Implement GitHub integration provider~~ ✓ (v1.8.0)
13. ~~Implement Jira integration provider~~ ✓ (v1.8.0)
14. Build scheduled sync job worker (cron-based)
15. Implement additional integration providers (Okta, GCP, Azure, etc.)
16. Add comprehensive tests (unit, integration, e2e)
17. Add OpenAPI/Swagger documentation
18. Set up CI/CD pipelines

## Contributing

This is part of the OpenGRC open-source project. See the main project README for contribution guidelines.

## License

Apache 2.0 or MIT (TBD)
