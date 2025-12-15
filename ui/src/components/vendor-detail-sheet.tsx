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
import { Loading } from "@/components/loading"
import {
  Save,
  Trash2,
  Edit2,
  Plus,
  FileText,
  ClipboardCheck,
  ExternalLink,
  Calendar,
  Shield,
  Building2,
  Globe,
} from "lucide-react"
import { useVendor, useVendorAssessments, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type {
  VendorWithAssessment,
  UpdateVendor,
  VendorAssessment,
  CreateVendorAssessment,
} from '@/types'
import { formatStatus, formatDate } from '@/types'

const criticalityVariants: Record<string, 'destructive' | 'warning' | 'secondary' | 'outline'> = {
  critical: 'destructive',
  high: 'warning',
  medium: 'secondary',
  low: 'outline',
}

const statusVariants: Record<string, 'success' | 'warning' | 'secondary'> = {
  active: 'success',
  under_review: 'warning',
  inactive: 'secondary',
}

const riskRatingVariants: Record<string, 'destructive' | 'warning' | 'secondary' | 'success'> = {
  critical: 'destructive',
  high: 'destructive',
  medium: 'warning',
  low: 'success',
}

interface AssessmentFormProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  vendorId: string
  onSuccess: () => void
}

function AssessmentForm({
  open,
  onOpenChange,
  vendorId,
  onSuccess,
}: AssessmentFormProps) {
  const [formData, setFormData] = useState<CreateVendorAssessment>({
    assessment_type: 'annual',
    risk_rating: 'medium',
    findings: '',
    recommendations: '',
  })

  const createMutation = useMutation(async (data: CreateVendorAssessment) => {
    return apiClient.post<VendorAssessment>(`/vendors/${vendorId}/assessments`, data)
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createMutation.mutate(formData)
      onOpenChange(false)
      setFormData({
        assessment_type: 'annual',
        risk_rating: 'medium',
        findings: '',
        recommendations: '',
      })
      onSuccess()
    } catch {
      // Error handled by mutation
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Add Assessment</DialogTitle>
          <DialogDescription>
            Record a new security assessment for this vendor.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="assessment_type" className="text-right">
                Type
              </Label>
              <Select
                value={formData.assessment_type}
                onValueChange={(value) => setFormData({ ...formData, assessment_type: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="initial">Initial</SelectItem>
                  <SelectItem value="annual">Annual</SelectItem>
                  <SelectItem value="incident">Incident Response</SelectItem>
                  <SelectItem value="renewal">Contract Renewal</SelectItem>
                  <SelectItem value="other">Other</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="risk_rating" className="text-right">
                Risk Rating
              </Label>
              <Select
                value={formData.risk_rating}
                onValueChange={(value) => setFormData({ ...formData, risk_rating: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="critical">Critical</SelectItem>
                  <SelectItem value="high">High</SelectItem>
                  <SelectItem value="medium">Medium</SelectItem>
                  <SelectItem value="low">Low</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="findings" className="text-right">
                Findings
              </Label>
              <Textarea
                id="findings"
                value={formData.findings || ''}
                onChange={(e) => setFormData({ ...formData, findings: e.target.value })}
                className="col-span-3"
                rows={3}
                placeholder="Document any security findings..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="recommendations" className="text-right">
                Recommendations
              </Label>
              <Textarea
                id="recommendations"
                value={formData.recommendations || ''}
                onChange={(e) => setFormData({ ...formData, recommendations: e.target.value })}
                className="col-span-3"
                rows={3}
                placeholder="Recommended actions or mitigations..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="next_assessment" className="text-right">
                Next Assessment
              </Label>
              <Input
                id="next_assessment"
                type="date"
                value={formData.next_assessment_date || ''}
                onChange={(e) => setFormData({ ...formData, next_assessment_date: e.target.value })}
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
              {createMutation.isLoading ? 'Saving...' : 'Save Assessment'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

interface VendorDetailSheetProps {
  vendorId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onUpdate?: () => void
  onDelete?: () => void
}

export function VendorDetailSheet({
  vendorId,
  open,
  onOpenChange,
  onUpdate,
  onDelete,
}: VendorDetailSheetProps) {
  const { data: vendor, isLoading, refetch } = useVendor(vendorId || '')
  const { data: assessments, refetch: refetchAssessments } = useVendorAssessments(vendorId || '')
  const [isEditing, setIsEditing] = useState(false)
  const [isAssessmentOpen, setIsAssessmentOpen] = useState(false)
  const [formData, setFormData] = useState<UpdateVendor>({})

  useEffect(() => {
    if (vendor) {
      setFormData({
        name: vendor.name,
        description: vendor.description || '',
        category: vendor.category || '',
        criticality: vendor.criticality || '',
        data_classification: vendor.data_classification || '',
        status: vendor.status,
        website: vendor.website || '',
        contract_start: vendor.contract_start || '',
        contract_end: vendor.contract_end || '',
      })
    }
  }, [vendor])

  useEffect(() => {
    if (!open) {
      setIsEditing(false)
    }
  }, [open])

  const updateMutation = useMutation(async (data: UpdateVendor) => {
    return apiClient.put<VendorWithAssessment>(`/vendors/${vendorId}`, data)
  })

  const deleteMutation = useMutation(async () => {
    return apiClient.delete(`/vendors/${vendorId}`)
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
    if (!confirm('Are you sure you want to delete this vendor? This action cannot be undone.')) return
    try {
      await deleteMutation.mutate(undefined)
      onOpenChange(false)
      onDelete?.()
    } catch {
      // Error handled by mutation
    }
  }

  const handleAssessmentSuccess = () => {
    refetch()
    refetchAssessments()
    onUpdate?.()
  }

  return (
    <>
      <Sheet open={open} onOpenChange={onOpenChange}>
        <SheetContent className="sm:max-w-2xl overflow-y-auto">
          {isLoading || !vendor ? (
            <div className="flex items-center justify-center h-full">
              <Loading />
            </div>
          ) : (
            <div className="space-y-6">
              <SheetHeader>
                <div className="flex items-center gap-2">
                  <Building2 className="h-5 w-5 text-primary" />
                  <SheetTitle>{vendor.name}</SheetTitle>
                  <Badge variant={statusVariants[vendor.status] || 'secondary'}>
                    {formatStatus(vendor.status)}
                  </Badge>
                </div>
                <SheetDescription>
                  {vendor.description || 'No description provided'}
                </SheetDescription>
              </SheetHeader>

              {/* Actions */}
              <div className="flex gap-2">
                {isEditing ? (
                  <>
                    <Button variant="outline" size="sm" onClick={() => setIsEditing(false)}>
                      Cancel
                    </Button>
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

              {/* Vendor Details */}
              <div className="space-y-4">
                <h3 className="font-semibold">Vendor Details</h3>

                {isEditing ? (
                  <div className="space-y-4">
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <Label htmlFor="name">Name</Label>
                        <Input
                          id="name"
                          value={formData.name || ''}
                          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="category">Category</Label>
                        <Select
                          value={formData.category || ''}
                          onValueChange={(value) => setFormData({ ...formData, category: value })}
                        >
                          <SelectTrigger>
                            <SelectValue placeholder="Select category" />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="saas">SaaS</SelectItem>
                            <SelectItem value="infrastructure">Infrastructure</SelectItem>
                            <SelectItem value="security">Security</SelectItem>
                            <SelectItem value="consulting">Consulting</SelectItem>
                            <SelectItem value="payment">Payment Processing</SelectItem>
                            <SelectItem value="hr">HR Services</SelectItem>
                            <SelectItem value="legal">Legal</SelectItem>
                            <SelectItem value="other">Other</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="description">Description</Label>
                      <Textarea
                        id="description"
                        value={formData.description || ''}
                        onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                        rows={2}
                      />
                    </div>
                    <div className="grid grid-cols-3 gap-4">
                      <div className="space-y-2">
                        <Label>Criticality</Label>
                        <Select
                          value={formData.criticality || ''}
                          onValueChange={(value) => setFormData({ ...formData, criticality: value })}
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="critical">Critical</SelectItem>
                            <SelectItem value="high">High</SelectItem>
                            <SelectItem value="medium">Medium</SelectItem>
                            <SelectItem value="low">Low</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <div className="space-y-2">
                        <Label>Data Access</Label>
                        <Select
                          value={formData.data_classification || ''}
                          onValueChange={(value) => setFormData({ ...formData, data_classification: value })}
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="public">Public</SelectItem>
                            <SelectItem value="internal">Internal</SelectItem>
                            <SelectItem value="confidential">Confidential</SelectItem>
                            <SelectItem value="restricted">Restricted</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <div className="space-y-2">
                        <Label>Status</Label>
                        <Select
                          value={formData.status || ''}
                          onValueChange={(value) => setFormData({ ...formData, status: value })}
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="active">Active</SelectItem>
                            <SelectItem value="inactive">Inactive</SelectItem>
                            <SelectItem value="under_review">Under Review</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="website">Website</Label>
                      <Input
                        id="website"
                        value={formData.website || ''}
                        onChange={(e) => setFormData({ ...formData, website: e.target.value })}
                        type="url"
                        placeholder="https://example.com"
                      />
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <Label htmlFor="contract_start">Contract Start</Label>
                        <Input
                          id="contract_start"
                          type="date"
                          value={formData.contract_start || ''}
                          onChange={(e) => setFormData({ ...formData, contract_start: e.target.value })}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="contract_end">Contract End</Label>
                        <Input
                          id="contract_end"
                          type="date"
                          value={formData.contract_end || ''}
                          onChange={(e) => setFormData({ ...formData, contract_end: e.target.value })}
                        />
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
                        <Label className="text-muted-foreground text-xs">Category</Label>
                        <p className="capitalize">{vendor.category || '-'}</p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Criticality</Label>
                        {vendor.criticality ? (
                          <Badge variant={criticalityVariants[vendor.criticality] || 'secondary'} className="mt-1">
                            {formatStatus(vendor.criticality)}
                          </Badge>
                        ) : (
                          <p>-</p>
                        )}
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Data Access</Label>
                        <p className="capitalize">{vendor.data_classification || '-'}</p>
                      </div>
                    </div>
                    {vendor.website && (
                      <div>
                        <Label className="text-muted-foreground text-xs">Website</Label>
                        <a
                          href={vendor.website}
                          target="_blank"
                          rel="noopener noreferrer"
                          className="flex items-center gap-1 text-primary hover:underline"
                        >
                          <Globe className="h-3 w-3" />
                          {vendor.website}
                          <ExternalLink className="h-3 w-3" />
                        </a>
                      </div>
                    )}
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <Label className="text-muted-foreground text-xs">Contract Start</Label>
                        <p className="flex items-center gap-1">
                          <Calendar className="h-3 w-3" />
                          {formatDate(vendor.contract_start)}
                        </p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Contract End</Label>
                        <p className={`flex items-center gap-1 ${
                          vendor.contract_end && new Date(vendor.contract_end) < new Date(Date.now() + 90 * 24 * 60 * 60 * 1000)
                            ? 'text-orange-500'
                            : ''
                        }`}>
                          <Calendar className="h-3 w-3" />
                          {formatDate(vendor.contract_end)}
                        </p>
                      </div>
                    </div>
                  </div>
                )}
              </div>

              <Separator />

              {/* Risk Assessment */}
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold flex items-center gap-2">
                    <Shield className="h-4 w-4" />
                    Risk Assessments
                    <Badge variant="secondary">{assessments?.length || 0}</Badge>
                  </h3>
                  <Button size="sm" onClick={() => setIsAssessmentOpen(true)}>
                    <Plus className="mr-2 h-4 w-4" />
                    Add Assessment
                  </Button>
                </div>

                {vendor.last_risk_rating && (
                  <div className="bg-muted/50 rounded-lg p-4">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-muted-foreground">Current Risk Rating</p>
                        <Badge variant={riskRatingVariants[vendor.last_risk_rating] || 'secondary'} className="mt-1">
                          {formatStatus(vendor.last_risk_rating)}
                        </Badge>
                      </div>
                      <div className="text-right">
                        <p className="text-sm text-muted-foreground">Last Assessed</p>
                        <p className="text-sm">{formatDate(vendor.last_assessment_date)}</p>
                      </div>
                      {vendor.next_assessment_date && (
                        <div className="text-right">
                          <p className="text-sm text-muted-foreground">Next Assessment</p>
                          <p className={`text-sm ${
                            new Date(vendor.next_assessment_date) < new Date()
                              ? 'text-red-500'
                              : ''
                          }`}>
                            {formatDate(vendor.next_assessment_date)}
                          </p>
                        </div>
                      )}
                    </div>
                  </div>
                )}

                {assessments && assessments.length > 0 ? (
                  <div className="space-y-2">
                    {assessments.map((assessment) => (
                      <div
                        key={assessment.id}
                        className="border rounded-lg p-3 hover:bg-muted/25"
                      >
                        <div className="flex items-center justify-between mb-2">
                          <div className="flex items-center gap-2">
                            <ClipboardCheck className="h-4 w-4 text-muted-foreground" />
                            <span className="font-medium capitalize">{assessment.assessment_type} Assessment</span>
                          </div>
                          <Badge variant={riskRatingVariants[assessment.risk_rating || 'medium'] || 'secondary'}>
                            {formatStatus(assessment.risk_rating || 'N/A')}
                          </Badge>
                        </div>
                        <p className="text-xs text-muted-foreground">
                          Assessed on {formatDate(assessment.assessed_at)}
                        </p>
                        {assessment.findings && (
                          <p className="text-sm mt-2 text-muted-foreground line-clamp-2">
                            {assessment.findings}
                          </p>
                        )}
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="text-center py-6 border rounded-lg border-dashed">
                    <ClipboardCheck className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                    <p className="text-sm text-muted-foreground mb-3">
                      No assessments recorded yet
                    </p>
                    <Button size="sm" variant="outline" onClick={() => setIsAssessmentOpen(true)}>
                      <Plus className="mr-2 h-4 w-4" />
                      Add First Assessment
                    </Button>
                  </div>
                )}
              </div>

              <Separator />

              {/* Documents placeholder */}
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold flex items-center gap-2">
                    <FileText className="h-4 w-4" />
                    Documents
                  </h3>
                  <Button size="sm" variant="outline" disabled>
                    <Plus className="mr-2 h-4 w-4" />
                    Upload
                  </Button>
                </div>
                <div className="text-center py-6 border rounded-lg border-dashed">
                  <FileText className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                  <p className="text-sm text-muted-foreground">
                    SOC 2 reports, contracts, and other documents
                  </p>
                </div>
              </div>
            </div>
          )}
        </SheetContent>
      </Sheet>

      {vendorId && (
        <AssessmentForm
          open={isAssessmentOpen}
          onOpenChange={setIsAssessmentOpen}
          vendorId={vendorId}
          onSuccess={handleAssessmentSuccess}
        />
      )}
    </>
  )
}
