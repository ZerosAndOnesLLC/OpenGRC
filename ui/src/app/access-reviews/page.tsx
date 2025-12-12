import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { DataTable } from "@/components/data-table"
import { Plus } from "lucide-react"

export default function AccessReviewsPage() {
  const campaigns: never[] = []

  const columns = [
    { key: "name", header: "Campaign Name" },
    { key: "integration", header: "Integration" },
    { key: "status", header: "Status" },
    { key: "started_at", header: "Started" },
    { key: "due_at", header: "Due Date" },
    { key: "progress", header: "Progress" },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Access Reviews"
        description="Conduct user access reviews and certifications"
      >
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          New Campaign
        </Button>
      </PageHeader>

      <DataTable
        data={campaigns}
        columns={columns}
        emptyMessage="No access review campaigns. Create a campaign to get started."
      />
    </div>
  )
}
