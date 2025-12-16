'use client'

import { useState, useEffect, useCallback } from 'react'
import { apiClient, ApiError } from '@/lib/api-client'

interface UseApiState<T> {
  data: T | null
  isLoading: boolean
  error: ApiError | null
}

interface UseApiOptions {
  enabled?: boolean
}

export function useApi<T>(
  endpoint: string,
  options: UseApiOptions = {}
): UseApiState<T> & { refetch: () => Promise<void> } {
  const { enabled = true } = options
  const [state, setState] = useState<UseApiState<T>>({
    data: null,
    isLoading: true,
    error: null,
  })

  const fetchData = useCallback(async () => {
    if (!enabled) {
      setState({ data: null, isLoading: false, error: null })
      return
    }

    setState(prev => ({ ...prev, isLoading: true, error: null }))

    try {
      const data = await apiClient.get<T>(endpoint)
      setState({ data, isLoading: false, error: null })
    } catch (error) {
      setState({
        data: null,
        isLoading: false,
        error: error as ApiError,
      })
    }
  }, [endpoint, enabled])

  useEffect(() => {
    fetchData()
  }, [fetchData])

  return {
    ...state,
    refetch: fetchData,
  }
}

export function useMutation<TData, TVariables>(
  mutationFn: (variables: TVariables) => Promise<TData>
) {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<ApiError | null>(null)
  const [data, setData] = useState<TData | null>(null)

  const mutate = useCallback(
    async (variables: TVariables) => {
      setIsLoading(true)
      setError(null)

      try {
        const result = await mutationFn(variables)
        setData(result)
        return result
      } catch (err) {
        const apiError = err as ApiError
        setError(apiError)
        throw apiError
      } finally {
        setIsLoading(false)
      }
    },
    [mutationFn]
  )

  return {
    mutate,
    isLoading,
    error,
    data,
    reset: () => {
      setError(null)
      setData(null)
    },
  }
}

// Specific hooks for each entity type
export function useControls(query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').ControlWithMappings[]>(`/controls${queryString}`)
}

export function useControl(id: string) {
  return useApi<import('@/types').ControlWithMappings>(`/controls/${id}`, {
    enabled: !!id,
  })
}

export function useControlStats() {
  return useApi<import('@/types').ControlStats>('/controls/stats')
}

export function useFrameworks() {
  return useApi<import('@/types').Framework[]>('/frameworks')
}

export function useFramework(id: string) {
  return useApi<import('@/types').FrameworkWithRequirements>(`/frameworks/${id}`, {
    enabled: !!id,
  })
}

export function useEvidence(query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').EvidenceWithLinks[]>(`/evidence${queryString}`)
}

export function useEvidenceStats() {
  return useApi<import('@/types').EvidenceStats>('/evidence/stats')
}

export function usePolicies(query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').PolicyWithStats[]>(`/policies${queryString}`)
}

export function usePolicyStats() {
  return useApi<import('@/types').PolicyStats>('/policies/stats')
}

export function useRisks(query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').RiskWithControls[]>(`/risks${queryString}`)
}

export function useRiskStats() {
  return useApi<import('@/types').RiskStats>('/risks/stats')
}

export function useVendors(query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').VendorWithAssessment[]>(`/vendors${queryString}`)
}

export function useVendor(id: string) {
  return useApi<import('@/types').VendorWithAssessment>(`/vendors/${id}`, {
    enabled: !!id,
  })
}

export function useVendorStats() {
  return useApi<import('@/types').VendorStats>('/vendors/stats')
}

export function useVendorAssessments(vendorId: string) {
  return useApi<import('@/types').VendorAssessment[]>(
    `/vendors/${vendorId}/assessments`,
    { enabled: !!vendorId }
  )
}

export function useAssets(query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AssetWithControls[]>(`/assets${queryString}`)
}

export function useAsset(id: string) {
  return useApi<import('@/types').AssetWithControls>(`/assets/${id}`, {
    enabled: !!id,
  })
}

export function useAssetStats() {
  return useApi<import('@/types').AssetStats>('/assets/stats')
}

export function useAudits(query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AuditWithStats[]>(`/audits${queryString}`)
}

export function useAuditStats() {
  return useApi<import('@/types').AuditStats>('/audits/stats')
}

export function useRiskHeatmap() {
  return useApi<import('@/types').RiskHeatmapData>('/risks/heatmap')
}

export function useGapAnalysis(frameworkId: string) {
  return useApi<import('@/types').FrameworkGapAnalysis>(
    `/frameworks/${frameworkId}/gap-analysis`,
    { enabled: !!frameworkId }
  )
}

// Integration hooks
export function useIntegrations(query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<{ data: import('@/types').IntegrationWithStats[], count: number }>(`/integrations${queryString}`)
}

export function useIntegration(id: string) {
  return useApi<{ data: import('@/types').IntegrationWithStats }>(`/integrations/${id}`, {
    enabled: !!id,
  })
}

export function useIntegrationStats() {
  return useApi<{ data: import('@/types').IntegrationStats }>('/integrations/stats')
}

export function useAvailableIntegrations() {
  return useApi<{ data: import('@/types').AvailableIntegration[], count: number }>('/integrations/available')
}

export function useIntegrationSyncLogs(integrationId: string, limit?: number) {
  return useApi<{ data: import('@/types').IntegrationSyncLog[], count: number }>(
    `/integrations/${integrationId}/logs${limit ? `?limit=${limit}` : ''}`,
    { enabled: !!integrationId }
  )
}

// Integration Health hooks
export function useIntegrationHealth() {
  return useApi<{ data: import('@/types').IntegrationHealthWithDetails[], count: number }>(
    '/integrations/health'
  )
}

export function useIntegrationHealthById(integrationId: string) {
  return useApi<{ data: import('@/types').IntegrationHealthWithDetails }>(
    `/integrations/${integrationId}/health`,
    { enabled: !!integrationId }
  )
}

export function useIntegrationHealthStats() {
  return useApi<{ data: import('@/types').IntegrationHealthStats }>(
    '/integrations/health/stats'
  )
}

export function useIntegrationHealthFailures(limit?: number) {
  return useApi<{ data: import('@/types').RecentFailure[], count: number }>(
    `/integrations/health/failures${limit ? `?limit=${limit}` : ''}`
  )
}

export function useIntegrationHealthTrend(hours?: number) {
  return useApi<{ data: import('@/types').HealthTrendPoint[], count: number }>(
    `/integrations/health/trend${hours ? `?hours=${hours}` : ''}`
  )
}

// AWS Integration hooks
export function useAwsOverview(integrationId: string) {
  return useApi<{ data: import('@/types').AwsOverview }>(
    `/integrations/${integrationId}/aws/overview`,
    { enabled: !!integrationId }
  )
}

export function useAwsIamUsers(integrationId: string, query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AwsPaginatedResponse<import('@/types').AwsIamUser>>(
    `/integrations/${integrationId}/aws/iam/users${queryString}`,
    { enabled: !!integrationId }
  )
}

export function useAwsIamRoles(integrationId: string, query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AwsPaginatedResponse<import('@/types').AwsIamRole>>(
    `/integrations/${integrationId}/aws/iam/roles${queryString}`,
    { enabled: !!integrationId }
  )
}

export function useAwsIamPolicies(integrationId: string, query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AwsPaginatedResponse<import('@/types').AwsIamPolicy>>(
    `/integrations/${integrationId}/aws/iam/policies${queryString}`,
    { enabled: !!integrationId }
  )
}

export function useAwsFindings(integrationId: string, query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AwsPaginatedResponse<import('@/types').AwsSecurityFinding>>(
    `/integrations/${integrationId}/aws/findings${queryString}`,
    { enabled: !!integrationId }
  )
}

export function useAwsFindingsSummary(integrationId: string) {
  return useApi<{ data: import('@/types').AwsFindingsSummary }>(
    `/integrations/${integrationId}/aws/findings/summary`,
    { enabled: !!integrationId }
  )
}

export function useAwsConfigRules(integrationId: string, query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AwsPaginatedResponse<import('@/types').AwsConfigRule>>(
    `/integrations/${integrationId}/aws/config-rules${queryString}`,
    { enabled: !!integrationId }
  )
}

export function useAwsS3Buckets(integrationId: string, query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AwsPaginatedResponse<import('@/types').AwsS3Bucket>>(
    `/integrations/${integrationId}/aws/s3/buckets${queryString}`,
    { enabled: !!integrationId }
  )
}

export function useAwsEc2Instances(integrationId: string, query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AwsPaginatedResponse<import('@/types').AwsEc2Instance>>(
    `/integrations/${integrationId}/aws/ec2/instances${queryString}`,
    { enabled: !!integrationId }
  )
}

export function useAwsSecurityGroups(integrationId: string, query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AwsPaginatedResponse<import('@/types').AwsSecurityGroup>>(
    `/integrations/${integrationId}/aws/ec2/security-groups${queryString}`,
    { enabled: !!integrationId }
  )
}

export function useAwsRdsInstances(integrationId: string, query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AwsPaginatedResponse<import('@/types').AwsRdsInstance>>(
    `/integrations/${integrationId}/aws/rds/instances${queryString}`,
    { enabled: !!integrationId }
  )
}

export function useAwsCloudTrailEvents(integrationId: string, query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AwsPaginatedResponse<import('@/types').AwsCloudTrailEvent>>(
    `/integrations/${integrationId}/aws/cloudtrail${queryString}`,
    { enabled: !!integrationId }
  )
}

export function useAwsCloudTrailStats(integrationId: string) {
  return useApi<{ data: import('@/types').AwsCloudTrailStats }>(
    `/integrations/${integrationId}/aws/cloudtrail/stats`,
    { enabled: !!integrationId }
  )
}

// Access Review hooks
export function useAccessReviewCampaigns(query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').CampaignWithStats[]>(`/access-reviews/campaigns${queryString}`)
}

export function useAccessReviewCampaign(id: string) {
  return useApi<import('@/types').CampaignWithStats>(`/access-reviews/campaigns/${id}`, {
    enabled: !!id,
  })
}

export function useAccessReviewStats() {
  return useApi<import('@/types').AccessReviewStats>('/access-reviews/stats')
}

export function useAccessReviewItems(campaignId: string, query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').AccessReviewItem[]>(
    `/access-reviews/campaigns/${campaignId}/items${queryString}`,
    { enabled: !!campaignId }
  )
}

export function useAccessRemovalLogs(campaignId: string) {
  return useApi<import('@/types').AccessRemovalLog[]>(
    `/access-reviews/campaigns/${campaignId}/removal-logs`,
    { enabled: !!campaignId }
  )
}

export function useAccessReviewCertification(campaignId: string) {
  return useApi<import('@/types').AccessReviewCertificationReport>(
    `/access-reviews/campaigns/${campaignId}/certification`,
    { enabled: !!campaignId }
  )
}

// Audit hooks
export function useAudit(id: string) {
  return useApi<import('@/types').AuditWithStats>(`/audits/${id}`, {
    enabled: !!id,
  })
}

export function useAuditRequests(auditId: string) {
  return useApi<import('@/types').AuditRequest[]>(
    `/audits/${auditId}/requests`,
    { enabled: !!auditId }
  )
}

export function useAuditFindings(auditId: string) {
  return useApi<import('@/types').AuditFinding[]>(
    `/audits/${auditId}/findings`,
    { enabled: !!auditId }
  )
}

export function useAuditEvidencePackage(auditId: string) {
  return useApi<import('@/types').AuditEvidencePackage>(
    `/audits/${auditId}/evidence-package`,
    { enabled: !!auditId }
  )
}

// Task hooks
export function useTasks(query?: Record<string, string | number | boolean>) {
  const queryString = query
    ? '?' + new URLSearchParams(
        Object.entries(query)
          .filter(([, v]) => v !== undefined && v !== '')
          .map(([k, v]) => [k, String(v)])
      ).toString()
    : ''
  return useApi<import('@/types').Task[]>(`/tasks${queryString}`)
}

export function useTask(id: string) {
  return useApi<import('@/types').Task>(`/tasks/${id}`, {
    enabled: !!id,
  })
}

export function useTaskStats() {
  return useApi<import('@/types').TaskStats>('/tasks/stats')
}

export function useTaskComments(taskId: string) {
  return useApi<import('@/types').TaskComment[]>(
    `/tasks/${taskId}/comments`,
    { enabled: !!taskId }
  )
}

export function useMyTasks() {
  return useApi<import('@/types').Task[]>('/tasks/my')
}

export function useOverdueTasks() {
  return useApi<import('@/types').Task[]>('/tasks/overdue')
}

export function useRecurringTasks() {
  return useApi<import('@/types').Task[]>('/tasks/recurring')
}

export function useTaskOccurrences(taskId: string) {
  return useApi<import('@/types').Task[]>(
    `/tasks/${taskId}/occurrences`,
    { enabled: !!taskId }
  )
}

export function useTaskRecurrenceHistory(taskId: string) {
  return useApi<import('@/types').TaskRecurrenceHistory[]>(
    `/tasks/${taskId}/recurrence-history`,
    { enabled: !!taskId }
  )
}
