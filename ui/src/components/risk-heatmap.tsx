'use client'

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { cn } from "@/lib/utils"

interface HeatmapCell {
  likelihood: number
  impact: number
  count: number
}

interface RiskHeatmapData {
  cells: HeatmapCell[]
  total_risks: number
  risks_with_scores: number
}

interface RiskHeatmapProps {
  data: RiskHeatmapData | null
  onCellClick?: (likelihood: number, impact: number) => void
  className?: string
}

function getCellColor(likelihood: number, impact: number): string {
  const score = likelihood * impact
  if (score >= 20) return 'bg-red-600 hover:bg-red-700 text-white'
  if (score >= 15) return 'bg-red-500 hover:bg-red-600 text-white'
  if (score >= 10) return 'bg-orange-500 hover:bg-orange-600 text-white'
  if (score >= 5) return 'bg-yellow-500 hover:bg-yellow-600 text-black'
  return 'bg-green-500 hover:bg-green-600 text-white'
}

function getCellBackgroundColor(likelihood: number, impact: number): string {
  const score = likelihood * impact
  if (score >= 20) return 'bg-red-100 dark:bg-red-950'
  if (score >= 15) return 'bg-red-50 dark:bg-red-900/50'
  if (score >= 10) return 'bg-orange-50 dark:bg-orange-900/50'
  if (score >= 5) return 'bg-yellow-50 dark:bg-yellow-900/50'
  return 'bg-green-50 dark:bg-green-900/50'
}

export function RiskHeatmap({ data, onCellClick, className }: RiskHeatmapProps) {
  // Build a lookup map for quick cell access
  const cellMap = new Map<string, number>()
  if (data?.cells) {
    for (const cell of data.cells) {
      cellMap.set(`${cell.likelihood}-${cell.impact}`, cell.count)
    }
  }

  const impactLabels = ['Negligible', 'Minor', 'Moderate', 'Major', 'Severe']
  const likelihoodLabels = ['Rare', 'Unlikely', 'Possible', 'Likely', 'Almost Certain']

  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle className="text-lg">Risk Heatmap</CardTitle>
        <CardDescription>
          {data ? `${data.risks_with_scores} of ${data.total_risks} risks scored` : 'Loading...'}
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="flex flex-col">
          {/* Y-axis label */}
          <div className="flex items-center mb-2">
            <div className="w-24" />
            <div className="flex-1 text-center text-xs font-medium text-muted-foreground">
              Impact
            </div>
          </div>

          {/* Impact headers */}
          <div className="flex items-center mb-1">
            <div className="w-24" />
            {[1, 2, 3, 4, 5].map((impact) => (
              <div
                key={impact}
                className="flex-1 text-center text-xs text-muted-foreground px-0.5"
                title={impactLabels[impact - 1]}
              >
                {impact}
              </div>
            ))}
          </div>

          {/* Grid rows - likelihood from 5 (top) to 1 (bottom) */}
          <div className="flex">
            {/* X-axis label */}
            <div className="w-6 flex items-center justify-center">
              <span className="text-xs font-medium text-muted-foreground -rotate-90 whitespace-nowrap">
                Likelihood
              </span>
            </div>
            <div className="flex-1">
              {[5, 4, 3, 2, 1].map((likelihood) => (
                <div key={likelihood} className="flex items-center mb-1">
                  <div
                    className="w-[72px] text-right pr-2 text-xs text-muted-foreground"
                    title={likelihoodLabels[likelihood - 1]}
                  >
                    {likelihood}
                  </div>
                  {[1, 2, 3, 4, 5].map((impact) => {
                    const count = cellMap.get(`${likelihood}-${impact}`) || 0
                    const hasRisks = count > 0
                    return (
                      <div
                        key={`${likelihood}-${impact}`}
                        className={cn(
                          "flex-1 aspect-square m-0.5 rounded flex items-center justify-center text-xs font-medium transition-colors cursor-pointer",
                          hasRisks
                            ? getCellColor(likelihood, impact)
                            : getCellBackgroundColor(likelihood, impact)
                        )}
                        onClick={() => onCellClick?.(likelihood, impact)}
                        title={`Likelihood: ${likelihood} (${likelihoodLabels[likelihood - 1]})\nImpact: ${impact} (${impactLabels[impact - 1]})\nScore: ${likelihood * impact}\nRisks: ${count}`}
                      >
                        {hasRisks ? count : ''}
                      </div>
                    )
                  })}
                </div>
              ))}
            </div>
          </div>

          {/* Legend */}
          <div className="flex items-center justify-center gap-4 mt-4 pt-4 border-t">
            <div className="flex items-center gap-1.5">
              <div className="w-3 h-3 rounded bg-green-500" />
              <span className="text-xs text-muted-foreground">Low (1-4)</span>
            </div>
            <div className="flex items-center gap-1.5">
              <div className="w-3 h-3 rounded bg-yellow-500" />
              <span className="text-xs text-muted-foreground">Medium (5-9)</span>
            </div>
            <div className="flex items-center gap-1.5">
              <div className="w-3 h-3 rounded bg-orange-500" />
              <span className="text-xs text-muted-foreground">High (10-14)</span>
            </div>
            <div className="flex items-center gap-1.5">
              <div className="w-3 h-3 rounded bg-red-500" />
              <span className="text-xs text-muted-foreground">Critical (15+)</span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}
