'use client'

import { useState, useEffect } from 'react'
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
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Separator } from "@/components/ui/separator"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import { Loading } from "@/components/loading"
import {
  Save,
  Trash2,
  Edit2,
  Plus,
  FileQuestion,
  AlertTriangle,
  ClipboardCheck,
  Calendar,
  Building2,
  CheckCircle,
  Clock,
  Package,
  FileText,
  Download,
  HardDrive,
  ListTodo,
} from "lucide-react"
import { useAudit, useAuditRequests, useAuditFindings, useAuditEvidencePackage, useFrameworks, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type {
  AuditWithStats,
  UpdateAudit,
  AuditRequest,
  AuditFinding,
  CreateAuditRequest,
  CreateAuditFinding,
  UpdateAuditFinding,
  AuditEvidencePackage,
} from '@/types'
import { formatStatus, formatDate, formatDateTime } from '@/types'

const statusVariants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning'> = {
  planning: 'secondary',
  fieldwork: 'warning',
  in_progress: 'warning',
  review: 'default',
  completed: 'success',
  cancelled: 'destructive',
}

const requestStatusVariants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning'> = {
  open: 'warning',
  in_progress: 'default',
  submitted: 'secondary',
  accepted: 'success',
  rejected: 'destructive',
}

const findingStatusVariants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline' | 'success' | 'warning'> = {
  open: 'destructive',
  remediation: 'warning',
  in_remediation: 'default',
  closed: 'success',
  accepted: 'secondary',
}

const findingTypeVariants: Record<string, 'default' | 'secondary' | 'destructive' | 'outline' | 'warning'> = {
  observation: 'secondary',
  exception: 'destructive',
  deficiency: 'destructive',
  recommendation: 'outline',
}

interface RequestFormProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  auditId: string
  onSuccess: () => void
}

function RequestForm({ open, onOpenChange, auditId, onSuccess }: RequestFormProps) {
  const [formData, setFormData] = useState<CreateAuditRequest>({
    title: '',
    description: '',
    request_type: 'document',
  })

  const createMutation = useMutation(async (data: CreateAuditRequest) => {
    return apiClient.post<AuditRequest>(`/audits/${auditId}/requests`, data)
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createMutation.mutate(formData)
      onOpenChange(false)
      setFormData({ title: '', description: '', request_type: 'document' })
      onSuccess()
    } catch {
      // Error handled by mutation
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Add Audit Request</DialogTitle>
          <DialogDescription>
            Create a new request from the auditor.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="title" className="text-right">Title *</Label>
              <Input
                id="title"
                value={formData.title}
                onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                className="col-span-3"
                placeholder="e.g., Access control policy documentation"
                required
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="request_type" className="text-right">Type</Label>
              <Select
                value={formData.request_type || 'document'}
                onValueChange={(value) => setFormData({ ...formData, request_type: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="document">Document Request</SelectItem>
                  <SelectItem value="evidence">Evidence Request</SelectItem>
                  <SelectItem value="interview">Interview</SelectItem>
                  <SelectItem value="walkthrough">Walkthrough</SelectItem>
                  <SelectItem value="clarification">Clarification</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="description" className="text-right">Description</Label>
              <Textarea
                id="description"
                value={formData.description || ''}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                className="col-span-3"
                rows={3}
                placeholder="Describe what the auditor is requesting..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="due_at" className="text-right">Due Date</Label>
              <Input
                id="due_at"
                type="date"
                value={formData.due_at ? formData.due_at.split('T')[0] : ''}
                onChange={(e) => setFormData({ ...formData, due_at: e.target.value ? `${e.target.value}T23:59:59Z` : undefined })}
                className="col-span-3"
              />
            </div>
          </div>
          {createMutation.error && (
            <div className="text-sm text-red-500 mb-4">{createMutation.error.message}</div>
          )}
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
            <Button type="submit" disabled={createMutation.isLoading}>
              {createMutation.isLoading ? 'Creating...' : 'Create Request'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

interface FindingFormProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  auditId: string
  onSuccess: () => void
}

function FindingForm({ open, onOpenChange, auditId, onSuccess }: FindingFormProps) {
  const [formData, setFormData] = useState<CreateAuditFinding>({
    title: '',
    finding_type: 'observation',
    description: '',
    recommendation: '',
    remediation_plan: '',
  })

  const createMutation = useMutation(async (data: CreateAuditFinding) => {
    return apiClient.post<AuditFinding>(`/audits/${auditId}/findings`, data)
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createMutation.mutate(formData)
      onOpenChange(false)
      setFormData({ title: '', finding_type: 'observation', description: '', recommendation: '', remediation_plan: '' })
      onSuccess()
    } catch {
      // Error handled by mutation
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Add Audit Finding</DialogTitle>
          <DialogDescription>
            Record a finding from the audit.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="title" className="text-right">Title *</Label>
              <Input
                id="title"
                value={formData.title}
                onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                className="col-span-3"
                placeholder="e.g., Missing access review documentation"
                required
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="finding_type" className="text-right">Type</Label>
              <Select
                value={formData.finding_type}
                onValueChange={(value: 'observation' | 'exception' | 'deficiency' | 'recommendation') => setFormData({ ...formData, finding_type: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="observation">Observation</SelectItem>
                  <SelectItem value="exception">Exception</SelectItem>
                  <SelectItem value="deficiency">Deficiency</SelectItem>
                  <SelectItem value="recommendation">Recommendation</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="description" className="text-right">Description</Label>
              <Textarea
                id="description"
                value={formData.description || ''}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                className="col-span-3"
                rows={3}
                placeholder="Describe the finding..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="recommendation" className="text-right">Recommendation</Label>
              <Textarea
                id="recommendation"
                value={formData.recommendation || ''}
                onChange={(e) => setFormData({ ...formData, recommendation: e.target.value })}
                className="col-span-3"
                rows={2}
                placeholder="Auditor's recommendation..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="remediation_plan" className="text-right">Remediation Plan</Label>
              <Textarea
                id="remediation_plan"
                value={formData.remediation_plan || ''}
                onChange={(e) => setFormData({ ...formData, remediation_plan: e.target.value })}
                className="col-span-3"
                rows={2}
                placeholder="How will this be addressed..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="remediation_due" className="text-right">Remediation Due</Label>
              <Input
                id="remediation_due"
                type="date"
                value={formData.remediation_due || ''}
                onChange={(e) => setFormData({ ...formData, remediation_due: e.target.value || undefined })}
                className="col-span-3"
              />
            </div>
          </div>
          {createMutation.error && (
            <div className="text-sm text-red-500 mb-4">{createMutation.error.message}</div>
          )}
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>Cancel</Button>
            <Button type="submit" disabled={createMutation.isLoading}>
              {createMutation.isLoading ? 'Creating...' : 'Create Finding'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

interface AuditDetailSheetProps {
  auditId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onUpdate?: () => void
  onDelete?: () => void
}

export function AuditDetailSheet({
  auditId,
  open,
  onOpenChange,
  onUpdate,
  onDelete,
}: AuditDetailSheetProps) {
  const { data: audit, isLoading, refetch } = useAudit(auditId || '')
  const { data: requests, refetch: refetchRequests } = useAuditRequests(auditId || '')
  const { data: findings, refetch: refetchFindings } = useAuditFindings(auditId || '')
  const { data: evidencePackage, isLoading: evidenceLoading } = useAuditEvidencePackage(auditId || '')
  const { data: frameworks } = useFrameworks()
  const [isEditing, setIsEditing] = useState(false)
  const [isRequestOpen, setIsRequestOpen] = useState(false)
  const [isFindingOpen, setIsFindingOpen] = useState(false)
  const [formData, setFormData] = useState<UpdateAudit>({})

  useEffect(() => {
    if (audit) {
      setFormData({
        name: audit.name,
        audit_type: audit.audit_type,
        framework_id: audit.framework_id || undefined,
        auditor_firm: audit.auditor_firm || '',
        auditor_contact: audit.auditor_contact || '',
        period_start: audit.period_start || '',
        period_end: audit.period_end || '',
        status: audit.status,
      })
    }
  }, [audit])

  useEffect(() => {
    if (!open) {
      setIsEditing(false)
    }
  }, [open])

  const updateMutation = useMutation(async (data: UpdateAudit) => {
    return apiClient.put<AuditWithStats>(`/audits/${auditId}`, data)
  })

  const deleteMutation = useMutation(async () => {
    return apiClient.delete(`/audits/${auditId}`)
  })

  const updateFindingMutation = useMutation(async ({ findingId, data }: { findingId: string; data: UpdateAuditFinding }) => {
    return apiClient.put<AuditFinding>(`/audits/${auditId}/findings/${findingId}`, data)
  })

  const createRemediationTaskMutation = useMutation(async (findingId: string) => {
    return apiClient.post(`/audits/${auditId}/findings/${findingId}/remediation-task`, {})
  })

  const handleSave = async () => {
    try {
      await updateMutation.mutate(formData)
      setIsEditing(false)
      refetch()
      onUpdate?.()
    } catch {
      // Error handled by mutation
    }
  }

  const handleDelete = async () => {
    if (!confirm('Are you sure you want to delete this audit? This action cannot be undone.')) return
    try {
      await deleteMutation.mutate(undefined)
      onOpenChange(false)
      onDelete?.()
    } catch {
      // Error handled by mutation
    }
  }

  const handleRequestSuccess = () => {
    refetch()
    refetchRequests()
    onUpdate?.()
  }

  const handleFindingSuccess = () => {
    refetch()
    refetchFindings()
    onUpdate?.()
  }

  const handleCloseFinding = async (findingId: string) => {
    try {
      await updateFindingMutation.mutate({ findingId, data: { status: 'closed' } })
      refetch()
      refetchFindings()
      onUpdate?.()
    } catch {
      // Error handled
    }
  }

  const handleCreateRemediationTask = async (findingId: string) => {
    try {
      await createRemediationTaskMutation.mutate(findingId)
      refetch()
      refetchFindings()
      onUpdate?.()
    } catch {
      // Error handled
    }
  }

  const frameworkName = audit?.framework_id && frameworks
    ? frameworks.find(f => f.id === audit.framework_id)?.name || 'Unknown'
    : null

  return (
    <>
      <Sheet open={open} onOpenChange={onOpenChange}>
        <SheetContent className="sm:max-w-3xl overflow-y-auto">
          {isLoading || !audit ? (
            <div className="flex items-center justify-center h-full">
              <Loading />
            </div>
          ) : (
            <div className="space-y-6">
              <SheetHeader>
                <div className="flex items-center gap-2">
                  <ClipboardCheck className="h-5 w-5 text-primary" />
                  <SheetTitle>{audit.name}</SheetTitle>
                  <Badge variant={statusVariants[audit.status] || 'secondary'}>
                    {formatStatus(audit.status)}
                  </Badge>
                </div>
                <SheetDescription>
                  {audit.auditor_firm ? `${audit.auditor_firm} audit` : 'Audit engagement'}
                  {frameworkName && ` - ${frameworkName}`}
                </SheetDescription>
              </SheetHeader>

              {/* Actions */}
              <div className="flex gap-2">
                {isEditing ? (
                  <>
                    <Button variant="outline" size="sm" onClick={() => setIsEditing(false)}>Cancel</Button>
                    <Button size="sm" onClick={handleSave} disabled={updateMutation.isLoading}>
                      <Save className="mr-2 h-4 w-4" />
                      {updateMutation.isLoading ? 'Saving...' : 'Save'}
                    </Button>
                  </>
                ) : (
                  <>
                    <Button variant="outline" size="sm" onClick={() => setIsEditing(true)}>
                      <Edit2 className="mr-2 h-4 w-4" />
                      Edit
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handleDelete}
                      disabled={deleteMutation.isLoading}
                      className="text-destructive hover:text-destructive"
                    >
                      <Trash2 className="mr-2 h-4 w-4" />
                      Delete
                    </Button>
                  </>
                )}
              </div>

              <Separator />

              {/* Audit Details */}
              <div className="space-y-4">
                <h3 className="font-semibold">Audit Details</h3>

                {isEditing ? (
                  <div className="space-y-4">
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <Label>Name</Label>
                        <Input
                          value={formData.name || ''}
                          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>Type</Label>
                        <Select
                          value={formData.audit_type || ''}
                          onValueChange={(value) => setFormData({ ...formData, audit_type: value })}
                        >
                          <SelectTrigger><SelectValue /></SelectTrigger>
                          <SelectContent>
                            <SelectItem value="external">External</SelectItem>
                            <SelectItem value="internal">Internal</SelectItem>
                            <SelectItem value="certification">Certification</SelectItem>
                            <SelectItem value="compliance">Compliance</SelectItem>
                            <SelectItem value="readiness">Readiness</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <Label>Auditor Firm</Label>
                        <Input
                          value={formData.auditor_firm || ''}
                          onChange={(e) => setFormData({ ...formData, auditor_firm: e.target.value })}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>Auditor Contact</Label>
                        <Input
                          value={formData.auditor_contact || ''}
                          onChange={(e) => setFormData({ ...formData, auditor_contact: e.target.value })}
                        />
                      </div>
                    </div>
                    <div className="grid grid-cols-3 gap-4">
                      <div className="space-y-2">
                        <Label>Period Start</Label>
                        <Input
                          type="date"
                          value={formData.period_start || ''}
                          onChange={(e) => setFormData({ ...formData, period_start: e.target.value })}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>Period End</Label>
                        <Input
                          type="date"
                          value={formData.period_end || ''}
                          onChange={(e) => setFormData({ ...formData, period_end: e.target.value })}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label>Status</Label>
                        <Select
                          value={formData.status || ''}
                          onValueChange={(value) => setFormData({ ...formData, status: value })}
                        >
                          <SelectTrigger><SelectValue /></SelectTrigger>
                          <SelectContent>
                            <SelectItem value="planning">Planning</SelectItem>
                            <SelectItem value="fieldwork">Fieldwork</SelectItem>
                            <SelectItem value="review">Review</SelectItem>
                            <SelectItem value="completed">Completed</SelectItem>
                            <SelectItem value="cancelled">Cancelled</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                    {updateMutation.error && (
                      <div className="text-sm text-red-500">{updateMutation.error.message}</div>
                    )}
                  </div>
                ) : (
                  <div className="space-y-3 text-sm">
                    <div className="grid grid-cols-3 gap-4">
                      <div>
                        <Label className="text-muted-foreground text-xs">Type</Label>
                        <p className="capitalize">{audit.audit_type || '-'}</p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Auditor</Label>
                        <p className="flex items-center gap-1">
                          <Building2 className="h-3 w-3" />
                          {audit.auditor_firm || '-'}
                        </p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Contact</Label>
                        <p>{audit.auditor_contact || '-'}</p>
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <Label className="text-muted-foreground text-xs">Period Start</Label>
                        <p className="flex items-center gap-1">
                          <Calendar className="h-3 w-3" />
                          {formatDate(audit.period_start)}
                        </p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Period End</Label>
                        <p className="flex items-center gap-1">
                          <Calendar className="h-3 w-3" />
                          {formatDate(audit.period_end)}
                        </p>
                      </div>
                    </div>
                    {frameworkName && (
                      <div>
                        <Label className="text-muted-foreground text-xs">Framework</Label>
                        <p>{frameworkName}</p>
                      </div>
                    )}
                  </div>
                )}
              </div>

              <Separator />

              {/* Stats Summary */}
              <div className="grid grid-cols-4 gap-4">
                <div className="text-center p-3 bg-muted/50 rounded-lg">
                  <p className="text-2xl font-bold">{audit.request_count}</p>
                  <p className="text-xs text-muted-foreground">Requests</p>
                </div>
                <div className="text-center p-3 bg-muted/50 rounded-lg">
                  <p className="text-2xl font-bold text-yellow-500">{audit.open_requests}</p>
                  <p className="text-xs text-muted-foreground">Open Requests</p>
                </div>
                <div className="text-center p-3 bg-muted/50 rounded-lg">
                  <p className="text-2xl font-bold">{audit.finding_count}</p>
                  <p className="text-xs text-muted-foreground">Findings</p>
                </div>
                <div className="text-center p-3 bg-muted/50 rounded-lg">
                  <p className="text-2xl font-bold text-red-500">{audit.open_findings}</p>
                  <p className="text-xs text-muted-foreground">Open Findings</p>
                </div>
              </div>

              <Separator />

              {/* Requests, Findings, and Evidence Tabs */}
              <Tabs defaultValue="requests" className="w-full">
                <TabsList className="grid w-full grid-cols-3">
                  <TabsTrigger value="requests" className="flex items-center gap-2">
                    <FileQuestion className="h-4 w-4" />
                    Requests ({requests?.length || 0})
                  </TabsTrigger>
                  <TabsTrigger value="findings" className="flex items-center gap-2">
                    <AlertTriangle className="h-4 w-4" />
                    Findings ({findings?.length || 0})
                  </TabsTrigger>
                  <TabsTrigger value="evidence" className="flex items-center gap-2">
                    <Package className="h-4 w-4" />
                    Evidence ({evidencePackage?.evidence_count || 0})
                  </TabsTrigger>
                </TabsList>

                <TabsContent value="requests" className="space-y-4 mt-4">
                  <div className="flex justify-end">
                    <Button size="sm" onClick={() => setIsRequestOpen(true)}>
                      <Plus className="mr-2 h-4 w-4" />
                      Add Request
                    </Button>
                  </div>

                  {requests && requests.length > 0 ? (
                    <div className="space-y-2">
                      {requests.map((request) => (
                        <div key={request.id} className="border rounded-lg p-3 hover:bg-muted/25">
                          <div className="flex items-center justify-between mb-2">
                            <div className="flex items-center gap-2">
                              <FileQuestion className="h-4 w-4 text-muted-foreground" />
                              <span className="font-medium">{request.title}</span>
                            </div>
                            <Badge variant={requestStatusVariants[request.status] || 'secondary'}>
                              {formatStatus(request.status)}
                            </Badge>
                          </div>
                          {request.description && (
                            <p className="text-sm text-muted-foreground line-clamp-2 mb-2">
                              {request.description}
                            </p>
                          )}
                          <div className="flex items-center justify-between text-xs text-muted-foreground">
                            <span className="capitalize">{request.request_type || 'Request'}</span>
                            {request.due_at && (
                              <span className={`flex items-center gap-1 ${
                                new Date(request.due_at) < new Date() && request.status === 'open'
                                  ? 'text-red-500'
                                  : ''
                              }`}>
                                <Clock className="h-3 w-3" />
                                Due: {formatDate(request.due_at)}
                              </span>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8 border rounded-lg border-dashed">
                      <FileQuestion className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                      <p className="text-sm text-muted-foreground mb-3">No requests yet</p>
                      <Button size="sm" variant="outline" onClick={() => setIsRequestOpen(true)}>
                        <Plus className="mr-2 h-4 w-4" />
                        Add First Request
                      </Button>
                    </div>
                  )}
                </TabsContent>

                <TabsContent value="findings" className="space-y-4 mt-4">
                  <div className="flex justify-end">
                    <Button size="sm" onClick={() => setIsFindingOpen(true)}>
                      <Plus className="mr-2 h-4 w-4" />
                      Add Finding
                    </Button>
                  </div>

                  {findings && findings.length > 0 ? (
                    <div className="space-y-2">
                      {findings.map((finding) => (
                        <div key={finding.id} className="border rounded-lg p-3 hover:bg-muted/25">
                          <div className="flex items-center justify-between mb-2">
                            <div className="flex items-center gap-2">
                              <AlertTriangle className="h-4 w-4 text-muted-foreground" />
                              <span className="font-medium">{finding.title}</span>
                              <Badge variant={findingTypeVariants[finding.finding_type] || 'secondary'} className="text-xs">
                                {formatStatus(finding.finding_type)}
                              </Badge>
                            </div>
                            <div className="flex items-center gap-2">
                              <Badge variant={findingStatusVariants[finding.status] || 'secondary'}>
                                {formatStatus(finding.status)}
                              </Badge>
                              {finding.status !== 'closed' && finding.status !== 'in_remediation' && (
                                <Button
                                  size="sm"
                                  variant="ghost"
                                  onClick={() => handleCreateRemediationTask(finding.id)}
                                  disabled={createRemediationTaskMutation.isLoading}
                                  title="Create remediation task"
                                >
                                  <ListTodo className="h-4 w-4" />
                                </Button>
                              )}
                              {finding.status !== 'closed' && (
                                <Button
                                  size="sm"
                                  variant="ghost"
                                  onClick={() => handleCloseFinding(finding.id)}
                                  disabled={updateFindingMutation.isLoading}
                                  title="Close finding"
                                >
                                  <CheckCircle className="h-4 w-4" />
                                </Button>
                              )}
                            </div>
                          </div>
                          {finding.description && (
                            <p className="text-sm text-muted-foreground line-clamp-2 mb-2">
                              {finding.description}
                            </p>
                          )}
                          {finding.recommendation && (
                            <div className="text-xs bg-muted/50 p-2 rounded mb-2">
                              <span className="font-medium">Recommendation: </span>
                              {finding.recommendation}
                            </div>
                          )}
                          <div className="flex items-center justify-between text-xs text-muted-foreground">
                            <span>Created: {formatDate(finding.created_at)}</span>
                            {finding.remediation_due && (
                              <span className={`flex items-center gap-1 ${
                                new Date(finding.remediation_due) < new Date() && finding.status !== 'closed'
                                  ? 'text-red-500'
                                  : ''
                              }`}>
                                <Clock className="h-3 w-3" />
                                Remediation Due: {formatDate(finding.remediation_due)}
                              </span>
                            )}
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8 border rounded-lg border-dashed">
                      <AlertTriangle className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                      <p className="text-sm text-muted-foreground mb-3">No findings yet</p>
                      <Button size="sm" variant="outline" onClick={() => setIsFindingOpen(true)}>
                        <Plus className="mr-2 h-4 w-4" />
                        Add First Finding
                      </Button>
                    </div>
                  )}
                </TabsContent>

                <TabsContent value="evidence" className="space-y-4 mt-4">
                  {evidenceLoading ? (
                    <div className="flex items-center justify-center py-8">
                      <Loading />
                    </div>
                  ) : evidencePackage && evidencePackage.evidence.length > 0 ? (
                    <>
                      {/* Evidence Package Summary */}
                      <div className="bg-muted/50 rounded-lg p-4">
                        <div className="flex items-center justify-between mb-3">
                          <h4 className="font-semibold flex items-center gap-2">
                            <Package className="h-4 w-4" />
                            Evidence Package
                          </h4>
                          <Button size="sm" variant="outline">
                            <Download className="mr-2 h-4 w-4" />
                            Export Package
                          </Button>
                        </div>
                        <div className="grid grid-cols-3 gap-4 text-sm">
                          <div>
                            <span className="text-muted-foreground">Total Items:</span>
                            <span className="ml-2 font-medium">{evidencePackage.evidence_count}</span>
                          </div>
                          <div>
                            <span className="text-muted-foreground">Total Size:</span>
                            <span className="ml-2 font-medium">
                              {(evidencePackage.total_file_size / 1024 / 1024).toFixed(2)} MB
                            </span>
                          </div>
                          <div>
                            <span className="text-muted-foreground">Generated:</span>
                            <span className="ml-2 font-medium">{formatDateTime(evidencePackage.generated_at)}</span>
                          </div>
                        </div>
                      </div>

                      {/* Evidence Items List */}
                      <div className="space-y-2">
                        {evidencePackage.evidence.map((item) => (
                          <div key={item.id} className="border rounded-lg p-3 hover:bg-muted/25">
                            <div className="flex items-center justify-between mb-2">
                              <div className="flex items-center gap-2">
                                <FileText className="h-4 w-4 text-muted-foreground" />
                                <span className="font-medium">{item.title}</span>
                                <Badge variant="outline" className="text-xs">
                                  {item.evidence_type}
                                </Badge>
                              </div>
                              {item.file_path && (
                                <Button size="sm" variant="ghost">
                                  <Download className="h-4 w-4" />
                                </Button>
                              )}
                            </div>
                            {item.description && (
                              <p className="text-sm text-muted-foreground line-clamp-2 mb-2">
                                {item.description}
                              </p>
                            )}
                            <div className="flex items-center justify-between text-xs text-muted-foreground">
                              <div className="flex items-center gap-3">
                                <span className="capitalize">{item.source}</span>
                                {item.file_size && (
                                  <span className="flex items-center gap-1">
                                    <HardDrive className="h-3 w-3" />
                                    {(item.file_size / 1024).toFixed(1)} KB
                                  </span>
                                )}
                              </div>
                              <span>Collected: {formatDate(item.collected_at)}</span>
                            </div>
                            {(item.linked_controls.length > 0 || item.linked_requests.length > 0) && (
                              <div className="flex flex-wrap gap-1 mt-2">
                                {item.linked_controls.map((control, idx) => (
                                  <Badge key={`ctrl-${idx}`} variant="secondary" className="text-xs">
                                    {control}
                                  </Badge>
                                ))}
                                {item.linked_requests.map((request, idx) => (
                                  <Badge key={`req-${idx}`} variant="outline" className="text-xs">
                                    {request}
                                  </Badge>
                                ))}
                              </div>
                            )}
                          </div>
                        ))}
                      </div>
                    </>
                  ) : (
                    <div className="text-center py-8 border rounded-lg border-dashed">
                      <Package className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                      <p className="text-sm text-muted-foreground mb-2">No evidence collected yet</p>
                      <p className="text-xs text-muted-foreground">
                        Evidence will appear here when linked through framework controls or audit request responses.
                      </p>
                    </div>
                  )}
                </TabsContent>
              </Tabs>
            </div>
          )}
        </SheetContent>
      </Sheet>

      {auditId && (
        <>
          <RequestForm
            open={isRequestOpen}
            onOpenChange={setIsRequestOpen}
            auditId={auditId}
            onSuccess={handleRequestSuccess}
          />
          <FindingForm
            open={isFindingOpen}
            onOpenChange={setIsFindingOpen}
            auditId={auditId}
            onSuccess={handleFindingSuccess}
          />
        </>
      )}
    </>
  )
}
