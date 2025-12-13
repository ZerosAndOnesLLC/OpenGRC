"use client"

import { useState, useEffect } from "react"
import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Plus, FileText, Loader2, CheckCircle2 } from "lucide-react"
import { apiClient } from "@/lib/api-client"
import { PolicyTemplateBrowser } from "@/components/policy-template-browser"
import { PolicyDetailSheet } from "@/components/policy-detail-sheet"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from "@/components/ui/dialog"
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

interface Policy {
  id: string
  code: string
  title: string
  category: string | null
  content: string | null
  version: number
  status: string
  owner_id: string | null
  effective_date: string | null
  review_date: string | null
  created_at: string
  acknowledgment_count: number
  pending_acknowledgments: number
}

interface PolicyTemplateDetail {
  id: string
  code: string
  title: string
  description: string
  category: string
  frameworks: string[]
  review_frequency: string
  content: string
  related_templates: string[]
  suggested_controls: string[]
}

const statusColors: Record<string, string> = {
  draft: "bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200",
  pending_approval: "bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200",
  published: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
  archived: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
}

const categoryLabels: Record<string, string> = {
  security: "Security",
  it: "IT",
  compliance: "Compliance",
  privacy: "Privacy",
  hr: "HR",
  operations: "Operations",
  business: "Business",
  other: "Other",
}

export default function PoliciesPage() {
  const [policies, setPolicies] = useState<Policy[]>([])
  const [loading, setLoading] = useState(true)
  const [templateBrowserOpen, setTemplateBrowserOpen] = useState(false)
  const [createDialogOpen, setCreateDialogOpen] = useState(false)
  const [creating, setCreating] = useState(false)
  const [selectedPolicyId, setSelectedPolicyId] = useState<string | null>(null)
  const [isDetailOpen, setIsDetailOpen] = useState(false)

  // Form state
  const [formData, setFormData] = useState({
    code: "",
    title: "",
    category: "",
    content: "",
  })

  useEffect(() => {
    loadPolicies()
  }, [])

  const handlePolicyClick = (policyId: string) => {
    setSelectedPolicyId(policyId)
    setIsDetailOpen(true)
  }

  const handleDetailClose = () => {
    setIsDetailOpen(false)
    setSelectedPolicyId(null)
  }

  const loadPolicies = async () => {
    try {
      setLoading(true)
      const data = await apiClient.get<Policy[]>("/policies")
      setPolicies(data)
    } catch (error) {
      console.error("Failed to load policies:", error)
    } finally {
      setLoading(false)
    }
  }

  const handleSelectTemplate = (template: PolicyTemplateDetail) => {
    setFormData({
      code: template.code,
      title: template.title,
      category: template.category,
      content: template.content,
    })
    setTemplateBrowserOpen(false)
    setCreateDialogOpen(true)
  }

  const handleCreatePolicy = async () => {
    try {
      setCreating(true)
      await apiClient.post("/policies", {
        code: formData.code,
        title: formData.title,
        category: formData.category || null,
        content: formData.content || null,
      })
      setCreateDialogOpen(false)
      setFormData({ code: "", title: "", category: "", content: "" })
      loadPolicies()
    } catch (error) {
      console.error("Failed to create policy:", error)
    } finally {
      setCreating(false)
    }
  }

  const openBlankPolicy = () => {
    setFormData({ code: "", title: "", category: "", content: "" })
    setCreateDialogOpen(true)
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Policies"
        description="Manage organizational policies and procedures"
      >
        <div className="flex gap-2">
          <Button variant="outline" onClick={() => setTemplateBrowserOpen(true)}>
            <FileText className="mr-2 h-4 w-4" />
            Use Template
          </Button>
          <Button onClick={openBlankPolicy}>
            <Plus className="mr-2 h-4 w-4" />
            Create Policy
          </Button>
        </div>
      </PageHeader>

      {loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      ) : policies.length > 0 ? (
        <div className="rounded-md border">
          <table className="w-full">
            <thead>
              <tr className="border-b bg-muted/50">
                <th className="p-3 text-left text-sm font-medium">Code</th>
                <th className="p-3 text-left text-sm font-medium">Policy Title</th>
                <th className="p-3 text-left text-sm font-medium">Category</th>
                <th className="p-3 text-left text-sm font-medium">Version</th>
                <th className="p-3 text-left text-sm font-medium">Status</th>
                <th className="p-3 text-left text-sm font-medium">Acknowledgments</th>
              </tr>
            </thead>
            <tbody>
              {policies.map((policy) => (
                <tr
                  key={policy.id}
                  className="border-b hover:bg-muted/25 cursor-pointer"
                  onClick={() => handlePolicyClick(policy.id)}
                >
                  <td className="p-3 text-sm font-mono">{policy.code}</td>
                  <td className="p-3 text-sm font-medium">{policy.title}</td>
                  <td className="p-3 text-sm">
                    {policy.category ? categoryLabels[policy.category] || policy.category : '-'}
                  </td>
                  <td className="p-3 text-sm">v{policy.version}</td>
                  <td className="p-3 text-sm">
                    <Badge className={statusColors[policy.status] || statusColors.draft}>
                      {policy.status.replace("_", " ")}
                    </Badge>
                  </td>
                  <td className="p-3 text-sm">
                    <div className="flex items-center gap-1">
                      <CheckCircle2 className="h-3 w-3 text-green-500" />
                      {policy.acknowledgment_count}
                      {policy.pending_acknowledgments > 0 && (
                        <span className="text-yellow-600 ml-1">
                          ({policy.pending_acknowledgments} pending)
                        </span>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <div className="flex flex-col items-center justify-center py-12 border rounded-md border-dashed">
          <FileText className="h-12 w-12 text-muted-foreground mb-4" />
          <h3 className="text-lg font-medium mb-2">No policies created</h3>
          <p className="text-muted-foreground text-sm mb-4 text-center">
            Use a template or create a new policy to get started.
          </p>
          <div className="flex gap-2">
            <Button variant="outline" onClick={() => setTemplateBrowserOpen(true)}>
              <FileText className="mr-2 h-4 w-4" />
              Use Template
            </Button>
            <Button onClick={openBlankPolicy}>
              <Plus className="mr-2 h-4 w-4" />
              Create Policy
            </Button>
          </div>
        </div>
      )}

      {/* Template Browser Modal */}
      <PolicyTemplateBrowser
        open={templateBrowserOpen}
        onOpenChange={setTemplateBrowserOpen}
        onSelectTemplate={handleSelectTemplate}
      />

      {/* Create/Edit Policy Dialog */}
      <Dialog open={createDialogOpen} onOpenChange={setCreateDialogOpen}>
        <DialogContent className="max-w-3xl max-h-[85vh] flex flex-col">
          <DialogHeader>
            <DialogTitle>
              {formData.code ? `Create Policy from Template` : "Create New Policy"}
            </DialogTitle>
            <DialogDescription>
              {formData.code
                ? "Review and customize the template before creating your policy."
                : "Create a new policy from scratch."}
            </DialogDescription>
          </DialogHeader>

          <div className="flex-1 overflow-auto space-y-4 py-4">
            <div className="grid grid-cols-2 gap-4">
              <div className="space-y-2">
                <Label htmlFor="code">Policy Code</Label>
                <Input
                  id="code"
                  value={formData.code}
                  onChange={(e) =>
                    setFormData({ ...formData, code: e.target.value })
                  }
                  placeholder="e.g., SEC-001"
                />
              </div>
              <div className="space-y-2">
                <Label htmlFor="category">Category</Label>
                <Select
                  value={formData.category}
                  onValueChange={(value) =>
                    setFormData({ ...formData, category: value })
                  }
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select category" />
                  </SelectTrigger>
                  <SelectContent>
                    {Object.entries(categoryLabels).map(([key, label]) => (
                      <SelectItem key={key} value={key}>
                        {label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="title">Policy Title</Label>
              <Input
                id="title"
                value={formData.title}
                onChange={(e) =>
                  setFormData({ ...formData, title: e.target.value })
                }
                placeholder="e.g., Information Security Policy"
              />
            </div>

            <div className="space-y-2">
              <Label htmlFor="content">Policy Content (Markdown)</Label>
              <Textarea
                id="content"
                value={formData.content}
                onChange={(e) =>
                  setFormData({ ...formData, content: e.target.value })
                }
                placeholder="Enter policy content in Markdown format..."
                className="min-h-[300px] font-mono text-sm"
              />
            </div>
          </div>

          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => setCreateDialogOpen(false)}
              disabled={creating}
            >
              Cancel
            </Button>
            <Button
              onClick={handleCreatePolicy}
              disabled={creating || !formData.code || !formData.title}
            >
              {creating ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  Creating...
                </>
              ) : (
                "Create Policy"
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <PolicyDetailSheet
        policyId={selectedPolicyId}
        open={isDetailOpen}
        onOpenChange={(open) => {
          if (!open) handleDetailClose()
        }}
        onUpdate={loadPolicies}
        onDelete={loadPolicies}
      />
    </div>
  )
}
