import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { DataTable } from "@/components/data-table"
import { Plus } from "lucide-react"

export default function AssetsPage() {
  const assets: never[] = []

  const columns = [
    { key: "name", header: "Asset Name" },
    { key: "type", header: "Type" },
    { key: "category", header: "Category" },
    { key: "classification", header: "Classification" },
    { key: "owner", header: "Owner" },
    { key: "status", header: "Status" },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Assets"
        description="Manage organizational assets and inventory"
      >
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          Add Asset
        </Button>
      </PageHeader>

      <DataTable
        data={assets}
        columns={columns}
        emptyMessage="No assets tracked. Add an asset to get started."
      />
    </div>
  )
}
