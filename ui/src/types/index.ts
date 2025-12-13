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
  asset_type: string
  category: string | null
  classification: string | null
  status: string
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
}

export interface AssetWithControls extends Asset {
  linked_control_count: number
  linked_controls?: LinkedControl[]
}

export interface CreateAsset {
  name: string
  description?: string
  asset_type: AssetType
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
  by_type: { asset_type: string; count: number }[]
  by_classification: { classification: string | null; count: number }[]
  by_status: { status: string; count: number }[]
  active: number
  warranty_expiring: number
}

// ==================== Audit ====================

export type AuditType = 'type1' | 'type2' | 'certification' | 'internal' | 'other'
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
