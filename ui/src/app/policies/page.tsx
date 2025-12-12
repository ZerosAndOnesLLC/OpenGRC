import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { DataTable } from "@/components/data-table"
import { Plus } from "lucide-react"

export default function PoliciesPage() {
  const policies: never[] = []

  const columns = [
    { key: "code", header: "Code" },
    { key: "title", header: "Policy Title" },
    { key: "category", header: "Category" },
    { key: "version", header: "Version" },
    { key: "status", header: "Status" },
    { key: "owner", header: "Owner" },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Policies"
        description="Manage organizational policies and procedures"
      >
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          Create Policy
        </Button>
      </PageHeader>

      <DataTable
        data={policies}
        columns={columns}
        emptyMessage="No policies created. Create a policy to get started."
      />
    </div>
  )
}
