'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { cn } from "@/lib/utils"
import { CheckCircle2, AlertCircle, Shield } from "lucide-react"

interface CategoryGapAnalysis {
  category: string | null
  total: number
  covered: number
  coverage_percentage: number
}

interface RequirementGapAnalysis {
  id: string
  code: string
  name: string
  category: string | null
  control_count: number
  is_covered: boolean
}

interface FrameworkGapAnalysis {
  framework_id: string
  framework_name: string
  total_requirements: number
  covered_requirements: number
  uncovered_requirements: number
  coverage_percentage: number
  by_category: CategoryGapAnalysis[]
  requirements: RequirementGapAnalysis[]
}

interface GapAnalysisCardProps {
  data: FrameworkGapAnalysis | null
  showDetails?: boolean
  className?: string
}

function getCoverageColor(percentage: number): string {
  if (percentage >= 80) return 'text-green-600 dark:text-green-400'
  if (percentage >= 50) return 'text-yellow-600 dark:text-yellow-400'
  return 'text-red-600 dark:text-red-400'
}

function getProgressColor(percentage: number): string {
  if (percentage >= 80) return 'bg-green-500'
  if (percentage >= 50) return 'bg-yellow-500'
  return 'bg-red-500'
}

export function GapAnalysisCard({ data, showDetails = false, className }: GapAnalysisCardProps) {
  if (!data) {
    return (
      <Card className={className}>
        <CardHeader>
          <CardTitle className="flex items-center gap-2 text-lg">
            <Shield className="h-5 w-5" />
            Gap Analysis
          </CardTitle>
          <CardDescription>Loading framework coverage data...</CardDescription>
        </CardHeader>
      </Card>
    )
  }

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-lg">
          <Shield className="h-5 w-5" />
          {data.framework_name} Coverage
        </CardTitle>
        <CardDescription>
          {data.covered_requirements} of {data.total_requirements} requirements covered
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Overall progress */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm font-medium">Overall Coverage</span>
            <span className={cn("text-sm font-bold", getCoverageColor(data.coverage_percentage))}>
              {data.coverage_percentage.toFixed(1)}%
            </span>
          </div>
          <div className="h-3 bg-secondary rounded-full overflow-hidden">
            <div
              className={cn("h-full transition-all", getProgressColor(data.coverage_percentage))}
              style={{ width: `${data.coverage_percentage}%` }}
            />
          </div>
        </div>

        {/* Summary stats */}
        <div className="grid grid-cols-3 gap-2 py-2">
          <div className="text-center">
            <div className="text-2xl font-bold text-green-600 dark:text-green-400">
              {data.covered_requirements}
            </div>
            <div className="text-xs text-muted-foreground">Covered</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold text-red-600 dark:text-red-400">
              {data.uncovered_requirements}
            </div>
            <div className="text-xs text-muted-foreground">Gaps</div>
          </div>
          <div className="text-center">
            <div className="text-2xl font-bold">{data.total_requirements}</div>
            <div className="text-xs text-muted-foreground">Total</div>
          </div>
        </div>

        {/* Coverage by category */}
        {data.by_category.length > 0 && (
          <div className="space-y-2">
            <h4 className="text-sm font-medium">By Category</h4>
            <div className="space-y-2">
              {data.by_category
                .sort((a, b) => b.total - a.total)
                .map((cat) => (
                  <div key={cat.category || 'uncategorized'} className="space-y-1">
                    <div className="flex items-center justify-between text-xs">
                      <span className="text-muted-foreground truncate max-w-[60%]">
                        {cat.category || 'Uncategorized'}
                      </span>
                      <span className={cn("font-medium", getCoverageColor(cat.coverage_percentage))}>
                        {cat.covered}/{cat.total} ({cat.coverage_percentage.toFixed(0)}%)
                      </span>
                    </div>
                    <div className="h-1.5 bg-secondary rounded-full overflow-hidden">
                      <div
                        className={cn("h-full transition-all", getProgressColor(cat.coverage_percentage))}
                        style={{ width: `${cat.coverage_percentage}%` }}
                      />
                    </div>
                  </div>
                ))}
            </div>
          </div>
        )}

        {/* Detailed requirements list (optional) */}
        {showDetails && data.requirements.length > 0 && (
          <div className="space-y-2 pt-2 border-t">
            <h4 className="text-sm font-medium">Requirements</h4>
            <div className="max-h-64 overflow-y-auto space-y-1">
              {data.requirements
                .sort((a, b) => {
                  // Show uncovered first, then by code
                  if (a.is_covered !== b.is_covered) return a.is_covered ? 1 : -1
                  return a.code.localeCompare(b.code)
                })
                .map((req) => (
                  <div
                    key={req.id}
                    className={cn(
                      "flex items-center justify-between p-2 rounded text-xs",
                      req.is_covered
                        ? "bg-green-50 dark:bg-green-950/30"
                        : "bg-red-50 dark:bg-red-950/30"
                    )}
                  >
                    <div className="flex items-center gap-2 min-w-0">
                      {req.is_covered ? (
                        <CheckCircle2 className="h-3.5 w-3.5 text-green-600 flex-shrink-0" />
                      ) : (
                        <AlertCircle className="h-3.5 w-3.5 text-red-600 flex-shrink-0" />
                      )}
                      <span className="font-mono text-muted-foreground">{req.code}</span>
                      <span className="truncate">{req.name}</span>
                    </div>
                    {req.is_covered && (
                      <Badge variant="secondary" className="text-xs">
                        {req.control_count} control{req.control_count !== 1 ? 's' : ''}
                      </Badge>
                    )}
                  </div>
                ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

// Compact version for dashboard
export function GapAnalysisSummary({
  frameworks
}: {
  frameworks: Array<{ id: string; name: string; coverage: number; total: number; covered: number }>
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-lg">
          <Shield className="h-5 w-5" />
          Framework Coverage
        </CardTitle>
        <CardDescription>Compliance gap analysis by framework</CardDescription>
      </CardHeader>
      <CardContent className="space-y-3">
        {frameworks.length === 0 ? (
          <p className="text-sm text-muted-foreground text-center py-4">
            No frameworks with requirements loaded
          </p>
        ) : (
          frameworks.map((fw) => (
            <div key={fw.id} className="space-y-1">
              <div className="flex items-center justify-between text-sm">
                <span className="font-medium truncate max-w-[60%]">{fw.name}</span>
                <span className={cn("font-bold", getCoverageColor(fw.coverage))}>
                  {fw.coverage.toFixed(0)}%
                </span>
              </div>
              <div className="flex items-center gap-2">
                <div className="flex-1 h-2 bg-secondary rounded-full overflow-hidden">
                  <div
                    className={cn("h-full transition-all", getProgressColor(fw.coverage))}
                    style={{ width: `${fw.coverage}%` }}
                  />
                </div>
                <span className="text-xs text-muted-foreground whitespace-nowrap">
                  {fw.covered}/{fw.total}
                </span>
              </div>
            </div>
          ))
        )}
      </CardContent>
    </Card>
  )
}
