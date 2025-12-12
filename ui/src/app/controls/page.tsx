import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { DataTable } from "@/components/data-table"
import { Plus } from "lucide-react"

export default function ControlsPage() {
  const controls: never[] = []

  const columns = [
    { key: "code", header: "Code" },
    { key: "name", header: "Control Name" },
    { key: "type", header: "Type" },
    { key: "frequency", header: "Frequency" },
    { key: "owner", header: "Owner" },
    { key: "status", header: "Status" },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Controls"
        description="Manage security controls and testing procedures"
      >
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          Add Control
        </Button>
      </PageHeader>

      <DataTable
        data={controls}
        columns={columns}
        emptyMessage="No controls defined. Add a control to get started."
      />
    </div>
  )
}
