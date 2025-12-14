-- AWS Integration Tables
-- Phase 2.2: AWS Cloud Integration for OpenGRC

-- AWS Resources (generic inventory for all resource types)
CREATE TABLE aws_resources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    aws_account_id VARCHAR(20) NOT NULL,
    region VARCHAR(50) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id VARCHAR(255) NOT NULL,
    resource_arn TEXT,
    name VARCHAR(255),
    status VARCHAR(50),
    tags JSONB DEFAULT '{}',
    configuration JSONB DEFAULT '{}',
    compliance_status VARCHAR(50),
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, resource_type, resource_id)
);

CREATE INDEX idx_aws_resources_org ON aws_resources(organization_id);
CREATE INDEX idx_aws_resources_integration ON aws_resources(integration_id);
CREATE INDEX idx_aws_resources_account_region ON aws_resources(aws_account_id, region);
CREATE INDEX idx_aws_resources_type ON aws_resources(resource_type);
CREATE INDEX idx_aws_resources_compliance ON aws_resources(compliance_status);

-- AWS IAM Users
CREATE TABLE aws_iam_users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    aws_account_id VARCHAR(20) NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    user_name VARCHAR(255) NOT NULL,
    arn TEXT NOT NULL,
    path VARCHAR(512) DEFAULT '/',
    created_date TIMESTAMPTZ,
    password_last_used TIMESTAMPTZ,
    mfa_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    mfa_devices JSONB DEFAULT '[]',
    access_keys JSONB DEFAULT '[]',
    attached_policies JSONB DEFAULT '[]',
    inline_policy_names JSONB DEFAULT '[]',
    groups JSONB DEFAULT '[]',
    tags JSONB DEFAULT '{}',
    risk_score INTEGER DEFAULT 0,
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, user_id)
);

CREATE INDEX idx_aws_iam_users_org ON aws_iam_users(organization_id);
CREATE INDEX idx_aws_iam_users_integration ON aws_iam_users(integration_id);
CREATE INDEX idx_aws_iam_users_mfa ON aws_iam_users(mfa_enabled);
CREATE INDEX idx_aws_iam_users_risk ON aws_iam_users(risk_score);

-- AWS IAM Roles
CREATE TABLE aws_iam_roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    aws_account_id VARCHAR(20) NOT NULL,
    role_id VARCHAR(100) NOT NULL,
    role_name VARCHAR(255) NOT NULL,
    arn TEXT NOT NULL,
    path VARCHAR(512) DEFAULT '/',
    description TEXT,
    assume_role_policy_document JSONB,
    max_session_duration INTEGER DEFAULT 3600,
    created_date TIMESTAMPTZ,
    attached_policies JSONB DEFAULT '[]',
    inline_policy_names JSONB DEFAULT '[]',
    last_used_at TIMESTAMPTZ,
    last_used_region VARCHAR(50),
    tags JSONB DEFAULT '{}',
    is_service_role BOOLEAN DEFAULT FALSE,
    allows_cross_account BOOLEAN DEFAULT FALSE,
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, role_id)
);

CREATE INDEX idx_aws_iam_roles_org ON aws_iam_roles(organization_id);
CREATE INDEX idx_aws_iam_roles_integration ON aws_iam_roles(integration_id);
CREATE INDEX idx_aws_iam_roles_service ON aws_iam_roles(is_service_role);
CREATE INDEX idx_aws_iam_roles_cross_account ON aws_iam_roles(allows_cross_account);

-- AWS IAM Policies
CREATE TABLE aws_iam_policies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    aws_account_id VARCHAR(20) NOT NULL,
    policy_id VARCHAR(100) NOT NULL,
    policy_name VARCHAR(255) NOT NULL,
    arn TEXT NOT NULL,
    path VARCHAR(512) DEFAULT '/',
    description TEXT,
    policy_document JSONB,
    attachment_count INTEGER DEFAULT 0,
    is_attachable BOOLEAN DEFAULT TRUE,
    is_aws_managed BOOLEAN DEFAULT FALSE,
    allows_admin_access BOOLEAN DEFAULT FALSE,
    uses_wildcard_resources BOOLEAN DEFAULT FALSE,
    risk_score INTEGER DEFAULT 0,
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, policy_id)
);

CREATE INDEX idx_aws_iam_policies_org ON aws_iam_policies(organization_id);
CREATE INDEX idx_aws_iam_policies_integration ON aws_iam_policies(integration_id);
CREATE INDEX idx_aws_iam_policies_admin ON aws_iam_policies(allows_admin_access);
CREATE INDEX idx_aws_iam_policies_risk ON aws_iam_policies(risk_score);

-- AWS Security Hub Findings
CREATE TABLE aws_security_findings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    aws_account_id VARCHAR(20) NOT NULL,
    region VARCHAR(50) NOT NULL,
    finding_id VARCHAR(512) NOT NULL,
    product_arn TEXT NOT NULL,
    product_name VARCHAR(255),
    generator_id VARCHAR(512),
    types JSONB DEFAULT '[]',
    title TEXT NOT NULL,
    description TEXT,
    severity_label VARCHAR(50) NOT NULL,
    severity_normalized INTEGER DEFAULT 0,
    workflow_status VARCHAR(50) DEFAULT 'NEW',
    record_state VARCHAR(50) DEFAULT 'ACTIVE',
    compliance_status VARCHAR(50),
    compliance_standards JSONB DEFAULT '[]',
    related_resources JSONB DEFAULT '[]',
    remediation_text TEXT,
    remediation_url TEXT,
    first_observed_at TIMESTAMPTZ,
    last_observed_at TIMESTAMPTZ,
    mapped_control_codes JSONB DEFAULT '[]',
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, finding_id)
);

CREATE INDEX idx_aws_security_findings_org ON aws_security_findings(organization_id);
CREATE INDEX idx_aws_security_findings_integration ON aws_security_findings(integration_id);
CREATE INDEX idx_aws_security_findings_severity ON aws_security_findings(severity_label);
CREATE INDEX idx_aws_security_findings_workflow ON aws_security_findings(workflow_status);
CREATE INDEX idx_aws_security_findings_compliance ON aws_security_findings(compliance_status);
CREATE INDEX idx_aws_security_findings_region ON aws_security_findings(region);
CREATE INDEX idx_aws_security_findings_observed ON aws_security_findings(last_observed_at);

-- AWS Config Rules
CREATE TABLE aws_config_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    aws_account_id VARCHAR(20) NOT NULL,
    region VARCHAR(50) NOT NULL,
    config_rule_name VARCHAR(255) NOT NULL,
    config_rule_arn TEXT NOT NULL,
    config_rule_id VARCHAR(100),
    description TEXT,
    source_owner VARCHAR(50),
    source_identifier VARCHAR(255),
    compliance_type VARCHAR(50) NOT NULL,
    compliant_count INTEGER DEFAULT 0,
    non_compliant_count INTEGER DEFAULT 0,
    mapped_control_codes JSONB DEFAULT '[]',
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, config_rule_name, region)
);

CREATE INDEX idx_aws_config_rules_org ON aws_config_rules(organization_id);
CREATE INDEX idx_aws_config_rules_integration ON aws_config_rules(integration_id);
CREATE INDEX idx_aws_config_rules_compliance ON aws_config_rules(compliance_type);
CREATE INDEX idx_aws_config_rules_region ON aws_config_rules(region);

-- AWS CloudTrail Events
CREATE TABLE aws_cloudtrail_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    aws_account_id VARCHAR(20) NOT NULL,
    region VARCHAR(50) NOT NULL,
    event_id VARCHAR(100) NOT NULL,
    event_name VARCHAR(255) NOT NULL,
    event_source VARCHAR(255) NOT NULL,
    event_time TIMESTAMPTZ NOT NULL,
    event_type VARCHAR(100),
    user_identity JSONB DEFAULT '{}',
    user_name VARCHAR(255),
    source_ip_address VARCHAR(50),
    user_agent TEXT,
    request_parameters JSONB DEFAULT '{}',
    response_elements JSONB DEFAULT '{}',
    error_code VARCHAR(100),
    error_message TEXT,
    is_root_action BOOLEAN DEFAULT FALSE,
    is_sensitive_action BOOLEAN DEFAULT FALSE,
    risk_level VARCHAR(50) DEFAULT 'LOW',
    resources JSONB DEFAULT '[]',
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, event_id)
);

CREATE INDEX idx_aws_cloudtrail_org ON aws_cloudtrail_events(organization_id);
CREATE INDEX idx_aws_cloudtrail_integration ON aws_cloudtrail_events(integration_id);
CREATE INDEX idx_aws_cloudtrail_event_time ON aws_cloudtrail_events(event_time);
CREATE INDEX idx_aws_cloudtrail_event_name ON aws_cloudtrail_events(event_name);
CREATE INDEX idx_aws_cloudtrail_event_source ON aws_cloudtrail_events(event_source);
CREATE INDEX idx_aws_cloudtrail_root ON aws_cloudtrail_events(is_root_action);
CREATE INDEX idx_aws_cloudtrail_sensitive ON aws_cloudtrail_events(is_sensitive_action);
CREATE INDEX idx_aws_cloudtrail_risk ON aws_cloudtrail_events(risk_level);

-- AWS S3 Buckets
CREATE TABLE aws_s3_buckets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    aws_account_id VARCHAR(20) NOT NULL,
    bucket_name VARCHAR(255) NOT NULL,
    region VARCHAR(50),
    creation_date TIMESTAMPTZ,
    encryption_enabled BOOLEAN DEFAULT FALSE,
    encryption_type VARCHAR(50),
    versioning_enabled BOOLEAN DEFAULT FALSE,
    logging_enabled BOOLEAN DEFAULT FALSE,
    is_public BOOLEAN DEFAULT FALSE,
    public_access_block JSONB DEFAULT '{}',
    bucket_policy JSONB,
    tags JSONB DEFAULT '{}',
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, bucket_name)
);

CREATE INDEX idx_aws_s3_buckets_org ON aws_s3_buckets(organization_id);
CREATE INDEX idx_aws_s3_buckets_integration ON aws_s3_buckets(integration_id);
CREATE INDEX idx_aws_s3_buckets_public ON aws_s3_buckets(is_public);
CREATE INDEX idx_aws_s3_buckets_encryption ON aws_s3_buckets(encryption_enabled);

-- AWS EC2 Instances
CREATE TABLE aws_ec2_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    aws_account_id VARCHAR(20) NOT NULL,
    region VARCHAR(50) NOT NULL,
    instance_id VARCHAR(50) NOT NULL,
    instance_type VARCHAR(50),
    state VARCHAR(50),
    private_ip VARCHAR(50),
    public_ip VARCHAR(50),
    vpc_id VARCHAR(50),
    subnet_id VARCHAR(50),
    security_groups JSONB DEFAULT '[]',
    iam_instance_profile VARCHAR(255),
    launch_time TIMESTAMPTZ,
    platform VARCHAR(50),
    architecture VARCHAR(20),
    ebs_optimized BOOLEAN DEFAULT FALSE,
    monitoring_enabled BOOLEAN DEFAULT FALSE,
    is_public BOOLEAN DEFAULT FALSE,
    tags JSONB DEFAULT '{}',
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, instance_id)
);

CREATE INDEX idx_aws_ec2_instances_org ON aws_ec2_instances(organization_id);
CREATE INDEX idx_aws_ec2_instances_integration ON aws_ec2_instances(integration_id);
CREATE INDEX idx_aws_ec2_instances_region ON aws_ec2_instances(region);
CREATE INDEX idx_aws_ec2_instances_state ON aws_ec2_instances(state);
CREATE INDEX idx_aws_ec2_instances_public ON aws_ec2_instances(is_public);

-- AWS EC2 Security Groups
CREATE TABLE aws_security_groups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    aws_account_id VARCHAR(20) NOT NULL,
    region VARCHAR(50) NOT NULL,
    group_id VARCHAR(50) NOT NULL,
    group_name VARCHAR(255) NOT NULL,
    description TEXT,
    vpc_id VARCHAR(50),
    inbound_rules JSONB DEFAULT '[]',
    outbound_rules JSONB DEFAULT '[]',
    has_risky_rules BOOLEAN DEFAULT FALSE,
    risky_rule_details JSONB DEFAULT '[]',
    tags JSONB DEFAULT '{}',
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, group_id)
);

CREATE INDEX idx_aws_security_groups_org ON aws_security_groups(organization_id);
CREATE INDEX idx_aws_security_groups_integration ON aws_security_groups(integration_id);
CREATE INDEX idx_aws_security_groups_risky ON aws_security_groups(has_risky_rules);
CREATE INDEX idx_aws_security_groups_vpc ON aws_security_groups(vpc_id);

-- AWS RDS Instances
CREATE TABLE aws_rds_instances (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    aws_account_id VARCHAR(20) NOT NULL,
    region VARCHAR(50) NOT NULL,
    db_instance_identifier VARCHAR(255) NOT NULL,
    db_instance_arn TEXT,
    db_instance_class VARCHAR(50),
    engine VARCHAR(50),
    engine_version VARCHAR(50),
    status VARCHAR(50),
    endpoint_address VARCHAR(255),
    endpoint_port INTEGER,
    availability_zone VARCHAR(50),
    multi_az BOOLEAN DEFAULT FALSE,
    publicly_accessible BOOLEAN DEFAULT FALSE,
    storage_encrypted BOOLEAN DEFAULT FALSE,
    kms_key_id VARCHAR(255),
    vpc_id VARCHAR(50),
    security_groups JSONB DEFAULT '[]',
    backup_retention_period INTEGER DEFAULT 0,
    deletion_protection BOOLEAN DEFAULT FALSE,
    iam_auth_enabled BOOLEAN DEFAULT FALSE,
    auto_minor_version_upgrade BOOLEAN DEFAULT TRUE,
    allocated_storage INTEGER,
    storage_type VARCHAR(50),
    tags JSONB DEFAULT '{}',
    last_synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, db_instance_identifier, region)
);

CREATE INDEX idx_aws_rds_instances_org ON aws_rds_instances(organization_id);
CREATE INDEX idx_aws_rds_instances_integration ON aws_rds_instances(integration_id);
CREATE INDEX idx_aws_rds_instances_region ON aws_rds_instances(region);
CREATE INDEX idx_aws_rds_instances_public ON aws_rds_instances(publicly_accessible);
CREATE INDEX idx_aws_rds_instances_encrypted ON aws_rds_instances(storage_encrypted);

-- AWS Control Mappings (AWS checks to SOC 2 controls)
CREATE TABLE aws_control_mappings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID REFERENCES organizations(id) ON DELETE CASCADE,
    aws_check_id VARCHAR(255) NOT NULL,
    aws_check_type VARCHAR(100) NOT NULL,
    control_id UUID NOT NULL REFERENCES controls(id) ON DELETE CASCADE,
    is_system_mapping BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(organization_id, aws_check_id, control_id)
);

CREATE INDEX idx_aws_control_mappings_org ON aws_control_mappings(organization_id);
CREATE INDEX idx_aws_control_mappings_check ON aws_control_mappings(aws_check_id);
CREATE INDEX idx_aws_control_mappings_control ON aws_control_mappings(control_id);

-- AWS Sync Status (per-service tracking)
CREATE TABLE aws_sync_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    integration_id UUID NOT NULL REFERENCES integrations(id) ON DELETE CASCADE,
    service_name VARCHAR(50) NOT NULL,
    region VARCHAR(50),
    last_sync_at TIMESTAMPTZ,
    last_sync_status VARCHAR(50) NOT NULL DEFAULT 'pending',
    last_error TEXT,
    records_synced INTEGER DEFAULT 0,
    sync_cursor TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(integration_id, service_name, region)
);

CREATE INDEX idx_aws_sync_status_integration ON aws_sync_status(integration_id);
CREATE INDEX idx_aws_sync_status_service ON aws_sync_status(service_name);
CREATE INDEX idx_aws_sync_status_status ON aws_sync_status(last_sync_status);

-- Trigger functions for updated_at
CREATE OR REPLACE FUNCTION update_aws_resources_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create triggers for all AWS tables
CREATE TRIGGER update_aws_resources_timestamp BEFORE UPDATE ON aws_resources FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
CREATE TRIGGER update_aws_iam_users_timestamp BEFORE UPDATE ON aws_iam_users FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
CREATE TRIGGER update_aws_iam_roles_timestamp BEFORE UPDATE ON aws_iam_roles FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
CREATE TRIGGER update_aws_iam_policies_timestamp BEFORE UPDATE ON aws_iam_policies FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
CREATE TRIGGER update_aws_security_findings_timestamp BEFORE UPDATE ON aws_security_findings FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
CREATE TRIGGER update_aws_config_rules_timestamp BEFORE UPDATE ON aws_config_rules FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
CREATE TRIGGER update_aws_s3_buckets_timestamp BEFORE UPDATE ON aws_s3_buckets FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
CREATE TRIGGER update_aws_ec2_instances_timestamp BEFORE UPDATE ON aws_ec2_instances FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
CREATE TRIGGER update_aws_security_groups_timestamp BEFORE UPDATE ON aws_security_groups FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
CREATE TRIGGER update_aws_rds_instances_timestamp BEFORE UPDATE ON aws_rds_instances FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
CREATE TRIGGER update_aws_control_mappings_timestamp BEFORE UPDATE ON aws_control_mappings FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
CREATE TRIGGER update_aws_sync_status_timestamp BEFORE UPDATE ON aws_sync_status FOR EACH ROW EXECUTE FUNCTION update_aws_resources_timestamp();
