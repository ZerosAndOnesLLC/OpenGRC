import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Shield, CheckSquare, FileText, AlertTriangle } from "lucide-react"

export default function DashboardPage() {
  const stats = [
    {
      title: "Active Controls",
      value: "0",
      description: "Controls implemented",
      icon: CheckSquare,
      trend: "+0%",
    },
    {
      title: "Evidence Items",
      value: "0",
      description: "Evidence collected",
      icon: FileText,
      trend: "+0%",
    },
    {
      title: "Open Risks",
      value: "0",
      description: "Risks identified",
      icon: AlertTriangle,
      trend: "0",
    },
    {
      title: "Frameworks",
      value: "0",
      description: "Compliance frameworks",
      icon: Shield,
      trend: "0",
    },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Dashboard"
        description="Overview of your compliance posture"
      />

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => {
          const Icon = stat.icon
          return (
            <Card key={stat.title}>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  {stat.title}
                </CardTitle>
                <Icon className="h-4 w-4 text-muted-foreground" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{stat.value}</div>
                <p className="text-xs text-muted-foreground">
                  {stat.description}
                </p>
              </CardContent>
            </Card>
          )
        })}
      </div>

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-7">
        <Card className="col-span-4">
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
            <CardDescription>
              Your recent compliance activities
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex h-64 items-center justify-center rounded-lg border border-dashed">
              <p className="text-sm text-muted-foreground">
                No recent activity
              </p>
            </div>
          </CardContent>
        </Card>

        <Card className="col-span-3">
          <CardHeader>
            <CardTitle>Upcoming Tasks</CardTitle>
            <CardDescription>
              Tasks requiring your attention
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex h-64 items-center justify-center rounded-lg border border-dashed">
              <p className="text-sm text-muted-foreground">No upcoming tasks</p>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  )
}
