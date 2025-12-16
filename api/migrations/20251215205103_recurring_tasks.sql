-- Add recurrence fields to tasks table for recurring tasks functionality

-- Recurrence pattern enum-like check
-- Patterns: daily, weekly, monthly, quarterly, yearly, custom
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS recurrence_pattern VARCHAR(20);
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS recurrence_interval INTEGER; -- e.g., every 2 weeks
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS recurrence_day_of_week INTEGER; -- 0=Sunday, 6=Saturday (for weekly)
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS recurrence_day_of_month INTEGER; -- 1-31 (for monthly)
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS recurrence_month_of_year INTEGER; -- 1-12 (for yearly)
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS recurrence_end_at TIMESTAMPTZ; -- When to stop recurring (NULL = no end)
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS recurrence_count INTEGER; -- Max occurrences (NULL = unlimited)
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS recurrence_occurrences INTEGER DEFAULT 0; -- Current count of occurrences created
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS is_recurring BOOLEAN DEFAULT FALSE;
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS parent_task_id UUID REFERENCES tasks(id) ON DELETE SET NULL; -- Links occurrences to template
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS next_occurrence_at TIMESTAMPTZ; -- When the next task should be created
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS last_occurrence_at TIMESTAMPTZ; -- When the last occurrence was created

-- Index for efficient recurring task queries
CREATE INDEX IF NOT EXISTS idx_tasks_is_recurring ON tasks(organization_id, is_recurring) WHERE is_recurring = true;
CREATE INDEX IF NOT EXISTS idx_tasks_next_occurrence ON tasks(organization_id, next_occurrence_at) WHERE next_occurrence_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_tasks_parent_task ON tasks(parent_task_id) WHERE parent_task_id IS NOT NULL;

-- Add constraint for valid recurrence patterns
ALTER TABLE tasks ADD CONSTRAINT valid_recurrence_pattern
    CHECK (recurrence_pattern IS NULL OR recurrence_pattern IN ('daily', 'weekly', 'biweekly', 'monthly', 'quarterly', 'yearly'));

-- Add constraint for valid recurrence interval
ALTER TABLE tasks ADD CONSTRAINT valid_recurrence_interval
    CHECK (recurrence_interval IS NULL OR recurrence_interval > 0);

-- Add constraint for valid day of week
ALTER TABLE tasks ADD CONSTRAINT valid_recurrence_day_of_week
    CHECK (recurrence_day_of_week IS NULL OR (recurrence_day_of_week >= 0 AND recurrence_day_of_week <= 6));

-- Add constraint for valid day of month
ALTER TABLE tasks ADD CONSTRAINT valid_recurrence_day_of_month
    CHECK (recurrence_day_of_month IS NULL OR (recurrence_day_of_month >= 1 AND recurrence_day_of_month <= 31));

-- Add constraint for valid month of year
ALTER TABLE tasks ADD CONSTRAINT valid_recurrence_month_of_year
    CHECK (recurrence_month_of_year IS NULL OR (recurrence_month_of_year >= 1 AND recurrence_month_of_year <= 12));

-- Create a table to track recurrence history
CREATE TABLE IF NOT EXISTS task_recurrence_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    occurrence_number INTEGER NOT NULL,
    created_task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    scheduled_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    skipped BOOLEAN DEFAULT FALSE,
    skip_reason VARCHAR(500)
);

CREATE INDEX IF NOT EXISTS idx_task_recurrence_history_task ON task_recurrence_history(task_id);

COMMENT ON COLUMN tasks.recurrence_pattern IS 'Recurrence frequency: daily, weekly, biweekly, monthly, quarterly, yearly';
COMMENT ON COLUMN tasks.recurrence_interval IS 'Interval multiplier (e.g., 2 means every 2 weeks for weekly pattern)';
COMMENT ON COLUMN tasks.recurrence_day_of_week IS 'Day of week for weekly tasks (0=Sunday, 6=Saturday)';
COMMENT ON COLUMN tasks.recurrence_day_of_month IS 'Day of month for monthly tasks (1-31)';
COMMENT ON COLUMN tasks.recurrence_month_of_year IS 'Month for yearly tasks (1-12)';
COMMENT ON COLUMN tasks.recurrence_end_at IS 'Stop creating occurrences after this date';
COMMENT ON COLUMN tasks.recurrence_count IS 'Maximum number of occurrences to create';
COMMENT ON COLUMN tasks.is_recurring IS 'True if this is a recurring task template';
COMMENT ON COLUMN tasks.parent_task_id IS 'Links generated occurrences back to the recurring template';
COMMENT ON COLUMN tasks.next_occurrence_at IS 'When the next task occurrence should be created';
