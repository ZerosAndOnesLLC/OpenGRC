'use client'

import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Loading } from "@/components/loading"
import { Progress } from "@/components/ui/progress"
import {
  Activity,
  AlertTriangle,
  CheckCircle2,
  Clock,
  RefreshCw,
  TrendingUp,
  XCircle,
  HelpCircle,
  ArrowLeft,
  Zap,
} from "lucide-react"
import Link from "next/link"
import {
  useIntegrationHealth,
  useIntegrationHealthStats,
  useIntegrationHealthFailures,
  useIntegrationHealthTrend,
} from '@/hooks/use-api'
import type { IntegrationHealthWithDetails, IntegrationHealthStats, RecentFailure, HealthStatus } from '@/types'
import { formatDateTime, formatRelativeTime } from '@/types'

const healthStatusConfig: Record<HealthStatus, {
  label: string
  variant: 'success' | 'warning' | 'destructive' | 'secondary'
  icon: React.ReactNode
  color: string
}> = {
  healthy: {
    label: 'Healthy',
    variant: 'success',
    icon: <CheckCircle2 className="h-4 w-4" />,
    color: 'text-green-500',
  },
  degraded: {
    label: 'Degraded',
    variant: 'warning',
    icon: <AlertTriangle className="h-4 w-4" />,
    color: 'text-yellow-500',
  },
  unhealthy: {
    label: 'Unhealthy',
    variant: 'destructive',
    icon: <XCircle className="h-4 w-4" />,
    color: 'text-red-500',
  },
  unknown: {
    label: 'Unknown',
    variant: 'secondary',
    icon: <HelpCircle className="h-4 w-4" />,
    color: 'text-gray-500',
  },
}

function HealthStatsCards({ stats }: { stats: IntegrationHealthStats | null }) {
  if (!stats) return null

  const healthPercentage = stats.total_integrations > 0
    ? ((stats.healthy_count / stats.total_integrations) * 100)
    : 100

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Total Integrations</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Activity className="h-5 w-5 text-primary" />
            <span className="text-2xl font-bold">{stats.total_integrations}</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Healthy</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <CheckCircle2 className="h-5 w-5 text-green-500" />
            <span className="text-2xl font-bold text-green-600">{stats.healthy_count}</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Degraded</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-yellow-500" />
            <span className="text-2xl font-bold text-yellow-600">{stats.degraded_count}</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Unhealthy</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <XCircle className="h-5 w-5 text-red-500" />
            <span className="text-2xl font-bold text-red-600">{stats.unhealthy_count}</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Success Rate (24h)</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <TrendingUp className="h-5 w-5 text-primary" />
            <span className="text-2xl font-bold">{stats.overall_success_rate_24h.toFixed(1)}%</span>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Avg Sync Time</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Zap className="h-5 w-5 text-primary" />
            <span className="text-2xl font-bold">
              {stats.average_sync_duration_ms
                ? `${(stats.average_sync_duration_ms / 1000).toFixed(1)}s`
                : 'N/A'}
            </span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function HealthCard({ item }: { item: IntegrationHealthWithDetails }) {
  const config = healthStatusConfig[item.health.status as HealthStatus] || healthStatusConfig.unknown
  const { health } = item

  return (
    <Card className={`border-l-4 ${
      item.health.status === 'healthy' ? 'border-l-green-500' :
      item.health.status === 'degraded' ? 'border-l-yellow-500' :
      item.health.status === 'unhealthy' ? 'border-l-red-500' :
      'border-l-gray-300'
    }`}>
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div>
            <CardTitle className="text-lg">{item.integration_name}</CardTitle>
            <CardDescription className="capitalize">
              {item.integration_type.replace('_', ' ')}
            </CardDescription>
          </div>
          <Badge variant={config.variant} className="flex items-center gap-1">
            {config.icon}
            {config.label}
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {/* Success Rate Bar */}
          <div className="space-y-1">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Success Rate (24h)</span>
              <span className="font-medium">{item.success_rate_24h.toFixed(1)}%</span>
            </div>
            <Progress
              value={item.success_rate_24h}
              className={`h-2 ${
                item.success_rate_24h >= 95 ? '[&>div]:bg-green-500' :
                item.success_rate_24h >= 80 ? '[&>div]:bg-yellow-500' :
                '[&>div]:bg-red-500'
              }`}
            />
          </div>

          {/* Metrics Grid */}
          <div className="grid grid-cols-2 gap-3 text-sm">
            <div>
              <span className="text-muted-foreground">Last Success</span>
              <p className="font-medium">
                {health.last_successful_sync_at
                  ? formatRelativeTime(health.last_successful_sync_at)
                  : 'Never'}
              </p>
            </div>
            <div>
              <span className="text-muted-foreground">Syncs (24h)</span>
              <p className="font-medium">
                {health.sync_success_count_24h + health.sync_failure_count_24h}
              </p>
            </div>
            <div>
              <span className="text-muted-foreground">Failures (24h)</span>
              <p className={`font-medium ${health.sync_failure_count_24h > 0 ? 'text-red-600' : ''}`}>
                {health.sync_failure_count_24h}
              </p>
            </div>
            <div>
              <span className="text-muted-foreground">Consecutive Failures</span>
              <p className={`font-medium ${health.consecutive_failures >= 3 ? 'text-red-600' : ''}`}>
                {health.consecutive_failures}
              </p>
            </div>
          </div>

          {/* Error Message */}
          {health.last_error_message && (
            <div className="p-2 bg-destructive/10 rounded text-sm text-destructive">
              <p className="font-medium mb-1">Last Error:</p>
              <p className="text-xs break-words">{health.last_error_message}</p>
              {health.last_error_at && (
                <p className="text-xs mt-1 opacity-75">
                  {formatRelativeTime(health.last_error_at)}
                </p>
              )}
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

function RecentFailuresCard({ failures }: { failures: RecentFailure[] }) {
  if (failures.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <AlertTriangle className="h-5 w-5" />
            Recent Failures
          </CardTitle>
          <CardDescription>No recent failures</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
            <CheckCircle2 className="h-12 w-12 mb-2 text-green-500" />
            <p>All integrations are running smoothly</p>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg flex items-center gap-2">
          <AlertTriangle className="h-5 w-5 text-destructive" />
          Recent Failures
        </CardTitle>
        <CardDescription>
          {failures.length} integration{failures.length !== 1 ? 's' : ''} with recent errors
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {failures.map((failure) => (
            <div
              key={`${failure.integration_id}-${failure.failed_at}`}
              className="flex items-start gap-3 p-3 bg-destructive/5 rounded-lg"
            >
              <XCircle className="h-5 w-5 text-destructive mt-0.5 shrink-0" />
              <div className="flex-1 min-w-0">
                <div className="flex items-center justify-between gap-2">
                  <p className="font-medium truncate">{failure.integration_name}</p>
                  <Badge variant="destructive" className="shrink-0">
                    {failure.consecutive_failures}x
                  </Badge>
                </div>
                <p className="text-xs text-muted-foreground capitalize">
                  {failure.integration_type.replace('_', ' ')}
                </p>
                {failure.error_message && (
                  <p className="text-sm text-destructive mt-1 break-words">
                    {failure.error_message}
                  </p>
                )}
                <p className="text-xs text-muted-foreground mt-1">
                  {formatRelativeTime(failure.failed_at)}
                </p>
              </div>
            </div>
          ))}
        </div>
      </CardContent>
    </Card>
  )
}

function OverallHealthCard({ stats }: { stats: IntegrationHealthStats | null }) {
  if (!stats) return null

  const total = stats.total_integrations
  const healthPercentage = total > 0 ? (stats.healthy_count / total) * 100 : 100

  let overallStatus: HealthStatus = 'healthy'
  if (stats.unhealthy_count > 0) {
    overallStatus = 'unhealthy'
  } else if (stats.degraded_count > 0) {
    overallStatus = 'degraded'
  } else if (stats.unknown_count === total) {
    overallStatus = 'unknown'
  }

  const config = healthStatusConfig[overallStatus]

  return (
    <Card className={`border-2 ${
      overallStatus === 'healthy' ? 'border-green-500 bg-green-50 dark:bg-green-950/20' :
      overallStatus === 'degraded' ? 'border-yellow-500 bg-yellow-50 dark:bg-yellow-950/20' :
      overallStatus === 'unhealthy' ? 'border-red-500 bg-red-50 dark:bg-red-950/20' :
      'border-gray-300'
    }`}>
      <CardContent className="pt-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-4">
            <div className={`p-3 rounded-full ${
              overallStatus === 'healthy' ? 'bg-green-100 dark:bg-green-900' :
              overallStatus === 'degraded' ? 'bg-yellow-100 dark:bg-yellow-900' :
              overallStatus === 'unhealthy' ? 'bg-red-100 dark:bg-red-900' :
              'bg-gray-100 dark:bg-gray-800'
            }`}>
              <div className={`h-8 w-8 ${config.color}`}>
                {overallStatus === 'healthy' && <CheckCircle2 className="h-8 w-8" />}
                {overallStatus === 'degraded' && <AlertTriangle className="h-8 w-8" />}
                {overallStatus === 'unhealthy' && <XCircle className="h-8 w-8" />}
                {overallStatus === 'unknown' && <HelpCircle className="h-8 w-8" />}
              </div>
            </div>
            <div>
              <h3 className="text-2xl font-bold">{config.label}</h3>
              <p className="text-muted-foreground">
                {stats.healthy_count} of {total} integrations healthy
              </p>
            </div>
          </div>
          <div className="text-right">
            <p className="text-3xl font-bold">{healthPercentage.toFixed(0)}%</p>
            <p className="text-sm text-muted-foreground">Health Score</p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

export default function IntegrationHealthPage() {
  const { data: healthData, isLoading: healthLoading, refetch: refetchHealth } = useIntegrationHealth()
  const { data: statsData, isLoading: statsLoading, refetch: refetchStats } = useIntegrationHealthStats()
  const { data: failuresData, refetch: refetchFailures } = useIntegrationHealthFailures(10)

  const handleRefresh = () => {
    refetchHealth()
    refetchStats()
    refetchFailures()
  }

  const integrations = healthData?.data || []
  const stats = statsData?.data || null
  const failures = failuresData?.data || []

  const isLoading = healthLoading || statsLoading

  if (isLoading) {
    return <Loading message="Loading health status..." />
  }

  // Group integrations by status
  const unhealthyIntegrations = integrations.filter(i => i.health.status === 'unhealthy')
  const degradedIntegrations = integrations.filter(i => i.health.status === 'degraded')
  const healthyIntegrations = integrations.filter(i => i.health.status === 'healthy')
  const unknownIntegrations = integrations.filter(i => i.health.status === 'unknown')

  return (
    <div className="space-y-6">
      <PageHeader
        title="Integration Health"
        description="Monitor the health and performance of your connected integrations"
        action={
          <div className="flex gap-2">
            <Button variant="outline" asChild>
              <Link href="/integrations/">
                <ArrowLeft className="mr-2 h-4 w-4" />
                Back to Integrations
              </Link>
            </Button>
            <Button variant="outline" onClick={handleRefresh}>
              <RefreshCw className="mr-2 h-4 w-4" />
              Refresh
            </Button>
          </div>
        }
      />

      {/* Overall Health Status */}
      <OverallHealthCard stats={stats} />

      {/* Stats Cards */}
      <HealthStatsCards stats={stats} />

      {/* Main Content Grid */}
      <div className="grid lg:grid-cols-3 gap-6">
        {/* Integration Health Cards */}
        <div className="lg:col-span-2 space-y-4">
          {/* Unhealthy First */}
          {unhealthyIntegrations.length > 0 && (
            <div className="space-y-3">
              <h3 className="text-lg font-semibold flex items-center gap-2 text-destructive">
                <XCircle className="h-5 w-5" />
                Unhealthy ({unhealthyIntegrations.length})
              </h3>
              <div className="grid md:grid-cols-2 gap-4">
                {unhealthyIntegrations.map((item) => (
                  <HealthCard key={item.integration_id} item={item} />
                ))}
              </div>
            </div>
          )}

          {/* Degraded */}
          {degradedIntegrations.length > 0 && (
            <div className="space-y-3">
              <h3 className="text-lg font-semibold flex items-center gap-2 text-yellow-600">
                <AlertTriangle className="h-5 w-5" />
                Degraded ({degradedIntegrations.length})
              </h3>
              <div className="grid md:grid-cols-2 gap-4">
                {degradedIntegrations.map((item) => (
                  <HealthCard key={item.integration_id} item={item} />
                ))}
              </div>
            </div>
          )}

          {/* Healthy */}
          {healthyIntegrations.length > 0 && (
            <div className="space-y-3">
              <h3 className="text-lg font-semibold flex items-center gap-2 text-green-600">
                <CheckCircle2 className="h-5 w-5" />
                Healthy ({healthyIntegrations.length})
              </h3>
              <div className="grid md:grid-cols-2 gap-4">
                {healthyIntegrations.map((item) => (
                  <HealthCard key={item.integration_id} item={item} />
                ))}
              </div>
            </div>
          )}

          {/* Unknown */}
          {unknownIntegrations.length > 0 && (
            <div className="space-y-3">
              <h3 className="text-lg font-semibold flex items-center gap-2 text-muted-foreground">
                <HelpCircle className="h-5 w-5" />
                Not Yet Synced ({unknownIntegrations.length})
              </h3>
              <div className="grid md:grid-cols-2 gap-4">
                {unknownIntegrations.map((item) => (
                  <HealthCard key={item.integration_id} item={item} />
                ))}
              </div>
            </div>
          )}

          {/* Empty State */}
          {integrations.length === 0 && (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-12">
                <Activity className="h-12 w-12 text-muted-foreground mb-4" />
                <h3 className="text-lg font-medium mb-2">No integrations configured</h3>
                <p className="text-muted-foreground text-center mb-4">
                  Add integrations to monitor their health status
                </p>
                <Button asChild>
                  <Link href="/integrations/">Configure Integrations</Link>
                </Button>
              </CardContent>
            </Card>
          )}
        </div>

        {/* Sidebar with Recent Failures */}
        <div className="space-y-4">
          <RecentFailuresCard failures={failures} />

          {/* Quick Stats */}
          {stats && (
            <Card>
              <CardHeader>
                <CardTitle className="text-lg flex items-center gap-2">
                  <TrendingUp className="h-5 w-5" />
                  7-Day Summary
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="space-y-2">
                  <div className="flex justify-between text-sm">
                    <span className="text-muted-foreground">Success Rate</span>
                    <span className="font-medium">{stats.overall_success_rate_7d.toFixed(1)}%</span>
                  </div>
                  <Progress
                    value={stats.overall_success_rate_7d}
                    className="h-2"
                  />
                </div>
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <p className="text-muted-foreground">Total Syncs (24h)</p>
                    <p className="text-xl font-bold">{stats.total_syncs_24h}</p>
                  </div>
                  <div>
                    <p className="text-muted-foreground">Failures (24h)</p>
                    <p className={`text-xl font-bold ${stats.total_failures_24h > 0 ? 'text-destructive' : ''}`}>
                      {stats.total_failures_24h}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>
    </div>
  )
}
