'use client'

import { use, useState } from 'react'
import { useRouter } from 'next/navigation'
import Link from 'next/link'
import { PageHeader } from "@/components/page-header"
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { Button } from "@/components/ui/button"
import { Badge } from "@/components/ui/badge"
import { Loading } from "@/components/loading"
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs"
import {
  ArrowLeft,
  RefreshCw,
  GitBranch,
  Shield,
  Users,
  AlertTriangle,
  CheckCircle2,
  XCircle,
  AlertCircle,
  FileCheck,
  ExternalLink,
  Lock,
  Bug,
  Key,
} from "lucide-react"
import { useToast } from "@/hooks/use-toast"
import { useIntegration, useIntegrationSyncLogs, useMutation } from '@/hooks/use-api'
import { apiClient } from '@/lib/api-client'
import { formatDateTime, formatRelativeTime } from '@/types'

export default function GitHubDashboardPage({ params }: { params: Promise<{ id: string }> }) {
  const resolvedParams = use(params)
  const integrationId = resolvedParams.id
  const router = useRouter()
  const [activeTab, setActiveTab] = useState('overview')
  const { toast } = useToast()

  const { data: integrationData, isLoading: integrationLoading, refetch: refetchIntegration } = useIntegration(integrationId)
  const { data: syncLogsData, isLoading: syncLogsLoading, refetch: refetchSyncLogs } = useIntegrationSyncLogs(integrationId, 1, 10)

  const syncMutation = useMutation(async (id: string) => {
    return apiClient.post(`/integrations/${id}/sync`, { full_sync: false })
  })

  const collectEvidenceMutation = useMutation(async (id: string) => {
    return apiClient.post(`/integrations/${id}/collect-evidence`, {})
  })

  const handleSync = async () => {
    try {
      await syncMutation.mutate(integrationId)
      toast({
        title: "Sync Started",
        description: "GitHub data sync has been initiated.",
      })
      setTimeout(() => {
        refetchIntegration()
        refetchSyncLogs()
      }, 2000)
    } catch (error) {
      console.error('Sync failed:', error)
      toast({
        title: "Sync Failed",
        description: error instanceof Error ? error.message : "An error occurred.",
        variant: "destructive",
      })
    }
  }

  const handleCollectEvidence = async () => {
    try {
      const result = await collectEvidenceMutation.mutate(integrationId)
      toast({
        title: "Evidence Collected",
        description: `Created ${result.data.evidence_created} evidence records from GitHub data.`,
      })
      refetchIntegration()
      refetchSyncLogs()
    } catch (error) {
      console.error('Evidence collection failed:', error)
      toast({
        title: "Evidence Collection Failed",
        description: error instanceof Error ? error.message : "An error occurred while collecting evidence.",
        variant: "destructive",
      })
    }
  }

  if (integrationLoading) {
    return <Loading message="Loading GitHub dashboard..." />
  }

  const integration = integrationData?.data
  const syncLogs = syncLogsData?.data?.sync_logs || []

  if (!integration) {
    return (
      <div className="flex flex-col items-center justify-center py-12">
        <AlertTriangle className="h-12 w-12 text-yellow-500 mb-4" />
        <h2 className="text-xl font-semibold mb-2">Integration Not Found</h2>
        <p className="text-muted-foreground mb-4">The requested GitHub integration could not be found.</p>
        <Button asChild>
          <Link href="/integrations/">Back to Integrations</Link>
        </Button>
      </div>
    )
  }

  if (integration.integration.integration_type !== 'github') {
    router.push('/integrations/')
    return null
  }

  const accountInfo = integration.integration.account_info || {}

  return (
    <div className="space-y-6">
      <PageHeader
        title={integration.integration.name}
        description={accountInfo.organization ? `Organization: ${accountInfo.organization}` : `User: ${accountInfo.login || 'Connected'}`}
        action={
          <div className="flex gap-2">
            <Button variant="outline" asChild>
              <Link href="/integrations/">
                <ArrowLeft className="mr-2 h-4 w-4" />
                Back
              </Link>
            </Button>
            <Button
              variant="outline"
              onClick={handleCollectEvidence}
              disabled={collectEvidenceMutation.isLoading || integration.integration.status === 'syncing'}
            >
              <FileCheck className={`mr-2 h-4 w-4 ${collectEvidenceMutation.isLoading ? 'animate-pulse' : ''}`} />
              {collectEvidenceMutation.isLoading ? 'Collecting...' : 'Collect Evidence'}
            </Button>
            <Button
              onClick={handleSync}
              disabled={integration.integration.status === 'syncing' || syncMutation.isLoading}
            >
              <RefreshCw className={`mr-2 h-4 w-4 ${integration.integration.status === 'syncing' || syncMutation.isLoading ? 'animate-spin' : ''}`} />
              Sync Now
            </Button>
          </div>
        }
      />

      <div className="flex items-center gap-4">
        <Badge variant={integration.integration.status === 'active' ? 'success' : integration.integration.status === 'error' ? 'destructive' : 'secondary'}>
          {integration.integration.status}
        </Badge>
        <span className="text-sm text-muted-foreground">
          Last synced: {formatRelativeTime(integration.integration.last_sync_at)}
        </span>
        {accountInfo.html_url && (
          <a
            href={accountInfo.html_url}
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm text-primary hover:underline flex items-center gap-1"
          >
            View on GitHub
            <ExternalLink className="h-3 w-3" />
          </a>
        )}
      </div>

      {integration.integration.last_error && (
        <Card className="border-destructive">
          <CardContent className="pt-4">
            <div className="flex items-start gap-3">
              <AlertCircle className="h-5 w-5 text-destructive mt-0.5" />
              <div>
                <p className="font-medium text-destructive">Last Sync Error</p>
                <p className="text-sm text-muted-foreground">{integration.integration.last_error}</p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      <Tabs value={activeTab} onValueChange={setActiveTab}>
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="services">Services</TabsTrigger>
          <TabsTrigger value="history">Sync History</TabsTrigger>
        </TabsList>

        <TabsContent value="overview" className="mt-6">
          <div className="grid gap-6 md:grid-cols-2">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <GitBranch className="h-5 w-5" />
                  Connection Details
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <p className="text-sm text-muted-foreground">Account</p>
                    <p className="font-medium">{accountInfo.login || 'N/A'}</p>
                  </div>
                  <div>
                    <p className="text-sm text-muted-foreground">Type</p>
                    <p className="font-medium">{accountInfo.type || 'N/A'}</p>
                  </div>
                  {accountInfo.organization && (
                    <div className="col-span-2">
                      <p className="text-sm text-muted-foreground">Organization</p>
                      <p className="font-medium">{accountInfo.organization}</p>
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Shield className="h-5 w-5" />
                  Data Collection
                </CardTitle>
                <CardDescription>
                  What GitHub data is being collected for compliance evidence
                </CardDescription>
              </CardHeader>
              <CardContent className="space-y-3">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <GitBranch className="h-4 w-4 text-muted-foreground" />
                    <span>Repositories</span>
                  </div>
                  <Badge variant="success">Active</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Lock className="h-4 w-4 text-muted-foreground" />
                    <span>Branch Protection</span>
                  </div>
                  <Badge variant="success">Active</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Bug className="h-4 w-4 text-muted-foreground" />
                    <span>Dependabot Alerts</span>
                  </div>
                  <Badge variant="success">Active</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <AlertTriangle className="h-4 w-4 text-muted-foreground" />
                    <span>Code Scanning</span>
                  </div>
                  <Badge variant="success">Active</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Key className="h-4 w-4 text-muted-foreground" />
                    <span>Secret Scanning</span>
                  </div>
                  <Badge variant="success">Active</Badge>
                </div>
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <Users className="h-4 w-4 text-muted-foreground" />
                    <span>Organization Members</span>
                  </div>
                  <Badge variant={accountInfo.organization ? "success" : "secondary"}>
                    {accountInfo.organization ? "Active" : "Org Only"}
                  </Badge>
                </div>
              </CardContent>
            </Card>
          </div>

          <Card className="mt-6">
            <CardHeader>
              <CardTitle>Evidence Types Generated</CardTitle>
              <CardDescription>
                Compliance evidence automatically generated from GitHub data
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-3">
                <div className="border rounded-lg p-4">
                  <h4 className="font-medium mb-1">Repository Inventory</h4>
                  <p className="text-sm text-muted-foreground">Complete list of repositories with visibility settings</p>
                  <div className="mt-2 flex flex-wrap gap-1">
                    <Badge variant="outline" className="text-xs">CC6.1</Badge>
                    <Badge variant="outline" className="text-xs">CC6.7</Badge>
                    <Badge variant="outline" className="text-xs">A1.1</Badge>
                  </div>
                </div>
                <div className="border rounded-lg p-4">
                  <h4 className="font-medium mb-1">Branch Protection Report</h4>
                  <p className="text-sm text-muted-foreground">Protection rules compliance for default branches</p>
                  <div className="mt-2 flex flex-wrap gap-1">
                    <Badge variant="outline" className="text-xs">CC6.1</Badge>
                    <Badge variant="outline" className="text-xs">CC6.6</Badge>
                    <Badge variant="outline" className="text-xs">CC8.1</Badge>
                  </div>
                </div>
                <div className="border rounded-lg p-4">
                  <h4 className="font-medium mb-1">Vulnerability Report</h4>
                  <p className="text-sm text-muted-foreground">Dependabot alerts and vulnerability status</p>
                  <div className="mt-2 flex flex-wrap gap-1">
                    <Badge variant="outline" className="text-xs">CC3.2</Badge>
                    <Badge variant="outline" className="text-xs">CC7.1</Badge>
                    <Badge variant="outline" className="text-xs">CC7.2</Badge>
                  </div>
                </div>
                <div className="border rounded-lg p-4">
                  <h4 className="font-medium mb-1">Code Security Report</h4>
                  <p className="text-sm text-muted-foreground">Code scanning alerts and security issues</p>
                  <div className="mt-2 flex flex-wrap gap-1">
                    <Badge variant="outline" className="text-xs">CC7.1</Badge>
                    <Badge variant="outline" className="text-xs">CC7.2</Badge>
                    <Badge variant="outline" className="text-xs">CC8.1</Badge>
                  </div>
                </div>
                <div className="border rounded-lg p-4">
                  <h4 className="font-medium mb-1">Secret Detection Report</h4>
                  <p className="text-sm text-muted-foreground">Exposed secrets and credentials in code</p>
                  <div className="mt-2 flex flex-wrap gap-1">
                    <Badge variant="outline" className="text-xs">CC6.1</Badge>
                    <Badge variant="outline" className="text-xs">CC6.7</Badge>
                    <Badge variant="outline" className="text-xs">CC7.2</Badge>
                  </div>
                </div>
                <div className="border rounded-lg p-4">
                  <h4 className="font-medium mb-1">Organization Members</h4>
                  <p className="text-sm text-muted-foreground">User access and admin permissions</p>
                  <div className="mt-2 flex flex-wrap gap-1">
                    <Badge variant="outline" className="text-xs">CC6.1</Badge>
                    <Badge variant="outline" className="text-xs">CC6.2</Badge>
                    <Badge variant="outline" className="text-xs">CC6.3</Badge>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="services" className="mt-6">
          <Card>
            <CardHeader>
              <CardTitle>Enabled Services</CardTitle>
              <CardDescription>
                GitHub services being monitored for this integration
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                {accountInfo.services_enabled ? (
                  Object.entries(accountInfo.services_enabled as Record<string, boolean>).map(([service, enabled]) => (
                    <div key={service} className="flex items-center justify-between py-2 border-b last:border-0">
                      <div>
                        <p className="font-medium capitalize">{service.replace(/_/g, ' ')}</p>
                        <p className="text-sm text-muted-foreground">
                          {service === 'repositories' && 'Syncs repository metadata and settings'}
                          {service === 'branch_protection' && 'Monitors branch protection rules'}
                          {service === 'dependabot_alerts' && 'Tracks dependency vulnerabilities'}
                          {service === 'code_scanning' && 'Monitors code security issues'}
                          {service === 'secret_scanning' && 'Detects exposed secrets in code'}
                          {service === 'members' && 'Tracks organization membership'}
                        </p>
                      </div>
                      {enabled ? (
                        <CheckCircle2 className="h-5 w-5 text-green-500" />
                      ) : (
                        <XCircle className="h-5 w-5 text-muted-foreground" />
                      )}
                    </div>
                  ))
                ) : (
                  <p className="text-muted-foreground">Service configuration not available. Run a sync to update.</p>
                )}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="history" className="mt-6">
          <Card>
            <CardHeader>
              <CardTitle>Sync History</CardTitle>
              <CardDescription>
                Recent synchronization runs for this integration
              </CardDescription>
            </CardHeader>
            <CardContent>
              {syncLogsLoading ? (
                <Loading message="Loading sync history..." />
              ) : syncLogs.length === 0 ? (
                <div className="text-center py-8">
                  <RefreshCw className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                  <p className="text-muted-foreground">No sync history available yet.</p>
                  <Button onClick={handleSync} className="mt-4" disabled={syncMutation.isLoading}>
                    Run First Sync
                  </Button>
                </div>
              ) : (
                <div className="space-y-4">
                  {syncLogs.map((log: any) => (
                    <div key={log.id} className="flex items-center justify-between py-3 border-b last:border-0">
                      <div className="flex items-center gap-3">
                        {log.status === 'completed' ? (
                          <CheckCircle2 className="h-5 w-5 text-green-500" />
                        ) : log.status === 'failed' ? (
                          <XCircle className="h-5 w-5 text-red-500" />
                        ) : (
                          <RefreshCw className="h-5 w-5 text-blue-500 animate-spin" />
                        )}
                        <div>
                          <p className="font-medium capitalize">{log.sync_type || 'Full'} Sync</p>
                          <p className="text-sm text-muted-foreground">{formatDateTime(log.started_at)}</p>
                        </div>
                      </div>
                      <div className="text-right">
                        <Badge variant={log.status === 'completed' ? 'success' : log.status === 'failed' ? 'destructive' : 'secondary'}>
                          {log.status}
                        </Badge>
                        {log.records_processed > 0 && (
                          <p className="text-sm text-muted-foreground mt-1">
                            {log.records_processed} records processed
                          </p>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  )
}
