-- Collaboration Features Migration
-- Adds support for comments & mentions, real-time collaboration, Slack/Teams integrations, and email digests

-- =====================================================
-- ENTITY COMMENTS (Generic comments for all entities)
-- =====================================================

CREATE TABLE entity_comments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL, -- control, evidence, policy, risk, audit, vendor, asset, audit_request, audit_finding
    entity_id UUID NOT NULL,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    -- For threaded replies
    parent_comment_id UUID REFERENCES entity_comments(id) ON DELETE CASCADE,
    -- Soft delete support
    deleted_at TIMESTAMPTZ,
    deleted_by UUID REFERENCES users(id),
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_entity_comments_org ON entity_comments(organization_id);
CREATE INDEX idx_entity_comments_entity ON entity_comments(entity_type, entity_id);
CREATE INDEX idx_entity_comments_user ON entity_comments(user_id);
CREATE INDEX idx_entity_comments_parent ON entity_comments(parent_comment_id) WHERE parent_comment_id IS NOT NULL;
CREATE INDEX idx_entity_comments_created ON entity_comments(entity_type, entity_id, created_at);

CREATE TRIGGER update_entity_comments_updated_at BEFORE UPDATE ON entity_comments FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =====================================================
-- COMMENT MENTIONS (@mentions in comments)
-- =====================================================

CREATE TABLE comment_mentions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    comment_id UUID NOT NULL REFERENCES entity_comments(id) ON DELETE CASCADE,
    mentioned_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(comment_id, mentioned_user_id)
);

CREATE INDEX idx_comment_mentions_user ON comment_mentions(mentioned_user_id);
CREATE INDEX idx_comment_mentions_comment ON comment_mentions(comment_id);

-- =====================================================
-- NOTIFICATION PREFERENCES
-- =====================================================

CREATE TABLE notification_preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Channel preferences
    in_app_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    email_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    slack_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    teams_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    -- Notification type preferences (JSON for flexibility)
    -- Example: {"task_assigned": true, "comment_mention": true, "policy_reminder": true}
    enabled_types JSONB NOT NULL DEFAULT '{}',
    -- Email digest preferences
    email_digest_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    email_digest_frequency VARCHAR(20) DEFAULT 'daily', -- daily, weekly, none
    email_digest_day_of_week INTEGER, -- 0-6 for weekly digests
    email_digest_hour INTEGER DEFAULT 9, -- Hour of day (0-23)
    -- Quiet hours (no notifications during these hours)
    quiet_hours_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    quiet_hours_start INTEGER, -- Hour of day (0-23)
    quiet_hours_end INTEGER, -- Hour of day (0-23)
    quiet_hours_timezone VARCHAR(50) DEFAULT 'UTC',
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, user_id)
);

CREATE INDEX idx_notification_preferences_user ON notification_preferences(user_id);
CREATE INDEX idx_notification_preferences_digest ON notification_preferences(email_digest_enabled, email_digest_frequency)
    WHERE email_digest_enabled = TRUE;

CREATE TRIGGER update_notification_preferences_updated_at BEFORE UPDATE ON notification_preferences FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- =====================================================
-- EMAIL DIGESTS
-- =====================================================

CREATE TABLE email_digests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    digest_type VARCHAR(20) NOT NULL, -- daily, weekly
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    -- Content summary
    notification_count INTEGER NOT NULL DEFAULT 0,
    task_count INTEGER NOT NULL DEFAULT 0,
    comment_count INTEGER NOT NULL DEFAULT 0,
    mention_count INTEGER NOT NULL DEFAULT 0,
    -- Full content as JSON for rendering
    content JSONB NOT NULL DEFAULT '{}',
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, sent, failed
    sent_at TIMESTAMPTZ,
    error_message TEXT,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_email_digests_user ON email_digests(user_id, created_at);
CREATE INDEX idx_email_digests_status ON email_digests(status) WHERE status = 'pending';
CREATE INDEX idx_email_digests_period ON email_digests(user_id, period_start, period_end);

-- =====================================================
-- SLACK INTEGRATION
-- =====================================================

CREATE TABLE slack_workspaces (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    team_id VARCHAR(50) NOT NULL, -- Slack team ID
    team_name VARCHAR(255) NOT NULL,
    -- OAuth tokens (encrypted)
    access_token TEXT NOT NULL,
    bot_user_id VARCHAR(50),
    bot_access_token TEXT,
    -- Webhook for incoming messages
    incoming_webhook_url TEXT,
    incoming_webhook_channel VARCHAR(100),
    -- Configuration
    default_channel_id VARCHAR(50),
    default_channel_name VARCHAR(100),
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, inactive, revoked
    last_activity_at TIMESTAMPTZ,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, team_id)
);

CREATE INDEX idx_slack_workspaces_org ON slack_workspaces(organization_id);
CREATE INDEX idx_slack_workspaces_team ON slack_workspaces(team_id);

CREATE TRIGGER update_slack_workspaces_updated_at BEFORE UPDATE ON slack_workspaces FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Map notification types to Slack channels
CREATE TABLE slack_channel_mappings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workspace_id UUID NOT NULL REFERENCES slack_workspaces(id) ON DELETE CASCADE,
    notification_type VARCHAR(100) NOT NULL, -- task_assigned, comment_mention, policy_reminder, security_alert, etc.
    channel_id VARCHAR(50) NOT NULL,
    channel_name VARCHAR(100) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, notification_type)
);

CREATE INDEX idx_slack_channel_mappings_workspace ON slack_channel_mappings(workspace_id);

CREATE TRIGGER update_slack_channel_mappings_updated_at BEFORE UPDATE ON slack_channel_mappings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- User-level Slack connections (for DMs)
CREATE TABLE slack_user_connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    workspace_id UUID NOT NULL REFERENCES slack_workspaces(id) ON DELETE CASCADE,
    slack_user_id VARCHAR(50) NOT NULL,
    slack_username VARCHAR(100),
    dm_channel_id VARCHAR(50), -- For direct messages
    connected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, workspace_id)
);

CREATE INDEX idx_slack_user_connections_user ON slack_user_connections(user_id);
CREATE INDEX idx_slack_user_connections_workspace ON slack_user_connections(workspace_id);

-- =====================================================
-- MICROSOFT TEAMS INTEGRATION
-- =====================================================

CREATE TABLE teams_tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    tenant_id VARCHAR(50) NOT NULL, -- Microsoft tenant ID
    tenant_name VARCHAR(255),
    -- OAuth tokens (encrypted)
    access_token TEXT NOT NULL,
    refresh_token TEXT,
    token_expires_at TIMESTAMPTZ,
    -- Bot configuration
    bot_id VARCHAR(100),
    service_url VARCHAR(500),
    -- Webhook for incoming messages
    incoming_webhook_url TEXT,
    -- Configuration
    default_team_id VARCHAR(100),
    default_channel_id VARCHAR(100),
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'active', -- active, inactive, revoked
    last_activity_at TIMESTAMPTZ,
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, tenant_id)
);

CREATE INDEX idx_teams_tenants_org ON teams_tenants(organization_id);
CREATE INDEX idx_teams_tenants_tenant ON teams_tenants(tenant_id);

CREATE TRIGGER update_teams_tenants_updated_at BEFORE UPDATE ON teams_tenants FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Map notification types to Teams channels
CREATE TABLE teams_channel_mappings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES teams_tenants(id) ON DELETE CASCADE,
    notification_type VARCHAR(100) NOT NULL,
    team_id VARCHAR(100) NOT NULL,
    channel_id VARCHAR(100) NOT NULL,
    channel_name VARCHAR(255),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, notification_type)
);

CREATE INDEX idx_teams_channel_mappings_tenant ON teams_channel_mappings(tenant_id);

CREATE TRIGGER update_teams_channel_mappings_updated_at BEFORE UPDATE ON teams_channel_mappings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- User-level Teams connections
CREATE TABLE teams_user_connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES teams_tenants(id) ON DELETE CASCADE,
    teams_user_id VARCHAR(100) NOT NULL,
    conversation_id VARCHAR(200), -- For 1:1 chats
    connected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, tenant_id)
);

CREATE INDEX idx_teams_user_connections_user ON teams_user_connections(user_id);
CREATE INDEX idx_teams_user_connections_tenant ON teams_user_connections(tenant_id);

-- =====================================================
-- REAL-TIME COLLABORATION
-- =====================================================

-- Track active WebSocket sessions
CREATE TABLE websocket_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_token VARCHAR(100) NOT NULL UNIQUE,
    -- Connection info
    connected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- User agent info
    ip_address VARCHAR(45),
    user_agent TEXT,
    -- Status
    status VARCHAR(20) NOT NULL DEFAULT 'connected' -- connected, disconnected
);

CREATE INDEX idx_websocket_sessions_user ON websocket_sessions(user_id);
CREATE INDEX idx_websocket_sessions_org ON websocket_sessions(organization_id);
CREATE INDEX idx_websocket_sessions_token ON websocket_sessions(session_token);
CREATE INDEX idx_websocket_sessions_heartbeat ON websocket_sessions(last_heartbeat_at) WHERE status = 'connected';

-- Track presence (who's viewing what)
CREATE TABLE collaboration_presence (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    session_id UUID NOT NULL REFERENCES websocket_sessions(id) ON DELETE CASCADE,
    -- What they're viewing
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    -- Activity tracking
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Editing status
    is_editing BOOLEAN NOT NULL DEFAULT FALSE,
    editing_field VARCHAR(100), -- Which field they're editing (for conflict resolution)
    UNIQUE(session_id, entity_type, entity_id)
);

CREATE INDEX idx_collaboration_presence_entity ON collaboration_presence(entity_type, entity_id);
CREATE INDEX idx_collaboration_presence_user ON collaboration_presence(user_id);
CREATE INDEX idx_collaboration_presence_session ON collaboration_presence(session_id);

-- Collaboration events for real-time sync
CREATE TABLE collaboration_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    -- Event details
    event_type VARCHAR(50) NOT NULL, -- comment_added, entity_updated, user_joined, user_left, typing_started, typing_stopped
    entity_type VARCHAR(50),
    entity_id UUID,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    -- Event data
    data JSONB NOT NULL DEFAULT '{}',
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Partition by time for efficient cleanup
CREATE INDEX idx_collaboration_events_org ON collaboration_events(organization_id, created_at);
CREATE INDEX idx_collaboration_events_entity ON collaboration_events(entity_type, entity_id, created_at);

-- =====================================================
-- ADD NOTIFICATION TYPE COLUMN TO NOTIFICATIONS
-- =====================================================

-- Add data column to notifications if not exists (for backward compatibility)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'notifications' AND column_name = 'data') THEN
        ALTER TABLE notifications ADD COLUMN data JSONB DEFAULT '{}';
    END IF;
END $$;

-- =====================================================
-- EMAIL DIGEST TEMPLATES
-- =====================================================

-- Insert default email digest templates
INSERT INTO email_templates (organization_id, template_type, subject, body_html, body_text, is_system)
VALUES
(NULL, 'daily_digest',
 '[OpenGRC] Your Daily Digest - {{date}}',
 '<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #333;">Your Daily Digest</h1>
<p>Hi {{user_name}},</p>
<p>Here''s a summary of activity from {{period_start}} to {{period_end}}:</p>

{{#if tasks_due}}
<h2 style="color: #2563eb;">Tasks Due</h2>
<ul>
{{#each tasks_due}}
<li><a href="{{../app_url}}/tasks/{{this.id}}/">{{this.title}}</a> - Due: {{this.due_at}}</li>
{{/each}}
</ul>
{{/if}}

{{#if mentions}}
<h2 style="color: #2563eb;">Mentions</h2>
<ul>
{{#each mentions}}
<li><strong>{{this.mentioned_by}}</strong> mentioned you in {{this.entity_type}}: <a href="{{../app_url}}/{{this.entity_type}}/{{this.entity_id}}/">{{this.entity_title}}</a></li>
{{/each}}
</ul>
{{/if}}

{{#if comments}}
<h2 style="color: #2563eb;">Recent Comments</h2>
<ul>
{{#each comments}}
<li><strong>{{this.user_name}}</strong> commented on {{this.entity_type}}: <a href="{{../app_url}}/{{this.entity_type}}/{{this.entity_id}}/">{{this.entity_title}}</a></li>
{{/each}}
</ul>
{{/if}}

{{#if notifications}}
<h2 style="color: #2563eb;">Other Notifications ({{notification_count}})</h2>
<ul>
{{#each notifications}}
<li>{{this.title}}</li>
{{/each}}
</ul>
{{/if}}

<p style="margin-top: 30px; color: #666;">
<a href="{{app_url}}/settings/notifications/">Manage notification preferences</a>
</p>
</body>
</html>',
 'Your Daily Digest

Hi {{user_name}},

Here''s a summary of activity from {{period_start}} to {{period_end}}:

{{#if tasks_due}}
TASKS DUE:
{{#each tasks_due}}
- {{this.title}} - Due: {{this.due_at}}
{{/each}}
{{/if}}

{{#if mentions}}
MENTIONS:
{{#each mentions}}
- {{this.mentioned_by}} mentioned you in {{this.entity_type}}: {{this.entity_title}}
{{/each}}
{{/if}}

{{#if comments}}
RECENT COMMENTS:
{{#each comments}}
- {{this.user_name}} commented on {{this.entity_type}}: {{this.entity_title}}
{{/each}}
{{/if}}

{{#if notifications}}
OTHER NOTIFICATIONS ({{notification_count}}):
{{#each notifications}}
- {{this.title}}
{{/each}}
{{/if}}

Manage notification preferences: {{app_url}}/settings/notifications/',
 TRUE),

(NULL, 'weekly_digest',
 '[OpenGRC] Your Weekly Digest - Week of {{week_start}}',
 '<!DOCTYPE html>
<html>
<head><meta charset="utf-8"></head>
<body style="font-family: Arial, sans-serif; max-width: 600px; margin: 0 auto; padding: 20px;">
<h1 style="color: #333;">Your Weekly Digest</h1>
<p>Hi {{user_name}},</p>
<p>Here''s a summary of activity from {{period_start}} to {{period_end}}:</p>

<div style="background: #f3f4f6; padding: 15px; border-radius: 8px; margin: 20px 0;">
<h3 style="margin-top: 0;">This Week at a Glance</h3>
<ul style="list-style: none; padding: 0;">
<li>Tasks completed: <strong>{{tasks_completed}}</strong></li>
<li>Tasks overdue: <strong>{{tasks_overdue}}</strong></li>
<li>New comments: <strong>{{comment_count}}</strong></li>
<li>Mentions: <strong>{{mention_count}}</strong></li>
</ul>
</div>

{{#if tasks_due}}
<h2 style="color: #2563eb;">Upcoming Tasks</h2>
<ul>
{{#each tasks_due}}
<li><a href="{{../app_url}}/tasks/{{this.id}}/">{{this.title}}</a> - Due: {{this.due_at}}</li>
{{/each}}
</ul>
{{/if}}

{{#if mentions}}
<h2 style="color: #2563eb;">You Were Mentioned</h2>
<ul>
{{#each mentions}}
<li><strong>{{this.mentioned_by}}</strong> mentioned you in {{this.entity_type}}: <a href="{{../app_url}}/{{this.entity_type}}/{{this.entity_id}}/">{{this.entity_title}}</a></li>
{{/each}}
</ul>
{{/if}}

<p style="margin-top: 30px; color: #666;">
<a href="{{app_url}}/settings/notifications/">Manage notification preferences</a>
</p>
</body>
</html>',
 'Your Weekly Digest

Hi {{user_name}},

Here''s a summary of activity from {{period_start}} to {{period_end}}:

THIS WEEK AT A GLANCE:
- Tasks completed: {{tasks_completed}}
- Tasks overdue: {{tasks_overdue}}
- New comments: {{comment_count}}
- Mentions: {{mention_count}}

{{#if tasks_due}}
UPCOMING TASKS:
{{#each tasks_due}}
- {{this.title}} - Due: {{this.due_at}}
{{/each}}
{{/if}}

{{#if mentions}}
YOU WERE MENTIONED:
{{#each mentions}}
- {{this.mentioned_by}} mentioned you in {{this.entity_type}}: {{this.entity_title}}
{{/each}}
{{/if}}

Manage notification preferences: {{app_url}}/settings/notifications/',
 TRUE)
ON CONFLICT DO NOTHING;

-- =====================================================
-- CLEANUP JOB SUPPORT
-- =====================================================

-- Function to clean up old collaboration events (keep 24 hours)
CREATE OR REPLACE FUNCTION cleanup_collaboration_events() RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM collaboration_events WHERE created_at < NOW() - INTERVAL '24 hours';
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up stale websocket sessions (no heartbeat in 5 minutes)
CREATE OR REPLACE FUNCTION cleanup_stale_websocket_sessions() RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    UPDATE websocket_sessions
    SET status = 'disconnected'
    WHERE status = 'connected'
      AND last_heartbeat_at < NOW() - INTERVAL '5 minutes';
    GET DIAGNOSTICS deleted_count = ROW_COUNT;

    -- Delete very old disconnected sessions (older than 24 hours)
    DELETE FROM websocket_sessions
    WHERE status = 'disconnected'
      AND last_heartbeat_at < NOW() - INTERVAL '24 hours';

    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to clean up old presence records
CREATE OR REPLACE FUNCTION cleanup_stale_presence() RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM collaboration_presence
    WHERE last_activity_at < NOW() - INTERVAL '30 minutes';
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;
