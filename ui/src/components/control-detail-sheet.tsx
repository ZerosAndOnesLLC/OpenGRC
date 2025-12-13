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
  Link2,
  Plus,
  CheckCircle2,
  BookOpen,
  X,
  Search,
  Edit2,
} from "lucide-react"
import { useControl, useFrameworks, useFramework, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type {
  ControlWithMappings,
  UpdateControl,
  MappedRequirement,
  FrameworkRequirement,
} from '@/types'
import { formatStatus } from '@/types'

const statusVariants: Record<string, 'success' | 'warning' | 'destructive' | 'secondary'> = {
  implemented: 'success',
  in_progress: 'warning',
  not_implemented: 'destructive',
  not_applicable: 'secondary',
}

interface RequirementSelectorProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  controlId: string
  existingRequirementIds: string[]
  onSuccess: () => void
}

function RequirementSelector({
  open,
  onOpenChange,
  controlId,
  existingRequirementIds,
  onSuccess,
}: RequirementSelectorProps) {
  const [selectedFramework, setSelectedFramework] = useState<string>('')
  const [selectedRequirements, setSelectedRequirements] = useState<string[]>([])
  const [search, setSearch] = useState('')

  const { data: frameworks } = useFrameworks()
  const { data: frameworkWithReqs, isLoading: loadingReqs } = useFramework(selectedFramework)

  const mapMutation = useMutation(async (requirementIds: string[]) => {
    return apiClient.post(`/controls/${controlId}/requirements`, {
      requirement_ids: requirementIds,
    })
  })

  const availableRequirements = frameworkWithReqs?.requirements?.filter(
    (req) => !existingRequirementIds.includes(req.id)
  ) || []

  const filteredRequirements = availableRequirements.filter(
    (req) =>
      search === '' ||
      req.code.toLowerCase().includes(search.toLowerCase()) ||
      req.name.toLowerCase().includes(search.toLowerCase()) ||
      req.description?.toLowerCase().includes(search.toLowerCase())
  )

  const groupedRequirements = filteredRequirements.reduce((acc, req) => {
    const category = req.category || 'Uncategorized'
    if (!acc[category]) acc[category] = []
    acc[category].push(req)
    return acc
  }, {} as Record<string, FrameworkRequirement[]>)

  const handleToggleRequirement = (reqId: string) => {
    setSelectedRequirements((prev) =>
      prev.includes(reqId)
        ? prev.filter((id) => id !== reqId)
        : [...prev, reqId]
    )
  }

  const handleSelectAll = () => {
    const allIds = filteredRequirements.map((r) => r.id)
    setSelectedRequirements(allIds)
  }

  const handleClearAll = () => {
    setSelectedRequirements([])
  }

  const handleSave = async () => {
    if (selectedRequirements.length === 0) return
    try {
      await mapMutation.mutate(selectedRequirements)
      setSelectedRequirements([])
      setSelectedFramework('')
      setSearch('')
      onOpenChange(false)
      onSuccess()
    } catch {
      // Error handled by mutation
    }
  }

  const handleClose = () => {
    setSelectedRequirements([])
    setSearch('')
    onOpenChange(false)
  }

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent className="sm:max-w-[800px] max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>Map Framework Requirements</DialogTitle>
          <DialogDescription>
            Select requirements from a compliance framework to map to this control.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 flex-1 overflow-hidden flex flex-col">
          <div className="space-y-2">
            <Label>Select Framework</Label>
            <Select value={selectedFramework} onValueChange={setSelectedFramework}>
              <SelectTrigger>
                <SelectValue placeholder="Choose a framework..." />
              </SelectTrigger>
              <SelectContent>
                {frameworks?.map((fw) => (
                  <SelectItem key={fw.id} value={fw.id}>
                    {fw.name} {fw.version && `(${fw.version})`}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {selectedFramework && (
            <>
              <div className="flex gap-2">
                <div className="relative flex-1">
                  <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    placeholder="Search requirements..."
                    value={search}
                    onChange={(e) => setSearch(e.target.value)}
                    className="pl-10"
                  />
                </div>
                <Button variant="outline" size="sm" onClick={handleSelectAll}>
                  Select All
                </Button>
                <Button variant="outline" size="sm" onClick={handleClearAll}>
                  Clear
                </Button>
              </div>

              <div className="flex-1 overflow-y-auto border rounded-md min-h-[300px]">
                {loadingReqs ? (
                  <div className="p-8 text-center text-muted-foreground">
                    Loading requirements...
                  </div>
                ) : filteredRequirements.length === 0 ? (
                  <div className="p-8 text-center text-muted-foreground">
                    {availableRequirements.length === 0
                      ? 'All requirements are already mapped to this control.'
                      : 'No requirements match your search.'}
                  </div>
                ) : (
                  <div className="divide-y">
                    {Object.entries(groupedRequirements).map(([category, reqs]) => (
                      <div key={category}>
                        <div className="px-4 py-2 bg-muted/50 font-medium text-sm sticky top-0">
                          {category}
                          <span className="text-muted-foreground ml-2">
                            ({reqs.length})
                          </span>
                        </div>
                        {reqs.map((req) => (
                          <label
                            key={req.id}
                            className="flex items-start gap-3 px-4 py-3 hover:bg-muted/25 cursor-pointer"
                          >
                            <input
                              type="checkbox"
                              checked={selectedRequirements.includes(req.id)}
                              onChange={() => handleToggleRequirement(req.id)}
                              className="mt-1"
                            />
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-2">
                                <span className="font-mono text-sm font-medium">
                                  {req.code}
                                </span>
                                <span className="text-sm">{req.name}</span>
                              </div>
                              {req.description && (
                                <p className="text-xs text-muted-foreground mt-1 line-clamp-2">
                                  {req.description}
                                </p>
                              )}
                            </div>
                          </label>
                        ))}
                      </div>
                    ))}
                  </div>
                )}
              </div>

              {selectedRequirements.length > 0 && (
                <div className="text-sm text-muted-foreground">
                  {selectedRequirements.length} requirement(s) selected
                </div>
              )}
            </>
          )}
        </div>

        {mapMutation.error && (
          <div className="text-sm text-red-500">
            {mapMutation.error.message}
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={handleClose}>
            Cancel
          </Button>
          <Button
            onClick={handleSave}
            disabled={selectedRequirements.length === 0 || mapMutation.isLoading}
          >
            {mapMutation.isLoading ? 'Mapping...' : `Map ${selectedRequirements.length} Requirement(s)`}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}

interface ControlDetailSheetProps {
  controlId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onUpdate?: () => void
  onDelete?: () => void
}

export function ControlDetailSheet({
  controlId,
  open,
  onOpenChange,
  onUpdate,
  onDelete,
}: ControlDetailSheetProps) {
  const { data: control, isLoading, refetch } = useControl(controlId || '')
  const [isEditing, setIsEditing] = useState(false)
  const [isMappingOpen, setIsMappingOpen] = useState(false)
  const [formData, setFormData] = useState<UpdateControl>({})

  useEffect(() => {
    if (control) {
      setFormData({
        code: control.code,
        name: control.name,
        description: control.description || '',
        control_type: control.control_type,
        frequency: control.frequency,
        status: control.status,
        implementation_notes: control.implementation_notes || '',
      })
    }
  }, [control])

  useEffect(() => {
    if (!open) {
      setIsEditing(false)
    }
  }, [open])

  const updateMutation = useMutation(async (data: UpdateControl) => {
    return apiClient.put<ControlWithMappings>(`/controls/${controlId}`, data)
  })

  const deleteMutation = useMutation(async () => {
    return apiClient.delete(`/controls/${controlId}`)
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
    if (!confirm('Are you sure you want to delete this control?')) return
    try {
      await deleteMutation.mutate(undefined)
      onOpenChange(false)
      onDelete?.()
    } catch {
      // Error handled by mutation
    }
  }

  const handleUnmapRequirement = async (reqId: string) => {
    try {
      await apiClient.delete(`/controls/${controlId}/requirements`, {
        requirement_ids: [reqId],
      })
      refetch()
      onUpdate?.()
    } catch (err) {
      console.error('Failed to unmap requirement:', err)
    }
  }

  const handleMappingSuccess = () => {
    refetch()
    onUpdate?.()
  }

  const existingRequirementIds = control?.mapped_requirements?.map((r) => r.id) || []

  return (
    <>
      <Sheet open={open} onOpenChange={onOpenChange}>
        <SheetContent className="sm:max-w-2xl overflow-y-auto">
          {isLoading || !control ? (
            <div className="flex items-center justify-center h-full">
              <Loading />
            </div>
          ) : (
            <div className="space-y-6">
              <SheetHeader>
                <div className="flex items-center gap-2">
                  <SheetTitle className="font-mono">{control.code}</SheetTitle>
                  <Badge variant={statusVariants[control.status] || 'secondary'}>
                    {formatStatus(control.status)}
                  </Badge>
                </div>
                <SheetDescription>{control.name}</SheetDescription>
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

              {/* Control Details */}
              <div className="space-y-4">
                <h3 className="font-semibold">Control Details</h3>

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
                        <Label htmlFor="name">Name</Label>
                        <Input
                          id="name"
                          value={formData.name || ''}
                          onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                        />
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
                        <Label>Type</Label>
                        <Select
                          value={formData.control_type}
                          onValueChange={(value) => setFormData({ ...formData, control_type: value })}
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="preventive">Preventive</SelectItem>
                            <SelectItem value="detective">Detective</SelectItem>
                            <SelectItem value="corrective">Corrective</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                      <div className="space-y-2">
                        <Label>Frequency</Label>
                        <Select
                          value={formData.frequency}
                          onValueChange={(value) => setFormData({ ...formData, frequency: value })}
                        >
                          <SelectTrigger>
                            <SelectValue />
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
                      <div className="space-y-2">
                        <Label>Status</Label>
                        <Select
                          value={formData.status}
                          onValueChange={(value) => setFormData({ ...formData, status: value })}
                        >
                          <SelectTrigger>
                            <SelectValue />
                          </SelectTrigger>
                          <SelectContent>
                            <SelectItem value="not_implemented">Not Implemented</SelectItem>
                            <SelectItem value="in_progress">In Progress</SelectItem>
                            <SelectItem value="implemented">Implemented</SelectItem>
                            <SelectItem value="not_applicable">Not Applicable</SelectItem>
                          </SelectContent>
                        </Select>
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="notes">Implementation Notes</Label>
                      <Textarea
                        id="notes"
                        value={formData.implementation_notes || ''}
                        onChange={(e) => setFormData({ ...formData, implementation_notes: e.target.value })}
                        rows={3}
                        placeholder="Document how this control is implemented..."
                      />
                    </div>
                    {updateMutation.error && (
                      <div className="text-sm text-red-500">{updateMutation.error.message}</div>
                    )}
                  </div>
                ) : (
                  <div className="space-y-3 text-sm">
                    {control.description && (
                      <div>
                        <Label className="text-muted-foreground text-xs">Description</Label>
                        <p>{control.description}</p>
                      </div>
                    )}
                    <div className="grid grid-cols-3 gap-4">
                      <div>
                        <Label className="text-muted-foreground text-xs">Type</Label>
                        <p className="capitalize">{control.control_type}</p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Frequency</Label>
                        <p className="capitalize">{control.frequency}</p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Status</Label>
                        <Badge variant={statusVariants[control.status] || 'secondary'} className="mt-1">
                          {formatStatus(control.status)}
                        </Badge>
                      </div>
                    </div>
                    {control.implementation_notes && (
                      <div>
                        <Label className="text-muted-foreground text-xs">Implementation Notes</Label>
                        <p className="whitespace-pre-wrap">{control.implementation_notes}</p>
                      </div>
                    )}
                  </div>
                )}
              </div>

              <Separator />

              {/* Mapped Requirements */}
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold flex items-center gap-2">
                    <Link2 className="h-4 w-4" />
                    Mapped Requirements
                    <Badge variant="secondary">{control.requirement_count}</Badge>
                  </h3>
                  <Button size="sm" onClick={() => setIsMappingOpen(true)}>
                    <Plus className="mr-2 h-4 w-4" />
                    Map
                  </Button>
                </div>

                {control.mapped_requirements && control.mapped_requirements.length > 0 ? (
                  <div className="space-y-2">
                    {Object.entries(
                      control.mapped_requirements.reduce((acc, req) => {
                        const fw = req.framework_name
                        if (!acc[fw]) acc[fw] = []
                        acc[fw].push(req)
                        return acc
                      }, {} as Record<string, MappedRequirement[]>)
                    ).map(([frameworkName, reqs]) => (
                      <div key={frameworkName} className="border rounded-lg">
                        <div className="px-3 py-2 bg-muted/50 flex items-center gap-2 text-sm">
                          <BookOpen className="h-3 w-3 text-primary" />
                          <span className="font-medium">{frameworkName}</span>
                          <Badge variant="outline" className="ml-auto text-xs">
                            {reqs.length}
                          </Badge>
                        </div>
                        <div className="divide-y">
                          {reqs.map((req) => (
                            <div
                              key={req.id}
                              className="px-3 py-2 flex items-center justify-between hover:bg-muted/25 text-sm"
                            >
                              <div className="flex items-center gap-2">
                                <CheckCircle2 className="h-3 w-3 text-green-500" />
                                <span className="font-mono text-xs">{req.code}</span>
                                <span className="text-muted-foreground">{req.name}</span>
                              </div>
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={() => handleUnmapRequirement(req.id)}
                                className="h-6 w-6 p-0 text-muted-foreground hover:text-destructive"
                              >
                                <X className="h-3 w-3" />
                              </Button>
                            </div>
                          ))}
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="text-center py-6 border rounded-lg border-dashed">
                    <Link2 className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                    <p className="text-sm text-muted-foreground mb-3">
                      No requirements mapped yet
                    </p>
                    <Button size="sm" variant="outline" onClick={() => setIsMappingOpen(true)}>
                      <Plus className="mr-2 h-4 w-4" />
                      Map Requirements
                    </Button>
                  </div>
                )}
              </div>
            </div>
          )}
        </SheetContent>
      </Sheet>

      {controlId && (
        <RequirementSelector
          open={isMappingOpen}
          onOpenChange={setIsMappingOpen}
          controlId={controlId}
          existingRequirementIds={existingRequirementIds}
          onSuccess={handleMappingSuccess}
        />
      )}
    </>
  )
}
