'use client'

import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Progress } from "@/components/ui/progress"
import { Loading } from "@/components/loading"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  useRiskPredictions,
  useRiskPredictionSummary,
  useRiskPredictionFactors
} from '@/hooks/use-api'
import {
  TrendingUp,
  TrendingDown,
  Minus,
  Target,
  AlertTriangle,
  Brain,
  Lightbulb,
  RefreshCcw,
  ChevronRight
} from "lucide-react"
import { formatDate, getRiskLevel } from '@/types'
import { apiClient } from '@/lib/api-client'
import { useMutation } from '@/hooks/use-api'
import { useState } from 'react'

function getConfidenceBadge(confidence: number) {
  if (confidence >= 0.8) {
    return <Badge className="bg-green-500">High ({(confidence * 100).toFixed(0)}%)</Badge>
  } else if (confidence >= 0.6) {
    return <Badge className="bg-yellow-500">Medium ({(confidence * 100).toFixed(0)}%)</Badge>
  }
  return <Badge variant="secondary">Low ({(confidence * 100).toFixed(0)}%)</Badge>
}

function getScoreChange(current: number | null, predicted: number) {
  if (current === null) return null
  const diff = predicted - current

  if (diff > 2) {
    return (
      <span className="flex items-center gap-1 text-red-600">
        <TrendingUp className="h-4 w-4" />
        +{diff.toFixed(1)}
      </span>
    )
  } else if (diff < -2) {
    return (
      <span className="flex items-center gap-1 text-green-600">
        <TrendingDown className="h-4 w-4" />
        {diff.toFixed(1)}
      </span>
    )
  }
  return (
    <span className="flex items-center gap-1 text-muted-foreground">
      <Minus className="h-4 w-4" />
      Stable
    </span>
  )
}

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

export default function RiskPredictionsPage() {
  const [expandedRisk, setExpandedRisk] = useState<string | null>(null)
  const { data: predictions, isLoading: predictionsLoading, refetch } = useRiskPredictions()
  const { data: summary, isLoading: summaryLoading } = useRiskPredictionSummary()
  const { data: factors, isLoading: factorsLoading } = useRiskPredictionFactors()

  const isLoading = predictionsLoading || summaryLoading || factorsLoading

  const { mutate: recomputePrediction, isLoading: computing } = useMutation(
    (riskId: string) => apiClient.post(`/analytics/predictions/${riskId}/recompute`)
  )

  const handleRecompute = async (riskId: string) => {
    try {
      await recomputePrediction(riskId)
      refetch()
    } catch (error) {
      console.error('Failed to recompute prediction:', error)
    }
  }

  if (isLoading) {
    return <Loading />
  }

  const summaryData = summary?.data
  const predictionList = predictions?.data ?? []
  const factorList = factors?.data ?? []

  return (
    <div className="space-y-6">
      <PageHeader
        title="Predictive Risk Scoring"
        description="ML-powered risk predictions and recommendations based on historical patterns"
      >
        <Button variant="outline" onClick={() => refetch()}>
          <RefreshCcw className="mr-2 h-4 w-4" />
          Refresh Predictions
        </Button>
      </PageHeader>

      {/* Summary Cards */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-5">
        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Total Predictions</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{summaryData?.total_predictions ?? 0}</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-green-600">High Confidence</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">{summaryData?.high_confidence_count ?? 0}</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-red-600">Increasing Risk</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-red-600 flex items-center gap-2">
              {summaryData?.increasing_risk_count ?? 0}
              <TrendingUp className="h-5 w-5" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium text-green-600">Decreasing Risk</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600 flex items-center gap-2">
              {summaryData?.decreasing_risk_count ?? 0}
              <TrendingDown className="h-5 w-5" />
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="pb-2">
            <CardTitle className="text-sm font-medium">Avg Confidence</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {((summaryData?.average_confidence ?? 0) * 100).toFixed(0)}%
            </div>
          </CardContent>
        </Card>
      </div>

      {/* Risk Predictions Table */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Brain className="h-5 w-5 text-primary" />
            Risk Predictions
          </CardTitle>
          <CardDescription>
            Predicted risk scores for the next {predictionList[0]?.time_horizon_days ?? 30} days
          </CardDescription>
        </CardHeader>
        <CardContent>
          {predictionList.length === 0 ? (
            <div className="text-center py-8">
              <Target className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
              <p className="text-muted-foreground">
                No risk predictions available yet. Predictions are generated automatically based on risk data.
              </p>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-20">Code</TableHead>
                  <TableHead>Risk</TableHead>
                  <TableHead className="text-center">Current Score</TableHead>
                  <TableHead className="text-center">Predicted Score</TableHead>
                  <TableHead className="text-center">Trend</TableHead>
                  <TableHead className="text-center">Confidence</TableHead>
                  <TableHead>Valid Until</TableHead>
                  <TableHead className="w-10"></TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {predictionList.map((prediction) => {
                  const currentLevel = getRiskLevel(prediction.current_score)
                  const predictedLevel = getRiskLevel(prediction.predicted_score)
                  const isExpanded = expandedRisk === prediction.id

                  return (
                    <>
                      <TableRow
                        key={prediction.id}
                        className="cursor-pointer hover:bg-muted/50"
                        onClick={() => setExpandedRisk(isExpanded ? null : prediction.id)}
                      >
                        <TableCell className="font-mono text-xs">{prediction.risk_code}</TableCell>
                        <TableCell className="font-medium max-w-xs truncate">
                          {prediction.risk_title}
                        </TableCell>
                        <TableCell className="text-center">
                          {prediction.current_score !== null ? (
                            <div className="flex items-center justify-center gap-2">
                              <span>{prediction.current_score.toFixed(1)}</span>
                              <Badge variant={getRiskBadgeVariant(currentLevel)} className="capitalize text-xs">
                                {currentLevel}
                              </Badge>
                            </div>
                          ) : (
                            <span className="text-muted-foreground">-</span>
                          )}
                        </TableCell>
                        <TableCell className="text-center">
                          <div className="flex items-center justify-center gap-2">
                            <span className="font-bold">{prediction.predicted_score.toFixed(1)}</span>
                            <Badge variant={getRiskBadgeVariant(predictedLevel)} className="capitalize text-xs">
                              {predictedLevel}
                            </Badge>
                          </div>
                        </TableCell>
                        <TableCell className="text-center">
                          {getScoreChange(prediction.current_score, prediction.predicted_score)}
                        </TableCell>
                        <TableCell className="text-center">
                          {getConfidenceBadge(prediction.confidence)}
                        </TableCell>
                        <TableCell className="text-muted-foreground text-sm">
                          {formatDate(prediction.expires_at)}
                        </TableCell>
                        <TableCell>
                          <ChevronRight className={`h-4 w-4 transition-transform ${isExpanded ? 'rotate-90' : ''}`} />
                        </TableCell>
                      </TableRow>
                      {isExpanded && (
                        <TableRow>
                          <TableCell colSpan={8} className="bg-muted/30">
                            <div className="p-4 space-y-4">
                              <div className="grid gap-4 md:grid-cols-2">
                                {/* Contributing Factors */}
                                <div>
                                  <h4 className="font-medium mb-2 flex items-center gap-2">
                                    <AlertTriangle className="h-4 w-4 text-orange-500" />
                                    Contributing Factors
                                  </h4>
                                  {prediction.contributing_factors ? (
                                    <ul className="text-sm space-y-1">
                                      {Object.entries(prediction.contributing_factors).map(([key, value]) => (
                                        <li key={key} className="flex items-center gap-2">
                                          <span className="w-2 h-2 rounded-full bg-orange-500" />
                                          {key}: {String(value)}
                                        </li>
                                      ))}
                                    </ul>
                                  ) : (
                                    <p className="text-sm text-muted-foreground">No specific factors identified</p>
                                  )}
                                </div>

                                {/* Recommended Actions */}
                                <div>
                                  <h4 className="font-medium mb-2 flex items-center gap-2">
                                    <Lightbulb className="h-4 w-4 text-yellow-500" />
                                    Recommended Actions
                                  </h4>
                                  {prediction.recommended_actions && prediction.recommended_actions.length > 0 ? (
                                    <ul className="text-sm space-y-1">
                                      {prediction.recommended_actions.map((action, idx) => (
                                        <li key={idx} className="flex items-center gap-2">
                                          <span className="w-2 h-2 rounded-full bg-green-500" />
                                          {action}
                                        </li>
                                      ))}
                                    </ul>
                                  ) : (
                                    <p className="text-sm text-muted-foreground">No specific recommendations</p>
                                  )}
                                </div>
                              </div>

                              <div className="flex justify-end pt-2 border-t">
                                <Button
                                  variant="outline"
                                  size="sm"
                                  onClick={(e) => {
                                    e.stopPropagation()
                                    handleRecompute(prediction.risk_id)
                                  }}
                                  disabled={computing}
                                >
                                  <RefreshCcw className={`mr-2 h-3 w-3 ${computing ? 'animate-spin' : ''}`} />
                                  Recompute
                                </Button>
                              </div>
                            </div>
                          </TableCell>
                        </TableRow>
                      )}
                    </>
                  )
                })}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Prediction Factors */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Target className="h-5 w-5 text-blue-500" />
            Prediction Factors
          </CardTitle>
          <CardDescription>
            Factors used in the risk prediction algorithm and their weights
          </CardDescription>
        </CardHeader>
        <CardContent>
          {factorList.length === 0 ? (
            <div className="text-center py-8">
              <p className="text-muted-foreground">
                No prediction factors configured.
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              {factorList.map((factor) => (
                <div key={factor.id} className="p-4 border rounded-lg">
                  <div className="flex items-center justify-between mb-2">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{factor.factor_name}</span>
                      {factor.is_system && (
                        <Badge variant="outline" className="text-xs">System</Badge>
                      )}
                    </div>
                    <Badge variant="secondary">{(factor.factor_weight * 100).toFixed(0)}% weight</Badge>
                  </div>
                  {factor.description && (
                    <p className="text-sm text-muted-foreground">{factor.description}</p>
                  )}
                  <Progress value={factor.factor_weight * 100} className="mt-2 h-2" />
                </div>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* How It Works */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Brain className="h-5 w-5 text-purple-500" />
            How Predictive Scoring Works
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-3">
            <div className="p-4 border rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
                  <span className="text-primary font-bold">1</span>
                </div>
                <h4 className="font-medium">Data Collection</h4>
              </div>
              <p className="text-sm text-muted-foreground">
                Historical risk data, control effectiveness, incident history, and external threat intelligence are collected.
              </p>
            </div>

            <div className="p-4 border rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
                  <span className="text-primary font-bold">2</span>
                </div>
                <h4 className="font-medium">Pattern Analysis</h4>
              </div>
              <p className="text-sm text-muted-foreground">
                Machine learning algorithms analyze patterns and correlations to identify risk trajectories.
              </p>
            </div>

            <div className="p-4 border rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <div className="w-8 h-8 rounded-full bg-primary/10 flex items-center justify-center">
                  <span className="text-primary font-bold">3</span>
                </div>
                <h4 className="font-medium">Score Generation</h4>
              </div>
              <p className="text-sm text-muted-foreground">
                Predicted scores are calculated with confidence levels and actionable recommendations are generated.
              </p>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
