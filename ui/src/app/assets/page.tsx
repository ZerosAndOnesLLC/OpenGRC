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
import { AssetDetailSheet } from "@/components/asset-detail-sheet"
import {
  Plus,
  Search,
  Filter,
  Server,
  HardDrive,
  Cloud,
  Shield,
  Wrench,
  Link2,
  AlertTriangle,
  RefreshCw,
} from "lucide-react"
import { useAssets, useAssetStats, useIntegrations, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type { AssetWithControls, CreateAsset, AssetStats, AssetType, AssetClassification, AssetStatus } from '@/types'
import { formatStatus } from '@/types'

const typeIcons: Record<string, React.ReactNode> = {
  hardware: <HardDrive className="h-4 w-4" />,
  software: <Server className="h-4 w-4" />,
  cloud: <Cloud className="h-4 w-4" />,
  network: <Link2 className="h-4 w-4" />,
}

const classificationVariants: Record<string, 'destructive' | 'warning' | 'secondary' | 'outline'> = {
  restricted: 'destructive',
  confidential: 'warning',
  internal: 'secondary',
  public: 'outline',
}

const statusVariants: Record<string, 'success' | 'warning' | 'secondary' | 'destructive'> = {
  active: 'success',
  under_review: 'warning',
  inactive: 'secondary',
  decommissioned: 'destructive',
}

const lifecycleVariants: Record<string, 'success' | 'warning' | 'secondary' | 'destructive' | 'outline'> = {
  procurement: 'outline',
  deployment: 'secondary',
  active: 'success',
  maintenance: 'warning',
  decommissioning: 'destructive',
  decommissioned: 'secondary',
}

function StatsCards({ stats }: { stats: AssetStats | null }) {
  if (!stats) return null

  const activeCount = stats.by_status.find(s => s.status === 'active')?.count || 0

  return (
    <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Total Assets</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Server className="h-5 w-5 text-primary" />
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
            <Server className="h-5 w-5 text-green-500" />
            <span className="text-2xl font-bold">{activeCount}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Maintenance Due</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Wrench className="h-5 w-5 text-yellow-500" />
            <span className="text-2xl font-bold">{stats.maintenance_due_soon}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Warranty Expiring</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-orange-500" />
            <span className="text-2xl font-bold">{stats.warranty_expiring_soon}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">From Integrations</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Cloud className="h-5 w-5 text-blue-500" />
            <span className="text-2xl font-bold">{stats.from_integrations}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

interface DiscoverResult {
  created: number
  updated: number
}

function DiscoverAssetsDialog({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}) {
  const [selectedIntegration, setSelectedIntegration] = useState<string>('')
  const [result, setResult] = useState<DiscoverResult | null>(null)

  const { data: integrations } = useIntegrations()

  // Filter to only show AWS integrations that are connected
  const awsIntegrations = integrations?.filter(
    (i) => i.type === 'aws' && i.status === 'connected'
  ) || []

  const discoverMutation = useMutation(async (integrationId: string) => {
    return apiClient.post<DiscoverResult>(`/assets/discover/${integrationId}`)
  })

  const handleDiscover = async () => {
    if (!selectedIntegration) return
    try {
      const res = await discoverMutation.mutate(selectedIntegration)
      setResult(res)
      onSuccess()
    } catch {
      // Error handled by mutation
    }
  }

  const handleClose = () => {
    setSelectedIntegration('')
    setResult(null)
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Discover Assets from AWS</DialogTitle>
          <DialogDescription>
            Automatically import EC2 instances, RDS databases, and S3 buckets from your connected AWS integrations.
          </DialogDescription>
        </DialogHeader>

        {result ? (
          <div className="py-6 text-center">
            <div className="text-4xl mb-4">
              {result.created > 0 || result.updated > 0 ? '🎉' : '✓'}
            </div>
            <h3 className="text-lg font-medium mb-2">Discovery Complete</h3>
            <p className="text-muted-foreground">
              {result.created > 0 && `${result.created} new asset(s) created`}
              {result.created > 0 && result.updated > 0 && ', '}
              {result.updated > 0 && `${result.updated} existing asset(s) updated`}
              {result.created === 0 && result.updated === 0 && 'No new assets found'}
            </p>
            <Button className="mt-4" onClick={handleClose}>
              Done
            </Button>
          </div>
        ) : awsIntegrations.length === 0 ? (
          <div className="py-6 text-center">
            <Cloud className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
            <h3 className="text-lg font-medium mb-2">No AWS Integrations</h3>
            <p className="text-muted-foreground text-sm mb-4">
              Connect an AWS integration first to discover assets automatically.
            </p>
            <Button variant="outline" onClick={handleClose}>
              Close
            </Button>
          </div>
        ) : (
          <>
            <div className="py-4 space-y-4">
              <div className="space-y-2">
                <Label>Select AWS Integration</Label>
                <Select value={selectedIntegration} onValueChange={setSelectedIntegration}>
                  <SelectTrigger>
                    <SelectValue placeholder="Choose an integration" />
                  </SelectTrigger>
                  <SelectContent>
                    {awsIntegrations.map((integration) => (
                      <SelectItem key={integration.id} value={integration.id}>
                        {integration.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              <div className="bg-muted/50 rounded-lg p-4 text-sm">
                <h4 className="font-medium mb-2">Resources to discover:</h4>
                <ul className="list-disc list-inside text-muted-foreground space-y-1">
                  <li>EC2 Instances (compute)</li>
                  <li>RDS Databases (database)</li>
                  <li>S3 Buckets (storage)</li>
                </ul>
              </div>
            </div>

            {discoverMutation.error && (
              <div className="text-sm text-red-500 mb-4">
                {discoverMutation.error.message}
              </div>
            )}

            <DialogFooter>
              <Button variant="outline" onClick={handleClose}>
                Cancel
              </Button>
              <Button
                onClick={handleDiscover}
                disabled={!selectedIntegration || discoverMutation.isLoading}
              >
                {discoverMutation.isLoading ? (
                  <>
                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    Discovering...
                  </>
                ) : (
                  <>
                    <Cloud className="mr-2 h-4 w-4" />
                    Discover Assets
                  </>
                )}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  )
}

function AssetForm({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}) {
  const [formData, setFormData] = useState<CreateAsset>({
    name: '',
    description: '',
    asset_type: 'hardware',
    classification: 'internal',
    lifecycle_stage: 'active',
  })

  const createMutation = useMutation(async (data: CreateAsset) => {
    return apiClient.post<AssetWithControls>('/assets', data)
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createMutation.mutate(formData)
      onOpenChange(false)
      setFormData({
        name: '',
        description: '',
        asset_type: 'hardware',
        classification: 'internal',
        lifecycle_stage: 'active',
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
          <DialogTitle>Add Asset</DialogTitle>
          <DialogDescription>
            Add a new asset to your inventory for tracking and compliance.
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
                placeholder="e.g., Production Database Server"
                required
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="description" className="text-right">
                Description
              </Label>
              <Textarea
                id="description"
                value={formData.description || ''}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                className="col-span-3"
                placeholder="Describe the asset..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="asset_type" className="text-right">
                Type
              </Label>
              <Select
                value={formData.asset_type}
                onValueChange={(value: AssetType) => setFormData({ ...formData, asset_type: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="hardware">Hardware</SelectItem>
                  <SelectItem value="software">Software</SelectItem>
                  <SelectItem value="data">Data</SelectItem>
                  <SelectItem value="network">Network</SelectItem>
                  <SelectItem value="cloud">Cloud</SelectItem>
                  <SelectItem value="physical">Physical</SelectItem>
                  <SelectItem value="people">People</SelectItem>
                  <SelectItem value="other">Other</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="category" className="text-right">
                Category
              </Label>
              <Input
                id="category"
                value={formData.category || ''}
                onChange={(e) => setFormData({ ...formData, category: e.target.value })}
                className="col-span-3"
                placeholder="e.g., Database, Compute, Storage"
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="classification" className="text-right">
                Classification
              </Label>
              <Select
                value={formData.classification}
                onValueChange={(value: AssetClassification) => setFormData({ ...formData, classification: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select classification" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="public">Public</SelectItem>
                  <SelectItem value="internal">Internal</SelectItem>
                  <SelectItem value="confidential">Confidential</SelectItem>
                  <SelectItem value="restricted">Restricted</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="lifecycle_stage" className="text-right">
                Lifecycle Stage
              </Label>
              <Select
                value={formData.lifecycle_stage || 'active'}
                onValueChange={(value) => setFormData({ ...formData, lifecycle_stage: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select stage" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="procurement">Procurement</SelectItem>
                  <SelectItem value="deployment">Deployment</SelectItem>
                  <SelectItem value="active">Active</SelectItem>
                  <SelectItem value="maintenance">Maintenance</SelectItem>
                  <SelectItem value="decommissioning">Decommissioning</SelectItem>
                  <SelectItem value="decommissioned">Decommissioned</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="location" className="text-right">
                Location
              </Label>
              <Input
                id="location"
                value={formData.location || ''}
                onChange={(e) => setFormData({ ...formData, location: e.target.value })}
                className="col-span-3"
                placeholder="e.g., AWS us-east-1, Office Building A"
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="ip_address" className="text-right">
                IP Address
              </Label>
              <Input
                id="ip_address"
                value={formData.ip_address || ''}
                onChange={(e) => setFormData({ ...formData, ip_address: e.target.value })}
                className="col-span-3"
                placeholder="e.g., 10.0.0.1"
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="purchase_date" className="text-right">
                Purchase Date
              </Label>
              <Input
                id="purchase_date"
                type="date"
                value={formData.purchase_date || ''}
                onChange={(e) => setFormData({ ...formData, purchase_date: e.target.value })}
                className="col-span-3"
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="warranty_until" className="text-right">
                Warranty Until
              </Label>
              <Input
                id="warranty_until"
                type="date"
                value={formData.warranty_until || ''}
                onChange={(e) => setFormData({ ...formData, warranty_until: e.target.value })}
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
              {createMutation.isLoading ? 'Creating...' : 'Add Asset'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default function AssetsPage() {
  const [search, setSearch] = useState('')
  const [typeFilter, setTypeFilter] = useState<string>('')
  const [classificationFilter, setClassificationFilter] = useState<string>('')
  const [lifecycleFilter, setLifecycleFilter] = useState<string>('')
  const [isCreateOpen, setIsCreateOpen] = useState(false)
  const [isDiscoverOpen, setIsDiscoverOpen] = useState(false)
  const [selectedAssetId, setSelectedAssetId] = useState<string | null>(null)
  const [isDetailOpen, setIsDetailOpen] = useState(false)

  const query: Record<string, string | number | boolean> = {}
  if (search) query.search = search
  if (typeFilter) query.asset_type = typeFilter
  if (classificationFilter) query.classification = classificationFilter
  if (lifecycleFilter) query.lifecycle_stage = lifecycleFilter

  const { data: assets, isLoading, error, refetch } = useAssets(query)
  const { data: stats, refetch: refetchStats } = useAssetStats()

  const handleSuccess = () => {
    refetch()
    refetchStats()
  }

  const handleRowClick = (assetId: string) => {
    setSelectedAssetId(assetId)
    setIsDetailOpen(true)
  }

  const handleDetailClose = () => {
    setIsDetailOpen(false)
    setSelectedAssetId(null)
  }

  if (isLoading) {
    return <Loading />
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <p className="text-red-500 mb-2">Failed to load assets</p>
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
        title="Assets"
        description="Manage organizational assets and inventory"
      >
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => setIsDiscoverOpen(true)}>
            <Cloud className="mr-2 h-4 w-4" />
            Discover from AWS
          </Button>
          <Button onClick={() => setIsCreateOpen(true)}>
            <Plus className="mr-2 h-4 w-4" />
            Add Asset
          </Button>
        </div>
      </PageHeader>

      <StatsCards stats={stats} />

      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search assets..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-10"
          />
        </div>
        <div className="flex gap-2">
          <Select value={typeFilter} onValueChange={setTypeFilter}>
            <SelectTrigger className="w-[140px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Types</SelectItem>
              <SelectItem value="hardware">Hardware</SelectItem>
              <SelectItem value="software">Software</SelectItem>
              <SelectItem value="data">Data</SelectItem>
              <SelectItem value="network">Network</SelectItem>
              <SelectItem value="cloud">Cloud</SelectItem>
              <SelectItem value="physical">Physical</SelectItem>
              <SelectItem value="people">People</SelectItem>
              <SelectItem value="other">Other</SelectItem>
            </SelectContent>
          </Select>
          <Select value={classificationFilter} onValueChange={setClassificationFilter}>
            <SelectTrigger className="w-[140px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Classification" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Levels</SelectItem>
              <SelectItem value="public">Public</SelectItem>
              <SelectItem value="internal">Internal</SelectItem>
              <SelectItem value="confidential">Confidential</SelectItem>
              <SelectItem value="restricted">Restricted</SelectItem>
            </SelectContent>
          </Select>
          <Select value={lifecycleFilter} onValueChange={setLifecycleFilter}>
            <SelectTrigger className="w-[150px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Lifecycle" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Stages</SelectItem>
              <SelectItem value="procurement">Procurement</SelectItem>
              <SelectItem value="deployment">Deployment</SelectItem>
              <SelectItem value="active">Active</SelectItem>
              <SelectItem value="maintenance">Maintenance</SelectItem>
              <SelectItem value="decommissioning">Decommissioning</SelectItem>
              <SelectItem value="decommissioned">Decommissioned</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {assets && assets.length > 0 ? (
        <div className="rounded-md border">
          <table className="w-full">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="p-3 text-left text-sm font-medium">Asset</th>
                <th className="p-3 text-left text-sm font-medium">Type</th>
                <th className="p-3 text-left text-sm font-medium">Classification</th>
                <th className="p-3 text-left text-sm font-medium">Lifecycle</th>
                <th className="p-3 text-left text-sm font-medium">Location</th>
                <th className="p-3 text-left text-sm font-medium">Controls</th>
              </tr>
            </thead>
            <tbody>
              {assets.map((asset) => (
                <tr
                  key={asset.id}
                  className="border-b hover:bg-muted/25 cursor-pointer"
                  onClick={() => handleRowClick(asset.id)}
                >
                  <td className="p-3 text-sm">
                    <div className="flex items-center gap-2">
                      {asset.integration_source && (
                        <Cloud className="h-3 w-3 text-blue-500" title={`From ${asset.integration_source}`} />
                      )}
                      <div>
                        <div className="font-medium">{asset.name}</div>
                        {asset.description && (
                          <div className="text-muted-foreground text-xs line-clamp-1">
                            {asset.description}
                          </div>
                        )}
                      </div>
                    </div>
                  </td>
                  <td className="p-3 text-sm">
                    <div className="flex items-center gap-1 capitalize">
                      {typeIcons[asset.asset_type || ''] || <Server className="h-4 w-4" />}
                      {asset.asset_type || '-'}
                    </div>
                  </td>
                  <td className="p-3 text-sm">
                    {asset.classification ? (
                      <Badge variant={classificationVariants[asset.classification] || 'secondary'}>
                        {formatStatus(asset.classification)}
                      </Badge>
                    ) : (
                      '-'
                    )}
                  </td>
                  <td className="p-3 text-sm">
                    {asset.lifecycle_stage ? (
                      <Badge variant={lifecycleVariants[asset.lifecycle_stage] || 'secondary'}>
                        {formatStatus(asset.lifecycle_stage)}
                      </Badge>
                    ) : (
                      '-'
                    )}
                  </td>
                  <td className="p-3 text-sm text-muted-foreground">
                    {asset.location || asset.ip_address || '-'}
                  </td>
                  <td className="p-3 text-sm">
                    <div className="flex items-center gap-1 text-muted-foreground">
                      <Shield className="h-3 w-3" />
                      {asset.linked_control_count}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Server className="h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No assets tracked</h3>
            <p className="text-muted-foreground text-sm mb-4">
              Add assets to track your IT inventory and compliance requirements.
            </p>
            <Button onClick={() => setIsCreateOpen(true)}>
              <Plus className="mr-2 h-4 w-4" />
              Add Your First Asset
            </Button>
          </CardContent>
        </Card>
      )}

      <AssetForm
        open={isCreateOpen}
        onOpenChange={setIsCreateOpen}
        onSuccess={handleSuccess}
      />

      <DiscoverAssetsDialog
        open={isDiscoverOpen}
        onOpenChange={setIsDiscoverOpen}
        onSuccess={handleSuccess}
      />

      <AssetDetailSheet
        assetId={selectedAssetId}
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
