-- Policy Reminders and Notifications for OpenGRC

-- Notifications table for in-app notifications
CREATE TABLE IF NOT EXISTS notifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    notification_type VARCHAR(50) NOT NULL, -- 'policy_acknowledgment', 'policy_reminder', 'policy_update'
    title VARCHAR(255) NOT NULL,
    message TEXT NOT NULL,
    data JSONB DEFAULT '{}', -- Additional context (policy_id, etc)
    read_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for efficient user notification queries
CREATE INDEX IF NOT EXISTS idx_notifications_user ON notifications(user_id, organization_id, read_at);
CREATE INDEX IF NOT EXISTS idx_notifications_created ON notifications(created_at DESC);

-- Policy reminder schedules
CREATE TABLE IF NOT EXISTS policy_reminder_schedules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    policy_id UUID NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    reminder_days INTEGER[] NOT NULL DEFAULT '{7, 3, 1}', -- Days before deadline to send reminders
    email_enabled BOOLEAN NOT NULL DEFAULT true,
    in_app_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, policy_id)
);

-- Track sent reminders to avoid duplicates
CREATE TABLE IF NOT EXISTS policy_reminders_sent (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    policy_id UUID NOT NULL REFERENCES policies(id) ON DELETE CASCADE,
    user_id UUID NOT NULL,
    policy_version INTEGER NOT NULL,
    reminder_type VARCHAR(50) NOT NULL, -- 'scheduled', 'manual'
    channel VARCHAR(50) NOT NULL, -- 'email', 'in_app', 'both'
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_date DATE NOT NULL DEFAULT CURRENT_DATE,
    UNIQUE(policy_id, user_id, policy_version, reminder_type, sent_date)
);

CREATE INDEX IF NOT EXISTS idx_reminders_sent_policy ON policy_reminders_sent(policy_id, user_id, policy_version);
CREATE INDEX IF NOT EXISTS idx_reminders_sent_date ON policy_reminders_sent(sent_at DESC);

-- Email templates for policy reminders
CREATE TABLE IF NOT EXISTS email_templates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE, -- NULL = system default
    template_type VARCHAR(50) NOT NULL, -- 'policy_reminder', 'policy_new', 'policy_updated'
    subject VARCHAR(500) NOT NULL,
    body_html TEXT NOT NULL,
    body_text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, template_type)
);

-- Insert default email templates
INSERT INTO email_templates (organization_id, template_type, subject, body_html, body_text)
VALUES
    (NULL, 'policy_reminder', 'Action Required: Please acknowledge {{policy_title}}',
     '<h2>Policy Acknowledgment Required</h2><p>Dear {{user_name}},</p><p>You have a pending policy that requires your acknowledgment:</p><p><strong>{{policy_title}}</strong> ({{policy_code}})</p><p>Please review and acknowledge this policy at your earliest convenience.</p><p><a href="{{acknowledge_url}}">Review and Acknowledge Policy</a></p><p>Thank you,<br/>{{organization_name}}</p>',
     'Policy Acknowledgment Required\n\nDear {{user_name}},\n\nYou have a pending policy that requires your acknowledgment:\n\n{{policy_title}} ({{policy_code}})\n\nPlease review and acknowledge this policy at your earliest convenience.\n\nVisit: {{acknowledge_url}}\n\nThank you,\n{{organization_name}}'),
    (NULL, 'policy_new', 'New Policy: {{policy_title}}',
     '<h2>New Policy Published</h2><p>Dear {{user_name}},</p><p>A new policy has been published that requires your acknowledgment:</p><p><strong>{{policy_title}}</strong> ({{policy_code}})</p><p>Please review and acknowledge this policy.</p><p><a href="{{acknowledge_url}}">Review and Acknowledge Policy</a></p><p>Thank you,<br/>{{organization_name}}</p>',
     'New Policy Published\n\nDear {{user_name}},\n\nA new policy has been published that requires your acknowledgment:\n\n{{policy_title}} ({{policy_code}})\n\nPlease review and acknowledge this policy.\n\nVisit: {{acknowledge_url}}\n\nThank you,\n{{organization_name}}'),
    (NULL, 'policy_updated', 'Updated Policy: {{policy_title}}',
     '<h2>Policy Updated</h2><p>Dear {{user_name}},</p><p>A policy has been updated and requires your re-acknowledgment:</p><p><strong>{{policy_title}}</strong> ({{policy_code}}) - Version {{policy_version}}</p><p>Please review the updated policy and acknowledge it.</p><p><a href="{{acknowledge_url}}">Review and Acknowledge Policy</a></p><p>Thank you,<br/>{{organization_name}}</p>',
     'Policy Updated\n\nDear {{user_name}},\n\nA policy has been updated and requires your re-acknowledgment:\n\n{{policy_title}} ({{policy_code}}) - Version {{policy_version}}\n\nPlease review the updated policy and acknowledge it.\n\nVisit: {{acknowledge_url}}\n\nThank you,\n{{organization_name}}')
ON CONFLICT (organization_id, template_type) DO NOTHING;
