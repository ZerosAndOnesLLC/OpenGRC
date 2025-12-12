import { cn } from "@/lib/utils"

interface Column<T> {
  key: keyof T | string
  header: string
  render?: (item: T) => React.ReactNode
  className?: string
}

interface DataTableProps<T> {
  data: T[]
  columns: Column<T>[]
  emptyMessage?: string
  className?: string
}

export function DataTable<T extends { id?: string | number }>({
  data,
  columns,
  emptyMessage = "No data available",
  className,
}: DataTableProps<T>) {
  if (data.length === 0) {
    return (
      <div className="flex h-64 items-center justify-center rounded-lg border border-dashed">
        <p className="text-sm text-muted-foreground">{emptyMessage}</p>
      </div>
    )
  }

  return (
    <div className={cn("rounded-lg border", className)}>
      <div className="overflow-x-auto">
        <table className="w-full">
          <thead className="border-b bg-muted/50">
            <tr>
              {columns.map((column, index) => (
                <th
                  key={String(column.key) || index}
                  className={cn(
                    "px-4 py-3 text-left text-sm font-medium",
                    column.className
                  )}
                >
                  {column.header}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="divide-y">
            {data.map((item, rowIndex) => (
              <tr
                key={item.id || rowIndex}
                className="transition-colors hover:bg-muted/50"
              >
                {columns.map((column, colIndex) => (
                  <td
                    key={String(column.key) || colIndex}
                    className={cn("px-4 py-3 text-sm", column.className)}
                  >
                    {column.render
                      ? column.render(item)
                      : String(item[column.key as keyof T] || "")}
                  </td>
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  )
}
