import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { DataTable } from "@/components/data-table"
import { Plus } from "lucide-react"

export default function VendorsPage() {
  const vendors: never[] = []

  const columns = [
    { key: "name", header: "Vendor Name" },
    { key: "category", header: "Category" },
    { key: "criticality", header: "Criticality" },
    { key: "risk_rating", header: "Risk Rating" },
    { key: "contract_end", header: "Contract End" },
    { key: "status", header: "Status" },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Vendors"
        description="Manage vendor risk assessments and documentation"
      >
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          Add Vendor
        </Button>
      </PageHeader>

      <DataTable
        data={vendors}
        columns={columns}
        emptyMessage="No vendors added. Add a vendor to get started."
      />
    </div>
  )
}
