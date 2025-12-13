'use client'

import { useState } from 'react'
import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Loading } from "@/components/loading"
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  Plus,
  RefreshCw,
  Settings,
  AlertCircle,
  CheckCircle2,
  Clock,
  Trash2,
  Plug,
  Cloud,
  Shield,
  Code,
  Server,
  Webhook,
  Activity,
} from "lucide-react"
import Link from "next/link"
import { useIntegrations, useAvailableIntegrations, useIntegrationStats, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type {
  IntegrationWithStats,
  AvailableIntegration,
  IntegrationStats,
  CreateIntegration
} from '@/types'
import { formatDateTime } from '@/types'

const statusVariants: Record<string, 'success' | 'warning' | 'destructive' | 'secondary'> = {
  active: 'success',
  syncing: 'warning',
  error: 'destructive',
  inactive: 'secondary',
}

const categoryIcons: Record<string, React.ReactNode> = {
  'Cloud Provider': <Cloud className="h-5 w-5" />,
  'Identity Provider': <Shield className="h-5 w-5" />,
  'DevOps': <Code className="h-5 w-5" />,
  'Infrastructure': <Server className="h-5 w-5" />,
  'Custom': <Webhook className="h-5 w-5" />,
}

function StatsCards({ stats }: { stats: IntegrationStats | null }) {
  if (!stats) return null

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Total Integrations</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Plug className="h-5 w-5 text-primary" />
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
            <CheckCircle2 className="h-5 w-5 text-green-500" />
            <span className="text-2xl font-bold">{stats.active}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Inactive</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Clock className="h-5 w-5 text-gray-500" />
            <span className="text-2xl font-bold">{stats.inactive}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Errors</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <AlertCircle className="h-5 w-5 text-red-500" />
            <span className="text-2xl font-bold">{stats.error}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function IntegrationCard({
  integration,
  onConfigure,
  onSync
}: {
  integration: IntegrationWithStats
  onConfigure: () => void
  onSync: () => void
}) {
  const { integration: int, sync_count, last_sync_status, records_synced } = integration

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-start justify-between">
          <div>
            <CardTitle className="text-lg">{int.name}</CardTitle>
            <CardDescription className="capitalize">{int.integration_type.replace('_', ' ')}</CardDescription>
          </div>
          <Badge variant={statusVariants[int.status] || 'secondary'}>
            {int.status}
          </Badge>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-3">
          <div className="grid grid-cols-2 gap-2 text-sm">
            <div>
              <span className="text-muted-foreground">Last Sync:</span>
              <p className="font-medium">{formatDateTime(int.last_sync_at)}</p>
            </div>
            <div>
              <span className="text-muted-foreground">Total Syncs:</span>
              <p className="font-medium">{sync_count}</p>
            </div>
          </div>

          {int.last_error && (
            <div className="p-2 bg-destructive/10 rounded text-sm text-destructive">
              {int.last_error}
            </div>
          )}

          <div className="flex gap-2 pt-2">
            <Button variant="outline" size="sm" onClick={onConfigure}>
              <Settings className="mr-2 h-4 w-4" />
              Configure
            </Button>
            <Button variant="outline" size="sm" onClick={onSync} disabled={int.status === 'syncing'}>
              <RefreshCw className={`mr-2 h-4 w-4 ${int.status === 'syncing' ? 'animate-spin' : ''}`} />
              Sync
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  )
}

function AvailableIntegrationCard({
  integration,
  onConnect
}: {
  integration: AvailableIntegration
  onConnect: () => void
}) {
  return (
    <Card className="hover:border-primary/50 transition-colors">
      <CardHeader className="pb-3">
        <div className="flex items-start gap-3">
          <div className="p-2 bg-muted rounded-lg">
            {categoryIcons[integration.category] || <Plug className="h-5 w-5" />}
          </div>
          <div className="flex-1">
            <CardTitle className="text-lg">{integration.name}</CardTitle>
            <CardDescription className="text-xs">{integration.category}</CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <p className="text-sm text-muted-foreground mb-3">{integration.description}</p>
        <div className="flex flex-wrap gap-1 mb-4">
          {integration.capabilities.slice(0, 3).map((cap) => (
            <Badge key={cap} variant="outline" className="text-xs">
              {cap}
            </Badge>
          ))}
          {integration.capabilities.length > 3 && (
            <Badge variant="outline" className="text-xs">
              +{integration.capabilities.length - 3} more
            </Badge>
          )}
        </div>
        <Button size="sm" className="w-full" onClick={onConnect}>
          <Plus className="mr-2 h-4 w-4" />
          Connect
        </Button>
      </CardContent>
    </Card>
  )
}

function AddIntegrationDialog({
  open,
  onOpenChange,
  selectedType,
  availableIntegrations,
  onSuccess,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  selectedType: AvailableIntegration | null
  availableIntegrations: AvailableIntegration[]
  onSuccess: () => void
}) {
  const [step, setStep] = useState<'select' | 'configure'>('select')
  const [selected, setSelected] = useState<AvailableIntegration | null>(selectedType)
  const [name, setName] = useState('')
  const [config, setConfig] = useState<Record<string, string>>({})

  const createMutation = useMutation(async (data: CreateIntegration) => {
    return apiClient.post('/integrations', data)
  })

  const handleSelectType = (integration: AvailableIntegration) => {
    setSelected(integration)
    setName(`My ${integration.name}`)
    setConfig({})
    setStep('configure')
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (!selected) return

    try {
      await createMutation.mutate({
        integration_type: selected.integration_type,
        name,
        config,
      })
      onOpenChange(false)
      onSuccess()
      // Reset state
      setStep('select')
      setSelected(null)
      setName('')
      setConfig({})
    } catch (error) {
      console.error('Failed to create integration:', error)
    }
  }

  const handleClose = () => {
    onOpenChange(false)
    setStep('select')
    setSelected(null)
    setName('')
    setConfig({})
  }

  const schema = selected?.config_schema as { properties?: Record<string, { title?: string; type?: string; secret?: boolean; placeholder?: string }> } | undefined

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>
            {step === 'select' ? 'Add Integration' : `Configure ${selected?.name}`}
          </DialogTitle>
          <DialogDescription>
            {step === 'select'
              ? 'Select an integration type to connect'
              : 'Enter the configuration for your integration'
            }
          </DialogDescription>
        </DialogHeader>

        {step === 'select' ? (
          <div className="grid grid-cols-2 gap-3 py-4">
            {availableIntegrations.map((integration) => (
              <button
                key={integration.integration_type}
                onClick={() => handleSelectType(integration)}
                className="text-left p-4 border rounded-lg hover:border-primary hover:bg-accent transition-colors"
              >
                <div className="flex items-center gap-3">
                  <div className="p-2 bg-muted rounded">
                    {categoryIcons[integration.category] || <Plug className="h-4 w-4" />}
                  </div>
                  <div>
                    <p className="font-medium">{integration.name}</p>
                    <p className="text-xs text-muted-foreground">{integration.category}</p>
                  </div>
                </div>
              </button>
            ))}
          </div>
        ) : (
          <form onSubmit={handleSubmit} className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="name">Integration Name</Label>
              <Input
                id="name"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="My AWS Integration"
                required
              />
            </div>

            {schema?.properties && Object.entries(schema.properties).map(([key, field]) => (
              <div key={key} className="space-y-2">
                <Label htmlFor={key}>{field.title || key}</Label>
                <Input
                  id={key}
                  type={field.secret ? 'password' : 'text'}
                  value={config[key] || ''}
                  onChange={(e) => setConfig({ ...config, [key]: e.target.value })}
                  placeholder={field.placeholder}
                />
              </div>
            ))}

            <DialogFooter className="pt-4">
              <Button type="button" variant="outline" onClick={() => setStep('select')}>
                Back
              </Button>
              <Button type="submit" disabled={createMutation.isLoading}>
                {createMutation.isLoading ? 'Creating...' : 'Create Integration'}
              </Button>
            </DialogFooter>
          </form>
        )}
      </DialogContent>
    </Dialog>
  )
}

function IntegrationDetailSheet({
  integration,
  open,
  onOpenChange,
  onDelete,
  onSync,
}: {
  integration: IntegrationWithStats | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onDelete: () => void
  onSync: () => void
}) {
  if (!integration) return null

  const { integration: int, sync_count, records_synced } = integration

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent className="w-[400px] sm:w-[540px]">
        <SheetHeader>
          <SheetTitle>{int.name}</SheetTitle>
          <SheetDescription className="capitalize">
            {int.integration_type.replace('_', ' ')}
          </SheetDescription>
        </SheetHeader>

        <div className="mt-6 space-y-6">
          <div className="flex items-center gap-2">
            <Badge variant={statusVariants[int.status] || 'secondary'}>
              {int.status}
            </Badge>
          </div>

          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Created</p>
              <p className="font-medium">{formatDateTime(int.created_at)}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Last Sync</p>
              <p className="font-medium">{formatDateTime(int.last_sync_at)}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Total Syncs</p>
              <p className="font-medium">{sync_count}</p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Records Synced</p>
              <p className="font-medium">{records_synced ?? 0}</p>
            </div>
          </div>

          {int.last_error && (
            <div className="p-3 bg-destructive/10 rounded-lg">
              <p className="text-sm font-medium text-destructive">Last Error</p>
              <p className="text-sm text-destructive mt-1">{int.last_error}</p>
            </div>
          )}

          <div className="flex gap-2 pt-4">
            <Button className="flex-1" onClick={onSync} disabled={int.status === 'syncing'}>
              <RefreshCw className={`mr-2 h-4 w-4 ${int.status === 'syncing' ? 'animate-spin' : ''}`} />
              Sync Now
            </Button>
            <Button variant="destructive" onClick={onDelete}>
              <Trash2 className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </SheetContent>
    </Sheet>
  )
}

export default function IntegrationsPage() {
  const [addDialogOpen, setAddDialogOpen] = useState(false)
  const [selectedIntegration, setSelectedIntegration] = useState<IntegrationWithStats | null>(null)
  const [detailOpen, setDetailOpen] = useState(false)

  const { data: integrationsData, isLoading: integrationsLoading, refetch: refetchIntegrations } = useIntegrations()
  const { data: availableData, isLoading: availableLoading } = useAvailableIntegrations()
  const { data: statsData, refetch: refetchStats } = useIntegrationStats()

  const syncMutation = useMutation(async (id: string) => {
    return apiClient.post(`/integrations/${id}/sync`, { full_sync: false })
  })

  const deleteMutation = useMutation(async (id: string) => {
    return apiClient.delete(`/integrations/${id}`)
  })

  const handleSync = async (id: string) => {
    try {
      await syncMutation.mutate(id)
      refetchIntegrations()
      refetchStats()
    } catch (error) {
      console.error('Sync failed:', error)
    }
  }

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this integration?')) return
    try {
      await deleteMutation.mutate(id)
      setDetailOpen(false)
      refetchIntegrations()
      refetchStats()
    } catch (error) {
      console.error('Delete failed:', error)
    }
  }

  const integrations = integrationsData?.data || []
  const available = availableData?.data || []
  const stats = statsData?.data || null

  const isLoading = integrationsLoading || availableLoading

  if (isLoading) {
    return <Loading message="Loading integrations..." />
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Integrations"
        description="Connect external systems for automated evidence collection and monitoring"
        action={
          <div className="flex gap-2">
            <Button variant="outline" asChild>
              <Link href="/integrations/health/">
                <Activity className="mr-2 h-4 w-4" />
                Health Monitor
              </Link>
            </Button>
            <Button onClick={() => setAddDialogOpen(true)}>
              <Plus className="mr-2 h-4 w-4" />
              Add Integration
            </Button>
          </div>
        }
      />

      <StatsCards stats={stats} />

      <Tabs defaultValue="connected" className="space-y-4">
        <TabsList>
          <TabsTrigger value="connected">
            Connected ({integrations.length})
          </TabsTrigger>
          <TabsTrigger value="available">
            Available ({available.length})
          </TabsTrigger>
        </TabsList>

        <TabsContent value="connected">
          {integrations.length === 0 ? (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-12">
                <Plug className="h-12 w-12 text-muted-foreground mb-4" />
                <h3 className="text-lg font-medium mb-2">No integrations connected</h3>
                <p className="text-muted-foreground text-center mb-4">
                  Connect your first integration to start automating evidence collection
                </p>
                <Button onClick={() => setAddDialogOpen(true)}>
                  <Plus className="mr-2 h-4 w-4" />
                  Add Integration
                </Button>
              </CardContent>
            </Card>
          ) : (
            <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
              {integrations.map((integration) => (
                <IntegrationCard
                  key={integration.integration.id}
                  integration={integration}
                  onConfigure={() => {
                    setSelectedIntegration(integration)
                    setDetailOpen(true)
                  }}
                  onSync={() => handleSync(integration.integration.id)}
                />
              ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="available">
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
            {available.map((integration) => (
              <AvailableIntegrationCard
                key={integration.integration_type}
                integration={integration}
                onConnect={() => setAddDialogOpen(true)}
              />
            ))}
          </div>
        </TabsContent>
      </Tabs>

      <AddIntegrationDialog
        open={addDialogOpen}
        onOpenChange={setAddDialogOpen}
        selectedType={null}
        availableIntegrations={available}
        onSuccess={() => {
          refetchIntegrations()
          refetchStats()
        }}
      />

      <IntegrationDetailSheet
        integration={selectedIntegration}
        open={detailOpen}
        onOpenChange={setDetailOpen}
        onDelete={() => selectedIntegration && handleDelete(selectedIntegration.integration.id)}
        onSync={() => selectedIntegration && handleSync(selectedIntegration.integration.id)}
      />
    </div>
  )
}
