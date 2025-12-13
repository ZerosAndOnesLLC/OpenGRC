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
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs"
import { Separator } from "@/components/ui/separator"
import { Loading } from "@/components/loading"
import {
  Save,
  Trash2,
  Edit2,
  FileText,
  CheckCircle2,
  Clock,
  Users,
  History,
  Send,
} from "lucide-react"
import { useApi, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'

interface Policy {
  id: string
  code: string
  title: string
  category: string | null
  content: string | null
  version: number
  status: string
  owner_id: string | null
  approver_id: string | null
  effective_date: string | null
  review_date: string | null
  created_at: string
  updated_at: string
  acknowledgment_count: number
  pending_acknowledgments: number
}

interface PolicyVersion {
  id: string
  version: number
  content: string | null
  change_summary: string | null
  changed_by: string | null
  created_at: string
}

interface PolicyAcknowledgment {
  id: string
  user_id: string
  policy_version: number
  acknowledged_at: string
  ip_address: string | null
}

interface UpdatePolicy {
  code?: string
  title?: string
  category?: string
  content?: string
  status?: string
  effective_date?: string | null
  review_date?: string | null
  change_summary?: string
}

const statusColors: Record<string, 'secondary' | 'warning' | 'success' | 'destructive'> = {
  draft: 'secondary',
  pending_approval: 'warning',
  published: 'success',
  archived: 'destructive',
}

const categoryLabels: Record<string, string> = {
  security: "Security",
  it: "IT",
  compliance: "Compliance",
  privacy: "Privacy",
  hr: "HR",
  operations: "Operations",
  business: "Business",
  other: "Other",
}

interface PolicyDetailSheetProps {
  policyId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onUpdate?: () => void
  onDelete?: () => void
}

export function PolicyDetailSheet({
  policyId,
  open,
  onOpenChange,
  onUpdate,
  onDelete,
}: PolicyDetailSheetProps) {
  const { data: policy, isLoading, refetch } = useApi<Policy>(
    `/policies/${policyId}`,
    { enabled: !!policyId && open }
  )
  const { data: versions } = useApi<PolicyVersion[]>(
    `/policies/${policyId}/versions`,
    { enabled: !!policyId && open }
  )
  const { data: acknowledgments, refetch: refetchAcks } = useApi<PolicyAcknowledgment[]>(
    `/policies/${policyId}/acknowledgments`,
    { enabled: !!policyId && open }
  )

  const [isEditing, setIsEditing] = useState(false)
  const [formData, setFormData] = useState<UpdatePolicy>({})
  const [activeTab, setActiveTab] = useState('content')

  useEffect(() => {
    if (policy) {
      setFormData({
        code: policy.code,
        title: policy.title,
        category: policy.category || '',
        content: policy.content || '',
        status: policy.status,
        effective_date: policy.effective_date,
        review_date: policy.review_date,
      })
    }
  }, [policy])

  useEffect(() => {
    if (!open) {
      setIsEditing(false)
      setActiveTab('content')
    }
  }, [open])

  const updateMutation = useMutation(async (data: UpdatePolicy) => {
    return apiClient.put<Policy>(`/policies/${policyId}`, data)
  })

  const deleteMutation = useMutation(async () => {
    return apiClient.delete(`/policies/${policyId}`)
  })

  const acknowledgeMutation = useMutation(async () => {
    return apiClient.post<PolicyAcknowledgment>(`/policies/${policyId}/acknowledge`)
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
    if (!confirm('Are you sure you want to delete this policy?')) return
    try {
      await deleteMutation.mutate(undefined)
      onOpenChange(false)
      onDelete?.()
    } catch {
      // Error handled by mutation
    }
  }

  const handleAcknowledge = async () => {
    try {
      await acknowledgeMutation.mutate(undefined)
      refetchAcks()
      refetch()
      onUpdate?.()
    } catch {
      // Error handled by mutation
    }
  }

  const handlePublish = async () => {
    if (!confirm('Publish this policy? It will become visible to all employees for acknowledgment.')) return
    try {
      await updateMutation.mutate({ status: 'published' })
      refetch()
      onUpdate?.()
    } catch {
      // Error handled by mutation
    }
  }

  // Check if current user has already acknowledged this version
  // In a real app, you'd compare against the current user's ID from auth context
  const hasAcknowledged = acknowledgments && acknowledgments.length > 0 &&
    policy && acknowledgments.some(a => a.policy_version === policy.version)

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="sm:max-w-2xl overflow-y-auto">
        {isLoading || !policy ? (
          <div className="flex items-center justify-center h-full">
            <Loading />
          </div>
        ) : (
          <div className="space-y-6">
            <SheetHeader>
              <div className="flex items-center gap-2">
                <FileText className="h-5 w-5 text-primary" />
                <SheetTitle className="font-mono">{policy.code}</SheetTitle>
                <Badge variant={statusColors[policy.status] || 'secondary'}>
                  {policy.status.replace('_', ' ')}
                </Badge>
              </div>
              <SheetDescription>{policy.title}</SheetDescription>
            </SheetHeader>

            {/* Actions */}
            <div className="flex gap-2 flex-wrap">
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
                  {policy.status === 'draft' && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={handlePublish}
                      disabled={updateMutation.isLoading}
                    >
                      <Send className="mr-2 h-4 w-4" />
                      Publish
                    </Button>
                  )}
                  {policy.status === 'published' && !hasAcknowledged && (
                    <Button
                      size="sm"
                      onClick={handleAcknowledge}
                      disabled={acknowledgeMutation.isLoading}
                    >
                      <CheckCircle2 className="mr-2 h-4 w-4" />
                      {acknowledgeMutation.isLoading ? 'Acknowledging...' : 'Acknowledge'}
                    </Button>
                  )}
                  {hasAcknowledged && (
                    <Badge variant="outline" className="bg-green-50 text-green-700 border-green-200">
                      <CheckCircle2 className="mr-1 h-3 w-3" />
                      Acknowledged
                    </Badge>
                  )}
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleDelete}
                    disabled={deleteMutation.isLoading}
                    className="text-destructive hover:text-destructive ml-auto"
                  >
                    <Trash2 className="mr-2 h-4 w-4" />
                    Delete
                  </Button>
                </>
              )}
            </div>

            <Separator />

            <Tabs value={activeTab} onValueChange={setActiveTab}>
              <TabsList className="grid w-full grid-cols-3">
                <TabsTrigger value="content">
                  <FileText className="mr-2 h-4 w-4" />
                  Content
                </TabsTrigger>
                <TabsTrigger value="acknowledgments">
                  <Users className="mr-2 h-4 w-4" />
                  Acks ({policy.acknowledgment_count})
                </TabsTrigger>
                <TabsTrigger value="versions">
                  <History className="mr-2 h-4 w-4" />
                  History
                </TabsTrigger>
              </TabsList>

              <TabsContent value="content" className="space-y-4 mt-4">
                {isEditing ? (
                  <div className="space-y-4">
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <Label htmlFor="code">Code</Label>
                        <Input
                          id="code"
                          value={formData.code || ''}
                          onChange={(e) => setFormData({ ...formData, code: e.target.value })}
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
                            {Object.entries(categoryLabels).map(([key, label]) => (
                              <SelectItem key={key} value={key}>{label}</SelectItem>
                            ))}
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="title">Title</Label>
                      <Input
                        id="title"
                        value={formData.title || ''}
                        onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                      />
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <Label htmlFor="effective_date">Effective Date</Label>
                        <Input
                          id="effective_date"
                          type="date"
                          value={formData.effective_date?.split('T')[0] || ''}
                          onChange={(e) => setFormData({ ...formData, effective_date: e.target.value })}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="review_date">Review Date</Label>
                        <Input
                          id="review_date"
                          type="date"
                          value={formData.review_date?.split('T')[0] || ''}
                          onChange={(e) => setFormData({ ...formData, review_date: e.target.value })}
                        />
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="status">Status</Label>
                      <Select
                        value={formData.status || ''}
                        onValueChange={(value) => setFormData({ ...formData, status: value })}
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="draft">Draft</SelectItem>
                          <SelectItem value="pending_approval">Pending Approval</SelectItem>
                          <SelectItem value="published">Published</SelectItem>
                          <SelectItem value="archived">Archived</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="content">Content (Markdown)</Label>
                      <Textarea
                        id="content"
                        value={formData.content || ''}
                        onChange={(e) => setFormData({ ...formData, content: e.target.value })}
                        rows={15}
                        className="font-mono text-sm"
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="change_summary">Change Summary</Label>
                      <Input
                        id="change_summary"
                        value={formData.change_summary || ''}
                        onChange={(e) => setFormData({ ...formData, change_summary: e.target.value })}
                        placeholder="Describe what changed..."
                      />
                    </div>
                    {updateMutation.error && (
                      <div className="text-sm text-red-500">{updateMutation.error.message}</div>
                    )}
                  </div>
                ) : (
                  <div className="space-y-4">
                    <div className="grid grid-cols-2 gap-4 text-sm">
                      <div>
                        <Label className="text-muted-foreground text-xs">Category</Label>
                        <p>{policy.category ? categoryLabels[policy.category] || policy.category : '-'}</p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Version</Label>
                        <p>v{policy.version}</p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Effective Date</Label>
                        <p>{policy.effective_date ? new Date(policy.effective_date).toLocaleDateString() : '-'}</p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Review Date</Label>
                        <p className={policy.review_date && new Date(policy.review_date) < new Date() ? 'text-destructive' : ''}>
                          {policy.review_date ? new Date(policy.review_date).toLocaleDateString() : '-'}
                        </p>
                      </div>
                    </div>
                    <Separator />
                    <div>
                      <Label className="text-muted-foreground text-xs mb-2 block">Policy Content</Label>
                      {policy.content ? (
                        <div className="prose prose-sm dark:prose-invert max-w-none border rounded-lg p-4 bg-muted/25">
                          <pre className="whitespace-pre-wrap font-sans text-sm">{policy.content}</pre>
                        </div>
                      ) : (
                        <p className="text-muted-foreground text-sm">No content available.</p>
                      )}
                    </div>
                  </div>
                )}
              </TabsContent>

              <TabsContent value="acknowledgments" className="mt-4">
                <div className="space-y-4">
                  <div className="flex items-center justify-between">
                    <div className="text-sm text-muted-foreground">
                      {policy.acknowledgment_count} total acknowledgments
                      {policy.pending_acknowledgments > 0 && (
                        <span className="text-yellow-600 ml-2">
                          ({policy.pending_acknowledgments} pending)
                        </span>
                      )}
                    </div>
                  </div>

                  {acknowledgments && acknowledgments.length > 0 ? (
                    <div className="border rounded-lg divide-y">
                      {acknowledgments.map((ack) => (
                        <div key={ack.id} className="p-3 flex items-center justify-between">
                          <div className="flex items-center gap-2">
                            <CheckCircle2 className="h-4 w-4 text-green-500" />
                            <span className="text-sm font-mono">{ack.user_id.slice(0, 8)}...</span>
                          </div>
                          <div className="text-sm text-muted-foreground flex items-center gap-2">
                            <span>v{ack.policy_version}</span>
                            <Clock className="h-3 w-3" />
                            <span>{new Date(ack.acknowledged_at).toLocaleString()}</span>
                          </div>
                        </div>
                      ))}
                    </div>
                  ) : (
                    <div className="text-center py-8 border rounded-lg border-dashed">
                      <Users className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                      <p className="text-sm text-muted-foreground">
                        No acknowledgments yet
                      </p>
                      {policy.status !== 'published' && (
                        <p className="text-xs text-muted-foreground mt-1">
                          Publish this policy to enable acknowledgments
                        </p>
                      )}
                    </div>
                  )}
                </div>
              </TabsContent>

              <TabsContent value="versions" className="mt-4">
                {versions && versions.length > 0 ? (
                  <div className="border rounded-lg divide-y">
                    {versions.map((ver) => (
                      <div key={ver.id} className="p-3">
                        <div className="flex items-center justify-between mb-1">
                          <span className="font-medium text-sm">Version {ver.version}</span>
                          <span className="text-xs text-muted-foreground">
                            {new Date(ver.created_at).toLocaleString()}
                          </span>
                        </div>
                        {ver.change_summary && (
                          <p className="text-sm text-muted-foreground">{ver.change_summary}</p>
                        )}
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="text-center py-8 border rounded-lg border-dashed">
                    <History className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                    <p className="text-sm text-muted-foreground">
                      No version history available
                    </p>
                  </div>
                )}
              </TabsContent>
            </Tabs>
          </div>
        )}
      </SheetContent>
    </Sheet>
  )
}
