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
  "version": "1.4.0"
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
11. Build scheduled sync job worker (cron-based)
12. Implement actual integration providers (AWS, GitHub, Okta, etc.)
13. Add comprehensive tests (unit, integration, e2e)
14. Add OpenAPI/Swagger documentation
15. Set up CI/CD pipelines

## Contributing

This is part of the OpenGRC open-source project. See the main project README for contribution guidelines.

## License

Apache 2.0 or MIT (TBD)
