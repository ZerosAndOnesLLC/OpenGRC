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
  "version": "0.2.0"
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
    │   └── framework.rs      # Framework & requirement models
    ├── services/
    │   ├── mod.rs            # Business logic layer with AppServices
    │   └── framework.rs      # Framework service with caching
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

#### Controls
- `GET /api/v1/controls` - List controls
- `GET /api/v1/controls/:id` - Get control by ID
- `POST /api/v1/controls` - Create control
- `PUT /api/v1/controls/:id` - Update control
- `DELETE /api/v1/controls/:id` - Delete control

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

#### Integrations
- `GET /api/v1/integrations` - List integrations
- `GET /api/v1/integrations/:id` - Get integration by ID
- `POST /api/v1/integrations` - Create integration
- `PUT /api/v1/integrations/:id` - Update integration
- `DELETE /api/v1/integrations/:id` - Delete integration

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
3. Implement Controls, Evidence, Policies, Risks routes
4. Build Dashboard API endpoints
5. Add comprehensive tests (unit, integration, e2e)
6. Add more middleware (rate limiting, request ID, etc.)
7. Implement integration framework for cloud providers
8. Add OpenAPI/Swagger documentation
9. Set up CI/CD pipelines

## Contributing

This is part of the OpenGRC open-source project. See the main project README for contribution guidelines.

## License

Apache 2.0 or MIT (TBD)
