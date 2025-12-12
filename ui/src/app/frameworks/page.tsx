import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { DataTable } from "@/components/data-table"
import { Plus } from "lucide-react"

export default function FrameworksPage() {
  const frameworks: never[] = []

  const columns = [
    { key: "name", header: "Framework Name" },
    { key: "version", header: "Version" },
    { key: "category", header: "Category" },
    { key: "status", header: "Status" },
  ]

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

      <DataTable
        data={frameworks}
        columns={columns}
        emptyMessage="No frameworks configured. Add a framework to get started."
      />
    </div>
  )
}
