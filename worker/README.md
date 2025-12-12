# OpenGRC Worker

Background job processor for OpenGRC. Handles async tasks like evidence collection, control testing, integration syncing, and notifications.

## Job Types

| Job Type | Description |
|----------|-------------|
| `evidence_collection` | Collect evidence from integrations (AWS, GitHub, etc.) |
| `control_testing` | Run automated control tests |
| `integration_sync` | Sync data from external integrations |
| `send_notification` | Send email/Slack notifications |
| `access_review_reminder` | Send access review reminders |
| `policy_acknowledgment_reminder` | Remind users to acknowledge policies |
| `report_generation` | Generate compliance reports |

## Prerequisites

- Rust 1.75+
- PostgreSQL 16+
- Redis 7+

## Setup

```bash
# Copy environment file
cp .env.example .env

# Edit .env with your settings
vim .env
```

## Development

```bash
# Run the worker
cargo run

# Run with debug logging
RUST_LOG=debug cargo run
```

## Architecture

The worker uses a Redis-based job queue:

1. **API enqueues jobs** to `opengrc:jobs`
2. **Worker pops jobs** and moves to `opengrc:jobs:processing`
3. **On success**, job is removed from processing queue
4. **On failure**, job is retried (up to max_attempts)

### Adding New Job Types

1. Add variant to `JobType` enum in `src/queue.rs`
2. Create new module in `src/jobs/`
3. Add match arm in `JobQueue::execute_job()`

## Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | Required |
| `DATABASE_MAX_CONNECTIONS` | DB pool size | `5` |
| `REDIS_URL` | Redis connection string | `redis://localhost:6379` |
| `WORKER_CONCURRENCY` | Concurrent job processors | `4` |
| `RUST_LOG` | Log level | `info` |
