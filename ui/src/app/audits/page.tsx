import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { DataTable } from "@/components/data-table"
import { Plus } from "lucide-react"

export default function AuditsPage() {
  const audits: never[] = []

  const columns = [
    { key: "name", header: "Audit Name" },
    { key: "framework", header: "Framework" },
    { key: "type", header: "Type" },
    { key: "auditor_firm", header: "Auditor" },
    { key: "period", header: "Period" },
    { key: "status", header: "Status" },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Audits"
        description="Manage audits and external auditor engagements"
      >
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          New Audit
        </Button>
      </PageHeader>

      <DataTable
        data={audits}
        columns={columns}
        emptyMessage="No audits in progress. Create an audit to get started."
      />
    </div>
  )
}
