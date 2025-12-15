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
import { VendorDetailSheet } from "@/components/vendor-detail-sheet"
import {
  Plus,
  Search,
  Filter,
  Building2,
  AlertTriangle,
  ShieldAlert,
  Calendar,
  FileText,
} from "lucide-react"
import { useVendors, useVendorStats, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type { VendorWithAssessment, CreateVendor, VendorStats, VendorCriticality, VendorStatus } from '@/types'
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

function StatsCards({ stats }: { stats: VendorStats | null }) {
  if (!stats) return null

  const criticalCount = stats.by_criticality.find(c => c.criticality === 'critical')?.count || 0
  const highCount = stats.by_criticality.find(c => c.criticality === 'high')?.count || 0

  return (
    <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Total Vendors</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Building2 className="h-5 w-5 text-primary" />
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
            <Building2 className="h-5 w-5 text-green-500" />
            <span className="text-2xl font-bold">{stats.active}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Critical/High</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-red-500" />
            <span className="text-2xl font-bold">{criticalCount + highCount}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Needs Assessment</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <ShieldAlert className="h-5 w-5 text-yellow-500" />
            <span className="text-2xl font-bold">{stats.needs_assessment}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Expiring Contracts</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Calendar className="h-5 w-5 text-orange-500" />
            <span className="text-2xl font-bold">{stats.contracts_expiring_soon}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function VendorForm({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}) {
  const [formData, setFormData] = useState<CreateVendor>({
    name: '',
    description: '',
    category: '',
    criticality: 'medium',
    data_classification: 'internal',
    status: 'active',
    website: '',
  })

  const createMutation = useMutation(async (data: CreateVendor) => {
    return apiClient.post<VendorWithAssessment>('/vendors', data)
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createMutation.mutate(formData)
      onOpenChange(false)
      setFormData({
        name: '',
        description: '',
        category: '',
        criticality: 'medium',
        data_classification: 'internal',
        status: 'active',
        website: '',
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
          <DialogTitle>Add Vendor</DialogTitle>
          <DialogDescription>
            Add a new vendor to track for third-party risk management.
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
                placeholder="e.g., Acme Corp"
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
                placeholder="Describe the vendor and services provided..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="category" className="text-right">
                Category
              </Label>
              <Select
                value={formData.category || ''}
                onValueChange={(value) => setFormData({ ...formData, category: value })}
              >
                <SelectTrigger className="col-span-3">
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
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="criticality" className="text-right">
                Criticality
              </Label>
              <Select
                value={formData.criticality}
                onValueChange={(value: VendorCriticality) => setFormData({ ...formData, criticality: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select criticality" />
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
              <Label htmlFor="data_classification" className="text-right">
                Data Access
              </Label>
              <Select
                value={formData.data_classification || ''}
                onValueChange={(value) => setFormData({ ...formData, data_classification: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select data classification" />
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
              <Label htmlFor="status" className="text-right">
                Status
              </Label>
              <Select
                value={formData.status}
                onValueChange={(value: VendorStatus) => setFormData({ ...formData, status: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="active">Active</SelectItem>
                  <SelectItem value="inactive">Inactive</SelectItem>
                  <SelectItem value="under_review">Under Review</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="website" className="text-right">
                Website
              </Label>
              <Input
                id="website"
                value={formData.website || ''}
                onChange={(e) => setFormData({ ...formData, website: e.target.value })}
                className="col-span-3"
                placeholder="https://example.com"
                type="url"
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="contract_start" className="text-right">
                Contract Start
              </Label>
              <Input
                id="contract_start"
                type="date"
                value={formData.contract_start || ''}
                onChange={(e) => setFormData({ ...formData, contract_start: e.target.value })}
                className="col-span-3"
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="contract_end" className="text-right">
                Contract End
              </Label>
              <Input
                id="contract_end"
                type="date"
                value={formData.contract_end || ''}
                onChange={(e) => setFormData({ ...formData, contract_end: e.target.value })}
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
              {createMutation.isLoading ? 'Creating...' : 'Add Vendor'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default function VendorsPage() {
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [criticalityFilter, setCriticalityFilter] = useState<string>('')
  const [categoryFilter, setCategoryFilter] = useState<string>('')
  const [isCreateOpen, setIsCreateOpen] = useState(false)
  const [selectedVendorId, setSelectedVendorId] = useState<string | null>(null)
  const [isDetailOpen, setIsDetailOpen] = useState(false)

  const query: Record<string, string | number | boolean> = {}
  if (search) query.search = search
  if (statusFilter) query.status = statusFilter
  if (criticalityFilter) query.criticality = criticalityFilter
  if (categoryFilter) query.category = categoryFilter

  const { data: vendors, isLoading, error, refetch } = useVendors(query)
  const { data: stats, refetch: refetchStats } = useVendorStats()

  const handleSuccess = () => {
    refetch()
    refetchStats()
  }

  const handleRowClick = (vendorId: string) => {
    setSelectedVendorId(vendorId)
    setIsDetailOpen(true)
  }

  const handleDetailClose = () => {
    setIsDetailOpen(false)
    setSelectedVendorId(null)
  }

  if (isLoading) {
    return <Loading />
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <p className="text-red-500 mb-2">Failed to load vendors</p>
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
        title="Vendors"
        description="Manage vendor risk assessments and documentation"
      >
        <Button onClick={() => setIsCreateOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Add Vendor
        </Button>
      </PageHeader>

      <StatsCards stats={stats} />

      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search vendors..."
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
              <SelectItem value="active">Active</SelectItem>
              <SelectItem value="inactive">Inactive</SelectItem>
              <SelectItem value="under_review">Under Review</SelectItem>
            </SelectContent>
          </Select>
          <Select value={criticalityFilter} onValueChange={setCriticalityFilter}>
            <SelectTrigger className="w-[140px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Criticality" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Levels</SelectItem>
              <SelectItem value="critical">Critical</SelectItem>
              <SelectItem value="high">High</SelectItem>
              <SelectItem value="medium">Medium</SelectItem>
              <SelectItem value="low">Low</SelectItem>
            </SelectContent>
          </Select>
          <Select value={categoryFilter} onValueChange={setCategoryFilter}>
            <SelectTrigger className="w-[140px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Category" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Categories</SelectItem>
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

      {vendors && vendors.length > 0 ? (
        <div className="rounded-md border">
          <table className="w-full">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="p-3 text-left text-sm font-medium">Vendor</th>
                <th className="p-3 text-left text-sm font-medium">Category</th>
                <th className="p-3 text-left text-sm font-medium">Criticality</th>
                <th className="p-3 text-left text-sm font-medium">Risk Rating</th>
                <th className="p-3 text-left text-sm font-medium">Contract End</th>
                <th className="p-3 text-left text-sm font-medium">Status</th>
              </tr>
            </thead>
            <tbody>
              {vendors.map((vendor) => (
                <tr
                  key={vendor.id}
                  className="border-b hover:bg-muted/25 cursor-pointer"
                  onClick={() => handleRowClick(vendor.id)}
                >
                  <td className="p-3 text-sm">
                    <div>
                      <div className="font-medium">{vendor.name}</div>
                      {vendor.description && (
                        <div className="text-muted-foreground text-xs line-clamp-1">
                          {vendor.description}
                        </div>
                      )}
                    </div>
                  </td>
                  <td className="p-3 text-sm capitalize">
                    {vendor.category || '-'}
                  </td>
                  <td className="p-3 text-sm">
                    {vendor.criticality && (
                      <Badge variant={criticalityVariants[vendor.criticality] || 'secondary'}>
                        {formatStatus(vendor.criticality)}
                      </Badge>
                    )}
                  </td>
                  <td className="p-3 text-sm">
                    {vendor.last_risk_rating ? (
                      <Badge variant={riskRatingVariants[vendor.last_risk_rating] || 'secondary'}>
                        {formatStatus(vendor.last_risk_rating)}
                      </Badge>
                    ) : (
                      <span className="text-muted-foreground text-xs">Not assessed</span>
                    )}
                  </td>
                  <td className="p-3 text-sm">
                    {vendor.contract_end ? (
                      <span className={
                        new Date(vendor.contract_end) < new Date(Date.now() + 90 * 24 * 60 * 60 * 1000)
                          ? 'text-orange-500'
                          : ''
                      }>
                        {formatDate(vendor.contract_end)}
                      </span>
                    ) : (
                      '-'
                    )}
                  </td>
                  <td className="p-3 text-sm">
                    <Badge variant={statusVariants[vendor.status] || 'secondary'}>
                      {formatStatus(vendor.status)}
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
            <Building2 className="h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No vendors added</h3>
            <p className="text-muted-foreground text-sm mb-4">
              Add vendors to track third-party risk and compliance requirements.
            </p>
            <Button onClick={() => setIsCreateOpen(true)}>
              <Plus className="mr-2 h-4 w-4" />
              Add Your First Vendor
            </Button>
          </CardContent>
        </Card>
      )}

      <VendorForm
        open={isCreateOpen}
        onOpenChange={setIsCreateOpen}
        onSuccess={handleSuccess}
      />

      <VendorDetailSheet
        vendorId={selectedVendorId}
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
