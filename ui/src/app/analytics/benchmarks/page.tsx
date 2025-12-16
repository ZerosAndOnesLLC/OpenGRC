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
import {
  useAvailableBenchmarks,
  useBenchmarkComparison,
  useLatestBenchmarkComparison
} from '@/hooks/use-api'
import {
  Scale,
  TrendingUp,
  TrendingDown,
  Trophy,
  AlertTriangle,
  Building,
  CheckCircle,
  XCircle,
  BarChart3,
  RefreshCcw
} from "lucide-react"
import { formatDate } from '@/types'
import { apiClient } from '@/lib/api-client'
import { useMutation } from '@/hooks/use-api'

function getPercentileBadge(percentile: number | null) {
  if (percentile === null) return null

  if (percentile >= 75) {
    return (
      <Badge className="bg-green-500">
        <Trophy className="h-3 w-3 mr-1" />
        Top {100 - percentile}%
      </Badge>
    )
  } else if (percentile >= 50) {
    return (
      <Badge className="bg-blue-500">
        Above Average
      </Badge>
    )
  } else if (percentile >= 25) {
    return (
      <Badge variant="warning" className="bg-yellow-500">
        Below Average
      </Badge>
    )
  }
  return (
    <Badge variant="destructive">
      Needs Improvement
    </Badge>
  )
}

function getComparisonIcon(orgValue: number, benchmarkValue: number) {
  if (orgValue >= benchmarkValue) {
    return <TrendingUp className="h-4 w-4 text-green-500" />
  }
  return <TrendingDown className="h-4 w-4 text-red-500" />
}

export default function BenchmarksPage() {
  const [selectedBenchmark, setSelectedBenchmark] = useState<string>('')
  const { data: benchmarks, isLoading: benchmarksLoading } = useAvailableBenchmarks()
  const { data: latestComparison, isLoading: latestLoading, refetch: refetchLatest } = useLatestBenchmarkComparison()
  const { data: selectedComparison, isLoading: comparisonLoading } = useBenchmarkComparison(selectedBenchmark)

  const isLoading = benchmarksLoading || latestLoading

  const { mutate: runComparison, isLoading: comparing } = useMutation(
    (benchmarkId: string) => apiClient.post(`/analytics/benchmarks/${benchmarkId}/compare`)
  )

  const handleRunComparison = async (benchmarkId: string) => {
    try {
      await runComparison(benchmarkId)
      refetchLatest()
    } catch (error) {
      console.error('Failed to run comparison:', error)
    }
  }

  if (isLoading) {
    return <Loading />
  }

  const benchmarkList = benchmarks?.data ?? []
  const comparison = selectedBenchmark ? selectedComparison?.data : latestComparison?.data

  return (
    <div className="space-y-6">
      <PageHeader
        title="Industry Benchmarks"
        description="Compare your compliance posture against industry standards and peers"
      >
        <Select value={selectedBenchmark} onValueChange={setSelectedBenchmark}>
          <SelectTrigger className="w-[250px]">
            <Building className="mr-2 h-4 w-4" />
            <SelectValue placeholder="Select industry benchmark" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="">Latest Comparison</SelectItem>
            {benchmarkList.map((benchmark) => (
              <SelectItem key={benchmark.id} value={benchmark.id}>
                {benchmark.industry_name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </PageHeader>

      {/* Available Benchmarks */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Scale className="h-5 w-5 text-primary" />
            Available Industry Benchmarks
          </CardTitle>
          <CardDescription>
            Select an industry to compare your organization&apos;s performance
          </CardDescription>
        </CardHeader>
        <CardContent>
          {benchmarkList.length === 0 ? (
            <div className="text-center py-8">
              <BarChart3 className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
              <p className="text-muted-foreground">
                No benchmark data available. Industry benchmarks will be populated as more data is collected.
              </p>
            </div>
          ) : (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {benchmarkList.map((benchmark) => (
                <Card
                  key={benchmark.id}
                  className={`cursor-pointer transition-all hover:border-primary ${selectedBenchmark === benchmark.id ? 'border-primary' : ''}`}
                  onClick={() => setSelectedBenchmark(benchmark.id)}
                >
                  <CardContent className="pt-6">
                    <div className="flex items-start justify-between mb-4">
                      <div>
                        <h3 className="font-medium">{benchmark.industry_name}</h3>
                        {benchmark.industry_code && (
                          <p className="text-xs text-muted-foreground">{benchmark.industry_code}</p>
                        )}
                      </div>
                      <Badge variant="outline">{benchmark.sample_size} orgs</Badge>
                    </div>

                    <div className="space-y-3">
                      <div>
                        <div className="flex justify-between text-sm mb-1">
                          <span>Avg Implementation</span>
                          <span className="font-medium">{benchmark.avg_implementation_rate.toFixed(1)}%</span>
                        </div>
                        <Progress value={benchmark.avg_implementation_rate} className="h-2" />
                      </div>

                      <div>
                        <div className="flex justify-between text-sm mb-1">
                          <span>Avg Coverage</span>
                          <span className="font-medium">{benchmark.avg_coverage_rate.toFixed(1)}%</span>
                        </div>
                        <Progress value={benchmark.avg_coverage_rate} className="h-2" />
                      </div>

                      <div className="flex justify-between text-sm pt-2 border-t">
                        <span>Top Quartile</span>
                        <span className="font-medium text-green-600">{benchmark.top_quartile_rate.toFixed(1)}%</span>
                      </div>
                    </div>

                    <div className="mt-4">
                      <Button
                        variant="outline"
                        size="sm"
                        className="w-full"
                        onClick={(e) => {
                          e.stopPropagation()
                          handleRunComparison(benchmark.id)
                        }}
                        disabled={comparing}
                      >
                        <RefreshCcw className={`mr-2 h-3 w-3 ${comparing ? 'animate-spin' : ''}`} />
                        Compare Now
                      </Button>
                    </div>

                    <p className="text-xs text-muted-foreground mt-2 text-center">
                      Data: {formatDate(benchmark.data_period_start)} - {formatDate(benchmark.data_period_end)}
                    </p>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Comparison Results */}
      {comparison && (
        <>
          <div className="grid gap-4 md:grid-cols-3">
            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Implementation Rate</CardTitle>
                <CardDescription>Your rate vs industry average</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="flex items-end justify-between">
                  <div>
                    <div className="text-3xl font-bold">
                      {comparison.org_implementation_rate.toFixed(1)}%
                    </div>
                    <p className="text-xs text-muted-foreground">
                      Industry avg: {comparison.avg_implementation_rate.toFixed(1)}%
                    </p>
                  </div>
                  <div className="flex flex-col items-end gap-2">
                    {getComparisonIcon(comparison.org_implementation_rate, comparison.avg_implementation_rate)}
                    {getPercentileBadge(comparison.implementation_percentile)}
                  </div>
                </div>
                <div className="mt-4 space-y-2">
                  <div className="flex justify-between text-sm">
                    <span>You</span>
                    <span className="font-medium">{comparison.org_implementation_rate.toFixed(1)}%</span>
                  </div>
                  <Progress value={comparison.org_implementation_rate} className="h-2 bg-muted" />
                  <div className="flex justify-between text-sm">
                    <span>Industry Avg</span>
                    <span className="font-medium">{comparison.avg_implementation_rate.toFixed(1)}%</span>
                  </div>
                  <Progress value={comparison.avg_implementation_rate} className="h-2 bg-muted" />
                  <div className="flex justify-between text-sm">
                    <span className="text-green-600">Top Quartile</span>
                    <span className="font-medium text-green-600">{comparison.top_quartile_rate.toFixed(1)}%</span>
                  </div>
                  <Progress value={comparison.top_quartile_rate} className="h-2 bg-green-100" />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Framework Coverage</CardTitle>
                <CardDescription>Your coverage vs industry average</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="flex items-end justify-between">
                  <div>
                    <div className="text-3xl font-bold">
                      {comparison.org_coverage_rate.toFixed(1)}%
                    </div>
                    <p className="text-xs text-muted-foreground">
                      Industry avg: {comparison.avg_coverage_rate.toFixed(1)}%
                    </p>
                  </div>
                  <div className="flex flex-col items-end gap-2">
                    {getComparisonIcon(comparison.org_coverage_rate, comparison.avg_coverage_rate)}
                    {getPercentileBadge(comparison.coverage_percentile)}
                  </div>
                </div>
                <div className="mt-4 space-y-2">
                  <div className="flex justify-between text-sm">
                    <span>You</span>
                    <span className="font-medium">{comparison.org_coverage_rate.toFixed(1)}%</span>
                  </div>
                  <Progress value={comparison.org_coverage_rate} className="h-2 bg-muted" />
                  <div className="flex justify-between text-sm">
                    <span>Industry Avg</span>
                    <span className="font-medium">{comparison.avg_coverage_rate.toFixed(1)}%</span>
                  </div>
                  <Progress value={comparison.avg_coverage_rate} className="h-2 bg-muted" />
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">Risk Score</CardTitle>
                <CardDescription>Lower is better</CardDescription>
              </CardHeader>
              <CardContent>
                <div className="flex items-end justify-between">
                  <div>
                    <div className="text-3xl font-bold">
                      {comparison.org_risk_score?.toFixed(1) ?? 'N/A'}
                    </div>
                    <p className="text-xs text-muted-foreground">
                      Industry avg: {comparison.avg_risk_score?.toFixed(1) ?? 'N/A'}
                    </p>
                  </div>
                  <div className="flex flex-col items-end gap-2">
                    {comparison.org_risk_score !== null && comparison.avg_risk_score !== null && (
                      <>
                        {comparison.org_risk_score <= comparison.avg_risk_score ? (
                          <TrendingDown className="h-4 w-4 text-green-500" />
                        ) : (
                          <TrendingUp className="h-4 w-4 text-red-500" />
                        )}
                      </>
                    )}
                    {getPercentileBadge(comparison.risk_percentile)}
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Strengths & Improvement Areas */}
          <div className="grid gap-4 md:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-green-600">
                  <CheckCircle className="h-5 w-5" />
                  Strengths
                </CardTitle>
                <CardDescription>
                  Areas where you outperform industry benchmarks
                </CardDescription>
              </CardHeader>
              <CardContent>
                {comparison.strengths && comparison.strengths.length > 0 ? (
                  <ul className="space-y-2">
                    {comparison.strengths.map((strength, idx) => (
                      <li key={idx} className="flex items-start gap-2 p-2 rounded-lg bg-green-50 dark:bg-green-950">
                        <CheckCircle className="h-4 w-4 text-green-600 mt-0.5" />
                        <span className="text-sm">{strength}</span>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <div className="text-center py-4">
                    <p className="text-sm text-muted-foreground">
                      Keep improving to identify your strengths compared to the industry.
                    </p>
                  </div>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2 text-orange-600">
                  <AlertTriangle className="h-5 w-5" />
                  Improvement Areas
                </CardTitle>
                <CardDescription>
                  Focus areas to reach industry standards
                </CardDescription>
              </CardHeader>
              <CardContent>
                {comparison.improvement_areas && comparison.improvement_areas.length > 0 ? (
                  <ul className="space-y-2">
                    {comparison.improvement_areas.map((area, idx) => (
                      <li key={idx} className="flex items-start gap-2 p-2 rounded-lg bg-orange-50 dark:bg-orange-950">
                        <XCircle className="h-4 w-4 text-orange-600 mt-0.5" />
                        <span className="text-sm">{area}</span>
                      </li>
                    ))}
                  </ul>
                ) : (
                  <div className="text-center py-4">
                    <Trophy className="h-8 w-8 text-green-500 mx-auto mb-2" />
                    <p className="text-sm text-muted-foreground">
                      Great job! No major improvement areas identified.
                    </p>
                  </div>
                )}
              </CardContent>
            </Card>
          </div>

          {/* Comparison Details */}
          <Card>
            <CardHeader>
              <CardTitle>Comparison Details</CardTitle>
              <CardDescription>
                Compared on {formatDate(comparison.compared_at)} against {comparison.industry_name}
              </CardDescription>
            </CardHeader>
            <CardContent>
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Metric</TableHead>
                    <TableHead className="text-right">Your Value</TableHead>
                    <TableHead className="text-right">Industry Average</TableHead>
                    <TableHead className="text-right">Industry Median</TableHead>
                    <TableHead className="text-right">Top Quartile</TableHead>
                    <TableHead className="text-center">Status</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  <TableRow>
                    <TableCell className="font-medium">Implementation Rate</TableCell>
                    <TableCell className="text-right font-bold">{comparison.org_implementation_rate.toFixed(1)}%</TableCell>
                    <TableCell className="text-right">{comparison.avg_implementation_rate.toFixed(1)}%</TableCell>
                    <TableCell className="text-right">{comparison.median_implementation_rate.toFixed(1)}%</TableCell>
                    <TableCell className="text-right text-green-600">{comparison.top_quartile_rate.toFixed(1)}%</TableCell>
                    <TableCell className="text-center">
                      {comparison.org_implementation_rate >= comparison.avg_implementation_rate ? (
                        <Badge className="bg-green-500">Above Avg</Badge>
                      ) : (
                        <Badge variant="destructive">Below Avg</Badge>
                      )}
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="font-medium">Framework Coverage</TableCell>
                    <TableCell className="text-right font-bold">{comparison.org_coverage_rate.toFixed(1)}%</TableCell>
                    <TableCell className="text-right">{comparison.avg_coverage_rate.toFixed(1)}%</TableCell>
                    <TableCell className="text-right">-</TableCell>
                    <TableCell className="text-right">-</TableCell>
                    <TableCell className="text-center">
                      {comparison.org_coverage_rate >= comparison.avg_coverage_rate ? (
                        <Badge className="bg-green-500">Above Avg</Badge>
                      ) : (
                        <Badge variant="destructive">Below Avg</Badge>
                      )}
                    </TableCell>
                  </TableRow>
                  <TableRow>
                    <TableCell className="font-medium">Risk Score</TableCell>
                    <TableCell className="text-right font-bold">{comparison.org_risk_score?.toFixed(1) ?? '-'}</TableCell>
                    <TableCell className="text-right">{comparison.avg_risk_score?.toFixed(1) ?? '-'}</TableCell>
                    <TableCell className="text-right">-</TableCell>
                    <TableCell className="text-right">-</TableCell>
                    <TableCell className="text-center">
                      {comparison.org_risk_score !== null && comparison.avg_risk_score !== null && (
                        comparison.org_risk_score <= comparison.avg_risk_score ? (
                          <Badge className="bg-green-500">Better</Badge>
                        ) : (
                          <Badge variant="destructive">Higher Risk</Badge>
                        )
                      )}
                    </TableCell>
                  </TableRow>
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </>
      )}

      {!comparison && benchmarkList.length > 0 && (
        <Card>
          <CardContent className="py-12 text-center">
            <Scale className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
            <h3 className="font-medium mb-2">No Comparison Yet</h3>
            <p className="text-muted-foreground mb-4">
              Select an industry benchmark above and click &quot;Compare Now&quot; to see how your organization measures up.
            </p>
          </CardContent>
        </Card>
      )}
    </div>
  )
}
