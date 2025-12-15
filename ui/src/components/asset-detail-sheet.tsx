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
  Server,
  Shield,
  MapPin,
  Network,
  Calendar,
  Wrench,
  Cloud,
  X,
  Search,
} from "lucide-react"
import { useAsset, useControls, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type {
  AssetWithControls,
  UpdateAsset,
  LinkedControl,
  ControlWithMappings,
} from '@/types'
import { formatStatus, formatDate } from '@/types'

const classificationVariants: Record<string, 'destructive' | 'warning' | 'secondary' | 'outline'> = {
  restricted: 'destructive',
  confidential: 'warning',
  internal: 'secondary',
  public: 'outline',
}

const lifecycleVariants: Record<string, 'success' | 'warning' | 'secondary' | 'destructive' | 'outline'> = {
  procurement: 'outline',
  deployment: 'secondary',
  active: 'success',
  maintenance: 'warning',
  decommissioning: 'destructive',
  decommissioned: 'secondary',
}

interface ControlSelectorProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  assetId: string
  existingControlIds: string[]
  onSuccess: () => void
}

function ControlSelector({
  open,
  onOpenChange,
  assetId,
  existingControlIds,
  onSuccess,
}: ControlSelectorProps) {
  const [selectedControls, setSelectedControls] = useState<string[]>([])
  const [search, setSearch] = useState('')

  const { data: controls } = useControls()

  const linkMutation = useMutation(async (controlIds: string[]) => {
    return apiClient.post(`/assets/${assetId}/controls`, { control_ids: controlIds })
  })

  const availableControls = controls?.filter(
    (c) => !existingControlIds.includes(c.id)
  ) || []

  const filteredControls = availableControls.filter(
    (c) =>
      search === '' ||
      c.code.toLowerCase().includes(search.toLowerCase()) ||
      c.name.toLowerCase().includes(search.toLowerCase())
  )

  const handleToggleControl = (controlId: string) => {
    setSelectedControls((prev) =>
      prev.includes(controlId)
        ? prev.filter((id) => id !== controlId)
        : [...prev, controlId]
    )
  }

  const handleSave = async () => {
    if (selectedControls.length === 0) return
    try {
      await linkMutation.mutate(selectedControls)
      setSelectedControls([])
      setSearch('')
      onOpenChange(false)
      onSuccess()
    } catch {
      // Error handled by mutation
    }
  }

  const handleClose = () => {
    setSelectedControls([])
    setSearch('')
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[600px] max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>Link Controls</DialogTitle>
          <DialogDescription>
            Select controls to link to this asset.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 flex-1 overflow-hidden flex flex-col">
          <div className="relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
            <Input
              placeholder="Search controls..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              className="pl-10"
            />
          </div>

          <div className="flex-1 overflow-y-auto border rounded-md min-h-[300px]">
            {filteredControls.length === 0 ? (
              <div className="p-8 text-center text-muted-foreground">
                {availableControls.length === 0
                  ? 'All controls are already linked to this asset.'
                  : 'No controls match your search.'}
              </div>
            ) : (
              <div className="divide-y">
                {filteredControls.map((control) => (
                  <label
                    key={control.id}
                    className="flex items-start gap-3 px-4 py-3 hover:bg-muted/25 cursor-pointer"
                  >
                    <input
                      type="checkbox"
                      checked={selectedControls.includes(control.id)}
                      onChange={() => handleToggleControl(control.id)}
                      className="mt-1"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-mono text-sm font-medium">
                          {control.code}
                        </span>
                        <span className="text-sm">{control.name}</span>
                      </div>
                      {control.description && (
                        <p className="text-xs text-muted-foreground mt-1 line-clamp-2">
                          {control.description}
                        </p>
                      )}
                    </div>
                  </label>
                ))}
              </div>
            )}
          </div>

          {selectedControls.length > 0 && (
            <div className="text-sm text-muted-foreground">
              {selectedControls.length} control(s) selected
            </div>
          )}
        </div>

        {linkMutation.error && (
          <div className="text-sm text-red-500">
            {linkMutation.error.message}
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={handleClose}>
            Cancel
          </Button>
          <Button
            onClick={handleSave}
            disabled={selectedControls.length === 0 || linkMutation.isLoading}
          >
            {linkMutation.isLoading ? 'Linking...' : `Link ${selectedControls.length} Control(s)`}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

interface AssetDetailSheetProps {
  assetId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onUpdate?: () => void
  onDelete?: () => void
}

export function AssetDetailSheet({
  assetId,
  open,
  onOpenChange,
  onUpdate,
  onDelete,
}: AssetDetailSheetProps) {
  const { data: asset, isLoading, refetch } = useAsset(assetId || '')
  const [isEditing, setIsEditing] = useState(false)
  const [isLinkingOpen, setIsLinkingOpen] = useState(false)
  const [formData, setFormData] = useState<UpdateAsset>({})

  useEffect(() => {
    if (asset) {
      setFormData({
        name: asset.name,
        description: asset.description || '',
        asset_type: asset.asset_type || '',
        category: asset.category || '',
        classification: asset.classification || '',
        status: asset.status || '',
        location: asset.location || '',
        ip_address: asset.ip_address || '',
        mac_address: asset.mac_address || '',
        purchase_date: asset.purchase_date || '',
        warranty_until: asset.warranty_until || '',
        lifecycle_stage: asset.lifecycle_stage || '',
        commissioned_date: asset.commissioned_date || '',
        next_maintenance_due: asset.next_maintenance_due || '',
        maintenance_frequency: asset.maintenance_frequency || '',
        end_of_life_date: asset.end_of_life_date || '',
        end_of_support_date: asset.end_of_support_date || '',
      })
    }
  }, [asset])

  useEffect(() => {
    if (!open) {
      setIsEditing(false)
    }
  }, [open])

  const updateMutation = useMutation(async (data: UpdateAsset) => {
    return apiClient.put<AssetWithControls>(`/assets/${assetId}`, data)
  })

  const deleteMutation = useMutation(async () => {
    return apiClient.delete(`/assets/${assetId}`)
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
    if (!confirm('Are you sure you want to delete this asset? This action cannot be undone.')) return
    try {
      await deleteMutation.mutate(undefined)
      onOpenChange(false)
      onDelete?.()
    } catch {
      // Error handled by mutation
    }
  }

  const handleUnlinkControl = async (controlId: string) => {
    try {
      await apiClient.delete(`/assets/${assetId}/controls`, { control_ids: [controlId] })
      refetch()
      onUpdate?.()
    } catch (err) {
      console.error('Failed to unlink control:', err)
    }
  }

  const handleLinkSuccess = () => {
    refetch()
    onUpdate?.()
  }

  const existingControlIds = asset?.linked_controls?.map((c) => c.id) || []

  return (
    <>
      <Sheet open={open} onOpenChange={onOpenChange}>
        <SheetContent className="sm:max-w-2xl overflow-y-auto">
          {isLoading || !asset ? (
            <div className="flex items-center justify-center h-full">
              <Loading />
            </div>
          ) : (
            <div className="space-y-6">
              <SheetHeader>
                <div className="flex items-center gap-2">
                  <Server className="h-5 w-5 text-primary" />
                  <SheetTitle>{asset.name}</SheetTitle>
                  {asset.integration_source && (
                    <Badge variant="outline" className="flex items-center gap-1">
                      <Cloud className="h-3 w-3" />
                      {asset.integration_source}
                    </Badge>
                  )}
                </div>
                <SheetDescription>
                  {asset.description || 'No description provided'}
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

              {/* Asset Details */}
              <div className="space-y-4">
                <h3 className="font-semibold">Asset Details</h3>

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
                        <Label htmlFor="asset_type">Type</Label>
                        <Select
                          value={formData.asset_type || ''}
                          onValueChange={(value) => setFormData({ ...formData, asset_type: value })}
                        >
                          <SelectTrigger>
                            <SelectValue />
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
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <Label>Classification</Label>
                        <Select
                          value={formData.classification || ''}
                          onValueChange={(value) => setFormData({ ...formData, classification: value })}
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
                        <Label>Lifecycle Stage</Label>
                        <Select
                          value={formData.lifecycle_stage || ''}
                          onValueChange={(value) => setFormData({ ...formData, lifecycle_stage: value })}
                        >
                          <SelectTrigger>
                            <SelectValue />
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
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <Label htmlFor="location">Location</Label>
                        <Input
                          id="location"
                          value={formData.location || ''}
                          onChange={(e) => setFormData({ ...formData, location: e.target.value })}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="ip_address">IP Address</Label>
                        <Input
                          id="ip_address"
                          value={formData.ip_address || ''}
                          onChange={(e) => setFormData({ ...formData, ip_address: e.target.value })}
                        />
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <Label htmlFor="warranty_until">Warranty Until</Label>
                        <Input
                          id="warranty_until"
                          type="date"
                          value={formData.warranty_until || ''}
                          onChange={(e) => setFormData({ ...formData, warranty_until: e.target.value })}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="next_maintenance_due">Next Maintenance</Label>
                        <Input
                          id="next_maintenance_due"
                          type="date"
                          value={formData.next_maintenance_due || ''}
                          onChange={(e) => setFormData({ ...formData, next_maintenance_due: e.target.value })}
                        />
                      </div>
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div className="space-y-2">
                        <Label htmlFor="end_of_life_date">End of Life</Label>
                        <Input
                          id="end_of_life_date"
                          type="date"
                          value={formData.end_of_life_date || ''}
                          onChange={(e) => setFormData({ ...formData, end_of_life_date: e.target.value })}
                        />
                      </div>
                      <div className="space-y-2">
                        <Label htmlFor="end_of_support_date">End of Support</Label>
                        <Input
                          id="end_of_support_date"
                          type="date"
                          value={formData.end_of_support_date || ''}
                          onChange={(e) => setFormData({ ...formData, end_of_support_date: e.target.value })}
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
                        <Label className="text-muted-foreground text-xs">Type</Label>
                        <p className="capitalize">{asset.asset_type || '-'}</p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Classification</Label>
                        {asset.classification ? (
                          <Badge variant={classificationVariants[asset.classification] || 'secondary'} className="mt-1">
                            {formatStatus(asset.classification)}
                          </Badge>
                        ) : (
                          <p>-</p>
                        )}
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Lifecycle Stage</Label>
                        {asset.lifecycle_stage ? (
                          <Badge variant={lifecycleVariants[asset.lifecycle_stage] || 'secondary'} className="mt-1">
                            {formatStatus(asset.lifecycle_stage)}
                          </Badge>
                        ) : (
                          <p>-</p>
                        )}
                      </div>
                    </div>
                    {(asset.location || asset.ip_address) && (
                      <div className="grid grid-cols-2 gap-4">
                        {asset.location && (
                          <div>
                            <Label className="text-muted-foreground text-xs">Location</Label>
                            <p className="flex items-center gap-1">
                              <MapPin className="h-3 w-3" />
                              {asset.location}
                            </p>
                          </div>
                        )}
                        {asset.ip_address && (
                          <div>
                            <Label className="text-muted-foreground text-xs">IP Address</Label>
                            <p className="flex items-center gap-1 font-mono text-xs">
                              <Network className="h-3 w-3" />
                              {asset.ip_address}
                            </p>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                )}
              </div>

              <Separator />

              {/* Lifecycle Info */}
              <div className="space-y-4">
                <h3 className="font-semibold flex items-center gap-2">
                  <Wrench className="h-4 w-4" />
                  Lifecycle & Maintenance
                </h3>
                <div className="grid grid-cols-2 gap-4 text-sm">
                  <div>
                    <Label className="text-muted-foreground text-xs">Purchase Date</Label>
                    <p className="flex items-center gap-1">
                      <Calendar className="h-3 w-3" />
                      {formatDate(asset.purchase_date)}
                    </p>
                  </div>
                  <div>
                    <Label className="text-muted-foreground text-xs">Warranty Until</Label>
                    <p className={`flex items-center gap-1 ${
                      asset.warranty_until && new Date(asset.warranty_until) < new Date(Date.now() + 30 * 24 * 60 * 60 * 1000)
                        ? 'text-orange-500'
                        : ''
                    }`}>
                      <Calendar className="h-3 w-3" />
                      {formatDate(asset.warranty_until)}
                    </p>
                  </div>
                  <div>
                    <Label className="text-muted-foreground text-xs">Last Maintenance</Label>
                    <p className="flex items-center gap-1">
                      <Calendar className="h-3 w-3" />
                      {formatDate(asset.last_maintenance_date)}
                    </p>
                  </div>
                  <div>
                    <Label className="text-muted-foreground text-xs">Next Maintenance Due</Label>
                    <p className={`flex items-center gap-1 ${
                      asset.next_maintenance_due && new Date(asset.next_maintenance_due) < new Date()
                        ? 'text-red-500'
                        : ''
                    }`}>
                      <Calendar className="h-3 w-3" />
                      {formatDate(asset.next_maintenance_due)}
                    </p>
                  </div>
                  <div>
                    <Label className="text-muted-foreground text-xs">End of Life</Label>
                    <p className="flex items-center gap-1">
                      <Calendar className="h-3 w-3" />
                      {formatDate(asset.end_of_life_date)}
                    </p>
                  </div>
                  <div>
                    <Label className="text-muted-foreground text-xs">End of Support</Label>
                    <p className="flex items-center gap-1">
                      <Calendar className="h-3 w-3" />
                      {formatDate(asset.end_of_support_date)}
                    </p>
                  </div>
                </div>
              </div>

              <Separator />

              {/* Linked Controls */}
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold flex items-center gap-2">
                    <Shield className="h-4 w-4" />
                    Linked Controls
                    <Badge variant="secondary">{asset.linked_control_count}</Badge>
                  </h3>
                  <Button size="sm" onClick={() => setIsLinkingOpen(true)}>
                    <Plus className="mr-2 h-4 w-4" />
                    Link
                  </Button>
                </div>

                {asset.linked_controls && asset.linked_controls.length > 0 ? (
                  <div className="space-y-2">
                    {asset.linked_controls.map((control) => (
                      <div
                        key={control.id}
                        className="flex items-center justify-between px-3 py-2 border rounded-lg hover:bg-muted/25"
                      >
                        <div className="flex items-center gap-2">
                          <Shield className="h-4 w-4 text-muted-foreground" />
                          <span className="font-mono text-sm">{control.code}</span>
                          <span className="text-sm text-muted-foreground">{control.name}</span>
                        </div>
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleUnlinkControl(control.id)}
                          className="h-6 w-6 p-0 text-muted-foreground hover:text-destructive"
                        >
                          <X className="h-3 w-3" />
                        </Button>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="text-center py-6 border rounded-lg border-dashed">
                    <Shield className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                    <p className="text-sm text-muted-foreground mb-3">
                      No controls linked yet
                    </p>
                    <Button size="sm" variant="outline" onClick={() => setIsLinkingOpen(true)}>
                      <Plus className="mr-2 h-4 w-4" />
                      Link Controls
                    </Button>
                  </div>
                )}
              </div>

              {/* Integration Info */}
              {asset.integration_source && (
                <>
                  <Separator />
                  <div className="space-y-4">
                    <h3 className="font-semibold flex items-center gap-2">
                      <Cloud className="h-4 w-4" />
                      Integration Source
                    </h3>
                    <div className="bg-muted/50 rounded-lg p-4 text-sm">
                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <Label className="text-muted-foreground text-xs">Source</Label>
                          <p className="capitalize">{asset.integration_source}</p>
                        </div>
                        <div>
                          <Label className="text-muted-foreground text-xs">External ID</Label>
                          <p className="font-mono text-xs">{asset.external_id || '-'}</p>
                        </div>
                        <div className="col-span-2">
                          <Label className="text-muted-foreground text-xs">Last Synced</Label>
                          <p>{asset.last_synced_at ? formatDate(asset.last_synced_at) : 'Never'}</p>
                        </div>
                      </div>
                    </div>
                  </div>
                </>
              )}
            </div>
          )}
        </SheetContent>
      </Sheet>

      {assetId && (
        <ControlSelector
          open={isLinkingOpen}
          onOpenChange={setIsLinkingOpen}
          assetId={assetId}
          existingControlIds={existingControlIds}
          onSuccess={handleLinkSuccess}
        />
      )}
    </>
  )
}
