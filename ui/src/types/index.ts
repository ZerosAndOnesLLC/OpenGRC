// OpenGRC TypeScript Types
// These types match the Rust API models

// ==================== Common Types ====================

export interface PaginatedResponse<T> {
  data: T[]
  total: number
  limit: number
  offset: number
}

// ==================== Organization & User ====================

export interface Organization {
  id: string
  name: string
  slug: string
  settings: Record<string, unknown>
  subscription_tier: string
  created_at: string
  updated_at: string
}

export interface User {
  id: string
  organization_id: string
  tv_user_id: string
  email: string
  name: string
  role: string
  last_login_at: string | null
  created_at: string
}

// ==================== Framework ====================

export interface Framework {
  id: string
  name: string
  version: string | null
  description: string | null
  category: string | null
  is_system: boolean
  created_at: string
}

export interface FrameworkRequirement {
  id: string
  framework_id: string
  code: string
  name: string
  description: string | null
  category: string | null
  parent_id: string | null
  sort_order: number
}

export interface FrameworkWithRequirements extends Framework {
  requirements: FrameworkRequirement[]
  requirement_count: number
}

export interface RequirementTree extends FrameworkRequirement {
  children: RequirementTree[]
}

export interface CreateFramework {
  name: string
  version?: string
  description?: string
  category?: string
  is_system?: boolean
}

export interface UpdateFramework {
  name?: string
  version?: string
  description?: string
  category?: string
}

export interface CreateFrameworkRequirement {
  code: string
  name: string
  description?: string
  category?: string
  parent_id?: string
  sort_order?: number
}

export interface RequirementGapAnalysis {
  id: string
  code: string
  name: string
  category: string | null
  control_count: number
  is_covered: boolean
}

export interface CategoryGapAnalysis {
  category: string | null
  total: number
  covered: number
  coverage_percentage: number
}

export interface FrameworkGapAnalysis {
  framework_id: string
  framework_name: string
  total_requirements: number
  covered_requirements: number
  uncovered_requirements: number
  coverage_percentage: number
  by_category: CategoryGapAnalysis[]
  requirements: RequirementGapAnalysis[]
}

// ==================== Control ====================

export type ControlType = 'preventive' | 'detective' | 'corrective'
export type ControlFrequency = 'continuous' | 'daily' | 'weekly' | 'monthly' | 'quarterly' | 'annual'
export type ControlStatus = 'not_implemented' | 'in_progress' | 'implemented' | 'not_applicable'

export interface Control {
  id: string
  organization_id: string
  code: string
  name: string
  description: string | null
  control_type: string
  frequency: string
  owner_id: string | null
  status: string
  implementation_notes: string | null
  created_at: string
  updated_at: string
}

export interface ControlWithMappings extends Control {
  requirement_count: number
  mapped_requirements?: MappedRequirement[]
}

export interface MappedRequirement {
  id: string
  framework_id: string
  framework_name: string
  code: string
  name: string
}

export interface ControlTest {
  id: string
  control_id: string
  name: string
  description: string | null
  test_type: string
  automation_config: Record<string, unknown> | null
  frequency: string | null
  next_due_at: string | null
  created_at: string
}

export interface ControlTestResult {
  id: string
  control_test_id: string
  performed_by: string | null
  performed_at: string
  status: string
  notes: string | null
  evidence_ids: string[] | null
  created_at: string
}

export interface CreateControl {
  code: string
  name: string
  description?: string
  control_type?: ControlType
  frequency?: ControlFrequency
  owner_id?: string
  status?: ControlStatus
  implementation_notes?: string
}

export interface UpdateControl {
  code?: string
  name?: string
  description?: string
  control_type?: string
  frequency?: string
  owner_id?: string
  status?: string
  implementation_notes?: string
}

export interface CreateControlTest {
  name: string
  description?: string
  test_type?: string
  automation_config?: Record<string, unknown>
  frequency?: string
  next_due_at?: string
}

export interface CreateTestResult {
  status: string
  notes?: string
  evidence_ids?: string[]
}

export interface ListControlsQuery {
  status?: string
  control_type?: string
  owner_id?: string
  search?: string
  limit?: number
  offset?: number
}

export interface ControlStats {
  total: number
  implemented: number
  in_progress: number
  not_implemented: number
  not_applicable: number
  implementation_percentage: number
}

// ==================== Evidence ====================

export type EvidenceType = 'document' | 'screenshot' | 'log' | 'automated' | 'config' | 'report'
export type EvidenceSource = 'manual' | 'aws' | 'github' | 'okta' | 'azure' | 'gcp' | 'datadog' | 'other'

export interface Evidence {
  id: string
  organization_id: string
  title: string
  description: string | null
  evidence_type: string
  source: string
  source_reference: string | null
  file_path: string | null
  file_size: number | null
  mime_type: string | null
  collected_at: string
  valid_from: string | null
  valid_until: string | null
  uploaded_by: string | null
  created_at: string
}

export interface EvidenceWithLinks extends Evidence {
  linked_control_count: number
  linked_controls?: LinkedControl[]
}

export interface LinkedControl {
  id: string
  code: string
  name: string
}

export interface CreateEvidence {
  title: string
  description?: string
  evidence_type?: EvidenceType
  source?: EvidenceSource
  source_reference?: string
  file_path?: string
  file_size?: number
  mime_type?: string
  valid_from?: string
  valid_until?: string
}

export interface UpdateEvidence {
  title?: string
  description?: string
  evidence_type?: string
  source?: string
  source_reference?: string
  valid_from?: string
  valid_until?: string
}

export interface ListEvidenceQuery {
  evidence_type?: string
  source?: string
  control_id?: string
  search?: string
  expired?: boolean
  limit?: number
  offset?: number
}

export interface PresignedUploadResponse {
  upload_url: string
  file_key: string
  evidence_id: string
}

export interface PresignedDownloadResponse {
  download_url: string
}

export interface EvidenceStats {
  total: number
  by_type: { evidence_type: string; count: number }[]
  by_source: { source: string; count: number }[]
  expiring_soon: number
  expired: number
}

// ==================== Policy ====================

export type PolicyStatus = 'draft' | 'pending_approval' | 'published' | 'archived'
export type PolicyCategory = 'security' | 'privacy' | 'hr' | 'it' | 'compliance' | 'operations' | 'business' | 'other'

export interface Policy {
  id: string
  organization_id: string
  code: string
  title: string
  category: string | null
  content: string | null
  version: number
  status: string
  owner_id: string | null
  approver_id: string | null
  approved_at: string | null
  effective_date: string | null
  review_date: string | null
  created_at: string
  updated_at: string
}

export interface PolicyWithStats extends Policy {
  acknowledgment_count: number
  pending_acknowledgments: number
}

export interface PolicyVersion {
  id: string
  policy_id: string
  version: number
  content: string | null
  changed_by: string | null
  change_summary: string | null
  created_at: string
}

export interface PolicyAcknowledgment {
  id: string
  policy_id: string
  policy_version: number
  user_id: string
  acknowledged_at: string
  ip_address: string | null
}

export interface CreatePolicy {
  code: string
  title: string
  category?: PolicyCategory
  content?: string
  owner_id?: string
  effective_date?: string
  review_date?: string
}

export interface UpdatePolicy {
  code?: string
  title?: string
  category?: string
  content?: string
  status?: string
  owner_id?: string
  effective_date?: string
  review_date?: string
  change_summary?: string
}

export interface ListPoliciesQuery {
  status?: string
  category?: string
  owner_id?: string
  search?: string
  needs_review?: boolean
  limit?: number
  offset?: number
}

export interface PolicyStats {
  total: number
  published: number
  draft: number
  pending_approval: number
  needs_review: number
  by_category: { category: string | null; count: number }[]
}

// ==================== Risk ====================

export type RiskStatus = 'identified' | 'assessed' | 'treating' | 'monitoring' | 'accepted' | 'closed'
export type RiskCategory = 'strategic' | 'operational' | 'financial' | 'compliance' | 'technology' | 'security' | 'reputational' | 'other'
export type RiskSource = 'internal' | 'external' | 'regulatory' | 'third_party' | 'technology' | 'other'

export interface Risk {
  id: string
  organization_id: string
  code: string
  title: string
  description: string | null
  category: string | null
  source: string | null
  likelihood: number | null
  impact: number | null
  inherent_score: number | null
  residual_likelihood: number | null
  residual_impact: number | null
  residual_score: number | null
  status: string
  owner_id: string | null
  treatment_plan: string | null
  identified_at: string
  review_date: string | null
  created_at: string
  updated_at: string
}

export interface RiskWithControls extends Risk {
  linked_control_count: number
  linked_controls?: LinkedControlSummary[]
}

export interface LinkedControlSummary {
  id: string
  code: string
  name: string
  effectiveness: string | null
}

export interface CreateRisk {
  code: string
  title: string
  description?: string
  category?: RiskCategory
  source?: RiskSource
  likelihood?: number
  impact?: number
  owner_id?: string
  treatment_plan?: string
  review_date?: string
}

export interface UpdateRisk {
  code?: string
  title?: string
  description?: string
  category?: string
  source?: string
  likelihood?: number
  impact?: number
  residual_likelihood?: number
  residual_impact?: number
  status?: string
  owner_id?: string
  treatment_plan?: string
  review_date?: string
}

export interface ListRisksQuery {
  status?: string
  category?: string
  source?: string
  owner_id?: string
  min_score?: number
  max_score?: number
  search?: string
  needs_review?: boolean
  limit?: number
  offset?: number
}

export interface RiskStats {
  total: number
  by_status: { status: string | null; count: number }[]
  by_category: { category: string | null; count: number }[]
  high_risks: number
  medium_risks: number
  low_risks: number
  needs_review: number
  average_inherent_score: number
  average_residual_score: number
}

export interface HeatmapCell {
  likelihood: number
  impact: number
  count: number
}

export interface RiskHeatmapData {
  cells: HeatmapCell[]
  total_risks: number
  risks_with_scores: number
}

export interface LinkControlsRequest {
  control_ids: string[]
  effectiveness?: string
}

// ==================== Vendor ====================

export type VendorCriticality = 'critical' | 'high' | 'medium' | 'low'
export type VendorStatus = 'active' | 'inactive' | 'under_review'

export interface Vendor {
  id: string
  organization_id: string
  name: string
  description: string | null
  category: string | null
  criticality: string | null
  data_classification: string | null
  status: string
  contract_start: string | null
  contract_end: string | null
  owner_id: string | null
  website: string | null
  created_at: string
  updated_at: string
}

export interface VendorWithAssessment extends Vendor {
  last_assessment_date: string | null
  last_risk_rating: string | null
  next_assessment_date: string | null
}

export interface VendorAssessment {
  id: string
  vendor_id: string
  assessment_type: string
  assessed_by: string | null
  assessed_at: string
  risk_rating: string | null
  findings: string | null
  recommendations: string | null
  next_assessment_date: string | null
  created_at: string
}

export interface CreateVendor {
  name: string
  description?: string
  category?: string
  criticality?: VendorCriticality
  data_classification?: string
  status?: VendorStatus
  contract_start?: string
  contract_end?: string
  owner_id?: string
  website?: string
}

export interface UpdateVendor {
  name?: string
  description?: string
  category?: string
  criticality?: string
  data_classification?: string
  status?: string
  contract_start?: string
  contract_end?: string
  owner_id?: string
  website?: string
}

export interface CreateVendorAssessment {
  assessment_type?: string
  risk_rating?: string
  findings?: string
  recommendations?: string
  next_assessment_date?: string
}

export interface ListVendorsQuery {
  status?: string
  criticality?: string
  category?: string
  owner_id?: string
  search?: string
  contract_expiring?: boolean
  limit?: number
  offset?: number
}

export interface VendorStats {
  total: number
  by_criticality: { criticality: string | null; count: number }[]
  by_category: { category: string | null; count: number }[]
  active: number
  under_review: number
  contracts_expiring_soon: number
  needs_assessment: number
}

// ==================== Asset ====================

export type AssetType = 'hardware' | 'software' | 'data' | 'service' | 'infrastructure' | 'other'
export type AssetClassification = 'public' | 'internal' | 'confidential' | 'restricted'
export type AssetStatus = 'active' | 'inactive' | 'decommissioned' | 'pending'

export interface Asset {
  id: string
  organization_id: string
  name: string
  description: string | null
  asset_type: string | null
  category: string | null
  classification: string | null
  status: string | null
  owner_id: string | null
  custodian_id: string | null
  location: string | null
  ip_address: string | null
  mac_address: string | null
  purchase_date: string | null
  warranty_until: string | null
  metadata: Record<string, unknown> | null
  created_at: string
  updated_at: string
  // Lifecycle tracking fields
  lifecycle_stage: string | null
  commissioned_date: string | null
  decommission_date: string | null
  last_maintenance_date: string | null
  next_maintenance_due: string | null
  maintenance_frequency: string | null
  end_of_life_date: string | null
  end_of_support_date: string | null
  // Integration tracking fields
  integration_source: string | null
  integration_id: string | null
  external_id: string | null
  last_synced_at: string | null
}

export interface AssetWithControls extends Asset {
  linked_control_count: number
  linked_controls?: LinkedControl[]
}

export interface CreateAsset {
  name: string
  description?: string
  asset_type?: AssetType
  category?: string
  classification?: AssetClassification
  status?: AssetStatus
  owner_id?: string
  custodian_id?: string
  location?: string
  ip_address?: string
  mac_address?: string
  purchase_date?: string
  warranty_until?: string
  metadata?: Record<string, unknown>
  // Lifecycle fields
  lifecycle_stage?: string
  commissioned_date?: string
  maintenance_frequency?: string
  end_of_life_date?: string
  end_of_support_date?: string
}

export interface UpdateAsset {
  name?: string
  description?: string
  asset_type?: string
  category?: string
  classification?: string
  status?: string
  owner_id?: string
  custodian_id?: string
  location?: string
  ip_address?: string
  mac_address?: string
  purchase_date?: string
  warranty_until?: string
  metadata?: Record<string, unknown>
  // Lifecycle fields
  lifecycle_stage?: string
  commissioned_date?: string
  decommission_date?: string
  last_maintenance_date?: string
  next_maintenance_due?: string
  maintenance_frequency?: string
  end_of_life_date?: string
  end_of_support_date?: string
}

export interface ListAssetsQuery {
  asset_type?: string
  classification?: string
  status?: string
  owner_id?: string
  search?: string
  limit?: number
  offset?: number
}

export interface AssetStats {
  total: number
  by_type: { asset_type: string | null; count: number }[]
  by_classification: { classification: string | null; count: number }[]
  by_status: { status: string | null; count: number }[]
  by_lifecycle_stage: { lifecycle_stage: string | null; count: number }[]
  warranty_expiring_soon: number
  maintenance_due_soon: number
  from_integrations: number
}

// ==================== Audit ====================

export type AuditType = 'external' | 'internal' | 'certification' | 'compliance' | 'readiness'
export type AuditStatus = 'planning' | 'fieldwork' | 'review' | 'completed' | 'cancelled'
export type FindingType = 'observation' | 'exception' | 'deficiency' | 'recommendation'
export type FindingStatus = 'open' | 'remediation' | 'closed' | 'accepted'
export type RequestStatus = 'open' | 'in_progress' | 'submitted' | 'accepted' | 'rejected'

export interface Audit {
  id: string
  organization_id: string
  name: string
  framework_id: string | null
  audit_type: string
  auditor_firm: string | null
  auditor_contact: string | null
  period_start: string | null
  period_end: string | null
  status: string
  created_at: string
  updated_at: string
}

export interface AuditWithStats extends Audit {
  request_count: number
  open_requests: number
  finding_count: number
  open_findings: number
}

export interface AuditRequest {
  id: string
  audit_id: string
  request_type: string | null
  title: string
  description: string | null
  status: string
  assigned_to: string | null
  due_at: string | null
  created_at: string
  updated_at: string
}

export interface AuditRequestResponse {
  id: string
  audit_request_id: string
  response_text: string | null
  evidence_ids: string[] | null
  responded_by: string | null
  responded_at: string
  created_at: string
}

export interface AuditFinding {
  id: string
  audit_id: string
  finding_type: string
  title: string
  description: string | null
  recommendation: string | null
  control_ids: string[] | null
  status: string
  remediation_plan: string | null
  remediation_due: string | null
  created_at: string
  updated_at: string
}

export interface CreateAudit {
  name: string
  framework_id?: string
  audit_type: AuditType
  auditor_firm?: string
  auditor_contact?: string
  period_start?: string
  period_end?: string
  status?: AuditStatus
}

export interface UpdateAudit {
  name?: string
  framework_id?: string
  audit_type?: string
  auditor_firm?: string
  auditor_contact?: string
  period_start?: string
  period_end?: string
  status?: string
}

export interface CreateAuditRequest {
  request_type?: string
  title: string
  description?: string
  assigned_to?: string
  due_at?: string
}

export interface CreateAuditFinding {
  finding_type: FindingType
  title: string
  description?: string
  recommendation?: string
  control_ids?: string[]
  remediation_plan?: string
  remediation_due?: string
}

export interface UpdateAuditFinding {
  finding_type?: string
  title?: string
  description?: string
  recommendation?: string
  control_ids?: string[]
  status?: string
  remediation_plan?: string
  remediation_due?: string
}

export interface CreateRequestResponse {
  response_text?: string
  evidence_ids?: string[]
}

export interface ListAuditsQuery {
  status?: string
  audit_type?: string
  framework_id?: string
  search?: string
  limit?: number
  offset?: number
}

export interface AuditStats {
  total: number
  by_type: { audit_type: string; count: number }[]
  active: number
  completed: number
  total_requests: number
  open_requests: number
  total_findings: number
  open_findings: number
}

export interface AuditEvidenceItem {
  id: string
  title: string
  description: string | null
  evidence_type: string
  source: string
  file_path: string | null
  file_size: number | null
  mime_type: string | null
  collected_at: string
  linked_controls: string[]
  linked_requests: string[]
}

export interface AuditEvidencePackage {
  audit_id: string
  audit_name: string
  framework_name: string | null
  period_start: string | null
  period_end: string | null
  evidence_count: number
  total_file_size: number
  evidence: AuditEvidenceItem[]
  generated_at: string
}

// ==================== Task ====================

export type TaskType = 'control_test' | 'evidence_collection' | 'review' | 'remediation' | 'general'
export type TaskStatus = 'open' | 'in_progress' | 'completed' | 'overdue'
export type TaskPriority = 'low' | 'medium' | 'high' | 'critical'
export type RecurrencePattern = 'daily' | 'weekly' | 'biweekly' | 'monthly' | 'quarterly' | 'yearly'

export interface Task {
  id: string
  organization_id: string
  title: string
  description: string | null
  task_type: string
  related_entity_type: string | null
  related_entity_id: string | null
  assignee_id: string | null
  assignee_name?: string | null
  assignee_email?: string | null
  due_at: string | null
  completed_at: string | null
  status: string
  priority: string
  created_by: string | null
  created_at: string
  updated_at: string
  // Recurrence fields
  is_recurring: boolean
  recurrence_pattern: string | null
  recurrence_interval: number | null
  recurrence_day_of_week: number | null
  recurrence_day_of_month: number | null
  recurrence_month_of_year: number | null
  recurrence_end_at: string | null
  recurrence_count: number | null
  recurrence_occurrences: number | null
  parent_task_id: string | null
  next_occurrence_at: string | null
  last_occurrence_at: string | null
}

export interface TaskComment {
  id: string
  task_id: string
  user_id: string
  user_name?: string
  user_email?: string
  content: string
  created_at: string
}

export interface CreateTask {
  title: string
  description?: string
  task_type: TaskType
  related_entity_type?: string
  related_entity_id?: string
  assignee_id?: string
  due_at?: string
  priority: TaskPriority
  // Recurrence fields
  is_recurring?: boolean
  recurrence_pattern?: RecurrencePattern
  recurrence_interval?: number
  recurrence_day_of_week?: number
  recurrence_day_of_month?: number
  recurrence_month_of_year?: number
  recurrence_end_at?: string
  recurrence_count?: number
}

export interface UpdateTask {
  title?: string
  description?: string
  task_type?: string
  assignee_id?: string
  due_at?: string | Date
  status?: string
  priority?: string
  // Recurrence fields
  is_recurring?: boolean
  recurrence_pattern?: string
  recurrence_interval?: number
  recurrence_day_of_week?: number
  recurrence_day_of_month?: number
  recurrence_month_of_year?: number
  recurrence_end_at?: string
  recurrence_count?: number
}

export interface TaskRecurrenceHistory {
  id: string
  task_id: string
  occurrence_number: number
  created_task_id: string | null
  scheduled_at: string
  created_at: string
  skipped: boolean
  skip_reason: string | null
}

export interface CreateTaskComment {
  content: string
}

export interface TaskStats {
  total: number
  open: number
  in_progress: number
  completed: number
  overdue: number
  by_type: { task_type: string; count: number }[]
  by_priority: { priority: string; count: number }[]
  by_assignee: { assignee_id: string | null; assignee_name: string | null; count: number }[]
  due_today: number
  due_this_week: number
}

// ==================== Integration ====================

export type IntegrationType = 'aws' | 'gcp' | 'azure' | 'github' | 'gitlab' | 'okta' | 'google_workspace' | 'azure_ad' | 'jira' | 'datadog' | 'pagerduty' | 'cloudflare' | 'webhook'
export type IntegrationStatus = 'active' | 'inactive' | 'error' | 'syncing'

export interface Integration {
  id: string
  organization_id: string
  integration_type: string
  name: string
  config: Record<string, unknown> | null
  status: string
  last_sync_at: string | null
  last_error: string | null
  created_at: string
  updated_at: string
}

export interface IntegrationWithStats {
  integration: Integration
  sync_count: number
  last_sync_status: string | null
  records_synced: number | null
}

export interface IntegrationSyncLog {
  id: string
  integration_id: string
  sync_type: string | null
  started_at: string
  completed_at: string | null
  status: string | null
  records_processed: number | null
  errors: Record<string, unknown> | null
  created_at: string
}

export interface AvailableIntegration {
  integration_type: string
  name: string
  description: string
  category: string
  capabilities: string[]
  config_schema: Record<string, unknown>
  logo_url: string | null
}

export interface IntegrationStats {
  total: number
  active: number
  inactive: number
  error: number
  by_type: { integration_type: string; count: number }[]
}

export interface TestConnectionResult {
  success: boolean
  message: string
  details: Record<string, unknown> | null
}

export interface CreateIntegration {
  integration_type: string
  name: string
  config?: Record<string, unknown>
}

export interface UpdateIntegration {
  name?: string
  config?: Record<string, unknown>
  status?: string
}

export interface TriggerSyncRequest {
  sync_type?: string
  full_sync?: boolean
}

// ==================== Integration Health ====================

export type HealthStatus = 'healthy' | 'degraded' | 'unhealthy' | 'unknown'

export interface IntegrationHealth {
  id: string
  integration_id: string
  status: HealthStatus
  last_successful_sync_at: string | null
  consecutive_failures: number
  sync_success_count_24h: number
  sync_failure_count_24h: number
  average_sync_duration_ms: number | null
  sync_success_count_7d: number
  sync_failure_count_7d: number
  last_check_at: string | null
  last_check_message: string | null
  last_error_at: string | null
  last_error_message: string | null
  created_at: string
  updated_at: string
}

export interface IntegrationHealthWithDetails {
  integration_id: string
  integration_name: string
  integration_type: string
  health: IntegrationHealth
  success_rate_24h: number
  success_rate_7d: number
}

export interface IntegrationHealthStats {
  total_integrations: number
  healthy_count: number
  degraded_count: number
  unhealthy_count: number
  unknown_count: number
  overall_success_rate_24h: number
  overall_success_rate_7d: number
  average_sync_duration_ms: number | null
  total_syncs_24h: number
  total_failures_24h: number
}

export interface HealthTrendPoint {
  timestamp: string
  healthy_count: number
  degraded_count: number
  unhealthy_count: number
  success_rate: number
}

export interface RecentFailure {
  integration_id: string
  integration_name: string
  integration_type: string
  error_message: string | null
  failed_at: string
  consecutive_failures: number
}

// ==================== AWS Integration ====================

export interface AwsOverview {
  account_id: string
  integration_id: string
  last_sync_at: string | null
  iam_stats: {
    total_users: number
    users_with_mfa: number
    users_with_access_keys: number
    total_roles: number
    total_policies: number
    admin_policies: number
  }
  security_stats: {
    total_findings: number
    critical: number
    high: number
    medium: number
    low: number
    informational: number
  }
  config_stats: {
    total_rules: number
    compliant: number
    non_compliant: number
    not_applicable: number
  }
  resource_stats: {
    s3_buckets: number
    ec2_instances: number
    rds_instances: number
    security_groups: number
  }
  cloudtrail_stats: {
    total_events_24h: number
    root_events: number
    sensitive_events: number
  }
}

export interface AwsIamUser {
  id: string
  user_name: string
  user_id: string
  arn: string
  path: string
  created_date: string
  password_last_used: string | null
  mfa_enabled: boolean
  mfa_devices: string[]
  access_keys: AwsAccessKey[]
  attached_policies: string[]
  inline_policies: string[]
  groups: string[]
  tags: Record<string, string>
}

export interface AwsAccessKey {
  access_key_id: string
  status: string
  created_date: string
  last_used_date: string | null
  last_used_service: string | null
  last_used_region: string | null
}

export interface AwsIamRole {
  id: string
  role_name: string
  role_id: string
  arn: string
  path: string
  assume_role_policy: string
  created_date: string
  max_session_duration: number
  attached_policies: string[]
  inline_policies: string[]
  last_used_at: string | null
  last_used_region: string | null
  tags: Record<string, string>
}

export interface AwsIamPolicy {
  id: string
  policy_name: string
  policy_id: string
  arn: string
  path: string
  policy_document: string
  attachment_count: number
  allows_admin_access: boolean
  uses_wildcard_resources: boolean
  risk_score: number
  is_aws_managed: boolean
}

export interface AwsSecurityFinding {
  id: string
  finding_id: string
  aws_account_id: string
  region: string
  product_name: string
  generator_id: string
  types: string[]
  title: string
  description: string
  severity_label: string
  severity_normalized: number
  workflow_status: string
  record_state: string
  compliance_status: string | null
  compliance_standards: string[]
  related_resources: AwsRelatedResource[]
  remediation: string | null
  first_observed_at: string
  last_observed_at: string
  mapped_control_codes: string[]
}

export interface AwsRelatedResource {
  type: string
  id: string
  partition: string
  region: string
}

export interface AwsConfigRule {
  id: string
  config_rule_name: string
  config_rule_arn: string
  description: string | null
  source_identifier: string
  source_owner: string
  compliance_type: string
  compliant_count: number
  non_compliant_count: number
  region: string
  mapped_control_codes: string[]
}

export interface AwsS3Bucket {
  id: string
  bucket_name: string
  region: string
  creation_date: string
  versioning_enabled: boolean
  encryption_enabled: boolean
  encryption_type: string | null
  public_access_blocked: boolean
  logging_enabled: boolean
  tags: Record<string, string>
}

export interface AwsEc2Instance {
  id: string
  instance_id: string
  instance_type: string
  region: string
  availability_zone: string
  state: string
  public_ip: string | null
  private_ip: string | null
  vpc_id: string | null
  subnet_id: string | null
  security_groups: string[]
  iam_role: string | null
  monitoring_enabled: boolean
  launch_time: string
  tags: Record<string, string>
}

export interface AwsSecurityGroup {
  id: string
  group_id: string
  group_name: string
  description: string
  vpc_id: string
  region: string
  inbound_rules: AwsSecurityGroupRule[]
  outbound_rules: AwsSecurityGroupRule[]
  tags: Record<string, string>
}

export interface AwsSecurityGroupRule {
  protocol: string
  from_port: number
  to_port: number
  source: string
}

export interface AwsRdsInstance {
  id: string
  db_instance_identifier: string
  db_instance_class: string
  engine: string
  engine_version: string
  region: string
  availability_zone: string
  multi_az: boolean
  publicly_accessible: boolean
  storage_encrypted: boolean
  storage_type: string
  allocated_storage: number
  vpc_id: string | null
  db_subnet_group: string | null
  security_groups: string[]
  status: string
  endpoint: string | null
  port: number | null
  created_time: string
  tags: Record<string, string>
}

export interface AwsCloudTrailEvent {
  id: string
  event_id: string
  event_name: string
  event_source: string
  event_time: string
  event_type: string
  user_identity: AwsUserIdentity
  source_ip_address: string | null
  user_agent: string | null
  region: string
  request_parameters: Record<string, unknown> | null
  response_elements: Record<string, unknown> | null
  error_code: string | null
  error_message: string | null
  is_root_action: boolean
  is_sensitive_action: boolean
  risk_level: string
}

export interface AwsUserIdentity {
  type: string
  principal_id: string | null
  arn: string | null
  account_id: string | null
  user_name: string | null
  access_key_id: string | null
}

export interface AwsFindingsSummary {
  total: number
  by_severity: {
    critical: number
    high: number
    medium: number
    low: number
    informational: number
  }
  by_status: {
    new: number
    notified: number
    resolved: number
    suppressed: number
  }
  by_product: Array<{ product: string; count: number }>
  top_types: Array<{ type: string; count: number }>
}

export interface AwsCloudTrailStats {
  total_events_24h: number
  events_by_hour: Array<{ hour: string; count: number }>
  top_users: Array<{ user: string; count: number }>
  top_events: Array<{ event: string; count: number }>
  root_activity_count: number
  sensitive_actions_count: number
  error_count: number
}

export interface AwsPaginatedResponse<T> {
  data: T[]
  total: number
  limit: number
  offset: number
}

// ==================== Access Reviews ====================

export type AccessReviewStatus = 'draft' | 'active' | 'completed' | 'cancelled'
export type ReviewDecisionStatus = 'pending' | 'approved' | 'revoked' | 'flagged'
export type AccessRemovalStatus = 'pending' | 'in_progress' | 'completed' | 'failed' | 'cancelled'

export interface AccessReviewCampaign {
  id: string
  organization_id: string
  name: string
  description: string | null
  status: string | null
  started_at: string | null
  due_at: string | null
  completed_at: string | null
  created_by: string | null
  created_at: string
  integration_type: string | null
  integration_id: string | null
  scope: Record<string, unknown> | null
  review_type: string | null
  reminder_sent_at: string | null
  last_sync_at: string | null
}

export interface CampaignWithStats extends AccessReviewCampaign {
  total_items: number
  pending_items: number
  approved_items: number
  revoked_items: number
}

export interface AccessReviewItem {
  id: string
  campaign_id: string
  user_identifier: string
  user_name: string | null
  user_email: string | null
  access_details: Record<string, unknown> | null
  reviewer_id: string | null
  review_status: string | null
  reviewed_at: string | null
  review_notes: string | null
  created_at: string
  integration_user_id: string | null
  department: string | null
  manager: string | null
  last_login_at: string | null
  risk_level: string | null
  mfa_enabled: boolean | null
  is_admin: boolean | null
  applications: Record<string, unknown>[] | null
  removal_requested_at: string | null
  removal_completed_at: string | null
  removal_ticket_id: string | null
}

export interface AccessRemovalLog {
  id: string
  access_review_item_id: string
  campaign_id: string
  organization_id: string
  user_identifier: string
  user_name: string | null
  access_type: string | null
  access_description: string | null
  action: string
  action_reason: string | null
  requested_by: string | null
  requested_at: string
  executed_by: string | null
  executed_at: string | null
  status: string | null
  error_message: string | null
  external_ticket_id: string | null
  external_ticket_url: string | null
  created_at: string
  updated_at: string
}

export interface CreateAccessReviewCampaign {
  name: string
  description?: string
  due_at?: string
  integration_type?: string
  integration_id?: string
  scope?: Record<string, unknown>
  review_type?: string
}

export interface UpdateAccessReviewCampaign {
  name?: string
  description?: string
  status?: string
  due_at?: string
}

export interface CreateAccessReviewItem {
  user_identifier: string
  user_name?: string
  user_email?: string
  access_details?: Record<string, unknown>
  integration_user_id?: string
  department?: string
  manager?: string
  last_login_at?: string
  risk_level?: string
  mfa_enabled?: boolean
  is_admin?: boolean
  applications?: Record<string, unknown>[]
}

export interface ReviewDecision {
  status: ReviewDecisionStatus
  notes?: string
}

export interface BulkReviewDecision {
  item_ids: string[]
  status: ReviewDecisionStatus
  notes?: string
}

export interface RequestRemoval {
  access_type?: string
  access_description?: string
  action: string
  action_reason?: string
  external_ticket_id?: string
  external_ticket_url?: string
}

export interface AccessReviewStats {
  total_campaigns: number
  active_campaigns: number
  completed_campaigns: number
  total_items: number
  pending_reviews: number
  approved_accesses: number
  revoked_accesses: number
  pending_removals: number
  completed_removals: number
  high_risk_users: number
  admin_users: number
  users_without_mfa: number
}

export interface AccessReviewCertificationReport {
  campaign_name: string
  campaign_status: string | null
  review_type: string | null
  integration_type: string | null
  started_at: string | null
  completed_at: string | null
  due_at: string | null
  total_items: number
  approved_items: number
  revoked_items: number
  pending_items: number
  high_risk_items: number
  admin_items: number
  items: AccessReviewCertificationRow[]
}

export interface AccessReviewCertificationRow {
  user_identifier: string
  user_name: string | null
  user_email: string | null
  department: string | null
  review_status: string | null
  reviewed_at: string | null
  reviewer_name: string | null
  review_notes: string | null
  risk_level: string | null
  is_admin: boolean | null
  mfa_enabled: boolean | null
  last_login_at: string | null
}

// ==================== Dashboard ====================

export interface DashboardStats {
  controls: ControlStats
  evidence: EvidenceStats
  policies: PolicyStats
  risks: RiskStats
  vendors: VendorStats
  assets: AssetStats
  audits: AuditStats
}

// ==================== Utility Types ====================

export function getRiskLevel(score: number | null): 'critical' | 'high' | 'medium' | 'low' | 'unknown' {
  if (score === null) return 'unknown'
  if (score >= 15) return 'critical'
  if (score >= 10) return 'high'
  if (score >= 5) return 'medium'
  return 'low'
}

export function formatStatus(status: string): string {
  return status
    .split('_')
    .map(word => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ')
}

export function formatDate(dateString: string | null): string {
  if (!dateString) return '-'
  return new Date(dateString).toLocaleDateString()
}

export function formatDateTime(dateString: string | null): string {
  if (!dateString) return '-'
  return new Date(dateString).toLocaleString()
}

export function formatRelativeTime(dateString: string | null): string {
  if (!dateString) return '-'

  const date = new Date(dateString)
  const now = new Date()
  const diffMs = now.getTime() - date.getTime()
  const diffSeconds = Math.floor(diffMs / 1000)
  const diffMinutes = Math.floor(diffSeconds / 60)
  const diffHours = Math.floor(diffMinutes / 60)
  const diffDays = Math.floor(diffHours / 24)

  if (diffSeconds < 60) {
    return 'just now'
  } else if (diffMinutes < 60) {
    return `${diffMinutes}m ago`
  } else if (diffHours < 24) {
    return `${diffHours}h ago`
  } else if (diffDays < 7) {
    return `${diffDays}d ago`
  } else {
    return formatDate(dateString)
  }
}
