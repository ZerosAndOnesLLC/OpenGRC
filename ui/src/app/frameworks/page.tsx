'use client'

import { useState } from 'react'
import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Loading } from "@/components/loading"
import { FrameworkDetailSheet } from "@/components/framework-detail-sheet"
import { Plus, Search, ClipboardList, ChevronRight, BookOpen } from "lucide-react"
import { useFrameworks, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type { Framework, CreateFramework } from '@/types'

interface FrameworkCardProps {
  framework: Framework
  onClick: () => void
}

function FrameworkCard({ framework, onClick }: FrameworkCardProps) {
  return (
    <Card
      className="hover:border-primary/50 transition-colors cursor-pointer"
      onClick={onClick}
    >
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="flex items-center gap-3">
            <div className="p-2 bg-primary/10 rounded-lg">
              <BookOpen className="h-5 w-5 text-primary" />
            </div>
            <div>
              <CardTitle className="text-lg">{framework.name}</CardTitle>
              {framework.version && (
                <p className="text-sm text-muted-foreground">Version {framework.version}</p>
              )}
            </div>
          </div>
          <div className="flex items-center gap-2">
            {framework.is_system && (
              <Badge variant="secondary">System</Badge>
            )}
            {framework.category && (
              <Badge variant="outline">{framework.category}</Badge>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {framework.description && (
          <p className="text-sm text-muted-foreground mb-4 line-clamp-2">
            {framework.description}
          </p>
        )}
        <div className="flex items-center justify-between">
          <span className="text-xs text-muted-foreground">
            Added {new Date(framework.created_at).toLocaleDateString()}
          </span>
          <Button variant="ghost" size="sm">
            View Requirements
            <ChevronRight className="ml-1 h-4 w-4" />
          </Button>
        </div>
      </CardContent>
    </Card>
  )
}

function CreateFrameworkForm({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}) {
  const [formData, setFormData] = useState<CreateFramework>({
    name: '',
    version: '',
    description: '',
    category: '',
    is_system: false,
  })

  const createMutation = useMutation(async (data: CreateFramework) => {
    return apiClient.post<Framework>('/frameworks', data)
  })

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    try {
      await createMutation.mutate(formData)
      onOpenChange(false)
      setFormData({
        name: '',
        version: '',
        description: '',
        category: '',
        is_system: false,
      })
      onSuccess()
    } catch {
      // Error handled by mutation
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle>Create Framework</DialogTitle>
          <DialogDescription>
            Add a new compliance framework to track requirements.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="space-y-4 py-4">
            <div className="space-y-2">
              <Label htmlFor="name">Name *</Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="e.g., ISO 27001:2022"
                required
              />
            </div>
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="version">Version</Label>
                <Input
                  id="version"
                  value={formData.version || ''}
                  onChange={(e) => setFormData({ ...formData, version: e.target.value })}
                  placeholder="e.g., 2022"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="category">Category</Label>
                <Input
                  id="category"
                  value={formData.category || ''}
                  onChange={(e) => setFormData({ ...formData, category: e.target.value })}
                  placeholder="e.g., Security"
                />
              </div>
            </div>
            <div className="space-y-2">
              <Label htmlFor="description">Description</Label>
              <Textarea
                id="description"
                value={formData.description || ''}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                placeholder="Describe the framework..."
                rows={3}
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
              {createMutation.isLoading ? 'Creating...' : 'Create Framework'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

export default function FrameworksPage() {
  const [search, setSearch] = useState('')
  const [isCreateOpen, setIsCreateOpen] = useState(false)
  const [selectedFrameworkId, setSelectedFrameworkId] = useState<string | null>(null)
  const [isDetailOpen, setIsDetailOpen] = useState(false)

  const { data: frameworks, isLoading, error, refetch } = useFrameworks()

  const filteredFrameworks = frameworks?.filter(framework =>
    search === '' ||
    framework.name.toLowerCase().includes(search.toLowerCase()) ||
    framework.description?.toLowerCase().includes(search.toLowerCase())
  )

  const handleFrameworkClick = (frameworkId: string) => {
    setSelectedFrameworkId(frameworkId)
    setIsDetailOpen(true)
  }

  const handleDetailClose = () => {
    setIsDetailOpen(false)
    setSelectedFrameworkId(null)
  }

  const handleSuccess = () => {
    refetch()
  }

  if (isLoading) {
    return <Loading />
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <p className="text-red-500 mb-2">Failed to load frameworks</p>
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
        title="Frameworks"
        description="Manage compliance frameworks and requirements"
      >
        <Button onClick={() => setIsCreateOpen(true)}>
          <Plus className="mr-2 h-4 w-4" />
          Add Framework
        </Button>
      </PageHeader>

      <div className="flex gap-4">
        <div className="relative flex-1 max-w-md">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search frameworks..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="pl-10"
          />
        </div>
      </div>

      {filteredFrameworks && filteredFrameworks.length > 0 ? (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {filteredFrameworks.map((framework) => (
            <FrameworkCard
              key={framework.id}
              framework={framework}
              onClick={() => handleFrameworkClick(framework.id)}
            />
          ))}
        </div>
      ) : (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <ClipboardList className="h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No frameworks found</h3>
            <p className="text-muted-foreground text-sm mb-4 text-center">
              {search
                ? 'No frameworks match your search.'
                : 'Add compliance frameworks to track requirements and controls.'}
            </p>
            {!search && (
              <Button onClick={() => setIsCreateOpen(true)}>
                <Plus className="mr-2 h-4 w-4" />
                Add Your First Framework
              </Button>
            )}
          </CardContent>
        </Card>
      )}

      <CreateFrameworkForm
        open={isCreateOpen}
        onOpenChange={setIsCreateOpen}
        onSuccess={handleSuccess}
      />

      <FrameworkDetailSheet
        frameworkId={selectedFrameworkId}
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
