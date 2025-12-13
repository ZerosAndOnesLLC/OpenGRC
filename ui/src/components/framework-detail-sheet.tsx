'use client'

import { useState, useEffect } from 'react'
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
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
  Plus,
  BookOpen,
  Edit2,
  ChevronRight,
  ChevronDown,
  FileText,
} from "lucide-react"
import { useFramework, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type {
  Framework,
  FrameworkWithRequirements,
  UpdateFramework,
  CreateFrameworkRequirement,
  FrameworkRequirement,
} from '@/types'

interface RequirementFormProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  frameworkId: string
  parentId?: string
  editRequirement?: FrameworkRequirement
  onSuccess: () => void
}

function RequirementForm({
  open,
  onOpenChange,
  frameworkId,
  parentId,
  editRequirement,
  onSuccess,
}: RequirementFormProps) {
  const [formData, setFormData] = useState<CreateFrameworkRequirement>({
    code: '',
    name: '',
    description: '',
    category: '',
    parent_id: parentId,
    sort_order: 0,
  })

  useEffect(() => {
    if (editRequirement) {
      setFormData({
        code: editRequirement.code,
        name: editRequirement.name,
        description: editRequirement.description || '',
        category: editRequirement.category || '',
        parent_id: editRequirement.parent_id || undefined,
        sort_order: editRequirement.sort_order,
      })
    } else {
      setFormData({
        code: '',
        name: '',
        description: '',
        category: '',
        parent_id: parentId,
        sort_order: 0,
      })
    }
  }, [editRequirement, parentId])

  const createMutation = useMutation(async (data: CreateFrameworkRequirement) => {
    return apiClient.post(`/frameworks/${frameworkId}/requirements`, data)
  })

  const updateMutation = useMutation(async (data: CreateFrameworkRequirement) => {
    return apiClient.put(`/frameworks/${frameworkId}/requirements/${editRequirement?.id}`, data)
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      if (editRequirement) {
        await updateMutation.mutate(formData)
      } else {
        await createMutation.mutate(formData)
      }
      onOpenChange(false)
      onSuccess()
    } catch {
      // Error handled by mutation
    }
  }

  const isLoading = createMutation.isLoading || updateMutation.isLoading
  const error = createMutation.error || updateMutation.error

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>{editRequirement ? 'Edit Requirement' : 'Add Requirement'}</DialogTitle>
          <DialogDescription>
            {editRequirement ? 'Update the requirement details.' : 'Add a new requirement to this framework.'}
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="code">Code *</Label>
              <Input
                id="code"
                value={formData.code}
                onChange={(e) => setFormData({ ...formData, code: e.target.value })}
                placeholder="e.g., CC1.1"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="name">Name *</Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="e.g., Control Environment"
                required
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={formData.description || ''}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                placeholder="Describe the requirement..."
                rows={3}
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="category">Category</Label>
              <Input
                id="category"
                value={formData.category || ''}
                onChange={(e) => setFormData({ ...formData, category: e.target.value })}
                placeholder="e.g., Security, Availability"
              />
            </div>
            <div className="space-y-2">
              <Label htmlFor="sort_order">Sort Order</Label>
              <Input
                id="sort_order"
                type="number"
                value={formData.sort_order || 0}
                onChange={(e) => setFormData({ ...formData, sort_order: parseInt(e.target.value) || 0 })}
              />
            </div>
          </div>
          {error && (
            <div className="text-sm text-red-500 mb-4">{error.message}</div>
          )}
          <DialogFooter>
            <Button type="button" variant="outline" onClick={() => onOpenChange(false)}>
              Cancel
            </Button>
            <Button type="submit" disabled={isLoading}>
              {isLoading ? 'Saving...' : editRequirement ? 'Update' : 'Add Requirement'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

interface RequirementItemProps {
  requirement: FrameworkRequirement
  allRequirements: FrameworkRequirement[]
  frameworkId: string
  onEdit: (req: FrameworkRequirement) => void
  onDelete: (req: FrameworkRequirement) => void
  onAddChild: (parentId: string) => void
}

function RequirementItem({
  requirement,
  allRequirements,
  frameworkId,
  onEdit,
  onDelete,
  onAddChild,
}: RequirementItemProps) {
  const [isExpanded, setIsExpanded] = useState(true)
  const children = allRequirements.filter((r) => r.parent_id === requirement.id)
  const hasChildren = children.length > 0

  return (
    <div className="border rounded-lg">
      <div className="flex items-start gap-2 p-3 hover:bg-muted/25">
        {hasChildren ? (
          <Button
            variant="ghost"
            size="sm"
            className="h-6 w-6 p-0"
            onClick={() => setIsExpanded(!isExpanded)}
          >
            {isExpanded ? (
              <ChevronDown className="h-4 w-4" />
            ) : (
              <ChevronRight className="h-4 w-4" />
            )}
          </Button>
        ) : (
          <div className="w-6" />
        )}
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-mono text-sm font-medium">{requirement.code}</span>
            <span className="text-sm">{requirement.name}</span>
            {requirement.category && (
              <Badge variant="outline" className="text-xs">
                {requirement.category}
              </Badge>
            )}
          </div>
          {requirement.description && (
            <p className="text-xs text-muted-foreground mt-1 line-clamp-2">
              {requirement.description}
            </p>
          )}
        </div>
        <div className="flex items-center gap-1">
          <Button
            variant="ghost"
            size="sm"
            className="h-7 w-7 p-0"
            onClick={() => onAddChild(requirement.id)}
            title="Add sub-requirement"
          >
            <Plus className="h-3 w-3" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 w-7 p-0"
            onClick={() => onEdit(requirement)}
          >
            <Edit2 className="h-3 w-3" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 w-7 p-0 text-destructive hover:text-destructive"
            onClick={() => onDelete(requirement)}
          >
            <Trash2 className="h-3 w-3" />
          </Button>
        </div>
      </div>
      {hasChildren && isExpanded && (
        <div className="pl-6 pr-3 pb-3 space-y-2">
          {children
            .sort((a, b) => a.sort_order - b.sort_order)
            .map((child) => (
              <RequirementItem
                key={child.id}
                requirement={child}
                allRequirements={allRequirements}
                frameworkId={frameworkId}
                onEdit={onEdit}
                onDelete={onDelete}
                onAddChild={onAddChild}
              />
            ))}
        </div>
      )}
    </div>
  )
}

interface FrameworkDetailSheetProps {
  frameworkId: string | null
  open: boolean
  onOpenChange: (open: boolean) => void
  onUpdate?: () => void
  onDelete?: () => void
}

export function FrameworkDetailSheet({
  frameworkId,
  open,
  onOpenChange,
  onUpdate,
  onDelete,
}: FrameworkDetailSheetProps) {
  const { data: framework, isLoading, refetch } = useFramework(frameworkId || '')
  const [isEditing, setIsEditing] = useState(false)
  const [isReqFormOpen, setIsReqFormOpen] = useState(false)
  const [editingRequirement, setEditingRequirement] = useState<FrameworkRequirement | undefined>()
  const [parentIdForNew, setParentIdForNew] = useState<string | undefined>()
  const [formData, setFormData] = useState<UpdateFramework>({})

  useEffect(() => {
    if (framework) {
      setFormData({
        name: framework.name,
        version: framework.version || '',
        description: framework.description || '',
        category: framework.category || '',
      })
    }
  }, [framework])

  useEffect(() => {
    if (!open) {
      setIsEditing(false)
      setEditingRequirement(undefined)
      setParentIdForNew(undefined)
    }
  }, [open])

  const updateMutation = useMutation(async (data: UpdateFramework) => {
    return apiClient.put<Framework>(`/frameworks/${frameworkId}`, data)
  })

  const deleteMutation = useMutation(async () => {
    return apiClient.delete(`/frameworks/${frameworkId}`)
  })

  const deleteReqMutation = useMutation(async (reqId: string) => {
    return apiClient.delete(`/frameworks/${frameworkId}/requirements/${reqId}`)
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
    if (!confirm('Are you sure you want to delete this framework? This will also delete all requirements.')) return
    try {
      await deleteMutation.mutate(undefined)
      onOpenChange(false)
      onDelete?.()
    } catch {
      // Error handled by mutation
    }
  }

  const handleEditRequirement = (req: FrameworkRequirement) => {
    setEditingRequirement(req)
    setParentIdForNew(undefined)
    setIsReqFormOpen(true)
  }

  const handleDeleteRequirement = async (req: FrameworkRequirement) => {
    if (!confirm(`Delete requirement "${req.code}"? This will also delete any sub-requirements.`)) return
    try {
      await deleteReqMutation.mutate(req.id)
      refetch()
      onUpdate?.()
    } catch {
      // Error handled by mutation
    }
  }

  const handleAddChildRequirement = (parentId: string) => {
    setEditingRequirement(undefined)
    setParentIdForNew(parentId)
    setIsReqFormOpen(true)
  }

  const handleAddRequirement = () => {
    setEditingRequirement(undefined)
    setParentIdForNew(undefined)
    setIsReqFormOpen(true)
  }

  const handleReqFormSuccess = () => {
    refetch()
    onUpdate?.()
  }

  // Get root requirements (no parent)
  const rootRequirements = framework?.requirements?.filter((r) => !r.parent_id) || []

  // Group by category for display
  const requirementsByCategory = rootRequirements.reduce((acc, req) => {
    const category = req.category || 'Uncategorized'
    if (!acc[category]) acc[category] = []
    acc[category].push(req)
    return acc
  }, {} as Record<string, FrameworkRequirement[]>)

  return (
    <>
      <Sheet open={open} onOpenChange={onOpenChange}>
        <SheetContent className="sm:max-w-2xl overflow-y-auto">
          {isLoading || !framework ? (
            <div className="flex items-center justify-center h-full">
              <Loading />
            </div>
          ) : (
            <div className="space-y-6">
              <SheetHeader>
                <div className="flex items-center gap-2">
                  <BookOpen className="h-5 w-5 text-primary" />
                  <SheetTitle>{framework.name}</SheetTitle>
                  {framework.is_system && (
                    <Badge variant="secondary">System</Badge>
                  )}
                </div>
                <SheetDescription>
                  {framework.version && `Version ${framework.version} - `}
                  {framework.requirement_count} requirements
                </SheetDescription>
              </SheetHeader>

              {/* Actions */}
              {!framework.is_system && (
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
              )}

              <Separator />

              {/* Framework Details */}
              <div className="space-y-4">
                <h3 className="font-semibold">Framework Details</h3>

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
                        <Label htmlFor="version">Version</Label>
                        <Input
                          id="version"
                          value={formData.version || ''}
                          onChange={(e) => setFormData({ ...formData, version: e.target.value })}
                        />
                      </div>
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="category">Category</Label>
                      <Input
                        id="category"
                        value={formData.category || ''}
                        onChange={(e) => setFormData({ ...formData, category: e.target.value })}
                      />
                    </div>
                    <div className="space-y-2">
                      <Label htmlFor="description">Description</Label>
                      <Textarea
                        id="description"
                        value={formData.description || ''}
                        onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                        rows={3}
                      />
                    </div>
                    {updateMutation.error && (
                      <div className="text-sm text-red-500">{updateMutation.error.message}</div>
                    )}
                  </div>
                ) : (
                  <div className="space-y-3 text-sm">
                    {framework.description && (
                      <div>
                        <Label className="text-muted-foreground text-xs">Description</Label>
                        <p>{framework.description}</p>
                      </div>
                    )}
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <Label className="text-muted-foreground text-xs">Version</Label>
                        <p>{framework.version || '-'}</p>
                      </div>
                      <div>
                        <Label className="text-muted-foreground text-xs">Category</Label>
                        <p>{framework.category || '-'}</p>
                      </div>
                    </div>
                  </div>
                )}
              </div>

              <Separator />

              {/* Requirements */}
              <div className="space-y-4">
                <div className="flex items-center justify-between">
                  <h3 className="font-semibold flex items-center gap-2">
                    <FileText className="h-4 w-4" />
                    Requirements
                    <Badge variant="secondary">{framework.requirement_count}</Badge>
                  </h3>
                  {!framework.is_system && (
                    <Button size="sm" onClick={handleAddRequirement}>
                      <Plus className="mr-2 h-4 w-4" />
                      Add
                    </Button>
                  )}
                </div>

                {framework.requirements && framework.requirements.length > 0 ? (
                  <div className="space-y-4">
                    {Object.entries(requirementsByCategory)
                      .sort(([a], [b]) => a.localeCompare(b))
                      .map(([category, reqs]) => (
                        <div key={category}>
                          <h4 className="text-sm font-medium text-muted-foreground mb-2">
                            {category}
                          </h4>
                          <div className="space-y-2">
                            {reqs
                              .sort((a, b) => a.sort_order - b.sort_order)
                              .map((req) => (
                                <RequirementItem
                                  key={req.id}
                                  requirement={req}
                                  allRequirements={framework.requirements || []}
                                  frameworkId={framework.id}
                                  onEdit={framework.is_system ? () => {} : handleEditRequirement}
                                  onDelete={framework.is_system ? () => {} : handleDeleteRequirement}
                                  onAddChild={framework.is_system ? () => {} : handleAddChildRequirement}
                                />
                              ))}
                          </div>
                        </div>
                      ))}
                  </div>
                ) : (
                  <div className="text-center py-6 border rounded-lg border-dashed">
                    <FileText className="h-8 w-8 text-muted-foreground mx-auto mb-2" />
                    <p className="text-sm text-muted-foreground mb-3">
                      No requirements defined yet
                    </p>
                    {!framework.is_system && (
                      <Button size="sm" variant="outline" onClick={handleAddRequirement}>
                        <Plus className="mr-2 h-4 w-4" />
                        Add Requirement
                      </Button>
                    )}
                  </div>
                )}
              </div>
            </div>
          )}
        </SheetContent>
      </Sheet>

      {frameworkId && (
        <RequirementForm
          open={isReqFormOpen}
          onOpenChange={setIsReqFormOpen}
          frameworkId={frameworkId}
          parentId={parentIdForNew}
          editRequirement={editingRequirement}
          onSuccess={handleReqFormSuccess}
        />
      )}
    </>
  )
}
