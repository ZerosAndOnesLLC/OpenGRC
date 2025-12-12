import { PageHeader } from "@/components/page-header"
import { Button } from "@/components/ui/button"
import { DataTable } from "@/components/data-table"
import { Plus } from "lucide-react"

export default function TasksPage() {
  const tasks: never[] = []

  const columns = [
    { key: "title", header: "Task" },
    { key: "type", header: "Type" },
    { key: "assignee", header: "Assignee" },
    { key: "priority", header: "Priority" },
    { key: "due_at", header: "Due Date" },
    { key: "status", header: "Status" },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Tasks"
        description="Manage compliance tasks and workflows"
      >
        <Button>
          <Plus className="mr-2 h-4 w-4" />
          Create Task
        </Button>
      </PageHeader>

      <DataTable
        data={tasks}
        columns={columns}
        emptyMessage="No tasks assigned. Create a task to get started."
      />
    </div>
  )
}
