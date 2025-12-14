"use client"

import { useState, useEffect } from "react"
import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { FileDown, FileText, Loader2 } from "lucide-react"
import { apiClient } from "@/lib/api-client"

interface ReportType {
  id: string
  name: string
  description: string
}

interface ReportTypesResponse {
  reports: ReportType[]
}

type DownloadFormat = "csv" | "pdf"

export default function ReportsPage() {
  const [reportTypes, setReportTypes] = useState<ReportType[]>([])
  const [loading, setLoading] = useState(true)
  const [downloading, setDownloading] = useState<string | null>(null)

  useEffect(() => {
    loadReportTypes()
  }, [])

  const loadReportTypes = async () => {
    try {
      setLoading(true)
      const data = await apiClient.get<ReportTypesResponse>("/reports/types")
      setReportTypes(data.reports)
    } catch (error) {
      console.error("Failed to load report types:", error)
      // Fallback to default reports if API fails
      setReportTypes([
        { id: "controls", name: "Control Health", description: "Control testing results and implementation status" },
        { id: "risks", name: "Risk Register", description: "Complete risk register with scores and mitigation status" },
        { id: "evidence", name: "Evidence Summary", description: "Evidence collection and control coverage" },
        { id: "policies", name: "Policy Acknowledgments", description: "Policy review and acknowledgment status" },
        { id: "vendors", name: "Vendor Risk", description: "Vendor risk assessment and contract status" },
        { id: "compliance-posture", name: "Compliance Posture", description: "Framework coverage and compliance status" },
      ])
    } finally {
      setLoading(false)
    }
  }

  const downloadReport = async (reportId: string, reportName: string, format: DownloadFormat) => {
    const downloadKey = `${reportId}-${format}`
    try {
      setDownloading(downloadKey)

      const token = localStorage.getItem("auth_token")
      const baseUrl = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3001"

      const response = await fetch(`${baseUrl}/api/v1/reports/${reportId}/${format}`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      })

      if (!response.ok) {
        const errorText = await response.text()
        throw new Error(errorText || "Failed to generate report")
      }

      const blob = await response.blob()
      const url = window.URL.createObjectURL(blob)
      const a = document.createElement("a")
      a.href = url

      // Get filename from Content-Disposition header or generate one
      const contentDisposition = response.headers.get("Content-Disposition")
      let filename = `opengrc-${reportId}-${new Date().toISOString().split("T")[0]}.${format}`
      if (contentDisposition) {
        const match = contentDisposition.match(/filename="(.+)"/)
        if (match) {
          filename = match[1]
        }
      }

      a.download = filename
      document.body.appendChild(a)
      a.click()
      window.URL.revokeObjectURL(url)
      document.body.removeChild(a)
    } catch (error) {
      console.error(`Failed to download ${reportName} ${format.toUpperCase()} report:`, error)
      alert(`Failed to download report: ${error instanceof Error ? error.message : "Unknown error"}`)
    } finally {
      setDownloading(null)
    }
  }

  return (
    <div className="space-y-6">
      <PageHeader
        title="Reports"
        description="Generate compliance and audit reports"
      />

      {loading ? (
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </div>
      ) : (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
          {reportTypes.map((report) => (
            <Card key={report.id}>
              <CardHeader>
                <CardTitle className="text-lg">{report.name}</CardTitle>
                <CardDescription>{report.description}</CardDescription>
              </CardHeader>
              <CardContent className="space-y-2">
                <div className="flex gap-2">
                  <Button
                    variant="outline"
                    className="flex-1"
                    onClick={() => downloadReport(report.id, report.name, "csv")}
                    disabled={downloading !== null}
                  >
                    {downloading === `${report.id}-csv` ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Generating...
                      </>
                    ) : (
                      <>
                        <FileDown className="mr-2 h-4 w-4" />
                        CSV
                      </>
                    )}
                  </Button>
                  <Button
                    variant="outline"
                    className="flex-1"
                    onClick={() => downloadReport(report.id, report.name, "pdf")}
                    disabled={downloading !== null}
                  >
                    {downloading === `${report.id}-pdf` ? (
                      <>
                        <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                        Generating...
                      </>
                    ) : (
                      <>
                        <FileText className="mr-2 h-4 w-4" />
                        PDF
                      </>
                    )}
                  </Button>
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      )}
    </div>
  )
}
