-- Enterprise Features Migration
-- Implements: RBAC, SSO/SAML, SCIM, Audit Exports, White-labeling

-- ============================================================================
-- 1. CUSTOM ROLES & PERMISSIONS (RBAC)
-- ============================================================================

-- Permission definitions
CREATE TABLE permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    code VARCHAR(100) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    resource VARCHAR(50) NOT NULL,
    action VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert system permissions
INSERT INTO permissions (code, name, description, resource, action) VALUES
-- Controls
('controls:create', 'Create Controls', 'Create new controls', 'controls', 'create'),
('controls:read', 'View Controls', 'View controls and their details', 'controls', 'read'),
('controls:update', 'Update Controls', 'Modify existing controls', 'controls', 'update'),
('controls:delete', 'Delete Controls', 'Delete controls', 'controls', 'delete'),
('controls:manage', 'Manage Controls', 'Full control management including tests and mappings', 'controls', 'manage'),
-- Evidence
('evidence:create', 'Upload Evidence', 'Upload new evidence', 'evidence', 'create'),
('evidence:read', 'View Evidence', 'View evidence and download files', 'evidence', 'read'),
('evidence:update', 'Update Evidence', 'Modify evidence metadata', 'evidence', 'update'),
('evidence:delete', 'Delete Evidence', 'Delete evidence', 'evidence', 'delete'),
('evidence:export', 'Export Evidence', 'Export evidence packages', 'evidence', 'export'),
-- Policies
('policies:create', 'Create Policies', 'Create new policies', 'policies', 'create'),
('policies:read', 'View Policies', 'View policies', 'policies', 'read'),
('policies:update', 'Update Policies', 'Modify policies', 'policies', 'update'),
('policies:delete', 'Delete Policies', 'Delete policies', 'policies', 'delete'),
('policies:approve', 'Approve Policies', 'Approve policy changes', 'policies', 'approve'),
('policies:publish', 'Publish Policies', 'Publish policies for acknowledgment', 'policies', 'publish'),
-- Risks
('risks:create', 'Create Risks', 'Create new risks', 'risks', 'create'),
('risks:read', 'View Risks', 'View risk register', 'risks', 'read'),
('risks:update', 'Update Risks', 'Modify risks', 'risks', 'update'),
('risks:delete', 'Delete Risks', 'Delete risks', 'risks', 'delete'),
('risks:assess', 'Assess Risks', 'Perform risk assessments', 'risks', 'assess'),
-- Vendors
('vendors:create', 'Create Vendors', 'Add new vendors', 'vendors', 'create'),
('vendors:read', 'View Vendors', 'View vendor information', 'vendors', 'read'),
('vendors:update', 'Update Vendors', 'Modify vendor information', 'vendors', 'update'),
('vendors:delete', 'Delete Vendors', 'Delete vendors', 'vendors', 'delete'),
('vendors:assess', 'Assess Vendors', 'Perform vendor assessments', 'vendors', 'assess'),
-- Assets
('assets:create', 'Create Assets', 'Add new assets', 'assets', 'create'),
('assets:read', 'View Assets', 'View asset inventory', 'assets', 'read'),
('assets:update', 'Update Assets', 'Modify asset information', 'assets', 'update'),
('assets:delete', 'Delete Assets', 'Delete assets', 'assets', 'delete'),
-- Audits
('audits:create', 'Create Audits', 'Create audit projects', 'audits', 'create'),
('audits:read', 'View Audits', 'View audit information', 'audits', 'read'),
('audits:update', 'Update Audits', 'Modify audit details', 'audits', 'update'),
('audits:delete', 'Delete Audits', 'Delete audits', 'audits', 'delete'),
('audits:manage', 'Manage Audits', 'Full audit management including requests and findings', 'audits', 'manage'),
-- Integrations
('integrations:create', 'Create Integrations', 'Connect new integrations', 'integrations', 'create'),
('integrations:read', 'View Integrations', 'View integration status', 'integrations', 'read'),
('integrations:update', 'Update Integrations', 'Modify integration settings', 'integrations', 'update'),
('integrations:delete', 'Delete Integrations', 'Remove integrations', 'integrations', 'delete'),
('integrations:sync', 'Sync Integrations', 'Trigger integration syncs', 'integrations', 'sync'),
-- Users & Roles
('users:read', 'View Users', 'View organization users', 'users', 'read'),
('users:invite', 'Invite Users', 'Invite new users to organization', 'users', 'invite'),
('users:update', 'Update Users', 'Modify user information', 'users', 'update'),
('users:delete', 'Remove Users', 'Remove users from organization', 'users', 'delete'),
('roles:read', 'View Roles', 'View custom roles', 'roles', 'read'),
('roles:manage', 'Manage Roles', 'Create, update, delete custom roles', 'roles', 'manage'),
-- Settings
('settings:read', 'View Settings', 'View organization settings', 'settings', 'read'),
('settings:update', 'Update Settings', 'Modify organization settings', 'settings', 'update'),
('settings:billing', 'Manage Billing', 'Manage subscription and billing', 'settings', 'billing'),
('settings:sso', 'Configure SSO', 'Configure SAML/SSO settings', 'settings', 'sso'),
('settings:scim', 'Configure SCIM', 'Configure SCIM provisioning', 'settings', 'scim'),
('settings:branding', 'Customize Branding', 'Customize organization branding', 'settings', 'branding'),
-- Reports & Analytics
('reports:read', 'View Reports', 'View compliance reports', 'reports', 'read'),
('reports:export', 'Export Reports', 'Export reports to PDF/CSV', 'reports', 'export'),
('reports:create', 'Create Reports', 'Create custom reports', 'reports', 'create'),
('analytics:read', 'View Analytics', 'View compliance analytics', 'analytics', 'read'),
-- Audit Logs
('audit_logs:read', 'View Audit Logs', 'View activity logs', 'audit_logs', 'read'),
('audit_logs:export', 'Export Audit Logs', 'Export audit logs for SIEM', 'audit_logs', 'export'),
-- Tasks
('tasks:create', 'Create Tasks', 'Create new tasks', 'tasks', 'create'),
('tasks:read', 'View Tasks', 'View tasks', 'tasks', 'read'),
('tasks:update', 'Update Tasks', 'Modify tasks', 'tasks', 'update'),
('tasks:delete', 'Delete Tasks', 'Delete tasks', 'tasks', 'delete'),
('tasks:assign', 'Assign Tasks', 'Assign tasks to users', 'tasks', 'assign'),
-- Frameworks
('frameworks:read', 'View Frameworks', 'View compliance frameworks', 'frameworks', 'read'),
('frameworks:manage', 'Manage Frameworks', 'Create and manage custom frameworks', 'frameworks', 'manage'),
-- Access Reviews
('access_reviews:create', 'Create Access Reviews', 'Create access review campaigns', 'access_reviews', 'create'),
('access_reviews:read', 'View Access Reviews', 'View access reviews', 'access_reviews', 'read'),
('access_reviews:review', 'Perform Access Reviews', 'Review and approve/revoke access', 'access_reviews', 'review'),
('access_reviews:manage', 'Manage Access Reviews', 'Full access review management', 'access_reviews', 'manage'),
-- AI Features
('ai:use', 'Use AI Features', 'Use AI-powered features', 'ai', 'use'),
('ai:configure', 'Configure AI', 'Configure AI provider settings', 'ai', 'configure');

-- Custom roles per organization
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, name)
);

-- Role to permission mapping
CREATE TABLE role_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id UUID NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(role_id, permission_id)
);

-- User to role mapping (many-to-many, replacing single role column)
CREATE TABLE user_roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_by UUID REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, role_id)
);

-- System role templates (seeded for new orgs)
CREATE TABLE system_role_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    permission_codes TEXT[] NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert system role templates
INSERT INTO system_role_templates (name, description, is_default, permission_codes) VALUES
('Owner', 'Organization owner with full access to all features including billing and organization deletion', FALSE, ARRAY[
    'controls:create', 'controls:read', 'controls:update', 'controls:delete', 'controls:manage',
    'evidence:create', 'evidence:read', 'evidence:update', 'evidence:delete', 'evidence:export',
    'policies:create', 'policies:read', 'policies:update', 'policies:delete', 'policies:approve', 'policies:publish',
    'risks:create', 'risks:read', 'risks:update', 'risks:delete', 'risks:assess',
    'vendors:create', 'vendors:read', 'vendors:update', 'vendors:delete', 'vendors:assess',
    'assets:create', 'assets:read', 'assets:update', 'assets:delete',
    'audits:create', 'audits:read', 'audits:update', 'audits:delete', 'audits:manage',
    'integrations:create', 'integrations:read', 'integrations:update', 'integrations:delete', 'integrations:sync',
    'users:read', 'users:invite', 'users:update', 'users:delete', 'roles:read', 'roles:manage',
    'settings:read', 'settings:update', 'settings:billing', 'settings:sso', 'settings:scim', 'settings:branding',
    'reports:read', 'reports:export', 'reports:create', 'analytics:read',
    'audit_logs:read', 'audit_logs:export',
    'tasks:create', 'tasks:read', 'tasks:update', 'tasks:delete', 'tasks:assign',
    'frameworks:read', 'frameworks:manage',
    'access_reviews:create', 'access_reviews:read', 'access_reviews:review', 'access_reviews:manage',
    'ai:use', 'ai:configure'
]),
('Admin', 'Administrator with access to all features except billing', FALSE, ARRAY[
    'controls:create', 'controls:read', 'controls:update', 'controls:delete', 'controls:manage',
    'evidence:create', 'evidence:read', 'evidence:update', 'evidence:delete', 'evidence:export',
    'policies:create', 'policies:read', 'policies:update', 'policies:delete', 'policies:approve', 'policies:publish',
    'risks:create', 'risks:read', 'risks:update', 'risks:delete', 'risks:assess',
    'vendors:create', 'vendors:read', 'vendors:update', 'vendors:delete', 'vendors:assess',
    'assets:create', 'assets:read', 'assets:update', 'assets:delete',
    'audits:create', 'audits:read', 'audits:update', 'audits:delete', 'audits:manage',
    'integrations:create', 'integrations:read', 'integrations:update', 'integrations:delete', 'integrations:sync',
    'users:read', 'users:invite', 'users:update', 'users:delete', 'roles:read', 'roles:manage',
    'settings:read', 'settings:update', 'settings:sso', 'settings:scim', 'settings:branding',
    'reports:read', 'reports:export', 'reports:create', 'analytics:read',
    'audit_logs:read', 'audit_logs:export',
    'tasks:create', 'tasks:read', 'tasks:update', 'tasks:delete', 'tasks:assign',
    'frameworks:read', 'frameworks:manage',
    'access_reviews:create', 'access_reviews:read', 'access_reviews:review', 'access_reviews:manage',
    'ai:use', 'ai:configure'
]),
('Compliance Manager', 'Full access to GRC features, limited admin access', FALSE, ARRAY[
    'controls:create', 'controls:read', 'controls:update', 'controls:delete', 'controls:manage',
    'evidence:create', 'evidence:read', 'evidence:update', 'evidence:delete', 'evidence:export',
    'policies:create', 'policies:read', 'policies:update', 'policies:delete', 'policies:approve', 'policies:publish',
    'risks:create', 'risks:read', 'risks:update', 'risks:delete', 'risks:assess',
    'vendors:create', 'vendors:read', 'vendors:update', 'vendors:delete', 'vendors:assess',
    'assets:create', 'assets:read', 'assets:update', 'assets:delete',
    'audits:create', 'audits:read', 'audits:update', 'audits:delete', 'audits:manage',
    'integrations:read', 'integrations:sync',
    'users:read',
    'settings:read',
    'reports:read', 'reports:export', 'reports:create', 'analytics:read',
    'audit_logs:read',
    'tasks:create', 'tasks:read', 'tasks:update', 'tasks:delete', 'tasks:assign',
    'frameworks:read', 'frameworks:manage',
    'access_reviews:create', 'access_reviews:read', 'access_reviews:review', 'access_reviews:manage',
    'ai:use'
]),
('Contributor', 'Can create and edit assigned items, view most content', TRUE, ARRAY[
    'controls:create', 'controls:read', 'controls:update',
    'evidence:create', 'evidence:read', 'evidence:update',
    'policies:read',
    'risks:create', 'risks:read', 'risks:update', 'risks:assess',
    'vendors:read', 'vendors:assess',
    'assets:read', 'assets:update',
    'audits:read',
    'integrations:read',
    'users:read',
    'reports:read',
    'analytics:read',
    'tasks:create', 'tasks:read', 'tasks:update',
    'frameworks:read',
    'access_reviews:read', 'access_reviews:review',
    'ai:use'
]),
('Viewer', 'Read-only access to compliance data', FALSE, ARRAY[
    'controls:read',
    'evidence:read',
    'policies:read',
    'risks:read',
    'vendors:read',
    'assets:read',
    'audits:read',
    'integrations:read',
    'users:read',
    'reports:read',
    'analytics:read',
    'tasks:read',
    'frameworks:read',
    'access_reviews:read'
]),
('Auditor', 'Read-only access with full audit workspace access', FALSE, ARRAY[
    'controls:read',
    'evidence:read', 'evidence:export',
    'policies:read',
    'risks:read',
    'vendors:read',
    'assets:read',
    'audits:read', 'audits:manage',
    'integrations:read',
    'users:read',
    'reports:read', 'reports:export',
    'analytics:read',
    'audit_logs:read',
    'tasks:read',
    'frameworks:read',
    'access_reviews:read'
]);

-- Indexes for RBAC
CREATE INDEX idx_roles_organization ON roles(organization_id);
CREATE INDEX idx_role_permissions_role ON role_permissions(role_id);
CREATE INDEX idx_user_roles_user ON user_roles(user_id);
CREATE INDEX idx_user_roles_role ON user_roles(role_id);

-- Trigger for roles updated_at
CREATE TRIGGER update_roles_updated_at BEFORE UPDATE ON roles FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 2. SSO/SAML CONFIGURATION
-- ============================================================================

CREATE TABLE sso_configurations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    provider_type VARCHAR(50) NOT NULL, -- 'saml', 'oidc'
    name VARCHAR(255) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    is_enforced BOOLEAN NOT NULL DEFAULT FALSE, -- Require SSO for all users

    -- SAML specific fields
    idp_entity_id VARCHAR(500),
    idp_sso_url VARCHAR(500),
    idp_slo_url VARCHAR(500),
    idp_certificate TEXT, -- X.509 certificate
    sp_entity_id VARCHAR(500),
    sp_acs_url VARCHAR(500),
    sp_slo_url VARCHAR(500),

    -- Attribute mappings (JSON)
    attribute_mappings JSONB DEFAULT '{"email": "email", "name": "name", "groups": "groups"}',

    -- OIDC specific fields (for future expansion)
    oidc_client_id VARCHAR(255),
    oidc_client_secret_encrypted TEXT,
    oidc_issuer_url VARCHAR(500),
    oidc_scopes TEXT[] DEFAULT ARRAY['openid', 'profile', 'email'],

    -- Security settings
    require_signed_assertions BOOLEAN DEFAULT TRUE,
    require_encrypted_assertions BOOLEAN DEFAULT FALSE,
    allowed_clock_skew_seconds INTEGER DEFAULT 180,

    -- Auto-provisioning settings
    auto_provision_users BOOLEAN DEFAULT TRUE,
    default_role_id UUID REFERENCES roles(id),

    -- Metadata
    metadata_url VARCHAR(500),
    metadata_xml TEXT,
    last_metadata_refresh TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, provider_type)
);

-- SAML session tracking
CREATE TABLE sso_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    session_index VARCHAR(255), -- SAML session index for SLO
    name_id VARCHAR(500), -- SAML NameID
    name_id_format VARCHAR(255),
    idp_session_id VARCHAR(255),
    attributes JSONB,
    valid_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- SSO domain verification (for enforcing SSO per domain)
CREATE TABLE sso_domains (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    sso_configuration_id UUID NOT NULL REFERENCES sso_configurations(id) ON DELETE CASCADE,
    domain VARCHAR(255) NOT NULL,
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    verification_token VARCHAR(255),
    verification_method VARCHAR(50) DEFAULT 'dns_txt', -- 'dns_txt', 'dns_cname', 'meta_tag'
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(domain)
);

-- Indexes for SSO
CREATE INDEX idx_sso_configurations_org ON sso_configurations(organization_id);
CREATE INDEX idx_sso_sessions_user ON sso_sessions(user_id);
CREATE INDEX idx_sso_sessions_org ON sso_sessions(organization_id, valid_until);
CREATE INDEX idx_sso_domains_domain ON sso_domains(domain);

-- Trigger for sso_configurations updated_at
CREATE TRIGGER update_sso_configurations_updated_at BEFORE UPDATE ON sso_configurations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 3. SCIM USER PROVISIONING
-- ============================================================================

CREATE TABLE scim_configurations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,

    -- SCIM endpoint settings
    base_url VARCHAR(500), -- Auto-generated

    -- Token authentication
    bearer_token_hash VARCHAR(255), -- Hashed bearer token
    token_created_at TIMESTAMPTZ,
    token_expires_at TIMESTAMPTZ,

    -- Provisioning settings
    auto_activate_users BOOLEAN DEFAULT TRUE,
    default_role_id UUID REFERENCES roles(id),
    sync_groups_as_roles BOOLEAN DEFAULT FALSE,

    -- Group to role mapping
    group_role_mappings JSONB DEFAULT '{}', -- {"Admins": "role_id", "Users": "role_id"}

    -- Deprovisioning behavior
    on_user_deactivate VARCHAR(50) DEFAULT 'suspend', -- 'suspend', 'delete'

    -- Stats
    last_sync_at TIMESTAMPTZ,
    total_users_provisioned INTEGER DEFAULT 0,
    total_groups_provisioned INTEGER DEFAULT 0,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id)
);

-- SCIM provisioned users (track external IDs)
CREATE TABLE scim_users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    scim_configuration_id UUID NOT NULL REFERENCES scim_configurations(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    external_id VARCHAR(255) NOT NULL, -- SCIM externalId
    scim_id VARCHAR(255) NOT NULL, -- Our SCIM ID (exposed to IdP)
    user_name VARCHAR(255) NOT NULL, -- SCIM userName
    active BOOLEAN NOT NULL DEFAULT TRUE,
    display_name VARCHAR(255),
    emails JSONB, -- Array of email objects
    name JSONB, -- {givenName, familyName, formatted}
    groups JSONB, -- Array of group references
    metadata JSONB, -- Additional SCIM attributes
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(scim_configuration_id, external_id),
    UNIQUE(scim_configuration_id, scim_id)
);

-- SCIM groups
CREATE TABLE scim_groups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    scim_configuration_id UUID NOT NULL REFERENCES scim_configurations(id) ON DELETE CASCADE,
    role_id UUID REFERENCES roles(id) ON DELETE SET NULL,
    external_id VARCHAR(255),
    scim_id VARCHAR(255) NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    members JSONB, -- Array of member references
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(scim_configuration_id, scim_id)
);

-- SCIM operation logs (for debugging)
CREATE TABLE scim_operation_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    scim_configuration_id UUID NOT NULL REFERENCES scim_configurations(id) ON DELETE CASCADE,
    operation VARCHAR(50) NOT NULL, -- 'create', 'read', 'update', 'delete', 'bulk'
    resource_type VARCHAR(50) NOT NULL, -- 'User', 'Group'
    resource_id VARCHAR(255),
    request_body JSONB,
    response_status INTEGER,
    response_body JSONB,
    error_message TEXT,
    ip_address VARCHAR(45),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for SCIM
CREATE INDEX idx_scim_configurations_org ON scim_configurations(organization_id);
CREATE INDEX idx_scim_users_config ON scim_users(scim_configuration_id);
CREATE INDEX idx_scim_users_user ON scim_users(user_id);
CREATE INDEX idx_scim_users_external ON scim_users(scim_configuration_id, external_id);
CREATE INDEX idx_scim_groups_config ON scim_groups(scim_configuration_id);
CREATE INDEX idx_scim_operation_logs_config ON scim_operation_logs(scim_configuration_id, created_at DESC);

-- Trigger for scim tables updated_at
CREATE TRIGGER update_scim_configurations_updated_at BEFORE UPDATE ON scim_configurations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_scim_users_updated_at BEFORE UPDATE ON scim_users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_scim_groups_updated_at BEFORE UPDATE ON scim_groups FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 4. AUDIT LOG EXPORTS (SIEM INTEGRATION)
-- ============================================================================

-- Audit export configurations (webhooks to SIEM)
CREATE TABLE audit_export_configurations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,

    -- Export destination
    export_type VARCHAR(50) NOT NULL, -- 'webhook', 's3', 'splunk', 'elastic'

    -- Webhook settings
    webhook_url VARCHAR(500),
    webhook_secret_encrypted TEXT, -- For signature verification
    webhook_headers JSONB DEFAULT '{}',

    -- S3 settings (for bulk exports)
    s3_bucket VARCHAR(255),
    s3_prefix VARCHAR(255),
    s3_region VARCHAR(50),
    s3_access_key_encrypted TEXT,
    s3_secret_key_encrypted TEXT,

    -- Format settings
    format VARCHAR(50) DEFAULT 'json', -- 'json', 'cef', 'leef', 'syslog'
    include_pii BOOLEAN DEFAULT FALSE,

    -- Filter settings (what to export)
    event_types TEXT[] DEFAULT ARRAY['*'], -- ['*'] for all, or specific types
    min_severity VARCHAR(50) DEFAULT 'info', -- 'debug', 'info', 'warning', 'error', 'critical'

    -- Delivery settings
    batch_size INTEGER DEFAULT 100,
    flush_interval_seconds INTEGER DEFAULT 60,
    retry_count INTEGER DEFAULT 3,

    -- Stats
    last_export_at TIMESTAMPTZ,
    total_events_exported BIGINT DEFAULT 0,
    total_failures BIGINT DEFAULT 0,
    last_error TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Audit export queue (pending events to export)
CREATE TABLE audit_export_queue (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    export_configuration_id UUID NOT NULL REFERENCES audit_export_configurations(id) ON DELETE CASCADE,
    activity_log_id UUID NOT NULL REFERENCES activity_logs(id) ON DELETE CASCADE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- 'pending', 'processing', 'sent', 'failed'
    attempts INTEGER DEFAULT 0,
    last_attempt_at TIMESTAMPTZ,
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add more fields to activity_logs for better SIEM integration
ALTER TABLE activity_logs ADD COLUMN IF NOT EXISTS severity VARCHAR(50) DEFAULT 'info';
ALTER TABLE activity_logs ADD COLUMN IF NOT EXISTS category VARCHAR(100);
ALTER TABLE activity_logs ADD COLUMN IF NOT EXISTS outcome VARCHAR(50) DEFAULT 'success'; -- 'success', 'failure'
ALTER TABLE activity_logs ADD COLUMN IF NOT EXISTS duration_ms INTEGER;
ALTER TABLE activity_logs ADD COLUMN IF NOT EXISTS request_id VARCHAR(100);
ALTER TABLE activity_logs ADD COLUMN IF NOT EXISTS session_id VARCHAR(255);
ALTER TABLE activity_logs ADD COLUMN IF NOT EXISTS resource_name VARCHAR(255);

-- Indexes for audit exports
CREATE INDEX idx_audit_export_configurations_org ON audit_export_configurations(organization_id);
CREATE INDEX idx_audit_export_queue_config ON audit_export_queue(export_configuration_id, status);
CREATE INDEX idx_audit_export_queue_status ON audit_export_queue(status, created_at);
CREATE INDEX idx_activity_logs_severity ON activity_logs(organization_id, severity, created_at);
CREATE INDEX idx_activity_logs_category ON activity_logs(organization_id, category, created_at);

-- Trigger for audit_export_configurations updated_at
CREATE TRIGGER update_audit_export_configurations_updated_at BEFORE UPDATE ON audit_export_configurations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 5. WHITE-LABELING / BRANDING
-- ============================================================================

CREATE TABLE branding_configurations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Logo and favicon
    logo_url VARCHAR(500),
    logo_dark_url VARCHAR(500), -- For dark mode
    favicon_url VARCHAR(500),

    -- Colors
    primary_color VARCHAR(7) DEFAULT '#3B82F6', -- Hex color
    secondary_color VARCHAR(7) DEFAULT '#10B981',
    accent_color VARCHAR(7) DEFAULT '#8B5CF6',

    -- Theme settings
    dark_mode_enabled BOOLEAN DEFAULT TRUE,
    default_theme VARCHAR(50) DEFAULT 'system', -- 'light', 'dark', 'system'

    -- Typography
    font_family VARCHAR(100) DEFAULT 'Inter',
    heading_font_family VARCHAR(100),

    -- Custom CSS (advanced)
    custom_css TEXT,

    -- Login page customization
    login_background_url VARCHAR(500),
    login_message TEXT,
    show_powered_by BOOLEAN DEFAULT TRUE,

    -- Email templates
    email_header_html TEXT,
    email_footer_html TEXT,
    email_logo_url VARCHAR(500),

    -- PDF report branding
    pdf_header_html TEXT,
    pdf_footer_html TEXT,
    pdf_cover_image_url VARCHAR(500),

    -- Custom domain
    custom_domain VARCHAR(255),
    custom_domain_verified BOOLEAN DEFAULT FALSE,
    custom_domain_ssl_status VARCHAR(50), -- 'pending', 'active', 'failed'

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id)
);

-- Custom domain verification
CREATE TABLE domain_verifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    domain VARCHAR(255) NOT NULL,
    verification_type VARCHAR(50) NOT NULL, -- 'dns_txt', 'dns_cname', 'http'
    verification_token VARCHAR(255) NOT NULL,
    verification_value VARCHAR(500), -- Expected DNS value or file content
    is_verified BOOLEAN NOT NULL DEFAULT FALSE,
    verified_at TIMESTAMPTZ,
    last_check_at TIMESTAMPTZ,
    check_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, domain)
);

-- Indexes for branding
CREATE INDEX idx_branding_configurations_org ON branding_configurations(organization_id);
CREATE INDEX idx_domain_verifications_org ON domain_verifications(organization_id);
CREATE INDEX idx_domain_verifications_domain ON domain_verifications(domain);

-- Trigger for branding_configurations updated_at
CREATE TRIGGER update_branding_configurations_updated_at BEFORE UPDATE ON branding_configurations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- 6. API KEYS FOR PROGRAMMATIC ACCESS
-- ============================================================================

CREATE TABLE api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL, -- Creator
    name VARCHAR(255) NOT NULL,
    description TEXT,

    -- Key details
    key_prefix VARCHAR(10) NOT NULL, -- First 10 chars for identification (e.g., "ogrc_live_")
    key_hash VARCHAR(255) NOT NULL, -- SHA256 hash of the full key

    -- Permissions
    scopes TEXT[] NOT NULL DEFAULT ARRAY['read'], -- ['read', 'write', 'admin']
    role_id UUID REFERENCES roles(id), -- Optional: inherit permissions from role

    -- Usage limits (enforced in Redis)
    rate_limit_per_minute INTEGER DEFAULT 60,
    rate_limit_per_hour INTEGER DEFAULT 1000,

    -- Validity
    expires_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,

    -- Status
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    revoked_at TIMESTAMPTZ,
    revoked_by UUID REFERENCES users(id),
    revoke_reason TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- API key usage logs (summary, details in Redis)
CREATE TABLE api_key_usage_daily (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    usage_date DATE NOT NULL,
    request_count BIGINT DEFAULT 0,
    error_count BIGINT DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(api_key_id, usage_date)
);

-- Indexes for API keys
CREATE INDEX idx_api_keys_org ON api_keys(organization_id);
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix);
CREATE INDEX idx_api_keys_active ON api_keys(organization_id, is_active) WHERE is_active = TRUE;
CREATE INDEX idx_api_key_usage_daily_key ON api_key_usage_daily(api_key_id, usage_date);

-- ============================================================================
-- 7. SUBSCRIPTION & USAGE TRACKING (for rate limiting tiers)
-- ============================================================================

-- Extend organizations table for subscription details
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS subscription_plan VARCHAR(50) DEFAULT 'free';
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS subscription_status VARCHAR(50) DEFAULT 'active';
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS subscription_started_at TIMESTAMPTZ;
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS subscription_ends_at TIMESTAMPTZ;
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS max_users INTEGER DEFAULT 5;
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS max_integrations INTEGER DEFAULT 2;
ALTER TABLE organizations ADD COLUMN IF NOT EXISTS features_enabled TEXT[] DEFAULT ARRAY['basic'];

-- Usage tracking (daily aggregates, real-time in Redis)
CREATE TABLE organization_usage_daily (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    usage_date DATE NOT NULL,
    api_requests BIGINT DEFAULT 0,
    storage_bytes BIGINT DEFAULT 0,
    users_count INTEGER DEFAULT 0,
    integrations_count INTEGER DEFAULT 0,
    evidence_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, usage_date)
);

-- Indexes for usage
CREATE INDEX idx_organization_usage_daily_org ON organization_usage_daily(organization_id, usage_date DESC);

-- ============================================================================
-- HELPER FUNCTION: Initialize roles for a new organization
-- ============================================================================

CREATE OR REPLACE FUNCTION initialize_organization_roles(org_id UUID)
RETURNS void AS $$
DECLARE
    template RECORD;
    new_role_id UUID;
    perm_id UUID;
    perm_code TEXT;
BEGIN
    -- Create roles from templates
    FOR template IN SELECT * FROM system_role_templates LOOP
        INSERT INTO roles (organization_id, name, description, is_system, is_default)
        VALUES (org_id, template.name, template.description, TRUE, template.is_default)
        RETURNING id INTO new_role_id;

        -- Assign permissions to the role
        FOREACH perm_code IN ARRAY template.permission_codes LOOP
            SELECT id INTO perm_id FROM permissions WHERE code = perm_code;
            IF perm_id IS NOT NULL THEN
                INSERT INTO role_permissions (role_id, permission_id)
                VALUES (new_role_id, perm_id);
            END IF;
        END LOOP;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- HELPER FUNCTION: Get user permissions
-- ============================================================================

CREATE OR REPLACE FUNCTION get_user_permissions(p_user_id UUID)
RETURNS TABLE(permission_code VARCHAR) AS $$
BEGIN
    RETURN QUERY
    SELECT DISTINCT p.code
    FROM user_roles ur
    JOIN role_permissions rp ON rp.role_id = ur.role_id
    JOIN permissions p ON p.id = rp.permission_id
    WHERE ur.user_id = p_user_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- HELPER FUNCTION: Check if user has permission
-- ============================================================================

CREATE OR REPLACE FUNCTION user_has_permission(p_user_id UUID, p_permission_code VARCHAR)
RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1
        FROM user_roles ur
        JOIN role_permissions rp ON rp.role_id = ur.role_id
        JOIN permissions p ON p.id = rp.permission_id
        WHERE ur.user_id = p_user_id AND p.code = p_permission_code
    );
END;
$$ LANGUAGE plpgsql;
