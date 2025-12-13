import { cn } from "@/lib/utils"

interface PageHeaderProps {
  title: React.ReactNode
  description?: string
  children?: React.ReactNode
  action?: React.ReactNode
  className?: string
}

export function PageHeader({
  title,
  description,
  children,
  action,
  className,
}: PageHeaderProps) {
  const actionContent = action || children
  return (
    <div className={cn("flex items-center justify-between", className)}>
      <div className="space-y-1">
        <h1 className="text-3xl font-bold tracking-tight">{title}</h1>
        {description && (
          <p className="text-muted-foreground">{description}</p>
        )}
      </div>
      {actionContent && <div className="flex items-center gap-2">{actionContent}</div>}
    </div>
  )
}
