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
import { Plus, Search, Filter, Shield, ShieldCheck, ShieldAlert, ShieldOff } from "lucide-react"
import { useControls, useControlStats, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type { ControlWithMappings, CreateControl, ControlStats, ControlStatus, ControlType, ControlFrequency } from '@/types'
import { formatStatus } from '@/types'

const statusVariants: Record<string, 'success' | 'warning' | 'destructive' | 'secondary'> = {
  implemented: 'success',
  in_progress: 'warning',
  not_implemented: 'destructive',
  not_applicable: 'secondary',
}

const typeLabels: Record<string, string> = {
  preventive: 'Preventive',
  detective: 'Detective',
  corrective: 'Corrective',
}

const frequencyLabels: Record<string, string> = {
  continuous: 'Continuous',
  daily: 'Daily',
  weekly: 'Weekly',
  monthly: 'Monthly',
  quarterly: 'Quarterly',
  annual: 'Annual',
}

function StatsCards({ stats }: { stats: ControlStats | null }) {
  if (!stats) return null

  return (
    <div className="grid grid-cols-2 md:grid-cols-5 gap-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Total Controls</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Shield className="h-5 w-5 text-primary" />
            <span className="text-2xl font-bold">{stats.total}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Implemented</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <ShieldCheck className="h-5 w-5 text-green-500" />
            <span className="text-2xl font-bold">{stats.implemented}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">In Progress</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <ShieldAlert className="h-5 w-5 text-yellow-500" />
            <span className="text-2xl font-bold">{stats.in_progress}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Not Implemented</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <ShieldOff className="h-5 w-5 text-red-500" />
            <span className="text-2xl font-bold">{stats.not_implemented}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Implementation</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{stats.implementation_percentage.toFixed(0)}%</div>
          <div className="mt-2 h-2 bg-secondary rounded-full overflow-hidden">
            <div
              className="h-full bg-primary transition-all"
              style={{ width: `${stats.implementation_percentage}%` }}
            />
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function ControlForm({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}) {
  const [formData, setFormData] = useState<CreateControl>({
    code: '',
    name: '',
    description: '',
    control_type: 'preventive',
    frequency: 'continuous',
    status: 'not_implemented',
    implementation_notes: '',
  })

  const createMutation = useMutation(async (data: CreateControl) => {
    return apiClient.post<ControlWithMappings>('/controls', data)
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createMutation.mutate(formData)
      onOpenChange(false)
      setFormData({
        code: '',
        name: '',
        description: '',
        control_type: 'preventive',
        frequency: 'continuous',
        status: 'not_implemented',
        implementation_notes: '',
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
          <DialogTitle>Add Control</DialogTitle>
          <DialogDescription>
            Define a new security control for your organization.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="code" className="text-right">
                Code *
              </Label>
              <Input
                id="code"
                value={formData.code}
                onChange={(e) => setFormData({ ...formData, code: e.target.value })}
                className="col-span-3"
                placeholder="e.g., AC-001"
                required
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="name" className="text-right">
                Name *
              </Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                className="col-span-3"
                placeholder="e.g., Access Control Policy"
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
                placeholder="Describe the control..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="type" className="text-right">
                Type
              </Label>
              <Select
                value={formData.control_type}
                onValueChange={(value: ControlType) => setFormData({ ...formData, control_type: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="preventive">Preventive</SelectItem>
                  <SelectItem value="detective">Detective</SelectItem>
                  <SelectItem value="corrective">Corrective</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="frequency" className="text-right">
                Frequency
              </Label>
              <Select
                value={formData.frequency}
                onValueChange={(value: ControlFrequency) => setFormData({ ...formData, frequency: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select frequency" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="continuous">Continuous</SelectItem>
                  <SelectItem value="daily">Daily</SelectItem>
                  <SelectItem value="weekly">Weekly</SelectItem>
                  <SelectItem value="monthly">Monthly</SelectItem>
                  <SelectItem value="quarterly">Quarterly</SelectItem>
                  <SelectItem value="annual">Annual</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="status" className="text-right">
                Status
              </Label>
              <Select
                value={formData.status}
                onValueChange={(value: ControlStatus) => setFormData({ ...formData, status: value })}
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select status" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="not_implemented">Not Implemented</SelectItem>
                  <SelectItem value="in_progress">In Progress</SelectItem>
                  <SelectItem value="implemented">Implemented</SelectItem>
                  <SelectItem value="not_applicable">Not Applicable</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="notes" className="text-right">
                Notes
              </Label>
              <Textarea
                id="notes"
                value={formData.implementation_notes || ''}
                onChange={(e) => setFormData({ ...formData, implementation_notes: e.target.value })}
                className="col-span-3"
                placeholder="Implementation notes..."
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
              {createMutation.isLoading ? 'Creating...' : 'Create Control'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default function ControlsPage() {
  const [search, setSearch] = useState('')
  const [statusFilter, setStatusFilter] = useState<string>('')
  const [typeFilter, setTypeFilter] = useState<string>('')
  const [isCreateOpen, setIsCreateOpen] = useState(false)

  const query: Record<string, string | number | boolean> = {}
  if (search) query.search = search
  if (statusFilter) query.status = statusFilter
  if (typeFilter) query.control_type = typeFilter

  const { data: controls, isLoading, error, refetch } = useControls(query)
  const { data: stats, refetch: refetchStats } = useControlStats()

  const handleSuccess = () => {
    refetch()
    refetchStats()
  }

  if (isLoading) {
    return <Loading />
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <p className="text-red-500 mb-2">Failed to load controls</p>
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
        title="Controls"
        description="Manage security controls and testing procedures"
      >
        <Button onClick={() => setIsCreateOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Add Control
        </Button>
      </PageHeader>

      <StatsCards stats={stats} />

      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search controls..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-10"
          />
        </div>
        <div className="flex gap-2">
          <Select value={statusFilter} onValueChange={setStatusFilter}>
            <SelectTrigger className="w-[160px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Status" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Statuses</SelectItem>
              <SelectItem value="implemented">Implemented</SelectItem>
              <SelectItem value="in_progress">In Progress</SelectItem>
              <SelectItem value="not_implemented">Not Implemented</SelectItem>
              <SelectItem value="not_applicable">Not Applicable</SelectItem>
            </SelectContent>
          </Select>
          <Select value={typeFilter} onValueChange={setTypeFilter}>
            <SelectTrigger className="w-[160px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Type" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Types</SelectItem>
              <SelectItem value="preventive">Preventive</SelectItem>
              <SelectItem value="detective">Detective</SelectItem>
              <SelectItem value="corrective">Corrective</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {controls && controls.length > 0 ? (
        <div className="rounded-md border">
          <table className="w-full">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="p-3 text-left text-sm font-medium">Code</th>
                <th className="p-3 text-left text-sm font-medium">Name</th>
                <th className="p-3 text-left text-sm font-medium">Type</th>
                <th className="p-3 text-left text-sm font-medium">Frequency</th>
                <th className="p-3 text-left text-sm font-medium">Status</th>
                <th className="p-3 text-left text-sm font-medium">Requirements</th>
              </tr>
            </thead>
            <tbody>
              {controls.map((control) => (
                <tr key={control.id} className="border-b hover:bg-muted/25 cursor-pointer">
                  <td className="p-3 text-sm font-mono">{control.code}</td>
                  <td className="p-3 text-sm">
                    <div>
                      <div className="font-medium">{control.name}</div>
                      {control.description && (
                        <div className="text-muted-foreground text-xs line-clamp-1">
                          {control.description}
                        </div>
                      )}
                    </div>
                  </td>
                  <td className="p-3 text-sm">
                    {typeLabels[control.control_type] || control.control_type}
                  </td>
                  <td className="p-3 text-sm">
                    {frequencyLabels[control.frequency] || control.frequency}
                  </td>
                  <td className="p-3 text-sm">
                    <Badge variant={statusVariants[control.status] || 'secondary'}>
                      {formatStatus(control.status)}
                    </Badge>
                  </td>
                  <td className="p-3 text-sm">
                    <span className="text-muted-foreground">
                      {control.requirement_count} mapped
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <Shield className="h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No controls defined</h3>
            <p className="text-muted-foreground text-sm mb-4">
              Add controls to track your security measures and compliance requirements.
            </p>
            <Button onClick={() => setIsCreateOpen(true)}>
              <Plus className="mr-2 h-4 w-4" />
              Add Your First Control
            </Button>
          </CardContent>
        </Card>
      )}

      <ControlForm
        open={isCreateOpen}
        onOpenChange={setIsCreateOpen}
        onSuccess={handleSuccess}
      />
    </div>
  )
}
