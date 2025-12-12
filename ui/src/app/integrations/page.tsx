import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Plus } from "lucide-react"

export default function IntegrationsPage() {
  const integrations = [
    {
      name: "AWS",
      description: "Connect to AWS for automated evidence collection",
      category: "Cloud Provider",
      status: "available",
    },
    {
      name: "GitHub",
      description: "Sync repositories and security alerts",
      category: "DevOps",
      status: "available",
    },
    {
      name: "Okta",
      description: "User directory and access management",
      category: "Identity Provider",
      status: "available",
    },
    {
      name: "Google Workspace",
      description: "User directory and security settings",
      category: "Identity Provider",
      status: "available",
    },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Integrations"
        description="Connect external systems for automated evidence collection"
      />

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {integrations.map((integration) => (
          <Card key={integration.name}>
            <CardHeader>
              <CardTitle className="text-lg">{integration.name}</CardTitle>
              <CardDescription>{integration.description}</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex items-center justify-between">
                <span className="text-xs text-muted-foreground">
                  {integration.category}
                </span>
                <Button size="sm">
                  <Plus className="mr-2 h-3 w-3" />
                  Connect
                </Button>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}
