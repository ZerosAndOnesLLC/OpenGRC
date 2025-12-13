'use client'

import { useState, useRef } from 'react'
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
import {
  Upload,
  Search,
  Filter,
  FileText,
  File,
  Download,
  Clock,
  AlertTriangle,
  CheckCircle,
} from "lucide-react"
import { useEvidence, useEvidenceStats, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import type {
  EvidenceWithLinks,
  CreateEvidence,
  EvidenceStats,
  EvidenceType,
  EvidenceSource,
  PresignedUploadResponse,
} from '@/types'
import { formatDate, formatStatus } from '@/types'

const typeLabels: Record<string, string> = {
  document: 'Document',
  screenshot: 'Screenshot',
  log: 'Log',
  automated: 'Automated',
  config: 'Configuration',
  report: 'Report',
}

const sourceLabels: Record<string, string> = {
  manual: 'Manual',
  aws: 'AWS',
  github: 'GitHub',
  okta: 'Okta',
  azure: 'Azure',
  gcp: 'GCP',
  datadog: 'Datadog',
  other: 'Other',
}

function StatsCards({ stats }: { stats: EvidenceStats | null }) {
  if (!stats) return null

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Total Evidence</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <FileText className="h-5 w-5 text-primary" />
            <span className="text-2xl font-bold">{stats.total}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">By Type</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-1">
            {stats.by_type.slice(0, 3).map((t) => (
              <Badge key={t.evidence_type} variant="secondary" className="text-xs">
                {typeLabels[t.evidence_type] || t.evidence_type}: {t.count}
              </Badge>
            ))}
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Expiring Soon</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <Clock className="h-5 w-5 text-yellow-500" />
            <span className="text-2xl font-bold">{stats.expiring_soon}</span>
          </div>
        </CardContent>
      </Card>
      <Card>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm font-medium text-muted-foreground">Expired</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center gap-2">
            <AlertTriangle className="h-5 w-5 text-red-500" />
            <span className="text-2xl font-bold">{stats.expired}</span>
          </div>
        </CardContent>
      </Card>
    </div>
  )
}

function EvidenceForm({
  open,
  onOpenChange,
  onSuccess,
}: {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSuccess: () => void
}) {
  const fileInputRef = useRef<HTMLInputElement>(null)
  const [formData, setFormData] = useState<CreateEvidence>({
    title: '',
    description: '',
    evidence_type: 'document',
    source: 'manual',
  })
  const [selectedFile, setSelectedFile] = useState<File | null>(null)
  const [uploading, setUploading] = useState(false)

  const createMutation = useMutation(async (data: CreateEvidence) => {
    return apiClient.post<{ id: string }>('/evidence', data)
  })

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      setSelectedFile(file)
      if (!formData.title) {
        setFormData({ ...formData, title: file.name })
      }
    }
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setUploading(true)

    try {
      // Create evidence record first
      const evidence = await createMutation.mutate(formData)

      // If a file was selected, upload it
      if (selectedFile && evidence) {
        // Get presigned upload URL
        const uploadResponse = await apiClient.post<PresignedUploadResponse>(
          `/evidence/${evidence.id}/upload-url`,
          {
            filename: selectedFile.name,
            content_type: selectedFile.type || 'application/octet-stream',
          }
        )

        // Upload file directly to S3
        await fetch(uploadResponse.upload_url, {
          method: 'PUT',
          body: selectedFile,
          headers: {
            'Content-Type': selectedFile.type || 'application/octet-stream',
          },
        })

        // Confirm upload
        await apiClient.post(`/evidence/${evidence.id}/confirm-upload`, {
          file_key: uploadResponse.file_key,
          file_size: selectedFile.size,
          mime_type: selectedFile.type || 'application/octet-stream',
        })
      }

      onOpenChange(false)
      setFormData({
        title: '',
        description: '',
        evidence_type: 'document',
        source: 'manual',
      })
      setSelectedFile(null)
      onSuccess()
    } catch {
      // Error handled by mutation
    } finally {
      setUploading(false)
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <DialogTitle>Upload Evidence</DialogTitle>
          <DialogDescription>
            Upload evidence to support your compliance controls.
          </DialogDescription>
        </DialogHeader>
        <form onSubmit={handleSubmit}>
          <div className="grid gap-4 py-4">
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="title" className="text-right">
                Title *
              </Label>
              <Input
                id="title"
                value={formData.title}
                onChange={(e) => setFormData({ ...formData, title: e.target.value })}
                className="col-span-3"
                placeholder="Evidence title"
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
                placeholder="Describe this evidence..."
              />
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="type" className="text-right">
                Type
              </Label>
              <Select
                value={formData.evidence_type}
                onValueChange={(value: EvidenceType) =>
                  setFormData({ ...formData, evidence_type: value })
                }
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select type" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="document">Document</SelectItem>
                  <SelectItem value="screenshot">Screenshot</SelectItem>
                  <SelectItem value="log">Log</SelectItem>
                  <SelectItem value="automated">Automated</SelectItem>
                  <SelectItem value="config">Configuration</SelectItem>
                  <SelectItem value="report">Report</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="source" className="text-right">
                Source
              </Label>
              <Select
                value={formData.source}
                onValueChange={(value: EvidenceSource) =>
                  setFormData({ ...formData, source: value })
                }
              >
                <SelectTrigger className="col-span-3">
                  <SelectValue placeholder="Select source" />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="manual">Manual</SelectItem>
                  <SelectItem value="aws">AWS</SelectItem>
                  <SelectItem value="github">GitHub</SelectItem>
                  <SelectItem value="okta">Okta</SelectItem>
                  <SelectItem value="azure">Azure</SelectItem>
                  <SelectItem value="gcp">GCP</SelectItem>
                  <SelectItem value="datadog">Datadog</SelectItem>
                  <SelectItem value="other">Other</SelectItem>
                </SelectContent>
              </Select>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label className="text-right">File</Label>
              <div className="col-span-3">
                <input
                  ref={fileInputRef}
                  type="file"
                  onChange={handleFileSelect}
                  className="hidden"
                />
                <div
                  onClick={() => fileInputRef.current?.click()}
                  className="border-2 border-dashed rounded-lg p-6 text-center cursor-pointer hover:border-primary/50 transition-colors"
                >
                  {selectedFile ? (
                    <div className="flex items-center justify-center gap-2">
                      <File className="h-5 w-5" />
                      <span className="text-sm">{selectedFile.name}</span>
                      <span className="text-xs text-muted-foreground">
                        ({(selectedFile.size / 1024).toFixed(1)} KB)
                      </span>
                    </div>
                  ) : (
                    <div>
                      <Upload className="h-8 w-8 mx-auto text-muted-foreground mb-2" />
                      <p className="text-sm text-muted-foreground">
                        Click to select a file or drag and drop
                      </p>
                      <p className="text-xs text-muted-foreground mt-1">
                        Max 50MB for direct upload
                      </p>
                    </div>
                  )}
                </div>
              </div>
            </div>
            <div className="grid grid-cols-4 items-center gap-4">
              <Label htmlFor="valid_until" className="text-right">
                Valid Until
              </Label>
              <Input
                id="valid_until"
                type="date"
                value={formData.valid_until || ''}
                onChange={(e) => setFormData({ ...formData, valid_until: e.target.value })}
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
            <Button type="submit" disabled={uploading || createMutation.isLoading}>
              {uploading ? 'Uploading...' : 'Upload Evidence'}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}

function EvidenceRow({ evidence }: { evidence: EvidenceWithLinks }) {
  const isExpired =
    evidence.valid_until && new Date(evidence.valid_until) < new Date()
  const isExpiringSoon =
    evidence.valid_until &&
    !isExpired &&
    new Date(evidence.valid_until) < new Date(Date.now() + 30 * 24 * 60 * 60 * 1000)

  const handleDownload = async () => {
    if (!evidence.file_path) return

    try {
      const response = await apiClient.get<{ download_url: string }>(
        `/evidence/${evidence.id}/download-url`
      )
      window.open(response.download_url, '_blank')
    } catch (err) {
      console.error('Failed to get download URL', err)
    }
  }

  return (
    <tr className="border-b hover:bg-muted/25">
      <td className="p-3 text-sm">
        <div className="flex items-center gap-2">
          <FileText className="h-4 w-4 text-muted-foreground" />
          <div>
            <div className="font-medium">{evidence.title}</div>
            {evidence.description && (
              <div className="text-muted-foreground text-xs line-clamp-1">
                {evidence.description}
              </div>
            )}
          </div>
        </div>
      </td>
      <td className="p-3 text-sm">
        <Badge variant="outline">
          {typeLabels[evidence.evidence_type] || evidence.evidence_type}
        </Badge>
      </td>
      <td className="p-3 text-sm">
        {sourceLabels[evidence.source] || evidence.source}
      </td>
      <td className="p-3 text-sm">{formatDate(evidence.collected_at)}</td>
      <td className="p-3 text-sm">
        {evidence.valid_until ? (
          <div className="flex items-center gap-1">
            {isExpired ? (
              <AlertTriangle className="h-4 w-4 text-red-500" />
            ) : isExpiringSoon ? (
              <Clock className="h-4 w-4 text-yellow-500" />
            ) : (
              <CheckCircle className="h-4 w-4 text-green-500" />
            )}
            <span className={isExpired ? 'text-red-500' : isExpiringSoon ? 'text-yellow-500' : ''}>
              {formatDate(evidence.valid_until)}
            </span>
          </div>
        ) : (
          <span className="text-muted-foreground">-</span>
        )}
      </td>
      <td className="p-3 text-sm">
        <span className="text-muted-foreground">{evidence.linked_control_count} linked</span>
      </td>
      <td className="p-3 text-sm">
        {evidence.file_path && (
          <Button variant="ghost" size="sm" onClick={handleDownload}>
            <Download className="h-4 w-4" />
          </Button>
        )}
      </td>
    </tr>
  )
}

export default function EvidencePage() {
  const [search, setSearch] = useState('')
  const [typeFilter, setTypeFilter] = useState<string>('')
  const [sourceFilter, setSourceFilter] = useState<string>('')
  const [isCreateOpen, setIsCreateOpen] = useState(false)

  const query: Record<string, string | number | boolean> = {}
  if (search) query.search = search
  if (typeFilter) query.evidence_type = typeFilter
  if (sourceFilter) query.source = sourceFilter

  const { data: evidence, isLoading, error, refetch } = useEvidence(query)
  const { data: stats, refetch: refetchStats } = useEvidenceStats()

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
          <p className="text-red-500 mb-2">Failed to load evidence</p>
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
      <PageHeader title="Evidence" description="Manage evidence and artifacts for compliance">
        <Button onClick={() => setIsCreateOpen(true)}>
          <Upload className="mr-2 h-4 w-4" />
          Upload Evidence
        </Button>
      </PageHeader>

      <StatsCards stats={stats} />

      <div className="flex flex-col sm:flex-row gap-4">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search evidence..."
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
              <SelectItem value="document">Document</SelectItem>
              <SelectItem value="screenshot">Screenshot</SelectItem>
              <SelectItem value="log">Log</SelectItem>
              <SelectItem value="automated">Automated</SelectItem>
              <SelectItem value="config">Configuration</SelectItem>
              <SelectItem value="report">Report</SelectItem>
            </SelectContent>
          </Select>
          <Select value={sourceFilter} onValueChange={setSourceFilter}>
            <SelectTrigger className="w-[140px]">
              <Filter className="mr-2 h-4 w-4" />
              <SelectValue placeholder="Source" />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="">All Sources</SelectItem>
              <SelectItem value="manual">Manual</SelectItem>
              <SelectItem value="aws">AWS</SelectItem>
              <SelectItem value="github">GitHub</SelectItem>
              <SelectItem value="okta">Okta</SelectItem>
              <SelectItem value="azure">Azure</SelectItem>
              <SelectItem value="gcp">GCP</SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>

      {evidence && evidence.length > 0 ? (
        <div className="rounded-md border">
          <table className="w-full">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="p-3 text-left text-sm font-medium">Title</th>
                <th className="p-3 text-left text-sm font-medium">Type</th>
                <th className="p-3 text-left text-sm font-medium">Source</th>
                <th className="p-3 text-left text-sm font-medium">Collected</th>
                <th className="p-3 text-left text-sm font-medium">Valid Until</th>
                <th className="p-3 text-left text-sm font-medium">Controls</th>
                <th className="p-3 text-left text-sm font-medium w-16"></th>
              </tr>
            </thead>
            <tbody>
              {evidence.map((item) => (
                <EvidenceRow key={item.id} evidence={item} />
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <Card>
          <CardContent className="flex flex-col items-center justify-center py-12">
            <FileText className="h-12 w-12 text-muted-foreground mb-4" />
            <h3 className="text-lg font-medium mb-2">No evidence uploaded</h3>
            <p className="text-muted-foreground text-sm mb-4">
              Upload evidence to document your compliance controls.
            </p>
            <Button onClick={() => setIsCreateOpen(true)}>
              <Upload className="mr-2 h-4 w-4" />
              Upload Your First Evidence
            </Button>
          </CardContent>
        </Card>
      )}

      <EvidenceForm open={isCreateOpen} onOpenChange={setIsCreateOpen} onSuccess={handleSuccess} />
    </div>
  )
}
