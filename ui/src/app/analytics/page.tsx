'use client'

import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Progress } from "@/components/ui/progress"
import { Loading } from "@/components/loading"
import {
  useExecutiveDashboard,
  useComplianceTrends
} from '@/hooks/use-api'
import {
  TrendingUp,
  TrendingDown,
  Minus,
  Shield,
  AlertTriangle,
  CheckCircle,
  Clock,
  Target,
  BarChart3,
  ArrowRight
} from "lucide-react"
import Link from "next/link"
import { Button } from "@/components/ui/button"

function getTrendIcon(direction: string | null) {
  switch (direction) {
    case 'up':
      return <TrendingUp className="h-4 w-4 text-green-500" />
    case 'down':
      return <TrendingDown className="h-4 w-4 text-red-500" />
    default:
      return <Minus className="h-4 w-4 text-muted-foreground" />
  }
}

function getMetricStatusColor(value: number, warning: number | null, critical: number | null): string {
  if (critical !== null && value <= critical) return 'text-red-600'
  if (warning !== null && value <= warning) return 'text-yellow-600'
  return 'text-green-600'
}

export default function ExecutiveDashboardPage() {
  const { data: dashboard, isLoading: dashboardLoading } = useExecutiveDashboard()
  const { data: trends, isLoading: trendsLoading } = useComplianceTrends(30)

  const isLoading = dashboardLoading || trendsLoading
  const snapshot = dashboard?.data?.snapshot
  const trendData = trends?.data

  if (isLoading) {
    return <Loading />
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Executive Dashboard"
        description="High-level compliance and risk overview for executive leadership"
      >
        <Link href="/analytics/reports/">
          <Button variant="outline">
            <BarChart3 className="mr-2 h-4 w-4" />
            Generate Report
          </Button>
        </Link>
      </PageHeader>

      {/* Key Metrics Overview */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Shield className="h-4 w-4 text-primary" />
              Control Implementation
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {snapshot?.implementation_percentage?.toFixed(1) ?? 0}%
            </div>
            <Progress
              value={snapshot?.implementation_percentage ?? 0}
              className="mt-2"
            />
            <p className="text-xs text-muted-foreground mt-2">
              {snapshot?.implemented_controls ?? 0} of {snapshot?.total_controls ?? 0} controls
            </p>
            {trendData?.changes && (
              <div className="flex items-center gap-1 mt-1">
                {trendData.changes.implementation_change >= 0 ? (
                  <TrendingUp className="h-3 w-3 text-green-500" />
                ) : (
                  <TrendingDown className="h-3 w-3 text-red-500" />
                )}
                <span className={`text-xs ${trendData.changes.implementation_change >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                  {trendData.changes.implementation_change >= 0 ? '+' : ''}{trendData.changes.implementation_change.toFixed(1)}% this month
                </span>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Target className="h-4 w-4 text-blue-500" />
              Framework Coverage
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {snapshot?.coverage_percentage?.toFixed(1) ?? 0}%
            </div>
            <Progress
              value={snapshot?.coverage_percentage ?? 0}
              className="mt-2"
            />
            <p className="text-xs text-muted-foreground mt-2">
              {snapshot?.covered_requirements ?? 0} of {snapshot?.total_requirements ?? 0} requirements
            </p>
            {trendData?.changes && (
              <div className="flex items-center gap-1 mt-1">
                {trendData.changes.coverage_change >= 0 ? (
                  <TrendingUp className="h-3 w-3 text-green-500" />
                ) : (
                  <TrendingDown className="h-3 w-3 text-red-500" />
                )}
                <span className={`text-xs ${trendData.changes.coverage_change >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                  {trendData.changes.coverage_change >= 0 ? '+' : ''}{trendData.changes.coverage_change.toFixed(1)}% this month
                </span>
              </div>
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 text-orange-500" />
              Risk Summary
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {snapshot?.total_risks ?? 0}
            </div>
            <div className="flex gap-2 mt-2">
              <Badge variant="destructive" className="text-xs">
                {snapshot?.high_risks ?? 0} High
              </Badge>
              <Badge variant="warning" className="text-xs bg-yellow-500">
                {snapshot?.medium_risks ?? 0} Medium
              </Badge>
              <Badge variant="secondary" className="text-xs">
                {snapshot?.low_risks ?? 0} Low
              </Badge>
            </div>
            <p className="text-xs text-muted-foreground mt-2">
              Avg Score: {snapshot?.average_risk_score?.toFixed(1) ?? 'N/A'}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Clock className="h-4 w-4 text-purple-500" />
              Task Status
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600">
              {snapshot?.overdue_tasks ?? 0}
            </div>
            <p className="text-xs text-muted-foreground mt-2">
              Overdue tasks requiring attention
            </p>
            <div className="flex gap-2 mt-2">
              <Badge variant="outline" className="text-xs">
                {snapshot?.total_policies ?? 0} Policies
              </Badge>
              <Badge variant="outline" className="text-xs">
                {snapshot?.total_evidence ?? 0} Evidence
              </Badge>
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Quick Actions & Status Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <CheckCircle className="h-5 w-5 text-green-500" />
              Policy Status
            </CardTitle>
            <CardDescription>
              Document approval and publication status
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-sm">Published</span>
              <Badge variant="secondary">{snapshot?.published_policies ?? 0}</Badge>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm">Pending Approval</span>
              <Badge variant="warning" className="bg-yellow-500">{snapshot?.pending_policies ?? 0}</Badge>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm">Total</span>
              <Badge>{snapshot?.total_policies ?? 0}</Badge>
            </div>
            <Link href="/policies/" className="block pt-2">
              <Button variant="outline" size="sm" className="w-full">
                View Policies <ArrowRight className="ml-2 h-4 w-4" />
              </Button>
            </Link>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-orange-500" />
              Evidence Health
            </CardTitle>
            <CardDescription>
              Evidence collection and expiration status
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-sm">Total Evidence</span>
              <Badge variant="secondary">{snapshot?.total_evidence ?? 0}</Badge>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm text-yellow-600">Expiring Soon</span>
              <Badge variant="warning" className="bg-yellow-500">{snapshot?.expiring_evidence ?? 0}</Badge>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm text-red-600">Expired</span>
              <Badge variant="destructive">{snapshot?.expired_evidence ?? 0}</Badge>
            </div>
            <Link href="/evidence/" className="block pt-2">
              <Button variant="outline" size="sm" className="w-full">
                Manage Evidence <ArrowRight className="ml-2 h-4 w-4" />
              </Button>
            </Link>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <Target className="h-5 w-5 text-blue-500" />
              Vendor Risk
            </CardTitle>
            <CardDescription>
              Third-party risk management status
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-3">
            <div className="flex justify-between items-center">
              <span className="text-sm">Total Vendors</span>
              <Badge variant="secondary">{snapshot?.total_vendors ?? 0}</Badge>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm text-red-600">High Risk</span>
              <Badge variant="destructive">{snapshot?.high_risk_vendors ?? 0}</Badge>
            </div>
            <div className="flex justify-between items-center">
              <span className="text-sm">Active Frameworks</span>
              <Badge>{snapshot?.total_frameworks ?? 0}</Badge>
            </div>
            <Link href="/vendors/" className="block pt-2">
              <Button variant="outline" size="sm" className="w-full">
                View Vendors <ArrowRight className="ml-2 h-4 w-4" />
              </Button>
            </Link>
          </CardContent>
        </Card>
      </div>

      {/* Analytics Quick Links */}
      <Card>
        <CardHeader>
          <CardTitle>Deep Dive Analytics</CardTitle>
          <CardDescription>
            Explore detailed analytics and insights
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
            <Link href="/analytics/trends/" className="block">
              <Card className="hover:bg-accent transition-colors cursor-pointer">
                <CardContent className="pt-6">
                  <div className="flex items-center gap-3">
                    <TrendingUp className="h-8 w-8 text-primary" />
                    <div>
                      <p className="font-medium">Compliance Trends</p>
                      <p className="text-sm text-muted-foreground">Historical progress</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </Link>

            <Link href="/analytics/predictions/" className="block">
              <Card className="hover:bg-accent transition-colors cursor-pointer">
                <CardContent className="pt-6">
                  <div className="flex items-center gap-3">
                    <Target className="h-8 w-8 text-orange-500" />
                    <div>
                      <p className="font-medium">Risk Predictions</p>
                      <p className="text-sm text-muted-foreground">ML-powered forecasts</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </Link>

            <Link href="/analytics/benchmarks/" className="block">
              <Card className="hover:bg-accent transition-colors cursor-pointer">
                <CardContent className="pt-6">
                  <div className="flex items-center gap-3">
                    <BarChart3 className="h-8 w-8 text-blue-500" />
                    <div>
                      <p className="font-medium">Benchmarks</p>
                      <p className="text-sm text-muted-foreground">Industry comparison</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </Link>

            <Link href="/analytics/reports/" className="block">
              <Card className="hover:bg-accent transition-colors cursor-pointer">
                <CardContent className="pt-6">
                  <div className="flex items-center gap-3">
                    <Shield className="h-8 w-8 text-green-500" />
                    <div>
                      <p className="font-medium">Report Builder</p>
                      <p className="text-sm text-muted-foreground">Custom reports</p>
                    </div>
                  </div>
                </CardContent>
              </Card>
            </Link>
          </div>
        </CardContent>
      </Card>

      {/* Executive Metrics */}
      {dashboard?.data?.metrics && dashboard.data.metrics.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Key Performance Indicators</CardTitle>
            <CardDescription>
              Tracked metrics and their current status
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {dashboard.data.metrics.map((metric) => (
                <div key={metric.id} className="p-4 border rounded-lg">
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">{metric.metric_name}</span>
                    {getTrendIcon(metric.trend_direction)}
                  </div>
                  <div className={`text-2xl font-bold mt-1 ${getMetricStatusColor(metric.metric_value, metric.threshold_warning, metric.threshold_critical)}`}>
                    {metric.metric_value.toFixed(1)}{metric.metric_unit ?? ''}
                  </div>
                  {metric.target_value && (
                    <div className="text-xs text-muted-foreground mt-1">
                      Target: {metric.target_value}{metric.metric_unit ?? ''}
                    </div>
                  )}
                  {metric.trend_percentage && (
                    <div className={`text-xs mt-1 ${metric.trend_percentage >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                      {metric.trend_percentage >= 0 ? '+' : ''}{metric.trend_percentage.toFixed(1)}% change
                    </div>
                  )}
                </div>
              ))}
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
