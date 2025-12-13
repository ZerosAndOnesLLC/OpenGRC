-- OAuth2 and Retry Logic for Integrations
-- Adds support for OAuth2 connection flow and robust error handling with retries

-- ============================================================================
-- OAUTH2 SUPPORT
-- ============================================================================

-- OAuth state table for CSRF protection during OAuth authorization flow
CREATE TABLE integration_oauth_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    integration_type VARCHAR(100) NOT NULL,
    state VARCHAR(255) NOT NULL UNIQUE,
    code_verifier VARCHAR(255), -- For PKCE flow
    redirect_uri TEXT,
    scopes TEXT[], -- Requested scopes
    metadata JSONB, -- Additional metadata (e.g., integration name to create)
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_oauth_states_state ON integration_oauth_states(state);
CREATE INDEX idx_oauth_states_org ON integration_oauth_states(organization_id);
CREATE INDEX idx_oauth_states_expires ON integration_oauth_states(expires_at);

-- Add OAuth-related columns to integrations table
ALTER TABLE integrations
ADD COLUMN auth_method VARCHAR(50) NOT NULL DEFAULT 'api_key',
ADD COLUMN oauth_access_token TEXT,
ADD COLUMN oauth_refresh_token TEXT,
ADD COLUMN oauth_token_expires_at TIMESTAMPTZ,
ADD COLUMN oauth_scopes TEXT[],
ADD COLUMN oauth_metadata JSONB;

-- Add constraint to validate auth_method
ALTER TABLE integrations
ADD CONSTRAINT check_auth_method CHECK (auth_method IN ('api_key', 'oauth2', 'service_account'));

-- Index for finding integrations with expiring tokens (for proactive refresh)
CREATE INDEX idx_integrations_token_expires ON integrations(oauth_token_expires_at)
WHERE oauth_token_expires_at IS NOT NULL;

-- ============================================================================
-- RETRY LOGIC AND ERROR HANDLING
-- ============================================================================

-- Create error category enum for better error classification
CREATE TYPE sync_error_category AS ENUM (
    'transient',      -- Temporary errors (network, timeout) - should retry
    'rate_limited',   -- Rate limit hit - retry with backoff
    'auth_failure',   -- Authentication/authorization error - may need re-auth
    'config_error',   -- Configuration problem - user needs to fix
    'permanent',      -- Permanent error - don't retry
    'unknown'         -- Unclassified error
);

-- Add retry tracking to sync logs
ALTER TABLE integration_sync_logs
ADD COLUMN retry_attempt INTEGER NOT NULL DEFAULT 0,
ADD COLUMN max_retries INTEGER NOT NULL DEFAULT 3,
ADD COLUMN error_category sync_error_category,
ADD COLUMN next_retry_at TIMESTAMPTZ,
ADD COLUMN retry_backoff_ms INTEGER,
ADD COLUMN parent_sync_id UUID REFERENCES integration_sync_logs(id);

-- Index for finding syncs that need retry
CREATE INDEX idx_sync_logs_retry ON integration_sync_logs(next_retry_at, status)
WHERE next_retry_at IS NOT NULL AND status = 'failed';

-- Add retry configuration to integrations
ALTER TABLE integrations
ADD COLUMN retry_enabled BOOLEAN NOT NULL DEFAULT true,
ADD COLUMN max_retry_attempts INTEGER NOT NULL DEFAULT 3,
ADD COLUMN retry_backoff_base_ms INTEGER NOT NULL DEFAULT 1000,
ADD COLUMN retry_backoff_max_ms INTEGER NOT NULL DEFAULT 300000,
ADD COLUMN circuit_breaker_threshold INTEGER NOT NULL DEFAULT 5,
ADD COLUMN circuit_breaker_reset_ms INTEGER NOT NULL DEFAULT 600000;

-- Circuit breaker state
CREATE TYPE circuit_breaker_state AS ENUM ('closed', 'open', 'half_open');

ALTER TABLE integration_health
ADD COLUMN circuit_breaker_state circuit_breaker_state NOT NULL DEFAULT 'closed',
ADD COLUMN circuit_breaker_opened_at TIMESTAMPTZ,
ADD COLUMN circuit_breaker_half_open_at TIMESTAMPTZ;

-- ============================================================================
-- FUNCTIONS AND TRIGGERS
-- ============================================================================

-- Function to calculate exponential backoff with jitter
CREATE OR REPLACE FUNCTION calculate_retry_backoff(
    attempt INTEGER,
    base_ms INTEGER DEFAULT 1000,
    max_ms INTEGER DEFAULT 300000
) RETURNS INTEGER AS $$
DECLARE
    backoff_ms INTEGER;
    jitter_ms INTEGER;
BEGIN
    -- Exponential backoff: base * 2^attempt
    backoff_ms := base_ms * POWER(2, attempt);

    -- Cap at max
    IF backoff_ms > max_ms THEN
        backoff_ms := max_ms;
    END IF;

    -- Add random jitter (0-25% of backoff)
    jitter_ms := FLOOR(RANDOM() * backoff_ms * 0.25);

    RETURN backoff_ms + jitter_ms;
END;
$$ LANGUAGE plpgsql;

-- Function to classify errors into categories
CREATE OR REPLACE FUNCTION classify_sync_error(
    error_code TEXT,
    error_message TEXT
) RETURNS sync_error_category AS $$
BEGIN
    -- Rate limiting
    IF error_code IN ('429', 'rate_limit', 'too_many_requests') OR
       error_message ILIKE '%rate limit%' OR
       error_message ILIKE '%too many requests%' OR
       error_message ILIKE '%quota exceeded%' THEN
        RETURN 'rate_limited'::sync_error_category;
    END IF;

    -- Authentication failures
    IF error_code IN ('401', '403', 'unauthorized', 'forbidden', 'invalid_token') OR
       error_message ILIKE '%unauthorized%' OR
       error_message ILIKE '%forbidden%' OR
       error_message ILIKE '%invalid token%' OR
       error_message ILIKE '%token expired%' OR
       error_message ILIKE '%authentication%' THEN
        RETURN 'auth_failure'::sync_error_category;
    END IF;

    -- Configuration errors
    IF error_code IN ('400', '404', 'not_found', 'invalid_config') OR
       error_message ILIKE '%not found%' OR
       error_message ILIKE '%invalid config%' OR
       error_message ILIKE '%missing required%' THEN
        RETURN 'config_error'::sync_error_category;
    END IF;

    -- Transient errors (network, timeout, server errors)
    IF error_code IN ('408', '500', '502', '503', '504', 'timeout', 'connection') OR
       error_message ILIKE '%timeout%' OR
       error_message ILIKE '%connection%' OR
       error_message ILIKE '%network%' OR
       error_message ILIKE '%temporary%' OR
       error_message ILIKE '%server error%' THEN
        RETURN 'transient'::sync_error_category;
    END IF;

    -- Default to unknown
    RETURN 'unknown'::sync_error_category;
END;
$$ LANGUAGE plpgsql;

-- Function to determine if a sync should be retried
CREATE OR REPLACE FUNCTION should_retry_sync(
    p_sync_log_id UUID
) RETURNS BOOLEAN AS $$
DECLARE
    v_sync_log integration_sync_logs%ROWTYPE;
    v_integration integrations%ROWTYPE;
    v_health integration_health%ROWTYPE;
BEGIN
    -- Get sync log
    SELECT * INTO v_sync_log FROM integration_sync_logs WHERE id = p_sync_log_id;
    IF NOT FOUND THEN
        RETURN FALSE;
    END IF;

    -- Only retry failed syncs
    IF v_sync_log.status != 'failed' THEN
        RETURN FALSE;
    END IF;

    -- Check retry attempts
    IF v_sync_log.retry_attempt >= v_sync_log.max_retries THEN
        RETURN FALSE;
    END IF;

    -- Get integration
    SELECT * INTO v_integration FROM integrations WHERE id = v_sync_log.integration_id;
    IF NOT FOUND OR NOT v_integration.retry_enabled THEN
        RETURN FALSE;
    END IF;

    -- Check circuit breaker
    SELECT * INTO v_health FROM integration_health WHERE integration_id = v_sync_log.integration_id;
    IF FOUND AND v_health.circuit_breaker_state = 'open' THEN
        -- Check if we should transition to half-open
        IF v_health.circuit_breaker_opened_at IS NOT NULL AND
           NOW() > v_health.circuit_breaker_opened_at + (v_integration.circuit_breaker_reset_ms || ' milliseconds')::interval THEN
            -- Transition to half-open (allow one retry)
            UPDATE integration_health
            SET circuit_breaker_state = 'half_open',
                circuit_breaker_half_open_at = NOW()
            WHERE integration_id = v_sync_log.integration_id;
            RETURN TRUE;
        END IF;
        RETURN FALSE;
    END IF;

    -- Don't retry permanent errors or config errors
    IF v_sync_log.error_category IN ('permanent', 'config_error') THEN
        RETURN FALSE;
    END IF;

    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update circuit breaker on sync completion
CREATE OR REPLACE FUNCTION update_circuit_breaker() RETURNS TRIGGER AS $$
DECLARE
    v_integration integrations%ROWTYPE;
    v_health integration_health%ROWTYPE;
BEGIN
    -- Only process status changes to completed or failed
    IF NEW.status NOT IN ('completed', 'failed') THEN
        RETURN NEW;
    END IF;

    -- Get integration config
    SELECT * INTO v_integration FROM integrations WHERE id = NEW.integration_id;
    IF NOT FOUND THEN
        RETURN NEW;
    END IF;

    -- Get current health
    SELECT * INTO v_health FROM integration_health WHERE integration_id = NEW.integration_id;

    IF NEW.status = 'completed' THEN
        -- Success: close circuit breaker
        IF v_health.circuit_breaker_state != 'closed' THEN
            UPDATE integration_health
            SET circuit_breaker_state = 'closed',
                circuit_breaker_opened_at = NULL,
                circuit_breaker_half_open_at = NULL
            WHERE integration_id = NEW.integration_id;
        END IF;
    ELSIF NEW.status = 'failed' THEN
        -- Failure: check if we should open circuit breaker
        IF v_health.consecutive_failures >= v_integration.circuit_breaker_threshold THEN
            UPDATE integration_health
            SET circuit_breaker_state = 'open',
                circuit_breaker_opened_at = NOW()
            WHERE integration_id = NEW.integration_id;
        END IF;

        -- If in half-open state and failed, go back to open
        IF v_health.circuit_breaker_state = 'half_open' THEN
            UPDATE integration_health
            SET circuit_breaker_state = 'open',
                circuit_breaker_opened_at = NOW()
            WHERE integration_id = NEW.integration_id;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_circuit_breaker
    AFTER UPDATE OF status ON integration_sync_logs
    FOR EACH ROW
    EXECUTE FUNCTION update_circuit_breaker();

-- Trigger to schedule retry on failure
CREATE OR REPLACE FUNCTION schedule_sync_retry() RETURNS TRIGGER AS $$
DECLARE
    v_integration integrations%ROWTYPE;
    v_backoff_ms INTEGER;
BEGIN
    -- Only process failed syncs
    IF NEW.status != 'failed' THEN
        RETURN NEW;
    END IF;

    -- Check if should retry
    IF NOT should_retry_sync(NEW.id) THEN
        RETURN NEW;
    END IF;

    -- Get integration for retry config
    SELECT * INTO v_integration FROM integrations WHERE id = NEW.integration_id;

    -- Calculate backoff
    v_backoff_ms := calculate_retry_backoff(
        NEW.retry_attempt,
        v_integration.retry_backoff_base_ms,
        v_integration.retry_backoff_max_ms
    );

    -- Update sync log with retry info
    UPDATE integration_sync_logs
    SET next_retry_at = NOW() + (v_backoff_ms || ' milliseconds')::interval,
        retry_backoff_ms = v_backoff_ms
    WHERE id = NEW.id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_schedule_sync_retry
    AFTER UPDATE OF status ON integration_sync_logs
    FOR EACH ROW
    WHEN (NEW.status = 'failed')
    EXECUTE FUNCTION schedule_sync_retry();

-- ============================================================================
-- CLEANUP
-- ============================================================================

-- Clean up expired OAuth states (can be called periodically)
CREATE OR REPLACE FUNCTION cleanup_expired_oauth_states() RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM integration_oauth_states
    WHERE expires_at < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Index for efficient cleanup
CREATE INDEX idx_oauth_states_cleanup ON integration_oauth_states(expires_at);
