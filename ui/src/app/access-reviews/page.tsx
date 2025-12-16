'use client'

import { useState } from 'react'
import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Progress } from "@/components/ui/progress"
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
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Loading } from "@/components/loading"
import {
  Plus,
  Search,
  Filter,
  Users,
  Clock,
  CheckCircle2,
  XCircle,
  AlertTriangle,
  ShieldAlert,
  Shield,
  RefreshCw,
  Download,
  Play,
  UserX,
  Key,
} from "lucide-react"
import {
  useAccessReviewCampaigns,
  useAccessReviewStats,
  useAccessReviewItems,
  useMutation,
  useIntegrations,
} from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type {
  CampaignWithStats,
  CreateAccessReviewCampaign,
  AccessReviewItem,
  AccessReviewStats,
  ReviewDecision,
  BulkReviewDecision,
} from '@/types'
import { formatStatus, formatDate, formatRelativeTime } from '@/types'

const statusVariants: Record<string, 'default' | 'secondary' | 'success' | 'warning' | 'destructive'> = {
  draft: 'secondary',
  active: 'warning',
  completed: 'success',
  cancelled: 'destructive',
}

const reviewStatusVariants: Record<string, 'default' | 'secondary' | 'success' | 'destructive' | 'warning'> = {
  pending: 'secondary',
  approved: 'success',
  revoked: 'destructive',
  flagged: 'warning',
}

const riskVariants: Record<string, 'default' | 'secondary' | 'warning' | 'destructive'> = {
  high: 'destructive',
  medium: 'warning',
  low: 'secondary',
}

function StatsCards({ stats }: { stats: AccessReviewStats | null }) {
  if (!stats) return null

  return (
    <div className="grid grid-cols-2 md:grid-cols-6 gap-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Total Campaigns</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Users className="h-5 w-5 text-primary" />
            <span className="text-2xl font-bold">{stats.total_campaigns}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Active</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Clock className="h-5 w-5 text-yellow-500" />
            <span className="text-2xl font-bold">{stats.active_campaigns}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Pending Reviews</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-orange-500" />
            <span className="text-2xl font-bold">{stats.pending_reviews}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">High Risk Users</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <ShieldAlert className="h-5 w-5 text-red-500" />
            <span className="text-2xl font-bold">{stats.high_risk_users}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Admin Users</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Key className="h-5 w-5 text-purple-500" />
            <span className="text-2xl font-bold">{stats.admin_users}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">No MFA</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-orange-500" />
            <span className="text-2xl font-bold">{stats.users_without_mfa}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function CreateCampaignDialog({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}) {
  const [formData, setFormData] = useState<CreateAccessReviewCampaign>({
    name: '',
    description: '',
    review_type: 'periodic',
  })

  const { data: integrationsData } = useIntegrations({ status: 'active' })
  const integrations = integrationsData?.data || []

  // Filter to identity provider integrations
  const idpIntegrations = integrations.filter(i =>
    ['okta', 'google_workspace', 'azure_ad', 'github'].includes(i.integration.integration_type)
  )

  const createMutation = useMutation(async (data: CreateAccessReviewCampaign) => {
    return apiClient.post<CampaignWithStats>('/access-reviews/campaigns', data)
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createMutation.mutate(formData)
      onOpenChange(false)
      setFormData({ name: '', description: '', review_type: 'periodic' })
      onSuccess()
    } catch {
      // Error handled by mutation
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>New Access Review Campaign</DialogTitle>
          <DialogDescription>
            Create a new user access review campaign to certify user access rights.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="name" className="text-right">Name *</Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="col-span-3"
                placeholder="e.g., Q4 2024 User Access Review"
                required
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="description" className="text-right">Description</Label>
              <Textarea
                id="description"
                value={formData.description || ''}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                className="col-span-3"
                placeholder="Describe the purpose of this review..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="review_type" className="text-right">Review Type</Label>
              <Select
                value={formData.review_type || 'periodic'}
                onValueChange={(value) => setFormData({ ...formData, review_type: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="periodic">Periodic Review</SelectItem>
                  <SelectItem value="termination">Termination Review</SelectItem>
                  <SelectItem value="role_change">Role Change Review</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="integration" className="text-right">Integration</Label>
              <Select
                value={formData.integration_id || ''}
                onValueChange={(value) => {
                  const integration = idpIntegrations.find(i => i.integration.id === value)
                  setFormData({
                    ...formData,
                    integration_id: value || undefined,
                    integration_type: integration?.integration.integration_type,
                  })
                }}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select integration (optional)" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="">None (manual import)</SelectItem>
                  {idpIntegrations.map((i) => (
                    <SelectItem key={i.integration.id} value={i.integration.id}>
                      {i.integration.name} ({formatStatus(i.integration.integration_type)})
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="due_at" className="text-right">Due Date</Label>
              <Input
                id="due_at"
                type="date"
                value={formData.due_at || ''}
                onChange={(e) => setFormData({ ...formData, due_at: e.target.value })}
                className="col-span-3"
              />
            </div>
          </div>
          {createMutation.error && (
            <div className="text-sm text-red-500 mb-4">{createMutation.error.message}</div>
          )}
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={createMutation.isLoading}>
              {createMutation.isLoading ? 'Creating...' : 'Create Campaign'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

function CampaignDetailSheet({
  campaign,
  open,
  onOpenChange,
  onUpdate,
}: {
  campaign: CampaignWithStats | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onUpdate: () => void
}) {
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [riskFilter, setRiskFilter] = useState<string>('')
  const [search, setSearch] = useState('')
  const [selectedItems, setSelectedItems] = useState<Set<string>>(new Set())

  const query: Record<string, string | number | boolean> = {}
  if (statusFilter) query.review_status = statusFilter
  if (riskFilter) query.risk_level = riskFilter
  if (search) query.search = search

  const { data: items, isLoading, refetch } = useAccessReviewItems(
    campaign?.id || '',
    query
  )

  const syncMutation = useMutation(async () => {
    return apiClient.post<number>(`/access-reviews/campaigns/${campaign?.id}/sync`)
  })

  const reviewMutation = useMutation(async (data: { itemId: string; decision: ReviewDecision }) => {
    return apiClient.post<AccessReviewItem>(
      `/access-reviews/campaigns/${campaign?.id}/items/${data.itemId}/review`,
      data.decision
    )
  })

  const bulkReviewMutation = useMutation(async (data: BulkReviewDecision) => {
    return apiClient.post<number>(
      `/access-reviews/campaigns/${campaign?.id}/bulk-review`,
      data
    )
  })

  const startCampaignMutation = useMutation(async () => {
    return apiClient.put<CampaignWithStats>(
      `/access-reviews/campaigns/${campaign?.id}`,
      { status: 'active' }
    )
  })

  const completeCampaignMutation = useMutation(async () => {
    return apiClient.put<CampaignWithStats>(
      `/access-reviews/campaigns/${campaign?.id}`,
      { status: 'completed' }
    )
  })

  const handleSync = async () => {
    try {
      await syncMutation.mutate(undefined)
      refetch()
      onUpdate()
    } catch {
      // Error handled by mutation
    }
  }

  const handleReview = async (itemId: string, status: 'approved' | 'revoked') => {
    try {
      await reviewMutation.mutate({ itemId, decision: { status } })
      refetch()
      onUpdate()
    } catch {
      // Error handled
    }
  }

  const handleBulkReview = async (status: 'approved' | 'revoked') => {
    if (selectedItems.size === 0) return
    try {
      await bulkReviewMutation.mutate({
        item_ids: Array.from(selectedItems),
        status,
      })
      setSelectedItems(new Set())
      refetch()
      onUpdate()
    } catch {
      // Error handled
    }
  }

  const handleStartCampaign = async () => {
    try {
      await startCampaignMutation.mutate(undefined)
      onUpdate()
    } catch {
      // Error handled
    }
  }

  const handleCompleteCampaign = async () => {
    try {
      await completeCampaignMutation.mutate(undefined)
      onUpdate()
    } catch {
      // Error handled
    }
  }

  const downloadCertification = () => {
    if (campaign) {
      window.open(`/api/v1/access-reviews/campaigns/${campaign.id}/certification/csv`, '_blank')
    }
  }

  const toggleSelectAll = () => {
    if (!items) return
    if (selectedItems.size === items.length) {
      setSelectedItems(new Set())
    } else {
      setSelectedItems(new Set(items.map(i => i.id)))
    }
  }

  const toggleSelect = (id: string) => {
    const newSet = new Set(selectedItems)
    if (newSet.has(id)) {
      newSet.delete(id)
    } else {
      newSet.add(id)
    }
    setSelectedItems(newSet)
  }

  if (!campaign) return null

  const progress = campaign.total_items > 0
    ? ((campaign.approved_items + campaign.revoked_items) / campaign.total_items) * 100
    : 0

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="w-full sm:max-w-[900px] overflow-y-auto">
        <SheetHeader>
          <SheetTitle className="flex items-center gap-2">
            {campaign.name}
            <Badge variant={statusVariants[campaign.status || 'draft'] || 'secondary'}>
              {formatStatus(campaign.status || 'draft')}
            </Badge>
          </SheetTitle>
          <SheetDescription>
            {campaign.description || 'No description'}
          </SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          {/* Campaign Info */}
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-muted-foreground">Review Type:</span>
              <span className="ml-2 font-medium">{formatStatus(campaign.review_type || 'periodic')}</span>
            </div>
            <div>
              <span className="text-muted-foreground">Integration:</span>
              <span className="ml-2 font-medium">{campaign.integration_type ? formatStatus(campaign.integration_type) : 'Manual'}</span>
            </div>
            <div>
              <span className="text-muted-foreground">Due Date:</span>
              <span className="ml-2 font-medium">{formatDate(campaign.due_at)}</span>
            </div>
            <div>
              <span className="text-muted-foreground">Last Synced:</span>
              <span className="ml-2 font-medium">{formatRelativeTime(campaign.last_sync_at)}</span>
            </div>
          </div>

          {/* Progress */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Review Progress</span>
              <span>{campaign.approved_items + campaign.revoked_items} / {campaign.total_items}</span>
            </div>
            <Progress value={progress} className="h-2" />
            <div className="flex gap-4 text-xs text-muted-foreground">
              <span className="flex items-center gap-1">
                <div className="w-2 h-2 rounded-full bg-yellow-500" />
                Pending: {campaign.pending_items}
              </span>
              <span className="flex items-center gap-1">
                <div className="w-2 h-2 rounded-full bg-green-500" />
                Approved: {campaign.approved_items}
              </span>
              <span className="flex items-center gap-1">
                <div className="w-2 h-2 rounded-full bg-red-500" />
                Revoked: {campaign.revoked_items}
              </span>
            </div>
          </div>

          {/* Actions */}
          <div className="flex flex-wrap gap-2">
            {campaign.status === 'draft' && (
              <Button onClick={handleStartCampaign} disabled={startCampaignMutation.isLoading}>
                <Play className="mr-2 h-4 w-4" />
                Start Campaign
              </Button>
            )}
            {campaign.status === 'active' && campaign.pending_items === 0 && (
              <Button onClick={handleCompleteCampaign} disabled={completeCampaignMutation.isLoading}>
                <CheckCircle2 className="mr-2 h-4 w-4" />
                Complete Campaign
              </Button>
            )}
            {campaign.integration_id && (
              <Button variant="outline" onClick={handleSync} disabled={syncMutation.isLoading}>
                <RefreshCw className={`mr-2 h-4 w-4 ${syncMutation.isLoading ? 'animate-spin' : ''}`} />
                Sync Users
              </Button>
            )}
            {campaign.status === 'completed' && (
              <Button variant="outline" onClick={downloadCertification}>
                <Download className="mr-2 h-4 w-4" />
                Download Report
              </Button>
            )}
          </div>

          {/* Filters */}
          <div className="flex flex-wrap gap-2">
            <div className="relative flex-1 min-w-[200px]">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="Search users..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
                className="pl-10"
              />
            </div>
            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger className="w-[130px]">
                <Filter className="mr-2 h-4 w-4" />
                <SelectValue placeholder="Status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">All</SelectItem>
                <SelectItem value="pending">Pending</SelectItem>
                <SelectItem value="approved">Approved</SelectItem>
                <SelectItem value="revoked">Revoked</SelectItem>
              </SelectContent>
            </Select>
            <Select value={riskFilter} onValueChange={setRiskFilter}>
              <SelectTrigger className="w-[130px]">
                <Filter className="mr-2 h-4 w-4" />
                <SelectValue placeholder="Risk" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="">All</SelectItem>
                <SelectItem value="high">High</SelectItem>
                <SelectItem value="medium">Medium</SelectItem>
                <SelectItem value="low">Low</SelectItem>
              </SelectContent>
            </Select>
          </div>

          {/* Bulk Actions */}
          {selectedItems.size > 0 && campaign.status === 'active' && (
            <div className="flex items-center gap-2 p-2 bg-muted rounded-md">
              <span className="text-sm text-muted-foreground">{selectedItems.size} selected</span>
              <Button size="sm" variant="outline" onClick={() => handleBulkReview('approved')}>
                <CheckCircle2 className="mr-1 h-3 w-3" />
                Approve All
              </Button>
              <Button size="sm" variant="outline" onClick={() => handleBulkReview('revoked')}>
                <XCircle className="mr-1 h-3 w-3" />
                Revoke All
              </Button>
              <Button size="sm" variant="ghost" onClick={() => setSelectedItems(new Set())}>
                Clear
              </Button>
            </div>
          )}

          {/* Items Table */}
          {isLoading ? (
            <Loading />
          ) : items && items.length > 0 ? (
            <div className="rounded-md border max-h-[400px] overflow-y-auto">
              <table className="w-full">
                <thead className="sticky top-0 bg-background">
                  <tr className="border-b bg-muted/50">
                    {campaign.status === 'active' && (
                      <th className="p-2 w-8">
                        <input
                          type="checkbox"
                          checked={selectedItems.size === items.length}
                          onChange={toggleSelectAll}
                          className="rounded"
                        />
                      </th>
                    )}
                    <th className="p-2 text-left text-xs font-medium">User</th>
                    <th className="p-2 text-left text-xs font-medium">Risk</th>
                    <th className="p-2 text-left text-xs font-medium">Admin</th>
                    <th className="p-2 text-left text-xs font-medium">MFA</th>
                    <th className="p-2 text-left text-xs font-medium">Last Login</th>
                    <th className="p-2 text-left text-xs font-medium">Status</th>
                    {campaign.status === 'active' && (
                      <th className="p-2 text-left text-xs font-medium">Actions</th>
                    )}
                  </tr>
                </thead>
                <tbody>
                  {items.map((item) => (
                    <tr key={item.id} className="border-b hover:bg-muted/25">
                      {campaign.status === 'active' && (
                        <td className="p-2">
                          <input
                            type="checkbox"
                            checked={selectedItems.has(item.id)}
                            onChange={() => toggleSelect(item.id)}
                            className="rounded"
                          />
                        </td>
                      )}
                      <td className="p-2 text-sm">
                        <div>
                          <div className="font-medium">{item.user_name || item.user_identifier}</div>
                          <div className="text-xs text-muted-foreground">{item.user_email || item.user_identifier}</div>
                          {item.department && (
                            <div className="text-xs text-muted-foreground">{item.department}</div>
                          )}
                        </div>
                      </td>
                      <td className="p-2 text-sm">
                        {item.risk_level && (
                          <Badge variant={riskVariants[item.risk_level] || 'secondary'} className="text-xs">
                            {formatStatus(item.risk_level)}
                          </Badge>
                        )}
                      </td>
                      <td className="p-2 text-sm">
                        {item.is_admin ? (
                          <Key className="h-4 w-4 text-purple-500" />
                        ) : (
                          <span className="text-muted-foreground">-</span>
                        )}
                      </td>
                      <td className="p-2 text-sm">
                        {item.mfa_enabled === true ? (
                          <Shield className="h-4 w-4 text-green-500" />
                        ) : item.mfa_enabled === false ? (
                          <Shield className="h-4 w-4 text-red-500" />
                        ) : (
                          <span className="text-muted-foreground">-</span>
                        )}
                      </td>
                      <td className="p-2 text-xs text-muted-foreground">
                        {formatRelativeTime(item.last_login_at)}
                      </td>
                      <td className="p-2 text-sm">
                        <Badge variant={reviewStatusVariants[item.review_status || 'pending'] || 'secondary'} className="text-xs">
                          {formatStatus(item.review_status || 'pending')}
                        </Badge>
                      </td>
                      {campaign.status === 'active' && (
                        <td className="p-2 text-sm">
                          {(!item.review_status || item.review_status === 'pending') && (
                            <div className="flex gap-1">
                              <Button
                                size="sm"
                                variant="ghost"
                                className="h-7 w-7 p-0 text-green-600 hover:text-green-700 hover:bg-green-50"
                                onClick={() => handleReview(item.id, 'approved')}
                                disabled={reviewMutation.isLoading}
                              >
                                <CheckCircle2 className="h-4 w-4" />
                              </Button>
                              <Button
                                size="sm"
                                variant="ghost"
                                className="h-7 w-7 p-0 text-red-600 hover:text-red-700 hover:bg-red-50"
                                onClick={() => handleReview(item.id, 'revoked')}
                                disabled={reviewMutation.isLoading}
                              >
                                <UserX className="h-4 w-4" />
                              </Button>
                            </div>
                          )}
                        </td>
                      )}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="text-center py-8 text-muted-foreground">
              No users in this review. {campaign.integration_id && 'Click "Sync Users" to import from integration.'}
            </div>
          )}
        </div>
      </SheetContent>
    </Sheet>
  )
}

export default function AccessReviewsPage() {
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [isCreateOpen, setIsCreateOpen] = useState(false)
  const [selectedCampaign, setSelectedCampaign] = useState<CampaignWithStats | null>(null)
  const [isDetailOpen, setIsDetailOpen] = useState(false)

  const query: Record<string, string | number | boolean> = {}
  if (statusFilter) query.status = statusFilter

  const { data: campaigns, isLoading, error, refetch } = useAccessReviewCampaigns(query)
  const { data: stats, refetch: refetchStats } = useAccessReviewStats()

  const handleSuccess = () => {
    refetch()
    refetchStats()
  }

  const handleRowClick = (campaign: CampaignWithStats) => {
    setSelectedCampaign(campaign)
    setIsDetailOpen(true)
  }

  const handleDetailClose = () => {
    setIsDetailOpen(false)
    setSelectedCampaign(null)
  }

  const filteredCampaigns = campaigns?.filter(c =>
    !search || c.name.toLowerCase().includes(search.toLowerCase())
  )

  if (isLoading) {
    return <Loading />
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <p className="text-red-500 mb-2">Failed to load access reviews</p>
          <p className="text-sm text-muted-foreground">{error.message}</p>
          <Button onClick={() => refetch()} className="mt-4">Retry</Button>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Access Reviews"
        description="Conduct user access reviews and certifications"
      >
        <Button onClick={() => setIsCreateOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          New Campaign
        </Button>
      </PageHeader>

      <StatsCards stats={stats} />

      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search campaigns..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-10"
          />
        </div>
        <Select value={statusFilter} onValueChange={setStatusFilter}>
          <SelectTrigger className="w-[140px]">
            <Filter className="mr-2 h-4 w-4" />
            <SelectValue placeholder="Status" />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="">All Statuses</SelectItem>
            <SelectItem value="draft">Draft</SelectItem>
            <SelectItem value="active">Active</SelectItem>
            <SelectItem value="completed">Completed</SelectItem>
            <SelectItem value="cancelled">Cancelled</SelectItem>
          </SelectContent>
        </Select>
      </div>

      {filteredCampaigns && filteredCampaigns.length > 0 ? (
        <div className="rounded-md border">
          <table className="w-full">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="p-3 text-left text-sm font-medium">Campaign</th>
                <th className="p-3 text-left text-sm font-medium">Integration</th>
                <th className="p-3 text-left text-sm font-medium">Status</th>
                <th className="p-3 text-left text-sm font-medium">Progress</th>
                <th className="p-3 text-left text-sm font-medium">Due Date</th>
              </tr>
            </thead>
            <tbody>
              {filteredCampaigns.map((campaign) => {
                const progress = campaign.total_items > 0
                  ? ((campaign.approved_items + campaign.revoked_items) / campaign.total_items) * 100
                  : 0

                return (
                  <tr
                    key={campaign.id}
                    className="border-b hover:bg-muted/25 cursor-pointer"
                    onClick={() => handleRowClick(campaign)}
                  >
                    <td className="p-3 text-sm">
                      <div>
                        <div className="font-medium">{campaign.name}</div>
                        {campaign.description && (
                          <div className="text-muted-foreground text-xs line-clamp-1">
                            {campaign.description}
                          </div>
                        )}
                      </div>
                    </td>
                    <td className="p-3 text-sm">
                      {campaign.integration_type ? (
                        <Badge variant="outline">{formatStatus(campaign.integration_type)}</Badge>
                      ) : (
                        <span className="text-muted-foreground">Manual</span>
                      )}
                    </td>
                    <td className="p-3 text-sm">
                      <Badge variant={statusVariants[campaign.status || 'draft'] || 'secondary'}>
                        {formatStatus(campaign.status || 'draft')}
                      </Badge>
                    </td>
                    <td className="p-3 text-sm">
                      <div className="flex items-center gap-2">
                        <Progress value={progress} className="h-2 w-20" />
                        <span className="text-xs text-muted-foreground">
                          {campaign.approved_items + campaign.revoked_items}/{campaign.total_items}
                        </span>
                      </div>
                    </td>
                    <td className="p-3 text-sm">
                      {campaign.due_at ? (
                        <span className={
                          new Date(campaign.due_at) < new Date()
                            ? 'text-red-500'
                            : new Date(campaign.due_at) < new Date(Date.now() + 7 * 24 * 60 * 60 * 1000)
                            ? 'text-orange-500'
                            : ''
                        }>
                          {formatDate(campaign.due_at)}
                        </span>
                      ) : (
                        '-'
                      )}
                    </td>
                  </tr>
                )
              })}
            </tbody>
          </table>
        </div>
      ) : (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Users className="h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No access review campaigns</h3>
            <p className="text-muted-foreground text-sm mb-4">
              Create a campaign to start reviewing user access rights.
            </p>
            <Button onClick={() => setIsCreateOpen(true)}>
              <Plus className="mr-2 h-4 w-4" />
              Create Your First Campaign
            </Button>
          </CardContent>
        </Card>
      )}

      <CreateCampaignDialog
        open={isCreateOpen}
        onOpenChange={setIsCreateOpen}
        onSuccess={handleSuccess}
      />

      <CampaignDetailSheet
        campaign={selectedCampaign}
        open={isDetailOpen}
        onOpenChange={(open) => {
          if (!open) handleDetailClose()
        }}
        onUpdate={() => {
          handleSuccess()
          // Refresh selected campaign data
          if (selectedCampaign) {
            const updated = campaigns?.find(c => c.id === selectedCampaign.id)
            if (updated) setSelectedCampaign(updated)
          }
        }}
      />
    </div>
  )
}
