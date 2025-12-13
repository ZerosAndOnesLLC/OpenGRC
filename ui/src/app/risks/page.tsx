'use client'

import { useState } from 'react'
import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Loading } from "@/components/loading"
import { RiskHeatmap } from "@/components/risk-heatmap"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Plus, AlertTriangle, Filter } from "lucide-react"
import { useRisks, useRiskHeatmap, useRiskStats } from '@/hooks/use-api'
import { formatStatus, getRiskLevel } from '@/types'

function getRiskBadgeVariant(level: string): "destructive" | "warning" | "success" | "secondary" {
  switch (level) {
    case 'critical':
    case 'high':
      return 'destructive'
    case 'medium':
      return 'warning'
    case 'low':
      return 'success'
    default:
      return 'secondary'
  }
}

function getScoreColor(score: number | null): string {
  if (score === null) return 'text-muted-foreground'
  if (score >= 15) return 'text-red-600 dark:text-red-400 font-bold'
  if (score >= 10) return 'text-orange-600 dark:text-orange-400 font-semibold'
  if (score >= 5) return 'text-yellow-600 dark:text-yellow-400'
  return 'text-green-600 dark:text-green-400'
}

export default function RisksPage() {
  const [filterLikelihood, setFilterLikelihood] = useState<number | null>(null)
  const [filterImpact, setFilterImpact] = useState<number | null>(null)

  const { data: risks, isLoading: risksLoading } = useRisks()
  const { data: heatmapData, isLoading: heatmapLoading } = useRiskHeatmap()
  const { data: stats, isLoading: statsLoading } = useRiskStats()

  const isLoading = risksLoading || heatmapLoading || statsLoading

  // Filter risks based on heatmap cell selection
  const filteredRisks = risks?.filter(r => {
    if (filterLikelihood !== null && r.likelihood !== filterLikelihood) return false
    if (filterImpact !== null && r.impact !== filterImpact) return false
    return true
  }) ?? []

  const handleCellClick = (likelihood: number, impact: number) => {
    if (filterLikelihood === likelihood && filterImpact === impact) {
      // Clear filter if clicking same cell
      setFilterLikelihood(null)
      setFilterImpact(null)
    } else {
      setFilterLikelihood(likelihood)
      setFilterImpact(impact)
    }
  }

  const clearFilter = () => {
    setFilterLikelihood(null)
    setFilterImpact(null)
  }

  if (isLoading) {
    return <Loading />
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Risk Register"
        description="Manage risk assessments and treatment plans"
      >
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          Add Risk
        </Button>
      </PageHeader>

      {/* Stats Overview */}
      {stats && (
        <div className="grid gap-4 md:grid-cols-4">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium">Total Risks</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.total}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-red-600">High/Critical</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-red-600">{stats.high_risks}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-yellow-600">Medium</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-yellow-600">{stats.medium_risks}</div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm font-medium text-green-600">Low</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-green-600">{stats.low_risks}</div>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Heatmap */}
      <div className="grid gap-4 lg:grid-cols-2">
        <RiskHeatmap
          data={heatmapData}
          onCellClick={handleCellClick}
        />
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-lg">
              <AlertTriangle className="h-5 w-5" />
              Risk Distribution
            </CardTitle>
            <CardDescription>
              Click on heatmap cells to filter the risk list below
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {stats?.by_category && stats.by_category.length > 0 ? (
                stats.by_category.map((cat) => (
                  <div key={cat.category || 'uncategorized'} className="flex items-center justify-between">
                    <span className="text-sm capitalize">{cat.category || 'Uncategorized'}</span>
                    <Badge variant="secondary">{cat.count}</Badge>
                  </div>
                ))
              ) : (
                <p className="text-sm text-muted-foreground text-center py-4">
                  No risks by category
                </p>
              )}
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Risk Table */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <CardTitle>
                {filterLikelihood !== null || filterImpact !== null
                  ? `Filtered Risks (${filteredRisks.length})`
                  : `All Risks (${risks?.length ?? 0})`}
              </CardTitle>
              <CardDescription>
                {filterLikelihood !== null && filterImpact !== null
                  ? `Showing risks with Likelihood ${filterLikelihood} and Impact ${filterImpact}`
                  : 'Click on heatmap cells to filter'}
              </CardDescription>
            </div>
            {(filterLikelihood !== null || filterImpact !== null) && (
              <Button variant="outline" size="sm" onClick={clearFilter}>
                <Filter className="mr-2 h-4 w-4" />
                Clear Filter
              </Button>
            )}
          </div>
        </CardHeader>
        <CardContent>
          {filteredRisks.length === 0 ? (
            <div className="flex items-center justify-center py-8">
              <p className="text-sm text-muted-foreground">
                {risks?.length === 0
                  ? 'No risks identified. Add a risk to get started.'
                  : 'No risks match the current filter.'}
              </p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-20">Code</TableHead>
                  <TableHead>Title</TableHead>
                  <TableHead>Category</TableHead>
                  <TableHead className="text-center">L</TableHead>
                  <TableHead className="text-center">I</TableHead>
                  <TableHead className="text-center">Score</TableHead>
                  <TableHead>Level</TableHead>
                  <TableHead>Status</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {filteredRisks.map((risk) => {
                  const level = getRiskLevel(risk.inherent_score)
                  return (
                    <TableRow key={risk.id} className="cursor-pointer hover:bg-muted/50">
                      <TableCell className="font-mono text-xs">{risk.code}</TableCell>
                      <TableCell className="font-medium max-w-xs truncate">{risk.title}</TableCell>
                      <TableCell className="capitalize">{risk.category || '-'}</TableCell>
                      <TableCell className="text-center">{risk.likelihood ?? '-'}</TableCell>
                      <TableCell className="text-center">{risk.impact ?? '-'}</TableCell>
                      <TableCell className={`text-center ${getScoreColor(risk.inherent_score)}`}>
                        {risk.inherent_score ?? '-'}
                      </TableCell>
                      <TableCell>
                        <Badge variant={getRiskBadgeVariant(level)} className="capitalize">
                          {level}
                        </Badge>
                      </TableCell>
                      <TableCell>
                        <Badge variant="outline" className="capitalize">
                          {formatStatus(risk.status)}
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
    </div>
  )
}
