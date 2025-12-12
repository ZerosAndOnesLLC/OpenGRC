import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { FileDown } from "lucide-react"

export default function ReportsPage() {
  const reports = [
    {
      title: "Compliance Posture",
      description: "Overall compliance status across all frameworks",
    },
    {
      title: "Control Health",
      description: "Control testing results and effectiveness",
    },
    {
      title: "Risk Heatmap",
      description: "Visual representation of risk landscape",
    },
    {
      title: "Evidence Summary",
      description: "Evidence collection and coverage report",
    },
    {
      title: "Policy Acknowledgments",
      description: "Policy review and acknowledgment status",
    },
    {
      title: "Vendor Risk",
      description: "Vendor risk assessment summary",
    },
  ]

  return (
    <div className="space-y-6">
      <PageHeader
        title="Reports"
        description="Generate compliance and audit reports"
      />

      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
        {reports.map((report) => (
          <Card key={report.title}>
            <CardHeader>
              <CardTitle className="text-lg">{report.title}</CardTitle>
              <CardDescription>{report.description}</CardDescription>
            </CardHeader>
            <CardContent>
              <Button variant="outline" className="w-full">
                <FileDown className="mr-2 h-4 w-4" />
                Generate Report
              </Button>
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  )
}
