-- Integration health tracking table
-- Stores health snapshots for monitoring integration status over time

CREATE TYPE health_status AS ENUM ('healthy', 'degraded', 'unhealthy', 'unknown');

CREATE TABLE integration_health (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,

    -- Current health status
    status health_status NOT NULL DEFAULT 'unknown',

    -- Sync metrics
    last_successful_sync_at TIMESTAMPTZ,
    consecutive_failures INTEGER NOT NULL DEFAULT 0,

    -- Performance metrics (rolling 24h window)
    sync_success_count_24h INTEGER NOT NULL DEFAULT 0,
    sync_failure_count_24h INTEGER NOT NULL DEFAULT 0,
    average_sync_duration_ms INTEGER,

    -- Performance metrics (rolling 7d window)
    sync_success_count_7d INTEGER NOT NULL DEFAULT 0,
    sync_failure_count_7d INTEGER NOT NULL DEFAULT 0,

    -- Last check details
    last_check_at TIMESTAMPTZ,
    last_check_message TEXT,
    last_error_at TIMESTAMPTZ,
    last_error_message TEXT,

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- One health record per integration
    UNIQUE(integration_id)
);

-- Index for fast lookups by status
CREATE INDEX idx_integration_health_status ON integration_health(status);
CREATE INDEX idx_integration_health_integration ON integration_health(integration_id);

-- Trigger to auto-update updated_at
CREATE TRIGGER update_integration_health_updated_at
    BEFORE UPDATE ON integration_health
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- Integration health history for trend analysis
CREATE TABLE integration_health_snapshots (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,

    -- Snapshot data
    status health_status NOT NULL,
    sync_success_rate DECIMAL(5,2), -- percentage 0.00 to 100.00
    average_sync_duration_ms INTEGER,
    error_count INTEGER NOT NULL DEFAULT 0,

    -- Snapshot timestamp
    snapshot_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for time-series queries
CREATE INDEX idx_health_snapshots_integration_time
    ON integration_health_snapshots(integration_id, snapshot_at DESC);

-- Partition by month for efficient storage (optional - can be enabled later)
-- This is prepared for high-volume deployments

-- Function to calculate health status based on metrics
CREATE OR REPLACE FUNCTION calculate_health_status(
    p_consecutive_failures INTEGER,
    p_success_count_24h INTEGER,
    p_failure_count_24h INTEGER,
    p_last_successful_sync TIMESTAMPTZ
) RETURNS health_status AS $$
DECLARE
    total_syncs INTEGER;
    error_rate DECIMAL;
    hours_since_success DECIMAL;
BEGIN
    -- If never synced, status is unknown
    IF p_last_successful_sync IS NULL THEN
        RETURN 'unknown'::health_status;
    END IF;

    -- If 3+ consecutive failures, unhealthy
    IF p_consecutive_failures >= 3 THEN
        RETURN 'unhealthy'::health_status;
    END IF;

    -- Calculate error rate
    total_syncs := p_success_count_24h + p_failure_count_24h;
    IF total_syncs > 0 THEN
        error_rate := (p_failure_count_24h::DECIMAL / total_syncs) * 100;
    ELSE
        error_rate := 0;
    END IF;

    -- If error rate > 20%, unhealthy
    IF error_rate > 20 THEN
        RETURN 'unhealthy'::health_status;
    END IF;

    -- If error rate 5-20%, degraded
    IF error_rate >= 5 THEN
        RETURN 'degraded'::health_status;
    END IF;

    -- Check if sync is overdue (no sync in 25 hours for daily schedule)
    hours_since_success := EXTRACT(EPOCH FROM (NOW() - p_last_successful_sync)) / 3600;
    IF hours_since_success > 25 THEN
        RETURN 'degraded'::health_status;
    END IF;

    RETURN 'healthy'::health_status;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Function to update health metrics after a sync completes
CREATE OR REPLACE FUNCTION update_integration_health_on_sync() RETURNS TRIGGER AS $$
DECLARE
    v_success BOOLEAN;
    v_duration_ms INTEGER;
    v_health_record integration_health%ROWTYPE;
BEGIN
    -- Determine if sync was successful
    v_success := NEW.status = 'completed';

    -- Calculate duration if completed
    IF NEW.completed_at IS NOT NULL THEN
        v_duration_ms := EXTRACT(EPOCH FROM (NEW.completed_at - NEW.started_at)) * 1000;
    END IF;

    -- Upsert health record
    INSERT INTO integration_health (
        integration_id,
        status,
        last_successful_sync_at,
        consecutive_failures,
        sync_success_count_24h,
        sync_failure_count_24h,
        average_sync_duration_ms,
        last_check_at,
        last_check_message,
        last_error_at,
        last_error_message
    )
    VALUES (
        NEW.integration_id,
        'unknown',
        CASE WHEN v_success THEN NEW.completed_at ELSE NULL END,
        CASE WHEN v_success THEN 0 ELSE 1 END,
        CASE WHEN v_success THEN 1 ELSE 0 END,
        CASE WHEN v_success THEN 0 ELSE 1 END,
        v_duration_ms,
        NOW(),
        NEW.status,
        CASE WHEN NOT v_success THEN NOW() ELSE NULL END,
        CASE WHEN NOT v_success THEN NEW.errors::TEXT ELSE NULL END
    )
    ON CONFLICT (integration_id) DO UPDATE SET
        last_successful_sync_at = CASE
            WHEN v_success THEN NEW.completed_at
            ELSE integration_health.last_successful_sync_at
        END,
        consecutive_failures = CASE
            WHEN v_success THEN 0
            ELSE integration_health.consecutive_failures + 1
        END,
        sync_success_count_24h = CASE
            WHEN v_success THEN integration_health.sync_success_count_24h + 1
            ELSE integration_health.sync_success_count_24h
        END,
        sync_failure_count_24h = CASE
            WHEN v_success THEN integration_health.sync_failure_count_24h
            ELSE integration_health.sync_failure_count_24h + 1
        END,
        average_sync_duration_ms = CASE
            WHEN v_success AND v_duration_ms IS NOT NULL THEN
                COALESCE((integration_health.average_sync_duration_ms + v_duration_ms) / 2, v_duration_ms)
            ELSE integration_health.average_sync_duration_ms
        END,
        last_check_at = NOW(),
        last_check_message = NEW.status,
        last_error_at = CASE WHEN NOT v_success THEN NOW() ELSE integration_health.last_error_at END,
        last_error_message = CASE WHEN NOT v_success THEN NEW.errors::TEXT ELSE integration_health.last_error_message END,
        updated_at = NOW();

    -- Get the updated health record to calculate status
    SELECT * INTO v_health_record
    FROM integration_health
    WHERE integration_id = NEW.integration_id;

    -- Update status based on calculated health
    UPDATE integration_health
    SET status = calculate_health_status(
        v_health_record.consecutive_failures,
        v_health_record.sync_success_count_24h,
        v_health_record.sync_failure_count_24h,
        v_health_record.last_successful_sync_at
    )
    WHERE integration_id = NEW.integration_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update health when sync log is updated (completed or failed)
CREATE TRIGGER update_health_on_sync_complete
    AFTER UPDATE OF status ON integration_sync_logs
    FOR EACH ROW
    WHEN (OLD.status = 'running' AND NEW.status IN ('completed', 'failed'))
    EXECUTE FUNCTION update_integration_health_on_sync();

-- Also trigger on insert with final status (for direct inserts)
CREATE TRIGGER update_health_on_sync_insert
    AFTER INSERT ON integration_sync_logs
    FOR EACH ROW
    WHEN (NEW.status IN ('completed', 'failed'))
    EXECUTE FUNCTION update_integration_health_on_sync();

-- Function to reset 24h counters (should be called by a scheduled job)
CREATE OR REPLACE FUNCTION reset_health_24h_counters() RETURNS void AS $$
BEGIN
    -- Move 24h stats to 7d before resetting
    UPDATE integration_health
    SET
        sync_success_count_7d = sync_success_count_7d + sync_success_count_24h,
        sync_failure_count_7d = sync_failure_count_7d + sync_failure_count_24h,
        sync_success_count_24h = 0,
        sync_failure_count_24h = 0,
        updated_at = NOW();
END;
$$ LANGUAGE plpgsql;

-- Function to reset 7d counters (should be called weekly by a scheduled job)
CREATE OR REPLACE FUNCTION reset_health_7d_counters() RETURNS void AS $$
BEGIN
    UPDATE integration_health
    SET
        sync_success_count_7d = 0,
        sync_failure_count_7d = 0,
        updated_at = NOW();
END;
$$ LANGUAGE plpgsql;

-- Create initial health records for existing integrations
INSERT INTO integration_health (integration_id, status)
SELECT id, 'unknown'::health_status
FROM integrations
ON CONFLICT (integration_id) DO NOTHING;
