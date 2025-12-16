'use client'

import { useState } from 'react'
import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Progress } from "@/components/ui/progress"
import { Loading } from "@/components/loading"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { useComplianceTrends, useComplianceSnapshot } from '@/hooks/use-api'
import {
  TrendingUp,
  TrendingDown,
  Minus,
  Calendar,
  Shield,
  Target,
  AlertTriangle,
  RefreshCcw
} from "lucide-react"
import { formatDate } from '@/types'
import { apiClient } from '@/lib/api-client'
import { useMutation } from '@/hooks/use-api'

function getTrendBadge(current: number, previous: number | null) {
  if (previous === null) return null
  const change = current - previous

  if (change > 0) {
    return (
      <Badge variant="secondary" className="bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300">
        <TrendingUp className="h-3 w-3 mr-1" />
        +{change.toFixed(1)}%
      </Badge>
    )
  } else if (change < 0) {
    return (
      <Badge variant="secondary" className="bg-red-100 text-red-700 dark:bg-red-900 dark:text-red-300">
        <TrendingDown className="h-3 w-3 mr-1" />
        {change.toFixed(1)}%
      </Badge>
    )
  }
  return (
    <Badge variant="secondary">
      <Minus className="h-3 w-3 mr-1" />
      No change
    </Badge>
  )
}

export default function ComplianceTrendsPage() {
  const [timeRange, setTimeRange] = useState('30')
  const { data: trends, isLoading, refetch } = useComplianceTrends(parseInt(timeRange))
  const { refetch: refetchSnapshot } = useComplianceSnapshot()

  const { mutate: captureSnapshot, isLoading: capturing } = useMutation(
    () => apiClient.post('/analytics/snapshots')
  )

  const handleCaptureSnapshot = async () => {
    try {
      await captureSnapshot({})
      refetch()
      refetchSnapshot()
    } catch (error) {
      console.error('Failed to capture snapshot:', error)
    }
  }

  if (isLoading) {
    return <Loading />
  }

  const current = trends?.data?.current
  const previous = trends?.data?.previous
  const trendPoints = trends?.data?.trend_points ?? []
  const changes = trends?.data?.changes

  return (
    <div className="space-y-6">
      <PageHeader
        title="Compliance Trends"
        description="Track your compliance posture over time and identify areas for improvement"
      >
        <div className="flex items-center gap-4">
          <Select value={timeRange} onValueChange={setTimeRange}>
            <SelectTrigger className="w-[180px]">
              <Calendar className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Select range" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="7">Last 7 days</SelectItem>
              <SelectItem value="30">Last 30 days</SelectItem>
              <SelectItem value="90">Last 90 days</SelectItem>
              <SelectItem value="180">Last 6 months</SelectItem>
              <SelectItem value="365">Last year</SelectItem>
            </SelectContent>
          </Select>
          <Button onClick={handleCaptureSnapshot} disabled={capturing}>
            <RefreshCcw className={`mr-2 h-4 w-4 ${capturing ? 'animate-spin' : ''}`} />
            Capture Snapshot
          </Button>
        </div>
      </PageHeader>

      {/* Current vs Previous Period Comparison */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Shield className="h-4 w-4 text-primary" />
              Control Implementation
            </CardTitle>
            <CardDescription>
              Current period vs previous
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-end justify-between">
              <div>
                <div className="text-3xl font-bold">
                  {current?.implementation_percentage?.toFixed(1) ?? 0}%
                </div>
                <p className="text-xs text-muted-foreground">
                  {current?.implemented_controls ?? 0} / {current?.total_controls ?? 0} controls
                </p>
              </div>
              {previous && getTrendBadge(
                current?.implementation_percentage ?? 0,
                previous.implementation_percentage
              )}
            </div>
            <Progress
              value={current?.implementation_percentage ?? 0}
              className="mt-4"
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <Target className="h-4 w-4 text-blue-500" />
              Framework Coverage
            </CardTitle>
            <CardDescription>
              Requirements addressed
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-end justify-between">
              <div>
                <div className="text-3xl font-bold">
                  {current?.coverage_percentage?.toFixed(1) ?? 0}%
                </div>
                <p className="text-xs text-muted-foreground">
                  {current?.covered_requirements ?? 0} / {current?.total_requirements ?? 0} requirements
                </p>
              </div>
              {previous && getTrendBadge(
                current?.coverage_percentage ?? 0,
                previous.coverage_percentage
              )}
            </div>
            <Progress
              value={current?.coverage_percentage ?? 0}
              className="mt-4"
            />
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 text-orange-500" />
              Risk Profile
            </CardTitle>
            <CardDescription>
              Average risk score
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-end justify-between">
              <div>
                <div className="text-3xl font-bold">
                  {current?.average_risk_score?.toFixed(1) ?? 'N/A'}
                </div>
                <p className="text-xs text-muted-foreground">
                  {current?.high_risks ?? 0} high risk items
                </p>
              </div>
              {previous && current && current.average_risk_score !== null && previous.average_risk_score !== null && (
                <Badge variant={
                  current.average_risk_score <= previous.average_risk_score
                    ? "secondary"
                    : "destructive"
                }>
                  {current.average_risk_score <= previous.average_risk_score ? (
                    <TrendingDown className="h-3 w-3 mr-1" />
                  ) : (
                    <TrendingUp className="h-3 w-3 mr-1" />
                  )}
                  {Math.abs(current.average_risk_score - previous.average_risk_score).toFixed(1)}
                </Badge>
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Detailed Metrics */}
      {current && (
        <div className="grid gap-4 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle>Control Status Breakdown</CardTitle>
              <CardDescription>
                Current control implementation status
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-green-600">Implemented</span>
                  <span className="font-medium">{current.implemented_controls}</span>
                </div>
                <Progress value={(current.implemented_controls / (current.total_controls || 1)) * 100} className="h-2 bg-muted" />
              </div>
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-yellow-600">In Progress</span>
                  <span className="font-medium">{current.in_progress_controls}</span>
                </div>
                <Progress value={(current.in_progress_controls / (current.total_controls || 1)) * 100} className="h-2 bg-muted" />
              </div>
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-red-600">Not Implemented</span>
                  <span className="font-medium">{current.not_implemented_controls}</span>
                </div>
                <Progress value={(current.not_implemented_controls / (current.total_controls || 1)) * 100} className="h-2 bg-muted" />
              </div>
              <div className="space-y-2">
                <div className="flex justify-between text-sm">
                  <span className="text-muted-foreground">Not Applicable</span>
                  <span className="font-medium">{current.not_applicable_controls}</span>
                </div>
                <Progress value={(current.not_applicable_controls / (current.total_controls || 1)) * 100} className="h-2 bg-muted" />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Risk Distribution</CardTitle>
              <CardDescription>
                Current risk levels across the organization
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-3 gap-4 text-center">
                <div className="p-4 rounded-lg bg-red-50 dark:bg-red-950">
                  <div className="text-2xl font-bold text-red-600">{current.high_risks}</div>
                  <div className="text-xs text-red-600">High Risk</div>
                </div>
                <div className="p-4 rounded-lg bg-yellow-50 dark:bg-yellow-950">
                  <div className="text-2xl font-bold text-yellow-600">{current.medium_risks}</div>
                  <div className="text-xs text-yellow-600">Medium Risk</div>
                </div>
                <div className="p-4 rounded-lg bg-green-50 dark:bg-green-950">
                  <div className="text-2xl font-bold text-green-600">{current.low_risks}</div>
                  <div className="text-xs text-green-600">Low Risk</div>
                </div>
              </div>

              <div className="pt-4 border-t">
                <h4 className="font-medium mb-3">Related Metrics</h4>
                <div className="space-y-2 text-sm">
                  <div className="flex justify-between">
                    <span>Total Vendors</span>
                    <Badge variant="outline">{current.total_vendors}</Badge>
                  </div>
                  <div className="flex justify-between">
                    <span>High Risk Vendors</span>
                    <Badge variant="destructive">{current.high_risk_vendors}</Badge>
                  </div>
                  <div className="flex justify-between">
                    <span>Overdue Tasks</span>
                    <Badge variant="warning" className="bg-yellow-500">{current.overdue_tasks}</Badge>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Trend History */}
      <Card>
        <CardHeader>
          <CardTitle>Historical Trend Data</CardTitle>
          <CardDescription>
            Daily snapshots over the selected time period
          </CardDescription>
        </CardHeader>
        <CardContent>
          {trendPoints.length === 0 ? (
            <div className="text-center py-8">
              <p className="text-muted-foreground">
                No historical data available. Capture snapshots regularly to build trend data.
              </p>
              <Button onClick={handleCaptureSnapshot} className="mt-4" disabled={capturing}>
                <RefreshCcw className={`mr-2 h-4 w-4 ${capturing ? 'animate-spin' : ''}`} />
                Capture First Snapshot
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Date</TableHead>
                  <TableHead className="text-right">Implementation %</TableHead>
                  <TableHead className="text-right">Coverage %</TableHead>
                  <TableHead className="text-right">Avg Risk Score</TableHead>
                  <TableHead className="text-right">High Risks</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {trendPoints.map((point, index) => {
                  const prevPoint = index < trendPoints.length - 1 ? trendPoints[index + 1] : null
                  return (
                    <TableRow key={point.date}>
                      <TableCell className="font-medium">
                        {formatDate(point.date)}
                      </TableCell>
                      <TableCell className="text-right">
                        <span className="mr-2">{point.implementation_percentage.toFixed(1)}%</span>
                        {prevPoint && (
                          point.implementation_percentage > prevPoint.implementation_percentage ? (
                            <TrendingUp className="h-3 w-3 text-green-500 inline" />
                          ) : point.implementation_percentage < prevPoint.implementation_percentage ? (
                            <TrendingDown className="h-3 w-3 text-red-500 inline" />
                          ) : null
                        )}
                      </TableCell>
                      <TableCell className="text-right">
                        <span className="mr-2">{point.coverage_percentage.toFixed(1)}%</span>
                        {prevPoint && (
                          point.coverage_percentage > prevPoint.coverage_percentage ? (
                            <TrendingUp className="h-3 w-3 text-green-500 inline" />
                          ) : point.coverage_percentage < prevPoint.coverage_percentage ? (
                            <TrendingDown className="h-3 w-3 text-red-500 inline" />
                          ) : null
                        )}
                      </TableCell>
                      <TableCell className="text-right">
                        {point.average_risk_score?.toFixed(1) ?? '-'}
                      </TableCell>
                      <TableCell className="text-right">
                        <Badge variant={point.high_risks > 5 ? "destructive" : "secondary"}>
                          {point.high_risks}
                        </Badge>
                      </TableCell>
                    </TableRow>
                  )
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Period Summary */}
      {changes && (
        <Card>
          <CardHeader>
            <CardTitle>Period Summary</CardTitle>
            <CardDescription>
              Changes over the last {timeRange} days
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid gap-4 md:grid-cols-3">
              <div className="p-4 border rounded-lg">
                <div className="text-sm text-muted-foreground">Implementation Change</div>
                <div className={`text-2xl font-bold ${changes.implementation_change >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                  {changes.implementation_change >= 0 ? '+' : ''}{changes.implementation_change.toFixed(1)}%
                </div>
              </div>
              <div className="p-4 border rounded-lg">
                <div className="text-sm text-muted-foreground">Coverage Change</div>
                <div className={`text-2xl font-bold ${changes.coverage_change >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                  {changes.coverage_change >= 0 ? '+' : ''}{changes.coverage_change.toFixed(1)}%
                </div>
              </div>
              <div className="p-4 border rounded-lg">
                <div className="text-sm text-muted-foreground">Risk Score Change</div>
                <div className={`text-2xl font-bold ${(changes.risk_change ?? 0) <= 0 ? 'text-green-600' : 'text-red-600'}`}>
                  {changes.risk_change !== null ? (
                    <>
                      {changes.risk_change >= 0 ? '+' : ''}{changes.risk_change.toFixed(1)}
                    </>
                  ) : (
                    'N/A'
                  )}
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
