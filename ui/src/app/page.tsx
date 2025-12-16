'use client'

import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Loading } from "@/components/loading"
import { RiskHeatmap } from "@/components/risk-heatmap"
import { GapAnalysisSummary } from "@/components/gap-analysis"
import {
  Shield,
  ShieldCheck,
  FileText,
  AlertTriangle,
  Building2,
  Server,
  ClipboardList,
  FileWarning,
  AlertCircle,
  CheckCircle2,
  Clock,
  ListTodo,
  User,
  Calendar,
  RefreshCw,
} from "lucide-react"
import Link from "next/link"
import {
  useControlStats,
  useEvidenceStats,
  usePolicyStats,
  useRiskStats,
  useVendorStats,
  useAssetStats,
  useAuditStats,
  useFrameworks,
  useRiskHeatmap,
  useGapAnalysis,
  useTaskStats,
  useOverdueTasks,
  useRecurringTasks,
} from '@/hooks/use-api'
import type { ControlStats, EvidenceStats, PolicyStats, RiskStats, VendorStats, AssetStats, AuditStats, Framework, TaskStats, Task } from '@/types'

function StatCard({
  title,
  value,
  description,
  icon: Icon,
  trend,
  trendUp,
}: {
  title: string
  value: string | number
  description: string
  icon: React.ElementType
  trend?: string
  trendUp?: boolean
}) {
  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
        <CardTitle className="text-sm font-medium">{title}</CardTitle>
        <Icon className="h-4 w-4 text-muted-foreground" />
      </CardHeader>
      <CardContent>
        <div className="text-2xl font-bold">{value}</div>
        <div className="flex items-center justify-between">
          <p className="text-xs text-muted-foreground">{description}</p>
          {trend && (
            <span className={`text-xs ${trendUp ? 'text-green-500' : 'text-muted-foreground'}`}>
              {trend}
            </span>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

function ControlsOverview({ stats }: { stats: ControlStats | null }) {
  if (!stats) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Shield className="h-5 w-5" />
          Controls Overview
        </CardTitle>
        <CardDescription>Implementation status of security controls</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">Implementation Progress</span>
            <span className="text-sm font-bold">{stats.implementation_percentage.toFixed(0)}%</span>
          </div>
          <div className="h-3 bg-secondary rounded-full overflow-hidden">
            <div
              className="h-full bg-primary transition-all"
              style={{ width: `${stats.implementation_percentage}%` }}
            />
          </div>
          <div className="grid grid-cols-2 gap-4 mt-4">
            <div className="flex items-center gap-2">
              <CheckCircle2 className="h-4 w-4 text-green-500" />
              <span className="text-sm">Implemented: {stats.implemented}</span>
            </div>
            <div className="flex items-center gap-2">
              <Clock className="h-4 w-4 text-yellow-500" />
              <span className="text-sm">In Progress: {stats.in_progress}</span>
            </div>
            <div className="flex items-center gap-2">
              <AlertCircle className="h-4 w-4 text-red-500" />
              <span className="text-sm">Not Implemented: {stats.not_implemented}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="h-4 w-4 rounded-full bg-gray-300" />
              <span className="text-sm">N/A: {stats.not_applicable}</span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

function RisksOverview({ stats }: { stats: RiskStats | null }) {
  if (!stats) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <AlertTriangle className="h-5 w-5" />
          Risk Overview
        </CardTitle>
        <CardDescription>Current risk posture by severity</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Badge variant="destructive">High</Badge>
              <span className="text-sm">{stats.high_risks} risks</span>
            </div>
            <div className="h-2 w-24 bg-secondary rounded-full overflow-hidden">
              <div
                className="h-full bg-red-500"
                style={{ width: `${stats.total > 0 ? (stats.high_risks / stats.total) * 100 : 0}%` }}
              />
            </div>
          </div>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Badge variant="warning">Medium</Badge>
              <span className="text-sm">{stats.medium_risks} risks</span>
            </div>
            <div className="h-2 w-24 bg-secondary rounded-full overflow-hidden">
              <div
                className="h-full bg-yellow-500"
                style={{ width: `${stats.total > 0 ? (stats.medium_risks / stats.total) * 100 : 0}%` }}
              />
            </div>
          </div>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Badge variant="success">Low</Badge>
              <span className="text-sm">{stats.low_risks} risks</span>
            </div>
            <div className="h-2 w-24 bg-secondary rounded-full overflow-hidden">
              <div
                className="h-full bg-green-500"
                style={{ width: `${stats.total > 0 ? (stats.low_risks / stats.total) * 100 : 0}%` }}
              />
            </div>
          </div>
          {stats.needs_review > 0 && (
            <div className="pt-2 border-t mt-4">
              <div className="flex items-center gap-2 text-yellow-600 dark:text-yellow-400">
                <FileWarning className="h-4 w-4" />
                <span className="text-sm font-medium">{stats.needs_review} risks need review</span>
              </div>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

function FrameworksList({ frameworks }: { frameworks: Framework[] | null }) {
  if (!frameworks || frameworks.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <ClipboardList className="h-5 w-5" />
            Compliance Frameworks
          </CardTitle>
          <CardDescription>Active compliance frameworks</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-8">
            <p className="text-sm text-muted-foreground">No frameworks loaded</p>
          </div>
        </CardContent>
      </Card>
    )
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <ClipboardList className="h-5 w-5" />
          Compliance Frameworks
        </CardTitle>
        <CardDescription>Active compliance frameworks</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          {frameworks.slice(0, 5).map((framework) => (
            <div key={framework.id} className="flex items-center justify-between">
              <div>
                <p className="text-sm font-medium">{framework.name}</p>
                {framework.version && (
                  <p className="text-xs text-muted-foreground">v{framework.version}</p>
                )}
              </div>
              {framework.is_system && (
                <Badge variant="secondary">System</Badge>
              )}
            </div>
          ))}
          {frameworks.length > 5 && (
            <p className="text-xs text-muted-foreground text-center pt-2">
              +{frameworks.length - 5} more frameworks
            </p>
          )}
        </div>
      </CardContent>
    </Card>
  )
}

function QuickStats({
  evidenceStats,
  policyStats,
  vendorStats,
  assetStats,
  auditStats,
}: {
  evidenceStats: EvidenceStats | null
  policyStats: PolicyStats | null
  vendorStats: VendorStats | null
  assetStats: AssetStats | null
  auditStats: AuditStats | null
}) {
  return (
    <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
      <StatCard
        title="Evidence Items"
        value={evidenceStats?.total ?? 0}
        description={`${evidenceStats?.expiring_soon ?? 0} expiring soon`}
        icon={FileText}
        trend={evidenceStats?.expired ? `${evidenceStats.expired} expired` : undefined}
      />
      <StatCard
        title="Policies"
        value={policyStats?.published ?? 0}
        description={`${policyStats?.total ?? 0} total policies`}
        icon={FileText}
        trend={policyStats?.needs_review ? `${policyStats.needs_review} need review` : undefined}
      />
      <StatCard
        title="Vendors"
        value={vendorStats?.active ?? 0}
        description={`${vendorStats?.total ?? 0} total vendors`}
        icon={Building2}
        trend={vendorStats?.under_review ? `${vendorStats.under_review} under review` : undefined}
      />
      <StatCard
        title="Assets"
        value={assetStats?.total ?? 0}
        description={assetStats?.from_integrations ? `${assetStats.from_integrations} from integrations` : 'Across all systems'}
        icon={Server}
      />
    </div>
  )
}

function AuditsOverview({ stats }: { stats: AuditStats | null }) {
  if (!stats) return null

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <ClipboardList className="h-5 w-5" />
          Audit Status
        </CardTitle>
        <CardDescription>Current audit activities</CardDescription>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-2 gap-4">
          <div>
            <p className="text-2xl font-bold">{stats.active}</p>
            <p className="text-xs text-muted-foreground">Active Audits</p>
          </div>
          <div>
            <p className="text-2xl font-bold">{stats.completed}</p>
            <p className="text-xs text-muted-foreground">Completed</p>
          </div>
          <div>
            <p className="text-2xl font-bold text-yellow-500">{stats.open_requests}</p>
            <p className="text-xs text-muted-foreground">Open Requests</p>
          </div>
          <div>
            <p className="text-2xl font-bold text-red-500">{stats.open_findings}</p>
            <p className="text-xs text-muted-foreground">Open Findings</p>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

function TaskWorkloadOverview({
  stats,
  overdueTasks,
  recurringTasks,
}: {
  stats: TaskStats | null
  overdueTasks: Task[] | null
  recurringTasks: Task[] | null
}) {
  if (!stats) return null

  const completionRate = stats.total > 0
    ? Math.round((stats.completed / stats.total) * 100)
    : 0

  return (
    <Card className="col-span-2">
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <ListTodo className="h-5 w-5" />
              Task Workload
            </CardTitle>
            <CardDescription>Team task distribution and progress</CardDescription>
          </div>
          <Link href="/tasks" className="text-sm text-primary hover:underline">
            View all tasks
          </Link>
        </div>
      </CardHeader>
      <CardContent>
        <div className="grid gap-6 md:grid-cols-3">
          {/* Task Status Summary */}
          <div className="space-y-4">
            <h4 className="text-sm font-medium">Status Overview</h4>
            <div className="grid grid-cols-2 gap-3">
              <div className="flex items-center gap-2">
                <div className="h-3 w-3 rounded-full bg-blue-500" />
                <span className="text-sm">{stats.open} Open</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="h-3 w-3 rounded-full bg-yellow-500" />
                <span className="text-sm">{stats.in_progress} In Progress</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="h-3 w-3 rounded-full bg-green-500" />
                <span className="text-sm">{stats.completed} Completed</span>
              </div>
              <div className="flex items-center gap-2">
                <div className="h-3 w-3 rounded-full bg-red-500" />
                <span className="text-sm">{stats.overdue} Overdue</span>
              </div>
            </div>
            <div className="pt-2">
              <div className="flex items-center justify-between text-sm mb-1">
                <span className="text-muted-foreground">Completion Rate</span>
                <span className="font-medium">{completionRate}%</span>
              </div>
              <div className="h-2 bg-secondary rounded-full overflow-hidden">
                <div
                  className="h-full bg-green-500 transition-all"
                  style={{ width: `${completionRate}%` }}
                />
              </div>
            </div>
          </div>

          {/* Upcoming Tasks */}
          <div className="space-y-4">
            <h4 className="text-sm font-medium flex items-center gap-2">
              <Calendar className="h-4 w-4" />
              Upcoming
            </h4>
            <div className="space-y-2">
              <div className="flex items-center justify-between p-2 rounded-lg bg-muted/50">
                <span className="text-sm">Due Today</span>
                <Badge variant={stats.due_today > 0 ? "warning" : "secondary"}>
                  {stats.due_today}
                </Badge>
              </div>
              <div className="flex items-center justify-between p-2 rounded-lg bg-muted/50">
                <span className="text-sm">Due This Week</span>
                <Badge variant="secondary">{stats.due_this_week}</Badge>
              </div>
              {recurringTasks && recurringTasks.length > 0 && (
                <div className="flex items-center justify-between p-2 rounded-lg bg-muted/50">
                  <span className="text-sm flex items-center gap-1">
                    <RefreshCw className="h-3 w-3" />
                    Recurring Tasks
                  </span>
                  <Badge variant="outline">{recurringTasks.length}</Badge>
                </div>
              )}
            </div>
          </div>

          {/* Top Assignees */}
          <div className="space-y-4">
            <h4 className="text-sm font-medium flex items-center gap-2">
              <User className="h-4 w-4" />
              Workload by Assignee
            </h4>
            <div className="space-y-2">
              {stats.by_assignee && stats.by_assignee.length > 0 ? (
                stats.by_assignee.slice(0, 5).map((assignee, index) => (
                  <div key={index} className="flex items-center justify-between">
                    <span className="text-sm truncate max-w-[150px]">
                      {assignee.assignee_name || 'Unassigned'}
                    </span>
                    <Badge variant="secondary">{assignee.count}</Badge>
                  </div>
                ))
              ) : (
                <p className="text-sm text-muted-foreground">No tasks assigned</p>
              )}
            </div>
          </div>
        </div>

        {/* Overdue Tasks Alert */}
        {overdueTasks && overdueTasks.length > 0 && (
          <div className="mt-4 pt-4 border-t">
            <div className="flex items-center gap-2 text-red-500 dark:text-red-400 mb-2">
              <AlertCircle className="h-4 w-4" />
              <span className="text-sm font-medium">
                {overdueTasks.length} overdue task{overdueTasks.length > 1 ? 's' : ''} require attention
              </span>
            </div>
            <div className="flex flex-wrap gap-2">
              {overdueTasks.slice(0, 3).map((task) => (
                <Link key={task.id} href={`/tasks?id=${task.id}`}>
                  <Badge variant="destructive" className="cursor-pointer hover:bg-red-600">
                    {task.title.length > 30 ? task.title.slice(0, 30) + '...' : task.title}
                  </Badge>
                </Link>
              ))}
              {overdueTasks.length > 3 && (
                <Link href="/tasks?overdue_only=true">
                  <Badge variant="outline" className="cursor-pointer">
                    +{overdueTasks.length - 3} more
                  </Badge>
                </Link>
              )}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

export default function DashboardPage() {
  const { data: controlStats, isLoading: controlsLoading } = useControlStats()
  const { data: evidenceStats, isLoading: evidenceLoading } = useEvidenceStats()
  const { data: policyStats, isLoading: policiesLoading } = usePolicyStats()
  const { data: riskStats, isLoading: risksLoading } = useRiskStats()
  const { data: vendorStats, isLoading: vendorsLoading } = useVendorStats()
  const { data: assetStats, isLoading: assetsLoading } = useAssetStats()
  const { data: auditStats, isLoading: auditsLoading } = useAuditStats()
  const { data: frameworks, isLoading: frameworksLoading } = useFrameworks()
  const { data: heatmapData } = useRiskHeatmap()
  const { data: taskStats, isLoading: tasksLoading } = useTaskStats()
  const { data: overdueTasks } = useOverdueTasks()
  const { data: recurringTasks } = useRecurringTasks()

  // Get gap analysis for the first framework with requirements
  const firstFrameworkId = frameworks?.[0]?.id || ''
  const { data: gapAnalysis } = useGapAnalysis(firstFrameworkId)

  const isLoading = controlsLoading || evidenceLoading || policiesLoading ||
    risksLoading || vendorsLoading || assetsLoading || auditsLoading || frameworksLoading || tasksLoading

  if (isLoading) {
    return <Loading />
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Dashboard"
        description="Overview of your compliance posture"
      />

      {/* Main Stats */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-5">
        <StatCard
          title="Total Controls"
          value={controlStats?.total ?? 0}
          description={`${controlStats?.implemented ?? 0} implemented`}
          icon={ShieldCheck}
          trend={controlStats ? `${controlStats.implementation_percentage.toFixed(0)}%` : undefined}
          trendUp={(controlStats?.implementation_percentage ?? 0) > 50}
        />
        <StatCard
          title="Total Risks"
          value={riskStats?.total ?? 0}
          description={`${riskStats?.high_risks ?? 0} high severity`}
          icon={AlertTriangle}
        />
        <StatCard
          title="Frameworks"
          value={frameworks?.length ?? 0}
          description="Compliance frameworks"
          icon={Shield}
        />
        <StatCard
          title="Active Audits"
          value={auditStats?.active ?? 0}
          description={`${auditStats?.open_requests ?? 0} open requests`}
          icon={ClipboardList}
        />
        <StatCard
          title="Open Tasks"
          value={(taskStats?.open ?? 0) + (taskStats?.in_progress ?? 0)}
          description={`${taskStats?.overdue ?? 0} overdue`}
          icon={ListTodo}
          trend={taskStats?.due_today ? `${taskStats.due_today} due today` : undefined}
        />
      </div>

      {/* Secondary Stats */}
      <QuickStats
        evidenceStats={evidenceStats}
        policyStats={policyStats}
        vendorStats={vendorStats}
        assetStats={assetStats}
        auditStats={auditStats}
      />

      {/* Detail Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <ControlsOverview stats={controlStats} />
        <RisksOverview stats={riskStats} />
        <FrameworksList frameworks={frameworks} />
      </div>

      {/* Risk Heatmap & Gap Analysis */}
      <div className="grid gap-4 md:grid-cols-2">
        <RiskHeatmap data={heatmapData} />
        {gapAnalysis ? (
          <GapAnalysisSummary
            frameworks={[
              {
                id: gapAnalysis.framework_id,
                name: gapAnalysis.framework_name,
                coverage: gapAnalysis.coverage_percentage,
                total: gapAnalysis.total_requirements,
                covered: gapAnalysis.covered_requirements,
              },
            ]}
          />
        ) : (
          <GapAnalysisSummary frameworks={[]} />
        )}
      </div>

      {/* Task Workload */}
      {taskStats && taskStats.total > 0 && (
        <div className="grid gap-4 md:grid-cols-2">
          <TaskWorkloadOverview
            stats={taskStats}
            overdueTasks={overdueTasks ?? null}
            recurringTasks={recurringTasks ?? null}
          />
        </div>
      )}

      {/* Audit Overview */}
      {auditStats && auditStats.total > 0 && (
        <div className="grid gap-4 md:grid-cols-2">
          <AuditsOverview stats={auditStats} />
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center gap-2">
                <ClipboardList className="h-5 w-5" />
                Recent Activity
              </CardTitle>
              <CardDescription>Latest compliance updates</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex h-48 items-center justify-center rounded-lg border border-dashed">
                <p className="text-sm text-muted-foreground">
                  Activity feed coming soon
                </p>
              </div>
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  )
}
