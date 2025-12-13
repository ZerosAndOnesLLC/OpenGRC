'use client'

import { useState } from 'react'
import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Input } from "@/components/ui/input"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Loading } from "@/components/loading"
import { Plus, Search, ClipboardList, ChevronRight, BookOpen } from "lucide-react"
import { useFrameworks } from '@/hooks/use-api'
import type { Framework } from '@/types'

function FrameworkCard({ framework }: { framework: Framework }) {
  return (
    <Card className="hover:border-primary/50 transition-colors cursor-pointer">
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

export default function FrameworksPage() {
  const [search, setSearch] = useState('')
  const { data: frameworks, isLoading, error, refetch } = useFrameworks()

  const filteredFrameworks = frameworks?.filter(framework =>
    search === '' ||
    framework.name.toLowerCase().includes(search.toLowerCase()) ||
    framework.description?.toLowerCase().includes(search.toLowerCase())
  )

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
        <Button>
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
            <FrameworkCard key={framework.id} framework={framework} />
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
              <Button>
                <Plus className="mr-2 h-4 w-4" />
                Add Your First Framework
              </Button>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  )
}
