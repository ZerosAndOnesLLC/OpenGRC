'use client'

import { useState } from 'react'
import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Loading } from "@/components/loading"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { useSavedReports, useReportTemplates } from '@/hooks/use-api'
import {
  FileBarChart,
  Plus,
  Play,
  Clock,
  Trash2,
  Edit2,
  Download,
  Calendar,
  FileText,
  Shield,
  AlertTriangle,
  Users,
  CheckSquare
} from "lucide-react"
import { formatDate, formatDateTime } from '@/types'
import { apiClient } from '@/lib/api-client'
import { useMutation } from '@/hooks/use-api'

const reportTypeIcons: Record<string, typeof FileText> = {
  compliance_summary: Shield,
  risk_assessment: AlertTriangle,
  control_status: CheckSquare,
  vendor_risk: Users,
  executive_summary: FileBarChart,
  custom: FileText,
}

function getReportIcon(reportType: string) {
  const Icon = reportTypeIcons[reportType] || FileText
  return <Icon className="h-5 w-5" />
}

export default function ReportBuilderPage() {
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [newReport, setNewReport] = useState({
    name: '',
    description: '',
    report_type: '',
    config: {} as Record<string, unknown>,
    is_scheduled: false,
    schedule_cron: '',
  })

  const { data: reports, isLoading: reportsLoading, refetch } = useSavedReports()
  const { data: templates, isLoading: templatesLoading } = useReportTemplates()

  const isLoading = reportsLoading || templatesLoading

  const { mutate: createReport, isLoading: creating } = useMutation(
    (data: typeof newReport) => apiClient.post('/analytics/reports', data)
  )

  const { mutate: deleteReport, isLoading: deleting } = useMutation(
    (reportId: string) => apiClient.delete(`/analytics/reports/${reportId}`)
  )

  const handleCreateReport = async () => {
    try {
      await createReport(newReport)
      setIsCreateDialogOpen(false)
      setNewReport({
        name: '',
        description: '',
        report_type: '',
        config: {},
        is_scheduled: false,
        schedule_cron: '',
      })
      refetch()
    } catch (error) {
      console.error('Failed to create report:', error)
    }
  }

  const handleDeleteReport = async (reportId: string) => {
    if (!confirm('Are you sure you want to delete this report?')) return
    try {
      await deleteReport(reportId)
      refetch()
    } catch (error) {
      console.error('Failed to delete report:', error)
    }
  }

  const handleSelectTemplate = (templateId: string) => {
    const template = templates?.data?.find(t => t.id === templateId)
    if (template) {
      setNewReport(prev => ({
        ...prev,
        name: template.name,
        description: template.description || '',
        report_type: template.report_type,
        config: template.default_config,
      }))
    }
  }

  if (isLoading) {
    return <Loading />
  }

  const reportList = reports?.data ?? []
  const templateList = templates?.data ?? []

  return (
    <div className="space-y-6">
      <PageHeader
        title="Custom Report Builder"
        description="Create, schedule, and manage custom compliance and analytics reports"
      >
        <Dialog open={isCreateDialogOpen} onOpenChange={setIsCreateDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="mr-2 h-4 w-4" />
              Create Report
            </Button>
          </DialogTrigger>
          <DialogContent className="max-w-2xl">
            <DialogHeader>
              <DialogTitle>Create New Report</DialogTitle>
              <DialogDescription>
                Configure a new report from scratch or start from a template
              </DialogDescription>
            </DialogHeader>

            <div className="space-y-4 py-4">
              {/* Template Selection */}
              <div className="space-y-2">
                <Label>Start from Template (Optional)</Label>
                <Select onValueChange={handleSelectTemplate}>
                  <SelectTrigger>
                    <SelectValue placeholder="Select a template..." />
                  </SelectTrigger>
                  <SelectContent>
                    {templateList.map((template) => (
                      <SelectItem key={template.id} value={template.id}>
                        {template.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {/* Report Name */}
              <div className="space-y-2">
                <Label htmlFor="name">Report Name *</Label>
                <Input
                  id="name"
                  value={newReport.name}
                  onChange={(e) => setNewReport(prev => ({ ...prev, name: e.target.value }))}
                  placeholder="e.g., Monthly Compliance Summary"
                />
              </div>

              {/* Description */}
              <div className="space-y-2">
                <Label htmlFor="description">Description</Label>
                <Textarea
                  id="description"
                  value={newReport.description}
                  onChange={(e) => setNewReport(prev => ({ ...prev, description: e.target.value }))}
                  placeholder="Brief description of this report..."
                  rows={3}
                />
              </div>

              {/* Report Type */}
              <div className="space-y-2">
                <Label>Report Type *</Label>
                <Select
                  value={newReport.report_type}
                  onValueChange={(value) => setNewReport(prev => ({ ...prev, report_type: value }))}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select report type..." />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="compliance_summary">Compliance Summary</SelectItem>
                    <SelectItem value="risk_assessment">Risk Assessment</SelectItem>
                    <SelectItem value="control_status">Control Status</SelectItem>
                    <SelectItem value="vendor_risk">Vendor Risk</SelectItem>
                    <SelectItem value="executive_summary">Executive Summary</SelectItem>
                    <SelectItem value="custom">Custom</SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* Schedule Toggle */}
              <div className="flex items-center gap-4">
                <div className="flex items-center gap-2">
                  <input
                    type="checkbox"
                    id="is_scheduled"
                    checked={newReport.is_scheduled}
                    onChange={(e) => setNewReport(prev => ({ ...prev, is_scheduled: e.target.checked }))}
                    className="h-4 w-4 rounded border-gray-300"
                  />
                  <Label htmlFor="is_scheduled">Schedule this report</Label>
                </div>
              </div>

              {/* Schedule Cron */}
              {newReport.is_scheduled && (
                <div className="space-y-2">
                  <Label htmlFor="schedule_cron">Schedule (Cron Expression)</Label>
                  <Input
                    id="schedule_cron"
                    value={newReport.schedule_cron}
                    onChange={(e) => setNewReport(prev => ({ ...prev, schedule_cron: e.target.value }))}
                    placeholder="e.g., 0 9 * * 1 (Mondays at 9am)"
                  />
                  <p className="text-xs text-muted-foreground">
                    Examples: &quot;0 9 * * 1&quot; (Weekly Mon 9am), &quot;0 0 1 * *&quot; (Monthly 1st at midnight)
                  </p>
                </div>
              )}
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={() => setIsCreateDialogOpen(false)}>
                Cancel
              </Button>
              <Button
                onClick={handleCreateReport}
                disabled={creating || !newReport.name || !newReport.report_type}
              >
                {creating ? 'Creating...' : 'Create Report'}
              </Button>
            </DialogFooter>
          </DialogContent>
        </Dialog>
      </PageHeader>

      {/* Report Templates */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileText className="h-5 w-5 text-primary" />
            Report Templates
          </CardTitle>
          <CardDescription>
            Pre-built templates to quickly create common reports
          </CardDescription>
        </CardHeader>
        <CardContent>
          {templateList.length === 0 ? (
            <div className="text-center py-8">
              <FileText className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
              <p className="text-muted-foreground">
                No report templates available yet.
              </p>
            </div>
          ) : (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {templateList.map((template) => (
                <Card
                  key={template.id}
                  className="cursor-pointer hover:border-primary transition-colors"
                  onClick={() => {
                    handleSelectTemplate(template.id)
                    setIsCreateDialogOpen(true)
                  }}
                >
                  <CardContent className="pt-6">
                    <div className="flex items-start gap-3">
                      <div className="p-2 rounded-lg bg-primary/10">
                        {getReportIcon(template.report_type)}
                      </div>
                      <div className="flex-1">
                        <h3 className="font-medium">{template.name}</h3>
                        {template.description && (
                          <p className="text-sm text-muted-foreground mt-1 line-clamp-2">
                            {template.description}
                          </p>
                        )}
                        <div className="flex gap-2 mt-2">
                          <Badge variant="outline" className="text-xs capitalize">
                            {template.report_type.replace('_', ' ')}
                          </Badge>
                          {template.category && (
                            <Badge variant="secondary" className="text-xs">
                              {template.category}
                            </Badge>
                          )}
                        </div>
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Saved Reports */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileBarChart className="h-5 w-5 text-blue-500" />
            Your Reports
          </CardTitle>
          <CardDescription>
            Manage your saved and scheduled reports
          </CardDescription>
        </CardHeader>
        <CardContent>
          {reportList.length === 0 ? (
            <div className="text-center py-8">
              <FileBarChart className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
              <h3 className="font-medium mb-2">No Reports Yet</h3>
              <p className="text-muted-foreground mb-4">
                Create your first report using a template or start from scratch.
              </p>
              <Button onClick={() => setIsCreateDialogOpen(true)}>
                <Plus className="mr-2 h-4 w-4" />
                Create First Report
              </Button>
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Report</TableHead>
                  <TableHead>Type</TableHead>
                  <TableHead>Schedule</TableHead>
                  <TableHead>Last Run</TableHead>
                  <TableHead>Next Run</TableHead>
                  <TableHead className="text-right">Actions</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {reportList.map((report) => (
                  <TableRow key={report.id}>
                    <TableCell>
                      <div className="flex items-start gap-3">
                        <div className="p-1.5 rounded bg-muted">
                          {getReportIcon(report.report_type)}
                        </div>
                        <div>
                          <div className="font-medium">{report.name}</div>
                          {report.description && (
                            <div className="text-sm text-muted-foreground line-clamp-1">
                              {report.description}
                            </div>
                          )}
                        </div>
                      </div>
                    </TableCell>
                    <TableCell>
                      <Badge variant="outline" className="capitalize">
                        {report.report_type.replace('_', ' ')}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      {report.is_scheduled ? (
                        <Badge className="bg-blue-500">
                          <Clock className="h-3 w-3 mr-1" />
                          Scheduled
                        </Badge>
                      ) : (
                        <Badge variant="secondary">Manual</Badge>
                      )}
                    </TableCell>
                    <TableCell className="text-muted-foreground text-sm">
                      {report.last_run_at ? formatDateTime(report.last_run_at) : '-'}
                    </TableCell>
                    <TableCell className="text-muted-foreground text-sm">
                      {report.next_run_at ? formatDateTime(report.next_run_at) : '-'}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-2">
                        <Button variant="ghost" size="icon" title="Run Report">
                          <Play className="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="icon" title="Download">
                          <Download className="h-4 w-4" />
                        </Button>
                        <Button variant="ghost" size="icon" title="Edit">
                          <Edit2 className="h-4 w-4" />
                        </Button>
                        <Button
                          variant="ghost"
                          size="icon"
                          title="Delete"
                          onClick={() => handleDeleteReport(report.id)}
                          disabled={deleting}
                        >
                          <Trash2 className="h-4 w-4 text-red-500" />
                        </Button>
                      </div>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>

      {/* Report Configuration Help */}
      <Card>
        <CardHeader>
          <CardTitle>Report Types Guide</CardTitle>
          <CardDescription>
            Learn about the different report types and their use cases
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            <div className="p-4 border rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Shield className="h-5 w-5 text-primary" />
                <h4 className="font-medium">Compliance Summary</h4>
              </div>
              <p className="text-sm text-muted-foreground">
                Overview of compliance status across all frameworks, including control implementation rates and gap analysis.
              </p>
            </div>

            <div className="p-4 border rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <AlertTriangle className="h-5 w-5 text-orange-500" />
                <h4 className="font-medium">Risk Assessment</h4>
              </div>
              <p className="text-sm text-muted-foreground">
                Detailed risk register with scores, trends, treatment status, and recommendations for mitigation.
              </p>
            </div>

            <div className="p-4 border rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <CheckSquare className="h-5 w-5 text-blue-500" />
                <h4 className="font-medium">Control Status</h4>
              </div>
              <p className="text-sm text-muted-foreground">
                Control implementation progress, test results, and evidence collection status by framework.
              </p>
            </div>

            <div className="p-4 border rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <Users className="h-5 w-5 text-purple-500" />
                <h4 className="font-medium">Vendor Risk</h4>
              </div>
              <p className="text-sm text-muted-foreground">
                Third-party vendor risk ratings, assessment history, and contract status overview.
              </p>
            </div>

            <div className="p-4 border rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <FileBarChart className="h-5 w-5 text-green-500" />
                <h4 className="font-medium">Executive Summary</h4>
              </div>
              <p className="text-sm text-muted-foreground">
                High-level metrics and KPIs designed for executive leadership and board presentations.
              </p>
            </div>

            <div className="p-4 border rounded-lg">
              <div className="flex items-center gap-2 mb-2">
                <FileText className="h-5 w-5 text-muted-foreground" />
                <h4 className="font-medium">Custom</h4>
              </div>
              <p className="text-sm text-muted-foreground">
                Build a completely custom report by selecting specific data sources and visualizations.
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Scheduling Tips */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Calendar className="h-5 w-5 text-blue-500" />
            Scheduling Tips
          </CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-4">
            <p className="text-sm text-muted-foreground">
              Reports can be scheduled using cron expressions. Here are some common examples:
            </p>
            <div className="grid gap-2 md:grid-cols-2">
              <div className="flex items-center gap-2 p-2 bg-muted rounded">
                <code className="text-xs bg-background px-2 py-1 rounded">0 9 * * 1</code>
                <span className="text-sm">Every Monday at 9:00 AM</span>
              </div>
              <div className="flex items-center gap-2 p-2 bg-muted rounded">
                <code className="text-xs bg-background px-2 py-1 rounded">0 0 1 * *</code>
                <span className="text-sm">First of every month at midnight</span>
              </div>
              <div className="flex items-center gap-2 p-2 bg-muted rounded">
                <code className="text-xs bg-background px-2 py-1 rounded">0 8 * * 1-5</code>
                <span className="text-sm">Weekdays at 8:00 AM</span>
              </div>
              <div className="flex items-center gap-2 p-2 bg-muted rounded">
                <code className="text-xs bg-background px-2 py-1 rounded">0 0 1 1,4,7,10 *</code>
                <span className="text-sm">Quarterly (Jan, Apr, Jul, Oct 1st)</span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
