'use client'

import { useState } from 'react'
import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
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
} from "@/components/ui/dialog"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Loading } from "@/components/loading"
import { AuditDetailSheet } from "@/components/audit-detail-sheet"
import {
  Plus,
  Search,
  Filter,
  ClipboardCheck,
  AlertTriangle,
  FileQuestion,
  CheckCircle,
  Clock,
} from "lucide-react"
import { useAudits, useAuditStats, useFrameworks, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type { AuditWithStats, CreateAudit, AuditStats, AuditType } from '@/types'
import { formatStatus, formatDate } from '@/types'

const statusVariants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning'> = {
  planning: 'secondary',
  fieldwork: 'warning',
  in_progress: 'warning',
  review: 'default',
  completed: 'success',
  cancelled: 'destructive',
}

const typeLabels: Record<string, string> = {
  internal: 'Internal',
  external: 'External',
  certification: 'Certification',
  compliance: 'Compliance',
  readiness: 'Readiness',
}

function StatsCards({ stats }: { stats: AuditStats | null }) {
  if (!stats) return null

  return (
    <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Total Audits</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <ClipboardCheck className="h-5 w-5 text-primary" />
            <span className="text-2xl font-bold">{stats.total}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Active</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Clock className="h-5 w-5 text-blue-500" />
            <span className="text-2xl font-bold">{stats.active}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Completed</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <CheckCircle className="h-5 w-5 text-green-500" />
            <span className="text-2xl font-bold">{stats.completed}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Open Requests</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <FileQuestion className="h-5 w-5 text-yellow-500" />
            <span className="text-2xl font-bold">{stats.open_requests}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Open Findings</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-red-500" />
            <span className="text-2xl font-bold">{stats.open_findings}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function AuditForm({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}) {
  const { data: frameworks } = useFrameworks()
  const [formData, setFormData] = useState<CreateAudit>({
    name: '',
    audit_type: 'external',
    auditor_firm: '',
    auditor_contact: '',
    period_start: '',
    period_end: '',
  })

  const createMutation = useMutation(async (data: CreateAudit) => {
    return apiClient.post<AuditWithStats>('/audits', data)
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createMutation.mutate(formData)
      onOpenChange(false)
      setFormData({
        name: '',
        audit_type: 'external',
        auditor_firm: '',
        auditor_contact: '',
        period_start: '',
        period_end: '',
      })
      onSuccess()
    } catch {
      // Error is handled by mutation
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Create Audit</DialogTitle>
          <DialogDescription>
            Set up a new audit engagement to track requests and findings.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="name" className="text-right">
                Name *
              </Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="col-span-3"
                placeholder="e.g., SOC 2 Type II 2024"
                required
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="audit_type" className="text-right">
                Type
              </Label>
              <Select
                value={formData.audit_type}
                onValueChange={(value: AuditType) => setFormData({ ...formData, audit_type: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="external">External</SelectItem>
                  <SelectItem value="internal">Internal</SelectItem>
                  <SelectItem value="certification">Certification</SelectItem>
                  <SelectItem value="compliance">Compliance</SelectItem>
                  <SelectItem value="readiness">Readiness</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="framework_id" className="text-right">
                Framework
              </Label>
              <Select
                value={formData.framework_id || ''}
                onValueChange={(value) => setFormData({ ...formData, framework_id: value || undefined })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select framework (optional)" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">None</SelectItem>
                  {frameworks?.map((framework) => (
                    <SelectItem key={framework.id} value={framework.id}>
                      {framework.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="auditor_firm" className="text-right">
                Auditor Firm
              </Label>
              <Input
                id="auditor_firm"
                value={formData.auditor_firm || ''}
                onChange={(e) => setFormData({ ...formData, auditor_firm: e.target.value })}
                className="col-span-3"
                placeholder="e.g., Deloitte, KPMG, EY"
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="auditor_contact" className="text-right">
                Auditor Contact
              </Label>
              <Input
                id="auditor_contact"
                value={formData.auditor_contact || ''}
                onChange={(e) => setFormData({ ...formData, auditor_contact: e.target.value })}
                className="col-span-3"
                placeholder="e.g., john.doe@auditor.com"
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="period_start" className="text-right">
                Period Start
              </Label>
              <Input
                id="period_start"
                type="date"
                value={formData.period_start || ''}
                onChange={(e) => setFormData({ ...formData, period_start: e.target.value })}
                className="col-span-3"
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="period_end" className="text-right">
                Period End
              </Label>
              <Input
                id="period_end"
                type="date"
                value={formData.period_end || ''}
                onChange={(e) => setFormData({ ...formData, period_end: e.target.value })}
                className="col-span-3"
              />
            </div>
          </div>
          {createMutation.error && (
            <div className="text-sm text-red-500 mb-4">
              {createMutation.error.message}
            </div>
          )}
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={createMutation.isLoading}>
              {createMutation.isLoading ? 'Creating...' : 'Create Audit'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default function AuditsPage() {
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [typeFilter, setTypeFilter] = useState<string>('')
  const [isCreateOpen, setIsCreateOpen] = useState(false)
  const [selectedAuditId, setSelectedAuditId] = useState<string | null>(null)
  const [isDetailOpen, setIsDetailOpen] = useState(false)

  const query: Record<string, string | number | boolean> = {}
  if (search) query.search = search
  if (statusFilter) query.status = statusFilter
  if (typeFilter) query.audit_type = typeFilter

  const { data: audits, isLoading, error, refetch } = useAudits(query)
  const { data: stats, refetch: refetchStats } = useAuditStats()

  const handleSuccess = () => {
    refetch()
    refetchStats()
  }

  const handleRowClick = (auditId: string) => {
    setSelectedAuditId(auditId)
    setIsDetailOpen(true)
  }

  const handleDetailClose = () => {
    setIsDetailOpen(false)
    setSelectedAuditId(null)
  }

  if (isLoading) {
    return <Loading />
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <p className="text-red-500 mb-2">Failed to load audits</p>
          <p className="text-sm text-muted-foreground">{error.message}</p>
          <Button onClick={() => refetch()} className="mt-4">
            Retry
          </Button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Audits"
        description="Manage audits and external auditor engagements"
      >
        <Button onClick={() => setIsCreateOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          New Audit
        </Button>
      </PageHeader>

      <StatsCards stats={stats} />

      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search audits..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-10"
          />
        </div>
        <div className="flex gap-2">
          <Select value={statusFilter} onValueChange={setStatusFilter}>
            <SelectTrigger className="w-[140px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Status" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Statuses</SelectItem>
              <SelectItem value="planning">Planning</SelectItem>
              <SelectItem value="fieldwork">Fieldwork</SelectItem>
              <SelectItem value="review">Review</SelectItem>
              <SelectItem value="completed">Completed</SelectItem>
              <SelectItem value="cancelled">Cancelled</SelectItem>
            </SelectContent>
          </Select>
          <Select value={typeFilter} onValueChange={setTypeFilter}>
            <SelectTrigger className="w-[140px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Types</SelectItem>
              <SelectItem value="external">External</SelectItem>
              <SelectItem value="internal">Internal</SelectItem>
              <SelectItem value="certification">Certification</SelectItem>
              <SelectItem value="compliance">Compliance</SelectItem>
              <SelectItem value="readiness">Readiness</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {audits && audits.length > 0 ? (
        <div className="rounded-md border">
          <table className="w-full">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="p-3 text-left text-sm font-medium">Audit</th>
                <th className="p-3 text-left text-sm font-medium">Type</th>
                <th className="p-3 text-left text-sm font-medium">Auditor</th>
                <th className="p-3 text-left text-sm font-medium">Period</th>
                <th className="p-3 text-left text-sm font-medium">Requests</th>
                <th className="p-3 text-left text-sm font-medium">Findings</th>
                <th className="p-3 text-left text-sm font-medium">Status</th>
              </tr>
            </thead>
            <tbody>
              {audits.map((audit) => (
                <tr
                  key={audit.id}
                  className="border-b hover:bg-muted/25 cursor-pointer"
                  onClick={() => handleRowClick(audit.id)}
                >
                  <td className="p-3 text-sm">
                    <div className="font-medium">{audit.name}</div>
                  </td>
                  <td className="p-3 text-sm">
                    {typeLabels[audit.audit_type] || audit.audit_type || '-'}
                  </td>
                  <td className="p-3 text-sm">
                    {audit.auditor_firm || '-'}
                  </td>
                  <td className="p-3 text-sm">
                    {audit.period_start && audit.period_end ? (
                      <span>
                        {formatDate(audit.period_start)} - {formatDate(audit.period_end)}
                      </span>
                    ) : (
                      '-'
                    )}
                  </td>
                  <td className="p-3 text-sm">
                    <div className="flex items-center gap-2">
                      <span>{audit.request_count}</span>
                      {audit.open_requests > 0 && (
                        <Badge variant="outline" className="text-xs">
                          {audit.open_requests} open
                        </Badge>
                      )}
                    </div>
                  </td>
                  <td className="p-3 text-sm">
                    <div className="flex items-center gap-2">
                      <span>{audit.finding_count}</span>
                      {audit.open_findings > 0 && (
                        <Badge variant="destructive" className="text-xs">
                          {audit.open_findings} open
                        </Badge>
                      )}
                    </div>
                  </td>
                  <td className="p-3 text-sm">
                    <Badge variant={statusVariants[audit.status] || 'secondary'}>
                      {formatStatus(audit.status)}
                    </Badge>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <ClipboardCheck className="h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No audits yet</h3>
            <p className="text-muted-foreground text-sm mb-4">
              Create an audit to start tracking requests and findings.
            </p>
            <Button onClick={() => setIsCreateOpen(true)}>
              <Plus className="mr-2 h-4 w-4" />
              Create Your First Audit
            </Button>
          </CardContent>
        </Card>
      )}

      <AuditForm
        open={isCreateOpen}
        onOpenChange={setIsCreateOpen}
        onSuccess={handleSuccess}
      />

      <AuditDetailSheet
        auditId={selectedAuditId}
        open={isDetailOpen}
        onOpenChange={(open) => {
          if (!open) handleDetailClose()
        }}
        onUpdate={handleSuccess}
        onDelete={handleSuccess}
      />
    </div>
  )
}
