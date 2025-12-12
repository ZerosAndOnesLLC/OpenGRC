import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { DataTable } from "@/components/data-table"
import { Upload } from "lucide-react"

export default function EvidencePage() {
  const evidence: never[] = []

  const columns = [
    { key: "title", header: "Title" },
    { key: "type", header: "Type" },
    { key: "source", header: "Source" },
    { key: "collected_at", header: "Collected" },
    { key: "valid_until", header: "Valid Until" },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Evidence"
        description="Manage evidence and artifacts for compliance"
      >
        <Button>
          <Upload className="mr-2 h-4 w-4" />
          Upload Evidence
        </Button>
      </PageHeader>

      <DataTable
        data={evidence}
        columns={columns}
        emptyMessage="No evidence uploaded. Upload evidence to get started."
      />
    </div>
  )
}
