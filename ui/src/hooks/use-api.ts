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

export function useVendorStats() {
  return useApi<import('@/types').VendorStats>('/vendors/stats')
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
