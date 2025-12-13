"use client"

import * as React from "react"
import { useState, useEffect } from "react"
import { FileText, Search, ChevronRight, Loader2 } from "lucide-react"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Badge } from "@/components/ui/badge"
import { apiClient } from "@/lib/api-client"

interface PolicyTemplate {
  id: string
  code: string
  title: string
  description: string
  category: string
  frameworks: string[]
  review_frequency: string
  related_templates: string[]
  suggested_controls: string[]
}

interface PolicyTemplateDetail extends PolicyTemplate {
  content: string
}

interface PolicyTemplateBrowserProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  onSelectTemplate: (template: PolicyTemplateDetail) => void
}

const categoryLabels: Record<string, string> = {
  security: "Security",
  it: "IT Operations",
  compliance: "Compliance & Risk",
  privacy: "Privacy",
  hr: "Human Resources",
}

const categoryColors: Record<string, string> = {
  security: "bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200",
  it: "bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200",
  compliance: "bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200",
  privacy: "bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200",
  hr: "bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200",
}

const frameworkLabels: Record<string, string> = {
  soc2: "SOC 2",
  iso27001: "ISO 27001",
  hipaa: "HIPAA",
  "pci-dss": "PCI DSS",
  gdpr: "GDPR",
  ccpa: "CCPA",
}

export function PolicyTemplateBrowser({
  open,
  onOpenChange,
  onSelectTemplate,
}: PolicyTemplateBrowserProps) {
  const [templates, setTemplates] = useState<PolicyTemplate[]>([])
  const [loading, setLoading] = useState(true)
  const [searchQuery, setSearchQuery] = useState("")
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null)
  const [selectedFramework, setSelectedFramework] = useState<string | null>(null)
  const [selectedTemplate, setSelectedTemplate] = useState<PolicyTemplateDetail | null>(null)
  const [loadingTemplate, setLoadingTemplate] = useState(false)

  useEffect(() => {
    if (open) {
      loadTemplates()
    }
  }, [open])

  const loadTemplates = async () => {
    try {
      setLoading(true)
      const data = await apiClient.get<PolicyTemplate[]>("/policy-templates")
      setTemplates(data)
    } catch (error) {
      console.error("Failed to load templates:", error)
    } finally {
      setLoading(false)
    }
  }

  const loadTemplateDetail = async (id: string) => {
    try {
      setLoadingTemplate(true)
      const data = await apiClient.get<PolicyTemplateDetail>(`/policy-templates/${id}`)
      setSelectedTemplate(data)
    } catch (error) {
      console.error("Failed to load template:", error)
    } finally {
      setLoadingTemplate(false)
    }
  }

  const filteredTemplates = templates.filter((template) => {
    const matchesSearch =
      !searchQuery ||
      template.title.toLowerCase().includes(searchQuery.toLowerCase()) ||
      template.description.toLowerCase().includes(searchQuery.toLowerCase()) ||
      template.code.toLowerCase().includes(searchQuery.toLowerCase())

    const matchesCategory = !selectedCategory || template.category === selectedCategory
    const matchesFramework = !selectedFramework || template.frameworks.includes(selectedFramework)

    return matchesSearch && matchesCategory && matchesFramework
  })

  const groupedTemplates = filteredTemplates.reduce((acc, template) => {
    const category = template.category
    if (!acc[category]) {
      acc[category] = []
    }
    acc[category].push(template)
    return acc
  }, {} as Record<string, PolicyTemplate[]>)

  const handleUseTemplate = () => {
    if (selectedTemplate) {
      onSelectTemplate(selectedTemplate)
      onOpenChange(false)
      setSelectedTemplate(null)
    }
  }

  const resetFilters = () => {
    setSearchQuery("")
    setSelectedCategory(null)
    setSelectedFramework(null)
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-4xl max-h-[85vh] flex flex-col">
        <DialogHeader>
          <DialogTitle>Policy Templates</DialogTitle>
          <DialogDescription>
            Choose from 25 professionally-written policy templates to get started quickly.
          </DialogDescription>
        </DialogHeader>

        {selectedTemplate ? (
          // Template Preview
          <div className="flex flex-col flex-1 overflow-hidden">
            <div className="flex items-center gap-2 mb-4">
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setSelectedTemplate(null)}
              >
                Back to templates
              </Button>
              <span className="text-muted-foreground">/</span>
              <span className="font-medium">{selectedTemplate.code}</span>
            </div>

            <div className="flex-1 overflow-auto border rounded-lg p-4 bg-muted/50">
              <div className="prose prose-sm dark:prose-invert max-w-none">
                <h2>{selectedTemplate.title}</h2>
                <div className="flex gap-2 flex-wrap mb-4">
                  <Badge className={categoryColors[selectedTemplate.category]}>
                    {categoryLabels[selectedTemplate.category]}
                  </Badge>
                  {selectedTemplate.frameworks.map((fw) => (
                    <Badge key={fw} variant="outline">
                      {frameworkLabels[fw] || fw}
                    </Badge>
                  ))}
                </div>
                <pre className="whitespace-pre-wrap text-sm font-sans bg-transparent p-0 border-0">
                  {selectedTemplate.content}
                </pre>
              </div>
            </div>

            <div className="flex justify-end gap-2 mt-4 pt-4 border-t">
              <Button variant="outline" onClick={() => setSelectedTemplate(null)}>
                Cancel
              </Button>
              <Button onClick={handleUseTemplate}>
                Use This Template
              </Button>
            </div>
          </div>
        ) : (
          // Template List
          <div className="flex flex-col flex-1 overflow-hidden">
            {/* Filters */}
            <div className="flex gap-2 mb-4">
              <div className="relative flex-1">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="Search templates..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="pl-9"
                />
              </div>
            </div>

            {/* Category/Framework filters */}
            <div className="flex gap-2 mb-4 flex-wrap">
              <div className="flex gap-1 flex-wrap">
                <Button
                  variant={selectedCategory === null ? "default" : "outline"}
                  size="sm"
                  onClick={() => setSelectedCategory(null)}
                >
                  All Categories
                </Button>
                {Object.entries(categoryLabels).map(([key, label]) => (
                  <Button
                    key={key}
                    variant={selectedCategory === key ? "default" : "outline"}
                    size="sm"
                    onClick={() => setSelectedCategory(key)}
                  >
                    {label}
                  </Button>
                ))}
              </div>
            </div>

            <div className="flex gap-1 mb-4 flex-wrap">
              <Button
                variant={selectedFramework === null ? "secondary" : "ghost"}
                size="sm"
                onClick={() => setSelectedFramework(null)}
              >
                All Frameworks
              </Button>
              {Object.entries(frameworkLabels).map(([key, label]) => (
                <Button
                  key={key}
                  variant={selectedFramework === key ? "secondary" : "ghost"}
                  size="sm"
                  onClick={() => setSelectedFramework(key)}
                >
                  {label}
                </Button>
              ))}
            </div>

            {(searchQuery || selectedCategory || selectedFramework) && (
              <div className="flex items-center gap-2 mb-2">
                <span className="text-sm text-muted-foreground">
                  {filteredTemplates.length} template{filteredTemplates.length !== 1 ? "s" : ""} found
                </span>
                <Button variant="ghost" size="sm" onClick={resetFilters}>
                  Clear filters
                </Button>
              </div>
            )}

            {/* Template List */}
            <div className="flex-1 overflow-auto">
              {loading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
              ) : filteredTemplates.length === 0 ? (
                <div className="flex flex-col items-center justify-center py-8 text-center">
                  <FileText className="h-12 w-12 text-muted-foreground mb-2" />
                  <p className="text-muted-foreground">No templates match your filters</p>
                  <Button variant="ghost" size="sm" onClick={resetFilters} className="mt-2">
                    Clear filters
                  </Button>
                </div>
              ) : (
                <div className="space-y-6">
                  {Object.entries(groupedTemplates).map(([category, categoryTemplates]) => (
                    <div key={category}>
                      <h3 className="font-semibold mb-2 flex items-center gap-2">
                        <Badge className={categoryColors[category]}>
                          {categoryLabels[category]}
                        </Badge>
                        <span className="text-sm text-muted-foreground">
                          ({categoryTemplates.length})
                        </span>
                      </h3>
                      <div className="grid gap-2">
                        {categoryTemplates.map((template) => (
                          <button
                            key={template.id}
                            className="flex items-start gap-3 p-3 rounded-lg border hover:bg-muted/50 transition-colors text-left w-full"
                            onClick={() => loadTemplateDetail(template.id)}
                            disabled={loadingTemplate}
                          >
                            <FileText className="h-5 w-5 mt-0.5 text-muted-foreground shrink-0" />
                            <div className="flex-1 min-w-0">
                              <div className="flex items-center gap-2">
                                <span className="font-medium">{template.title}</span>
                                <span className="text-xs text-muted-foreground">
                                  {template.code}
                                </span>
                              </div>
                              <p className="text-sm text-muted-foreground line-clamp-1">
                                {template.description}
                              </p>
                              <div className="flex gap-1 mt-1 flex-wrap">
                                {template.frameworks.slice(0, 4).map((fw) => (
                                  <Badge key={fw} variant="outline" className="text-xs">
                                    {frameworkLabels[fw] || fw}
                                  </Badge>
                                ))}
                                {template.frameworks.length > 4 && (
                                  <Badge variant="outline" className="text-xs">
                                    +{template.frameworks.length - 4}
                                  </Badge>
                                )}
                              </div>
                            </div>
                            <ChevronRight className="h-5 w-5 text-muted-foreground shrink-0" />
                          </button>
                        ))}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </div>
          </div>
        )}
      </DialogContent>
    </Dialog>
  )
}
