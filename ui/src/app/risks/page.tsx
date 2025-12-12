import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { DataTable } from "@/components/data-table"
import { Plus } from "lucide-react"

export default function RisksPage() {
  const risks: never[] = []

  const columns = [
    { key: "code", header: "Code" },
    { key: "title", header: "Risk Title" },
    { key: "category", header: "Category" },
    { key: "likelihood", header: "Likelihood" },
    { key: "impact", header: "Impact" },
    { key: "score", header: "Score" },
    { key: "status", header: "Status" },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Risks"
        description="Manage risk register and assessments"
      >
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          Add Risk
        </Button>
      </PageHeader>

      <DataTable
        data={risks}
        columns={columns}
        emptyMessage="No risks identified. Add a risk to get started."
      />
    </div>
  )
}
